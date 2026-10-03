//! Interactive transaction: preview on a working copy, commit atomically,
//! cancel with exactness (F-01, 09.3).
//!
//! Tools must not `submit_command` on every pointer move (which flooded undo
//! with N entries). Instead: `begin` clones the document, `update` stages
//! commands on the working copy for overlay preview, `commit` pushes one
//! combined `ChangeSet` to `History`, `cancel` discards without touching the
//! real document or history.

use petunia_design_document::{Change, ChangeSet, Document};
use petunia_design_foundation::ObjectId;
use petunia_design_foundation::PetuniaError;
use std::{collections::HashMap, mem::Discriminant};

use crate::commands::{self, CommandRequest};
use crate::history::History;

/// One interactive gesture staged off-document.
#[derive(Debug)]
pub struct Transaction {
    /// Snapshot compared at commit, protecting intervening command writes.
    baseline: Document,
    /// Working copy receiving staged commands.
    working: Document,
    /// Combined staged changes in application order.
    staged: ChangeSet,
    /// One retained before/after pair per property within a structural epoch.
    properties: HashMap<(Discriminant<Change>, ObjectId), usize>,
    /// Human label for history panels (e.g. `"Move"`, `"Resize"`).
    label: String,
}

impl Transaction {
    /// Begins a transaction from the live document snapshot.
    #[must_use]
    pub fn begin(document: &Document, label: impl Into<String>) -> Self {
        Self {
            baseline: document.clone(),
            working: document.clone(),
            staged: ChangeSet::empty(),
            properties: HashMap::new(),
            label: label.into(),
        }
    }

    /// Stages one command on the working copy, accumulating its changes.
    /// Returns the command's incremental `ChangeSet` for overlay preview.
    pub fn update(&mut self, request: &CommandRequest) -> Result<ChangeSet, PetuniaError> {
        let delta = if request.command.is_atomic_primitive() {
            commands::execute(&mut self.working, request)?
        } else {
            let mut next = self.working.clone();
            let delta = commands::execute(&mut next, request)?;
            self.working = next;
            delta
        };
        for change in delta.changes.iter().cloned() {
            if let Some(id) = property_object(&change) {
                let key = (std::mem::discriminant(&change), id);
                if let Some(&index) = self.properties.get(&key) {
                    merge_property(&mut self.staged.changes[index], change);
                } else {
                    self.properties.insert(key, self.staged.len());
                    self.staged.push(change);
                }
            } else {
                // Structural edits can remove/recreate or reparent an identity.
                // Never merge a property across those replay dependencies.
                self.properties.clear();
                self.staged.push(change);
            }
        }
        Ok(delta)
    }

    /// Read-only view of the working copy for overlay preview.
    #[must_use]
    pub fn preview(&self) -> &Document {
        &self.working
    }

    /// Combined staged changes so far.
    #[must_use]
    pub fn staged(&self) -> &ChangeSet {
        &self.staged
    }

    /// Human label for this gesture.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// True when nothing was staged (commit would be a NoOp).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.staged.is_empty()
    }

    /// Commits atomically: replaces the live document with the working copy
    /// and records one combined entry. Empty transactions are a NoOp and
    /// leave history untouched.
    pub fn commit(
        mut self,
        document: &mut Document,
        history: &mut History,
    ) -> Result<(), PetuniaError> {
        if self.staged.is_empty() {
            return Ok(());
        }
        if *document != self.baseline {
            return Err(PetuniaError::invalid_input(
                "transaction conflict: document changed during preview",
            ));
        }
        self.working.validate()?;
        self.staged
            .changes
            .retain(|change| !property_is_noop(change));
        history.record(self.staged)?;
        *document = self.working;
        Ok(())
    }

    /// Cancels: discards the working copy. The live document and history are
    /// untouched, so cancel is exactly the pre-gesture state.
    pub fn cancel(self) {}
}

