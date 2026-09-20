//! Canonical AubrietaGuiBridge application facade (09.27).
//!
//! Exposes a coarse-grained, UI-agnostic application facade over document sessions.
//! UI toolkits (GPUI, Floem, tests, MockGuiAdapter) interact exclusively through this bridge.

use std::collections::HashMap;

use aubrieta_application::{ActionId, ActionRequest, CapabilityRegistry, Command, CommandRequest};
use aubrieta_document::{AppearanceStack, ChangeSet, Document};
use aubrieta_foundation::{AubrietaError, ObjectId, SurfaceId};

use super::ports::{
    ActionQueryPort, CommandPort, DocumentQueryPort, InspectionPort, PropertyPort, SelectionPort,
};
use super::session::DocumentSession;
use super::view_models::{
    ActionStateMap, ActionStateViewModel, DocumentSummary, HistoryItemViewModel,
    HistoryPresentationModel, LayersPresentationModel, PropertiesPresentationModel,
    SelectionViewModel, SessionSnapshot,
};

/// Coarse-grained facade connecting external UI adapters to the Aubrieta engine.
#[derive(Debug)]
pub struct AubrietaGuiBridge {
    /// Active document session, if any.
    active_session: Option<DocumentSession>,
    /// Global capability registry for tool/command authorization.
    capabilities: CapabilityRegistry,
}

impl Default for AubrietaGuiBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl AubrietaGuiBridge {
    /// Creates a fresh GUI bridge with core capabilities initialized.
    #[must_use]
    pub fn new() -> Self {
        let mut capabilities = CapabilityRegistry::new();
        // Register core capabilities with explicit states
        capabilities.register(aubrieta_application::CapabilityInfo::available(
            "aubrieta.document.edit",
            "aubrieta_ui_gpui",
        ));
        capabilities.register(aubrieta_application::CapabilityInfo::available(
            "aubrieta.export.vector",
            "aubrieta_io",
        ));
        capabilities.register(aubrieta_application::CapabilityInfo::available(
            "aubrieta.export.raster",
            "aubrieta_io",
        ));
        capabilities.register(aubrieta_application::CapabilityInfo::available(
            "aubrieta.color.proof",
            "aubrieta_color",
        ));

        Self {
            active_session: None,
            capabilities,
        }
    }

    /// Initializes a new empty document session with default surface.
    pub fn new_document(&mut self, title: impl Into<String>) -> Result<(), AubrietaError> {
        let mut session = DocumentSession::new(title);
        // Ensure default surface exists
        let mut gen = aubrieta_foundation::IdGenerator::new();
        let default_surface = gen.next_surface();
        session
            .document
            .surfaces
            .push(aubrieta_document::Surface::new(default_surface, "Canvas"));
        session.active_surface = Some(default_surface);
        self.active_session = Some(session);
        Ok(())
    }

    /// Attaches an existing document into the bridge.
    pub fn open_document(
        &mut self,
        title: impl Into<String>,
        document: Document,
    ) -> Result<(), AubrietaError> {
        let mut session = DocumentSession::with_document(title, document);
        if session.active_surface.is_none() {
            session.active_surface = session.document.surfaces.first().map(|s| s.id);
        }
        self.active_session = Some(session);
        Ok(())
    }

    /// Closes the active session, checking unsaved dirty state.
    pub fn close_session(&mut self, force: bool) -> Result<bool, AubrietaError> {
        if let Some(session) = &self.active_session {
            if session.is_dirty() && !force {
                return Ok(false); // Unsaved changes require user decision
            }
        }
        self.active_session = None;
        Ok(true)
    }

    /// Accesses the active document session.
    #[must_use]
    pub fn session(&self) -> Option<&DocumentSession> {
        self.active_session.as_ref()
    }

    /// Mutably accesses the active document session.
    pub fn session_mut(&mut self) -> Option<&mut DocumentSession> {
        self.active_session.as_mut()
    }

    /// Returns a reference to the global capability registry.
    #[must_use]
    pub fn capabilities(&self) -> &CapabilityRegistry {
        &self.capabilities
    }

    /// Helper to borrow active session or return error if closed.
    #[allow(dead_code)]
    fn session_req(&self) -> Result<&DocumentSession, AubrietaError> {
        self.active_session
            .as_ref()
            .ok_or_else(|| AubrietaError::invalid_input("no active document session"))
    }

