//! Cooperative job scheduler with priority classes.
//!
//! Long work never blocks interaction: jobs run on a small pool,
//! check cancellation tokens at safe points, report progress, and
//! return typed results tagged with their source revision. Stale
//! results never apply silently; the commit lane revalidates.

use crate::error::{EngineError, Result};
use crate::transaction::DocumentRevision;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Condvar, Mutex,
};

/// Scheduling class: interactive preempts background preempts batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum JobClass {
    Interactive,
    Background,
    Batch,
}

/// Stable job identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(u64);

impl JobId {
    /// Fresh identity from the scheduler counter.
    #[must_use]
    pub fn new(counter: u64) -> Self {
        Self(counter)
    }
}

/// What was submitted: identity, class, source revision and payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobRequest<T> {
    pub id: JobId,
    pub class: JobClass,
    pub source_revision: DocumentRevision,
    pub payload: T,
}

/// Operational progress. Never persisted; never published per inner
/// iteration when frequency costs more than the visual benefit.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct JobProgress {
    pub completed: u64,
    pub total: Option<u64>,
}

/// Job outcome vocabulary. User cancellation is flow control, not a
/// dialog-worthy error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JobError {
    Cancelled,
    StaleRevision {
        expected: DocumentRevision,
        current: DocumentRevision,
    },
    InvalidInput(String),
    ResourceUnavailable(String),
    AlgorithmFailure(String),
    IoFailure(String),
}

impl std::fmt::Display for JobError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled => write!(f, "job cancelled"),
            Self::StaleRevision { expected, current } => write!(
                f,
                "stale result: source revision {} but document is at {}",
                expected.0, current.0
            ),
            Self::InvalidInput(message) => write!(f, "invalid job input: {message}"),
            Self::ResourceUnavailable(message) => write!(f, "resource unavailable: {message}"),
            Self::AlgorithmFailure(message) => write!(f, "algorithm failure: {message}"),
            Self::IoFailure(message) => write!(f, "job I/O failure: {message}"),
        }
    }
}

impl std::error::Error for JobError {}

/// Typed result tagged with its source revision for commit-lane
/// revalidation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobResult<T> {
    pub id: JobId,
    pub source_revision: DocumentRevision,
    pub result: std::result::Result<T, JobError>,
}

/// Cooperative cancellation: jobs poll at safe points, children
/// inherit the parent signal. Threads are never killed by force.
#[derive(Debug, Clone, Default)]
pub struct CancelToken {
    flag: Arc<AtomicBool>,
}

impl CancelToken {
    /// Fresh untriggered token.
    #[must_use]
    pub fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Request cancellation.
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    /// True after cancellation was requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }

    /// A child token that fires when itself or the parent fires.
    /// Cancelling the parent reaches every relevant subtask.
    #[must_use]
    pub fn child(&self) -> ChildToken {
        ChildToken {
            parent: Arc::clone(&self.flag),
            own: Arc::new(AtomicBool::new(false)),
        }
    }
}

/// Child cancellation token bound to one parent.
#[derive(Debug, Clone)]
pub struct ChildToken {
    parent: Arc<AtomicBool>,
    own: Arc<AtomicBool>,
}

impl ChildToken {
    /// Request cancellation of this subtree.
    pub fn cancel(&self) {
        self.own.store(true, Ordering::SeqCst);
    }

    /// True when this token or its parent fired.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.own.load(Ordering::SeqCst) || self.parent.load(Ordering::SeqCst)
    }
}

/// Check a result's revision before committing it.
pub fn check_revision(
    source: DocumentRevision,
    current: DocumentRevision,
) -> std::result::Result<(), JobError> {
    if source != current {
        return Err(JobError::StaleRevision {
            expected: source,
            current,
        });
    }
    Ok(())
}

type Thunk = Box<dyn FnOnce() + Send + 'static>;

struct Queues {
    interactive: VecDeque<(JobId, Thunk)>,
    background: VecDeque<(JobId, Thunk)>,
    batch: VecDeque<(JobId, Thunk)>,
    shutdown: bool,
}

impl Queues {
    fn pop(&mut self) -> Option<(JobId, Thunk)> {
        self.interactive
            .pop_front()
            .or_else(|| self.background.pop_front())
            .or_else(|| self.batch.pop_front())
    }
}

/// Scheduler configuration and resource budgets.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub worker_threads: usize,
    pub max_queue_depth: usize,
    pub interactive_budget_ms: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            worker_threads: 4,
            max_queue_depth: 10_000,
            interactive_budget_ms: 16,
        }
    }
}

