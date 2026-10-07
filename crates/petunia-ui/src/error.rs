//! Error types for petunia-ui.

use petunia_core::CoreError;
use petunia_engine::EngineError;
use petunia_render::RenderError;
use thiserror::Error;

/// UI errors.
#[derive(Debug, Error)]
pub enum UiError {
    #[error("Core domain error: {0}")]
    Core(#[from] CoreError),

    #[error("Engine error: {0}")]
    Engine(#[from] EngineError),

    #[error("Render error: {0}")]
    Render(#[from] RenderError),

    #[error("UI state error: {0}")]
    State(String),
}

/// Convenience result type for UI operations.
pub type Result<T> = std::result::Result<T, UiError>;
