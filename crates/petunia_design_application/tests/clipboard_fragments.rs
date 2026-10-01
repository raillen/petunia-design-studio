//! Whole-subtree clipboard regressions for the deferred final MVP batch.
use petunia_design_application::{
    session::DocumentSession, ActionId, ActionRequest, Command, CommandRequest,
};
use petunia_design_document::{
    ContainerRole, Document, DocumentMutator, DocumentObject, ShapeKind,
};
use petunia_design_foundation::{ObjectId, SurfaceId};
fn fixture() -> DocumentSession {
    let mut document = Document::new();
    let surface = SurfaceId::new(1);
    let mut mutator = DocumentMutator::new(&mut document);
    mutator.add_surface(surface, "Source").unwrap();
    for n in 2..=3 {
        let mut object = DocumentObject::new(ObjectId::new(n), format!("Child {n}"));
        object.bounds = Some([n as f64 * 10., 20., 8., 8.]);
        object.shape = Some(ShapeKind::Rectangle {
            corner_radii: [0.; 4],
        });
        object.fill = Some("#ff0000".into());
        mutator.add_object(surface, object).unwrap();
    }
    drop(mutator);
    let mut session = DocumentSession::with_document("Source", document);
    session
        .execute_command(CommandRequest::new(Command::GroupObjects {
            surface,
            group_id: ObjectId::new(4),
            child_ids: vec![ObjectId::new(2), ObjectId::new(3)],
            role: ContainerRole::Group,
        }))
        .unwrap();
    session.selection.select_exact(vec![ObjectId::new(4)]);
    session
}
fn action(session: &mut DocumentSession, id: &str) {
    session
        .dispatch_action(ActionRequest::new(ActionId::new(id), serde_json::json!({})))
        .unwrap();
}
#[test]
fn duplicate_preserves_children_with_fresh_edges_and_one_undo_step() {
    let mut session = fixture();
    let before = session.document().clone();
    action(&mut session, "ptnd.action.edit.duplicate");
    session.document().validate().unwrap();
    let selected = session.selection.selected_ids.clone();
    assert_eq!(selected.len(), 1);
    assert_ne!(selected[0], ObjectId::new(4));
    let group = session.find_object(selected[0]).unwrap();
    assert_eq!(group.children.len(), 2);
    for child in &group.children {
        assert_ne!(*child, ObjectId::new(2));
        assert_ne!(*child, ObjectId::new(3));
        assert_eq!(session.find_object(*child).unwrap().parent, Some(group.id));
    }
    session.undo().unwrap();
    assert_eq!(session.document(), &before);
    session.redo().unwrap();
    session.document().validate().unwrap();
}
#[test]
fn deleting_group_removes_descendants_instead_of_ungrouping() {
    let mut session = fixture();
    let before = session.document().clone();
    action(&mut session, "ptnd.action.edit.delete");
    assert!(session.document().surfaces()[0].objects().is_empty());
    session.undo().unwrap();
    assert_eq!(session.document(), &before);
}
#[test]
fn paste_from_another_artboard_uses_its_origin_and_keeps_the_source() {
    let source = fixture();
    let objects = source.capture_clipboard_fragment().unwrap();
    let before = source.document().clone();
    let mut document = Document::new();
    let target = SurfaceId::new(100);
    let mut mutator = DocumentMutator::new(&mut document);
    mutator.add_surface(target, "Target").unwrap();
    mutator
        .set_surface_geometry(target, [1000., -500.], [300., 300.])
        .unwrap();
    drop(mutator);
    let mut destination = DocumentSession::with_document("Target", document);
    destination
        .paste_clipboard_fragment(target, objects, [0., 0.])
        .unwrap();
    destination.document().validate().unwrap();
    assert_eq!(source.document(), &before);
    let selected = destination.selection.selected_ids[0];
    let bounds = destination.cached_world_bounds(selected).unwrap();
    assert!(bounds[0] >= 1000.);
    assert!(bounds[1] < 0.);
    destination.undo().unwrap();
    assert!(destination.document().surfaces()[0].objects().is_empty());
}
#[test]
fn detached_child_copy_preserves_world_frame_and_original_tree() {
    let mut session = fixture();
    session
        .execute_command(CommandRequest::new(Command::SetBounds {
            id: ObjectId::new(4),
            bounds: session.find_object(ObjectId::new(4)).unwrap().bounds,
            rotation: 0.4,
        }))
        .unwrap();
    session.selection.select_exact(vec![ObjectId::new(2)]);
    let before = session.document().clone();
    let world = before.world_transform_checked(ObjectId::new(2)).unwrap();
    let fragment = session.capture_clipboard_fragment().unwrap();
    assert_eq!(fragment.len(), 1);
    assert_eq!(fragment[0].parent, None);
    let mut document = Document::new();
    let mut mutator = DocumentMutator::new(&mut document);
    mutator.add_surface(SurfaceId::new(1), "Fragment").unwrap();
    mutator
        .add_objects_bulk(SurfaceId::new(1), fragment)
        .unwrap();
    let restored = document.world_transform_checked(ObjectId::new(2)).unwrap();
    for (a, b) in world.coeffs.iter().zip(restored.coeffs.iter()) {
        assert!((a - b).abs() < 1e-8);
    }
    assert_eq!(session.document(), &before);
}
