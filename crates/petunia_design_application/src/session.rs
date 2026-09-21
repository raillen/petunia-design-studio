//! Document session ownership and selection session management (09.24).
//!
//! Moved from the legacy GUI bridge crate: session ownership is domain
//! logic (document + history + revision + selection rules), toolkit-free.
//! `document` and `history` are private: all mutations flow through
//! `execute_command` / `dispatch_action` / `undo` / `redo` (A2).

use petunia_design_document::{
    AlignmentMode, ChangeSet, DataBinding, DataSourceDefinition, DistributionAxis, Document,
    DocumentObject, Surface,
};
use petunia_design_foundation::{PetuniaError, IdGenerator, ObjectId, SurfaceId};

use super::view_models::{
    DataBindingViewModel, DataMergePresentationModel, DataSourceViewModel, DocumentSummary,
    FieldViewModel, LayerRowViewModel, LayersPresentationModel, PropertiesPresentationModel,
    SelectionViewModel, SessionSnapshot, SurfaceRowViewModel,
};

use crate::{ActionRequest, Command, CommandRequest, History};

/// Selection session state: ordered selection of stable IDs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SelectionSession {
    /// Ordered selection list. The last intentionally clicked item is the primary/key object.
    pub selected_ids: Vec<ObjectId>,
    /// Explicit key object override, if any.
    pub key_object_override: Option<ObjectId>,
}

impl SelectionSession {
    /// Creates a fresh empty selection session.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Selects an explicit set of IDs.
    pub fn select_exact(&mut self, ids: Vec<ObjectId>) {
        self.selected_ids = ids;
        self.key_object_override = None;
    }

    /// Toggles an object ID in the selection.
    pub fn toggle(&mut self, id: ObjectId) {
        if let Some(pos) = self.selected_ids.iter().position(|&x| x == id) {
            self.selected_ids.remove(pos);
            if self.key_object_override == Some(id) {
                self.key_object_override = None;
            }
        } else {
            self.selected_ids.push(id);
        }
    }

    /// Clears the selection.
    pub fn clear(&mut self) {
        self.selected_ids.clear();
        self.key_object_override = None;
    }

    /// Resolves the primary/key object according to stable 10.1 rules.
    #[must_use]
    pub fn key_object(&self) -> Option<ObjectId> {
        self.key_object_override
            .or_else(|| self.selected_ids.last().copied())
    }

    /// Removes deleted IDs from the selection.
    pub fn prune_missing(&mut self, valid_ids: &[ObjectId]) {
        self.selected_ids.retain(|id| valid_ids.contains(id));
        if let Some(key) = self.key_object_override {
            if !valid_ids.contains(&key) {
                self.key_object_override = None;
            }
        }
    }
}

/// Document session owning one open document, history, and selection (09.24).
#[derive(Debug)]
pub struct DocumentSession {
    /// Canonical document storage. Private: read via `document()` and
    /// targeted accessors; mutate only via command lane (A2).
    document: Document,
    /// Transactional undo/redo history. Private: use
    /// `execute_command` / `undo` / `redo` so revision stays exact (A2).
    history: History,
    /// Viewport/window-shared selection session.
    pub selection: SelectionSession,
    /// Non-document view state (camera, rulers, snapping). Lives here so view
    /// actions travel the same Action lane as document actions (15.B).
    pub view: crate::view_camera::ViewState,
    /// Monotonic ID generator for session-originated objects.
    id_generator: IdGenerator,
    /// Document title or filename.
    title: String,
    /// Filesystem location this session was last saved to or opened from.
    /// `None` means the project has never been written, which is what forces
    /// `file.save` to ask for `file.save_as`.
    path: Option<std::path::PathBuf>,
    /// Active surface for editing.
    active_surface: Option<SurfaceId>,
    /// Revision monotonic counter.
    current_revision: u64,
    /// Revision at last explicit save.
    saved_revision: u64,
}

