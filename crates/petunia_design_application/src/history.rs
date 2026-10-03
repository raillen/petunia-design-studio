//! Undo/redo history over [`petunia_design_document::ChangeSet`] entries.

use petunia_design_document::{ChangeSet, Document};
use petunia_design_foundation::PetuniaError;

use crate::commands::{self, CommandRequest};

#[derive(Default)]
struct PayloadSize(usize);
impl std::io::Write for PayloadSize {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self.0.saturating_add(bytes.len());
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// Count retained immutable resource bytes without serializing millions of
// byte-array elements on the UI thread. Shape replacements charge only tiles
// whose Arc identity changed; descriptors/maps are charged separately.
fn bounded_history_payload(changes: &ChangeSet) -> (ChangeSet, usize) {
    use petunia_design_document::{Change, ShapeKind};
    fn strip(shape: &mut Option<ShapeKind>) -> usize {
        let bytes = match shape {
            Some(ShapeKind::Raster { layer }) => layer
                .resident_bytes()
                .saturating_add(layer.cmyk_profile().map_or(0, |p| p.bytes().len()))
                .saturating_add(layer.tiles().resident_tile_count() * 64)
                .saturating_add(256),
            Some(ShapeKind::Image {
                data: Some(source), ..
            }) => source.resident_bytes(),
            _ => return 0,
        };
        match shape {
            Some(ShapeKind::Image { data, .. }) => *data = None,
            _ => *shape = None,
        }
        bytes
    }
    let mut metadata = changes.clone();
    let mut resources = 0usize;
    for change in &mut metadata.changes {
        let bytes = match change {
            Change::ShapeChanged { previous, next, .. } => {
                let bytes = match (&*previous, &*next) {
                    (
                        Some(ShapeKind::Raster { layer: a }),
                        Some(ShapeKind::Raster { layer: b }),
                    ) => a
                        .tiles()
                        .changed_retained_bytes(b.tiles())
                        .saturating_add(
                            if a.cmyk_profile().map(|p| p.id()) == b.cmyk_profile().map(|p| p.id())
                            {
                                0
                            } else {
                                a.cmyk_profile()
                                    .map_or(0, |p| p.bytes().len())
                                    .saturating_add(b.cmyk_profile().map_or(0, |p| p.bytes().len()))
                            },
                        )
                        .saturating_add(512),
                    (
                        Some(ShapeKind::Image { data: Some(a), .. }),
                        Some(ShapeKind::Image { data: Some(b), .. }),
                    ) if std::sync::Arc::ptr_eq(a, b) => 0,
                    _ => strip(previous).saturating_add(strip(next)),
                };
                strip(previous);
                strip(next);
                bytes
            }
            Change::ObjectAdded { object, .. } | Change::ObjectRemoved { object, .. } => {
                strip(&mut object.shape)
            }
            Change::SurfaceCmykProfileChanged { previous, next, .. } => {
                let bytes = previous
                    .as_ref()
                    .map_or(0, |p| p.bytes().len())
                    .saturating_add(next.as_ref().map_or(0, |p| p.bytes().len()));
                *previous = None;
                *next = None;
                bytes
            }
            Change::BatchSurfacesAdded { surfaces } => {
                let mut bytes = 0usize;
                for surface in surfaces {
                    bytes = bytes.saturating_add(
                        surface.cmyk_profile.as_ref().map_or(0, |p| p.bytes().len()),
                    );
                    let mut objects = surface.objects().to_vec();
                    for object in &mut objects {
                        bytes = bytes.saturating_add(strip(&mut object.shape));
                    }
                    // Metadata-only projection; the retained ChangeSet and the
                    // canonical document are never modified by accounting.
                    let mut descriptor = petunia_design_document::Surface::with_objects(
                        surface.id,
                        surface.name.clone(),
                        objects,
                    );
                    descriptor.origin = surface.origin;
                    descriptor.dimensions = surface.dimensions;
                    descriptor.background = surface.background.clone();
                    descriptor.bleed = surface.bleed;
                    descriptor.margins = surface.margins;
                    descriptor.guides = surface.guides.clone();
                    descriptor.export_enabled = surface.export_enabled;
                    *surface = descriptor;
                }
                bytes
            }
            _ => 0,
        };
        resources = resources.saturating_add(bytes);
    }
    (metadata, resources)
}

/// Bounded undo stack. Redo is cleared on every new execution.
#[derive(Debug)]
pub struct History {
    /// Executed entries, oldest first.
    undo: Vec<ChangeSet>,
    /// Undone entries available for redo.
    redo: Vec<ChangeSet>,
    /// Maximum retained undo entries.
    limit: usize,
    undo_states: Vec<(u64, u64, usize)>,
    redo_states: Vec<(u64, u64, usize)>,
    state: u64,
    next_state: u64,
    retained_bytes: usize,
    byte_limit: usize,
}

impl Default for History {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl History {
    /// Creates bounded history. Zero selects the default 1000-entry limit.
    /// The payload budget is an encoded-size estimate, not a process RSS cap.
    #[must_use]
    pub fn new(limit: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            limit: if limit == 0 { 1000 } else { limit },
            undo_states: Vec::new(),
            redo_states: Vec::new(),
            state: 0,
            next_state: 1,
            retained_bytes: 0,
            byte_limit: 512 * 1024 * 1024,
        }
    }

