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

/// Scheduling class. Priority applies at cooperative task boundaries.
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
    QueueFull {
        limit: usize,
    },
    SchedulerShutdown,
    ShutdownTimeout,
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
            Self::QueueFull { limit } => write!(f, "job queue limit reached: {limit}"),
            Self::SchedulerShutdown => write!(f, "scheduler is shut down"),
            Self::ShutdownTimeout => write!(f, "workers have not reached a cancellation point"),
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

struct QueuedJob {
    id: JobId,
    token: CancelToken,
    run: Thunk,
    reject: Box<dyn FnOnce(JobError) + Send + 'static>,
}

struct Queues {
    interactive: VecDeque<QueuedJob>,
    background: VecDeque<QueuedJob>,
    batch: VecDeque<QueuedJob>,
    running: std::collections::HashMap<JobId, CancelToken>,
    shutdown: bool,
    priority_since: std::time::Instant,
    next_batch: bool,
}

impl Queues {
    fn depth(&self) -> usize {
        self.interactive.len() + self.background.len() + self.batch.len()
    }

    fn lower_priority(&mut self) -> Option<QueuedJob> {
        // Alternate the two lower classes at each fairness opportunity.
        let job = if self.next_batch {
            self.batch
                .pop_front()
                .or_else(|| self.background.pop_front())
        } else {
            self.background
                .pop_front()
                .or_else(|| self.batch.pop_front())
        };
        self.next_batch = !self.next_batch;
        job
    }

    fn pop(&mut self, reserved: bool, budget: std::time::Duration) -> Option<QueuedJob> {
        if reserved {
            return self.interactive.pop_front();
        }
        if self.priority_since.elapsed() >= budget {
            self.priority_since = std::time::Instant::now();
            if let Some(job) = self.lower_priority() {
                return Some(job);
            }
        }
        self.interactive
            .pop_front()
            .or_else(|| self.lower_priority())
    }
}

/// Scheduler configuration and resource budgets.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub worker_threads: usize,
    pub max_queue_depth: usize,
    /// Maximum priority burst before a lower-class fairness opportunity.
    /// This does not forcibly interrupt a running closure.
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

/// Bounded priority scheduler with an interactive lane and lower-class
/// fairness at task boundaries. Results and progress use caller channels.
pub struct Scheduler {
    queues: Arc<(Mutex<Queues>, Condvar)>,
    config: SchedulerConfig,
    counter: std::sync::atomic::AtomicU64,
    workers: Vec<std::thread::JoinHandle<()>>,
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
    pub fn with_config(mut config: SchedulerConfig) -> Self {
        config.worker_threads = config.worker_threads.max(1);
        let threads = config.worker_threads;
        let queues = Arc::new((
            Mutex::new(Queues {
                interactive: VecDeque::new(),
                background: VecDeque::new(),
                batch: VecDeque::new(),
                shutdown: false,
                running: std::collections::HashMap::new(),
                priority_since: std::time::Instant::now(),
                next_batch: false,
            }),
            Condvar::new(),
        ));
        let mut workers = Vec::with_capacity(threads);
        let budget = std::time::Duration::from_millis(config.interactive_budget_ms.max(1));
        for worker in 0..threads {
            let shared = Arc::clone(&queues);
            // One lane stays available under background/batch saturation.
            let reserved = threads > 1 && worker == 0;
            workers.push(std::thread::spawn(move || loop {
                let task = {
                    let (lock, signal) = &*shared;
                    let mut queues = lock.lock().unwrap_or_else(|error| error.into_inner());
                    loop {
                        if queues.shutdown {
                            return;
                        }
                        if let Some(task) = queues.pop(reserved, budget) {
                            queues.running.insert(task.id, task.token.clone());
                            break task;
                        }
                        queues = signal
                            .wait(queues)
                            .unwrap_or_else(|error| error.into_inner());
                    }
                };
                let id = task.id;
                (task.run)();
                let (lock, signal) = &*shared;
                lock.lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .running
                    .remove(&id);
                signal.notify_all();
            }));
        }
        Self {
            queues,
            config,
            counter: std::sync::atomic::AtomicU64::new(1),
            workers,
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
        let rejected_sender = sender.clone();
        let reject = Box::new(move |error| {
            let _ = rejected_sender.send(JobResult {
                id,
                source_revision,
                result: Err(error),
            });
        });
        let thunk: Thunk = Box::new(move || {
            let result = if worker_token.is_cancelled() {
                Err(JobError::Cancelled)
            } else {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    work(request.payload, worker_token.clone(), progress)
                })) {
                    Ok(result) if !worker_token.is_cancelled() => result,
                    Ok(_) => Err(JobError::Cancelled),
                    Err(_) => Err(JobError::AlgorithmFailure("job panicked".to_string())),
                }
            };
            let _ = sender.send(JobResult {
                id,
                source_revision,
                result,
            });
        });
        let queued = QueuedJob {
            id,
            token: token.clone(),
            run: thunk,
            reject,
        };
        let (lock, signal) = &*self.queues;
        let mut queues = lock.lock().unwrap_or_else(|error| error.into_inner());
        let submission_error = if queues.shutdown {
            (queued.reject)(JobError::SchedulerShutdown);
            Some(JobError::SchedulerShutdown)
        } else if queues.depth() >= self.config.max_queue_depth {
            let error = JobError::QueueFull {
                limit: self.config.max_queue_depth,
            };
            (queued.reject)(error.clone());
            Some(error)
        } else {
            match class {
                JobClass::Interactive => queues.interactive.push_back(queued),
                JobClass::Background => queues.background.push_back(queued),
                JobClass::Batch => queues.batch.push_back(queued),
            }
            signal.notify_all();
            None
        };
        JobHandle {
            id,
            token,
            receiver,
            queues: Arc::downgrade(&self.queues),
            submission_error,
        }
    }

    /// Return admission failures immediately, before the caller stores a handle.
    pub fn try_submit<T, F>(
        &self,
        class: JobClass,
        source_revision: DocumentRevision,
        payload: T,
        progress: mpsc::Sender<JobProgress>,
        work: F,
    ) -> std::result::Result<JobHandle<T>, JobError>
    where
        T: Send + 'static,
        F: FnOnce(T, CancelToken, mpsc::Sender<JobProgress>) -> std::result::Result<T, JobError>
            + Send
            + 'static,
    {
        let handle = self.submit(class, source_revision, payload, progress, work);
        if let Some(error) = handle.submission_error.clone() {
            Err(error)
        } else {
            Ok(handle)
        }
    }

    /// Cancel queued and running work, then join workers up to the deadline.
    /// A closure must poll its token: Rust cannot forcibly interrupt user code.
    /// On timeout this method can be retried after the closure returns.
    pub fn shutdown(&mut self, timeout: std::time::Duration) -> std::result::Result<(), JobError> {
        self.cancel_all();
        let deadline = std::time::Instant::now() + timeout;
        while self.workers.iter().any(|worker| !worker.is_finished()) {
            let now = std::time::Instant::now();
            if now >= deadline {
                return Err(JobError::ShutdownTimeout);
            }
            let (lock, signal) = &*self.queues;
            let queues = lock.lock().unwrap_or_else(|error| error.into_inner());
            let _ = signal.wait_timeout(
                queues,
                (deadline - now).min(std::time::Duration::from_millis(10)),
            );
        }
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
        Ok(())
    }

    fn cancel_all(&self) {
        let (lock, signal) = &*self.queues;
        let pending = {
            let mut queues = lock.lock().unwrap_or_else(|error| error.into_inner());
            queues.shutdown = true;
            for token in queues.running.values() {
                token.cancel();
            }
            let mut pending: Vec<_> = queues.interactive.drain(..).collect();
            pending.extend(queues.background.drain(..));
            pending.extend(queues.batch.drain(..));
            pending
        };
        for job in pending {
            job.token.cancel();
            (job.reject)(JobError::Cancelled);
        }
        signal.notify_all();
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        self.cancel_all();
        // Join completed workers without waiting forever on non-cooperative code.
        for worker in self.workers.drain(..) {
            if worker.is_finished() {
                let _ = worker.join();
            }
        }
    }
}