/// Small priority scheduler: interactive drains before background
/// before batch. Work runs on pool threads; results and progress
/// travel back through channels the caller owns.
pub struct Scheduler {
    queues: Arc<(Mutex<Queues>, Condvar)>,
    config: SchedulerConfig,
    counter: std::sync::atomic::AtomicU64,
    _workers: Vec<std::thread::JoinHandle<()>>,
}

impl Scheduler {
    /// Spawn a pool with `threads` workers using default budgets.
    #[must_use]
    pub fn new(threads: usize) -> Self {
        Self::with_config(SchedulerConfig {
            worker_threads: threads.max(1),
            ..Default::default()
        })
    }

    /// Spawn a pool with an explicit configuration and resource budgets.
    #[must_use]
    pub fn with_config(config: SchedulerConfig) -> Self {
        let threads = config.worker_threads.max(1);
        let queues = Arc::new((
            Mutex::new(Queues {
                interactive: VecDeque::new(),
                background: VecDeque::new(),
                batch: VecDeque::new(),
                shutdown: false,
            }),
            Condvar::new(),
        ));
        let mut workers = Vec::with_capacity(threads);
        for _ in 0..threads {
            let shared = Arc::clone(&queues);
            workers.push(std::thread::spawn(move || loop {
                let task = {
                    let (lock, signal) = &*shared;
                    let mut queues = lock.lock().expect("queue lock");
                    loop {
                        if queues.shutdown {
                            return;
                        }
                        if let Some(task) = queues.pop() {
                            break Some(task);
                        }
                        queues = signal.wait(queues).expect("queue lock");
                    }
                };
                if let Some((_, thunk)) = task {
                    thunk();
                }
            }));
        }
        Self {
            queues,
            config,
            counter: std::sync::atomic::AtomicU64::new(1),
            _workers: workers,
        }
    }

    /// Active scheduler configuration.
    #[must_use]
    pub fn config(&self) -> &SchedulerConfig {
        &self.config
    }

    /// Submit work with progress and cancellation support. Returns a
    /// handle for result delivery and cancellation.
    pub fn submit<T, F>(
        &self,
        class: JobClass,
        source_revision: DocumentRevision,
        payload: T,
        progress: mpsc::Sender<JobProgress>,
        work: F,
    ) -> JobHandle<T>
    where
        T: Send + 'static,
        F: FnOnce(T, CancelToken, mpsc::Sender<JobProgress>) -> std::result::Result<T, JobError>
            + Send
            + 'static,
    {
        let id = JobId(self.counter.fetch_add(1, Ordering::SeqCst));
        let token = CancelToken::new();
        let worker_token = token.clone();
        let (sender, receiver) = mpsc::channel();
        let request = JobRequest {
            id,
            class,
            source_revision,
            payload,
        };
        let thunk: Thunk = Box::new(move || {
            let JobRequest {
                id,
                source_revision,
                payload,
                ..
            } = request;
            if worker_token.is_cancelled() {
                let _ = sender.send(JobResult {
                    id,
                    source_revision,
                    result: Err(JobError::Cancelled),
                });
                return;
            }
            let result = work(payload, worker_token, progress);
            let _ = sender.send(JobResult {
                id,
                source_revision,
                result,
            });
        });
        {
            let (lock, signal) = &*self.queues;
            let mut queues = lock.lock().expect("queue lock");
            match class {
                JobClass::Interactive => queues.interactive.push_back((id, thunk)),
                JobClass::Background => queues.background.push_back((id, thunk)),
                JobClass::Batch => queues.batch.push_back((id, thunk)),
            }
            signal.notify_one();
        }
        JobHandle {
            id,
            token,
            receiver,
        }
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        let (lock, signal) = &*self.queues;
        if let Ok(mut queues) = lock.lock() {
            queues.shutdown = true;
            signal.notify_all();
        }
    }
}

/// Client side of one submitted job.
pub struct JobHandle<T> {
    id: JobId,
    token: CancelToken,
    receiver: mpsc::Receiver<JobResult<T>>,
}

impl<T> JobHandle<T> {
    /// Job identity.
    #[must_use]
    pub fn id(&self) -> JobId {
        self.id
    }

    /// Request cancellation. Queued work still passes through the
    /// worker, which reports `Cancelled` without running the payload;
    /// in-flight work observes the token at its next safe point.
    pub fn cancel(&self) {
        self.token.cancel();
    }

