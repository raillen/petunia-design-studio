//! UI bridge subsystem (09.27).

pub mod gui_bridge;
pub mod ports;
pub mod session;
pub mod view_models;

pub use gui_bridge::AubrietaGuiBridge;
pub use ports::{
    ActionQueryPort, CommandPort, DocumentQueryPort, InspectionPort, PropertyPort, SelectionPort,
};
pub use session::{DocumentSession, SelectionSession};
pub use view_models::{
    ActionStateMap, ActionStateViewModel, DialogRequest, DocumentSummary, HistoryItemViewModel,
    HistoryPresentationModel, LayerRowViewModel, LayersPresentationModel,
    PropertiesPresentationModel, SelectionViewModel, SessionSnapshot, SurfaceRowViewModel,
};
