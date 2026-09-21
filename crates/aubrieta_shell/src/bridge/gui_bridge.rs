//! Canonical AubrietaGuiBridge application facade (09.27).
//!
//! Exposes a coarse-grained, UI-agnostic application facade over document sessions.
//! UI toolkits (GPUI, Floem, tests, MockGuiAdapter) interact exclusively through this bridge.

use std::collections::HashMap;

use aubrieta_application::{ActionId, ActionRequest, CapabilityRegistry, Command, CommandRequest};
use aubrieta_document::{
    AppearanceStack, BindingId, Bleed, ChangeSet, ContainerRole, DataBinding, DataSourceDefinition,
    DataSourceId, Document, Guide, Margins, ShapeKind,
};
use aubrieta_foundation::{AubrietaError, ObjectId, SurfaceId};
use aubrieta_geometry::BooleanOp;

use aubrieta_application::ports::{
    ActionQueryPort, CommandPort, DocumentQueryPort, HierarchyPort, InspectionPort, PropertyPort,
    SelectionPort, SurfacePort, VariableDataPort,
};
use aubrieta_application::session::DocumentSession;
use aubrieta_application::view_models::{
    ActionStateMap, ActionStateViewModel, DataMergePresentationModel, DocumentSummary,
    HistoryItemViewModel, HistoryPresentationModel, LayersPresentationModel,
    PropertiesPresentationModel, SelectionViewModel, SessionSnapshot,
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
            "aubrieta_shell",
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

    /// Initializes a new empty document session with default surface (A1).
    /// The initial surface goes through the command lane so undo, history
    /// and revision stay exact from the first mutation.
    pub fn new_document(&mut self, title: impl Into<String>) -> Result<(), AubrietaError> {
        let mut session = DocumentSession::new(title);
        let surface_id = session.next_surface_id();
        session.execute_command(CommandRequest::new(Command::CreateSurface {
            id: surface_id,
            name: "Canvas".to_string(),
        }))?;
        session.set_active_surface(surface_id);
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
        if session.active_surface()().is_none() {
            if let Some(first) = session.surfaces().first() {
                session.set_active_surface(first.id);
            }
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

    /// Allocates the next unique surface ID in the active session.
    pub fn next_surface_id(&mut self) -> Result<SurfaceId, AubrietaError> {
        self.session_req_mut().map(|s| s.next_surface_id())
    }

    /// Sets the currently active surface.
    pub fn set_active_surface(&mut self, surface: SurfaceId) -> Result<(), AubrietaError> {
        InspectionPort::set_active_surface(self, surface)
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

    /// Creates a shape or text object with explicit shape, geometry and appearance.
    #[allow(clippy::too_many_arguments)]
    pub fn create_shape_object(
        &mut self,
        surface: SurfaceId,
        id: ObjectId,
        name: String,
        shape: ShapeKind,
        bounds: Option<[f64; 4]>,
        fill: Option<String>,
        stroke: Option<String>,
        stroke_width: f64,
    ) -> Result<ChangeSet, AubrietaError> {
        self.submit_command(CommandRequest::new(Command::CreateShapeObject {
            surface,
            id,
            name,
            shape,
            bounds,
            fill,
            stroke,
            stroke_width,
        }))
    }

    /// Sets an object's vector shape or text descriptor.
    pub fn set_shape(
        &mut self,
        id: ObjectId,
        shape: Option<ShapeKind>,
    ) -> Result<ChangeSet, AubrietaError> {
        self.submit_command(CommandRequest::new(Command::SetShape { id, shape }))
    }

    /// Executes a vector boolean operation on two objects.
    pub fn apply_boolean(
        &mut self,
        surface: SurfaceId,
        target_id: ObjectId,
        subject_id: ObjectId,
        clip_id: ObjectId,
        op: BooleanOp,
    ) -> Result<ChangeSet, AubrietaError> {
        self.submit_command(CommandRequest::new(Command::ApplyBoolean {
            surface,
            target_id,
            subject_id,
            clip_id,
            op,
        }))
    }

    /// Converts a parametric shape or text object to an editable vector path (10.3, 10.6).
    pub fn convert_to_curves(&mut self, id: ObjectId) -> Result<ChangeSet, AubrietaError> {
        self.submit_command(CommandRequest::new(Command::ConvertToCurves { id }))
    }

    /// Bakes corner geometry into an explicit vector path (10.2, 10.3).
    pub fn bake_corners(&mut self, id: ObjectId) -> Result<ChangeSet, AubrietaError> {
        self.submit_command(CommandRequest::new(Command::BakeCorners { id }))
    }

    /// Offsets a path or object bounds outward or inward (10.3).
    pub fn offset_path(&mut self, id: ObjectId, delta: f64) -> Result<ChangeSet, AubrietaError> {
        self.submit_command(CommandRequest::new(Command::OffsetPath { id, delta }))
    }

    /// Sets the blend mode of an object (10.4).
    pub fn set_blend_mode(
        &mut self,
        id: ObjectId,
        blend_mode: aubrieta_document::BlendMode,
    ) -> Result<ChangeSet, AubrietaError> {
        let mut app = if let Some(session) = self.active_session.as_ref() {
            session
                .document
                .find_object(id)
                .map(|o| o.effective_appearance())
                .unwrap_or_default()
        } else {
            aubrieta_document::AppearanceStack::new()
        };
        app.blend_mode = blend_mode;
        self.submit_command(CommandRequest::new(Command::SetAppearance {
            id,
            appearance: Some(app),
        }))
    }

    /// Adds a fill layer to the object's appearance stack (10.4).
    pub fn add_fill(
        &mut self,
        id: ObjectId,
        paint: aubrieta_document::Paint,
    ) -> Result<ChangeSet, AubrietaError> {
        let mut app = if let Some(session) = self.active_session.as_ref() {
            session
                .document
                .find_object(id)
                .map(|o| o.effective_appearance())
                .unwrap_or_default()
        } else {
            aubrieta_document::AppearanceStack::new()
        };
        let next_id = app.fills.iter().map(|f| f.id).max().unwrap_or(0) + 1;
        let mut fill_item = aubrieta_document::FillItem::solid(next_id, "aubrieta.blue/500");
        fill_item.paint = paint;
        app.fills.push(fill_item);
        self.submit_command(CommandRequest::new(Command::SetAppearance {
            id,
            appearance: Some(app),
        }))
    }

    /// Removes a fill layer by its ID (10.4).
    pub fn remove_fill(
        &mut self,
        id: ObjectId,
        fill_id: u32,
    ) -> Result<ChangeSet, AubrietaError> {
        let mut app = if let Some(session) = self.active_session.as_ref() {
            session
                .document
                .find_object(id)
                .map(|o| o.effective_appearance())
                .unwrap_or_default()
        } else {
            aubrieta_document::AppearanceStack::new()
        };
        app.fills.retain(|f| f.id != fill_id);
        self.submit_command(CommandRequest::new(Command::SetAppearance {
            id,
            appearance: Some(app),
        }))
    }

    /// Adds a stroke layer to the object's appearance stack (10.4).
    pub fn add_stroke(
        &mut self,
        id: ObjectId,
        paint: aubrieta_document::Paint,
        width: f64,
    ) -> Result<ChangeSet, AubrietaError> {
        let mut app = if let Some(session) = self.active_session.as_ref() {
            session
                .document
                .find_object(id)
                .map(|o| o.effective_appearance())
                .unwrap_or_default()
        } else {
            aubrieta_document::AppearanceStack::new()
        };
        let next_id = app.strokes.iter().map(|s| s.id).max().unwrap_or(0) + 1;
        let mut stroke_item =
            aubrieta_document::StrokeItem::solid(next_id, "aubrieta.gray/700", width);
        stroke_item.paint = paint;
        app.strokes.push(stroke_item);
        self.submit_command(CommandRequest::new(Command::SetAppearance {
            id,
            appearance: Some(app),
        }))
    }

    /// Removes a stroke layer by its ID (10.4).
    pub fn remove_stroke(
        &mut self,
        id: ObjectId,
        stroke_id: u32,
    ) -> Result<ChangeSet, AubrietaError> {
        let mut app = if let Some(session) = self.active_session.as_ref() {
            session
                .document
                .find_object(id)
                .map(|o| o.effective_appearance())
                .unwrap_or_default()
        } else {
            aubrieta_document::AppearanceStack::new()
        };
        app.strokes.retain(|s| s.id != stroke_id);
        self.submit_command(CommandRequest::new(Command::SetAppearance {
            id,
            appearance: Some(app),
        }))
    }

    /// Sets linear gradient fill with two stops on the object (10.4).
    pub fn set_linear_gradient_fill(
        &mut self,
        id: ObjectId,
        color1: impl Into<String>,
        color2: impl Into<String>,
    ) -> Result<ChangeSet, AubrietaError> {
        let mut app = if let Some(session) = self.active_session.as_ref() {
            session
                .document
                .find_object(id)
                .map(|o| o.effective_appearance())
                .unwrap_or_default()
        } else {
            aubrieta_document::AppearanceStack::new()
        };
        let grad = aubrieta_document::LinearGradient::new(
            [0.0, 0.0],
            [1.0, 0.0],
            vec![
                aubrieta_document::GradientStop::new(0.0, color1),
                aubrieta_document::GradientStop::new(1.0, color2),
            ],
        );
        if let Some(first_fill) = app.fills.first_mut() {
            first_fill.paint = aubrieta_document::Paint::LinearGradient(grad);
        } else {
            app.fills.push(aubrieta_document::FillItem::linear_gradient(1, grad));
        }
        self.submit_command(CommandRequest::new(Command::SetAppearance {
            id,
            appearance: Some(app),
        }))
    }

    /// Sets radial gradient fill with two stops on the object (10.4).
    pub fn set_radial_gradient_fill(
        &mut self,
        id: ObjectId,
        color1: impl Into<String>,
        color2: impl Into<String>,
    ) -> Result<ChangeSet, AubrietaError> {
        let mut app = if let Some(session) = self.active_session.as_ref() {
            session
                .document
                .find_object(id)
                .map(|o| o.effective_appearance())
                .unwrap_or_default()
        } else {
            aubrieta_document::AppearanceStack::new()
        };
        let grad = aubrieta_document::RadialGradient::new(
            [0.5, 0.5],
            0.5,
            vec![
                aubrieta_document::GradientStop::new(0.0, color1),
                aubrieta_document::GradientStop::new(1.0, color2),
            ],
        );
        if let Some(first_fill) = app.fills.first_mut() {
            first_fill.paint = aubrieta_document::Paint::RadialGradient(grad);
        } else {
            app.fills.push(aubrieta_document::FillItem::radial_gradient(1, grad));
        }
        self.submit_command(CommandRequest::new(Command::SetAppearance {
            id,
            appearance: Some(app),
        }))
    }

    /// Aligns multiple objects relative to their collective bounds (10.1).
    pub fn align_objects(
        &mut self,
        surface: SurfaceId,
        ids: Vec<ObjectId>,
        mode: aubrieta_document::AlignmentMode,
    ) -> Result<ChangeSet, AubrietaError> {
        self.submit_command(CommandRequest::new(Command::AlignObjects {
            surface,
            ids,
            mode,
        }))
    }

    /// Distributes objects evenly along an axis (10.1).
    pub fn distribute_objects(
        &mut self,
        surface: SurfaceId,
        ids: Vec<ObjectId>,
        axis: aubrieta_document::DistributionAxis,
    ) -> Result<ChangeSet, AubrietaError> {
        self.submit_command(CommandRequest::new(Command::DistributeObjects {
            surface,
            ids,
            axis,
        }))
    }

    /// Slices or splits a path object at a specific point (10.2).
    pub fn slice_path(
        &mut self,
        id: ObjectId,
        point: [f64; 2],
    ) -> Result<ChangeSet, AubrietaError> {
        self.submit_command(CommandRequest::new(Command::SlicePath { id, point }))
    }

    /// Groups objects into a container with a designated role (10.5 One-Tree).
    pub fn group_objects(
        &mut self,
        surface: SurfaceId,
        group_id: ObjectId,
        child_ids: Vec<ObjectId>,
        role: ContainerRole,
    ) -> Result<ChangeSet, AubrietaError> {
        HierarchyPort::group_objects(self, surface, group_id, child_ids, role)
    }

    /// Ungroups a container object.
    pub fn ungroup(&mut self, group_id: ObjectId) -> Result<ChangeSet, AubrietaError> {
        HierarchyPort::ungroup(self, group_id)
    }

    /// Reparents an object to a new container or root.
    pub fn reparent_object(
        &mut self,
        id: ObjectId,
        new_parent: Option<ObjectId>,
        target_index: usize,
        preserve_world_transform: bool,
    ) -> Result<ChangeSet, AubrietaError> {
        HierarchyPort::reparent_object(self, id, new_parent, target_index, preserve_world_transform)
    }

    /// Creates a clipping mask group.
    pub fn create_clip_group(
        &mut self,
        surface: SurfaceId,
        group_id: ObjectId,
        mask_id: ObjectId,
        content_ids: Vec<ObjectId>,
    ) -> Result<ChangeSet, AubrietaError> {
        HierarchyPort::create_clip_group(self, surface, group_id, mask_id, content_ids)
    }

    /// Releases a clipping mask group.
    pub fn release_clip_group(&mut self, group_id: ObjectId) -> Result<ChangeSet, AubrietaError> {
        HierarchyPort::release_clip_group(self, group_id)
    }

    /// Sets surface origin and dimensions (10.7).
    pub fn set_surface_geometry(
        &mut self,
        surface: SurfaceId,
        origin: [f64; 2],
        dimensions: [f64; 2],
    ) -> Result<ChangeSet, AubrietaError> {
        SurfacePort::set_surface_geometry(self, surface, origin, dimensions)
    }

    /// Sets surface bleed insets (10.7).
    pub fn set_surface_bleed(
        &mut self,
        surface: SurfaceId,
        bleed: Bleed,
    ) -> Result<ChangeSet, AubrietaError> {
        SurfacePort::set_surface_bleed(self, surface, bleed)
    }

    /// Sets surface safe margin insets (10.7).
    pub fn set_surface_margins(
        &mut self,
        surface: SurfaceId,
        margins: Margins,
    ) -> Result<ChangeSet, AubrietaError> {
        SurfacePort::set_surface_margins(self, surface, margins)
    }

    /// Sets surface background color/token (10.7).
    pub fn set_surface_background(
        &mut self,
        surface: SurfaceId,
        background: Option<String>,
    ) -> Result<ChangeSet, AubrietaError> {
        SurfacePort::set_surface_background(self, surface, background)
    }

    /// Adds a layout guide to a surface (10.7).
    pub fn add_surface_guide(
        &mut self,
        surface: SurfaceId,
        guide: Guide,
    ) -> Result<ChangeSet, AubrietaError> {
        SurfacePort::add_surface_guide(self, surface, guide)
    }

    /// Removes a layout guide from a surface (10.7).
    pub fn remove_surface_guide(
        &mut self,
        surface: SurfaceId,
        guide_id: u32,
    ) -> Result<ChangeSet, AubrietaError> {
        SurfacePort::remove_surface_guide(self, surface, guide_id)
    }

    /// Moves an object to another surface (10.7).
    pub fn move_object_to_surface(
        &mut self,
        id: ObjectId,
        target_surface: SurfaceId,
        preserve_world_transform: bool,
    ) -> Result<ChangeSet, AubrietaError> {
        SurfacePort::move_object_to_surface(self, id, target_surface, preserve_world_transform)
    }

    /// Registers or imports a variable data source (10.11).
    pub fn import_data_source(
        &mut self,
        source: DataSourceDefinition,
    ) -> Result<ChangeSet, AubrietaError> {
        VariableDataPort::import_data_source(self, source)
    }

    /// Removes a variable data source (10.11).
    pub fn remove_data_source(&mut self, id: DataSourceId) -> Result<ChangeSet, AubrietaError> {
        VariableDataPort::remove_data_source(self, id)
    }

    /// Adds a data binding (10.11).
    pub fn add_data_binding(&mut self, binding: DataBinding) -> Result<ChangeSet, AubrietaError> {
        VariableDataPort::add_data_binding(self, binding)
    }

    /// Removes a data binding (10.11).
    pub fn remove_data_binding(&mut self, id: BindingId) -> Result<ChangeSet, AubrietaError> {
        VariableDataPort::remove_data_binding(self, id)
    }

    /// Materializes merged records into generated surfaces on the pasteboard (10.11).
    pub fn materialize_merge(
        &mut self,
        source_id: DataSourceId,
        template_surface: SurfaceId,
    ) -> Result<ChangeSet, AubrietaError> {
        VariableDataPort::materialize_merge(self, source_id, template_surface)
    }

    /// Resolves the Variable Data presentation model (10.11).
    #[must_use]
    pub fn query_variable_data(&self) -> DataMergePresentationModel {
        VariableDataPort::query_variable_data(self)
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

            for (i, entry) in session.history().undo_entries().iter().enumerate() {
                undo_items.push(HistoryItemViewModel {
                    index: i,
                    description: format!("Operation #{}", i + 1),
                    change_count: entry.len(),
                });
            }

            for (i, entry) in session.history().redo_entries().iter().enumerate() {
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
                can_undo: session.history().can_undo(),
                can_redo: session.history().can_redo(),
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
        self.active_session.as_ref().and_then(|s| s.active_surface())
    }

    fn set_active_surface(&mut self, id: SurfaceId) -> Result<(), AubrietaError> {
        let session = self.session_req_mut()?;
        if session.surfaces().iter().any(|s| s.id == id) {
            session.set_active_surface(id);
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

impl HierarchyPort for AubrietaGuiBridge {
    fn group_objects(
        &mut self,
        surface: SurfaceId,
        group_id: ObjectId,
        child_ids: Vec<ObjectId>,
        role: ContainerRole,
    ) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::GroupObjects {
            surface,
            group_id,
            child_ids,
            role,
        });
        self.submit_command(cmd)
    }

    fn ungroup(&mut self, group_id: ObjectId) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::Ungroup { group_id });
        self.submit_command(cmd)
    }

    fn reparent_object(
        &mut self,
        id: ObjectId,
        new_parent: Option<ObjectId>,
        target_index: usize,
        preserve_world_transform: bool,
    ) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::ReparentObject {
            id,
            new_parent,
            target_index,
            preserve_world_transform,
        });
        self.submit_command(cmd)
    }

    fn create_clip_group(
        &mut self,
        surface: SurfaceId,
        group_id: ObjectId,
        mask_id: ObjectId,
        content_ids: Vec<ObjectId>,
    ) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::CreateClipGroup {
            surface,
            group_id,
            mask_id,
            content_ids,
        });
        self.submit_command(cmd)
    }

    fn release_clip_group(&mut self, group_id: ObjectId) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::ReleaseClipGroup { group_id });
        self.submit_command(cmd)
    }
}