impl DocumentSession {
    /// Creates a new document session with a fresh document.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        let document = Document::default();
        let active_surface = document.surfaces().first().map(|s| s.id);
        let max_id = document
            .surfaces()
            .iter()
            .map(|s| s.id.raw())
            .chain(
                document
                    .surfaces()
                    .iter()
                    .flat_map(|s| s.objects().iter().map(|o| o.id.raw())),
            )
            .max()
            .unwrap_or(0);
        Self {
            document,
            history: History::new(0),
            selection: SelectionSession::new(),
            view: crate::view_camera::ViewState::default(),
            id_generator: IdGenerator::with_start(max_id + 1),
            title: title.into(),
            path: None,
            active_surface,
            current_revision: 0,
            saved_revision: 0,
        }
    }

    /// Creates a session wrapping an existing document.
    #[must_use]
    pub fn with_document(title: impl Into<String>, document: Document) -> Self {
        let active_surface = document.surfaces().first().map(|s| s.id);
        let max_id = document
            .surfaces()
            .iter()
            .map(|s| s.id.raw())
            .chain(
                document
                    .surfaces()
                    .iter()
                    .flat_map(|s| s.objects().iter().map(|o| o.id.raw())),
            )
            .max()
            .unwrap_or(0);
        Self {
            document,
            history: History::new(0),
            selection: SelectionSession::new(),
            view: crate::view_camera::ViewState::default(),
            id_generator: IdGenerator::with_start(max_id + 1),
            title: title.into(),
            path: None,
            active_surface,
            current_revision: 0,
            saved_revision: 0,
        }
    }

    /// Allocates a new monotonically increasing ObjectId that is guaranteed unique within the document.
    pub fn next_object_id(&mut self) -> ObjectId {
        let max_existing = self
            .document
            .surfaces()
            .iter()
            .flat_map(|s| s.objects().iter().map(|o| o.id.raw()))
            .max()
            .unwrap_or(0);
        let id = self.id_generator.next_object();
        if id.raw() <= max_existing {
            self.id_generator = IdGenerator::with_start(max_existing + 1);
            self.id_generator.next_object()
        } else {
            id
        }
    }

    /// Allocates a new monotonically increasing SurfaceId guaranteed unique within the document.
    pub fn next_surface_id(&mut self) -> SurfaceId {
        let max_existing = self
            .document
            .surfaces()
            .iter()
            .map(|s| s.id.raw())
            .max()
            .unwrap_or(0);
        let id = self.id_generator.next_surface();
        if id.raw() <= max_existing {
            self.id_generator = IdGenerator::with_start(max_existing + 1);
            self.id_generator.next_surface()
        } else {
            id
        }
    }

    /// True if unsaved modifications exist.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.current_revision != self.saved_revision
    }

    /// Marks the current revision as saved.
    pub fn mark_saved(&mut self) {
        self.saved_revision = self.current_revision;
    }

    /// Read-only view of the canonical document (A2).
    #[must_use]
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Read-only view of the undo/redo history (A2).
    #[must_use]
    pub fn history(&self) -> &History {
        &self.history
    }

    /// Session title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Active editing surface, if any.
    #[must_use]
    pub fn active_surface(&self) -> Option<SurfaceId> {
        self.active_surface
    }

    /// Switches the active editing surface.
    pub fn set_active_surface(&mut self, surface: SurfaceId) {
        self.active_surface = Some(surface);
    }

    /// Current monotonic revision.
    #[must_use]
    pub fn current_revision(&self) -> u64 {
        self.current_revision
    }

    /// Revision at last explicit save.
    #[must_use]
    pub fn saved_revision(&self) -> u64 {
        self.saved_revision
    }

    /// Filesystem location of this session, if it has one.
    #[must_use]
    pub fn path(&self) -> Option<&std::path::Path> {
        self.path.as_deref()
    }

    /// Writes the document to `path` as a native `.PTND` package and records
    /// the location as the clean save point.
    ///
    /// The suffix policy lives in `petunia_design_io`: native writes always
    /// carry `.PTND`, so a legacy `.aubrieta`/`.aubri` path is refused here
    /// rather than silently overwriting the user's old file.
    pub fn save_to(&mut self, path: &std::path::Path) -> Result<std::path::PathBuf, PetuniaError> {
        let target = petunia_design_io::with_native_extension(path);
        petunia_design_io::save_package(&self.document, &target)?;
        self.path = Some(target.clone());
        self.title = target
            .file_name()
            .map_or_else(|| self.title.clone(), |name| name.to_string_lossy().into_owned());
        self.mark_saved();
        Ok(target)
    }

    /// Finds an object anywhere in the document.
    #[must_use]
    pub fn find_object(&self, id: ObjectId) -> Option<&DocumentObject> {
        self.document.find_object(id)
    }

    /// Finds a surface by stable ID.
    pub fn surface(&self, id: SurfaceId) -> Result<&Surface, PetuniaError> {
        self.document.surface(id)
    }

    /// All surfaces in document order.
    #[must_use]
    pub fn surfaces(&self) -> &[petunia_design_document::Surface] {
        self.document.surfaces()
    }

    /// Finds a variable data source by stable ID.
    #[must_use]
    pub fn data_source(
        &self,
        id: petunia_design_document::DataSourceId,
    ) -> Option<&DataSourceDefinition> {
        self.document.data_source(id)
    }

    /// All variable data bindings in definition order.
    #[must_use]
    pub fn bindings(&self) -> &[DataBinding] {
        self.document.bindings()
    }

    /// Executes a command request through history, updating the revision and pruning selection.
    /// NoOp change sets prune the selection but never bump the revision,
    /// keeping `current_revision` exact across undo/redo/MCP (F-22).
    pub fn execute_command(&mut self, request: CommandRequest) -> Result<ChangeSet, PetuniaError> {
        let changes = self.history.execute(&mut self.document, &request)?;
        if !changes.is_empty() {
            self.current_revision += 1;
        }
        self.prune_selection();
        Ok(changes)
    }

    /// Dispatches an action request by translating it into validated commands.
    /// Legacy `aubrieta.*` action identifiers are accepted on read and
    /// normalized to `ptnd.*` (15.A); new code never emits the old form.
    pub fn dispatch_action(&mut self, request: ActionRequest) -> Result<ChangeSet, PetuniaError> {
        // Two read-only migration shims, applied in order: the product rename
        // (`aubrieta.*` -> `ptnd.*`) and the action grammar move
        // (`ptnd.<domain>.*` -> `ptnd.action.<domain>.*`, 15.G).
        let renamed = petunia_design_foundation::normalized(request.action.as_str());
        let action = petunia_design_foundation::normalize_action_id(&renamed);
        match action.as_str() {
            "ptnd.action.edit.delete" => {
                let mut combined = ChangeSet::empty();
                let to_delete = self.selection.selected_ids.clone();
                for id in to_delete {
                    let cmd = CommandRequest::new(Command::DeleteObject { id });
                    let changes = self.execute_command(cmd)?;
                    for c in changes.changes {
                        combined.push(c);
                    }
                }
                self.selection.clear();
                Ok(combined)
            }
            "ptnd.action.edit.select_all" => {
                self.select_all();
                Ok(ChangeSet::empty())
            }
            "ptnd.action.edit.deselect" => {
                self.selection.clear();
                Ok(ChangeSet::empty())
            }
            // File actions persist the document; they never mutate it, so the
            // returned ChangeSet stays empty and history is untouched.
            "ptnd.action.file.save" => {
                let path = match self.path.clone() {
                    Some(existing) => existing,
                    None => request_path(&request.payload)?,
                };
                self.save_to(&path)?;
                Ok(ChangeSet::empty())
            }
            "ptnd.action.file.save_as" => {
                let path = request_path(&request.payload)?;
                self.save_to(&path)?;
                Ok(ChangeSet::empty())
            }
            // View actions mutate view state, not the document, so they
            // return an empty ChangeSet: something observable happened, but
            // nothing entered history (15.B).
            "ptnd.action.view.zoom_in" => {
                self.view.zoom_in();
                Ok(ChangeSet::empty())
            }
            "ptnd.action.view.zoom_out" => {
                self.view.zoom_out();
                Ok(ChangeSet::empty())
            }
            "ptnd.action.view.zoom_100" => {
                self.view.camera.reset_100();
                Ok(ChangeSet::empty())
            }
            "ptnd.action.view.fit_surface" => {
                if let Some(id) = self.active_surface() {
                    if let Ok(surface) = self.document.surface(id) {
                        let [x, y, w, h] = surface.bounds();
                        self.view
                            .fit_rect(petunia_design_geometry::GRect::new(x, y, x + w, y + h));
                    }
                }
                Ok(ChangeSet::empty())
            }
            "ptnd.action.view.toggle_rulers" => {
                self.view.toggle_rulers();
                Ok(ChangeSet::empty())
            }
            "ptnd.action.view.toggle_snapping" => {
                self.view.toggle_snapping();
                Ok(ChangeSet::empty())
            }
            "ptnd.action.object.align" => {
                let (surface, ids, mode) = align_payload(&request.payload)?;
                let cmd = CommandRequest::new(Command::AlignObjects { surface, ids, mode });
                self.execute_command(cmd)
            }
            "ptnd.action.object.distribute" => {
                let (surface, ids, axis) = distribute_payload(&request.payload)?;
                let cmd = CommandRequest::new(Command::DistributeObjects { surface, ids, axis });
                self.execute_command(cmd)
            }
            _ => Err(PetuniaError::invalid_input(format!(
                "unsupported action in session: `{}`",
                request.action.as_str()
            ))),
        }
    }

    /// Executes a batch of commands atomically as one undo entry (F-01).
    /// Used by creation gestures (pen/pencil/shape/text/artboard): one
    /// gesture commits exactly one history entry. Empty batches are a NoOp.
    pub fn transact(
        &mut self,
        label: &str,
        cmds: Vec<Command>,
    ) -> Result<ChangeSet, PetuniaError> {
        if cmds.is_empty() {
            return Ok(ChangeSet::empty());
        }
        let mut tx = super::transaction::Transaction::begin(&self.document, label);
        for cmd in cmds {
            tx.update(&CommandRequest::new(cmd))?;
        }
        let staged = tx.staged().clone();
        if staged.is_empty() {
            return Ok(staged);
        }
        tx.commit(&mut self.document, &mut self.history);
        self.current_revision += 1;
        self.prune_selection();
        Ok(staged)
    }

    /// Undoes the last committed command.
    pub fn undo(&mut self) -> Result<bool, PetuniaError> {
        let undone = self.history.undo(&mut self.document)?;
        if undone {
            self.current_revision += 1;
            self.prune_selection();
        }
        Ok(undone)
    }

    /// Redoes the last undone command.
    pub fn redo(&mut self) -> Result<bool, PetuniaError> {
        let redone = self.history.redo(&mut self.document)?;
        if redone {
            self.current_revision += 1;
            self.prune_selection();
        }
        Ok(redone)
    }

    /// Selects all objects on the active surface.
    pub fn select_all(&mut self) {
        if let Some(surface_id) = self.active_surface {
            if let Ok(surface) = self.document.surface(surface_id) {
                let all_ids: Vec<ObjectId> = surface.objects().iter().map(|o| o.id).collect();
                self.selection.select_exact(all_ids);
            }
        }
    }

    /// Prunes selection against currently existing objects in the document.
    pub fn prune_selection(&mut self) {
        let valid_ids: Vec<ObjectId> = self
            .document
            .surfaces()
            .iter()
            .flat_map(|s| s.objects().iter().map(|o| o.id))
            .collect();
        self.selection.prune_missing(&valid_ids);
    }

    /// Resolves high-level document metrics.
    #[must_use]
    pub fn summary(&self) -> DocumentSummary {
        let total_objects = self.document.surfaces().iter().map(|s| s.objects().len()).sum();
        DocumentSummary {
            title: self.title.clone(),
            surface_count: self.document.surfaces().len(),
            total_objects,
            revision: self.current_revision,
            is_dirty: self.is_dirty(),
        }
    }

    /// Builds the selection view model.
    #[must_use]
    pub fn selection_view_model(&self) -> SelectionViewModel {
        let count = self.selection.selected_ids.len();
        let key_object = self.selection.key_object();

        // Calculate combined bounds across all selected objects
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        let mut has_bounds = false;

        for &id in &self.selection.selected_ids {
            if let Some(obj) = self.document.find_object(id) {
                if let Some([x, y, w, h]) = obj.bounds {
                    has_bounds = true;
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x + w);
                    max_y = max_y.max(y + h);
                }
            }
        }

        let combined_bounds = if has_bounds && max_x >= min_x && max_y >= min_y {
            Some([min_x, min_y, max_x - min_x, max_y - min_y])
        } else {
            None
        };

        SelectionViewModel {
            selected_ids: self.selection.selected_ids.clone(),
            key_object,
            combined_bounds,
            count,
            is_empty: count == 0,
        }
    }

    /// Builds a full session snapshot.
    #[must_use]
    pub fn snapshot(&self) -> SessionSnapshot {
        let total_objects = self.document.surfaces().iter().map(|s| s.objects().len()).sum();
        SessionSnapshot {
            active_surface: self.active_surface,
            title: self.title.clone(),
            revision: self.current_revision,
            is_dirty: self.is_dirty(),
            surface_count: self.document.surfaces().len(),
            total_objects,
            selected_count: self.selection.selected_ids.len(),
            can_undo: self.history.can_undo(),
            can_redo: self.history.can_redo(),
        }
    }

    /// Builds the layers panel presentation model.
    #[must_use]
    pub fn layers_presentation_model(&self) -> LayersPresentationModel {
        let mut surfaces = Vec::new();
        let mut rows = Vec::new();

        for surface in self.document.surfaces() {
            let is_active = self.active_surface == Some(surface.id);
            surfaces.push(SurfaceRowViewModel {
                id: surface.id,
                name: surface.name.clone(),
                is_active,
                object_count: surface.objects().len(),
                origin: surface.origin,
                dimensions: surface.dimensions,
                bleed: surface.bleed,
                margins: surface.margins,
                background: surface.background.clone(),
                guide_count: surface.guides.len(),
            });

            let mut visited = std::collections::HashSet::new();

            // First emit root-level objects and recursively their subtrees
            for obj in surface.objects() {
                if obj.parent.is_none() {
                    Self::push_layer_tree_rows(
                        surface,
                        obj,
                        0,
                        &self.selection.selected_ids,
                        &mut rows,
                        &mut visited,
                    );
                }
            }

            // Fallback for any unparented/orphaned nodes
            for obj in surface.objects() {
                if !visited.contains(&obj.id) {
                    Self::push_layer_tree_rows(
                        surface,
                        obj,
                        0,
                        &self.selection.selected_ids,
                        &mut rows,
                        &mut visited,
                    );
                }
            }
        }

        let selected_count = rows.iter().filter(|r| r.is_selected).count();
        let total_count = rows.len();

        LayersPresentationModel {
            surfaces,
            rows,
            total_count,
            selected_count,
        }
    }

    fn push_layer_tree_rows(
        surface: &petunia_design_document::Surface,
        obj: &petunia_design_document::DocumentObject,
        depth: usize,
        selected_ids: &[ObjectId],
        rows: &mut Vec<LayerRowViewModel>,
        visited: &mut std::collections::HashSet<ObjectId>,
    ) {
        if !visited.insert(obj.id) {
            return;
        }
        let is_selected = selected_ids.contains(&obj.id);
        rows.push(LayerRowViewModel {
            id: obj.id,
            surface_id: surface.id,
            name: obj.name.clone(),
            visible: obj.visible,
            locked: obj.locked,
            is_selected,
            depth,
            parent_id: obj.parent,
            is_container: obj.is_container(),
            role: obj.role,
            is_clip_mask: obj.is_clip_mask,
            clip_mask_id: obj.clip_mask_id,
            children_count: obj.children.len(),
            fill_token: obj.fill.clone(),
            stroke_token: obj.stroke.clone(),
            opacity: obj.opacity,
            bounds: obj.bounds,
        });

        for &child_id in &obj.children {
            if let Some(child) = surface.objects().iter().find(|o| o.id == child_id) {
                Self::push_layer_tree_rows(surface, child, depth + 1, selected_ids, rows, visited);
            }
        }
    }

    /// Builds the properties inspector presentation model.
    #[must_use]
    pub fn properties_presentation_model(&self) -> PropertiesPresentationModel {
        if self.selection.selected_ids.is_empty() {
            let active_surface = self.active_surface.and_then(|surf_id| {
                self.document
                    .surface(surf_id)
                    .ok()
                    .map(|s| SurfaceRowViewModel {
                        id: s.id,
                        name: s.name.clone(),
                        is_active: true,
                        object_count: s.objects().len(),
                        origin: s.origin,
                        dimensions: s.dimensions,
                        bleed: s.bleed,
                        margins: s.margins,
                        background: s.background.clone(),
                        guide_count: s.guides.len(),
                    })
            });
            return PropertiesPresentationModel {
                active_surface,
                ..PropertiesPresentationModel::default()
            };
        }

        let key_id = self.selection.key_object();
        let key_obj = key_id.and_then(|id| self.document.find_object(id));

        if self.selection.selected_ids.len() == 1 {
            if let Some(obj) = key_obj {
                return PropertiesPresentationModel {
                    selection_empty: false,
                    is_mixed: false,
                    name: Some(obj.name.clone()),
                    fill: obj.fill.clone(),
                    stroke: obj.stroke.clone(),
                    stroke_width: obj.stroke_width,
                    opacity: obj.opacity,
                    visible: obj.visible,
                    locked: obj.locked,
                    bounds: obj.bounds,
                    rotation: obj.rotation,
                    appearance: obj.appearance.clone(),
                    active_surface: None,
                };
            }
        }

        // Multi-selection: aggregate values
        let selected_objects: Vec<&DocumentObject> = self
            .selection
            .selected_ids
            .iter()
            .filter_map(|&id| self.document.find_object(id))
            .collect();

        let first_fill = selected_objects.first().and_then(|o| o.fill.as_ref());
        let is_mixed_fill = selected_objects
            .iter()
            .any(|o| o.fill.as_ref() != first_fill);

        let sel_vm = self.selection_view_model();

        PropertiesPresentationModel {
            selection_empty: false,
            is_mixed: is_mixed_fill || selected_objects.len() > 1,
            name: key_obj
                .map(|o| format!("{} (and {} others)", o.name, selected_objects.len() - 1)),
            fill: if is_mixed_fill {
                None
            } else {
                first_fill.cloned()
            },
            stroke: None,
            stroke_width: 1.0,
            opacity: 1.0,
            visible: true,
            locked: selected_objects.iter().any(|o| o.locked),
            bounds: sel_vm.combined_bounds,
            rotation: 0.0,
            appearance: None,
            active_surface: None,
        }
    }

    /// Builds the presentation model for the Variable Data / Data Merge panel (10.11).
    #[must_use]
    pub fn data_merge_presentation_model(&self) -> DataMergePresentationModel {
        let mut sources = Vec::new();
        let mut total_records = 0;

        for ds in self.document.data_sources() {
            total_records += ds.records.len();
            let fields: Vec<FieldViewModel> = ds
                .schema
                .fields
                .iter()
                .map(|f| FieldViewModel {
                    id: f.id,
                    name: f.name.clone(),
                    field_type: f.field_type,
                })
                .collect();

            sources.push(DataSourceViewModel {
                id: ds.id,
                name: ds.name.clone(),
                format: ds.format,
                field_count: ds.schema.fields.len(),
                record_count: ds.records.len(),
                fields,
            });
        }

        let mut bindings = Vec::new();
        for b in self.document.bindings() {
            let field_name = self
                .document
                .data_source(b.source_id)
                .and_then(|ds| ds.schema.field(b.field_id))
                .map(|f| f.name.clone())
                .unwrap_or_else(|| format!("Field#{}", b.field_id.raw()));

            let target_object_name = self
                .document
                .find_object(b.target_object)
                .map(|o| o.name.clone())
                .unwrap_or_else(|| format!("Object#{}", b.target_object.raw()));

            bindings.push(DataBindingViewModel {
                id: b.id,
                source_id: b.source_id,
                field_id: b.field_id,
                field_name,
                target_object: b.target_object,
                target_object_name,
                target_property: b.target_property,
                formatter: b.formatter.clone(),
            });
        }

        DataMergePresentationModel {
            sources,
            bindings,
            preview_record: None,
            total_records,
        }
    }
}

