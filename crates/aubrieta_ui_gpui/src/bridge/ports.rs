//! Canonical semantic application ports (09.27).
//!
//! UI adapters depend inward on these narrow semantic ports. Domain logic
//! never imports UI toolkit types.

use aubrieta_application::{ActionId, ActionRequest, CommandRequest};
use aubrieta_document::ChangeSet;
use aubrieta_foundation::{AubrietaError, ObjectId, SurfaceId};

use super::view_models::{
    ActionStateMap, DocumentSummary, LayersPresentationModel, PropertiesPresentationModel,
    SelectionViewModel, SessionSnapshot,
};

/// Discovers actions and their enabled/checked/disabled-reason state.
pub trait ActionQueryPort {
    /// Returns the active state of all registered actions in the current context.
    fn query_actions(&self) -> ActionStateMap;
    /// Tests whether a specific action can currently be invoked.
    fn is_action_enabled(&self, action: &ActionId) -> bool;
    /// Dispatches an action request by identifier.
    fn dispatch_action(&mut self, request: ActionRequest) -> Result<ChangeSet, AubrietaError>;
}

/// Executes validated commands and transactions with undo/redo guarantees.
pub trait CommandPort {
    /// Submits a validated command request through the authoritative mutator lane.
    fn submit_command(&mut self, request: CommandRequest) -> Result<ChangeSet, AubrietaError>;
    /// Undoes the last committed command.
    fn undo(&mut self) -> Result<bool, AubrietaError>;
    /// Redoes the last undone command.
    fn redo(&mut self) -> Result<bool, AubrietaError>;
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
    fn set_fill(&mut self, id: ObjectId, fill: Option<String>) -> Result<ChangeSet, AubrietaError>;
    /// Sets an object's stroke token and width.
    fn set_stroke(
        &mut self,
        id: ObjectId,
        stroke: Option<String>,
        width: f64,
    ) -> Result<ChangeSet, AubrietaError>;
    /// Sets an object's opacity in `[0.0, 1.0]`.
    fn set_opacity(&mut self, id: ObjectId, opacity: f64) -> Result<ChangeSet, AubrietaError>;
    /// Sets an object's visibility flag.
    fn set_visibility(&mut self, id: ObjectId, visible: bool) -> Result<ChangeSet, AubrietaError>;
    /// Sets an object's locked flag.
    fn set_locked(&mut self, id: ObjectId, locked: bool) -> Result<ChangeSet, AubrietaError>;
    /// Sets an object's bounds `[x, y, w, h]` and rotation in document coordinates.
    fn set_bounds(
        &mut self,
        id: ObjectId,
        bounds: Option<[f64; 4]>,
        rotation: f64,
    ) -> Result<ChangeSet, AubrietaError>;
    /// Sets an object's appearance stack (10.4).
    fn set_appearance(
        &mut self,
        id: ObjectId,
        appearance: Option<aubrieta_document::AppearanceStack>,
    ) -> Result<ChangeSet, AubrietaError>;
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
    fn set_active_surface(&mut self, id: SurfaceId) -> Result<(), AubrietaError>;
    /// Returns the current document revision number.
    fn revision(&self) -> u64;
    /// Returns whether there are unsaved modifications.
    fn is_dirty(&self) -> bool;
}
