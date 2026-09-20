#![forbid(unsafe_code)]

//! Application contracts: actions, commands, undo history, capabilities.
//!
//! Flow: UI/Shortcut/Plugin/MCP -> [`ActionRequest`] -> [`Command`] ->
//! `DocumentMutator` -> `ChangeSet`. History owns undo/redo.

mod actions;
mod capabilities;
mod commands;
mod history;

pub use actions::{ActionId, ActionRequest};
pub use capabilities::{CapabilityInfo, CapabilityRegistry, CapabilityState};
pub use commands::{Command, CommandRequest};
pub use history::History;