/// Parses an align payload: `{surface: u64, ids: [u64], mode: left|center|right|top|middle|bottom}`.
fn align_payload(
    payload: &serde_json::Value,
) -> Result<
    (
        SurfaceId,
        Vec<ObjectId>,
        AlignmentMode,
    ),
    PetuniaError,
> {
    let surface = payload
        .get("surface")
        .and_then(serde_json::Value::as_u64)
        .map(SurfaceId::new)
        .ok_or_else(|| PetuniaError::invalid_input("align payload requires `surface` id"))?;
    let ids = payload
        .get("ids")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_u64().map(ObjectId::new))
                .collect::<Vec<_>>()
        })
        .filter(|ids| ids.len() >= 2)
        .ok_or_else(|| {
            PetuniaError::invalid_input("align payload requires at least 2 `ids`")
        })?;
    let mode = match payload.get("mode").and_then(serde_json::Value::as_str) {
        Some("left") => AlignmentMode::Left,
        Some("center") => AlignmentMode::Center,
        Some("right") => AlignmentMode::Right,
        Some("top") => AlignmentMode::Top,
        Some("middle") => AlignmentMode::Middle,
        Some("bottom") => AlignmentMode::Bottom,
        other => {
            return Err(PetuniaError::invalid_input(format!(
                "align payload requires mode left|center|right|top|middle|bottom, got {other:?}"
            )));
        }
    };
    Ok((surface, ids, mode))
}

