//! Tests for AubrietaGuiBridge, semantic application ports, and reactive presentation models.

use aubrieta_application::{ActionId, ActionRequest, Command, CommandRequest};
use aubrieta_foundation::IdGenerator;
use aubrieta_shell::bridge::*;

#[test]
fn bridge_new_document_and_snapshot_lifecycle() {
    let mut bridge = AubrietaGuiBridge::new();
    assert!(bridge.session().is_none());

    bridge.new_document("Artwork 1").expect("new document");
    assert!(bridge.session().is_some());

    let snap = bridge.snapshot();
    assert_eq!(snap.title, "Artwork 1");
    assert_eq!(snap.surface_count, 1);
    assert_eq!(snap.total_objects, 0);
    assert_eq!(snap.selected_count, 0);
    // A1: the initial canvas is a real committed command, so a fresh
    // document is dirty with one undoable entry (exact revision lane).
    assert_eq!(snap.revision, 1);
    assert!(snap.is_dirty);
    assert!(snap.can_undo);
    assert!(!snap.can_redo);
}

#[test]
fn bridge_command_execution_undo_and_redo_parity() {
    let mut bridge = AubrietaGuiBridge::new();
    bridge.new_document("Undo Test").expect("doc");

    let surface_id = bridge.active_surface().expect("surface");
    let mut gen = IdGenerator::new();
    let obj_id = gen.next_object();

    // 1. Submit create object command
    let create_cmd = CommandRequest::new(Command::CreateObject {
        surface: surface_id,
        id: obj_id,
        name: "Rect 1".to_string(),
    });
    let changes = bridge.submit_command(create_cmd).expect("create");
    assert_eq!(changes.len(), 1);

    assert!(bridge.can_undo());
    assert!(!bridge.can_redo());
    assert!(bridge.is_dirty());

    // 2. Undo removes the object; the initial canvas entry (A1) remains,
    // so undo is still available afterwards.
    let undone = bridge.undo().expect("undo");
    assert!(undone);
    assert!(bridge.can_undo());
    assert!(bridge.can_redo());

    // Verify object removed in session
    assert!(bridge
        .session()
        .unwrap()
        .document()
        .find_object(obj_id)
        .is_none());

    // 2b. Undo again removes the initial canvas itself.
    let undone_canvas = bridge.undo().expect("undo canvas");
    assert!(undone_canvas);
    assert!(!bridge.can_undo());
    assert_eq!(bridge.snapshot().surface_count, 0);

    // 3. Redo restores the canvas, then the object.
    let redone_canvas = bridge.redo().expect("redo canvas");
    assert!(redone_canvas);
    assert_eq!(bridge.snapshot().surface_count, 1);
    let redone = bridge.redo().expect("redo");
    assert!(redone);
    assert!(bridge.can_undo());
    assert!(!bridge.can_redo());

    // Verify object restored
    assert!(bridge
        .session()
        .unwrap()
        .document()
        .find_object(obj_id)
        .is_some());
}

#[test]
fn bridge_action_query_enabled_states_and_dispatch() {
    let mut bridge = AubrietaGuiBridge::new();

    // Actions before document open
    let actions_none = bridge.query_actions();
    assert!(
        !actions_none
            .get(&ActionId::new("aubrieta.file.save"))
            .unwrap()
            .is_enabled
    );
    assert!(
        !actions_none
            .get(&ActionId::new("aubrieta.edit.undo"))
            .unwrap()
            .is_enabled
    );
    assert!(
        !actions_none
            .get(&ActionId::new("aubrieta.edit.delete"))
            .unwrap()
            .is_enabled
    );

    bridge.new_document("Actions Test").expect("doc");
    let surface_id = bridge.active_surface().expect("surface");

    let mut gen = IdGenerator::new();
    let obj_id = gen.next_object();
    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id: obj_id,
            name: "Item".to_string(),
        }))
        .expect("create");

    // Save should be enabled (dirty), undo enabled
    let actions_dirty = bridge.query_actions();
    assert!(
        actions_dirty
            .get(&ActionId::new("aubrieta.file.save"))
            .unwrap()
            .is_enabled
    );
    assert!(
        actions_dirty
            .get(&ActionId::new("aubrieta.edit.undo"))
            .unwrap()
            .is_enabled
    );
    assert!(
        !actions_dirty
            .get(&ActionId::new("aubrieta.edit.delete"))
            .unwrap()
            .is_enabled
    );

    // Select the object
    bridge.set_selection(vec![obj_id]);
    let actions_selected = bridge.query_actions();
    assert!(
        actions_selected
            .get(&ActionId::new("aubrieta.edit.delete"))
            .unwrap()
            .is_enabled
    );

    // Dispatch delete action
    let del_res = bridge
        .dispatch_action(ActionRequest::without_payload(ActionId::new(
            "aubrieta.edit.delete",
        )))
        .expect("delete");
    assert_eq!(del_res.len(), 1);
    assert!(bridge.selection().is_empty);
}

#[test]
fn bridge_property_edits_and_layers_presentation() {
    let mut bridge = AubrietaGuiBridge::new();
    bridge.new_document("Props Test").expect("doc");
    let surface_id = bridge.active_surface().expect("surface");

    let mut gen = IdGenerator::new();
    let obj_id = gen.next_object();
    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id: obj_id,
            name: "Card".to_string(),
        }))
        .expect("create");

    bridge.set_selection(vec![obj_id]);

    // Edit fill
    bridge
        .set_fill(obj_id, Some("aubrieta.teal/600".to_string()))
        .expect("fill");
    // Edit stroke
    bridge
        .set_stroke(obj_id, Some("aubrieta.gray/900".to_string()), 2.5)
        .expect("stroke");
    // Edit opacity
    bridge.set_opacity(obj_id, 0.85).expect("opacity");
    // Edit bounds
    bridge
        .set_bounds(obj_id, Some([10.0, 20.0, 300.0, 200.0]), 0.1)
        .expect("bounds");

    // Query properties presentation model
    let props = bridge.query_properties();
    assert!(!props.selection_empty);
    assert_eq!(props.fill.as_deref(), Some("aubrieta.teal/600"));
    assert_eq!(props.stroke.as_deref(), Some("aubrieta.gray/900"));
    assert_eq!(props.stroke_width, 2.5);
    assert!((props.opacity - 0.85).abs() < 1e-6);
    assert_eq!(props.bounds, Some([10.0, 20.0, 300.0, 200.0]));

    // Query layers presentation model
    let layers = bridge.query_layers();
    assert_eq!(layers.surfaces.len(), 1);
    assert_eq!(layers.rows.len(), 1);
    assert_eq!(layers.rows[0].id, obj_id);
    assert!(layers.rows[0].is_selected);
    assert_eq!(
        layers.rows[0].fill_token.as_deref(),
        Some("aubrieta.teal/600")
    );
}

#[test]
fn bridge_close_session_dirty_protection() {
    let mut bridge = AubrietaGuiBridge::new();
    bridge.new_document("Protect Test").expect("doc");
    let surface_id = bridge.active_surface().expect("surface");
    let mut gen = IdGenerator::new();
    let obj_id = gen.next_object();

    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id: obj_id,
            name: "Unsaved".to_string(),
        }))
        .expect("cmd");

    // Close without force should fail if dirty
    let closed = bridge.close_session(false).expect("close check");
    assert!(!closed);
    assert!(bridge.session().is_some());

    // Force close succeeds
    let force_closed = bridge.close_session(true).expect("force close");
    assert!(force_closed);
    assert!(bridge.session().is_none());
}
