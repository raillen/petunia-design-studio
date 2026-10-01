//! Bounded background work with cooperative cancellation and revision-tagged results.
use std::collections::{HashMap, VecDeque};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

use crate::{CancellationToken, JobManager};

/// Admission, execution and publication failures remain distinguishable.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum JobFailure {
    #[error("background queue is full")]
    QueueFull,
    #[error("background executor is shut down")]
    Shutdown,
    #[error("background job was cancelled")]
    Cancelled,
    #[error("background job failed: {0}")]
    Failed(String),
    #[error("background job panicked")]
    Panicked,
    #[error("background result belongs to revision {source_revision}, current revision is {current_revision}")]
    StaleRevision {
        source_revision: u64,
        current_revision: u64,
    },
    #[error("invalid worker/queue configuration")]
    InvalidConfiguration,
}

/// Values carry the revision of their immutable source snapshot.
#[derive(Debug)]
pub struct JobResult<T> {
    pub source_revision: u64,
    pub value: T,
}

/// Worker capabilities; no mutable document reference or GUI types.
#[derive(Clone, Debug)]
pub struct JobContext {
    id: u64,
    token: CancellationToken,
    manager: JobManager,
}
impl JobContext {
    pub fn cancellation(&self) -> &CancellationToken {
        &self.token
    }
    pub fn check_cancelled(&self) -> Result<(), JobFailure> {
        if self.token.is_cancelled() {
            Err(JobFailure::Cancelled)
        } else {
            Ok(())
        }
    }
    pub fn report_progress(&self, percent: u8) {
        self.manager.update_progress(self.id, percent);
    }
}

/// The owner polls without blocking the UI. Dropping a handle cancels its work.
pub struct JobHandle<T> {
    id: u64,
    token: CancellationToken,
    manager: JobManager,
    receiver: mpsc::Receiver<Result<JobResult<T>, JobFailure>>,
}
impl<T> JobHandle<T> {
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn cancel(&self) {
        self.token.cancel();
        self.manager.cancel_job(self.id);
    }
    /// Returns no value while pending; rejects cancellation and stale revisions
    /// before the owning application can turn a result into document commands.
    pub fn try_result(&self, current_revision: u64) -> Result<Option<T>, JobFailure> {
        if self.token.is_cancelled() {
            return Err(JobFailure::Cancelled);
        }
        match self.receiver.try_recv() {
            Ok(Ok(result)) if result.source_revision == current_revision => Ok(Some(result.value)),
            Ok(Ok(result)) => Err(JobFailure::StaleRevision {
                source_revision: result.source_revision,
                current_revision,
            }),
            Ok(Err(error)) => Err(error),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => Err(JobFailure::Shutdown),
        }
    }
}
impl<T> Drop for JobHandle<T> {
    fn drop(&mut self) {
        self.cancel();
    }
}

struct Work {
    id: u64,
    token: CancellationToken,
    run: Box<dyn FnOnce() + Send>,
}
#[derive(Default)]
struct Queue {
    pending: VecDeque<Work>,
    active: HashMap<u64, CancellationToken>,
    shutdown: bool,
}
struct Shared {
    queue: Mutex<Queue>,
    available: Condvar,
    manager: JobManager,
}

/// Fixed worker count, bounded pending work and one result slot per owner.
/// Jobs must poll cancellation between bounded chunks; shutdown cannot forcibly
/// interrupt arbitrary user code. Drop signals cancellation without blocking UI.
pub struct JobExecutor {
    shared: Arc<Shared>,
    workers: Vec<JoinHandle<()>>,
    capacity: usize,
}