    /// Executes a command, records its change set, clears redo (F-22).
    /// Empty change sets are a `NoOp`: they are returned but never pushed
    /// and never clear the redo stack, so alignment with zero bounds or
    /// redundant sets do not pollute undo.
    pub fn execute(
        &mut self,
        document: &mut Document,
        request: &CommandRequest,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut next = document.clone();
        let changes = commands::execute(&mut next, request)?;
        if changes.is_empty() {
            return Ok(changes);
        }
        next.validate()?;
        self.record(changes.clone())?;
        *document = next;
        Ok(changes)
    }

    /// Records an externally built change set (e.g. `Transaction::commit`).
    /// NoOps are ignored; redo is cleared only for real commits.
    pub fn record(&mut self, changes: ChangeSet) -> Result<(), PetuniaError> {
        if changes.is_empty() {
            return Ok(());
        }
        let mut size = PayloadSize::default();
        let (metadata, resources) = bounded_history_payload(&changes);
        serde_json::to_writer(&mut size, &metadata)
            .map_err(|e| PetuniaError::invalid_input(format!("history payload: {e}")))?;
        let bytes = size.0.saturating_add(resources);
        if bytes > self.byte_limit {
            return Err(PetuniaError::invalid_input(
                "edit exceeds history payload budget; document was not changed",
            ));
        }
        for (_, _, size) in self.redo_states.drain(..) {
            self.retained_bytes = self.retained_bytes.saturating_sub(size);
        }
        self.redo.clear();
        while !self.undo.is_empty()
            && (self.undo.len() >= self.limit
                || self.retained_bytes.saturating_add(bytes) > self.byte_limit)
        {
            self.undo.remove(0);
            let (_, _, size) = self.undo_states.remove(0);
            self.retained_bytes = self.retained_bytes.saturating_sub(size);
        }
        let before = self.state;
        self.state = self.next_state;
        self.next_state = self
            .next_state
            .checked_add(1)
            .expect("history state space exhausted");
        self.undo.push(changes);
        self.undo_states.push((before, self.state, bytes));
        self.retained_bytes = self.retained_bytes.saturating_add(bytes);
        Ok(())
    }

    /// Executes a command and reports whether it committed (F-22).
    pub fn execute_result(
        &mut self,
        document: &mut Document,
        request: &CommandRequest,
    ) -> Result<crate::commands::CommandResult, PetuniaError> {
        let changes = self.execute(document, request)?;
        if changes.is_empty() {
            Ok(crate::commands::CommandResult::NoOp)
        } else {
            Ok(crate::commands::CommandResult::Committed(changes))
        }
    }

