//! Canonical PetuniaDesignGuiBridge application facade (09.27).
//!
//! Exposes a coarse-grained, UI-agnostic application facade over document sessions.
//! UI toolkits (GPUI, Floem, tests, MockGuiAdapter) interact exclusively through this bridge.

use std::collections::HashMap;

use petunia_design_application::{
    ActionId, ActionRequest, CapabilityRegistry, Command, CommandRequest,
};
use petunia_design_document::{
    AppearanceStack, BindingId, Bleed, ChangeSet, ContainerRole, DataBinding, DataSourceDefinition,
    DataSourceId, Document, Guide, Margins, ShapeKind,
};
use petunia_design_foundation::{ObjectId, PetuniaError, SurfaceId};
use petunia_design_geometry::BooleanOp;

use petunia_design_application::menus::{self, ActionContext};
use petunia_design_application::ports::{
    ActionQueryPort, CommandPort, DocumentQueryPort, HierarchyPort, InspectionPort, PropertyPort,
    SelectionPort, SurfacePort, VariableDataPort,
};
use petunia_design_application::session::DocumentSession;
use petunia_design_application::view_models::{
    ActionStateMap, ActionStateViewModel, DataMergePresentationModel, DocumentSummary,
    HistoryItemViewModel, HistoryPresentationModel, LayersPresentationModel,
    PropertiesPresentationModel, SelectionViewModel, SessionSnapshot,
};
use petunia_design_resources::i18n::{Locale, LocalizationService};

use petunia_design_application::tools::ToolKind;

use crate::context_toolbar;
use crate::menu::{self, MenuBarPresentationModel, MenuItemPresentation};

/// Coarse-grained facade connecting external UI adapters to the Petunia engine.
#[derive(Debug)]
pub struct PetuniaDesignGuiBridge {
    /// Active document session, if any.
    active_session: Option<DocumentSession>,
    /// Global capability registry for tool/command authorization.
    capabilities: CapabilityRegistry,
    /// Localization service carrying the canonical shell/menu catalog (09.16).
    localization: LocalizationService,
    /// Active UI locale. Canonical source is `en-US`; the UI sets the product
    /// default, the bridge does not guess one (12.6).
    locale: Locale,
    /// Persona the shell is in (15.G). Always a registered persona id: the
    /// shell switches by id, never by a UI-local index.
    active_persona: &'static str,
    /// User order and visibility of the context toolbar. The catalog stays the
    /// source of which entries exist; this only arranges them.
    toolbar_layout: context_toolbar::ToolbarLayout,
}

impl Default for PetuniaDesignGuiBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl PetuniaDesignGuiBridge {
    /// Creates a fresh GUI bridge with core capabilities initialized.
    #[must_use]
    pub fn new() -> Self {
        let mut capabilities = CapabilityRegistry::new();
        // Register core capabilities with explicit states
        capabilities.register(petunia_design_application::CapabilityInfo::available(
            "ptnd.document.edit",
            "petunia_design_shell",
        ));
        capabilities.register(petunia_design_application::CapabilityInfo::available(
            "ptnd.export.vector",
            "petunia_design_io",
        ));
        capabilities.register(petunia_design_application::CapabilityInfo::available(
            "ptnd.export.raster",
            "petunia_design_io",
        ));
        capabilities.register(petunia_design_application::CapabilityInfo::available(
            "ptnd.color.proof",
            "petunia_design_color",
        ));

        Self {
            active_session: None,
            capabilities,
            localization: LocalizationService::with_shell_catalog(),
            locale: Locale::EnUs,
            active_persona: petunia_design_application::surfaces::PERSONA_VECTOR,
            toolbar_layout: context_toolbar::ToolbarLayout::canonical(),
        }
    }

    /// Switches the UI locale for every resolved label (12.6).
    pub fn set_locale(&mut self, locale: Locale) {
        self.locale = locale;
    }

    /// The active UI locale.
    #[must_use]
    pub fn locale(&self) -> &Locale {
        &self.locale
    }