impl JobExecutor {
    pub fn new(
        worker_count: usize,
        max_pending: usize,
        manager: JobManager,
    ) -> Result<Self, JobFailure> {
        if worker_count == 0 || worker_count > 64 || max_pending == 0 || max_pending > 100_000 {
            return Err(JobFailure::InvalidConfiguration);
        }
        let shared = Arc::new(Shared {
            queue: Mutex::new(Queue::default()),
            available: Condvar::new(),
            manager,
        });
        let mut executor = Self {
            shared,
            workers: Vec::new(),
            capacity: max_pending,
        };
        for index in 0..worker_count {
            let shared = executor.shared.clone();
            let worker = thread::Builder::new()
                .name(format!("ptnd-worker-{index}"))
                .spawn(move || worker(shared))
                .map_err(|e| JobFailure::Failed(format!("worker startup: {e}")))?;
            executor.workers.push(worker);
        }
        Ok(executor)
    }

    pub fn submit<T, F>(
        &self,
        label: impl Into<String>,
        source_revision: u64,
        task: F,
    ) -> Result<JobHandle<T>, JobFailure>
    where
        T: Send + 'static,
        F: FnOnce(&JobContext) -> Result<T, JobFailure> + Send + 'static,
    {
        let mut queue = self.shared.queue.lock().unwrap_or_else(|e| e.into_inner());
        if queue.shutdown {
            return Err(JobFailure::Shutdown);
        }
        if queue.pending.len() >= self.capacity {
            return Err(JobFailure::QueueFull);
        }
        let (id, token) = self.shared.manager.enqueue_job(label.into());
        let (sender, receiver) = mpsc::sync_channel(1);
        let context = JobContext {
            id,
            token: token.clone(),
            manager: self.shared.manager.clone(),
        };
        let run = Box::new(move || {
            let result = if context.check_cancelled().is_err() || !context.manager.start_job(id) {
                Err(JobFailure::Cancelled)
            } else {
                catch_unwind(AssertUnwindSafe(|| task(&context)))
                    .unwrap_or(Err(JobFailure::Panicked))
            };
            let result = if context.token.is_cancelled() {
                Err(JobFailure::Cancelled)
            } else {
                result
            };
            match &result {
                Ok(_) => context.manager.complete_job(id),
                Err(JobFailure::Cancelled) => context.manager.cancel_job(id),
                Err(_) => context.manager.fail_job(id),
            }
            // One result, one slot: publication never waits for a stalled owner.
            let _ = sender.try_send(result.map(|value| JobResult {
                source_revision,
                value,
            }));
        });
        queue.pending.push_back(Work {
            id,
            token: token.clone(),
            run,
        });
        self.shared.available.notify_one();
        Ok(JobHandle {
            id,
            token,
            manager: self.shared.manager.clone(),
            receiver,
        })
    }

    fn signal_shutdown(&self) {
        let mut queue = self.shared.queue.lock().unwrap_or_else(|e| e.into_inner());
        queue.shutdown = true;
        for work in &queue.pending {
            work.token.cancel();
            self.shared.manager.cancel_job(work.id);
        }
        for (id, token) in &queue.active {
            token.cancel();
            self.shared.manager.cancel_job(*id);
        }
        queue.pending.clear();
        self.shared.available.notify_all();
    }

    /// Explicit blocking shutdown for application teardown/tests; use cooperative
    /// tasks. Dropping the executor only signals cancellation and detaches joins.
    pub fn shutdown_and_join(mut self) {
        self.signal_shutdown();
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}
impl Drop for JobExecutor {
    fn drop(&mut self) {
        self.signal_shutdown();
    }
}

fn worker(shared: Arc<Shared>) {
    loop {
        let work = {
            let mut queue = shared.queue.lock().unwrap_or_else(|e| e.into_inner());
            loop {
                if queue.shutdown {
                    return;
                }
                if let Some(work) = queue.pending.pop_front() {
                    queue.active.insert(work.id, work.token.clone());
                    break work;
                }
                queue = shared
                    .available
                    .wait(queue)
                    .unwrap_or_else(|e| e.into_inner());
            }
        };
        (work.run)();
        let mut queue = shared.queue.lock().unwrap_or_else(|e| e.into_inner());
        queue.active.remove(&work.id);
    }
}
