use super::*;
use serde_json::json;

fn controller() -> DesktopController {
    DesktopController::new()
}
fn temporary_directory() -> PathBuf {
    let path = std::env::temp_dir().join(format!("petunia-desktop-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn authorial_shape_fill_transform_undo_save_reopen_and_render() {
    let mut desktop = controller();
    desktop
        .invoke(
            "shape",
            &json!({"kind":"rectangle","x":20,"y":20,"width":40,"height":40}),
        )
        .unwrap();
    assert_eq!(desktop.state()["selectionCount"], 1);
    desktop
        .invoke("setFill", &json!({"color":"#F04020"}))
        .unwrap();
    desktop
        .invoke("transform", &json!({"x":30,"y":30,"width":60,"height":50}))
        .unwrap();
    assert_eq!(desktop.state()["selection"]["width"], 60.0);
    desktop.invoke("undo", &json!({})).unwrap();
    assert_eq!(desktop.state()["selection"]["width"], 40.0);
    desktop.invoke("redo", &json!({})).unwrap();
    let pixels = desktop.render_rgba(128, 128, 1.0).unwrap();
    assert_eq!(pixels.len(), 128 * 128 * 4);
    assert!(pixels
        .as_chunks::<4>()
        .0
        .iter()
        .any(|pixel| pixel[0] > 200 && pixel[1] < 100 && pixel[2] < 80));
    let directory = temporary_directory();
    let path = directory.join("desenho.ptnd");
    desktop.invoke("saveAs", &json!({"path":path})).unwrap();
    assert_eq!(desktop.state()["dirty"], false);
    let mut reopened = controller();
    reopened.invoke("open", &json!({"path":path})).unwrap();
    assert_eq!(reopened.state()["layers"].as_array().unwrap().len(), 1);
    // Open fits the page. Restore the same view to compare actual authorial pixels.
    reopened.session.view_mut().scale = 1.0;
    reopened.session.view_mut().pan_x = 0.0;
    reopened.session.view_mut().pan_y = 0.0;
    assert_eq!(reopened.render_rgba(128, 128, 1.0).unwrap(), pixels);
    desktop
        .invoke("exportPng", &json!({"path":directory.join("desenho.png")}))
        .unwrap();
    assert_eq!(
        image::open(directory.join("desenho.png")).unwrap().width(),
        desktop.page_size().0 as u32
    );
    assert!(desktop.invoke("saveAs", &json!({"path":path})).is_err());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn invalid_edits_are_atomic_and_dirty_document_is_guarded() {
    let mut desktop = controller();
    desktop.invoke("shape", &json!({"kind":"ellipse"})).unwrap();
    let snapshot = desktop.session.document().to_json().unwrap();
    for (command, payload) in [
        ("transform", json!({"width":-1})),
        ("setFill", json!({"color":"#🦀123"})),
        ("shape", json!({"kind":"triangle"})),
        ("locked", json!({"id":"not-an-id"})),
    ] {
        assert!(desktop.invoke(command, &payload).is_err());
        assert_eq!(desktop.session.document().to_json().unwrap(), snapshot);
    }
    assert!(matches!(
        desktop.invoke("new", &json!({})),
        Err(DesktopError::UnsavedDocument)
    ));
    assert!(matches!(
        desktop.invoke("open", &json!({"path":"absent.ptnd"})),
        Err(DesktopError::UnsavedDocument)
    ));
    assert_eq!(desktop.session.document().to_json().unwrap(), snapshot);
    desktop.invoke("new", &json!({"discard":true})).unwrap();
    assert_eq!(desktop.state()["dirty"], false);
}

#[test]
fn staged_pen_and_shape_guard_tool_persona_and_history_until_cancel() {
    let mut desktop = controller();
    desktop
        .invoke("chooseTool", &json!({"tool":"pen"}))
        .unwrap();
    desktop
        .invoke("pointer", &json!({"phase":"down","x":10,"y":10}))
        .unwrap();
    desktop
        .invoke("pointer", &json!({"phase":"up","x":10,"y":10}))
        .unwrap();
    assert_eq!(desktop.state()["gestureActive"], true);
    assert!(matches!(
        desktop.invoke("chooseTool", &json!({"tool":"select"})),
        Err(DesktopError::PendingInteraction)
    ));
    assert!(matches!(
        desktop.invoke("persona", &json!({"operation":"activate","id":"layout"})),
        Err(DesktopError::PendingInteraction)
    ));
    assert!(matches!(
        desktop.invoke("undo", &json!({})),
        Err(DesktopError::PendingInteraction)
    ));
    desktop.invoke("escape", &json!({})).unwrap();
    assert_eq!(desktop.state()["gestureActive"], false);
    desktop
        .invoke("chooseTool", &json!({"tool":"rectangle"}))
        .unwrap();
    desktop
        .invoke("pointer", &json!({"phase":"down","x":10,"y":10}))
        .unwrap();
    desktop
        .invoke("pointer", &json!({"phase":"move","x":50,"y":70}))
        .unwrap();
    assert_eq!(desktop.session.revision().0, 0);
    desktop
        .invoke("pointer", &json!({"phase":"up","x":50,"y":70}))
        .unwrap();
    assert_eq!(desktop.session.revision().0, 1);
    assert_eq!(desktop.state()["selection"]["height"], 60.0);
}

#[test]
fn pen_enter_commits_one_open_path_and_modifiers_preserve_selection() {
    let mut desktop = controller();
    desktop
        .invoke("chooseTool", &json!({"tool":"pen"}))
        .unwrap();
    for (x, y) in [(10, 10), (80, 30)] {
        desktop
            .invoke("pointer", &json!({"phase":"down","x":x,"y":y}))
            .unwrap();
        desktop
            .invoke("pointer", &json!({"phase":"up","x":x,"y":y}))
            .unwrap();
    }
    desktop.invoke("enter", &json!({})).unwrap();
    assert_eq!(desktop.session.revision().0, 1);
    assert_eq!(desktop.state()["gestureActive"], false);
    let id = desktop
        .session
        .document()
        .scene
        .page_roots(desktop.session.active_page())
        .unwrap()[0];
    assert!(
        !desktop
            .session
            .document()
            .scene
            .get_node(id)
            .unwrap()
            .item_path()
            .unwrap()
            .contours[0]
            .closed
    );
}

#[test]
fn customizations_persist_outside_document_and_invalid_batch_does_not_apply() {
    let directory = temporary_directory();
    let path = directory.join("fresh-config/petunia-studio/preferences.json");
    let mut desktop = DesktopController::with_preferences(path.clone());
    desktop
        .invoke(
            "preferences",
            &json!({"appearance":"highContrast","iconFamily":"tabler","iconStyle":"fill","uiScale":1.5}),
        )
        .unwrap();
    desktop
        .invoke(
            "persona",
            &json!({"operation":"duplicate","id":"vector","name":"Ilustração"}),
        )
        .unwrap();
    desktop
        .invoke(
            "rebind",
            &json!({"action":"tool.select","shortcut":"Ctrl+L"}),
        )
        .unwrap();
    assert_eq!(desktop.session.revision().0, 0);
    let previous = desktop.preferences.clone();
    assert!(desktop
        .invoke(
            "preferences",
            &json!({"appearance":"light","density":"invalid"})
        )
        .is_err());
    assert_eq!(desktop.preferences, previous);
    assert!(desktop
        .invoke("preferences", &json!({"appearance":"light","uiScale":3.0}))
        .is_err());
    assert_eq!(desktop.preferences, previous);
    let reopened = DesktopController::with_preferences(path);
    assert_eq!(reopened.preferences, previous);
    assert_eq!(
        reopened
            .session
            .shortcuts()
            .combo_for(petunia_ui::ActionId::SelectTool)
            .unwrap()
            .display(),
        "Ctrl+l"
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn lock_visibility_duplicate_delete_are_undoable() {
    let mut desktop = controller();
    desktop
        .invoke("shape", &json!({"kind":"rectangle"}))
        .unwrap();
    let id = desktop.session.selection().single().unwrap();
    desktop
        .invoke("locked", &json!({"id":id,"locked":true}))
        .unwrap();
    assert!(!desktop.session.is_editable(id));
    assert!(desktop.invoke("selectLayer", &json!({"id":id})).is_err());
    desktop.invoke("undo", &json!({})).unwrap();
    assert!(desktop.session.is_editable(id));
    desktop.invoke("selectLayer", &json!({"id":id})).unwrap();
    desktop.invoke("duplicate", &json!({})).unwrap();
    assert_eq!(desktop.session.document().scene.len(), 2);
    desktop.invoke("delete", &json!({})).unwrap();
    assert_eq!(desktop.session.document().scene.len(), 1);
    desktop.invoke("undo", &json!({})).unwrap();
    assert_eq!(desktop.session.document().scene.len(), 2);
    desktop
        .invoke("visibility", &json!({"id":id,"visible":false}))
        .unwrap();
    assert!(
        !desktop
            .session
            .document()
            .scene
            .get_node(id)
            .unwrap()
            .visible
    );
}

#[test]
fn semantic_node_selection_numeric_move_preserves_ids_and_handles() {
    let mut desktop = controller();
    desktop
        .invoke(
            "shape",
            &json!({"kind":"ellipse","x":20,"y":20,"width":80,"height":60}),
        )
        .unwrap();
    let rows = desktop.state()["nodes"].as_array().unwrap().clone();
    assert_eq!(rows.len(), 4);
    let target = &rows[0];
    let object = desktop.session.selection().single().unwrap();
    let before = desktop
        .session
        .document()
        .scene
        .get_node(object)
        .unwrap()
        .item_path()
        .unwrap()
        .clone();
    desktop
        .invoke(
            "selectNode",
            &json!({"object":target["object"],"contour":target["contour"],"node":target["node"]}),
        )
        .unwrap();
    assert_eq!(desktop.state()["nodes"][0]["selected"], true);
    assert!(desktop.state()["overlays"]
        .as_array()
        .unwrap()
        .iter()
        .any(|overlay| overlay["type"] == "node" && overlay["selected"] == true));
    desktop
        .invoke("command", &json!({"id":"nudge.right"}))
        .unwrap();
    assert_eq!(
        desktop.state()["nodes"][0]["x"].as_f64().unwrap(),
        target["x"].as_f64().unwrap() + 1.0
    );
    assert_eq!(desktop.state()["nodes"][1]["x"], rows[1]["x"]);
    desktop.invoke("undo", &json!({})).unwrap();
    desktop.invoke("nudge", &json!({"dx":0,"dy":10})).unwrap();
    assert_eq!(
        desktop.state()["nodes"][0]["y"].as_f64().unwrap(),
        target["y"].as_f64().unwrap() + 10.0
    );
    assert_eq!(desktop.state()["nodes"][1]["y"], rows[1]["y"]);
    desktop.invoke("undo", &json!({})).unwrap();
    let x = target["x"].as_f64().unwrap() + 12.0;
    let y = target["y"].as_f64().unwrap() - 3.0;
    desktop
        .invoke(
            "moveNode",
            &json!({"object":target["object"],"contour":0,"node":0,"x":x,"y":y}),
        )
        .unwrap();
    let moved = desktop
        .session
        .document()
        .scene
        .get_node(object)
        .unwrap()
        .item_path()
        .unwrap();
    assert_eq!(moved.contours[0].id, before.contours[0].id);
    assert_eq!(
        moved.contours[0].nodes[0].id,
        before.contours[0].nodes[0].id
    );
    assert_eq!(
        moved.contours[0].nodes[0].handle_in.unwrap().x,
        before.contours[0].nodes[0].handle_in.unwrap().x + 12.0
    );
    desktop.invoke("undo", &json!({})).unwrap();
    assert_eq!(
        desktop
            .session
            .document()
            .scene
            .get_node(object)
            .unwrap()
            .item_path()
            .unwrap(),
        &before
    );
    let revision = desktop.session.revision();
    assert!(desktop
        .invoke(
            "moveNode",
            &json!({"object":target["object"],"contour":99,"node":0,"x":x,"y":y})
        )
        .is_err());
    assert_eq!(desktop.session.revision(), revision);
}

#[test]
fn unchanged_authorial_fields_do_not_create_history_and_alpha_roundtrips() {
    let mut desktop = controller();
    desktop
        .invoke("shape", &json!({"kind":"rectangle"}))
        .unwrap();
    let selection = desktop.state()["selection"].clone();
    let revision = desktop.session.revision();
    desktop.invoke("transform", &selection).unwrap();
    desktop
        .invoke("setFill", &json!({"color":"#000000"}))
        .unwrap();
    assert_eq!(desktop.session.revision(), revision);
    desktop
        .invoke("setFill", &json!({"color":"#8066AA22"}))
        .unwrap();
    assert_eq!(desktop.state()["selection"]["fill"], "#8066AA22");
    desktop.invoke("undo", &json!({})).unwrap();
    assert_eq!(desktop.state()["selection"]["fill"], "#000000");
}

#[test]
fn export_requires_explicit_overwrite_and_preserves_existing_file() {
    let directory = temporary_directory();
    let path = directory.join("existing.png");
    let original = b"Existing file contents";
    std::fs::write(&path, original).unwrap();
    let mut desktop = controller();
    desktop
        .invoke("shape", &json!({"kind":"rectangle"}))
        .unwrap();
    assert!(desktop.invoke("exportPng", &json!({"path":path})).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    desktop
        .invoke("exportPng", &json!({"path":path,"overwrite":true}))
        .unwrap();
    assert!(image::open(&path).is_ok());
    assert!(std::fs::read_dir(&directory).unwrap().all(|entry| !entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with(".petunia-export")));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn undo_insertion_drops_stale_selection() {
    let mut desktop = controller();
    desktop
        .invoke("shape", &json!({"kind":"rectangle"}))
        .unwrap();
    assert_eq!(desktop.state()["selectionCount"], 1);
    desktop.invoke("undo", &json!({})).unwrap();
    assert_eq!(desktop.state()["selectionCount"], 0);
    assert_eq!(desktop.state()["selection"]["editable"], false);
    desktop.invoke("redo", &json!({})).unwrap();
    assert_eq!(desktop.state()["layers"].as_array().unwrap().len(), 1);
}