    /// The persona the shell is currently in (15.G).
    #[must_use]
    pub fn persona(&self) -> &'static str {
        self.active_persona
    }

    /// Switches persona, by registered id.
    ///
    /// Returns `false` for an id the registry does not know and for one that is
    /// not a persona, so the shell cannot invent a mode the registry has never
    /// declared.
    pub fn set_persona(&mut self, persona: &str) -> bool {
        let Some(entry) = petunia_design_application::surfaces::surface(persona) else {
            return false;
        };
        if entry.kind != petunia_design_application::surfaces::SurfaceKind::Persona {
            return false;
        }
        self.active_persona = entry.id;
        true
    }

    /// Every persona the switcher offers, resolved for the active locale.
    #[must_use]
    pub fn personas(&self) -> Vec<menu::PersonaPresentation> {
        menu::present_personas(&self.localization, &self.locale)
    }

    /// The one-line hint describing what the given persona is for.
    #[must_use]
    pub fn persona_hint(&self, persona: &str) -> Option<String> {
        menu::persona_hint_text_id(persona).map(|id| self.localization.text(id, &self.locale))
    }

    /// The localization service resolving `ptnd.text.*` ids.
    #[must_use]
    pub fn localization(&self) -> &LocalizationService {
        &self.localization
    }

    /// Session facts that decide what the menu and palette can offer (15.G).
    #[must_use]
    pub fn action_context(&self) -> ActionContext {
        match &self.active_session {
            Some(session) => ActionContext {
                has_document: true,
                selection_count: session.selection.selected_ids.len(),
                can_undo: session.history().can_undo(),
                can_redo: session.history().can_redo(),
                is_dirty: session.is_dirty(),
                command_palette_open: session.view.command_palette_open,
                persona: self.active_persona,
            },
            None => ActionContext {
                has_document: false,
                command_palette_open: false,
                persona: self.active_persona,
                ..ActionContext::default()
            },
        }
    }

    /// Registry-driven menu bar with localized labels (15.G, 08.2).
    ///
    /// The structure comes from `petunia_design_application::menus`; this text
    /// is the only thing the shell adds.
    #[must_use]
    pub fn query_menu_bar(&self) -> MenuBarPresentationModel {
        let families = menus::menu_bar(&self.action_context());
        menu::present_menu_bar(&families, &self.localization, &self.locale)
    }

    /// Command palette index: exactly the currently enabled menu items.
    #[must_use]
    pub fn query_command_index(&self) -> Vec<MenuItemPresentation> {
        menu::present_command_index(&self.localization, &self.locale, &self.action_context())
    }

    /// The centred shell control cluster, resolved for the active locale.
    #[must_use]
    pub fn query_shell_controls(&self) -> Vec<menu::ShellControlPresentation> {
        menu::present_shell_controls(&self.localization, &self.locale, &self.action_context())
    }

    /// Rows of the popup under the zoom readout (08.2). Same registry items the
    /// View submenu shows, resolved for the active locale.
    #[must_use]
    pub fn query_zoom_levels(&self) -> Vec<MenuItemPresentation> {
        menu::present_zoom_levels(&self.localization, &self.locale, &self.action_context())
    }

    /// The localized label of any registry surface, by id.
    #[must_use]
    pub fn surface_label(&self, id: &str) -> Option<String> {
        menu::surface_label(&self.localization, &self.locale, id)
    }

    /// The canonical context toolbar, resolved for the active tool (08.23).
    ///
    /// Contextuality lives here, not in the UI: entries that do not apply to
    /// this tool are absent from the answer, and entries that apply but cannot
    /// run carry the localized reason they cannot.
    #[must_use]
    pub fn query_context_toolbar(
        &self,
        tool: ToolKind,
        has_selection: bool,
    ) -> Vec<context_toolbar::ToolbarItemPresentation> {
        context_toolbar::present_layout(
            &self.toolbar_layout,
            &self.localization,
            &self.locale,
            &self.action_context(),
            tool,
            has_selection,
        )
    }

    /// The context toolbar as the customization dialog edits it: every slot,
    /// hidden ones included, with catalog labels.
    #[must_use]
    pub fn query_toolbar_catalog(&self) -> Vec<context_toolbar::ToolbarCatalogRow> {
        context_toolbar::present_catalog(&self.toolbar_layout, &self.localization, &self.locale)
    }

    /// Shows or hides one catalog entry. Returns false when the id is unknown
    /// or the entry may not be hidden (the spacer).
    pub fn toolbar_set_visible(&mut self, id: &str, visible: bool) -> bool {
        self.toolbar_layout.set_visible(id, visible)
    }

    /// Shows or hides the slot at `index`. User dividers share an empty id, so
    /// the dialog addresses them by place, not by id.
    pub fn toolbar_set_slot_visible(&mut self, index: usize, visible: bool) -> bool {
        self.toolbar_layout.set_slot_visible(index, visible)
    }

    /// Moves one slot by `delta` places. Negative moves toward the start.
    pub fn toolbar_move(&mut self, index: usize, delta: i32) -> bool {
        self.toolbar_layout.move_slot(index, delta)
    }

    /// Inserts a user divider after `after`. `None` appends.
    pub fn toolbar_insert_divider(&mut self, after: Option<usize>) {
        self.toolbar_layout.insert_divider(after);
    }

    /// Removes a user-inserted divider. A catalog entry cannot be removed.
    pub fn toolbar_remove_divider(&mut self, index: usize) -> bool {
        self.toolbar_layout.remove_divider(index)
    }

    /// Restores the catalog order, every entry visible.
    pub fn toolbar_reset(&mut self) {
        self.toolbar_layout = context_toolbar::ToolbarLayout::canonical();
    }

    /// Initializes a new empty document session with default surface (A1).
    /// The initial surface goes through the command lane so undo, history
    /// and revision stay exact from the first mutation.
    pub fn new_document(&mut self, title: impl Into<String>) -> Result<(), PetuniaError> {
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
    ) -> Result<(), PetuniaError> {
        let mut session = DocumentSession::with_document(title, document);
        if session.active_surface().is_none() {
            if let Some(first) = session.surfaces().first() {
                session.set_active_surface(first.id);
            }
        }
        self.active_session = Some(session);
        Ok(())
    }

    /// Opens a project from disk, replacing the active session.
    ///
    /// A legacy package decodes into a canonical document but deliberately
    /// records **no path**: the next save must go through Save As so the
    /// original `.aubrieta`/`.aubri` file is never overwritten (15.A).
    pub fn open_path(&mut self, path: &std::path::Path) -> Result<(), PetuniaError> {
        let opened = petunia_design_io::open_package(path)?;
        let title = path.file_name().map_or_else(
            || "Untitled".to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        let mut session = DocumentSession::with_document(title, opened.document);
        if session.active_surface().is_none() {
            if let Some(first) = session.surfaces().first() {
                session.set_active_surface(first.id);
            }
        }
        if opened.format == petunia_design_io::PackageFormat::Ptnd {
            session.adopt_path(path.to_path_buf());
        }
        self.active_session = Some(session);
        Ok(())
    }

    /// Closes the active session, checking unsaved dirty state.
    pub fn close_session(&mut self, force: bool) -> Result<bool, PetuniaError> {
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

    /// Mutable access to the active session's **view** state only.
    ///
    /// View state (camera, rulers, snapping, palette) is not document state:
    /// it does not travel the command lane, and the `document`/`history`
    /// fields stay private so this cannot become a mutation back door (A2).
    pub fn view_state_mut(
        &mut self,
    ) -> Option<&mut petunia_design_application::view_camera::ViewState> {
        self.active_session
            .as_mut()
            .map(|session| &mut session.view)
    }

    /// Returns a reference to the global capability registry.
    #[must_use]
    pub fn capabilities(&self) -> &CapabilityRegistry {
        &self.capabilities
    }

    /// Helper to borrow active session or return error if closed.
    #[allow(dead_code)]
    fn session_req(&self) -> Result<&DocumentSession, PetuniaError> {
        self.active_session
            .as_ref()
            .ok_or_else(|| PetuniaError::invalid_input("no active document session"))
    }

    /// Helper to mutably borrow active session or return error if closed.
    fn session_req_mut(&mut self) -> Result<&mut DocumentSession, PetuniaError> {
        self.active_session
            .as_mut()
            .ok_or_else(|| PetuniaError::invalid_input("no active document session"))
    }

    /// Allocates the next unique object ID in the active session.
    pub fn next_object_id(&mut self) -> Result<ObjectId, PetuniaError> {
        self.session_req_mut().map(|s| s.next_object_id())
    }

    /// Allocates the next unique surface ID in the active session.
    pub fn next_surface_id(&mut self) -> Result<SurfaceId, PetuniaError> {
        self.session_req_mut().map(|s| s.next_surface_id())
    }

    /// Sets the currently active surface.
    pub fn set_active_surface(&mut self, surface: SurfaceId) -> Result<(), PetuniaError> {
        InspectionPort::set_active_surface(self, surface)
    }

    /// Submits a validated command request.
    pub fn submit_command(&mut self, request: CommandRequest) -> Result<ChangeSet, PetuniaError> {
        CommandPort::submit_command(self, request)
    }

    /// Submits a batch of commands atomically as one undo entry (F-01).
    /// Creation gestures build their list via
    /// `petunia_design_application::creation` and commit here.
    pub fn submit_all(
        &mut self,
        label: &str,
        commands: Vec<Command>,
    ) -> Result<ChangeSet, PetuniaError> {
        let session = self.session_req_mut()?;
        session.transact(label, commands)
    }

    /// Undoes the last committed command.
    pub fn undo(&mut self) -> Result<bool, PetuniaError> {
        CommandPort::undo(self)
    }

    /// Redoes the last undone command.
    pub fn redo(&mut self) -> Result<bool, PetuniaError> {
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
    ) -> Result<ChangeSet, PetuniaError> {
        PropertyPort::set_fill(self, id, fill)
    }

    /// Sets stroke.
    pub fn set_stroke(
        &mut self,
        id: ObjectId,
        stroke: Option<String>,
        width: f64,
    ) -> Result<ChangeSet, PetuniaError> {
        PropertyPort::set_stroke(self, id, stroke, width)
    }

    /// Sets opacity.
    pub fn set_opacity(&mut self, id: ObjectId, opacity: f64) -> Result<ChangeSet, PetuniaError> {
        PropertyPort::set_opacity(self, id, opacity)
    }

    /// Sets visibility.
    pub fn set_visibility(
        &mut self,
        id: ObjectId,
        visible: bool,
    ) -> Result<ChangeSet, PetuniaError> {
        PropertyPort::set_visibility(self, id, visible)
    }

    /// Sets locked.
    pub fn set_locked(&mut self, id: ObjectId, locked: bool) -> Result<ChangeSet, PetuniaError> {
        PropertyPort::set_locked(self, id, locked)
    }

    /// Sets bounds.
    pub fn set_bounds(
        &mut self,
        id: ObjectId,
        bounds: Option<[f64; 4]>,
        rotation: f64,
    ) -> Result<ChangeSet, PetuniaError> {
        PropertyPort::set_bounds(self, id, bounds, rotation)
    }

    /// Sets appearance stack.
    pub fn set_appearance(
        &mut self,
        id: ObjectId,
        appearance: Option<AppearanceStack>,
    ) -> Result<ChangeSet, PetuniaError> {
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
    ) -> Result<ChangeSet, PetuniaError> {
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
    ) -> Result<ChangeSet, PetuniaError> {
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
    ) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::ApplyBoolean {
            surface,
            target_id,
            subject_id,
            clip_id,
            op,
        }))
    }

    /// Converts a parametric shape or text object to an editable vector path (10.3, 10.6).
    pub fn convert_to_curves(&mut self, id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::ConvertToCurves { id }))
    }

    /// Bakes corner geometry into an explicit vector path (10.2, 10.3).
    pub fn bake_corners(&mut self, id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::BakeCorners { id }))
    }

    /// Offsets a path or object bounds outward or inward (10.3).
    pub fn offset_path(&mut self, id: ObjectId, delta: f64) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::OffsetPath { id, delta }))
    }

    /// Sets the blend mode of an object (10.4, F-18).
    pub fn set_blend_mode(
        &mut self,
        id: ObjectId,
        blend_mode: petunia_design_document::BlendMode,
    ) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::SetStackBlend {
            id,
            blend_mode,
        }))
    }

    /// Adds a fill layer to the object's appearance stack (10.4, F-18).
    pub fn add_fill(
        &mut self,
        id: ObjectId,
        paint: petunia_design_document::Paint,
    ) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::AddFill {
            id,
            fill: petunia_design_document::FillItem {
                id: 0,
                paint,
                opacity: 1.0,
                blend_mode: petunia_design_document::BlendMode::Normal,
                visible: true,
            },
        }))
    }

    /// Removes a fill layer by its ID (10.4, F-18).
    pub fn remove_fill(&mut self, id: ObjectId, fill_id: u32) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::RemoveFill { id, fill_id }))
    }

    /// Adds a stroke layer to the object's appearance stack (10.4, F-18).
    pub fn add_stroke(
        &mut self,
        id: ObjectId,
        paint: petunia_design_document::Paint,
        width: f64,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut stroke_item = petunia_design_document::StrokeItem::solid(0, "ptnd.gray/700", width);
        stroke_item.paint = paint;
        self.submit_command(CommandRequest::new(Command::AddStroke {
            id,
            stroke: stroke_item,
        }))
    }

    /// Removes a stroke layer by its ID (10.4, F-18).
    pub fn remove_stroke(
        &mut self,
        id: ObjectId,
        stroke_id: u32,
    ) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::RemoveStroke { id, stroke_id }))
    }

    /// Sets linear gradient fill with two stops on the object (10.4).
    /// Secondary entries are preserved: the gradient applies to the primary
    /// fill (or appends one when the stack is empty).
    pub fn set_linear_gradient_fill(
        &mut self,
        id: ObjectId,
        color1: impl Into<String>,
        color2: impl Into<String>,
    ) -> Result<ChangeSet, PetuniaError> {
        let stack = self
            .active_session
            .as_ref()
            .and_then(|s| s.find_object(id))
            .map(|o| o.effective_appearance())
            .unwrap_or_default();
        let paint = petunia_design_document::Paint::LinearGradient(
            petunia_design_document::LinearGradient::new(
                [0.0, 0.0],
                [1.0, 0.0],
                vec![
                    petunia_design_document::GradientStop::new(0.0, color1),
                    petunia_design_document::GradientStop::new(1.0, color2),
                ],
            ),
        );
        let stack =
            petunia_design_application::appearance_service::with_primary_gradient(stack, paint);
        self.submit_command(CommandRequest::new(Command::SetAppearance {
            id,
            appearance: Some(stack),
        }))
    }

    /// Sets radial gradient fill with two stops on the object (10.4).
    /// Secondary entries are preserved (see `set_linear_gradient_fill`).
    pub fn set_radial_gradient_fill(
        &mut self,
        id: ObjectId,
        color1: impl Into<String>,
        color2: impl Into<String>,
    ) -> Result<ChangeSet, PetuniaError> {
        let stack = self
            .active_session
            .as_ref()
            .and_then(|s| s.find_object(id))
            .map(|o| o.effective_appearance())
            .unwrap_or_default();
        let paint = petunia_design_document::Paint::RadialGradient(
            petunia_design_document::RadialGradient::new(
                [0.5, 0.5],
                0.5,
                vec![
                    petunia_design_document::GradientStop::new(0.0, color1),
                    petunia_design_document::GradientStop::new(1.0, color2),
                ],
            ),
        );
        let stack =
            petunia_design_application::appearance_service::with_primary_gradient(stack, paint);
        self.submit_command(CommandRequest::new(Command::SetAppearance {
            id,
            appearance: Some(stack),
        }))
    }

    /// Aligns multiple objects relative to their collective bounds (10.1).
    pub fn align_objects(
        &mut self,
        surface: SurfaceId,
        ids: Vec<ObjectId>,
        mode: petunia_design_document::AlignmentMode,
    ) -> Result<ChangeSet, PetuniaError> {
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
        axis: petunia_design_document::DistributionAxis,
    ) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::DistributeObjects {
            surface,
            ids,
            axis,
        }))
    }

    /// Slices or splits a path object at a specific point (10.2).
    pub fn slice_path(&mut self, id: ObjectId, point: [f64; 2]) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::SlicePath { id, point }))
    }

    /// Groups objects into a container with a designated role (10.5 One-Tree).
    pub fn group_objects(
        &mut self,
        surface: SurfaceId,
        group_id: ObjectId,
        child_ids: Vec<ObjectId>,
        role: ContainerRole,
    ) -> Result<ChangeSet, PetuniaError> {
        HierarchyPort::group_objects(self, surface, group_id, child_ids, role)
    }

    /// Ungroups a container object.
    pub fn ungroup(&mut self, group_id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        HierarchyPort::ungroup(self, group_id)
    }

    /// Reparents an object to a new container or root.
    pub fn reparent_object(
        &mut self,
        id: ObjectId,
        new_parent: Option<ObjectId>,
        target_index: usize,
        preserve_world_transform: bool,
    ) -> Result<ChangeSet, PetuniaError> {
        HierarchyPort::reparent_object(self, id, new_parent, target_index, preserve_world_transform)
    }

    /// Creates a clipping mask group.
    pub fn create_clip_group(
        &mut self,
        surface: SurfaceId,
        group_id: ObjectId,
        mask_id: ObjectId,
        content_ids: Vec<ObjectId>,
    ) -> Result<ChangeSet, PetuniaError> {
        HierarchyPort::create_clip_group(self, surface, group_id, mask_id, content_ids)
    }

    /// Releases a clipping mask group.
    pub fn release_clip_group(&mut self, group_id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        HierarchyPort::release_clip_group(self, group_id)
    }

    /// Sets surface origin and dimensions (10.7).
    pub fn set_surface_geometry(
        &mut self,
        surface: SurfaceId,
        origin: [f64; 2],
        dimensions: [f64; 2],
    ) -> Result<ChangeSet, PetuniaError> {
        SurfacePort::set_surface_geometry(self, surface, origin, dimensions)
    }

    /// Sets surface bleed insets (10.7).
    pub fn set_surface_bleed(
        &mut self,
        surface: SurfaceId,
        bleed: Bleed,
    ) -> Result<ChangeSet, PetuniaError> {
        SurfacePort::set_surface_bleed(self, surface, bleed)
    }

    /// Sets surface safe margin insets (10.7).
    pub fn set_surface_margins(
        &mut self,
        surface: SurfaceId,
        margins: Margins,
    ) -> Result<ChangeSet, PetuniaError> {
        SurfacePort::set_surface_margins(self, surface, margins)
    }

    /// Sets surface background color/token (10.7).
    pub fn set_surface_background(
        &mut self,
        surface: SurfaceId,
        background: Option<String>,
    ) -> Result<ChangeSet, PetuniaError> {
        SurfacePort::set_surface_background(self, surface, background)
    }

    /// Adds a layout guide to a surface (10.7).
    pub fn add_surface_guide(
        &mut self,
        surface: SurfaceId,
        guide: Guide,
    ) -> Result<ChangeSet, PetuniaError> {
        SurfacePort::add_surface_guide(self, surface, guide)
    }

    /// Removes a layout guide from a surface (10.7).
    pub fn remove_surface_guide(
        &mut self,
        surface: SurfaceId,
        guide_id: u32,
    ) -> Result<ChangeSet, PetuniaError> {
        SurfacePort::remove_surface_guide(self, surface, guide_id)
    }

    /// Moves an object to another surface (10.7).
    pub fn move_object_to_surface(
        &mut self,
        id: ObjectId,
        target_surface: SurfaceId,
        preserve_world_transform: bool,
    ) -> Result<ChangeSet, PetuniaError> {
        SurfacePort::move_object_to_surface(self, id, target_surface, preserve_world_transform)
    }

    /// Registers or imports a variable data source (10.11).
    pub fn import_data_source(
        &mut self,
        source: DataSourceDefinition,
    ) -> Result<ChangeSet, PetuniaError> {
        VariableDataPort::import_data_source(self, source)
    }

    /// Removes a variable data source (10.11).
    pub fn remove_data_source(&mut self, id: DataSourceId) -> Result<ChangeSet, PetuniaError> {
        VariableDataPort::remove_data_source(self, id)
    }

    /// Adds a data binding (10.11).
    pub fn add_data_binding(&mut self, binding: DataBinding) -> Result<ChangeSet, PetuniaError> {
        VariableDataPort::add_data_binding(self, binding)
    }

    /// Removes a data binding (10.11).
    pub fn remove_data_binding(&mut self, id: BindingId) -> Result<ChangeSet, PetuniaError> {
        VariableDataPort::remove_data_binding(self, id)
    }

    /// Materializes merged records into generated surfaces on the pasteboard (10.11).
    pub fn materialize_merge(
        &mut self,
        source_id: DataSourceId,
        template_surface: SurfaceId,
    ) -> Result<ChangeSet, PetuniaError> {
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
    pub fn dispatch_action(&mut self, request: ActionRequest) -> Result<ChangeSet, PetuniaError> {
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

impl ActionQueryPort for PetuniaDesignGuiBridge {
    fn query_actions(&self) -> ActionStateMap {
        // Derived from the same registry rule table the menu uses, so a menu
        // item and an action query can never disagree about availability.
        let context = self.action_context();
        let mut states: HashMap<String, ActionStateViewModel> = HashMap::new();
        for family in menus::menu_bar(&context) {
            for item in family.items() {
                // One action may own several parameterized items (align,
                // boolean): the action is invokable when any variant is.
                let reason = item
                    .disabled_reason_id
                    .as_deref()
                    .map(|id| self.localization.text(id, &self.locale));
                let entry =
                    states
                        .entry(item.action_id.clone())
                        .or_insert_with(|| ActionStateViewModel {
                            action_id: ActionId::new(item.action_id.clone()),
                            is_enabled: false,
                            is_checked: false,
                            disabled_reason: reason,
                        });
                if item.enabled {
                    entry.is_enabled = true;
                    entry.disabled_reason = None;
                }
            }
        }
        ActionStateMap { states }
    }

    fn is_action_enabled(&self, action: &ActionId) -> bool {
        let map = self.query_actions();
        map.get(action).is_some_and(|s| s.is_enabled)
    }

    fn dispatch_action(&mut self, request: ActionRequest) -> Result<ChangeSet, PetuniaError> {
        // Document-lifecycle actions replace the session, so they are handled
        // by the host before the active session is even borrowed.
        let normalized = petunia_design_foundation::normalized(request.action.as_str());
        let action = petunia_design_foundation::normalize_action_id(&normalized);
        match action.as_str() {
            "ptnd.action.file.new" => {
                self.new_document("Untitled")?;
                return Ok(ChangeSet::empty());
            }
            "ptnd.action.file.open" => {
                let path = request
                    .payload
                    .get("path")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| {
                        PetuniaError::invalid_input(
                            "file.open requires a non-empty `path` payload field",
                        )
                    })?;
                self.open_path(std::path::Path::new(path))?;
                return Ok(ChangeSet::empty());
            }
            _ => {}
        }
        let session = self.session_req_mut()?;
        session.dispatch_action(request)
    }
}

impl CommandPort for PetuniaDesignGuiBridge {
    fn submit_command(&mut self, request: CommandRequest) -> Result<ChangeSet, PetuniaError> {
        let session = self.session_req_mut()?;
        session.execute_command(request)
    }

    fn undo(&mut self) -> Result<bool, PetuniaError> {
        let session = self.session_req_mut()?;
        session.undo()
    }

    fn redo(&mut self) -> Result<bool, PetuniaError> {
        let session = self.session_req_mut()?;
        session.redo()
    }

    fn can_undo(&self) -> bool {
        self.active_session
            .as_ref()
            .is_some_and(|s| s.history().can_undo())
    }

    fn can_redo(&self) -> bool {
        self.active_session
            .as_ref()
            .is_some_and(|s| s.history().can_redo())
    }
}

impl PropertyPort for PetuniaDesignGuiBridge {
    fn query_properties(&self) -> PropertiesPresentationModel {
        self.active_session
            .as_ref()
            .map_or_else(PropertiesPresentationModel::default, |s| {
                s.properties_presentation_model()
            })
    }

    fn set_fill(&mut self, id: ObjectId, fill: Option<String>) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::SetFill { id, fill });
        self.submit_command(cmd)
    }

    fn set_stroke(
        &mut self,
        id: ObjectId,
        stroke: Option<String>,
        width: f64,
    ) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::SetStroke { id, stroke, width });
        self.submit_command(cmd)
    }

    fn set_opacity(&mut self, id: ObjectId, opacity: f64) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::SetOpacity { id, opacity });
        self.submit_command(cmd)
    }

    fn set_visibility(&mut self, id: ObjectId, visible: bool) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::SetVisibility { id, visible });
        self.submit_command(cmd)
    }

    fn set_locked(&mut self, id: ObjectId, locked: bool) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::SetLocked { id, locked });
        self.submit_command(cmd)
    }

    fn set_bounds(
        &mut self,
        id: ObjectId,
        bounds: Option<[f64; 4]>,
        rotation: f64,
    ) -> Result<ChangeSet, PetuniaError> {
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
        appearance: Option<petunia_design_document::AppearanceStack>,
    ) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::SetAppearance { id, appearance });
        self.submit_command(cmd)
    }
}

