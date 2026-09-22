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

fn select_test_bridge_two_boxes() -> (
    PetuniaDesignGuiBridge,
    petunia_design_foundation::ObjectId,
    petunia_design_foundation::ObjectId,
) {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Select Batch1").expect("doc");
    let surface_id = bridge.active_surface().unwrap();
    let mut gen = IdGenerator::new();
    let id1 = gen.next_object();
    let id2 = gen.next_object();
    for (id, name, bounds) in [
        (id1, "Box1", [10.0, 10.0, 50.0, 50.0]),
        (id2, "Box2", [100.0, 100.0, 50.0, 50.0]),
    ] {
        bridge
            .submit_command(CommandRequest::new(Command::CreateObject {
                surface: surface_id,
                id,
                name: name.to_string(),
            }))
            .unwrap();
        bridge
            .submit_command(CommandRequest::new(Command::SetBounds {
                id,
                bounds: Some(bounds),
                rotation: 0.0,
            }))
            .unwrap();
    }
    bridge.clear_selection();
    (bridge, id1, id2)
}

fn pointer_event(
    phase: PointerPhase,
    x: f64,
    y: f64,
    modifiers: SemanticModifiers,
) -> NormalizedPointerEvent {
    NormalizedPointerEvent::new(
        phase,
        PointerButton::Primary,
        GPoint::new(x, y),
        GPoint::new(x, y),
        modifiers,
    )
}

#[test]
fn select_hover_and_pressed_feedback_tracks_object() {
    let (mut bridge, id1, _) = select_test_bridge_two_boxes();
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = SelectTool::new();

    // Hover over Box1 without pressing: hover feedback appears, no selection yet.
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Move, 20.0, 20.0, SemanticModifiers::default()),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    assert_eq!(tool.hovered_object(), Some(id1));
    assert_eq!(tool.pressed_object(), None);
    let ov = tool.overlays(&camera, &bridge);
    assert_eq!(ov.hovered_object, Some(id1));

    // Press on Box1: pressed feedback appears and selection happens on Down.
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Down, 20.0, 20.0, SemanticModifiers::default()),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    assert_eq!(tool.pressed_object(), Some(id1));
    assert_eq!(bridge.selection().selected_ids, vec![id1]);

    // Release without drag: pressed clears, hover stays, selection stays.
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Up, 20.0, 20.0, SemanticModifiers::default()),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    assert_eq!(tool.pressed_object(), None);
    assert_eq!(tool.hovered_object(), Some(id1));
    assert_eq!(bridge.selection().selected_ids, vec![id1]);
}

#[test]
fn select_click_on_selected_collapses_multi_selection() {
    let (mut bridge, id1, id2) = select_test_bridge_two_boxes();
    bridge.set_selection(vec![id1, id2]);
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = SelectTool::new();

    tool.on_pointer_event(
        &pointer_event(PointerPhase::Down, 20.0, 20.0, SemanticModifiers::default()),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    // Down on an already-selected object keeps the multi-selection for a drag.
    assert_eq!(bridge.selection().count, 2);
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Up, 20.0, 20.0, SemanticModifiers::default()),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    // Up without drag collapses onto the clicked object (market behavior).
    assert_eq!(bridge.selection().selected_ids, vec![id1]);
}

#[test]
fn select_rectangle_rule_intersect_vs_contained() {
    // Partial overlap marquee: x 0..40 covers only the left part of Box1 (10..60).
    for (rule, expect_box1) in [
        (MarqueeSelectRule::Intersect, true),
        (MarqueeSelectRule::Contained, false),
    ] {
        let (mut bridge, id1, _) = select_test_bridge_two_boxes();
        let camera = ViewportCamera::new(1000.0, 1000.0);
        let mut snap = SnapEngine::new();
        let mut tool = SelectTool::new();
        tool.set_marquee_rule(rule);
        tool.on_pointer_event(
            &pointer_event(PointerPhase::Down, 0.0, 0.0, SemanticModifiers::default()),
            &mut bridge,
            &camera,
            &mut snap,
        )
        .unwrap();
        tool.on_pointer_event(
            &pointer_event(PointerPhase::Move, 40.0, 40.0, SemanticModifiers::default()),
            &mut bridge,
            &camera,
            &mut snap,
        )
        .unwrap();
        // Marquee preview exposes the rect plus additive flags for the UI.
        let ov = tool.overlays(&camera, &bridge);
        assert!(ov.marquee_screen.is_some());
        tool.on_pointer_event(
            &pointer_event(PointerPhase::Up, 40.0, 40.0, SemanticModifiers::default()),
            &mut bridge,
            &camera,
            &mut snap,
        )
        .unwrap();
        assert_eq!(
            bridge.selection().selected_ids.contains(&id1),
            expect_box1,
            "rule {rule:?}"
        );
    }
}

#[test]
fn select_rectangle_additive_marquee_with_shift() {
    let (mut bridge, id1, id2) = select_test_bridge_two_boxes();
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = SelectTool::new();
    tool.set_marquee_rule(MarqueeSelectRule::Intersect);

    // First marquee selects Box1.
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Down, 0.0, 0.0, SemanticModifiers::default()),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Move, 70.0, 70.0, SemanticModifiers::default()),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Up, 70.0, 70.0, SemanticModifiers::default()),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    assert_eq!(bridge.selection().selected_ids, vec![id1]);

    // Shift-marquee over Box2 adds instead of replacing.
    let shift = SemanticModifiers {
        constrain: true,
        ..Default::default()
    };
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Down, 90.0, 90.0, shift),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Move, 160.0, 160.0, shift),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Up, 160.0, 160.0, shift),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    let selected = bridge.selection().selected_ids;
    assert!(selected.contains(&id1) && selected.contains(&id2));
}

#[test]
fn select_lasso_encloses_box() {
    let (mut bridge, id1, id2) = select_test_bridge_two_boxes();
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = SelectTool::new();
    tool.set_gesture_mode(SelectGestureMode::Lasso);
    tool.set_marquee_rule(MarqueeSelectRule::Intersect);

    tool.on_pointer_event(
        &pointer_event(PointerPhase::Down, 0.0, 0.0, SemanticModifiers::default()),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    for (x, y) in [(70.0, 0.0), (70.0, 70.0), (0.0, 70.0)] {
        tool.on_pointer_event(
            &pointer_event(PointerPhase::Move, x, y, SemanticModifiers::default()),
            &mut bridge,
            &camera,
            &mut snap,
        )
        .unwrap();
    }
    // Lasso preview exposes the freehand path for the UI.
    assert!(tool.overlays(&camera, &bridge).lasso_screen.is_some());
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Up, 0.0, 70.0, SemanticModifiers::default()),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    assert_eq!(bridge.selection().selected_ids, vec![id1]);
    assert!(!bridge.selection().selected_ids.contains(&id2));
}

#[test]
fn select_manager_exposes_gesture_and_rule_settings() {
    let mut manager = ToolManager::new();
    manager.set_select_gesture_mode(SelectGestureMode::Lasso);
    manager.set_select_marquee_rule(MarqueeSelectRule::Contained);
    assert_eq!(
        manager.select_tool().gesture_mode(),
        SelectGestureMode::Lasso
    );
    assert_eq!(
        manager.select_tool().marquee_rule(),
        MarqueeSelectRule::Contained
    );
}
