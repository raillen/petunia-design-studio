//! Platform service errors.

use thiserror::Error;

/// Errors arising from operating system platform operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlatformError {
    /// Clipboard operation failed or format unsupported.
    #[error("clipboard error: {0}")]
    ClipboardError(String),

    /// File dialog cancelled or I/O failure.
    #[error("file dialog error: {0}")]
    FileDialogError(String),

    /// Environment query failure.
    #[error("system environment query failed: {0}")]
    EnvironmentError(String),

    /// Feature unsupported by current platform backend.
    #[error("unsupported platform capability: {0}")]
    Unsupported(String),
}