    /// Helper to mutably borrow active session or return error if closed.
    fn session_req_mut(&mut self) -> Result<&mut DocumentSession, AubrietaError> {
        self.active_session
            .as_mut()
            .ok_or_else(|| AubrietaError::invalid_input("no active document session"))
    }

    /// Allocates the next unique object ID in the active session.
    pub fn next_object_id(&mut self) -> Result<ObjectId, AubrietaError> {
        self.session_req_mut().map(|s| s.next_object_id())
    }

    /// Submits a validated command request.
    pub fn submit_command(&mut self, request: CommandRequest) -> Result<ChangeSet, AubrietaError> {
        CommandPort::submit_command(self, request)
    }

    /// Undoes the last committed command.
    pub fn undo(&mut self) -> Result<bool, AubrietaError> {
        CommandPort::undo(self)
    }

    /// Redoes the last undone command.
    pub fn redo(&mut self) -> Result<bool, AubrietaError> {
        CommandPort::redo(self)
    }

    /// Returns whether undo is available.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        CommandPort::can_undo(self)
    }

    /// Returns whether redo is available.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        CommandPort::can_redo(self)
    }

    /// Returns current selection view-model.
    #[must_use]
    pub fn selection(&self) -> SelectionViewModel {
        SelectionPort::selection(self)
    }

    /// Replaces selection with explicit IDs.
    pub fn set_selection(&mut self, ids: Vec<ObjectId>) {
        SelectionPort::set_selection(self, ids);
    }

    /// Toggles an ID in selection.
    pub fn toggle_selection(&mut self, id: ObjectId) {
        SelectionPort::toggle_selection(self, id);
    }

    /// Clears selection.
    pub fn clear_selection(&mut self) {
        SelectionPort::clear_selection(self);
    }

    /// Selects all objects on active surface.
    pub fn select_all(&mut self) {
        SelectionPort::select_all(self);
    }

    /// Queries properties view-model.
    #[must_use]
    pub fn query_properties(&self) -> PropertiesPresentationModel {
        PropertyPort::query_properties(self)
    }

    /// Sets fill token.
    pub fn set_fill(
        &mut self,
        id: ObjectId,
        fill: Option<String>,
    ) -> Result<ChangeSet, AubrietaError> {
        PropertyPort::set_fill(self, id, fill)
    }

    /// Sets stroke.
    pub fn set_stroke(
        &mut self,
        id: ObjectId,
        stroke: Option<String>,
        width: f64,
    ) -> Result<ChangeSet, AubrietaError> {
        PropertyPort::set_stroke(self, id, stroke, width)
    }

    /// Sets opacity.
    pub fn set_opacity(&mut self, id: ObjectId, opacity: f64) -> Result<ChangeSet, AubrietaError> {
        PropertyPort::set_opacity(self, id, opacity)
    }

    /// Sets visibility.
    pub fn set_visibility(
        &mut self,
        id: ObjectId,
        visible: bool,
    ) -> Result<ChangeSet, AubrietaError> {
        PropertyPort::set_visibility(self, id, visible)
    }

    /// Sets locked.
    pub fn set_locked(&mut self, id: ObjectId, locked: bool) -> Result<ChangeSet, AubrietaError> {
        PropertyPort::set_locked(self, id, locked)
    }

    /// Sets bounds.
    pub fn set_bounds(
        &mut self,
        id: ObjectId,
        bounds: Option<[f64; 4]>,
        rotation: f64,
    ) -> Result<ChangeSet, AubrietaError> {
        PropertyPort::set_bounds(self, id, bounds, rotation)
    }

    /// Sets appearance stack.
    pub fn set_appearance(
        &mut self,
        id: ObjectId,
        appearance: Option<AppearanceStack>,
    ) -> Result<ChangeSet, AubrietaError> {
        PropertyPort::set_appearance(self, id, appearance)
    }

    /// Resolves snapshot.
    #[must_use]
    pub fn snapshot(&self) -> SessionSnapshot {
        DocumentQueryPort::snapshot(self)
    }

    /// Resolves layers presentation model.
    #[must_use]
    pub fn query_layers(&self) -> LayersPresentationModel {
        DocumentQueryPort::query_layers(self)
    }

    /// Queries actions state map.
    #[must_use]
    pub fn query_actions(&self) -> ActionStateMap {
        ActionQueryPort::query_actions(self)
    }

    /// Dispatches action request.
    pub fn dispatch_action(&mut self, request: ActionRequest) -> Result<ChangeSet, AubrietaError> {
        ActionQueryPort::dispatch_action(self, request)
    }

    /// Resolves active surface.
    #[must_use]
    pub fn active_surface(&self) -> Option<SurfaceId> {
        InspectionPort::active_surface(self)
    }

    /// Checks if session has unsaved modifications.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        InspectionPort::is_dirty(self)
    }

    /// Resolves presentation model for the history/undo stack.
    #[must_use]
    pub fn query_history(&self) -> HistoryPresentationModel {
        if let Some(session) = &self.active_session {
            let mut undo_items = Vec::new();
            let mut redo_items = Vec::new();

            for (i, entry) in session.history.undo_entries().iter().enumerate() {
                undo_items.push(HistoryItemViewModel {
                    index: i,
                    description: format!("Operation #{}", i + 1),
                    change_count: entry.len(),
                });
            }

            for (i, entry) in session.history.redo_entries().iter().enumerate() {
                redo_items.push(HistoryItemViewModel {
                    index: i,
                    description: format!("Redo #{}", i + 1),
                    change_count: entry.len(),
                });
            }

            let active_label = undo_items.last().map(|it| it.description.clone());

            HistoryPresentationModel {
                undo_stack: undo_items,
                redo_stack: redo_items,
                can_undo: session.history.can_undo(),
                can_redo: session.history.can_redo(),
                active_undo_label: active_label,
            }
        } else {
            HistoryPresentationModel::default()
        }
    }
}

