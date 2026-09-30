//! Tests for PetuniaDesignGuiBridge, semantic application ports, and reactive presentation models.

use std::sync::Arc;

use petunia_design_application::{
    create_shape_commands, ActionId, ActionRequest, Command, CommandRequest,
};
use petunia_design_document::{
    AdjustmentItem, AdjustmentKind, AppearanceStack, EffectItem, EffectKind, ShapeKind,
};
use petunia_design_foundation::{IdGenerator, ObjectId};
use petunia_design_shell::bridge::*;
use petunia_design_shell::shell::PetuniaShell;

#[test]
fn bridge_tracks_background_jobs() {
    let bridge = PetuniaDesignGuiBridge::new();
    assert_eq!(bridge.jobs().list_jobs().len(), 0);
    let (id, token) = bridge.jobs().spawn_job("Exporting PDF");
    assert_eq!(bridge.jobs().list_jobs().len(), 1);
    bridge.jobs().update_progress(id, 50);
    assert_eq!(bridge.jobs().list_jobs()[0].percent, 50);
    bridge.jobs().cancel_job(id);
    assert!(token.is_cancelled());
}

#[test]
fn canvas_snapshot_uses_world_frame_and_rotation() {
    let mut shell = PetuniaShell::new(1000.0, 800.0);
    shell.new_document("World Frame").expect("new document");
    let surface = shell.bridge.active_surface().expect("surface");
    let id = ObjectId::new(1);

    shell
        .bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface,
            id,
            name: "Rotated".to_string(),
        }))
        .expect("create");
    shell
        .bridge
        .submit_command(CommandRequest::new(Command::SetBounds {
            id,
            bounds: Some([10.0, 20.0, 40.0, 30.0]),
            rotation: std::f64::consts::FRAC_PI_2,
        }))
        .expect("frame");

    let snapshot = shell.canvas_snapshot();
    let object = snapshot
        .objects
        .iter()
        .find(|object| object.id == id)
        .expect("world object projection");
    assert_eq!(object.frame_origin, [10.0, 20.0]);
    assert_eq!(object.size, [40.0, 30.0]);
    assert!((object.rotation - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
    assert!((object.world_bounds[0] + 20.0).abs() < 1e-9);
    assert!((object.world_bounds[1] - 20.0).abs() < 1e-9);
    assert!((object.world_bounds[2] - 30.0).abs() < 1e-9);
    assert!((object.world_bounds[3] - 40.0).abs() < 1e-9);
}

/// Fixture for the snapshot-cache tests (DOSSIER §15 item 3): one shaped
/// object with evaluable geometry plus a non-empty appearance stack, so the
/// cached payload (`outline`/`shape`/`effects`/`adjustments`) is meaningful.
fn snapshot_cache_fixture() -> (PetuniaShell, ObjectId) {
    let mut shell = PetuniaShell::new(1000.0, 800.0);
    shell.new_document("Snapshot Cache").expect("new document");
    let surface = shell.bridge.active_surface().expect("surface");
    let id = ObjectId::new(1);
    let mut commands = create_shape_commands(
        surface,
        id,
        "Cached".to_string(),
        ShapeKind::Rectangle {
            corner_radii: [0.0; 4],
        },
        Some([10.0, 20.0, 40.0, 30.0]),
        Some("ptnd.blue/500".to_string()),
        None,
    );
    commands.push(Command::SetAppearance {
        id,
        appearance: Some(AppearanceStack {
            effects: vec![EffectItem {
                id: 1,
                kind: EffectKind::GaussianBlur { radius: 2.0 },
                visible: true,
            }],
            adjustments: vec![AdjustmentItem::new(1, AdjustmentKind::default_levels())],
            ..AppearanceStack::default()
        }),
    });
    shell
        .bridge
        .submit_all("Build snapshot-cache fixture", commands)
        .expect("fixture commands");
    (shell, id)
}

#[test]
fn snapshot_cache_shares_payload_allocations_without_mutation() {
    let (shell, id) = snapshot_cache_fixture();
    let first = shell.canvas_snapshot();
    let second = shell.canvas_snapshot();
    let a = first
        .objects
        .iter()
        .find(|object| object.id == id)
        .expect("object in first snapshot");
    let b = second
        .objects
        .iter()
        .find(|object| object.id == id)
        .expect("object in second snapshot");
    assert!(a.outline.is_some(), "fixture must have evaluable geometry");
    assert!(a.shape.is_some(), "fixture must carry its shape");
    assert!(!a.effects.is_empty(), "fixture must carry effects");
    assert!(!a.adjustments.is_empty(), "fixture must carry adjustments");
    assert!(
        Arc::ptr_eq(a.outline.as_ref().unwrap(), b.outline.as_ref().unwrap()),
        "outline allocation must be shared while (revision, selection) is unchanged"
    );
    assert!(
        Arc::ptr_eq(a.shape.as_ref().unwrap(), b.shape.as_ref().unwrap()),
        "shape allocation must be shared while (revision, selection) is unchanged"
    );
    assert!(
        Arc::ptr_eq(&a.effects, &b.effects),
        "effects allocation must be shared while (revision, selection) is unchanged"
    );
    assert!(
        Arc::ptr_eq(&a.adjustments, &b.adjustments),
        "adjustments allocation must be shared while (revision, selection) is unchanged"
    );
    assert_eq!(first, second, "cached snapshot must stay observable-equal");
}

#[test]
fn snapshot_cache_tracks_selection_version_in_active_flags() {
    let (mut shell, id) = snapshot_cache_fixture();
    let before = shell.canvas_snapshot();
    assert!(
        !before
            .objects
            .iter()
            .find(|object| object.id == id)
            .expect("object before select")
            .active,
        "nothing is active before selection"
    );
    shell.bridge.set_selection(vec![id]);
    let selected = shell.canvas_snapshot();
    assert!(
        selected
            .objects
            .iter()
            .find(|object| object.id == id)
            .expect("object after select")
            .active,
        "set_selection must flip the active flag through the selection version"
    );
    shell.bridge.toggle_selection(id);
    let cleared = shell.canvas_snapshot();
    assert!(
        !cleared
            .objects
            .iter()
            .find(|object| object.id == id)
            .expect("object after toggle")
            .active,
        "toggle off must clear the active flag through the selection version"
    );
}

#[test]
fn snapshot_cache_rebuilds_payload_on_document_revision() {
    let (mut shell, id) = snapshot_cache_fixture();
    let before = shell.canvas_snapshot();
    let before_projection = before
        .objects
        .iter()
        .find(|object| object.id == id)
        .expect("object before mutation");
    shell
        .bridge
        .submit_all(
            "Nudge",
            vec![Command::SetBounds {
                id,
                bounds: Some([20.0, 30.0, 40.0, 30.0]),
                rotation: 0.0,
            }],
        )
        .expect("transact mutates through one undo entry");
    let after = shell.canvas_snapshot();
    let after_projection = after
        .objects
        .iter()
        .find(|object| object.id == id)
        .expect("object after mutation");
    assert!(
        !Arc::ptr_eq(
            before_projection.outline.as_ref().unwrap(),
            after_projection.outline.as_ref().unwrap()
        ),
        "a transact revision bump must rebuild the outline allocation"
    );
    assert!(
        !Arc::ptr_eq(&before_projection.effects, &after_projection.effects),
        "a transact revision bump must rebuild the effects allocation"
    );
    assert_eq!(
        after_projection.world_bounds[0], 20.0,
        "the rebuilt snapshot must carry the mutated frame"
    );
}

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

#[test]
fn file_close_action_closes_active_session_and_guards_dirty() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Doc To Close").expect("doc");
    assert!(bridge.is_dirty());

    // Without force, dispatching file.close is rejected
    let res = bridge.dispatch_action(ActionRequest::new(
        ActionId::new("ptnd.action.file.close"),
        serde_json::json!({ "force": false }),
    ));
    assert!(
        res.is_err(),
        "closing dirty session without force must fail"
    );
    assert!(bridge.session().is_some());

    // With force, dispatching file.close closes the session
    let res_force = bridge.dispatch_action(ActionRequest::new(
        ActionId::new("ptnd.action.file.close"),
        serde_json::json!({ "force": true }),
    ));
    assert!(res_force.is_ok(), "force close must succeed");
    assert!(bridge.session().is_none(), "no session should remain");
}

