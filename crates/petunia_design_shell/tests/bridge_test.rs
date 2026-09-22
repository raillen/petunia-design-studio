//! Tests for PetuniaDesignGuiBridge, semantic application ports, and reactive presentation models.

use petunia_design_application::{ActionId, ActionRequest, Command, CommandRequest};
use petunia_design_foundation::IdGenerator;
use petunia_design_shell::bridge::*;

#[test]
fn bridge_new_document_and_snapshot_lifecycle() {
    let mut bridge = PetuniaDesignGuiBridge::new();
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
    let mut bridge = PetuniaDesignGuiBridge::new();
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
    let mut bridge = PetuniaDesignGuiBridge::new();

    // Actions before document open
    let actions_none = bridge.query_actions();
    assert!(
        !actions_none
            .get(&ActionId::new("ptnd.file.save"))
            .unwrap()
            .is_enabled
    );
    assert!(
        !actions_none
            .get(&ActionId::new("ptnd.action.edit.undo"))
            .unwrap()
            .is_enabled
    );
    assert!(
        !actions_none
            .get(&ActionId::new("ptnd.action.edit.delete"))
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
            .get(&ActionId::new("ptnd.file.save"))
            .unwrap()
            .is_enabled
    );
    assert!(
        actions_dirty
            .get(&ActionId::new("ptnd.action.edit.undo"))
            .unwrap()
            .is_enabled
    );
    assert!(
        !actions_dirty
            .get(&ActionId::new("ptnd.action.edit.delete"))
            .unwrap()
            .is_enabled
    );

    // Select the object
    bridge.set_selection(vec![obj_id]);
    let actions_selected = bridge.query_actions();
    assert!(
        actions_selected
            .get(&ActionId::new("ptnd.action.edit.delete"))
            .unwrap()
            .is_enabled
    );

    // Dispatch delete action
    let del_res = bridge
        .dispatch_action(ActionRequest::without_payload(ActionId::new(
            "ptnd.action.edit.delete",
        )))
        .expect("delete");
    assert_eq!(del_res.len(), 1);
    assert!(bridge.selection().is_empty);
}

#[test]
fn bridge_property_edits_and_layers_presentation() {
    let mut bridge = PetuniaDesignGuiBridge::new();
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
        .set_fill(obj_id, Some("ptnd.teal/600".to_string()))
        .expect("fill");
    // Edit stroke
    bridge
        .set_stroke(obj_id, Some("ptnd.gray/900".to_string()), 2.5)
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
    assert_eq!(props.fill.as_deref(), Some("ptnd.teal/600"));
    assert_eq!(props.stroke.as_deref(), Some("ptnd.gray/900"));
    assert_eq!(props.stroke_width, 2.5);
    assert!((props.opacity - 0.85).abs() < 1e-6);
    assert_eq!(props.bounds, Some([10.0, 20.0, 300.0, 200.0]));

    // Query layers presentation model
    let layers = bridge.query_layers();
    assert_eq!(layers.surfaces.len(), 1);
    assert_eq!(layers.rows.len(), 1);
    assert_eq!(layers.rows[0].id, obj_id);
    assert!(layers.rows[0].is_selected);
    assert_eq!(layers.rows[0].fill_token.as_deref(), Some("ptnd.teal/600"));
}

#[test]
fn bridge_close_session_dirty_protection() {
    let mut bridge = PetuniaDesignGuiBridge::new();
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

#[test]
fn file_new_replaces_the_session_through_the_action_lane() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Artwork 1").expect("new document");
    let surface = bridge.active_surface().expect("surface");
    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface,
            id: petunia_design_foundation::ObjectId::new(1),
            name: "Rect".to_string(),
        }))
        .expect("create");
    assert_eq!(bridge.snapshot().total_objects, 1);

    bridge
        .dispatch_action(ActionRequest::new(
            ActionId::new("ptnd.action.file.new"),
            serde_json::json!({}),
        ))
        .expect("file.new must dispatch");

    let snap = bridge.snapshot();
    assert_eq!(snap.title, "Untitled");
    assert_eq!(
        snap.total_objects, 0,
        "file.new must discard the old document"
    );
    assert_eq!(snap.surface_count, 1, "the new session keeps its canvas");
}

