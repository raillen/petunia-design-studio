//! Tests for interactive vector editing tools (Select, Pen, Node, Shape).

use petunia_design_application::{Command, CommandRequest};
use petunia_design_foundation::IdGenerator;
use petunia_design_geometry::GPoint;
use petunia_design_shell::bridge::PetuniaDesignGuiBridge;
use petunia_design_shell::canvas::{SnapEngine, ViewportCamera};
use petunia_design_shell::tools::*;

#[test]
fn select_tool_click_and_toggle_selection() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Select Test").expect("doc");
    let surface_id = bridge.active_surface().unwrap();

    let mut gen = IdGenerator::new();
    let id1 = gen.next_object();
    let id2 = gen.next_object();

    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id: id1,
            name: "Box1".to_string(),
        }))
        .unwrap();
    bridge
        .submit_command(CommandRequest::new(Command::SetBounds {
            id: id1,
            bounds: Some([10.0, 10.0, 50.0, 50.0]),
            rotation: 0.0,
        }))
        .unwrap();

    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id: id2,
            name: "Box2".to_string(),
        }))
        .unwrap();
    bridge
        .submit_command(CommandRequest::new(Command::SetBounds {
            id: id2,
            bounds: Some([100.0, 100.0, 50.0, 50.0]),
            rotation: 0.0,
        }))
        .unwrap();

    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = SelectTool::new();

    // 1. Click on Box 1 (point 20, 20)
    let ev1 = NormalizedPointerEvent::new(
        PointerPhase::Down,
        PointerButton::Primary,
        GPoint::new(20.0, 20.0),
        GPoint::new(20.0, 20.0),
        SemanticModifiers::default(),
    );
    tool.on_pointer_event(&ev1, &mut bridge, &camera, &mut snap)
        .unwrap();
    assert_eq!(bridge.selection().selected_ids, vec![id1]);

    // 2. Shift-click on Box 2 (point 120, 120) -> should add to selection
    let mods_shift = SemanticModifiers {
        constrain: true,
        ..Default::default()
    };
    let ev2 = NormalizedPointerEvent::new(
        PointerPhase::Down,
        PointerButton::Primary,
        GPoint::new(120.0, 120.0),
        GPoint::new(120.0, 120.0),
        mods_shift,
    );
    tool.on_pointer_event(&ev2, &mut bridge, &camera, &mut snap)
        .unwrap();
    assert_eq!(bridge.selection().count, 2);
}

#[test]
fn select_tool_drag_translation_and_duplicate_drag() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Drag Test").expect("doc");
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
    bridge
        .submit_command(CommandRequest::new(Command::SetBounds {
            id: id1,
            bounds: Some([50.0, 50.0, 100.0, 100.0]),
            rotation: 0.0,
        }))
        .unwrap();

    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = SelectTool::new();

    // Duplicate-drag gesture: Alt/Option held
    let mods_alt = SemanticModifiers {
        duplicate: true,
        ..Default::default()
    };

    // Pointer down on object
    let ev_down = NormalizedPointerEvent::new(
        PointerPhase::Down,
        PointerButton::Primary,
        GPoint::new(60.0, 60.0),
        GPoint::new(60.0, 60.0),
        mods_alt,
    );
    tool.on_pointer_event(&ev_down, &mut bridge, &camera, &mut snap)
        .unwrap();

    // Pointer move to (160, 160) -> dx = +100, dy = +100
    let ev_move = NormalizedPointerEvent::new(
        PointerPhase::Move,
        PointerButton::Primary,
        GPoint::new(160.0, 160.0),
        GPoint::new(160.0, 160.0),
        mods_alt,
    );
    tool.on_pointer_event(&ev_move, &mut bridge, &camera, &mut snap)
        .unwrap();

    // Pointer up -> commit duplicate
    let ev_up = NormalizedPointerEvent::new(
        PointerPhase::Up,
        PointerButton::Primary,
        GPoint::new(160.0, 160.0),
        GPoint::new(160.0, 160.0),
        mods_alt,
    );
    tool.on_pointer_event(&ev_up, &mut bridge, &camera, &mut snap)
        .unwrap();

    // Should have 2 objects in document now: original + duplicate
    let snap_doc = bridge.snapshot();
    assert_eq!(snap_doc.total_objects, 2);

    // Newly duplicated object should be selected and positioned at (150, 150)
    let sel = bridge.selection();
    assert_eq!(sel.count, 1);
    let dup_id = sel.selected_ids[0];
    assert_ne!(dup_id, id1);

    let dup_obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(dup_id)
        .unwrap();
    assert_eq!(dup_obj.bounds, Some([150.0, 150.0, 100.0, 100.0]));
}