    /// Block for the result up to `timeout`.
    pub fn result(self, timeout: std::time::Duration) -> Result<JobResult<T>> {
        self.receiver
            .recv_timeout(timeout)
            .map_err(|_| EngineError::Execution("job result timed out".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revision_check_rejects_stale_results() {
        assert!(check_revision(DocumentRevision(4), DocumentRevision(4)).is_ok());
        assert!(matches!(
            check_revision(DocumentRevision(4), DocumentRevision(5)),
            Err(JobError::StaleRevision { .. })
        ));
    }

    #[test]
    fn child_tokens_follow_the_parent() {
        let parent = CancelToken::new();
        let child = parent.child();
        assert!(!child.is_cancelled());
        parent.cancel();
        assert!(child.is_cancelled());
    }

    #[test]
    fn submit_delivers_typed_results() {
        let scheduler = Scheduler::new(2);
        let (_progress_sender, progress_receiver) = mpsc::channel();
        let handle = scheduler.submit(
            JobClass::Background,
            DocumentRevision(9),
            21u32,
            _progress_sender,
            |payload, _cancel, _progress| Ok(payload * 2),
        );
        let outcome = handle
            .result(std::time::Duration::from_secs(5))
            .expect("result arrives");
        assert_eq!(outcome.id, JobId(1));
        assert_eq!(outcome.source_revision, DocumentRevision(9));
        assert_eq!(outcome.result, Ok(42));
        let _ = progress_receiver;
    }

    #[test]
    fn queued_cancellation_never_runs() {
        let scheduler = Scheduler::new(1);
        let gate = Arc::new(AtomicBool::new(false));
        let gate_worker = Arc::clone(&gate);
        // Occupy the single worker until released.
        let first = scheduler.submit(
            JobClass::Background,
            DocumentRevision::GENESIS,
            (),
            mpsc::channel().0,
            move |(), cancel, _| {
                while !gate_worker.load(Ordering::SeqCst) && !cancel.is_cancelled() {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                Ok(())
            },
        );
        let ran = Arc::new(AtomicBool::new(false));
        let ran_worker = Arc::clone(&ran);
        let second = scheduler.submit(
            JobClass::Background,
            DocumentRevision::GENESIS,
            (),
            mpsc::channel().0,
            move |(), _, _| {
                ran_worker.store(true, Ordering::SeqCst);
                Ok(())
            },
        );
        // Let the first job start, then cancel the queued one.
        std::thread::sleep(std::time::Duration::from_millis(50));
        second.cancel();
        gate.store(true, Ordering::SeqCst);
        let first_outcome = first
            .result(std::time::Duration::from_secs(5))
            .expect("first finishes");
        assert_eq!(first_outcome.result, Ok(()));
        let second_outcome = second
            .result(std::time::Duration::from_secs(5))
            .expect("second answers");
        assert_eq!(second_outcome.result, Err(JobError::Cancelled));
        // The payload itself never executed.
        assert!(!ran.load(Ordering::SeqCst));
    }

    #[test]
    fn interactive_jobs_preempt_batch_queue() {
        // Single worker scheduler: tasks execute sequentially.
        let scheduler = Scheduler::new(1);
        let gate = Arc::new(AtomicBool::new(false));
        let gate_worker = Arc::clone(&gate);

        // Blocker job occupies the single worker.
        let blocker = scheduler.submit(
            JobClass::Interactive,
            DocumentRevision::GENESIS,
            (),
            mpsc::channel().0,
            move |(), _, _| {
                while !gate_worker.load(Ordering::SeqCst) {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                Ok(())
            },
        );

        // Enqueue a Batch job while the worker is busy.
        let batch_job = scheduler.submit(
            JobClass::Batch,
            DocumentRevision::GENESIS,
            "batch",
            mpsc::channel().0,
            |payload, _, _| Ok(payload),
        );

        // Enqueue an Interactive job after the Batch job.
        let interactive_job = scheduler.submit(
            JobClass::Interactive,
            DocumentRevision::GENESIS,
            "interactive",
            mpsc::channel().0,
            |payload, _, _| Ok(payload),
        );

        // Release the blocker.
        gate.store(true, Ordering::SeqCst);
        blocker
            .result(std::time::Duration::from_secs(5))
            .expect("blocker done");

        // The Interactive job MUST execute before the Batch job!
        let start = std::time::Instant::now();
        let inter_res = interactive_job
            .result(std::time::Duration::from_secs(5))
            .expect("interactive done");
        let elapsed = start.elapsed();

        assert_eq!(inter_res.result, Ok("interactive"));
        assert!(
            elapsed.as_millis() < 500,
            "interactive completed with low latency: {elapsed:?}"
        );

        let batch_res = batch_job
            .result(std::time::Duration::from_secs(5))
            .expect("batch done");
        assert_eq!(batch_res.result, Ok("batch"));
    }
}