impl DocumentQueryPort for PetuniaDesignGuiBridge {
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

impl SelectionPort for PetuniaDesignGuiBridge {
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

impl InspectionPort for PetuniaDesignGuiBridge {
    fn active_surface(&self) -> Option<SurfaceId> {
        self.active_session
            .as_ref()
            .and_then(|s| s.active_surface())
    }

    fn set_active_surface(&mut self, id: SurfaceId) -> Result<(), PetuniaError> {
        let session = self.session_req_mut()?;
        if session.surfaces().iter().any(|s| s.id == id) {
            session.set_active_surface(id);
            Ok(())
        } else {
            Err(PetuniaError::not_found(format!(
                "surface `{id}` not found in document"
            )))
        }
    }

    fn revision(&self) -> u64 {
        self.active_session
            .as_ref()
            .map_or(0, |s| s.current_revision())
    }

    fn is_dirty(&self) -> bool {
        self.active_session.as_ref().is_some_and(|s| s.is_dirty())
    }
}

impl HierarchyPort for PetuniaDesignGuiBridge {
    fn group_objects(
        &mut self,
        surface: SurfaceId,
        group_id: ObjectId,
        child_ids: Vec<ObjectId>,
        role: ContainerRole,
    ) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::GroupObjects {
            surface,
            group_id,
            child_ids,
            role,
        });
        self.submit_command(cmd)
    }

