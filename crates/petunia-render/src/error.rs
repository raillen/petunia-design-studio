//! Error types for petunia-render.

use petunia_core::CoreError;
use thiserror::Error;

/// Rendering errors.
#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Core domain error: {0}")]
    Core(#[from] CoreError),

    #[error("Backend initialization failed: {0}")]
    Initialization(String),

    #[error("Frame rendering error: {0}")]
    Draw(String),
}

/// Convenience result type for rendering operations.
pub type Result<T> = std::result::Result<T, RenderError>;
