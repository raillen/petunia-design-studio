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
pub mod export_service;
pub mod geo_cache;
pub mod hierarchy_service;
mod history;
pub mod interaction;
pub mod menus;
pub mod ports;
pub mod selection_mask;
pub mod session;
pub mod spatial_index;
pub mod surface_service;
pub mod surfaces;
pub mod tools;
mod transaction;
pub mod view_camera;
pub mod view_models;

pub use actions::{ActionId, ActionRequest};
pub use capabilities::{CapabilityInfo, CapabilityRegistry, CapabilityState};
pub use commands::{Command, CommandRequest, CommandResult};
pub use creation::{create_artboard_commands, create_shape_commands};
pub use geo_cache::GeoCache;
pub use history::History;
pub use interaction::{NormalizedPointerEvent, PointerButton, PointerPhase, SemanticModifiers};
pub use ports::{
    ActionQueryPort, CommandPort, DocumentQueryPort, HierarchyPort, InspectionPort, PropertyPort,
    SelectionPort, SurfacePort, VariableDataPort,
};
pub use selection_mask::{RasterSelection, SelectionMode, SelectionShape};
pub use session::{DocumentSession, SelectionSession};
pub use tools::ToolKind;
pub use transaction::Transaction;
pub use view_models::{
    ActionStateMap, ActionStateViewModel, DataBindingViewModel, DataMergePresentationModel,
    DataSourceViewModel, DialogRequest, DocumentSummary, FieldViewModel, HistoryItemViewModel,
    HistoryPresentationModel, LayerRowViewModel, LayersPresentationModel,
    PropertiesPresentationModel, SelectionViewModel, SessionSnapshot, SurfaceRowViewModel,
};
