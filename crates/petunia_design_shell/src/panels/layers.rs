//! Layers panel controller and presentation interaction (10.5).

use petunia_design_application::{ActionId, ActionRequest, Command, CommandRequest};
use petunia_design_document::ChangeSet;
use petunia_design_foundation::{ObjectId, PetuniaError, SurfaceId};

use crate::bridge::{LayersPresentationModel, PetuniaDesignGuiBridge};

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
    pub fn query_model(&self, bridge: &PetuniaDesignGuiBridge) -> LayersPresentationModel {
        bridge.query_layers()
    }

    /// Toggles an object's visibility flag in the canonical document.
    pub fn toggle_visibility(
        &self,
        bridge: &mut PetuniaDesignGuiBridge,
        id: ObjectId,
    ) -> Result<ChangeSet, PetuniaError> {
        let current = bridge
            .session()
            .and_then(|s| s.find_object(id))
            .map(|o| o.visible)
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` not found")))?;

        let cmd = CommandRequest::new(Command::SetVisibility {
            id,
            visible: !current,
        });
        bridge.submit_command(cmd)
    }

    /// Toggles an object's locked status in the canonical document.
    pub fn toggle_lock(
        &self,
        bridge: &mut PetuniaDesignGuiBridge,
        id: ObjectId,
    ) -> Result<ChangeSet, PetuniaError> {
        let current = bridge
            .session()
            .and_then(|s| s.find_object(id))
            .map(|o| o.locked)
            .ok_or_else(|| PetuniaError::not_found(format!("object `{id}` not found")))?;

        let cmd = CommandRequest::new(Command::SetLocked {
            id,
            locked: !current,
        });
        bridge.submit_command(cmd)
    }

    /// Selects a row, optionally toggling or adding to selection.
    pub fn select_row(&self, bridge: &mut PetuniaDesignGuiBridge, id: ObjectId, additive: bool) {
        if additive {
            bridge.toggle_selection(id);
        } else {
            bridge.set_selection(vec![id]);
        }
    }

    /// Reorders an object within its surface hierarchy.
    pub fn reorder_row(
        &self,
        bridge: &mut PetuniaDesignGuiBridge,
        surface: SurfaceId,
        id: ObjectId,
        new_index: usize,
    ) -> Result<ChangeSet, PetuniaError> {
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
        bridge: &mut PetuniaDesignGuiBridge,
    ) -> Result<ChangeSet, PetuniaError> {
        bridge.dispatch_action(ActionRequest::without_payload(ActionId::new(
            "ptnd.action.edit.delete",
        )))
    }

    /// Groups currently selected objects into a container with a designated role (10.5 One-Tree).
    pub fn group_selection(
        &self,
        bridge: &mut PetuniaDesignGuiBridge,
        role: petunia_design_document::ContainerRole,
    ) -> Result<ChangeSet, PetuniaError> {
        use petunia_design_application::hierarchy_service;
        let sel_ids = bridge.selection().selected_ids;
        let surface = bridge
            .active_surface()
            .ok_or_else(|| PetuniaError::invalid_input("no active surface"))?;
        let group_id = bridge.next_object_id()?;
        let plan = hierarchy_service::plan_group(surface, group_id, sel_ids, role)?;
        let changes =
            bridge.submit_all("Group objects", hierarchy_service::group_commands(plan))?;
        bridge.set_selection(vec![group_id]);
        Ok(changes)
    }

    /// Ungroups currently selected container objects.
    pub fn ungroup_selection(
        &self,
        bridge: &mut PetuniaDesignGuiBridge,
    ) -> Result<ChangeSet, PetuniaError> {
        let sel_ids = bridge.selection().selected_ids;
        let mut combined = ChangeSet::empty();
        for id in sel_ids {
            let is_container = bridge
                .session()
                .and_then(|s| s.find_object(id))
                .is_some_and(|o| o.is_container());
            if is_container {
                let changes = bridge.ungroup(id)?;
                for c in changes.changes {
                    combined.push(c);
                }
            }
        }
        Ok(combined)
    }

    /// Reparents an object to a new container or root with visual position preservation.
    pub fn reparent_row(
        &self,
        bridge: &mut PetuniaDesignGuiBridge,
        id: ObjectId,
        new_parent: Option<ObjectId>,
        target_index: usize,
    ) -> Result<ChangeSet, PetuniaError> {
        bridge.reparent_object(id, new_parent, target_index, true)
    }

    /// Creates a clipping mask where the first selected object clips the rest.
    pub fn create_clipping_mask(
        &self,
        bridge: &mut PetuniaDesignGuiBridge,
    ) -> Result<ChangeSet, PetuniaError> {
        use petunia_design_application::hierarchy_service;
        let sel_ids = bridge.selection().selected_ids;
        let surface = bridge
            .active_surface()
            .ok_or_else(|| PetuniaError::invalid_input("no active surface"))?;
        let group_id = bridge.next_object_id()?;
        let plan = hierarchy_service::plan_clip_group(surface, group_id, &sel_ids)?;
        let changes = bridge.submit_all(
            "Create clip group",
            hierarchy_service::clip_group_commands(plan),
        )?;
        bridge.set_selection(vec![group_id]);
        Ok(changes)
    }

    /// Releases a clipping mask group.
    pub fn release_clipping_mask(
        &self,
        bridge: &mut PetuniaDesignGuiBridge,
        group_id: ObjectId,
    ) -> Result<ChangeSet, PetuniaError> {
        bridge.release_clip_group(group_id)
    }

    /// Moves an object from its current surface to another surface (10.7).
    pub fn move_row_to_surface(
        &self,
        bridge: &mut PetuniaDesignGuiBridge,
        id: ObjectId,
        target_surface: SurfaceId,
    ) -> Result<ChangeSet, PetuniaError> {
        bridge.move_object_to_surface(id, target_surface, true)
    }

    /// Updates surface geometry on the pasteboard (10.7).
    pub fn set_surface_geometry(
        &self,
        bridge: &mut PetuniaDesignGuiBridge,
        surface: SurfaceId,
        origin: [f64; 2],
        dimensions: [f64; 2],
    ) -> Result<ChangeSet, PetuniaError> {
        bridge.set_surface_geometry(surface, origin, dimensions)
    }

    /// Updates surface bleed insets (10.7).
    pub fn set_surface_bleed(
        &self,
        bridge: &mut PetuniaDesignGuiBridge,
        surface: SurfaceId,
        bleed: petunia_design_document::Bleed,
    ) -> Result<ChangeSet, PetuniaError> {
        bridge.set_surface_bleed(surface, bleed)
    }

    /// Updates surface safe margins (10.7).
    pub fn set_surface_margins(
        &self,
        bridge: &mut PetuniaDesignGuiBridge,
        surface: SurfaceId,
        margins: petunia_design_document::Margins,
    ) -> Result<ChangeSet, PetuniaError> {
        bridge.set_surface_margins(surface, margins)
    }

    /// Adds a layout guide to a surface (10.7).
    pub fn add_guide(
        &self,
        bridge: &mut PetuniaDesignGuiBridge,
        surface: SurfaceId,
        guide: petunia_design_document::Guide,
    ) -> Result<ChangeSet, PetuniaError> {
        bridge.add_surface_guide(surface, guide)
    }

    /// Removes a layout guide from a surface (10.7).
    pub fn remove_guide(
        &self,
        bridge: &mut PetuniaDesignGuiBridge,
        surface: SurfaceId,
        guide_id: u32,
    ) -> Result<ChangeSet, PetuniaError> {
        bridge.remove_surface_guide(surface, guide_id)
    }
}