#[test]
fn file_quit_action_closes_all_sessions_and_guards_dirty() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Doc A").expect("doc a");
    bridge.new_document("Doc B").expect("doc b");
    assert_eq!(bridge.sessions().len(), 2);
    assert!(bridge.any_session_dirty());

    // Without force, quit is rejected and nothing closes
    let res = bridge.dispatch_action(ActionRequest::new(
        ActionId::new("ptnd.action.file.quit"),
        serde_json::json!({ "force": false }),
    ));
    assert!(
        res.is_err(),
        "quitting with a dirty document without force must fail"
    );
    assert_eq!(
        bridge.sessions().len(),
        2,
        "a refused quit must not close a single tab"
    );

    // With force, quit closes every tab
    let res_force = bridge.dispatch_action(ActionRequest::new(
        ActionId::new("ptnd.action.file.quit"),
        serde_json::json!({ "force": true }),
    ));
    assert!(res_force.is_ok(), "force quit must succeed");
    assert_eq!(bridge.sessions().len(), 0);
    assert!(bridge.session().is_none());
    assert_eq!(bridge.active_session_index(), None);
}

#[test]
fn multi_doc_keeps_tabs_switches_and_closes_the_right_one() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("First Doc").expect("first");
    let surface_1 = bridge.active_surface().expect("surface 1");

    bridge.new_document("Second Doc").expect("second");
    let surface_2 = bridge.active_surface().expect("surface 2");
    assert_eq!(surface_1, surface_2, "surface IDs are scoped per document");

    let titles: Vec<String> = bridge
        .sessions()
        .iter()
        .map(|session| session.title().to_string())
        .collect();
    assert_eq!(
        titles,
        vec!["First Doc", "Second Doc"],
        "file.new adds a tab"
    );
    assert_eq!(bridge.active_session_index(), Some(1));

    // Switching activates that document's own surface.
    assert!(bridge.switch_session(0).expect("switch to first tab"));
    assert_eq!(bridge.active_session_index(), Some(0));
    assert_eq!(bridge.session().expect("active").title(), "First Doc");
    assert_eq!(bridge.active_surface(), Some(surface_1));

    // A stale tab index is a no-op, not a panic and not a close.
    assert!(!bridge.switch_session(9).expect("stale index is a miss"));
    assert_eq!(bridge.sessions().len(), 2);

    // Closing the active tab promotes the neighbour to its left.
    assert!(bridge.close_session_at(0, true).expect("close first"));
    assert_eq!(bridge.sessions().len(), 1);
    assert_eq!(bridge.active_session_index(), Some(0));
    assert_eq!(bridge.active_surface(), Some(surface_2));

    // Closing the last tab leaves no session.
    assert!(bridge.close_session_at(0, true).expect("close last"));
    assert_eq!(bridge.sessions().len(), 0);
    assert!(bridge.session().is_none());
    assert_eq!(bridge.active_session_index(), None);
}

#[test]
fn closing_a_dirty_background_tab_needs_confirmation() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Dirty First").expect("first");
    bridge.new_document("Second").expect("second");

    // Tab 0 is dirty (the initial canvas) and is not the active tab.
    assert!(!bridge
        .close_session_at(0, false)
        .expect("unconfirmed close is reported, not fatal"));
    assert_eq!(bridge.sessions().len(), 2, "a refused close keeps the tab");

    assert!(bridge
        .close_session_at(0, true)
        .expect("confirmed close succeeds"));
    assert_eq!(bridge.sessions().len(), 1);
    assert_eq!(
        bridge.active_session_index(),
        Some(0),
        "closing a background tab must not change the active one"
    );
}
