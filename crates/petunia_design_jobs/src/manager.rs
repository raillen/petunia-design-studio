//! In-memory job manager tracking background work status, progress, and cancellation.

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

/// Thread-safe in-memory manager for background jobs.
#[derive(Clone, Debug, Default)]
pub struct JobManager {
    next_id: Arc<AtomicU64>,
    jobs: Arc<Mutex<Vec<JobRecord>>>,
}

impl JobManager {
    /// Creates an empty job manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_id: Arc::new(AtomicU64::new(1)),
            jobs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Spawns and tracks a new job, returning its unique id and cancellation token.
    pub fn spawn_job(&self, label: impl Into<String>) -> (u64, CancellationToken) {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let token = CancellationToken::new();
        let record = JobRecord {
            id,
            label: label.into(),
            percent: 0,
            state: JobState::Running,
            token: token.clone(),
        };
        if let Ok(mut lock) = self.jobs.lock() {
            lock.push(record);
        }
        (id, token)
    }

    /// Updates progress for a given job.
    pub fn update_progress(&self, id: u64, percent: u8) {
        if let Ok(mut lock) = self.jobs.lock() {
            if let Some(job) = lock.iter_mut().find(|j| j.id == id) {
                if job.state == JobState::Running {
                    job.percent = percent.min(100);
                }
            }
        }
    }

    /// Marks a job as completed (100% progress).
    pub fn complete_job(&self, id: u64) {
        if let Ok(mut lock) = self.jobs.lock() {
            if let Some(job) = lock.iter_mut().find(|j| j.id == id) {
                if job.state != JobState::Running || job.token.is_cancelled() {
                    return;
                }
                job.state = JobState::Completed;
                job.percent = 100;
            }
        }
    }

    /// Marks a job as failed with a reason.
    pub fn fail_job(&self, id: u64) {
        if let Ok(mut lock) = self.jobs.lock() {
            if let Some(job) = lock.iter_mut().find(|j| j.id == id) {
                if job.state != JobState::Running && job.state != JobState::Queued {
                    return;
                }
                job.state = JobState::Failed;
            }
        }
    }

    /// Cancels a job and signals its cancellation token.
    pub fn cancel_job(&self, id: u64) {
        if let Ok(mut lock) = self.jobs.lock() {
            if let Some(job) = lock.iter_mut().find(|j| j.id == id) {
                if job.state != JobState::Running && job.state != JobState::Queued {
                    return;
                }
                job.token.cancel();
                job.state = JobState::Cancelled;
            }
        }
    }

    /// Removes completed and cancelled jobs from the list.
    pub fn clear_completed(&self) {
        if let Ok(mut lock) = self.jobs.lock() {
            lock.retain(|j| j.state == JobState::Running || j.state == JobState::Queued);
        }
    }

    /// Returns a snapshot list of all tracked jobs.
    #[must_use]
    pub fn list_jobs(&self) -> Vec<JobInfo> {
        if let Ok(lock) = self.jobs.lock() {
            lock.iter()
                .map(|j| JobInfo {
                    id: j.id,
                    label: j.label.clone(),
                    percent: j.percent,
                    state: j.state,
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}
