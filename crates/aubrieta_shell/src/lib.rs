#![forbid(unsafe_code)]

//! Aubrieta toolkit-neutral interactive shell (08.15, 09.24, 09.27).
//!
//! Architectural Invariant:
//! Domain crates (`document`, `geometry`, `color`, `text`, `raster`, `commands`,
//! `evaluation`, `render`) never import shell or toolkit types. Session
//! ownership, ports and view-models live in `aubrieta_application`; this
//! crate holds only viewport, tools, panels and shell composition.

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
