//! UI panels and presentation controllers (10.4, 10.5).

pub mod history;
pub mod layers;
pub mod properties;

pub use history::HistoryPanelController;
pub use layers::LayersPanelController;
pub use properties::PropertiesPanelController;
