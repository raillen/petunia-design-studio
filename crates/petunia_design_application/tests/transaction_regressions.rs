use petunia_design_application::{Command, CommandRequest, DocumentSession, History, Transaction};
use petunia_design_document::{Change, ChangeSet, Document, DocumentMutator, DocumentObject};
use petunia_design_foundation::{ObjectId, SurfaceId};

fn fixture() -> Document {
    let mut doc = Document::new();
    let mut m = DocumentMutator::new(&mut doc);
    m.add_surface(SurfaceId::new(1), "Page").unwrap();
    m.add_object(
        SurfaceId::new(1),
        DocumentObject::new(ObjectId::new(2), "Original"),
    )
    .unwrap();
    doc
}

#[test]
fn conflicting_preview_preserves_intervening_edit_and_history() {
    let mut doc = fixture();
    let mut history = History::default();
    let mut tx = Transaction::begin(&doc, "Preview");
    tx.update(&CommandRequest::new(Command::RenameObject {
        id: ObjectId::new(2),
        name: "Preview".into(),
    }))
    .unwrap();
    history
        .execute(
            &mut doc,
            &CommandRequest::new(Command::RenameObject {
                id: ObjectId::new(2),
                name: "Concurrent".into(),
            }),
        )
        .unwrap();
    let expected = doc.clone();
    assert!(tx.commit(&mut doc, &mut history).is_err());
    assert_eq!(doc, expected);
    assert_eq!(history.undo_len(), 1);
    history.undo(&mut doc).unwrap();
    assert_eq!(doc, fixture());
}

#[test]
fn failed_undo_keeps_the_entry_and_discards_partial_revert() {
    let mut doc = fixture();
    let mut history = History::default();
    let mut changes = ChangeSet::empty();
    // Revert is reverse ordered: rename succeeds before missing object fails.
    changes.push(Change::NameChanged {
        id: ObjectId::new(999),
        previous: "Missing".into(),
        next: "Other".into(),
    });
    changes.push(Change::NameChanged {
        id: ObjectId::new(2),
        previous: "Wrong".into(),
        next: "Original".into(),
    });
    history.record(changes).unwrap();
    let expected = doc.clone();
    let state = history.state_id();
    assert!(history.undo(&mut doc).is_err());
    assert_eq!(doc, expected);
    assert_eq!(history.undo_len(), 1);
    assert_eq!(history.redo_len(), 0);
    assert_eq!(history.state_id(), state);
}

#[test]
fn failed_redo_keeps_the_entry_and_discards_partial_replay() {
    let mut doc = fixture();
    let mut history = History::default();
    history
        .execute(
            &mut doc,
            &CommandRequest::new(Command::RenameObject {
                id: ObjectId::new(2),
                name: "Changed".into(),
            }),
        )
        .unwrap();
    history.undo(&mut doc).unwrap();
    DocumentMutator::new(&mut doc)
        .remove_object(ObjectId::new(2))
        .unwrap();
    let expected = doc.clone();
    assert!(history.redo(&mut doc).is_err());
    assert_eq!(doc, expected);
    assert_eq!(history.redo_len(), 1);
    assert_eq!(history.undo_len(), 0);
}

#[test]
fn undo_to_savepoint_is_clean_and_new_branch_is_dirty() {
    let mut session = DocumentSession::with_document("Save", fixture());
    session.mark_saved();
    session
        .execute_command(CommandRequest::new(Command::RenameObject {
            id: ObjectId::new(2),
            name: "First".into(),
        }))
        .unwrap();
    assert!(session.is_dirty());
    session.undo().unwrap();
    assert!(!session.is_dirty());
    session.redo().unwrap();
    assert!(session.is_dirty());
    session.mark_saved();
    session.undo().unwrap();
    assert!(session.is_dirty());
    session
        .execute_command(CommandRequest::new(Command::RenameObject {
            id: ObjectId::new(2),
            name: "Branch".into(),
        }))
        .unwrap();
    assert!(session.is_dirty());
    assert!(!session.redo().unwrap());
}

#[test]
fn history_budget_rejects_oversized_edit_before_publication() {
    let mut doc = fixture();
    let mut history = History::with_budget(2, 64);
    let expected = doc.clone();
    assert!(history
        .execute(
            &mut doc,
            &CommandRequest::new(Command::RenameObject {
                id: ObjectId::new(2),
                name: "x".repeat(1024)
            })
        )
        .is_err());
    assert_eq!(doc, expected);
    assert_eq!(history.undo_len(), 0);
    assert_eq!(history.retained_bytes(), 0);
}

#[test]
fn retention_keeps_state_identity_after_eviction_and_branching() {
    let mut doc = fixture();
    let mut history = History::new(2);
    for name in ["one", "two", "three"] {
        history
            .execute(
                &mut doc,
                &CommandRequest::new(Command::RenameObject {
                    id: ObjectId::new(2),
                    name: name.into(),
                }),
            )
            .unwrap();
    }
    assert_eq!(history.undo_len(), 2);
    history.undo(&mut doc).unwrap();
    history.undo(&mut doc).unwrap();
    assert_eq!(doc.find_object(ObjectId::new(2)).unwrap().name, "one");
    assert!(!history.undo(&mut doc).unwrap());
    let previous = history.state_id();
    history
        .execute(
            &mut doc,
            &CommandRequest::new(Command::RenameObject {
                id: ObjectId::new(2),
                name: "branch".into(),
            }),
        )
        .unwrap();
    assert_ne!(history.state_id(), previous);
    assert_eq!(history.redo_len(), 0);
}

