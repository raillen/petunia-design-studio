//! Undo/redo history over [`aubrieta_document::ChangeSet`] entries.

use aubrieta_document::{ChangeSet, Document};
use aubrieta_foundation::AubrietaError;

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

    /// Executes a command, records its change set, clears redo.
    pub fn execute(
        &mut self,
        document: &mut Document,
        request: &CommandRequest,
    ) -> Result<ChangeSet, AubrietaError> {
        let changes = commands::execute(document, request)?;
        if self.limit > 0 && self.undo.len() >= self.limit {
            self.undo.remove(0);
        }
        self.undo.push(changes.clone());
        self.redo.clear();
        Ok(changes)
    }

    /// Undoes the most recent entry. Returns false when history is empty.
    pub fn undo(&mut self, document: &mut Document) -> Result<bool, AubrietaError> {
        let Some(changes) = self.undo.pop() else {
            return Ok(false);
        };
        DocumentMutator::new(document).revert(&changes)?;
        self.redo.push(changes);
        Ok(true)
    }

    /// Redoes the most recently undone entry. Returns false when empty.
    pub fn redo(&mut self, document: &mut Document) -> Result<bool, AubrietaError> {
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

use aubrieta_document::Change;
use aubrieta_document::DocumentMutator;

/// Re-applies a change set forward (redo primitive).
struct Replayer;

impl Replayer {
    fn replay(document: &mut Document, changes: &ChangeSet) -> Result<(), AubrietaError> {
        let mut mutator = DocumentMutator::new(document);
        for change in &changes.changes {
            match change.clone() {
                Change::SurfaceAdded { id, name } => {
                    mutator.add_surface(id, name)?;
                }
                Change::ObjectAdded { surface, object } => {
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
                Change::ObjectReordered {
                    surface,
                    id,
                    next_index,
                    ..
                } => {
                    mutator.reorder_object(surface, id, next_index)?;
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
    use aubrieta_foundation::IdGenerator;

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
}
