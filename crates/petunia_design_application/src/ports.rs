//! Canonical semantic application ports (09.27).
//!
//! UI adapters depend inward on these narrow semantic ports. Domain logic
//! never imports UI toolkit types.

use petunia_design_document::ChangeSet;
use petunia_design_foundation::{PetuniaError, ObjectId, SurfaceId};

use super::view_models::{
    ActionStateMap, DocumentSummary, LayersPresentationModel, PropertiesPresentationModel,
    SelectionViewModel, SessionSnapshot,
};

use crate::{ActionId, ActionRequest, CommandRequest};

/// Discovers actions and their enabled/checked/disabled-reason state.
pub trait ActionQueryPort {
    /// Returns the active state of all registered actions in the current context.
    fn query_actions(&self) -> ActionStateMap;
    /// Tests whether a specific action can currently be invoked.
    fn is_action_enabled(&self, action: &ActionId) -> bool;
    /// Dispatches an action request by identifier.
    fn dispatch_action(&mut self, request: ActionRequest) -> Result<ChangeSet, PetuniaError>;
}

/// Executes validated commands and transactions with undo/redo guarantees.
pub trait CommandPort {
    /// Submits a validated command request through the authoritative mutator lane.
    fn submit_command(&mut self, request: CommandRequest) -> Result<ChangeSet, PetuniaError>;
    /// Undoes the last committed command.
    fn undo(&mut self) -> Result<bool, PetuniaError>;
    /// Redoes the last undone command.
    fn redo(&mut self) -> Result<bool, PetuniaError>;
    /// Returns whether undo is available.
    fn can_undo(&self) -> bool;
    /// Returns whether redo is available.
    fn can_redo(&self) -> bool;
}

/// Typed semantic property query and edit port.
pub trait PropertyPort {
    /// Resolves presentation properties for the current selection or active document.
    fn query_properties(&self) -> PropertiesPresentationModel;
    /// Sets an object's fill token reference.
    fn set_fill(&mut self, id: ObjectId, fill: Option<String>) -> Result<ChangeSet, PetuniaError>;
    /// Sets an object's stroke token and width.
    fn set_stroke(
        &mut self,
        id: ObjectId,
        stroke: Option<String>,
        width: f64,
    ) -> Result<ChangeSet, PetuniaError>;
    /// Sets an object's opacity in `[0.0, 1.0]`.
    fn set_opacity(&mut self, id: ObjectId, opacity: f64) -> Result<ChangeSet, PetuniaError>;
    /// Sets an object's visibility flag.
    fn set_visibility(&mut self, id: ObjectId, visible: bool) -> Result<ChangeSet, PetuniaError>;
    /// Sets an object's locked flag.
    fn set_locked(&mut self, id: ObjectId, locked: bool) -> Result<ChangeSet, PetuniaError>;
    /// Sets an object's bounds `[x, y, w, h]` and rotation in document coordinates.
    fn set_bounds(
        &mut self,
        id: ObjectId,
        bounds: Option<[f64; 4]>,
        rotation: f64,
    ) -> Result<ChangeSet, PetuniaError>;
    /// Sets an object's appearance stack (10.4).
    fn set_appearance(
        &mut self,
        id: ObjectId,
        appearance: Option<petunia_design_document::AppearanceStack>,
    ) -> Result<ChangeSet, PetuniaError>;
}

/// Query port for immutable document summaries and session snapshots.
pub trait DocumentQueryPort {
    /// Produces a full lightweight snapshot of the current session state.
    fn snapshot(&self) -> SessionSnapshot;
    /// Returns high-level document metrics (surfaces, objects count, dirty status).
    fn summary(&self) -> DocumentSummary;
    /// Resolves the unified layers tree presentation model for the layers panel.
    fn query_layers(&self) -> LayersPresentationModel;
}

/// Selection query and mutation port without widget ownership.
pub trait SelectionPort {
    /// Returns the current selection view-model.
    fn selection(&self) -> SelectionViewModel;
    /// Replaces the selection with an explicit set of object IDs.
    fn set_selection(&mut self, ids: Vec<ObjectId>);
    /// Toggles an object's presence in the active selection.
    fn toggle_selection(&mut self, id: ObjectId);
    /// Clears the selection.
    fn clear_selection(&mut self);
    /// Selects all eligible objects on the active surface.
    fn select_all(&mut self);
}

