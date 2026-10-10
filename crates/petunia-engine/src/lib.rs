//! # petunia-engine
//!
//! Business logic, command transactions, undo/redo stack, and snapping engine
//! for the Petunia Design Studio editor.
//!
//! Strict invariants:
//! - `#![forbid(unsafe_code)]`
//! - Completely decoupled from GUI frameworks.

#![forbid(unsafe_code)]

pub mod analysis;
pub mod brush;
pub mod color_management;
pub mod command;
pub mod compile;
pub mod error;
pub mod filter;
pub mod fragments;
pub mod generated;
pub mod geometry;
pub mod history;
pub mod io;
pub mod jobs;
pub mod journal;
pub mod mcp;
pub mod persistence;
pub mod plugins;
pub mod ptnd;
pub mod recovery;
pub mod snapping;
pub mod spatial;
pub mod text;
pub mod tiles;
pub mod trace;
pub mod transaction;

pub use command::{AddNodeCommand, Command, CommandHistory, TransformNodeCommand};
pub use compile::{
    attach_clips, compile_document, flatten_tolerance, headless_frame, ClipResolver,
    PathClipResolver,
};
pub use error::{EngineError, Result};
pub use history::{History, HistoryDescription, HistoryEntry};
pub use snapping::{snap_point, SnapConfig, SnapGuide, SnapOrientation, SnapResult};
pub use transaction::{
    apply_quiet, commit_transaction, prepare_transaction, AppliedTransaction, CommandId,
    CommitError, DocumentOp, DocumentRevision, EffectParameter, MergeKey, PreparedTransaction,
    TransactionError, TransactionRequest,
};