impl SurfacePort for AubrietaGuiBridge {
    fn set_surface_geometry(
        &mut self,
        surface: SurfaceId,
        origin: [f64; 2],
        dimensions: [f64; 2],
    ) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::SetSurfaceGeometry {
            surface,
            origin,
            dimensions,
        });
        self.submit_command(cmd)
    }

    fn set_surface_bleed(
        &mut self,
        surface: SurfaceId,
        bleed: Bleed,
    ) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::SetSurfaceBleed { surface, bleed });
        self.submit_command(cmd)
    }

    fn set_surface_margins(
        &mut self,
        surface: SurfaceId,
        margins: Margins,
    ) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::SetSurfaceMargins { surface, margins });
        self.submit_command(cmd)
    }

    fn set_surface_background(
        &mut self,
        surface: SurfaceId,
        background: Option<String>,
    ) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::SetSurfaceBackground {
            surface,
            background,
        });
        self.submit_command(cmd)
    }

    fn add_surface_guide(
        &mut self,
        surface: SurfaceId,
        guide: Guide,
    ) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::AddGuide { surface, guide });
        self.submit_command(cmd)
    }

    fn remove_surface_guide(
        &mut self,
        surface: SurfaceId,
        guide_id: u32,
    ) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::RemoveGuide { surface, guide_id });
        self.submit_command(cmd)
    }

    fn move_object_to_surface(
        &mut self,
        id: ObjectId,
        target_surface: SurfaceId,
        preserve_world_transform: bool,
    ) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::MoveObjectToSurface {
            id,
            target_surface,
            preserve_world_transform,
        });
        self.submit_command(cmd)
    }
}

impl VariableDataPort for AubrietaGuiBridge {
    fn import_data_source(
        &mut self,
        source: DataSourceDefinition,
    ) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::AddDataSource { source });
        self.submit_command(cmd)
    }

    fn remove_data_source(&mut self, id: DataSourceId) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::RemoveDataSource { id });
        self.submit_command(cmd)
    }

    fn add_data_binding(&mut self, binding: DataBinding) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::AddDataBinding { binding });
        self.submit_command(cmd)
    }

    fn remove_data_binding(&mut self, id: BindingId) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::RemoveDataBinding { id });
        self.submit_command(cmd)
    }

    fn materialize_merge(
        &mut self,
        source_id: DataSourceId,
        template_surface: SurfaceId,
    ) -> Result<ChangeSet, AubrietaError> {
        let cmd = CommandRequest::new(Command::MaterializeDataMerge {
            source_id,
            template_surface,
        });
        self.submit_command(cmd)
    }

    fn query_variable_data(&self) -> DataMergePresentationModel {
        self.active_session
            .as_ref()
            .map_or_else(DataMergePresentationModel::default, |s| {
                s.data_merge_presentation_model()
            })
    }
}
