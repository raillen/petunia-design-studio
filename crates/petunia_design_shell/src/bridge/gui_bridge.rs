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
    /// Open document sessions in tab order.
    sessions: Vec<DocumentSession>,
    /// Active tab index, or `None` when there are no open sessions.
    active_index: Option<usize>,
    /// Full-scene snapshot cache keyed by `(doc_revision, selection_version,
    /// surface)`. Cleared whenever the session itself is replaced or closed:
    /// stable ids from a previous document must never resolve into the new
    /// one through a stale key.
    snapshot_cache: std::cell::RefCell<crate::canvas::SnapshotCache>,
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
    /// Application clipboard shared across document tabs; artwork resources remain immutable.
    clipboard: Vec<petunia_design_document::DocumentObject>,
    clipboard_origin: [f64; 2],
    /// Background jobs tracker (renders, bakes, exports, indexing).
    jobs: petunia_design_jobs::JobManager,
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

        capabilities.register(petunia_design_application::CapabilityInfo::available(
            "ptnd.export.pdf",
            "petunia_design_io",
        ));
        capabilities.register(petunia_design_application::CapabilityInfo::disabled("ptnd.export.pdf-x4", "petunia_design_io", "PDF/X-4 OutputIntent, float process paints, overprint and independent print validation are not complete"));
        capabilities.register(petunia_design_application::CapabilityInfo::available(
            "ptnd.raster.cmyk",
            "petunia_design_raster",
        ));
        capabilities.register(petunia_design_application::CapabilityInfo::available(
            "ptnd.export.cmyk-tiff-layer",
            "petunia_design_io",
        ));
        capabilities.register(petunia_design_application::CapabilityInfo::disabled("ptnd.color.native-ink-proof", "petunia_design_render", "whole-page direct ink proof/overprint composition is not complete; RGB-derived proof does not preserve process separations"));

        Self {
            sessions: Vec::new(),
            clipboard: Vec::new(),
            clipboard_origin: [0., 0.],
            active_index: None,
            snapshot_cache: std::cell::RefCell::new(crate::canvas::SnapshotCache::new()),
            capabilities,
            localization: LocalizationService::with_shell_catalog(),
            locale: Locale::EnUs,
            active_persona: petunia_design_application::surfaces::PERSONA_VECTOR,
            toolbar_layout: context_toolbar::ToolbarLayout::canonical(),
            jobs: petunia_design_jobs::JobManager::new(),
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

    /// Background jobs tracker (renders, bakes, exports, indexing).
    #[must_use]
    pub fn jobs(&self) -> &petunia_design_jobs::JobManager {
        &self.jobs
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

    /// The one-line hint describing what the given tool is for in the active locale (Ledger M17).
    #[must_use]
    pub fn tool_hint(&self, tool: ToolKind) -> Option<String> {
        menu::tool_hint_text_id(tool).map(|id| self.localization.text(id, &self.locale))
    }

    /// Pluralized item count formatted for the active session locale (Ledger M20).
    #[must_use]
    pub fn plural_items(&self, count: usize) -> String {
        self.localization.plural_items(count, &self.locale)
    }

    /// The localization service resolving `ptnd.text.*` ids.
    #[must_use]
    pub fn localization(&self) -> &LocalizationService {
        &self.localization
    }

    /// Session facts that decide what the menu and palette can offer (15.G).
    #[must_use]
    pub fn action_context(&self) -> ActionContext {
        match self.active() {
            Some(session) => ActionContext {
                has_document: true,
                has_cmyk_layer_profile: session.native_layer_profile().is_some(),
                selection_count: session.selection.selected_ids.len(),
                can_undo: session.history().can_undo(),
                can_redo: session.history().can_redo(),
                is_dirty: session.is_dirty(),
                clipboard_non_empty: !self.clipboard.is_empty() || !session.clipboard().is_empty(),
                native_clipboard_available:
                    petunia_design_platform::native_clipboard::LinuxClipboardBackend::detect()
                        .is_ok(),
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
        self.push_session(session);
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
        self.push_session(session);
        self.snapshot_cache.borrow_mut().clear();
        Ok(())
    }

    /// Attaches already admitted import data. Imports require Save As into PTND.
    pub fn attach_imported_document(
        &mut self,
        title: String,
        document: Document,
    ) -> Result<(), PetuniaError> {
        document.validate()?;
        self.push_session(DocumentSession::with_recovered_document(title, document));
        self.snapshot_cache.borrow_mut().clear();
        Ok(())
    }
    /// Publish admitted native clipboard data through the ordinary command lane.
    #[allow(clippy::too_many_arguments)] // Captured clipboard completion guards and placement.
    pub fn complete_clipboard_fragment(
        &mut self,
        target: petunia_design_application::session::SessionIdentity,
        revision: u64,
        surface: SurfaceId,
        fragment: Vec<petunia_design_document::DocumentObject>,
        cut: Option<Vec<ObjectId>>,
        paste: bool,
        origin: [f64; 2],
    ) -> Result<ChangeSet, PetuniaError> {
        let session = self
            .sessions
            .iter_mut()
            .find(|s| s.identity() == target)
            .ok_or_else(|| PetuniaError::not_found("clipboard target tab was closed"))?;
        if (paste || cut.is_some()) && session.current_revision() != revision {
            return Err(PetuniaError::invalid_input(
                "clipboard target changed; retry the operation",
            ));
        }
        let result = if paste {
            session.paste_clipboard_fragment(surface, fragment.clone(), origin)?
        } else if let Some(ids) = cut {
            session.cut_clipboard_fragment(&ids, fragment.clone())?
        } else {
            session.set_clipboard(fragment.clone());
            ChangeSet::empty()
        };
        session.set_clipboard_at(fragment.clone(), origin);
        self.clipboard = fragment;
        self.clipboard_origin = origin;
        Ok(result)
    }

    /// Savepoint publication is scoped to the captured tab, even if it is inactive.
    pub fn acknowledge_saved_snapshot(
        &mut self,
        target: petunia_design_application::session::SessionIdentity,
        path: std::path::PathBuf,
        revision: u64,
        history_state: u64,
    ) -> Result<bool, PetuniaError> {
        let session = self
            .sessions
            .iter_mut()
            .find(|s| s.identity() == target)
            .ok_or_else(|| PetuniaError::not_found("saved document tab was closed"))?;
        session.acknowledge_saved_snapshot(path, revision, history_state);
        Ok(!session.is_dirty())
    }
    /// Worker import data enters the ordinary command lane only after its source guard.
    pub fn place_prepared_image(
        &mut self,
        target: petunia_design_application::session::SessionIdentity,
        revision: u64,
        surface: SurfaceId,
        path: std::path::PathBuf,
        source: std::sync::Arc<petunia_design_raster::EncodedImage>,
        size: [u32; 2],
    ) -> Result<ChangeSet, PetuniaError> {
        let shape = ShapeKind::Image {
            path: path.to_string_lossy().into_owned(),
            data: Some(source),
        };
        self.place_prepared_shape(target, revision, surface, path, shape, size)
    }
    /// Publishes worker-admitted native samples as an editable ink layer.
    pub fn place_prepared_cmyk_layer(
        &mut self,
        target: petunia_design_application::session::SessionIdentity,
        revision: u64,
        surface: SurfaceId,
        path: std::path::PathBuf,
        layer: std::sync::Arc<petunia_design_raster::RasterLayer>,
    ) -> Result<ChangeSet, PetuniaError> {
        if !layer.is_cmyk() {
            return Err(PetuniaError::invalid_input(
                "CMYK placement requires native ink",
            ));
        }
        let size = [layer.width(), layer.height()];
        self.place_prepared_shape(
            target,
            revision,
            surface,
            path,
            ShapeKind::Raster { layer },
            size,
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn place_prepared_shape(
        &mut self,
        target: petunia_design_application::session::SessionIdentity,
        revision: u64,
        surface: SurfaceId,
        path: std::path::PathBuf,
        shape: ShapeKind,
        size: [u32; 2],
    ) -> Result<ChangeSet, PetuniaError> {
        if size.contains(&0) {
            return Err(PetuniaError::invalid_input("empty prepared image"));
        }
        let session = self
            .sessions
            .iter_mut()
            .find(|s| s.identity() == target)
            .ok_or_else(|| PetuniaError::not_found("image target tab was closed"))?;
        if session.current_revision() != revision {
            return Err(PetuniaError::invalid_input(
                "image target changed during admission; retry placement",
            ));
        }
        let board = session.document().surface(surface)?;
        let fit = (board.dimensions[0] * 0.8 / f64::from(size[0]))
            .min(board.dimensions[1] * 0.8 / f64::from(size[1]))
            .min(1.);
        let width = f64::from(size[0]) * fit;
        let height = f64::from(size[1]) * fit;
        let bounds = [
            board.origin[0] + (board.dimensions[0] - width) * 0.5,
            board.origin[1] + (board.dimensions[1] - height) * 0.5,
            width,
            height,
        ];
        let id = session.next_object_id();
        let changes = session.execute_command(CommandRequest::new(Command::CreateShapeObject {
            surface,
            id,
            name: path
                .file_stem()
                .map_or_else(|| "Image".into(), |n| n.to_string_lossy().into_owned()),
            shape,
            bounds: Some(bounds),
            fill: None,
            stroke: None,
            stroke_width: 0.,
        }))?;
        session.selection.select_exact(vec![id]);
        Ok(changes)
    }

    /// Publishes a worker-validated ICC assignment to its captured document.
    /// No color conversion is implicit, and tab switching cannot retarget it.
    pub fn assign_prepared_cmyk_profile(
        &mut self,
        target: petunia_design_application::session::SessionIdentity,
        revision: u64,
        surface: SurfaceId,
        profile: petunia_design_color::IccProfile,
    ) -> Result<ChangeSet, PetuniaError> {
        let session = self
            .sessions
            .iter_mut()
            .find(|s| s.identity() == target)
            .ok_or_else(|| PetuniaError::not_found("ICC target tab was closed"))?;
        if session.current_revision() != revision {
            return Err(PetuniaError::invalid_input(
                "ICC target changed during admission; retry assignment",
            ));
        }
        session.execute_command(CommandRequest::new(Command::SetSurfaceCmykProfile {
            surface,
            profile: Some(profile),
        }))
    }
    pub fn assign_prepared_cmyk_layer_profile(
        &mut self,
        target: petunia_design_application::session::SessionIdentity,
        revision: u64,
        object: ObjectId,
        profile: petunia_design_color::IccProfile,
    ) -> Result<ChangeSet, PetuniaError> {
        let session = self
            .sessions
            .iter_mut()
            .find(|s| s.identity() == target)
            .ok_or_else(|| PetuniaError::not_found("ICC layer target tab was closed"))?;
        if session.current_revision() != revision {
            return Err(PetuniaError::invalid_input(
                "ICC layer changed during admission; retry assignment",
            ));
        }
        session.execute_command(CommandRequest::new(Command::AssignRasterCmykProfile {
            id: object,
            profile,
        }))
    }

    /// Opens a project from disk, replacing the active session.
    ///
    /// A legacy package decodes into a canonical document but deliberately
    /// records **no path**: the next save must go through Save As so the
    /// original `.aubrieta`/`.aubri` file is never overwritten (15.A).
    pub fn open_path(&mut self, path: &std::path::Path) -> Result<(), PetuniaError> {
        if path
            .extension()
            .and_then(|v| v.to_str())
            .is_some_and(|v| v.eq_ignore_ascii_case("svg"))
        {
            let document = petunia_design_io::read_svg(path)?;
            let title = path.file_stem().map_or_else(
                || "SVG".to_owned(),
                |name| name.to_string_lossy().into_owned(),
            );
            self.push_session(DocumentSession::with_recovered_document(title, document));
            self.snapshot_cache.borrow_mut().clear();
            return Ok(());
        }
        self.attach_opened_package(path, petunia_design_io::open_package(path)?, false)
    }
    pub fn open_recovery_path(&mut self, path: &std::path::Path) -> Result<(), PetuniaError> {
        self.attach_opened_package(path, petunia_design_io::open_package(path)?, true)
    }
    pub fn attach_opened_package(
        &mut self,
        path: &std::path::Path,
        opened: petunia_design_io::OpenedPackage,
        require_recovery: bool,
    ) -> Result<(), PetuniaError> {
        opened.document.validate()?;
        if require_recovery && opened.recovery.is_none() {
            return Err(PetuniaError::invalid_input(
                "selected package is not a recovery snapshot",
            ));
        }
        let recovery = opened.recovery.is_some();
        let title = opened.recovery.map(|meta| meta.title).unwrap_or_else(|| {
            path.file_name().map_or_else(
                || "Untitled".to_string(),
                |name| name.to_string_lossy().into_owned(),
            )
        });
        let mut session = if recovery {
            DocumentSession::with_recovered_document(title, opened.document)
        } else {
            DocumentSession::with_document(title, opened.document)
        };
        if opened.format == petunia_design_io::PackageFormat::Ptnd && !recovery {
            session.adopt_path(path.to_path_buf());
        }
        self.push_session(session);
        Ok(())
    }

    /// Reads the active session's index, if any.
    fn active_idx(&self) -> Option<usize> {
        self.active_index
            .filter(|index| *index < self.sessions.len())
    }

    /// Borrows the active session or returns the closed-session error.
    fn active(&self) -> Option<&DocumentSession> {
        self.active_idx().and_then(|index| self.sessions.get(index))
    }

    /// Mutably borrows the active session or returns the closed-session error.
    fn active_mut(&mut self) -> Option<&mut DocumentSession> {
        self.active_idx()
            .and_then(|index| self.sessions.get_mut(index))
    }

    /// Closes the active session, checking unsaved dirty state.
    ///
    /// A closed tab activates the neighbour to its left; closing the last tab
    /// leaves no session. Returns `false` when the document is dirty and the
    /// caller did not confirm the discard.
    pub fn close_session(&mut self, force: bool) -> Result<bool, PetuniaError> {
        if let Some(session) = self.active() {
            if session.is_dirty() && !force {
                return Ok(false); // Unsaved changes require user decision
            }
        }
        if self.active_idx().is_none() {
            return Ok(true);
        }
        let index = self.active_idx().expect("active tab exists");
        let _ = self.close_session_at(index, force);
        Ok(true)
    }

    /// Opens a session as a new tab and activates it.
    ///
    /// The opened document goes last; every existing tab stays in place:
    /// `file.new`/`file.open` keep open documents, they do not replace them.
    fn push_session(&mut self, session: DocumentSession) {
        self.sessions.push(session);
        self.active_index = Some(self.sessions.len() - 1);
        self.snapshot_cache.borrow_mut().clear();
    }

    /// Closes every open session.
    ///
    /// Returns `false` without closing anything when any document is dirty and
    /// the caller did not confirm: the decision is all-or-nothing so a quit
    /// never leaves a half-closed window.
    pub fn close_all_sessions(&mut self, force: bool) -> Result<bool, PetuniaError> {
        if !force && self.any_session_dirty() {
            return Ok(false);
        }
        self.sessions.clear();
        self.active_index = None;
        self.snapshot_cache.borrow_mut().clear();
        Ok(true)
    }

    /// True when any open document has unsaved changes.
    #[must_use]
    pub fn any_session_dirty(&self) -> bool {
        self.sessions.iter().any(DocumentSession::is_dirty)
    }

    /// Every open session in tab order.
    ///
    /// Read-only: sessions are opened and closed through the action lane, never
    /// mutated from a presentation read.
    #[must_use]
    pub fn sessions(&self) -> Vec<&DocumentSession> {
        self.sessions.iter().collect()
    }

    /// The active tab index in `sessions()` order, or `None` when closed.
    #[must_use]
    pub fn active_session_index(&self) -> Option<usize> {
        self.active_idx()
    }

    /// Activates the tab at `index` in `sessions()` order.
    ///
    /// A miss is a no-op error: a stale tab click closes no document and opens
    /// none, it simply does nothing.
    pub fn switch_session(&mut self, index: usize) -> Result<bool, PetuniaError> {
        if index >= self.sessions.len() {
            return Ok(false);
        }
        self.active_index = Some(index);
        self.snapshot_cache.borrow_mut().clear();
        Ok(true)
    }

    /// Closes the tab at `index` in `sessions()` order.
    ///
    /// Returns `false` when the document is dirty and not confirmed, or when
    /// `index` is out of range. Closing the active tab activates the tab to its
    /// left; closing an inactive tab leaves the active one alone.
    pub fn close_session_at(&mut self, index: usize, force: bool) -> Result<bool, PetuniaError> {
        if index >= self.sessions.len() {
            return Ok(false);
        }
        if self.sessions[index].is_dirty() && !force {
            return Ok(false);
        }
        let active = self.active_idx();
        self.sessions.remove(index);
        self.active_index = match active {
            Some(_) if self.sessions.is_empty() => None,
            Some(current) if index == current => {
                Some(current.saturating_sub(1).min(self.sessions.len() - 1))
            }
            Some(current) if index < current => Some(current - 1),
            other => other,
        };
        self.snapshot_cache.borrow_mut().clear();
        Ok(true)
    }

    /// Accesses the active document session.
    #[must_use]
    pub fn session(&self) -> Option<&DocumentSession> {
        self.active()
    }

    /// Reads the full-scene snapshot cache (interior-mutable, `GeoCache`
    /// pattern: shared `&self` readers, keyed entry, no signature churn).
    #[must_use]
    pub(crate) fn snapshot_cache(&self) -> std::cell::Ref<'_, crate::canvas::SnapshotCache> {
        self.snapshot_cache.borrow()
    }

    /// Stores a fresh full-scene evaluation in the snapshot cache.
    pub(crate) fn snapshot_cache_mut(&self) -> std::cell::RefMut<'_, crate::canvas::SnapshotCache> {
        self.snapshot_cache.borrow_mut()
    }

    /// Mutable access to the active session's **view** state only.
    ///
    /// View state (camera, rulers, snapping, palette) is not document state:
    /// it does not travel the command lane, and the `document`/`history`
    /// fields stay private so this cannot become a mutation back door (A2).
    pub fn view_state_mut(
        &mut self,
    ) -> Option<&mut petunia_design_application::view_camera::ViewState> {
        self.active_mut().map(|session| &mut session.view)
    }

    /// Returns a reference to the global capability registry.
    #[must_use]
    pub fn capabilities(&self) -> &CapabilityRegistry {
        &self.capabilities
    }

    /// Helper to borrow active session or return error if closed.
    #[allow(dead_code)]
    fn session_req(&self) -> Result<&DocumentSession, PetuniaError> {
        self.active()
            .ok_or_else(|| PetuniaError::invalid_input("no active document session"))
    }

    /// Helper to mutably borrow active session or return error if closed.
    fn session_req_mut(&mut self) -> Result<&mut DocumentSession, PetuniaError> {
        self.active_mut()
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

    /// Publishes a disposable paint draft through the application command lane.
    pub fn commit_raster_stroke(
        &mut self,
        stroke: petunia_design_application::raster_edit::RasterStroke,
    ) -> Result<ChangeSet, PetuniaError> {
        stroke.commit(self.session_req_mut()?)
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

    /// Returns the transient raster selection mask (marching ants, 10.9).
    #[must_use]
    pub fn raster_selection(&self) -> petunia_design_application::RasterSelection {
        self.session()
            .map(|s| s.raster_selection.clone())
            .unwrap_or_default()
    }

    /// Evaluated outline, memoized by session revision (F1).
    /// Hot loops (hover, overlays, covering) must prefer this over
    /// `find_object().evaluated_path()`.
    #[must_use]
    pub fn cached_path(&self, id: ObjectId) -> Option<petunia_design_geometry::GPath> {
        self.session()?.cached_path(id)
    }

    /// Evaluated bounds, memoized by session revision (F1).
    #[must_use]
    pub fn cached_bounds(&self, id: ObjectId) -> Option<[f64; 4]> {
        self.session()?.cached_bounds(id)
    }

    /// Explicit world evaluated bounds, when the frame is migrated.
    #[must_use]
    pub fn cached_world_bounds(&self, id: ObjectId) -> Option<[f64; 4]> {
        let session = self.session()?;
        if let Some(scene) = session.active_surface().and_then(|surface| {
            self.snapshot_cache
                .borrow()
                .prepared_scene(session.current_revision(), surface)
        }) {
            if let Some(node) = scene.node(id) {
                if let Some(bounds) = node.text_bounds() {
                    let bounds = node.local_to_world().transform_rect(bounds);
                    return Some([bounds.x0, bounds.y0, bounds.width(), bounds.height()]);
                }
            }
        }
        session.cached_world_bounds(id)
    }

    /// Nominal world frame bounds, available for legacy paths too.
    #[must_use]
    pub fn cached_world_frame_bounds(&self, id: ObjectId) -> Option<[f64; 4]> {
        self.session()?.cached_world_frame_bounds(id)
    }

    /// Exact hit against the explicit world-space outline.
    #[must_use]
    pub fn cached_world_hit(
        &self,
        id: ObjectId,
        pt: petunia_design_geometry::GPoint,
        tol: f64,
    ) -> bool {
        let Some(session) = self.session() else {
            return false;
        };
        if let Some(scene) = session.active_surface().and_then(|surface| {
            self.snapshot_cache
                .borrow()
                .prepared_scene(session.current_revision(), surface)
        }) {
            if let Some(hit) = scene.node(id).and_then(|node| node.text_hit(pt)) {
                return hit;
            }
        }
        session.cached_world_hit(id, pt, tol)
    }
    /// Add worker-shaped artistic overflow to the spatial prefilter, retaining
    /// canonical stacking order rather than HashMap iteration order.
    pub fn spatial_candidates_point(
        &self,
        point: petunia_design_geometry::GPoint,
        tolerance: f64,
    ) -> Vec<ObjectId> {
        let Some(session) = self.session() else {
            return Vec::new();
        };
        let mut ids: std::collections::HashSet<_> = session
            .spatial_candidates_point(point, tolerance)
            .into_iter()
            .collect();
        if let Some(scene) = session.active_surface().and_then(|surface| {
            self.snapshot_cache
                .borrow()
                .prepared_scene(session.current_revision(), surface)
        }) {
            for node in scene.text_nodes() {
                if let Some(bounds) = node.text_bounds() {
                    let b = node.local_to_world().transform_rect(bounds);
                    if point.x >= b.x0 - tolerance
                        && point.x <= b.x1 + tolerance
                        && point.y >= b.y0 - tolerance
                        && point.y <= b.y1 + tolerance
                    {
                        ids.insert(node.id());
                    }
                }
            }
        }
        session
            .document()
            .surfaces()
            .iter()
            .flat_map(|surface| surface.objects().iter().rev())
            .filter(|object| ids.contains(&object.id))
            .map(|object| object.id)
            .collect()
    }
    /// Hit-test against the memoized evaluated outline (F1 + F2).
    /// `tol` should come from `zoom_flatten_tol`. Visibility/locking stay
    /// at the call site, as with `hit_test` today.
    #[must_use]
    pub fn cached_hit(&self, id: ObjectId, pt: petunia_design_geometry::GPoint, tol: f64) -> bool {
        self.session().is_some_and(|s| s.cached_hit(id, pt, tol))
    }

    /// Flattened evaluated outline at `tol`, memoized (F2).
    #[must_use]
    pub fn cached_polygons(
        &self,
        id: ObjectId,
        tol: f64,
    ) -> Option<Vec<Vec<petunia_design_geometry::GPoint>>> {
        self.session()?.cached_polygons(id, tol)
    }

    /// Outline sample at fraction `t`, memoized (F2).
    #[must_use]
    pub fn cached_sample_at(
        &self,
        id: ObjectId,
        t: f64,
        tol: f64,
    ) -> Option<(petunia_design_geometry::GPoint, f64)> {
        self.session()?.cached_sample_at(id, t, tol)
    }

    /// Nearest outline fraction, memoized (F2).
    #[must_use]
    pub fn cached_nearest_t(
        &self,
        id: ObjectId,
        pt: petunia_design_geometry::GPoint,
        tol: f64,
    ) -> Option<f64> {
        self.session()?.cached_nearest_t(id, pt, tol)
    }

    /// Cache entry count (diagnostics and tests).
    #[must_use]
    pub fn geo_cache_len(&self) -> usize {
        self.session().map_or(0, |s| s.geo_cache.borrow().len())
    }

    /// Total flatten lookups served (F7.3 probe: shared-flatten spans).
    #[must_use]
    pub fn flatten_lookup_count(&self) -> u64 {
        self.session().map_or(0, |s| s.flatten_lookup_count())
    }

    /// Actual flatten computations, i.e. cache misses (F7.3 probe).
    #[must_use]
    pub fn flatten_compute_count(&self) -> u64 {
        self.session().map_or(0, |s| s.flatten_compute_count())
    }

    /// Resets the F7.3 flatten probes without dropping cached geometry.
    pub fn reset_flatten_stats(&self) {
        if let Some(session) = self.session() {
            session.reset_flatten_stats();
        }
    }

    /// Combines one shape into the raster mask (session state, no undo).
    pub fn combine_raster_selection(
        &mut self,
        shape: petunia_design_application::SelectionShape,
        mode: petunia_design_application::SelectionMode,
    ) {
        if let Some(session) = self.active_mut() {
            session.raster_selection.combine(&shape, mode);
        }
    }

    /// Clears the raster mask (Ctrl+D equivalent).
    pub fn clear_raster_selection(&mut self) {
        if let Some(session) = self.active_mut() {
            session.raster_selection.clear();
        }
    }

    /// Inverts the raster mask inside the active surface bounds.
    /// Empty masks stay empty.
    pub fn invert_raster_selection(&mut self) {
        let frame = self.active().and_then(|s| {
            let surface_id = s.active_surface()?;
            let surface = s.surface(surface_id).ok()?;
            let [x, y, w, h] = surface.bounds();
            use petunia_design_geometry::GPoint;
            Some(vec![
                GPoint::new(x, y),
                GPoint::new(x + w, y),
                GPoint::new(x + w, y + h),
                GPoint::new(x, y + h),
            ])
        });
        if let (Some(session), Some(frame)) = (self.active_mut(), frame) {
            session.raster_selection.invert_in(&frame);
        }
    }

    /// Grows (positive) or shrinks (negative) the raster mask.
    pub fn grow_raster_selection(&mut self, delta: f64) {
        if let Some(session) = self.active_mut() {
            session.raster_selection.grow(delta);
        }
    }

    /// Sets the raster feather radius (render-time parameter).
    pub fn set_raster_feather(&mut self, radius: f64) {
        if let Some(session) = self.active_mut() {
            session.raster_selection.set_feather(radius);
        }
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

    /// Renames an object by stable ID.
    pub fn rename_object(
        &mut self,
        id: ObjectId,
        name: impl Into<String>,
    ) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::RenameObject {
            id,
            name: name.into(),
        }))
    }

    /// Bakes corner geometry into an explicit vector path (10.2, 10.3).
    pub fn bake_corners(&mut self, id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::BakeCorners { id }))
    }

    /// Offsets an outline, non-destructively (09.31, 10.3).
    /// Upserts the live `ContourOffset` modifier; base geometry is untouched.
    pub fn offset_path(&mut self, id: ObjectId, delta: f64) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::OffsetPath { id, delta }))
    }

    /// Replaces an object's live modifier chain (09.31, one undo entry).
    pub fn set_modifiers(
        &mut self,
        id: ObjectId,
        modifiers: Vec<petunia_design_document::ModifierItem>,
    ) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::SetModifiers { id, modifiers }))
    }

    /// Sets an object's live contour offset with join/cap style (09.31).
    /// Zero distance removes the entry. Base geometry is never touched.
    pub fn set_contour_offset(
        &mut self,
        id: ObjectId,
        distance: f64,
        join: petunia_design_geometry::OffsetJoin,
        cap: petunia_design_geometry::OffsetCap,
    ) -> Result<ChangeSet, PetuniaError> {
        let next = {
            let session = self.session_req_mut()?;
            let obj = session.document().find_object(id).ok_or_else(|| {
                PetuniaError::invalid_input(format!("object `{id}` does not exist"))
            })?;
            let mut next = obj.modifiers.clone();
            if distance.abs() < 1e-9 {
                next.retain(|m| {
                    !matches!(
                        m.kind,
                        petunia_design_document::ModifierKind::ContourOffset { .. }
                    )
                });
            } else if let Some(entry) = next.iter_mut().find(|m| {
                matches!(
                    m.kind,
                    petunia_design_document::ModifierKind::ContourOffset { .. }
                )
            }) {
                entry.kind = petunia_design_document::ModifierKind::ContourOffset {
                    distance,
                    join,
                    cap,
                };
            } else {
                let nid = next.iter().map(|m| m.id).max().unwrap_or(0) + 1;
                next.push(petunia_design_document::ModifierItem::enabled(
                    nid,
                    petunia_design_document::ModifierKind::ContourOffset {
                        distance,
                        join,
                        cap,
                    },
                ));
            }
            next
        };
        self.set_modifiers(id, next)
    }

    /// Bakes live contour offsets into base geometry (explicit user op, 09.31).
    pub fn bake_contour(&mut self, id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::BakeContour { id }))
    }

    /// Bakes live transparency gradients into base opacity (explicit, 09.31).
    pub fn bake_transparency(&mut self, id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::BakeTransparency { id }))
    }

    /// Bakes all live geometry-domain modifiers (explicit user op, 09.31).
    pub fn bake_geometry(&mut self, id: ObjectId) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::BakeGeometry { id }))
    }

    /// Sets a live perspective quad, non-destructively (09.31, 10.8).
    pub fn set_perspective(
        &mut self,
        id: ObjectId,
        quad: [[f64; 2]; 4],
    ) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::SetPerspective { id, quad }))
    }

    /// Sets a live rectangular crop, non-destructively (09.31, 08.24).
    pub fn set_crop_rect(
        &mut self,
        id: ObjectId,
        rect: [f64; 4],
    ) -> Result<ChangeSet, PetuniaError> {
        self.submit_command(CommandRequest::new(Command::SetCropRect { id, rect }))
    }

    /// Sets a live transparency gradient vector (09.31, replaces the
    /// whole-stack opacity proxy). Default stops run opaque to transparent.
    /// Zero-length vectors clear the entry. Base geometry is never touched.
    pub fn set_transparency_vector(
        &mut self,
        id: ObjectId,
        start: [f64; 2],
        end: [f64; 2],
    ) -> Result<ChangeSet, PetuniaError> {
        use petunia_design_document::{ModifierItem, ModifierKind, OpacityStop};
        let next = {
            let session = self.session_req_mut()?;
            let obj = session.document().find_object(id).ok_or_else(|| {
                PetuniaError::invalid_input(format!("object `{id}` does not exist"))
            })?;
            let mut next = obj.modifiers.clone();
            let degenerate = (end[0] - start[0]).hypot(end[1] - start[1]) < 1e-9;
            next.retain(|m| !matches!(m.kind, ModifierKind::TransparentGradient { .. }));
            if !degenerate {
                let nid = next.iter().map(|m| m.id).max().unwrap_or(0) + 1;
                next.push(ModifierItem::enabled(
                    nid,
                    ModifierKind::TransparentGradient {
                        start,
                        end,
                        stops: vec![OpacityStop::new(0.0, 1.0), OpacityStop::new(1.0, 0.0)],
                    },
                ));
            }
            next
        };
        self.set_modifiers(id, next)
    }

    /// Enables or disables one chain entry (future modifier-list UI).
    pub fn set_modifier_enabled(
        &mut self,
        id: ObjectId,
        modifier_id: u32,
        enabled: bool,
    ) -> Result<ChangeSet, PetuniaError> {
        let next = {
            let session = self.session_req_mut()?;
            let obj = session.document().find_object(id).ok_or_else(|| {
                PetuniaError::invalid_input(format!("object `{id}` does not exist"))
            })?;
            let mut next = obj.modifiers.clone();
            if let Some(entry) = next.iter_mut().find(|m| m.id == modifier_id) {
                entry.enabled = enabled;
            }
            next
        };
        self.set_modifiers(id, next)
    }

    /// Removes one chain entry, if present (future modifier-list UI).
    pub fn remove_modifier(
        &mut self,
        id: ObjectId,
        modifier_id: u32,
    ) -> Result<ChangeSet, PetuniaError> {
        let next = {
            let session = self.session_req_mut()?;
            let obj = session.document().find_object(id).ok_or_else(|| {
                PetuniaError::invalid_input(format!("object `{id}` does not exist"))
            })?;
            obj.modifiers
                .iter()
                .filter(|m| m.id != modifier_id)
                .cloned()
                .collect()
        };
        self.set_modifiers(id, next)
    }

    /// Moves one chain entry to the front (evaluated first).
    pub fn move_modifier_to_front(
        &mut self,
        id: ObjectId,
        modifier_id: u32,
    ) -> Result<ChangeSet, PetuniaError> {
        let next = {
            let session = self.session_req_mut()?;
            let obj = session.document().find_object(id).ok_or_else(|| {
                PetuniaError::invalid_input(format!("object `{id}` does not exist"))
            })?;
            let mut next = obj.modifiers.clone();
            if let Some(pos) = next.iter().position(|m| m.id == modifier_id) {
                let entry = next.remove(pos);
                next.insert(0, entry);
            }
            next
        };
        self.set_modifiers(id, next)
    }

    /// Reads an object's live modifier chain.
    #[must_use]
    pub fn modifiers(&self, id: ObjectId) -> Vec<petunia_design_document::ModifierItem> {
        self.session()
            .and_then(|s| s.find_object(id))
            .map(|o| o.modifiers.clone())
            .unwrap_or_default()
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
            .active()
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
            .active()
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
        if let Some(session) = self.active() {
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
        // Document-lifecycle actions replace or drop the tab set, so they are
        // handled by the host before the active session is even borrowed.
        let normalized = petunia_design_foundation::normalized(request.action.as_str());
        let action = petunia_design_foundation::normalize_action_id(&normalized);
        match action.as_str() {
            "ptnd.action.file.new" => {
                self.new_document("Untitled")?;
                return Ok(ChangeSet::empty());
            }
            "ptnd.action.file.recover" => {
                let path = request
                    .payload
                    .get("path")
                    .and_then(serde_json::Value::as_str)
                    .filter(|path| !path.trim().is_empty())
                    .ok_or_else(|| PetuniaError::invalid_input("recovery requires a path"))?;
                self.open_recovery_path(std::path::Path::new(path))?;
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
            "ptnd.action.file.close" => {
                // `force` is the UI's confirmed-dirty answer, never a default.
                let force = request
                    .payload
                    .get("force")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);
                let index = request
                    .payload
                    .get("index")
                    .and_then(serde_json::Value::as_u64)
                    .map_or_else(
                        || self.active_session_index(),
                        |index| usize::try_from(index).ok(),
                    );
                let Some(index) = index else {
                    return Ok(ChangeSet::empty());
                };
                if !self.close_session_at(index, force)? {
                    return Err(PetuniaError::invalid_input(
                        "file.close refused: the document has unsaved changes",
                    ));
                }
                return Ok(ChangeSet::empty());
            }
            "ptnd.action.file.quit" => {
                let force = request
                    .payload
                    .get("force")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);
                if !self.close_all_sessions(force)? {
                    return Err(PetuniaError::invalid_input(
                        "file.quit refused: an open document has unsaved changes",
                    ));
                }
                return Ok(ChangeSet::empty());
            }
            _ => {}
        }
        if action == "ptnd.action.edit.paste" && !self.clipboard.is_empty() {
            let clipboard = self.clipboard.clone();
            let origin = self.clipboard_origin;
            self.session_req_mut()?.set_clipboard_at(clipboard, origin);
        }
        let result = self.session_req_mut()?.dispatch_action(request);
        if result.is_ok()
            && matches!(
                action.as_str(),
                "ptnd.action.edit.copy" | "ptnd.action.edit.cut" | "ptnd.action.edit.paste"
            )
        {
            self.clipboard_origin = self.session_req_mut()?.clipboard_origin();
            self.clipboard = self.session_req_mut()?.clipboard().to_vec();
        }
        result
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
        self.active().is_some_and(|s| s.history().can_undo())
    }

    fn can_redo(&self) -> bool {
        self.active().is_some_and(|s| s.history().can_redo())
    }
}