/// Parses a distribute payload: `{surface: u64, ids: [u64], axis: horizontal|vertical}`.
fn distribute_payload(
    payload: &serde_json::Value,
) -> Result<
    (
        SurfaceId,
        Vec<ObjectId>,
        DistributionAxis,
    ),
    PetuniaError,
> {
    let surface = payload
        .get("surface")
        .and_then(serde_json::Value::as_u64)
        .map(SurfaceId::new)
        .ok_or_else(|| {
            PetuniaError::invalid_input("distribute payload requires `surface` id")
        })?;
    let ids = payload
        .get("ids")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_u64().map(ObjectId::new))
                .collect::<Vec<_>>()
        })
        .filter(|ids| ids.len() >= 3)
        .ok_or_else(|| {
            PetuniaError::invalid_input("distribute payload requires at least 3 `ids`")
        })?;
    let axis = match payload.get("axis").and_then(serde_json::Value::as_str) {
        Some("horizontal") => DistributionAxis::Horizontal,
        Some("vertical") => DistributionAxis::Vertical,
        other => {
            return Err(PetuniaError::invalid_input(format!(
                "distribute payload requires axis horizontal|vertical, got {other:?}"
            )));
        }
    };
    Ok((surface, ids, axis))
}

#[cfg(test)]
mod view_action_tests {
    use super::*;
    use crate::ActionId;
    use serde_json::json;