#[test]
fn file_open_loads_a_ptnd_package_and_records_its_path() {
    let dir = std::env::temp_dir().join("petunia-design-bridge-open");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("opened.PTND");

    // Produce a real package through the session save path.
    {
        let mut bridge = PetuniaDesignGuiBridge::new();
        bridge.new_document("Source").expect("new document");
        let surface = bridge.active_surface().expect("surface");
        bridge
            .submit_command(CommandRequest::new(Command::CreateObject {
                surface,
                id: petunia_design_foundation::ObjectId::new(7),
                name: "Saved".to_string(),
            }))
            .expect("create");
        bridge
            .dispatch_action(ActionRequest::new(
                ActionId::new("ptnd.action.file.save_as"),
                serde_json::json!({ "path": path.to_string_lossy() }),
            ))
            .expect("save_as");
    }

    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge
        .dispatch_action(ActionRequest::new(
            ActionId::new("ptnd.action.file.open"),
            serde_json::json!({ "path": path.to_string_lossy() }),
        ))
        .expect("file.open must dispatch");

    let snap = bridge.snapshot();
    assert_eq!(snap.total_objects, 1, "the saved object must come back");
    assert_eq!(snap.title, "opened.PTND");
    assert!(
        !bridge.is_dirty(),
        "opening a native package establishes a clean save point"
    );
}

#[test]
fn file_open_without_a_path_is_rejected_with_a_reason() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Artwork").expect("new document");
    let error = bridge
        .dispatch_action(ActionRequest::new(
            ActionId::new("ptnd.action.file.open"),
            serde_json::json!({}),
        ))
        .expect_err("file.open cannot guess a location");
    assert!(error.to_string().contains("path"), "{error}");
}

#[test]
fn pre_grammar_file_ids_still_dispatch() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Artwork").expect("new document");
    bridge
        .dispatch_action(ActionRequest::new(
            ActionId::new("ptnd.file.new"),
            serde_json::json!({}),
        ))
        .expect("the pre-grammar id must still resolve");
    assert_eq!(bridge.snapshot().title, "Untitled");
}

#[test]
fn opening_a_legacy_package_forces_save_as_and_never_overwrites_it() {
    let dir = std::env::temp_dir().join("petunia-design-bridge-legacy");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // Produce a real package, then disguise it under the legacy suffix.
    let native = dir.join("source.PTND");
    {
        let mut bridge = PetuniaDesignGuiBridge::new();
        bridge.new_document("Source").expect("new document");
        let surface = bridge.active_surface().expect("surface");
        bridge
            .submit_command(CommandRequest::new(Command::CreateObject {
                surface,
                id: petunia_design_foundation::ObjectId::new(3),
                name: "Kept".to_string(),
            }))
            .expect("create");
        bridge
            .dispatch_action(ActionRequest::new(
                ActionId::new("ptnd.action.file.save_as"),
                serde_json::json!({ "path": native.to_string_lossy() }),
            ))
            .expect("save_as");
    }
    let legacy = dir.join("old-project.aubrieta");
    let legacy_bytes_before = std::fs::read(&native).unwrap();
    std::fs::write(&legacy, &legacy_bytes_before).unwrap();

    // Open it: the document loads, but no path is recorded.
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge
        .dispatch_action(ActionRequest::new(
            ActionId::new("ptnd.action.file.open"),
            serde_json::json!({ "path": legacy.to_string_lossy() }),
        ))
        .expect("a legacy project must still open");
    assert_eq!(bridge.snapshot().total_objects, 1);

    // Save with no payload must refuse: there is no recorded location.
    let error = bridge
        .dispatch_action(ActionRequest::new(
            ActionId::new("ptnd.action.file.save"),
            serde_json::json!({}),
        ))
        .expect_err("Save As must be required for a migrated project");
    assert!(error.to_string().contains("path"), "{error}");

    // Save As to the legacy path upgrades to .PTND and leaves the original alone.
    bridge
        .dispatch_action(ActionRequest::new(
            ActionId::new("ptnd.action.file.save_as"),
            serde_json::json!({ "path": legacy.to_string_lossy() }),
        ))
        .expect("save_as must upgrade the suffix");
    assert_eq!(
        std::fs::read(&legacy).unwrap(),
        legacy_bytes_before,
        "the legacy file must not be overwritten"
    );
    assert!(dir.join("old-project.PTND").exists());
}