    /// Undoes the most recent entry. Returns false when history is empty.
    pub fn undo(&mut self, document: &mut Document) -> Result<bool, PetuniaError> {
        let Some(changes) = self.undo.last() else {
            return Ok(false);
        };
        let mut next = document.clone();
        DocumentMutator::new(&mut next).revert(changes)?;
        next.validate()?;
        *document = next;
        let changes = self.undo.pop().expect("entry checked above");
        let states = self.undo_states.pop().expect("history metadata is paired");
        self.state = states.0;
        self.redo.push(changes);
        self.redo_states.push(states);
        Ok(true)
    }

    /// Redoes the most recently undone entry. Returns false when empty.
    pub fn redo(&mut self, document: &mut Document) -> Result<bool, PetuniaError> {
        let Some(changes) = self.redo.last() else {
            return Ok(false);
        };
        let mut next = document.clone();
        Replayer::replay(&mut next, changes)?;
        next.validate()?;
        *document = next;
        let changes = self.redo.pop().expect("entry checked above");
        let states = self.redo_states.pop().expect("history metadata is paired");
        self.state = states.1;
        self.undo.push(changes);
        self.undo_states.push(states);
        Ok(true)
    }

    /// Number of undoable entries.
    #[must_use]
    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    /// Number of redoable entries.
    #[must_use]
    pub fn redo_len(&self) -> usize {
        self.redo.len()
    }

    /// True when undo is available.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// True when redo is available.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Slice of undo entries.
    #[must_use]
    pub fn undo_entries(&self) -> &[ChangeSet] {
        &self.undo
    }

    /// Slice of redo entries.
    #[must_use]
    pub fn redo_entries(&self) -> &[ChangeSet] {
        &self.redo
    }

    /// Stable identity of document content across undo and redo.
    #[must_use]
    pub fn state_id(&self) -> u64 {
        self.state
    }

