//! Compile-time failures for snapshot evaluation.

use thiserror::Error;

/// Failures while compiling a document into a snapshot. Failure
/// never mutates the document; degradable primitives carry a
/// [`CompileWarning`] instead of failing.
#[derive(Debug, Error)]
pub enum CompileError {
    /// A referenced resource has no resolvable content.
    #[error("Missing resource: {0}")]
    MissingResource(String),

    /// An effect the compiler cannot describe to render.
    #[error("Unsupported effect: {0}")]
    UnsupportedEffect(String),

    /// Evaluation itself failed.
    #[error("Evaluation failure: {0}")]
    EvaluationFailure(String),

    /// Text could not be laid out or shaped.
    #[error("Text layout failure: {0}")]
    TextLayoutFailure(String),

    /// A complexity guard tripped before rendering.
    #[error("Complexity guard exceeded: {0}")]
    ComplexityGuardExceeded(String),

    /// Compilation was cancelled.
    #[error("Compilation cancelled")]
    Cancelled,
}

/// Convenience result type for snapshot compilation.
pub type Result<T> = std::result::Result<T, CompileError>;

/// Non-fatal fidelity note attached to degraded primitives.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CompileWarning {
    /// Authorial source of the degraded primitive.
    pub source: petunia_core::ObjectId,
    /// What was degraded and why.
    pub message: String,
}
