//! Error types for petunia-core.

use thiserror::Error;

/// Core domain errors.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Object with ID {0} not found in scene")]
    ObjectNotFound(String),

    #[error("Invalid path data: {0}")]
    InvalidPath(String),

    #[error("Cycle detected in scene hierarchy at object {0}")]
    CycleDetected(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invariant violation: {0}")]
    InvariantViolation(String),
}

/// Convenience result type for core operations.
pub type Result<T> = std::result::Result<T, CoreError>;