impl PropertyPort for PetuniaDesignGuiBridge {
    fn query_properties(&self) -> PropertiesPresentationModel {
        self.active()
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
        self.active().map_or_else(
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
        self.active()
            .map_or_else(DocumentSummary::default, |s| s.summary())
    }

    fn query_layers(&self) -> LayersPresentationModel {
        self.active()
            .map_or_else(LayersPresentationModel::default, |s| {
                s.layers_presentation_model()
            })
    }
}

impl SelectionPort for PetuniaDesignGuiBridge {
    fn selection(&self) -> SelectionViewModel {
        self.active()
            .map_or_else(SelectionViewModel::default, |s| s.selection_view_model())
    }

    fn set_selection(&mut self, ids: Vec<ObjectId>) {
        if let Some(session) = self.active_mut() {
            session.selection.select_exact(ids);
            session.prune_selection();
        }
    }

    fn toggle_selection(&mut self, id: ObjectId) {
        if let Some(session) = self.active_mut() {
            session.selection.toggle(id);
            session.prune_selection();
        }
    }

    fn clear_selection(&mut self) {
        if let Some(session) = self.active_mut() {
            session.selection.clear();
        }
    }

    fn select_all(&mut self) {
        if let Some(session) = self.active_mut() {
            session.select_all();
        }
    }
}

impl InspectionPort for PetuniaDesignGuiBridge {
    fn active_surface(&self) -> Option<SurfaceId> {
        self.active().and_then(|s| s.active_surface())
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
        self.active().map_or(0, |s| s.current_revision())
    }

    fn is_dirty(&self) -> bool {
        self.active().is_some_and(|s| s.is_dirty())
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
        self.active()
            .map_or_else(DataMergePresentationModel::default, |s| {
                s.data_merge_presentation_model()
            })
    }
}
