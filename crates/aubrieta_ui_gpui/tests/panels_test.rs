//! Tests for UI panels (Layers, Properties, History).

use aubrieta_application::{Command, CommandRequest};
use aubrieta_foundation::IdGenerator;
use aubrieta_ui_gpui::bridge::AubrietaGuiBridge;
use aubrieta_ui_gpui::panels::*;

#[test]
fn layers_panel_visibility_lock_and_reorder() {
    let mut bridge = AubrietaGuiBridge::new();
    bridge.new_document("Layers Test").expect("doc");
    let surface_id = bridge.active_surface().unwrap();

    let mut gen = IdGenerator::new();
    let id1 = gen.next_object();
    let id2 = gen.next_object();

    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id: id1,
            name: "Layer 1".to_string(),
        }))
        .unwrap();

    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id: id2,
            name: "Layer 2".to_string(),
        }))
        .unwrap();

    let controller = LayersPanelController::new();

    // 1. Check initial layers model
    let model = controller.query_model(&bridge);
    assert_eq!(model.rows.len(), 2);
    assert_eq!(model.rows[0].id, id1);
    assert_eq!(model.rows[1].id, id2);
    assert!(model.rows[0].visible);
    assert!(!model.rows[0].locked);

    // 2. Toggle visibility of Layer 1
    controller.toggle_visibility(&mut bridge, id1).unwrap();
    let m_vis = controller.query_model(&bridge);
    assert!(!m_vis.rows[0].visible);

    // 3. Toggle lock of Layer 1
    controller.toggle_lock(&mut bridge, id1).unwrap();
    let m_lock = controller.query_model(&bridge);
    assert!(m_lock.rows[0].locked);

    // 4. Reorder: Move Layer 2 to index 0
    controller
        .reorder_row(&mut bridge, surface_id, id2, 0)
        .unwrap();
    let m_reorder = controller.query_model(&bridge);
    assert_eq!(m_reorder.rows[0].id, id2);
    assert_eq!(m_reorder.rows[1].id, id1);
}

#[test]
fn properties_panel_updates_fill_stroke_and_opacity() {
    let mut bridge = AubrietaGuiBridge::new();
    bridge.new_document("Props Test").expect("doc");
    let surface_id = bridge.active_surface().unwrap();

    let mut gen = IdGenerator::new();
    let id1 = gen.next_object();

    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id: id1,
            name: "Shape".to_string(),
        }))
        .unwrap();

    bridge.set_selection(vec![id1]);

    let controller = PropertiesPanelController::new();

    // Set fill
    controller
        .set_fill(&mut bridge, Some("aubrieta.indigo/500".to_string()))
        .unwrap();
    // Set stroke
    controller
        .set_stroke(&mut bridge, Some("aubrieta.gray/800".to_string()), 3.0)
        .unwrap();
    // Set opacity
    controller.set_opacity(&mut bridge, 0.75).unwrap();
    // Set bounds
    controller
        .set_bounds(&mut bridge, [100.0, 100.0, 250.0, 180.0], 0.0)
        .unwrap();

    let model = controller.query_model(&bridge);
    assert_eq!(model.fill.as_deref(), Some("aubrieta.indigo/500"));
    assert_eq!(model.stroke.as_deref(), Some("aubrieta.gray/800"));
    assert_eq!(model.stroke_width, 3.0);
    assert!((model.opacity - 0.75).abs() < 1e-6);
    assert_eq!(model.bounds, Some([100.0, 100.0, 250.0, 180.0]));
}

#[test]
fn history_panel_undo_redo_inspection() {
    let mut bridge = AubrietaGuiBridge::new();
    bridge.new_document("History Test").expect("doc");
    let surface_id = bridge.active_surface().unwrap();

    let mut gen = IdGenerator::new();
    let id1 = gen.next_object();

    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id: id1,
            name: "Item".to_string(),
        }))
        .unwrap();

    let controller = HistoryPanelController::new();

    let m1 = controller.query_model(&bridge);
    assert_eq!(m1.undo_stack.len(), 1);
    assert_eq!(m1.redo_stack.len(), 0);
    assert!(m1.can_undo);
    assert!(!m1.can_redo);

    // Undo via controller
    controller.undo(&mut bridge).unwrap();
    let m2 = controller.query_model(&bridge);
    assert_eq!(m2.undo_stack.len(), 0);
    assert_eq!(m2.redo_stack.len(), 1);
    assert!(!m2.can_undo);
    assert!(m2.can_redo);

    // Redo via controller
    controller.redo(&mut bridge).unwrap();
    let m3 = controller.query_model(&bridge);
    assert_eq!(m3.undo_stack.len(), 1);
    assert_eq!(m3.redo_stack.len(), 0);
    assert!(m3.can_undo);
    assert!(!m3.can_redo);
}
