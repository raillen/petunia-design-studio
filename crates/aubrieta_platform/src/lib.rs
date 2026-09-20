//! Platform services, operating system abstraction traits and headless adapters (09.17).
//!
//! Provides OS-neutral boundaries for:
//! - Clipboard operations (text, vector SVG, images, document slices)
//! - Native file open/save/folder dialogs
//! - Environment queries (locales, display scale, theme preferences)
//!
//! All services provide deterministic headless implementations to guarantee headless-first
//! testability and CI compatibility without a live desktop environment.

pub mod clipboard;
pub mod dialogs;
pub mod env;
pub mod error;

pub use clipboard::{ClipboardContent, ClipboardService, HeadlessClipboard};
pub use dialogs::{FileDialogService, FileFilter, HeadlessFileDialog};
pub use env::{EnvironmentService, HeadlessEnvironment, SystemEnvironment};
pub use error::PlatformError;