#[test]
fn pen_tool_bezier_construction_and_commit() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Pen Test").expect("doc");

    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = PenTool::new();

    // 1. First anchor at (100, 100)
    let p1 = GPoint::new(100.0, 100.0);
    let ev1 = NormalizedPointerEvent::new(
        PointerPhase::Down,
        PointerButton::Primary,
        p1,
        p1,
        SemanticModifiers::default(),
    );
    tool.on_pointer_event(&ev1, &mut bridge, &camera, &mut snap)
        .unwrap();
    assert!(tool.is_active());

    // 2. Second anchor at (200, 150) with drag tangent
    let p2 = GPoint::new(200.0, 150.0);
    let ev2_down = NormalizedPointerEvent::new(
        PointerPhase::Down,
        PointerButton::Primary,
        p2,
        p2,
        SemanticModifiers::default(),
    );
    tool.on_pointer_event(&ev2_down, &mut bridge, &camera, &mut snap)
        .unwrap();

    // Drag handle to (220, 170)
    let ev2_drag = NormalizedPointerEvent::new(
        PointerPhase::Move,
        PointerButton::Primary,
        GPoint::new(220.0, 170.0),
        GPoint::new(220.0, 170.0),
        SemanticModifiers::default(),
    );
    tool.on_pointer_event(&ev2_drag, &mut bridge, &camera, &mut snap)
        .unwrap();

    let ev2_up = NormalizedPointerEvent::new(
        PointerPhase::Up,
        PointerButton::Primary,
        GPoint::new(220.0, 170.0),
        GPoint::new(220.0, 170.0),
        SemanticModifiers::default(),
    );
    tool.on_pointer_event(&ev2_up, &mut bridge, &camera, &mut snap)
        .unwrap();

    // Commit path explicitly
    let changes = tool.commit_path(&mut bridge, false).unwrap();
    assert!(!changes.is_empty());
    assert!(!tool.is_active());

    let snap_doc = bridge.snapshot();
    assert_eq!(snap_doc.total_objects, 1);
    assert_eq!(bridge.selection().count, 1);
}

#[test]
fn shape_tool_creates_constrained_square() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Shape Test").expect("doc");

    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = ShapeTool::new(ShapeKind::Rectangle);

    let mods_shift = SemanticModifiers {
        constrain: true, // 1:1 square
        ..Default::default()
    };

    // Down at (50, 50)
    let p0 = GPoint::new(50.0, 50.0);
    let ev_down = NormalizedPointerEvent::new(
        PointerPhase::Down,
        PointerButton::Primary,
        p0,
        p0,
        mods_shift,
    );
    tool.on_pointer_event(&ev_down, &mut bridge, &camera, &mut snap)
        .unwrap();

    // Drag to (150, 100) -> w = 100, h = 50 -> constrain makes both 100!
    let p1 = GPoint::new(150.0, 100.0);
    let ev_move = NormalizedPointerEvent::new(
        PointerPhase::Move,
        PointerButton::Primary,
        p1,
        p1,
        mods_shift,
    );
    tool.on_pointer_event(&ev_move, &mut bridge, &camera, &mut snap)
        .unwrap();

    let ev_up =
        NormalizedPointerEvent::new(PointerPhase::Up, PointerButton::Primary, p1, p1, mods_shift);
    tool.on_pointer_event(&ev_up, &mut bridge, &camera, &mut snap)
        .unwrap();

    assert_eq!(bridge.snapshot().total_objects, 1);
    let sel_obj_id = bridge.selection().selected_ids[0];
    let sel_obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(sel_obj_id)
        .unwrap();

    assert_eq!(sel_obj.name, "Rectangle");
    // Dimensions should be 100 x 100 because of constrain
    assert_eq!(sel_obj.bounds, Some([50.0, 50.0, 100.0, 100.0]));
}

#[test]
fn tool_manager_tool_switching_cancels_provisional_gestures() {
    let mut manager = ToolManager::new();
    assert_eq!(manager.active_kind(), ToolKind::Select);

    manager.set_tool(ToolKind::Pen);
    assert_eq!(manager.active_kind(), ToolKind::Pen);

    manager.set_tool(ToolKind::Rectangle);
    assert_eq!(manager.active_kind(), ToolKind::Rectangle);
}
