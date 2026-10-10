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

    #[error("Unknown page: {0}")]
    UnknownPage(String),

    #[error("Duplicate object ID: {0}")]
    DuplicateObject(String),

    #[error("Not a container: {0}")]
    NotAContainer(String),

    #[error("Invalid parent: {0}")]
    InvalidParent(String),

    #[error("Non-invertible transform: {0}")]
    NonInvertibleTransform(String),

    #[error("Dangling reference: {0}")]
    DanglingReference(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invariant violation: {0}")]
    InvariantViolation(String),

    #[error("Unsupported color transform: {0}")]
    UnsupportedTransform(String),
}

/// Convenience result type for core operations.
pub type Result<T> = std::result::Result<T, CoreError>;
