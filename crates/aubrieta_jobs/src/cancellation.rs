//! Cooperative cancellation: jobs poll the token, owners cancel explicitly.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Cheaply cloneable cancellation flag shared between owner and job.
#[derive(Clone, Debug, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Creates a live (non-cancelled) token.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Signals cancellation. Idempotent.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// True after [`Self::cancel`] was called.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// Deterministic progress report (0..=100) with a static label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobProgress {
    /// Completion percent.
    pub percent: u8,
    /// What the job is doing. No user artwork or secrets here.
    pub label: String,
}

impl JobProgress {
    /// Creates a progress report, clamping percent to 100.
    #[must_use]
    pub fn new(percent: u8, label: impl Into<String>) -> Self {
        Self {
            percent: percent.min(100),
            label: label.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancel_is_visible_to_clones() {
        let token = CancellationToken::new();
        let worker = token.clone();
        assert!(!worker.is_cancelled());
        token.cancel();
        assert!(worker.is_cancelled());
    }
}
