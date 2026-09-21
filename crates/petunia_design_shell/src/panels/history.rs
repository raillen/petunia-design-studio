//! History and undo/redo panel controller (09.3, 09.27).

use petunia_design_foundation::PetuniaError;

use crate::bridge::{PetuniaDesignGuiBridge, HistoryPresentationModel};

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
    pub fn query_model(&self, bridge: &PetuniaDesignGuiBridge) -> HistoryPresentationModel {
        bridge.query_history()
    }

    /// Triggers an undo operation across the bridge.
    pub fn undo(&self, bridge: &mut PetuniaDesignGuiBridge) -> Result<bool, PetuniaError> {
        bridge.undo()
    }

    /// Triggers a redo operation across the bridge.
    pub fn redo(&self, bridge: &mut PetuniaDesignGuiBridge) -> Result<bool, PetuniaError> {
        bridge.redo()
    }
}
