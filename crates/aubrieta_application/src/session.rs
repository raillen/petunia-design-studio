//! Document session ownership and selection session management (09.24).
//!
//! Moved from the legacy GUI bridge crate: session ownership is domain
//! logic (document + history + revision + selection rules), toolkit-free.
//! `document` and `history` are private: all mutations flow through
//! `execute_command` / `dispatch_action` / `undo` / `redo` (A2).

use aubrieta_document::{
    ChangeSet, DataBinding, DataSourceDefinition, Document, DocumentObject, Surface,
};
use aubrieta_foundation::{AubrietaError, IdGenerator, ObjectId, SurfaceId};

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
    /// Monotonic ID generator for session-originated objects.
    id_generator: IdGenerator,
    /// Document title or filename.
    title: String,
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
            id_generator: IdGenerator::with_start(max_id + 1),
            title: title.into(),
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
            id_generator: IdGenerator::with_start(max_id + 1),
            title: title.into(),
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

    /// Finds an object anywhere in the document.
    #[must_use]
    pub fn find_object(&self, id: ObjectId) -> Option<&DocumentObject> {
        self.document.find_object(id)
    }

    /// Finds a surface by stable ID.
    pub fn surface(&self, id: SurfaceId) -> Result<&Surface, AubrietaError> {
        self.document.surface(id)
    }

    /// All surfaces in document order.
    #[must_use]
    pub fn surfaces(&self) -> &[aubrieta_document::Surface] {
        self.document.surfaces()
    }

    /// Finds a variable data source by stable ID.
    #[must_use]
    pub fn data_source(
        &self,
        id: aubrieta_document::DataSourceId,
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
    pub fn execute_command(&mut self, request: CommandRequest) -> Result<ChangeSet, AubrietaError> {
        let changes = self.history.execute(&mut self.document, &request)?;
        if !changes.is_empty() {
            self.current_revision += 1;
        }
        self.prune_selection();
        Ok(changes)
    }

    /// Dispatches an action request by translating it into validated commands.
    pub fn dispatch_action(&mut self, request: ActionRequest) -> Result<ChangeSet, AubrietaError> {
        match request.action.as_str() {
            "aubrieta.edit.delete" => {
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
            "aubrieta.edit.select_all" => {
                self.select_all();
                Ok(ChangeSet::empty())
            }
            "aubrieta.edit.deselect" => {
                self.selection.clear();
                Ok(ChangeSet::empty())
            }
            _ => Err(AubrietaError::invalid_input(format!(
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
    ) -> Result<ChangeSet, AubrietaError> {
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
    pub fn undo(&mut self) -> Result<bool, AubrietaError> {
        let undone = self.history.undo(&mut self.document)?;
        if undone {
            self.current_revision += 1;
            self.prune_selection();
        }
        Ok(undone)
    }

    /// Redoes the last undone command.
    pub fn redo(&mut self) -> Result<bool, AubrietaError> {
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
        surface: &aubrieta_document::Surface,
        obj: &aubrieta_document::DocumentObject,
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