    fn session() -> DocumentSession {
        DocumentSession::new("view")
    }

    fn dispatch(session: &mut DocumentSession, action: &str) -> ChangeSet {
        session
            .dispatch_action(ActionRequest::new(ActionId::new(action), json!({})))
            .expect("view action must dispatch")
    }

    #[test]
    fn view_actions_change_view_state_without_touching_the_document() {
        let mut session = session();
        let revision = session.current_revision();

        dispatch(&mut session, "ptnd.action.view.zoom_in");
        assert!(session.view.camera.zoom > 1.0);

        dispatch(&mut session, "ptnd.action.view.zoom_out");
        assert!((session.view.camera.zoom - 1.0).abs() < 1e-9);

        dispatch(&mut session, "ptnd.action.view.zoom_in");
        dispatch(&mut session, "ptnd.action.view.zoom_100");
        assert!((session.view.camera.zoom - 1.0).abs() < 1e-9);

        assert_eq!(session.current_revision(), revision, "view must not mutate the document");
    }

    #[test]
    fn view_toggles_flip_and_are_idempotent_per_call() {
        let mut session = session();
        let rulers = session.view.rulers_visible;
        let snapping = session.view.snapping_enabled;

        dispatch(&mut session, "ptnd.action.view.toggle_rulers");
        dispatch(&mut session, "ptnd.action.view.toggle_snapping");
        assert_eq!(session.view.rulers_visible, !rulers);
        assert_eq!(session.view.snapping_enabled, !snapping);

        dispatch(&mut session, "ptnd.action.view.toggle_rulers");
        dispatch(&mut session, "ptnd.action.view.toggle_snapping");
        assert_eq!(session.view.rulers_visible, rulers);
        assert_eq!(session.view.snapping_enabled, snapping);
    }

