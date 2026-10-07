//! # petunia-render
//!
//! Rendering backend contracts, options, and reference software rasterizer
//! for the Petunia Design Studio editor.
//!
//! Strict invariants:
//! - `#![forbid(unsafe_code)]`
//! - Vendor-agnostic rendering contract (`RenderBackend`).
//! - Complete separation from core domain models.

#![forbid(unsafe_code)]

pub mod backend;
pub mod error;
pub mod software;

pub use backend::{RenderBackend, RenderOptions};
pub use error::{RenderError, Result};
pub use software::SoftwareReferenceRenderer;
