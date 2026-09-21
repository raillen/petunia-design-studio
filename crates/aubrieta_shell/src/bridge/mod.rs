//! UI bridge subsystem (09.27).

pub mod gui_bridge;

pub use aubrieta_application::ports::{
    ActionQueryPort, CommandPort, DocumentQueryPort, HierarchyPort, InspectionPort, PropertyPort,
    SelectionPort, SurfacePort, VariableDataPort,
};
pub use aubrieta_application::session::{DocumentSession, SelectionSession};
pub use aubrieta_application::view_models::{
    ActionStateMap, ActionStateViewModel, DataBindingViewModel, DataMergePresentationModel,
    DataSourceViewModel, DialogRequest, DocumentSummary, FieldViewModel, HistoryItemViewModel,
    HistoryPresentationModel, LayerRowViewModel, LayersPresentationModel,
    PropertiesPresentationModel, SelectionViewModel, SessionSnapshot, SurfaceRowViewModel,
};
pub use gui_bridge::AubrietaGuiBridge;