    #[test]
    fn fit_surface_is_a_safe_no_op_without_an_active_surface() {
        // `surface.create` is declared but not dispatched yet, so a fresh
        // session has nothing to frame. The action must stay harmless; the
        // framing math itself is covered by `view_camera::view_state_tests`.
        let mut session = session();
        let camera = session.view.camera.clone();
        dispatch(&mut session, "ptnd.action.view.fit_surface");
        assert_eq!(session.view.camera, camera);
        assert!(session.active_surface().is_none());
    }

    #[test]
    fn pre_grammar_view_ids_still_dispatch() {
        let mut session = session();
        // A project or plugin written before the grammar move sends the old id.
        dispatch(&mut session, "ptnd.view.zoom_in");
        assert!(session.view.camera.zoom > 1.0);
    }
}

/// Extracts the mandatory `path` string from a file-action payload.
fn request_path(payload: &serde_json::Value) -> Result<std::path::PathBuf, PetuniaError> {
    payload
        .get("path")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(std::path::PathBuf::from)
        .ok_or_else(|| {
            PetuniaError::invalid_input("file action requires a non-empty `path` payload field")
        })
}

#[cfg(test)]
mod file_action_tests {
    use super::*;
    use crate::ActionId;
    use serde_json::json;

    fn dir(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("petunia-design-file-actions-{name}"));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("temp dir");
        d
    }

    fn dispatch_with(
        session: &mut DocumentSession,
        action: &str,
        payload: serde_json::Value,
    ) -> Result<ChangeSet, PetuniaError> {
        session.dispatch_action(ActionRequest::new(ActionId::new(action), payload))
    }

    #[test]
    fn save_as_writes_a_ptnd_package_and_clears_the_dirty_flag() {
        let dir = dir("save-as");
        let target = dir.join("project");
        let mut session = DocumentSession::new("untitled");

        dispatch_with(
            &mut session,
            "ptnd.action.file.save_as",
            json!({ "path": target.to_string_lossy() }),
        )
        .expect("save_as must succeed");

        // The io layer owns the suffix policy: `.PTND` is appended.
        let written = dir.join("project.PTND");
        assert!(written.exists(), "expected {}", written.display());
        assert_eq!(session.path(), Some(written.as_path()));
        assert!(!session.is_dirty(), "saving establishes a clean save point");
    }

    #[test]
    fn save_reuses_the_recorded_path_without_a_payload() {
        let dir = dir("save-reuse");
        let target = dir.join("reuse.PTND");
        let mut session = DocumentSession::new("untitled");
        dispatch_with(
            &mut session,
            "ptnd.action.file.save_as",
            json!({ "path": target.to_string_lossy() }),
        )
        .unwrap();

        // Diverge from the save point, then save again with no payload.
        session
            .dispatch_action(ActionRequest::new(
                ActionId::new("ptnd.action.edit.delete"),
                json!({}),
            ))
            .ok();
        let modified = std::fs::metadata(&target).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));

        dispatch_with(&mut session, "ptnd.action.file.save", json!({}))
            .expect("save must reuse the recorded path");
        let rewritten = std::fs::metadata(&target).unwrap().modified().unwrap();
        assert!(rewritten > modified, "save must rewrite the package");
    }

    #[test]
    fn save_without_a_path_or_payload_is_rejected_with_a_reason() {
        let mut session = DocumentSession::new("untitled");
        let error = dispatch_with(&mut session, "ptnd.action.file.save", json!({}))
            .expect_err("a never-saved project cannot guess a location");
        assert!(
            error.to_string().contains("path"),
            "the rejection must name the missing field: {error}"
        );
    }

    #[test]
    fn file_actions_never_enter_document_history() {
        let dir = dir("no-history");
        let mut session = DocumentSession::new("untitled");
        let revision = session.current_revision();
        dispatch_with(
            &mut session,
            "ptnd.action.file.save_as",
            json!({ "path": dir.join("h.PTND").to_string_lossy() }),
        )
        .unwrap();
        assert_eq!(session.current_revision(), revision);
    }

    #[test]
    fn a_legacy_suffix_is_upgraded_instead_of_overwritten() {
        let dir = dir("legacy-upgrade");
        let legacy = dir.join("old.aubrieta");
        let mut session = DocumentSession::new("untitled");
        let written = dispatch_with(
            &mut session,
            "ptnd.action.file.save_as",
            json!({ "path": legacy.to_string_lossy() }),
        )
        .map(|_| session.path().map(std::path::Path::to_path_buf))
        .unwrap()
        .unwrap();
        assert_eq!(written, dir.join("old.PTND"));
        assert!(!legacy.exists(), "the legacy file must be left untouched");
    }
}
