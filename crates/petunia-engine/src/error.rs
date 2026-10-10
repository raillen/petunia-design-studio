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

    #[error("History budget exceeded: {0}")]
    BudgetExceeded(String),

    /// Malformed container structure, checksum or layout.
    #[error("PTND package error: {0}")]
    Package(String),

    /// Manifest shape, schema, identity or entry-set mismatch.
    #[error("PTND manifest error: {0}")]
    Manifest(String),

    /// Required capability the build does not implement.
    #[error("PTND capability error: {0}")]
    Capability(String),

    /// Size, count, ratio or depth budget exceeded.
    #[error("PTND limit exceeded: {0}")]
    Limit(String),
}

/// Convenience result type for engine operations.
pub type Result<T> = std::result::Result<T, EngineError>;
