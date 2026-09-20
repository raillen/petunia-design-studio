//! Layers panel controller and presentation interaction (10.5).

use aubrieta_application::{ActionId, ActionRequest, Command, CommandRequest};
use aubrieta_document::ChangeSet;
use aubrieta_foundation::{AubrietaError, ObjectId, SurfaceId};

use crate::bridge::{AubrietaGuiBridge, LayersPresentationModel};

/// Controller managing the Layers Panel interactions over the unified object tree (10.5).
#[derive(Debug, Default)]
pub struct LayersPanelController;

impl LayersPanelController {
    /// Creates a fresh layers panel controller.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Resolves the current layers presentation model.
    #[must_use]
    pub fn query_model(&self, bridge: &AubrietaGuiBridge) -> LayersPresentationModel {
        bridge.query_layers()
    }

    /// Toggles an object's visibility flag in the canonical document.
    pub fn toggle_visibility(
        &self,
        bridge: &mut AubrietaGuiBridge,
        id: ObjectId,
    ) -> Result<ChangeSet, AubrietaError> {
        let current = bridge
            .session()
            .and_then(|s| s.document.find_object(id))
            .map(|o| o.visible)
            .ok_or_else(|| AubrietaError::not_found(format!("object `{id}` not found")))?;

        let cmd = CommandRequest::new(Command::SetVisibility {
            id,
            visible: !current,
        });
        bridge.submit_command(cmd)
    }

    /// Toggles an object's locked status in the canonical document.
    pub fn toggle_lock(
        &self,
        bridge: &mut AubrietaGuiBridge,
        id: ObjectId,
    ) -> Result<ChangeSet, AubrietaError> {
        let current = bridge
            .session()
            .and_then(|s| s.document.find_object(id))
            .map(|o| o.locked)
            .ok_or_else(|| AubrietaError::not_found(format!("object `{id}` not found")))?;

        let cmd = CommandRequest::new(Command::SetLocked {
            id,
            locked: !current,
        });
        bridge.submit_command(cmd)
    }

    /// Selects a row, optionally toggling or adding to selection.
    pub fn select_row(&self, bridge: &mut AubrietaGuiBridge, id: ObjectId, additive: bool) {
        if additive {
            bridge.toggle_selection(id);
        } else {
            bridge.set_selection(vec![id]);
        }
    }

    /// Reorders an object within its surface hierarchy.
    pub fn reorder_row(
        &self,
        bridge: &mut AubrietaGuiBridge,
        surface: SurfaceId,
        id: ObjectId,
        new_index: usize,
    ) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::ReorderObject {
            surface,
            id,
            new_index,
        });
        bridge.submit_command(cmd)
    }

    /// Deletes all currently selected objects via semantic action.
    pub fn delete_selected(
        &self,
        bridge: &mut AubrietaGuiBridge,
    ) -> Result<ChangeSet, AubrietaError> {
        bridge.dispatch_action(ActionRequest::without_payload(ActionId::new(
            "aubrieta.edit.delete",
        )))
    }
}
