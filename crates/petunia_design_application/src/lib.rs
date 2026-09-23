#![forbid(unsafe_code)]

//! Application contracts: actions, commands, undo history, capabilities.
//!
//! Flow: UI/Shortcut/Plugin/MCP -> [`ActionRequest`] -> [`Command`] ->
//! `DocumentMutator` -> `ChangeSet`. History owns undo/redo.

mod actions;
pub mod appearance_service;
pub mod boolean_service;
mod capabilities;
mod commands;
mod creation;
pub mod data_merge;
mod history;
pub mod hierarchy_service;
pub mod interaction;
pub mod ports;
pub mod selection_mask;
pub mod session;
pub mod surfaces;
pub mod view_camera;
pub mod surface_service;
pub mod tools;
mod transaction;
pub mod view_models;

pub use actions::{ActionId, ActionRequest};
pub use capabilities::{CapabilityInfo, CapabilityRegistry, CapabilityState};
pub use commands::{Command, CommandRequest, CommandResult};
pub use creation::{create_artboard_commands, create_shape_commands};
pub use history::History;
pub use interaction::{NormalizedPointerEvent, PointerButton, PointerPhase, SemanticModifiers};
pub use ports::{
    ActionQueryPort, CommandPort, DocumentQueryPort, HierarchyPort, InspectionPort, PropertyPort,
    SelectionPort, SurfacePort, VariableDataPort,
};
pub use session::{DocumentSession, SelectionSession};
pub use selection_mask::{RasterSelection, SelectionMode, SelectionShape};
pub use tools::ToolKind;
pub use transaction::Transaction;
pub use view_models::{
    ActionStateMap, ActionStateViewModel, DataBindingViewModel, DataMergePresentationModel,
    DataSourceViewModel, DialogRequest, DocumentSummary, FieldViewModel, HistoryItemViewModel,
    HistoryPresentationModel, LayerRowViewModel, LayersPresentationModel,
    PropertiesPresentationModel, SelectionViewModel, SessionSnapshot, SurfaceRowViewModel,
};