    fn ungroup(&mut self, group_id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::Ungroup { group_id });
        self.submit_command(cmd)
    }

    fn reparent_object(
        &mut self,
        id: ObjectId,
        new_parent: Option<ObjectId>,
        target_index: usize,
        preserve_world_transform: bool,
    ) -> Result<ChangeSet, PetuniaError> {
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
    ) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::CreateClipGroup {
            surface,
            group_id,
            mask_id,
            content_ids,
        });
        self.submit_command(cmd)
    }

    fn release_clip_group(&mut self, group_id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::ReleaseClipGroup { group_id });
        self.submit_command(cmd)
    }
}

impl SurfacePort for PetuniaDesignGuiBridge {
    fn set_surface_geometry(
        &mut self,
        surface: SurfaceId,
        origin: [f64; 2],
        dimensions: [f64; 2],
    ) -> Result<ChangeSet, PetuniaError> {
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
    ) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::SetSurfaceBleed { surface, bleed });
        self.submit_command(cmd)
    }

    fn set_surface_margins(
        &mut self,
        surface: SurfaceId,
        margins: Margins,
    ) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::SetSurfaceMargins { surface, margins });
        self.submit_command(cmd)
    }

    fn set_surface_background(
        &mut self,
        surface: SurfaceId,
        background: Option<String>,
    ) -> Result<ChangeSet, PetuniaError> {
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
    ) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::AddGuide { surface, guide });
        self.submit_command(cmd)
    }

    fn remove_surface_guide(
        &mut self,
        surface: SurfaceId,
        guide_id: u32,
    ) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::RemoveGuide { surface, guide_id });
        self.submit_command(cmd)
    }

    fn move_object_to_surface(
        &mut self,
        id: ObjectId,
        target_surface: SurfaceId,
        preserve_world_transform: bool,
    ) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::MoveObjectToSurface {
            id,
            target_surface,
            preserve_world_transform,
        });
        self.submit_command(cmd)
    }
}

impl VariableDataPort for PetuniaDesignGuiBridge {
    fn import_data_source(
        &mut self,
        source: DataSourceDefinition,
    ) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::AddDataSource { source });
        self.submit_command(cmd)
    }

    fn remove_data_source(&mut self, id: DataSourceId) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::RemoveDataSource { id });
        self.submit_command(cmd)
    }

    fn add_data_binding(&mut self, binding: DataBinding) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::AddDataBinding { binding });
        self.submit_command(cmd)
    }

    fn remove_data_binding(&mut self, id: BindingId) -> Result<ChangeSet, PetuniaError> {
        let cmd = CommandRequest::new(Command::RemoveDataBinding { id });
        self.submit_command(cmd)
    }

    fn materialize_merge(
        &mut self,
        source_id: DataSourceId,
        template_surface: SurfaceId,
    ) -> Result<ChangeSet, PetuniaError> {
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