/// Client side of one submitted job.
pub struct JobHandle<T> {
    id: JobId,
    token: CancelToken,
    receiver: mpsc::Receiver<JobResult<T>>,
    queues: std::sync::Weak<(Mutex<Queues>, Condvar)>,
    submission_error: Option<JobError>,
}

impl<T> JobHandle<T> {
    /// Job identity.
    #[must_use]
    pub fn id(&self) -> JobId {
        self.id
    }

    /// Cancel queued work immediately and release its queue capacity.
    /// In-flight work observes the token at its next safe point.
    pub fn cancel(&self) {
        self.token.cancel();
        let Some(shared) = self.queues.upgrade() else {
            return;
        };
        let (lock, signal) = &*shared;
        let removed = {
            let mut queues = lock.lock().unwrap_or_else(|error| error.into_inner());
            let mut removed = queues
                .interactive
                .iter()
                .position(|job| job.id == self.id)
                .and_then(|at| queues.interactive.remove(at));
            if removed.is_none() {
                removed = queues
                    .background
                    .iter()
                    .position(|job| job.id == self.id)
                    .and_then(|at| queues.background.remove(at));
            }
            if removed.is_none() {
                removed = queues
                    .batch
                    .iter()
                    .position(|job| job.id == self.id)
                    .and_then(|at| queues.batch.remove(at));
            }
            removed
        };
        if let Some(job) = removed {
            (job.reject)(JobError::Cancelled);
        }
        signal.notify_all();
    }

    /// Poll result delivery without blocking an event loop.
    pub fn try_result(&self) -> Result<Option<JobResult<T>>> {
        match self.receiver.try_recv() {
            Ok(result) => Ok(Some(result)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => Err(EngineError::Execution(
                "job result channel disconnected".to_string(),
            )),
        }
    }

    /// Block for the result up to `timeout`.
    pub fn result(self, timeout: std::time::Duration) -> Result<JobResult<T>> {
        self.receiver.recv_timeout(timeout).map_err(|error| {
            EngineError::Execution(match error {
                mpsc::RecvTimeoutError::Timeout => "job result timed out".to_string(),
                mpsc::RecvTimeoutError::Disconnected => {
                    "job result channel disconnected".to_string()
                }
            })
        })
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
