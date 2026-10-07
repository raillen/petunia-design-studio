//! # petunia-ui
//!
//! Desktop application shell, user interaction mapping, tool management,
//! and canvas coordination for Petunia Design Studio.
//!
//! Strict invariants:
//! - `#![forbid(unsafe_code)]`
//! - Dispatches intentions to `petunia-engine`.
//! - Keeps UI decoupled from core document data structures.

#![forbid(unsafe_code)]

pub mod app;
pub mod error;
pub mod input;

pub use app::StudioSession;
pub use error::{Result, UiError};
pub use input::{PointerEvent, ToolKind, UserAction};
