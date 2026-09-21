#![forbid(unsafe_code)]

//! Application contracts: actions, commands, undo history, capabilities.
//!
//! Flow: UI/Shortcut/Plugin/MCP -> [`ActionRequest`] -> [`Command`] ->
//! `DocumentMutator` -> `ChangeSet`. History owns undo/redo.

mod actions;
mod capabilities;
mod commands;
mod history;
mod interaction;
mod ports;
mod session;
mod tools;
mod transaction;
mod view_models;

pub use actions::{ActionId, ActionRequest};
pub use capabilities::{CapabilityInfo, CapabilityRegistry, CapabilityState};
pub use commands::{Command, CommandRequest, CommandResult};
pub use history::History;
pub use interaction::{NormalizedPointerEvent, PointerButton, PointerPhase, SemanticModifiers};
pub use ports::{
    ActionQueryPort, CommandPort, DocumentQueryPort, HierarchyPort, InspectionPort, PropertyPort,
    SelectionPort, SurfacePort, VariableDataPort,
};
pub use session::{DocumentSession, SelectionSession};
pub use tools::ToolKind;
pub use transaction::Transaction;
pub use view_models::{
    ActionStateMap, ActionStateViewModel, DataBindingViewModel, DataMergePresentationModel,
    DataSourceViewModel, DialogRequest, DocumentSummary, FieldViewModel, HistoryItemViewModel,
    HistoryPresentationModel, LayerRowViewModel, LayersPresentationModel,
    PropertiesPresentationModel, SelectionViewModel, SessionSnapshot, SurfaceRowViewModel,
};
