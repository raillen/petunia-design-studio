//! History and undo/redo panel controller (09.3, 09.27).

use aubrieta_foundation::AubrietaError;

use crate::bridge::{AubrietaGuiBridge, HistoryPresentationModel};

/// Controller managing the History panel and undo/redo stack interactions.
#[derive(Debug, Default)]
pub struct HistoryPanelController;

impl HistoryPanelController {
    /// Creates a fresh history panel controller.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Resolves the current history presentation model.
    #[must_use]
    pub fn query_model(&self, bridge: &AubrietaGuiBridge) -> HistoryPresentationModel {
        bridge.query_history()
    }

    /// Triggers an undo operation across the bridge.
    pub fn undo(&self, bridge: &mut AubrietaGuiBridge) -> Result<bool, AubrietaError> {
        bridge.undo()
    }

    /// Triggers a redo operation across the bridge.
    pub fn redo(&self, bridge: &mut AubrietaGuiBridge) -> Result<bool, AubrietaError> {
        bridge.redo()
    }
}