#[test]
fn failed_primitive_staging_preserves_preview_and_prior_changes() {
    let doc = fixture();
    let mut tx = Transaction::begin(&doc, "Atomic staging");
    tx.update(&CommandRequest::new(Command::SetBounds {
        id: ObjectId::new(2),
        bounds: Some([10.0, 10.0, 100.0, 100.0]),
        rotation: 0.0,
    }))
    .unwrap();
    let expected = tx.preview().clone();
    let prior = tx.staged().clone();
    let bad = Command::SetShape {
        id: ObjectId::new(2),
        shape: Some(petunia_design_document::ShapeKind::Rectangle {
            corner_radii: [f64::NAN; 4],
        }),
    };
    assert!(tx.update(&CommandRequest::new(bad)).is_err());
    assert_eq!(tx.preview(), &expected);
    assert_eq!(tx.staged(), &prior);
    let bad = Command::SetBounds {
        id: ObjectId::new(2),
        bounds: Some([0.0, 0.0, -1.0, 100.0]),
        rotation: 0.0,
    };
    assert!(tx.update(&CommandRequest::new(bad)).is_err());
    assert_eq!(tx.preview(), &expected);
    assert_eq!(tx.staged(), &prior);
    assert_eq!(doc, fixture());
}

#[test]
fn batch_export_selection_is_undoable_and_survives_reload() {
    let mut doc = fixture();
    let before = doc.clone();
    let mut history = History::default();
    history
        .execute(
            &mut doc,
            &CommandRequest::new(Command::SetSurfaceExportEnabled {
                surface: SurfaceId::new(1),
                enabled: false,
            }),
        )
        .unwrap();
    assert!(!doc.surface(SurfaceId::new(1)).unwrap().export_enabled);
    assert_eq!(Document::from_json(&doc.to_json().unwrap()).unwrap(), doc);
    history.undo(&mut doc).unwrap();
    assert_eq!(doc, before);
    history.redo(&mut doc).unwrap();
    assert!(!doc.surface(SurfaceId::new(1)).unwrap().export_enabled);
}

#[test]
fn modifier_move_resize_and_bake_survive_history_branch_and_reload() {
    use petunia_design_document::ShapeKind;
    use petunia_design_geometry::GPoint;
    let mut doc = fixture();
    let id = ObjectId::new(2);
    let mut m = DocumentMutator::new(&mut doc);
    m.set_bounds(id, Some([100.0, 200.0, 100.0, 100.0]), 0.0)
        .unwrap();
    m.set_shape(
        id,
        Some(ShapeKind::Rectangle {
            corner_radii: [0.0; 4],
        }),
    )
    .unwrap();
    m.set_crop_rect(id, [110.0, 210.0, 50.0, 30.0]).unwrap();
    let source = doc.clone();
    let mut history = History::default();
    history
        .execute(
            &mut doc,
            &CommandRequest::new(Command::SetBounds {
                id,
                bounds: Some([-20.0, 70.0, 200.0, 50.0]),
                rotation: 0.0,
            }),
        )
        .unwrap();
    let moved = doc.clone();
    history
        .execute(&mut doc, &CommandRequest::new(Command::BakeGeometry { id }))
        .unwrap();
    let baked = doc.clone();
    assert!(baked.find_object(id).unwrap().modifiers.is_empty());
    assert!(baked
        .evaluated_path_world(id)
        .unwrap()
        .contains_point(GPoint::new(40.0, 80.0), 0.25));
    assert_eq!(
        Document::from_json(&baked.to_json().unwrap()).unwrap(),
        baked
    );
    history.undo(&mut doc).unwrap();
    assert_eq!(doc, moved);
    history.undo(&mut doc).unwrap();
    assert_eq!(doc, source);
    history.redo(&mut doc).unwrap();
    assert_eq!(doc, moved);
    history.redo(&mut doc).unwrap();
    assert_eq!(doc, baked);
    history.undo(&mut doc).unwrap();
    history
        .execute(
            &mut doc,
            &CommandRequest::new(Command::RenameObject {
                id,
                name: "New branch".into(),
            }),
        )
        .unwrap();
    assert_eq!(history.redo_len(), 0);
    assert_eq!(
        doc.find_object(id).unwrap().modifiers,
        moved.find_object(id).unwrap().modifiers
    );
}

#[test]
fn consecutive_guide_creation_has_distinct_ids_and_stable_replay() {
    use petunia_design_document::GuideOrientation;
    let mut doc = fixture();
    let mut history = History::default();
    for orientation in [GuideOrientation::Horizontal, GuideOrientation::Vertical] {
        history
            .execute(
                &mut doc,
                &CommandRequest::new(Command::CreateGuide {
                    surface: SurfaceId::new(1),
                    orientation,
                    position: 30.0,
                }),
            )
            .unwrap();
    }
    let before = doc.clone();
    let guides = &doc.surface(SurfaceId::new(1)).unwrap().guides;
    assert_eq!(guides.len(), 2);
    assert_ne!(guides[0].id, guides[1].id);
    history.undo(&mut doc).unwrap();
    assert_eq!(doc.surface(SurfaceId::new(1)).unwrap().guides.len(), 1);
    history.redo(&mut doc).unwrap();
    assert_eq!(doc, before);
}