fn property_object(change: &Change) -> Option<ObjectId> {
    match change {
        Change::NameChanged { id, .. }
        | Change::FillChanged { id, .. }
        | Change::VisibilityChanged { id, .. }
        | Change::LockChanged { id, .. }
        | Change::OpacityChanged { id, .. }
        | Change::StrokeChanged { id, .. }
        | Change::BoundsChanged { id, .. }
        | Change::ShapeChanged { id, .. }
        | Change::TextStyleChanged { id, .. }
        | Change::AppearanceChanged { id, .. }
        | Change::ModifiersChanged { id, .. }
        | Change::MaskModeChanged { id, .. } => Some(*id),
        _ => None,
    }
}

fn merge_property(previous: &mut Change, latest: Change) {
    macro_rules! merge {
        ($($variant:ident),* $(,)?) => {
            match (previous, latest) {
                $((Change::$variant { next, .. }, Change::$variant { next: value, .. }) => *next = value,)*
                (Change::BoundsChanged { next_bounds, next_rotation, .. },
                 Change::BoundsChanged { next_bounds: bounds, next_rotation: rotation, .. }) => {
                    *next_bounds = bounds;
                    *next_rotation = rotation;
                }
                (Change::StrokeChanged { next_stroke, next_width, .. },
                 Change::StrokeChanged { next_stroke: stroke, next_width: width, .. }) => {
                    *next_stroke = stroke;
                    *next_width = width;
                }
                _ => unreachable!("coalesced properties have the same discriminant"),
            }
        }
    }
    merge!(
        NameChanged,
        FillChanged,
        VisibilityChanged,
        LockChanged,
        OpacityChanged,
        ShapeChanged,
        TextStyleChanged,
        AppearanceChanged,
        ModifiersChanged,
        MaskModeChanged
    );
}