/// High-level inspection port for testing, headless conformance, and agents.
pub trait InspectionPort {
    /// Returns the active surface ID, if any.
    fn active_surface(&self) -> Option<SurfaceId>;
    /// Sets the active surface by ID.
    fn set_active_surface(&mut self, id: SurfaceId) -> Result<(), PetuniaError>;
    /// Returns the current document revision number.
    fn revision(&self) -> u64;
    /// Returns whether there are unsaved modifications.
    fn is_dirty(&self) -> bool;
}

/// Hierarchy and grouping operations under the One-Tree invariant (10.5).
pub trait HierarchyPort {
    /// Groups the specified objects into a container object with a designated role.
    fn group_objects(
        &mut self,
        surface: SurfaceId,
        group_id: ObjectId,
        child_ids: Vec<ObjectId>,
        role: petunia_design_document::ContainerRole,
    ) -> Result<ChangeSet, PetuniaError>;
    /// Ungroups a container object.
    fn ungroup(&mut self, group_id: ObjectId) -> Result<ChangeSet, PetuniaError>;
    /// Reparents an object to a new container or root.
    fn reparent_object(
        &mut self,
        id: ObjectId,
        new_parent: Option<ObjectId>,
        target_index: usize,
        preserve_world_transform: bool,
    ) -> Result<ChangeSet, PetuniaError>;
    /// Creates a clipping mask group.
    fn create_clip_group(
        &mut self,
        surface: SurfaceId,
        group_id: ObjectId,
        mask_id: ObjectId,
        content_ids: Vec<ObjectId>,
    ) -> Result<ChangeSet, PetuniaError>;
    /// Releases a clipping mask group.
    fn release_clip_group(&mut self, group_id: ObjectId) -> Result<ChangeSet, PetuniaError>;
}

/// Multi-surface, artboard, bleed, margin, and guide layout port (10.7).
pub trait SurfacePort {
    /// Resizes or repositions a surface (artboard) on the canvas pasteboard.
    fn set_surface_geometry(
        &mut self,
        surface: SurfaceId,
        origin: [f64; 2],
        dimensions: [f64; 2],
    ) -> Result<ChangeSet, PetuniaError>;

    /// Updates bleed insets on a surface.
    fn set_surface_bleed(
        &mut self,
        surface: SurfaceId,
        bleed: petunia_design_document::Bleed,
    ) -> Result<ChangeSet, PetuniaError>;

    /// Updates safe margins on a surface.
    fn set_surface_margins(
        &mut self,
        surface: SurfaceId,
        margins: petunia_design_document::Margins,
    ) -> Result<ChangeSet, PetuniaError>;

    /// Sets or clears the surface background color/token.
    fn set_surface_background(
        &mut self,
        surface: SurfaceId,
        background: Option<String>,
    ) -> Result<ChangeSet, PetuniaError>;

    /// Adds a layout guide to a surface.
    fn add_surface_guide(
        &mut self,
        surface: SurfaceId,
        guide: petunia_design_document::Guide,
    ) -> Result<ChangeSet, PetuniaError>;

    /// Removes a layout guide from a surface by guide ID.
    fn remove_surface_guide(
        &mut self,
        surface: SurfaceId,
        guide_id: u32,
    ) -> Result<ChangeSet, PetuniaError>;

    /// Moves an object from its current surface to another surface.
    fn move_object_to_surface(
        &mut self,
        id: ObjectId,
        target_surface: SurfaceId,
        preserve_world_transform: bool,
    ) -> Result<ChangeSet, PetuniaError>;
}

/// Variable Data and Data Merge operations port (10.11).
pub trait VariableDataPort {
    /// Registers or imports a tabular data source.
    fn import_data_source(
        &mut self,
        source: petunia_design_document::DataSourceDefinition,
    ) -> Result<ChangeSet, PetuniaError>;

    /// Removes a data source and its cascading bindings.
    fn remove_data_source(
        &mut self,
        id: petunia_design_document::DataSourceId,
    ) -> Result<ChangeSet, PetuniaError>;

    /// Adds a data binding between a field and a document object property.
    fn add_data_binding(
        &mut self,
        binding: petunia_design_document::DataBinding,
    ) -> Result<ChangeSet, PetuniaError>;

    /// Removes a data binding.
    fn remove_data_binding(
        &mut self,
        id: petunia_design_document::BindingId,
    ) -> Result<ChangeSet, PetuniaError>;

    /// Materializes records into generated surfaces on the pasteboard.
    fn materialize_merge(
        &mut self,
        source_id: petunia_design_document::DataSourceId,
        template_surface: SurfaceId,
    ) -> Result<ChangeSet, PetuniaError>;

    /// Resolves the current Variable Data presentation model.
    fn query_variable_data(&self) -> super::view_models::DataMergePresentationModel;
}