impl ActionQueryPort for AubrietaGuiBridge {
    fn query_actions(&self) -> ActionStateMap {
        let mut states = HashMap::new();
        let has_session = self.active_session.is_some();
        let has_selection = self
            .active_session
            .as_ref()
            .is_some_and(|s| !s.selection.selected_ids.is_empty());
        let can_undo = self
            .active_session
            .as_ref()
            .is_some_and(|s| s.history.can_undo());
        let can_redo = self
            .active_session
            .as_ref()
            .is_some_and(|s| s.history.can_redo());
        let is_dirty = self.active_session.as_ref().is_some_and(|s| s.is_dirty());

        let actions = [
            (
                "aubrieta.file.save",
                has_session && is_dirty,
                if !has_session {
                    Some("No active document".to_string())
                } else if !is_dirty {
                    Some("No unsaved modifications".to_string())
                } else {
                    None
                },
            ),
            (
                "aubrieta.file.export",
                has_session,
                if has_session {
                    None
                } else {
                    Some("No active document".to_string())
                },
            ),
            (
                "aubrieta.edit.undo",
                can_undo,
                if can_undo {
                    None
                } else {
                    Some("Nothing to undo".to_string())
                },
            ),
            (
                "aubrieta.edit.redo",
                can_redo,
                if can_redo {
                    None
                } else {
                    Some("Nothing to redo".to_string())
                },
            ),
            (
                "aubrieta.edit.delete",
                has_selection,
                if has_selection {
                    None
                } else {
                    Some("No selection to delete".to_string())
                },
            ),
            (
                "aubrieta.edit.select_all",
                has_session,
                if has_session {
                    None
                } else {
                    Some("No active document".to_string())
                },
            ),
            (
                "aubrieta.edit.deselect",
                has_selection,
                if has_selection {
                    None
                } else {
                    Some("No selection to deselect".to_string())
                },
            ),
        ];

        for (id_str, enabled, disabled_reason) in actions {
            let action_id = ActionId::new(id_str);
            states.insert(
                id_str.to_string(),
                ActionStateViewModel {
                    action_id,
                    is_enabled: enabled,
                    is_checked: false,
                    disabled_reason,
                },
            );
        }

        ActionStateMap { states }
    }

    fn is_action_enabled(&self, action: &ActionId) -> bool {
        let map = self.query_actions();
        map.get(action).is_some_and(|s| s.is_enabled)
    }

    fn dispatch_action(&mut self, request: ActionRequest) -> Result<ChangeSet, AubrietaError> {
        let session = self.session_req_mut()?;
        session.dispatch_action(request)
    }
}

impl CommandPort for AubrietaGuiBridge {
    fn submit_command(&mut self, request: CommandRequest) -> Result<ChangeSet, AubrietaError> {
        let session = self.session_req_mut()?;
        session.execute_command(request)
    }

    fn undo(&mut self) -> Result<bool, AubrietaError> {
        let session = self.session_req_mut()?;
        session.undo()
    }

    fn redo(&mut self) -> Result<bool, AubrietaError> {
        let session = self.session_req_mut()?;
        session.redo()
    }