fn property_is_noop(change: &Change) -> bool {
    macro_rules! equals {
        ($($variant:ident),* $(,)?) => {
            match change {
                $(Change::$variant { previous, next, .. } => previous == next,)*
                Change::BoundsChanged { previous_bounds, next_bounds, previous_rotation, next_rotation, .. } =>
                    previous_bounds == next_bounds && previous_rotation == next_rotation,
                Change::StrokeChanged { previous_stroke, next_stroke, previous_width, next_width, .. } =>
                    previous_stroke == next_stroke && previous_width == next_width,
                _ => false,
            }
        }
    }
    equals!(
        NameChanged,
        FillChanged,
        VisibilityChanged,
        LockChanged,
        OpacityChanged,
        ShapeChanged,
        TextStyleChanged,
        AppearanceChanged,
        ModifiersChanged,
        MaskModeChanged
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{Command, CommandRequest};
    use petunia_design_foundation::IdGenerator;

    fn test_doc() -> (Document, IdGenerator, petunia_design_foundation::SurfaceId) {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let surface = gen.next_surface();
        {
            let mut mutator = petunia_design_document::DocumentMutator::new(&mut doc);
            mutator.add_surface(surface, "Page").unwrap();
        }
        (doc, gen, surface)
    }

    #[test]
    fn preview_does_not_touch_live_document_until_commit() {
        let (mut doc, mut gen, surface) = test_doc();
        let obj = gen.next_object();
        let mut tx = Transaction::begin(&doc, "Create");
        tx.update(&CommandRequest::new(Command::CreateObject {
            surface,
            id: obj,
            name: "Box".to_string(),
        }))
        .unwrap();
        // Live doc untouched during preview.
        assert!(doc.find_object(obj).is_none());
        assert!(tx.preview().find_object(obj).is_some());
        let mut history = History::new(100);
        tx.commit(&mut doc, &mut history).unwrap();
        assert!(doc.find_object(obj).is_some());
        assert_eq!(history.undo_len(), 1);
    }

    #[test]
    fn cancel_leaves_document_and_history_untouched() {
        let (mut doc, mut gen, surface) = test_doc();
        let obj = gen.next_object();
        let mut tx = Transaction::begin(&doc, "Create");
        tx.update(&CommandRequest::new(Command::CreateObject {
            surface,
            id: obj,
            name: "Box".to_string(),
        }))
        .unwrap();
        let history = History::new(100);
        tx.cancel();
        assert!(doc.find_object(obj).is_none());
        assert_eq!(history.undo_len(), 0);
        let _ = &mut doc;
    }

    #[test]
    fn empty_commit_is_noop() {
        let (mut doc, _gen, _surface) = test_doc();
        let tx = Transaction::begin(&doc, "Empty");
        let mut history = History::new(100);
        tx.commit(&mut doc, &mut history).unwrap();
        assert_eq!(history.undo_len(), 0);
    }
    #[test]
    fn a_long_gesture_retains_one_before_after_pair_and_replays_exactly() {
        let (mut doc, mut generator, surface) = test_doc();
        let id = generator.next_object();
        commands::execute(
            &mut doc,
            &CommandRequest::new(Command::CreateObject {
                surface,
                id,
                name: "Box".into(),
            }),
        )
        .unwrap();
        let baseline = doc.clone();
        let mut tx = Transaction::begin(&doc, "Move");
        for index in 0..4096 {
            tx.update(&CommandRequest::new(Command::SetBounds {
                id,
                bounds: Some([f64::from(index), 1., 10., 20.]),
                rotation: 0.,
            }))
            .unwrap();
        }
        assert_eq!(tx.staged().len(), 1);
        assert_eq!(doc, baseline);
        let expected = tx.preview().clone();
        let mut history = History::new(10);
        tx.commit(&mut doc, &mut history).unwrap();
        assert_eq!(doc, expected);
        assert_eq!(history.undo_len(), 1);
        assert!(history.undo(&mut doc).unwrap());
        assert_eq!(doc, baseline);
        assert!(history.redo(&mut doc).unwrap());
        assert_eq!(doc, expected);
    }
    #[test]
    fn a_return_to_origin_keeps_redo_and_does_not_add_history() {
        let (mut doc, mut generator, surface) = test_doc();
        let id = generator.next_object();
        commands::execute(
            &mut doc,
            &CommandRequest::new(Command::CreateObject {
                surface,
                id,
                name: "Box".into(),
            }),
        )
        .unwrap();
        let mut history = History::new(10);
        history
            .record(
                commands::execute(
                    &mut doc,
                    &CommandRequest::new(Command::RenameObject {
                        id,
                        name: "Renamed".into(),
                    }),
                )
                .unwrap(),
            )
            .unwrap();
        history.undo(&mut doc).unwrap();
        let baseline = doc.clone();
        let mut tx = Transaction::begin(&doc, "Opacity");
        for opacity in [0.2, 0.5, 1.] {
            tx.update(&CommandRequest::new(Command::SetOpacity { id, opacity }))
                .unwrap();
        }
        tx.commit(&mut doc, &mut history).unwrap();
        assert_eq!(doc, baseline);
        assert_eq!(history.undo_len(), 0);
        assert_eq!(history.redo_len(), 1);
        assert!(history.redo(&mut doc).unwrap());
        assert_eq!(doc.find_object(id).unwrap().name, "Renamed");
    }
    #[test]
    fn deleting_and_recreating_an_identity_is_a_coalescing_barrier() {
        let (mut doc, mut generator, surface) = test_doc();
        let id = generator.next_object();
        commands::execute(
            &mut doc,
            &CommandRequest::new(Command::CreateObject {
                surface,
                id,
                name: "Original".into(),
            }),
        )
        .unwrap();
        let baseline = doc.clone();
        let mut tx = Transaction::begin(&doc, "Replace");
        for command in [
            Command::RenameObject {
                id,
                name: "Intermediate".into(),
            },
            Command::DeleteObject { id },
            Command::CreateObject {
                surface,
                id,
                name: "Replacement".into(),
            },
            Command::RenameObject {
                id,
                name: "Final".into(),
            },
        ] {
            tx.update(&CommandRequest::new(command)).unwrap();
        }
        assert_eq!(tx.staged().len(), 4);
        let expected = tx.preview().clone();
        let mut history = History::new(10);
        tx.commit(&mut doc, &mut history).unwrap();
        history.undo(&mut doc).unwrap();
        assert_eq!(doc, baseline);
        history.redo(&mut doc).unwrap();
        assert_eq!(doc, expected);
    }
}
