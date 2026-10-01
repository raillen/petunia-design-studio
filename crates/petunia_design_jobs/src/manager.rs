//! In-memory job manager tracking background work status, progress, and cancellation.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::CancellationToken;

/// State of an individual background task.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    /// Scheduled / pending execution.
    Queued,
    /// Actively processing.
    Running,
    /// Successfully finished.
    Completed,
    /// Cancelled by user or system.
    Cancelled,
    /// Terminated with an error.
    Failed,
}

/// Snapshot description of a job for UI and diagnostic rendering.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobInfo {
    /// Unique identifier for the job.
    pub id: u64,
    /// Human-readable label for the work being performed.
    pub label: String,
    /// Completion percentage (0..=100).
    pub percent: u8,
    /// Current lifecycle state.
    pub state: JobState,
}

#[derive(Clone, Debug)]
struct JobRecord {
    id: u64,
    label: String,
    percent: u8,
    state: JobState,
    token: CancellationToken,
}

const MAX_TERMINAL_HISTORY: usize = 256;
#[derive(Debug, Default)]
struct Records {
    jobs: HashMap<u64, JobRecord>,
    terminal: VecDeque<u64>,
}
impl Records {
    fn finish(&mut self, id: u64, state: JobState) {
        let Some(job) = self.jobs.get_mut(&id) else {
            return;
        };
        if !matches!(job.state, JobState::Running | JobState::Queued) {
            return;
        }
        if state == JobState::Completed && job.state != JobState::Running {
            return;
        }
        if state == JobState::Cancelled {
            job.token.cancel();
        }
        job.state = if job.token.is_cancelled() {
            JobState::Cancelled
        } else {
            state
        };
        if job.state == JobState::Completed {
            job.percent = 100;
        }
        self.terminal.push_back(id);
        while self.terminal.len() > MAX_TERMINAL_HISTORY {
            if let Some(old) = self.terminal.pop_front() {
                self.jobs.remove(&old);
            }
        }
    }
}

/// Thread-safe in-memory manager; keeps all active jobs and at most 256 terminal records.
#[derive(Clone, Debug)]
pub struct JobManager {
    next_id: Arc<AtomicU64>,
    jobs: Arc<Mutex<Records>>,
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}

impl JobManager {
    /// Creates an empty job manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_id: Arc::new(AtomicU64::new(1)),
            jobs: Arc::new(Mutex::new(Records::default())),
        }
    }

    /// Spawns and tracks a new job, returning its unique id and cancellation token.
    pub fn spawn_job(&self, label: impl Into<String>) -> (u64, CancellationToken) {
        self.register(label.into(), JobState::Running)
    }

    pub(crate) fn enqueue_job(&self, label: String) -> (u64, CancellationToken) {
        self.register(label, JobState::Queued)
    }

    fn register(&self, mut label: String, state: JobState) -> (u64, CancellationToken) {
        if label.len() > 1024 {
            let mut end = 1024;
            while !label.is_char_boundary(end) {
                end -= 1;
            }
            label.truncate(end);
        }
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let token = CancellationToken::new();
        let record = JobRecord {
            id,
            label,
            percent: 0,
            state,
            token: token.clone(),
        };
        if let Ok(mut lock) = self.jobs.lock() {
            lock.jobs.insert(id, record);
        }
        (id, token)
    }

    pub(crate) fn start_job(&self, id: u64) -> bool {
        if let Ok(mut jobs) = self.jobs.lock() {
            if let Some(job) = jobs.jobs.get_mut(&id) {
                if job.state == JobState::Queued && !job.token.is_cancelled() {
                    job.state = JobState::Running;
                    return true;
                }
            }
        }
        false
    }

    /// Updates progress for a given job.
    pub fn update_progress(&self, id: u64, percent: u8) {
        if let Ok(mut lock) = self.jobs.lock() {
            if let Some(job) = lock.jobs.get_mut(&id) {
                if job.state == JobState::Running {
                    job.percent = percent.min(100);
                }
            }
        }
    }

    /// Marks a running job completed; cancellation wins racing completion.
    pub fn complete_job(&self, id: u64) {
        if let Ok(mut lock) = self.jobs.lock() {
            lock.finish(id, JobState::Completed);
        }
    }
    /// Marks an active job failed; terminal states cannot be rewritten.
    pub fn fail_job(&self, id: u64) {
        if let Ok(mut lock) = self.jobs.lock() {
            lock.finish(id, JobState::Failed);
        }
    }
    /// Cancels an active job and signals its token.
    pub fn cancel_job(&self, id: u64) {
        if let Ok(mut lock) = self.jobs.lock() {
            lock.finish(id, JobState::Cancelled);
        }
    }

    /// Removes completed and cancelled jobs from the list.
    pub fn clear_completed(&self) {
        if let Ok(mut lock) = self.jobs.lock() {
            lock.jobs
                .retain(|_, j| j.state == JobState::Running || j.state == JobState::Queued);
            lock.terminal.clear();
        }
    }

    /// Returns a snapshot list of all tracked jobs.
    #[must_use]
    pub fn list_jobs(&self) -> Vec<JobInfo> {
        if let Ok(lock) = self.jobs.lock() {
            let mut result: Vec<_> = lock
                .jobs
                .values()
                .map(|j| JobInfo {
                    id: j.id,
                    label: j.label.clone(),
                    percent: j.percent,
                    state: j.state,
                })
                .collect();
            result.sort_unstable_by_key(|job| job.id);
            result
        } else {
            Vec::new()
        }
    }
}
