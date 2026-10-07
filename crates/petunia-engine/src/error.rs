//! Error types for petunia-engine.

use petunia_core::CoreError;
use thiserror::Error;

/// Engine errors.
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Core domain error: {0}")]
    Core(#[from] CoreError),

    #[error("No command available to undo")]
    NothingToUndo,

    #[error("No command available to redo")]
    NothingToRedo,

    #[error("Operation execution failed: {0}")]
    Execution(String),
}

/// Convenience result type for engine operations.
pub type Result<T> = std::result::Result<T, EngineError>;
