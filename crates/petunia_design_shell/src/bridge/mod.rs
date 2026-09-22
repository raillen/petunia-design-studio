//! UI bridge subsystem (09.27).

pub mod gui_bridge;

pub use gui_bridge::PetuniaDesignGuiBridge;
pub use petunia_design_application::ports::{
    ActionQueryPort, CommandPort, DocumentQueryPort, HierarchyPort, InspectionPort, PropertyPort,
    SelectionPort, SurfacePort, VariableDataPort,
};
pub use petunia_design_application::session::{DocumentSession, SelectionSession};
pub use petunia_design_application::view_models::{
    ActionStateMap, ActionStateViewModel, DataBindingViewModel, DataMergePresentationModel,
    DataSourceViewModel, DialogRequest, DocumentSummary, FieldViewModel, HistoryItemViewModel,
    HistoryPresentationModel, LayerRowViewModel, LayersPresentationModel,
    PropertiesPresentationModel, SelectionViewModel, SessionSnapshot, SurfaceRowViewModel,
};
