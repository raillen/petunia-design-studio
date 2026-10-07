//! # petunia-engine
//!
//! Business logic, command transactions, undo/redo stack, and snapping engine
//! for the Petunia Design Studio editor.
//!
//! Strict invariants:
//! - `#![forbid(unsafe_code)]`
//! - Completely decoupled from GUI frameworks.

#![forbid(unsafe_code)]

pub mod command;
pub mod error;
pub mod snapping;

pub use command::{AddNodeCommand, Command, CommandHistory, TransformNodeCommand};
pub use error::{EngineError, Result};
pub use snapping::{snap_point, SnapConfig, SnapGuide, SnapOrientation, SnapResult};