    /// Approximate retained encoded payload bytes across both stacks.
    #[must_use]
    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    /// Configurable retention budget; oversized edits fail before publication.
    #[must_use]
    pub fn with_budget(limit: usize, byte_limit: usize) -> Self {
        Self {
            byte_limit,
            ..Self::new(limit)
        }
    }
}

use petunia_design_document::Change;
use petunia_design_document::DocumentMutator;

/// Re-applies a change set forward (redo primitive).
struct Replayer;

impl Replayer {
    fn replay(document: &mut Document, changes: &ChangeSet) -> Result<(), PetuniaError> {
        let mut mutator = DocumentMutator::new(document);
        for change in &changes.changes {
            match change.clone() {
                Change::SurfaceAdded { id, name } => {
                    mutator.add_surface(id, name)?;
                }
                Change::ObjectAdded {
                    surface, object, ..
                } => {
                    mutator.add_object(surface, object)?;
                }
                Change::ObjectRemoved { object, .. } => {
                    mutator.remove_object(object.id)?;
                }
                Change::NameChanged { id, next, .. } => {
                    mutator.rename_object(id, next)?;
                }
                Change::FillChanged { id, next, .. } => {
                    mutator.set_fill(id, next)?;
                }
                Change::VisibilityChanged { id, next, .. } => {
                    mutator.set_visibility(id, next)?;
                }
                Change::LockChanged { id, next, .. } => {
                    mutator.set_locked(id, next)?;
                }
                Change::OpacityChanged { id, next, .. } => {
                    mutator.set_opacity(id, next)?;
                }
                Change::StrokeChanged {
                    id,
                    next_stroke,
                    next_width,
                    ..
                } => {
                    mutator.set_stroke(id, next_stroke, next_width)?;
                }
                Change::BoundsChanged {
                    id,
                    next_bounds,
                    next_rotation,
                    ..
                } => {
                    mutator.set_bounds(id, next_bounds, next_rotation)?;
                }
                Change::TextStyleChanged { id, next, .. } => {
                    mutator.set_text_style(id, next)?;
                }
                Change::ShapeChanged { id, next, .. } => {
                    mutator.set_shape(id, next)?;
                }
                Change::ObjectReordered {
                    surface,
                    id,
                    next_index,
                    ..
                } => {
                    mutator.reorder_object(surface, id, next_index)?;
                }
                Change::AppearanceChanged { id, next, .. } => {
                    mutator.set_appearance(id, next)?;
                }
                Change::ModifiersChanged { id, next, .. } => {
                    mutator.set_modifiers(id, next)?;
                }
                Change::Reparented {
                    id, next_parent, ..
                } => {
                    mutator.force_parent(id, next_parent)?;
                }
                Change::ChildrenChanged {
                    id, next_children, ..
                } => {
                    mutator.force_children(id, next_children)?;
                }
                Change::ContainerRoleChanged { id, next, .. } => {
                    mutator.force_role(id, next)?;
                }
                Change::MaskModeChanged { id, next, .. } => {
                    mutator.set_mask_mode(id, next)?;
                }
                Change::ClipMaskChanged {
                    id,
                    next_mask,
                    next_is_mask,
                    ..
                } => {
                    mutator.force_clip_mask(id, next_mask, next_is_mask)?;
                }
                Change::SurfaceGeometryChanged {
                    id,
                    next_origin,
                    next_dimensions,
                    ..
                } => {
                    mutator.set_surface_geometry(id, next_origin, next_dimensions)?;
                }
                Change::SurfaceBleedChanged { id, next, .. } => {
                    mutator.set_surface_bleed(id, next)?;
                }
                Change::SurfaceMarginsChanged { id, next, .. } => {
                    mutator.set_surface_margins(id, next)?;
                }
                Change::SurfaceBackgroundChanged { id, next, .. } => {
                    mutator.set_surface_background(id, next)?;
                }
                Change::SurfaceCmykProfileChanged { id, next, .. } => {
                    mutator.set_surface_cmyk_profile(id, next)?;
                }
                Change::SurfaceExportEnabledChanged { id, next, .. } => {
                    mutator.set_surface_export_enabled(id, next)?;
                }
                Change::SurfaceGuideAdded { surface, guide } => {
                    mutator.add_surface_guide(surface, guide)?;
                }
                Change::SurfaceGuideRemoved { surface, guide } => {
                    mutator.remove_surface_guide(surface, guide.id)?;
                }
                Change::DataSourceAdded { source } => {
                    mutator.add_data_source(source)?;
                }
                Change::DataSourceRemoved { source } => {
                    mutator.remove_data_source(source.id)?;
                }
                Change::DataBindingAdded { binding } => {
                    mutator.add_data_binding(binding)?;
                }
                Change::DataBindingRemoved { binding } => {
                    mutator.remove_data_binding(binding.id)?;
                }
                Change::BatchSurfacesAdded { surfaces } => {
                    for s in surfaces {
                        mutator.attach_surface(s);
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{Command, CommandRequest};
    use petunia_design_foundation::IdGenerator;

    #[test]
    fn execute_undo_redo_roundtrip() {
        let mut gen = IdGenerator::new();
        let mut doc = Document::default();
        let mut history = History::new(100);
        let surface = gen.next_surface();
        let object = gen.next_object();

        history
            .execute(
                &mut doc,
                &CommandRequest::new(Command::CreateSurface {
                    id: surface,
                    name: "Page".to_string(),
                }),
            )
            .expect("surface");
        history
            .execute(
                &mut doc,
                &CommandRequest::new(Command::CreateObject {
                    surface,
                    id: object,
                    name: "Rect".to_string(),
                }),
            )
            .expect("object");
        assert!(doc.find_object(object).is_some());

        assert!(history.undo(&mut doc).expect("undo"));
        assert!(doc.find_object(object).is_none());
        assert!(history.redo(&mut doc).expect("redo"));
        assert!(doc.find_object(object).is_some());
    }

    #[test]
    fn execute_undo_redo_grouping_and_reparenting() {
        let mut gen = IdGenerator::new();
        let mut doc = Document::default();
        let mut history = History::new(100);
        let surface = gen.next_surface();
        let o1 = gen.next_object();
        let o2 = gen.next_object();
        let grp = gen.next_object();

        history
            .execute(
                &mut doc,
                &CommandRequest::new(Command::CreateSurface {
                    id: surface,
                    name: "Page".to_string(),
                }),
            )
            .unwrap();
        history
            .execute(
                &mut doc,
                &CommandRequest::new(Command::CreateObject {
                    surface,
                    id: o1,
                    name: "Obj1".to_string(),
                }),
            )
            .unwrap();
        history
            .execute(
                &mut doc,
                &CommandRequest::new(Command::CreateObject {
                    surface,
                    id: o2,
                    name: "Obj2".to_string(),
                }),
            )
            .unwrap();

        // Group objects
        history
            .execute(
                &mut doc,
                &CommandRequest::new(Command::GroupObjects {
                    surface,
                    group_id: grp,
                    child_ids: vec![o1, o2],
                    role: petunia_design_document::ContainerRole::Group,
                }),
            )
            .unwrap();

        assert_eq!(doc.find_object(o1).unwrap().parent, Some(grp));

        // Undo grouping
        history.undo(&mut doc).unwrap();
        assert_eq!(doc.find_object(o1).unwrap().parent, None);
        assert!(doc.find_object(grp).is_none());

        // Redo grouping
        history.redo(&mut doc).unwrap();
        assert_eq!(doc.find_object(o1).unwrap().parent, Some(grp));
        assert!(doc.find_object(grp).is_some());
    }

    #[test]
    fn execute_undo_redo_variable_data_merge() {
        use petunia_design_document::{
            BindingId, DataBinding, DataSourceId, DataSourceParser, MissingValuePolicy,
            TargetProperty, ValueFormatter,
        };

        let mut gen = IdGenerator::new();
        let mut doc = Document::default();
        let mut history = History::new(100);
        let surface = gen.next_surface();
        let object = gen.next_object();

        history
            .execute(
                &mut doc,
                &CommandRequest::new(Command::CreateSurface {
                    id: surface,
                    name: "Card".to_string(),
                }),
            )
            .unwrap();
        history
            .execute(
                &mut doc,
                &CommandRequest::new(Command::CreateObject {
                    surface,
                    id: object,
                    name: "Title".to_string(),
                }),
            )
            .unwrap();

        // 1. Add Data Source
        let csv = "Name\nAda\nAlan";
        let ds =
            DataSourceParser::parse_delimited(DataSourceId::new(1), "data.csv", csv, ',').unwrap();
        history
            .execute(
                &mut doc,
                &CommandRequest::new(Command::AddDataSource { source: ds }),
            )
            .unwrap();
        assert_eq!(doc.data_sources().len(), 1);

        // 2. Add Data Binding
        let binding = DataBinding {
            id: BindingId::new(1),
            source_id: DataSourceId::new(1),
            field_id: petunia_design_document::FieldId::new(1),
            target_object: object,
            target_property: TargetProperty::TextContent,
            formatter: ValueFormatter::Uppercase,
            missing_policy: MissingValuePolicy::Skip,
        };
        history
            .execute(
                &mut doc,
                &CommandRequest::new(Command::AddDataBinding { binding }),
            )
            .unwrap();
        assert_eq!(doc.bindings().len(), 1);

        // 3. Materialize merge (2 records -> 2 new surfaces)
        history
            .execute(
                &mut doc,
                &CommandRequest::new(Command::MaterializeDataMerge {
                    source_id: DataSourceId::new(1),
                    template_surface: surface,
                }),
            )
            .unwrap();
        assert_eq!(doc.surfaces().len(), 3);

        // 4. Undo merge
        assert!(history.undo(&mut doc).unwrap());
        assert_eq!(doc.surfaces().len(), 1);

        // 5. Redo merge
        assert!(history.redo(&mut doc).unwrap());
        assert_eq!(doc.surfaces().len(), 3);
    }
}
