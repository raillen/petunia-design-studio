//! Undo/redo history over [`petunia_design_document::ChangeSet`] entries.

use petunia_design_document::{ChangeSet, Document};
use petunia_design_foundation::PetuniaError;

use crate::commands::{self, CommandRequest};

/// Bounded undo stack. Redo is cleared on every new execution.
#[derive(Debug, Default)]
pub struct History {
    /// Executed entries, oldest first.
    undo: Vec<ChangeSet>,
    /// Undone entries available for redo.
    redo: Vec<ChangeSet>,
    /// Maximum retained undo entries.
    limit: usize,
}

impl History {
    /// Creates history with a retention limit (0 means unbounded for MVP).
    #[must_use]
    pub fn new(limit: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            limit,
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
        let changes = commands::execute(document, request)?;
        if changes.is_empty() {
            return Ok(changes);
        }
        self.record(changes.clone());
        Ok(changes)
    }

    /// Records an externally built change set (e.g. `Transaction::commit`).
    /// NoOps are ignored; redo is cleared only for real commits.
    pub fn record(&mut self, changes: ChangeSet) {
        if changes.is_empty() {
            return;
        }
        if self.limit > 0 && self.undo.len() >= self.limit {
            self.undo.remove(0);
        }
        self.undo.push(changes);
        self.redo.clear();
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
        let Some(changes) = self.undo.pop() else {
            return Ok(false);
        };
        DocumentMutator::new(document).revert(&changes)?;
        self.redo.push(changes);
        Ok(true)
    }

    /// Redoes the most recently undone entry. Returns false when empty.
    pub fn redo(&mut self, document: &mut Document) -> Result<bool, PetuniaError> {
        let Some(changes) = self.redo.pop() else {
            return Ok(false);
        };
        Replayer::replay(document, &changes)?;
        self.undo.push(changes);
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
                Change::ObjectAdded { surface, object, .. } => {
                    mutator.add_object(surface, object)?;
                }
                Change::ObjectRemoved { object, .. } => {
                    mutator.remove_object(object.id)?;
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
