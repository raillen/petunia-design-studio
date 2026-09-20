#![forbid(unsafe_code)]

//! Aubrieta UI GPUI Crate (08.15, 09.24, 09.27).
//!
//! Architectural Invariant:
//! Domain crates (`document`, `geometry`, `color`, `text`, `raster`, `commands`,
//! `evaluation`, `render`) never import GUI toolkit types. UI receives DTOs/view-models
//! and sends `ActionRequest`/`CommandRequest` across `AubrietaGuiBridge`.

pub mod bridge;
pub mod canvas;
pub mod panels;
pub mod shell;
pub mod tools;

pub use bridge::*;
pub use canvas::*;
pub use panels::*;
pub use shell::*;
pub use tools::*;