    fn can_undo(&self) -> bool {
        self.active_session
            .as_ref()
            .is_some_and(|s| s.history.can_undo())
    }

    fn can_redo(&self) -> bool {
        self.active_session
            .as_ref()
            .is_some_and(|s| s.history.can_redo())
    }
}

impl PropertyPort for AubrietaGuiBridge {
    fn query_properties(&self) -> PropertiesPresentationModel {
        self.active_session
            .as_ref()
            .map_or_else(PropertiesPresentationModel::default, |s| {
                s.properties_presentation_model()
            })
    }

    fn set_fill(&mut self, id: ObjectId, fill: Option<String>) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::SetFill { id, fill });
        self.submit_command(cmd)
    }

    fn set_stroke(
        &mut self,
        id: ObjectId,
        stroke: Option<String>,
        width: f64,
    ) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::SetStroke { id, stroke, width });
        self.submit_command(cmd)
    }

    fn set_opacity(&mut self, id: ObjectId, opacity: f64) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::SetOpacity { id, opacity });
        self.submit_command(cmd)
    }

    fn set_visibility(&mut self, id: ObjectId, visible: bool) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::SetVisibility { id, visible });
        self.submit_command(cmd)
    }

    fn set_locked(&mut self, id: ObjectId, locked: bool) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::SetLocked { id, locked });
        self.submit_command(cmd)
    }

    fn set_bounds(
        &mut self,
        id: ObjectId,
        bounds: Option<[f64; 4]>,
        rotation: f64,
    ) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::SetBounds {
            id,
            bounds,
            rotation,
        });
        self.submit_command(cmd)
    }

    fn set_appearance(
        &mut self,
        id: ObjectId,
        appearance: Option<aubrieta_document::AppearanceStack>,
    ) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::SetAppearance { id, appearance });
        self.submit_command(cmd)
    }
}

impl DocumentQueryPort for AubrietaGuiBridge {
    fn snapshot(&self) -> SessionSnapshot {
        self.active_session.as_ref().map_or_else(
            || SessionSnapshot {
                active_surface: None,
                title: "No Document".to_string(),
                revision: 0,
                is_dirty: false,
                surface_count: 0,
                total_objects: 0,
                selected_count: 0,
                can_undo: false,
                can_redo: false,
            },
            |s| s.snapshot(),
        )
    }

    fn summary(&self) -> DocumentSummary {
        self.active_session
            .as_ref()
            .map_or_else(DocumentSummary::default, |s| s.summary())
    }

    fn query_layers(&self) -> LayersPresentationModel {
        self.active_session
            .as_ref()
            .map_or_else(LayersPresentationModel::default, |s| {
                s.layers_presentation_model()
            })
    }
}

impl SelectionPort for AubrietaGuiBridge {
    fn selection(&self) -> SelectionViewModel {
        self.active_session
            .as_ref()
            .map_or_else(SelectionViewModel::default, |s| s.selection_view_model())
    }

    fn set_selection(&mut self, ids: Vec<ObjectId>) {
        if let Some(session) = &mut self.active_session {
            session.selection.select_exact(ids);
            session.prune_selection();
        }
    }

    fn toggle_selection(&mut self, id: ObjectId) {
        if let Some(session) = &mut self.active_session {
            session.selection.toggle(id);
            session.prune_selection();
        }
    }

    fn clear_selection(&mut self) {
        if let Some(session) = &mut self.active_session {
            session.selection.clear();
        }
    }

    fn select_all(&mut self) {
        if let Some(session) = &mut self.active_session {
            session.select_all();
        }
    }
}

impl InspectionPort for AubrietaGuiBridge {
    fn active_surface(&self) -> Option<SurfaceId> {
        self.active_session.as_ref().and_then(|s| s.active_surface)
    }

    fn set_active_surface(&mut self, id: SurfaceId) -> Result<(), AubrietaError> {
        let session = self.session_req_mut()?;
        if session.document.surfaces.iter().any(|s| s.id == id) {
            session.active_surface = Some(id);
            Ok(())
        } else {
            Err(AubrietaError::not_found(format!(
                "surface `{id}` not found in document"
            )))
        }
    }

    fn revision(&self) -> u64 {
        self.active_session
            .as_ref()
            .map_or(0, |s| s.current_revision)
    }

    fn is_dirty(&self) -> bool {
        self.active_session.as_ref().is_some_and(|s| s.is_dirty())
    }
}
