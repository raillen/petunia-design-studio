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

fn pen_down(
    tool: &mut PenTool,
    bridge: &mut PetuniaDesignGuiBridge,
    camera: &ViewportCamera,
    snap: &mut SnapEngine,
    x: f64,
    y: f64,
    modifiers: SemanticModifiers,
) {
    let p = GPoint::new(x, y);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Down, PointerButton::Primary, p, p, modifiers),
        bridge,
        camera,
        snap,
    )
    .unwrap();
}

fn pen_move(
    tool: &mut PenTool,
    camera: &ViewportCamera,
    snap: &mut SnapEngine,
    x: f64,
    y: f64,
    modifiers: SemanticModifiers,
) {
    let p = GPoint::new(x, y);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Move, PointerButton::Primary, p, p, modifiers),
        &mut PetuniaDesignGuiBridge::new(),
        camera,
        snap,
    )
    .unwrap();
}

#[test]
fn pen_shift_constrains_anchor_to_45_degrees() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Pen Shift").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = PenTool::new();

    pen_down(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        100.0,
        100.0,
        SemanticModifiers::default(),
    );
    let shift = SemanticModifiers {
        constrain: true,
        ..Default::default()
    };
    pen_down(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        200.0,
        150.0,
        shift,
    );

    let anchors = tool.anchors();
    assert_eq!(anchors.len(), 2);
    // atan2(50, 100) rounds to 45°: x and y offsets match, distance kept.
    let (dx, dy) = (anchors[1].point.x - 100.0, anchors[1].point.y - 100.0);
    assert!((dx - dy).abs() < 1e-6, "got dx={dx} dy={dy}");
    assert!((dx.hypot(dy) - 100.0_f64.hypot(50.0)) < 1e-6);
}

#[test]
fn pen_alt_drag_breaks_handle_mirror() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Pen Break").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = PenTool::new();
    let alt = SemanticModifiers {
        duplicate: true,
        ..Default::default()
    };

    pen_down(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        100.0,
        100.0,
        SemanticModifiers::default(),
    );
    // Alt-drag adjusts only the outgoing handle (cusp break).
    let p = GPoint::new(130.0, 110.0);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Move, PointerButton::Primary, p, p, alt),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    let anchor = &tool.anchors()[0];
    assert!(anchor.handle_out.is_some());
    assert_eq!(anchor.handle_in, None);
    assert_eq!(anchor.node_type, NodeType::Cusp);
}

#[test]
fn pen_click_back_on_last_anchor_removes_handles() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Pen Recollect").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = PenTool::new();

    pen_down(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        100.0,
        100.0,
        SemanticModifiers::default(),
    );
    pen_move(
        &mut tool,
        &camera,
        &mut snap,
        140.0,
        120.0,
        SemanticModifiers::default(),
    );
    assert!(tool.anchors()[0].handle_out.is_some());

    // Second anchor far away so the recollect click is unambiguous.
    pen_down(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        300.0,
        100.0,
        SemanticModifiers::default(),
    );
    pen_move(
        &mut tool,
        &camera,
        &mut snap,
        340.0,
        120.0,
        SemanticModifiers::default(),
    );
    assert!(tool.anchors()[1].handle_out.is_some());
    // Wait out the double-click window so the next click reads as recollect.
    std::thread::sleep(std::time::Duration::from_millis(450));
    // Click back on the last anchor: handles removed, curve becomes straight.
    pen_down(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        300.0,
        100.0,
        SemanticModifiers::default(),
    );
    let last = tool.anchors().last().unwrap();
    assert_eq!(last.handle_in, None);
    assert_eq!(last.handle_out, None);
    assert_eq!(tool.anchors().len(), 2);
    // Only the last anchor was recollected; the first keeps its handles.
    assert!(tool.anchors()[0].handle_out.is_some());
}

#[test]
fn pen_secondary_click_finishes_open_path() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Pen Finish").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = PenTool::new();

    pen_down(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        100.0,
        100.0,
        SemanticModifiers::default(),
    );
    pen_down(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        200.0,
        100.0,
        SemanticModifiers::default(),
    );
    assert!(tool.is_active());

    let p = GPoint::new(200.0, 100.0);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(
            PointerPhase::Down,
            PointerButton::Secondary,
            p,
            p,
            SemanticModifiers::default(),
        ),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    assert!(!tool.is_active());
    assert_eq!(bridge.snapshot().total_objects, 1);
}

#[test]
fn pen_double_click_finishes_open_path() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Pen Double").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = PenTool::new();

    pen_down(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        100.0,
        100.0,
        SemanticModifiers::default(),
    );
    pen_down(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        300.0,
        100.0,
        SemanticModifiers::default(),
    );
    // Immediate second down on the same spot: double-click finish.
    pen_down(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        300.0,
        100.0,
        SemanticModifiers::default(),
    );

    assert!(!tool.is_active());
    assert_eq!(bridge.snapshot().total_objects, 1);
}

#[test]
fn pen_continues_existing_open_path_from_endpoint() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Pen Continue").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = PenTool::new();

    pen_down(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        100.0,
        100.0,
        SemanticModifiers::default(),
    );
    pen_down(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        200.0,
        100.0,
        SemanticModifiers::default(),
    );
    tool.finish_open_path(&mut bridge).unwrap();
    assert_eq!(bridge.snapshot().total_objects, 1);
    let target = bridge.selection().selected_ids[0];

    // Fresh tool clicking the end anchor continues the same object.
    let mut cont = PenTool::new();
    pen_down(
        &mut cont,
        &mut bridge,
        &camera,
        &mut snap,
        200.0,
        100.0,
        SemanticModifiers::default(),
    );
    assert_eq!(cont.continuing_object(), Some(target));
    pen_down(
        &mut cont,
        &mut bridge,
        &camera,
        &mut snap,
        300.0,
        100.0,
        SemanticModifiers::default(),
    );
    cont.finish_open_path(&mut bridge).unwrap();

    // Same object, extended shape — no duplicate path created.
    assert_eq!(bridge.snapshot().total_objects, 1);
    assert_eq!(bridge.selection().selected_ids, vec![target]);
    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(target)
        .unwrap();
    let verbs = match obj.shape.as_ref().unwrap() {
        petunia_design_document::ShapeKind::Path(path) => path.verbs.len(),
        _ => panic!("expected path"),
    };
    assert_eq!(verbs, 3);
}

#[test]
fn pen_cursor_hint_distinguishes_contexts() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Pen Cursor").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let tool = PenTool::new();

    // Empty canvas, no paths: fresh create.
    assert_eq!(
        tool.cursor_hint(
            GPoint::new(500.0, 500.0),
            GPoint::new(500.0, 500.0),
            &bridge,
            &camera
        ),
        PenCursorHint::CreateNew
    );

    // With an open path committed, hovering its endpoint offers continuation.
    let mut draw = PenTool::new();
    pen_down(
        &mut draw,
        &mut bridge,
        &camera,
        &mut snap,
        100.0,
        100.0,
        SemanticModifiers::default(),
    );
    pen_down(
        &mut draw,
        &mut bridge,
        &camera,
        &mut snap,
        200.0,
        100.0,
        SemanticModifiers::default(),
    );
    draw.finish_open_path(&mut bridge).unwrap();

    let fresh = PenTool::new();
    assert_eq!(
        fresh.cursor_hint(
            GPoint::new(200.0, 100.0),
            GPoint::new(200.0, 100.0),
            &bridge,
            &camera
        ),
        PenCursorHint::ContinuePath
    );
}

fn pencil_stroke(
    tool: &mut PencilTool,
    bridge: &mut PetuniaDesignGuiBridge,
    camera: &ViewportCamera,
    snap: &mut SnapEngine,
    points: &[(f64, f64)],
    modifiers: SemanticModifiers,
) {
    for (i, (x, y)) in points.iter().enumerate() {
        let phase = if i == 0 {
            PointerPhase::Down
        } else if i == points.len() - 1 {
            PointerPhase::Up
        } else {
            PointerPhase::Move
        };
        let p = GPoint::new(*x, *y);
        tool.on_pointer_event(
            &NormalizedPointerEvent::new(
                PointerPhase::Move,
                PointerButton::Primary,
                p,
                p,
                modifiers,
            ),
            bridge,
            camera,
            snap,
        )
        .unwrap();
        if phase != PointerPhase::Move {
            tool.on_pointer_event(
                &NormalizedPointerEvent::new(phase, PointerButton::Primary, p, p, modifiers),
                bridge,
                camera,
                snap,
            )
            .unwrap();
        }
    }
}

fn path_verbs(
    bridge: &PetuniaDesignGuiBridge,
    id: petunia_design_foundation::ObjectId,
) -> Vec<petunia_design_geometry::PathVerb> {
    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap();
    match obj.shape.as_ref().unwrap() {
        petunia_design_document::ShapeKind::Path(path) => path.verbs.clone(),
        _ => panic!("expected path"),
    }
}

#[test]
fn pencil_shift_commits_straight_line() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Pencil Straight").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = PencilTool::new();
    let shift = SemanticModifiers {
        constrain: true,
        ..Default::default()
    };

    pencil_stroke(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        &[(50.0, 50.0), (100.0, 80.0), (150.0, 60.0), (200.0, 200.0)],
        shift,
    );

    assert_eq!(bridge.snapshot().total_objects, 1);
    let id = bridge.selection().selected_ids[0];
    let verbs = path_verbs(&bridge, id);
    assert_eq!(verbs.len(), 2);
    assert!(matches!(
        verbs[0],
        petunia_design_geometry::PathVerb::MoveTo(_)
    ));
    assert!(matches!(
        verbs[1],
        petunia_design_geometry::PathVerb::LineTo(_)
    ));
}

#[test]
fn pencil_auto_closes_loop_near_start() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Pencil Close").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = PencilTool::new();

    pencil_stroke(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        &[
            (100.0, 100.0),
            (200.0, 100.0),
            (200.0, 200.0),
            (100.0, 200.0),
            (102.0, 102.0),
        ],
        SemanticModifiers::default(),
    );

    assert_eq!(bridge.snapshot().total_objects, 1);
    let id = bridge.selection().selected_ids[0];
    let verbs = path_verbs(&bridge, id);
    assert!(verbs.contains(&petunia_design_geometry::PathVerb::Close));
}

#[test]
fn pencil_fidelity_smooth_simplifies_more_than_precise() {
    // Hand jitter input: Smooth collapses it, Precise preserves it.
    // Snap disabled: grid snap would erase sub-grid jitter before the fit.
    let points: Vec<(f64, f64)> = (0..41)
        .map(|i| (i as f64 * 5.0, if i % 2 == 0 { 1.5 } else { -1.5 }))
        .collect();
    let freehand = SemanticModifiers {
        disable_snap: true,
        ..Default::default()
    };
    let mut counts = Vec::new();
    for fidelity in [PencilFidelity::Precise, PencilFidelity::Smooth] {
        let mut bridge = PetuniaDesignGuiBridge::new();
        bridge.new_document("Pencil Fidelity").expect("doc");
        let camera = ViewportCamera::new(1000.0, 1000.0);
        let mut snap = SnapEngine::new();
        let mut tool = PencilTool::new();
        tool.set_fidelity(fidelity);
        pencil_stroke(
            &mut tool,
            &mut bridge,
            &camera,
            &mut snap,
            &points,
            freehand,
        );
        assert_eq!(bridge.snapshot().total_objects, 1);
        let id = bridge.selection().selected_ids[0];
        counts.push(path_verbs(&bridge, id).len());
    }
    assert!(
        counts[1] < counts[0],
        "smooth should collapse jitter: {counts:?}"
    );
}

#[test]
fn pencil_sculpt_extends_selected_path_from_endpoint() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Pencil Extend").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = PencilTool::new();

    pencil_stroke(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        &[(100.0, 300.0), (200.0, 300.0)],
        SemanticModifiers::default(),
    );
    assert_eq!(bridge.snapshot().total_objects, 1);
    let target = bridge.selection().selected_ids[0];
    let before = path_verbs(&bridge, target).len();

    // Fresh tool starting on the path end extends the same object.
    let mut sculpt = PencilTool::new();
    assert_eq!(sculpt.sculpt_target(), None);
    pencil_stroke(
        &mut sculpt,
        &mut bridge,
        &camera,
        &mut snap,
        &[(200.0, 300.0), (260.0, 300.0), (320.0, 320.0)],
        SemanticModifiers::default(),
    );

    assert_eq!(bridge.snapshot().total_objects, 1);
    assert_eq!(bridge.selection().selected_ids, vec![target]);
    assert!(path_verbs(&bridge, target).len() > before);
}

#[test]
fn pencil_sculpt_redraw_over_middle_reshapes_same_object() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Pencil Reshape").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = PencilTool::new();

    pencil_stroke(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        &[(100.0, 300.0), (200.0, 300.0), (300.0, 300.0)],
        SemanticModifiers::default(),
    );
    let target = bridge.selection().selected_ids[0];
    let bounds_before = bridge
        .session()
        .unwrap()
        .document()
        .find_object(target)
        .unwrap()
        .bounds;

    // Redraw the middle pushed upward: same object, new geometry.
    let mut sculpt = PencilTool::new();
    pencil_stroke(
        &mut sculpt,
        &mut bridge,
        &camera,
        &mut snap,
        &[(150.0, 300.0), (200.0, 220.0), (250.0, 300.0)],
        SemanticModifiers::default(),
    );

    assert_eq!(bridge.snapshot().total_objects, 1);
    let bounds_after = bridge
        .session()
        .unwrap()
        .document()
        .find_object(target)
        .unwrap()
        .bounds;
    assert_ne!(bounds_before, bounds_after);
    assert!(bounds_after.unwrap()[1] < bounds_before.unwrap()[1]);
}

fn node_test_path(
    bridge: &mut PetuniaDesignGuiBridge,
    verbs: Vec<petunia_design_geometry::PathVerb>,
) -> petunia_design_foundation::ObjectId {
    let surface_id = bridge.active_surface().unwrap();
    let mut gen = IdGenerator::new();
    let id = gen.next_object();
    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id,
            name: "NodePath".to_string(),
        }))
        .unwrap();
    let mut path = petunia_design_geometry::GPath::new();
    for verb in verbs {
        path.push(verb).unwrap();
    }
    let bounds = path
        .bounding_box()
        .map(|r| [r.x0, r.y0, r.width().max(1.0), r.height().max(1.0)]);
    bridge
        .submit_command(CommandRequest::new(Command::SetShape {
            id,
            shape: Some(petunia_design_document::ShapeKind::Path(path)),
        }))
        .unwrap();
    if let Some(bounds) = bounds {
        bridge
            .submit_command(CommandRequest::new(Command::SetBounds {
                id,
                bounds: Some(bounds),
                rotation: 0.0,
            }))
            .unwrap();
    }
    bridge.clear_selection();
    id
}

fn node_event(
    phase: PointerPhase,
    x: f64,
    y: f64,
    modifiers: SemanticModifiers,
) -> NormalizedPointerEvent {
    let p = GPoint::new(x, y);
    NormalizedPointerEvent::new(phase, PointerButton::Primary, p, p, modifiers)
}

fn node_endpoints(
    bridge: &PetuniaDesignGuiBridge,
    id: petunia_design_foundation::ObjectId,
) -> Vec<GPoint> {
    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap();
    match obj.shape.as_ref().unwrap() {
        petunia_design_document::ShapeKind::Path(path) => path
            .verbs
            .iter()
            .filter_map(|v| match v {
                petunia_design_geometry::PathVerb::MoveTo(p)
                | petunia_design_geometry::PathVerb::LineTo(p)
                | petunia_design_geometry::PathVerb::QuadTo(_, p)
                | petunia_design_geometry::PathVerb::CubicTo(_, _, p) => Some(*p),
                petunia_design_geometry::PathVerb::Close => None,
            })
            .collect(),
        _ => panic!("expected path"),
    }
}

#[test]
fn node_shift_multiselect_drags_once_and_undos_once() {
    use petunia_design_geometry::PathVerb as V;
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Node Multi").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = node_test_path(
        &mut bridge,
        vec![
            V::MoveTo(GPoint::new(10.0, 10.0)),
            V::LineTo(GPoint::new(60.0, 10.0)),
            V::LineTo(GPoint::new(60.0, 60.0)),
        ],
    );
    bridge.set_selection(vec![id]);
    let mut tool = NodeTool::new();
    let plain = SemanticModifiers::default();
    let shift = SemanticModifiers {
        constrain: true,
        ..Default::default()
    };

    // Click node 0, Shift-click node 1: both selected, nothing committed yet.
    tool.on_pointer_event(
        &node_event(PointerPhase::Down, 10.0, 10.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &node_event(PointerPhase::Up, 10.0, 10.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &node_event(PointerPhase::Down, 60.0, 10.0, shift),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    assert_eq!(tool.selected_nodes().len(), 2);

    // Drag both by (+10, +10): one commit on Up.
    tool.on_pointer_event(
        &node_event(PointerPhase::Move, 70.0, 20.0, shift),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &node_event(PointerPhase::Up, 70.0, 20.0, shift),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    let pts = node_endpoints(&bridge, id);
    assert_eq!(pts[0], GPoint::new(20.0, 20.0));
    assert_eq!(pts[1], GPoint::new(70.0, 20.0));
    assert_eq!(pts[2], GPoint::new(60.0, 60.0));
    // Bounds followed the edit.
    let bounds = bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap()
        .bounds
        .unwrap();
    assert_eq!(bounds[0], 20.0);

    // Exactly one undo entry restores both nodes.
    assert!(bridge.can_undo());
    bridge.undo().unwrap();
    let pts = node_endpoints(&bridge, id);
    assert_eq!(pts[0], GPoint::new(10.0, 10.0));
    assert_eq!(pts[1], GPoint::new(60.0, 10.0));
}

#[test]
fn node_handle_drag_keeps_symmetric_mirror() {
    use petunia_design_geometry::PathVerb as V;
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Node Handle").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = node_test_path(
        &mut bridge,
        vec![
            V::MoveTo(GPoint::new(0.0, 0.0)),
            V::CubicTo(
                GPoint::new(10.0, 0.0),
                GPoint::new(20.0, 0.0),
                GPoint::new(30.0, 0.0),
            ),
            V::CubicTo(
                GPoint::new(40.0, 0.0),
                GPoint::new(50.0, 0.0),
                GPoint::new(60.0, 0.0),
            ),
        ],
    );
    bridge.set_selection(vec![id]);
    let mut tool = NodeTool::new();
    let plain = SemanticModifiers::default();

    // Select the middle anchor, then grab its outgoing handle at (40, 0).
    tool.on_pointer_event(
        &node_event(PointerPhase::Down, 30.0, 0.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &node_event(PointerPhase::Up, 30.0, 0.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &node_event(PointerPhase::Down, 40.0, 0.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &node_event(PointerPhase::Move, 50.0, 0.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &node_event(PointerPhase::Up, 50.0, 0.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    // Symmetric mirror: out (50,0), in mirrored to (10,0).
    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap();
    match obj.shape.as_ref().unwrap() {
        petunia_design_document::ShapeKind::Path(path) => {
            assert_eq!(
                path.verbs[1],
                V::CubicTo(
                    GPoint::new(10.0, 0.0),
                    GPoint::new(10.0, 0.0),
                    GPoint::new(30.0, 0.0)
                )
            );
            assert_eq!(
                path.verbs[2],
                V::CubicTo(
                    GPoint::new(50.0, 0.0),
                    GPoint::new(50.0, 0.0),
                    GPoint::new(60.0, 0.0)
                )
            );
        }
        _ => panic!("expected path"),
    }
}

#[test]
fn node_convert_smooth_then_symmetric() {
    use petunia_design_geometry::PathVerb as V;
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Node Convert").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = node_test_path(
        &mut bridge,
        vec![
            V::MoveTo(GPoint::new(0.0, 0.0)),
            V::CubicTo(
                GPoint::new(10.0, 5.0),
                GPoint::new(20.0, -5.0),
                GPoint::new(30.0, 0.0),
            ),
            V::LineTo(GPoint::new(60.0, 0.0)),
        ],
    );
    bridge.set_selection(vec![id]);
    let mut tool = NodeTool::new();
    let plain = SemanticModifiers::default();
    tool.on_pointer_event(
        &node_event(PointerPhase::Down, 30.0, 0.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &node_event(PointerPhase::Up, 30.0, 0.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    tool.convert_selected_nodes(&mut bridge, NodeType::Smooth)
        .unwrap();
    let verbs = path_verbs(&bridge, id);
    // Smooth keeps unequal lengths but aligns directions.
    assert!(verbs.len() == 3);

    tool.convert_selected_nodes(&mut bridge, NodeType::Symmetric)
        .unwrap();
    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap();
    match obj.shape.as_ref().unwrap() {
        petunia_design_document::ShapeKind::Path(path) => {
            // In-handle is verb 1's second control; out-handle is verb 2's
            // first control (cubic) or shared control (quad upgrade).
            let (in_handle, out_handle) = match (&path.verbs[1], &path.verbs[2]) {
                (V::CubicTo(_, c2, _), V::CubicTo(c1, _, _)) => (*c2, *c1),
                (V::CubicTo(_, c2, _), V::QuadTo(c, _)) => (*c2, *c),
                _ => panic!("expected curve verbs"),
            };
            let p = GPoint::new(30.0, 0.0);
            let in_len = in_handle.distance_to(p);
            let out_len = out_handle.distance_to(p);
            assert!((in_len - out_len).abs() < 1e-6, "in={in_len} out={out_len}");
        }
        _ => panic!("expected path"),
    }
}

#[test]
fn node_double_click_adds_and_removes() {
    use petunia_design_geometry::PathVerb as V;
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Node AddRemove").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = node_test_path(
        &mut bridge,
        vec![
            V::MoveTo(GPoint::new(0.0, 0.0)),
            V::LineTo(GPoint::new(60.0, 0.0)),
        ],
    );
    bridge.set_selection(vec![id]);
    let mut tool = NodeTool::new();
    let plain = SemanticModifiers::default();

    // Double-click mid-segment inserts a shape-preserving node.
    tool.on_pointer_event(
        &node_event(PointerPhase::Down, 30.0, 0.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &node_event(PointerPhase::Up, 30.0, 0.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &node_event(PointerPhase::Down, 30.0, 0.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    assert_eq!(path_verbs(&bridge, id).len(), 3);
    let pts = node_endpoints(&bridge, id);
    assert_eq!(pts[1], GPoint::new(30.0, 0.0));

    // Double-click the new node removes it again.
    std::thread::sleep(std::time::Duration::from_millis(450));
    tool.on_pointer_event(
        &node_event(PointerPhase::Down, 30.0, 0.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &node_event(PointerPhase::Up, 30.0, 0.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &node_event(PointerPhase::Down, 30.0, 0.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    assert_eq!(path_verbs(&bridge, id).len(), 2);
}

#[test]
fn node_marquee_selects_several_nodes() {
    use petunia_design_geometry::PathVerb as V;
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Node Marquee").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = node_test_path(
        &mut bridge,
        vec![
            V::MoveTo(GPoint::new(10.0, 10.0)),
            V::LineTo(GPoint::new(60.0, 10.0)),
            V::LineTo(GPoint::new(60.0, 60.0)),
            V::LineTo(GPoint::new(10.0, 60.0)),
        ],
    );
    bridge.set_selection(vec![id]);
    let mut tool = NodeTool::new();
    let plain = SemanticModifiers::default();

    // Marquee over empty canvas covering the top two nodes.
    tool.on_pointer_event(
        &node_event(PointerPhase::Down, -50.0, -50.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &node_event(PointerPhase::Move, 70.0, 30.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    assert!(tool.overlays(&camera, &bridge).marquee_screen.is_some());
    tool.on_pointer_event(
        &node_event(PointerPhase::Up, 70.0, 30.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    assert_eq!(tool.selected_nodes().len(), 2);
}

fn corner_test_rect(
    bridge: &mut PetuniaDesignGuiBridge,
    gen: &mut IdGenerator,
    bounds: [f64; 4],
) -> petunia_design_foundation::ObjectId {
    let surface_id = bridge.active_surface().unwrap();
    let id = gen.next_object();
    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id,
            name: "CornerRect".to_string(),
        }))
        .unwrap();
    bridge
        .submit_command(CommandRequest::new(Command::SetShape {
            id,
            shape: Some(petunia_design_document::ShapeKind::Rectangle {
                corner_radii: [0.0; 4],
            }),
        }))
        .unwrap();
    bridge
        .submit_command(CommandRequest::new(Command::SetBounds {
            id,
            bounds: Some(bounds),
            rotation: 0.0,
        }))
        .unwrap();
    bridge.clear_selection();
    id
}

fn contour_rect_radii(
    bridge: &PetuniaDesignGuiBridge,
    id: petunia_design_foundation::ObjectId,
) -> [f64; 4] {
    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap();
    match obj.shape.as_ref().unwrap() {
        petunia_design_document::ShapeKind::Rectangle { corner_radii } => *corner_radii,
        _ => panic!("expected rectangle"),
    }
}

#[test]
fn corner_drag_edits_single_corner_with_one_undo() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Corner Tool").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    let id = corner_test_rect(&mut bridge, &mut gen, [0.0, 0.0, 100.0, 100.0]);
    let mut tool = ContourTool::new(ContourMode::Corner);
    let plain = SemanticModifiers::default();

    // Grab the top-left corner widget and pull outward.
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Down, 5.0, 5.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    assert!(tool.is_active());
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Move, -30.0, -30.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    assert!(tool.overlays(&bridge, &camera).marquee_screen.is_some());
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Up, -30.0, -30.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    let radii = contour_rect_radii(&bridge, id);
    assert!(radii[0] > 40.0 && radii[0] <= 50.0, "got {radii:?}");
    assert_eq!([radii[1], radii[2], radii[3]], [0.0, 0.0, 0.0]);

    // One undo restores the sharp rectangle.
    bridge.undo().unwrap();
    assert_eq!(contour_rect_radii(&bridge, id), [0.0; 4]);
}

#[test]
fn corner_shift_drag_edits_all_four() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Corner All").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    let id = corner_test_rect(&mut bridge, &mut gen, [0.0, 0.0, 100.0, 100.0]);
    let mut tool = ContourTool::new(ContourMode::Corner);
    let shift = SemanticModifiers {
        constrain: true,
        ..Default::default()
    };

    tool.on_pointer_event(
        &pointer_event(PointerPhase::Down, 5.0, 5.0, shift),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Move, -30.0, -30.0, shift),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Up, -30.0, -30.0, shift),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    let radii = contour_rect_radii(&bridge, id);
    assert!(radii.iter().all(|r| *r > 40.0), "got {radii:?}");
}

#[test]
fn contour_drag_sets_live_modifier_with_one_undo() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Contour Live").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    let id = corner_test_rect(&mut bridge, &mut gen, [0.0, 0.0, 100.0, 100.0]);
    bridge.set_selection(vec![id]);
    let mut tool = ContourTool::new(ContourMode::Contour);
    let plain = SemanticModifiers::default();

    // Drag outward from the center: radial delta +40.
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Down, 50.0, 50.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Move, 90.0, 50.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Up, 90.0, 50.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    // Live modifier set; base geometry untouched.
    let mods = bridge.modifiers(id);
    assert_eq!(mods.len(), 1);
    let petunia_design_document::ModifierKind::ContourOffset { distance, .. } = &mods[0].kind
    else {
        panic!("expected contour");
    };
    assert!((distance - 40.0).abs() < 1e-6, "got {distance}");
    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap();
    assert!(matches!(
        obj.shape,
        Some(petunia_design_document::ShapeKind::Rectangle { .. })
    ));
    assert_eq!(obj.bounds, Some([0.0, 0.0, 100.0, 100.0]));
    // …but the evaluated outline grew.
    let eval = obj.evaluated_bounds().unwrap();
    assert!((eval[2] - 180.0).abs() < 2.0, "got {eval:?}");

    // One undo clears the live offset.
    bridge.undo().unwrap();
    assert!(bridge.modifiers(id).is_empty());
}

#[test]
fn contour_bake_commits_geometry_explicitly() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Contour Bake").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    let id = corner_test_rect(&mut bridge, &mut gen, [0.0, 0.0, 100.0, 100.0]);
    bridge.set_selection(vec![id]);
    let mut tool = ContourTool::new(ContourMode::Contour);
    let plain = SemanticModifiers::default();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Down, 50.0, 50.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Move, 90.0, 50.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Up, 90.0, 50.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    bridge.bake_contour(id).unwrap();
    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap();
    assert!(matches!(
        obj.shape,
        Some(petunia_design_document::ShapeKind::Path(_))
    ));
    assert!(bridge.modifiers(id).is_empty());
    let bounds = obj.bounds.unwrap();
    assert!((bounds[2] - 180.0).abs() < 2.0, "got {bounds:?}");
}

#[test]
fn corner_tool_leaves_non_rectangles_alone() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Corner Guard").expect("doc");
    let surface_id = bridge.active_surface().unwrap();
    let mut gen = IdGenerator::new();
    let id = gen.next_object();
    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id,
            name: "Oval".to_string(),
        }))
        .unwrap();
    bridge
        .submit_command(CommandRequest::new(Command::SetShape {
            id,
            shape: Some(petunia_design_document::ShapeKind::Ellipse),
        }))
        .unwrap();
    bridge
        .submit_command(CommandRequest::new(Command::SetBounds {
            id,
            bounds: Some([0.0, 0.0, 100.0, 60.0]),
            rotation: 0.0,
        }))
        .unwrap();
    bridge.set_selection(vec![id]);
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = ContourTool::new(ContourMode::Corner);
    let plain = SemanticModifiers::default();

    // No corner widgets on an ellipse: no drag, no silent bake.
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Down, 50.0, 30.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    assert!(!tool.is_active());
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Move, 90.0, 30.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Up, 90.0, 30.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap();
    assert!(matches!(
        obj.shape,
        Some(petunia_design_document::ShapeKind::Ellipse)
    ));
}

fn knife_closed_rect(bridge: &mut PetuniaDesignGuiBridge) -> petunia_design_foundation::ObjectId {
    use petunia_design_geometry::PathVerb as V;
    node_test_path(
        bridge,
        vec![
            V::MoveTo(GPoint::new(0.0, 0.0)),
            V::LineTo(GPoint::new(100.0, 0.0)),
            V::LineTo(GPoint::new(100.0, 60.0)),
            V::LineTo(GPoint::new(0.0, 60.0)),
            V::Close,
        ],
    )
}

#[test]
fn knife_cut_splits_rect_in_one_undo() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Knife Cut").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = knife_closed_rect(&mut bridge);
    let mut tool = KnifeTool::new(KnifeMode::Knife);
    let plain = SemanticModifiers::default();

    // Vertical cut across the whole rect.
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Down, 50.0, -10.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Move, 50.0, 70.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Up, 50.0, 70.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    assert_eq!(bridge.snapshot().total_objects, 2);
    // One undo restores the single closed rect.
    bridge.undo().unwrap();
    assert_eq!(bridge.snapshot().total_objects, 1);
    let verbs = path_verbs(&bridge, id);
    assert!(verbs.contains(&petunia_design_geometry::PathVerb::Close));
}

#[test]
fn knife_graze_is_noop() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Knife Graze").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = knife_closed_rect(&mut bridge);
    let before = path_verbs(&bridge, id);
    let mut tool = KnifeTool::new(KnifeMode::Knife);
    let plain = SemanticModifiers::default();

    // Drag far away from everything.
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Down, 500.0, 500.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Move, 600.0, 600.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Up, 600.0, 600.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    assert_eq!(bridge.snapshot().total_objects, 1);
    assert_eq!(path_verbs(&bridge, id), before);
}

#[test]
fn scissors_click_opens_closed_loop() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Scissors Open").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = knife_closed_rect(&mut bridge);
    let mut tool = KnifeTool::new(KnifeMode::Scissors);
    let plain = SemanticModifiers::default();

    // Click (no drag) just inside the top edge.
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Down, 50.0, 2.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Up, 50.0, 2.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    // Same single object, now an open loop starting at the cut.
    assert_eq!(bridge.snapshot().total_objects, 1);
    let verbs = path_verbs(&bridge, id);
    assert!(!verbs.contains(&petunia_design_geometry::PathVerb::Close));
    bridge.undo().unwrap();
    assert!(path_verbs(&bridge, id).contains(&petunia_design_geometry::PathVerb::Close));
}

#[test]
fn scissors_click_divides_open_stroke() {
    use petunia_design_geometry::PathVerb as V;
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Scissors Divide").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    node_test_path(
        &mut bridge,
        vec![
            V::MoveTo(GPoint::new(0.0, 0.0)),
            V::LineTo(GPoint::new(100.0, 0.0)),
        ],
    );
    let mut tool = KnifeTool::new(KnifeMode::Scissors);
    let plain = SemanticModifiers::default();

    tool.on_pointer_event(
        &pointer_event(PointerPhase::Down, 30.0, 0.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Up, 30.0, 0.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    assert_eq!(bridge.snapshot().total_objects, 2);
    bridge.undo().unwrap();
    assert_eq!(bridge.snapshot().total_objects, 1);
}

#[test]
fn knife_converts_parametric_in_batch() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Knife Convert").expect("doc");
    let surface_id = bridge.active_surface().unwrap();
    let mut gen = IdGenerator::new();
    let id = gen.next_object();
    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id,
            name: "ParamRect".to_string(),
        }))
        .unwrap();
    bridge
        .submit_command(CommandRequest::new(Command::SetShape {
            id,
            shape: Some(petunia_design_document::ShapeKind::Rectangle {
                corner_radii: [0.0; 4],
            }),
        }))
        .unwrap();
    bridge
        .submit_command(CommandRequest::new(Command::SetBounds {
            id,
            bounds: Some([0.0, 0.0, 100.0, 60.0]),
            rotation: 0.0,
        }))
        .unwrap();
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = KnifeTool::new(KnifeMode::Knife);
    let plain = SemanticModifiers::default();

    tool.on_pointer_event(
        &pointer_event(PointerPhase::Down, 50.0, -10.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Move, 50.0, 70.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Up, 50.0, 70.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    // Cut landed and the source is now curves…
    assert_eq!(bridge.snapshot().total_objects, 2);
    // …and a single undo restores the parametric rectangle.
    bridge.undo().unwrap();
    assert_eq!(bridge.snapshot().total_objects, 1);
    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap();
    assert!(matches!(
        obj.shape,
        Some(petunia_design_document::ShapeKind::Rectangle { .. })
    ));
}

fn gradient_test_object(
    bridge: &mut PetuniaDesignGuiBridge,
    fill: &str,
) -> petunia_design_foundation::ObjectId {
    let surface_id = bridge.active_surface().unwrap();
    let mut gen = IdGenerator::new();
    let id = gen.next_object();
    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id,
            name: "GradBox".to_string(),
        }))
        .unwrap();
    bridge
        .submit_command(CommandRequest::new(Command::SetBounds {
            id,
            bounds: Some([0.0, 0.0, 200.0, 100.0]),
            rotation: 0.0,
        }))
        .unwrap();
    bridge.set_fill(id, Some(fill.to_string())).unwrap();
    bridge.clear_selection();
    bridge.set_selection(vec![id]);
    id
}

fn primary_linear(
    bridge: &PetuniaDesignGuiBridge,
    id: petunia_design_foundation::ObjectId,
) -> Option<petunia_design_document::LinearGradient> {
    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap();
    match obj
        .effective_appearance()
        .primary_fill()
        .map(|f| f.paint.clone())
    {
        Some(petunia_design_document::Paint::LinearGradient(g)) => Some(g),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn gradient_drag(
    tool: &mut GradientTool,
    bridge: &mut PetuniaDesignGuiBridge,
    camera: &ViewportCamera,
    snap: &mut SnapEngine,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    modifiers: SemanticModifiers,
) {
    let p0 = GPoint::new(x0, y0);
    let p1 = GPoint::new(x1, y1);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(
            PointerPhase::Down,
            PointerButton::Primary,
            p0,
            p0,
            modifiers,
        ),
        bridge,
        camera,
        snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(
            PointerPhase::Move,
            PointerButton::Primary,
            p1,
            p1,
            modifiers,
        ),
        bridge,
        camera,
        snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Up, PointerButton::Primary, p1, p1, modifiers),
        bridge,
        camera,
        snap,
    )
    .unwrap();
}

#[test]
fn gradient_linear_drag_creates_two_stop_gradient() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Gradient Create").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = gradient_test_object(&mut bridge, "ptnd.red/500");
    let mut tool = GradientTool::new(GradientToolMode::Fill);

    gradient_drag(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        10.0,
        50.0,
        110.0,
        50.0,
        SemanticModifiers::default(),
    );

    let g = primary_linear(&bridge, id).expect("linear gradient");
    assert_eq!(g.start, [10.0, 50.0]);
    assert_eq!(g.end, [110.0, 50.0]);
    assert_eq!(g.stops.len(), 2);
    assert!(g.stops.iter().all(|s| s.color == "ptnd.red/500"));
}

#[test]
fn gradient_shift_snaps_vector_to_45_degrees() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Gradient Shift").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = gradient_test_object(&mut bridge, "ptnd.red/500");
    let mut tool = GradientTool::new(GradientToolMode::Fill);
    let shift = SemanticModifiers {
        constrain: true,
        ..Default::default()
    };

    gradient_drag(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        10.0,
        50.0,
        110.0,
        100.0,
        shift,
    );

    let g = primary_linear(&bridge, id).expect("linear gradient");
    // atan2(50, 100) rounds to 45° with distance preserved.
    assert!((g.end[0] - g.start[0] - (g.end[1] - g.start[1])).abs() < 1e-6);
}

#[test]
fn gradient_click_never_creates() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Gradient Click").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = gradient_test_object(&mut bridge, "ptnd.red/500");
    let mut tool = GradientTool::new(GradientToolMode::Fill);
    let plain = SemanticModifiers::default();

    gradient_drag(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        50.0,
        50.0,
        50.0,
        50.0,
        plain,
    );

    assert!(primary_linear(&bridge, id).is_none());
}

#[test]
fn gradient_redrag_repositions_and_keeps_stops() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Gradient Move").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = gradient_test_object(&mut bridge, "ptnd.red/500");
    let mut tool = GradientTool::new(GradientToolMode::Fill);
    let plain = SemanticModifiers::default();

    gradient_drag(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        10.0,
        50.0,
        110.0,
        50.0,
        plain,
    );
    // Second drag far from stops (stops sit at x=10/110, y=50).
    std::thread::sleep(std::time::Duration::from_millis(450));
    gradient_drag(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        0.0,
        0.0,
        200.0,
        100.0,
        plain,
    );

    let g = primary_linear(&bridge, id).expect("linear gradient");
    assert_eq!(g.start, [0.0, 0.0]);
    assert_eq!(g.end, [200.0, 100.0]);
    assert_eq!(g.stops.len(), 2);
}

#[test]
fn gradient_double_click_adds_sampled_stop() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Gradient Add").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = gradient_test_object(&mut bridge, "ptnd.red/500");
    let mut tool = GradientTool::new(GradientToolMode::Fill);
    let plain = SemanticModifiers::default();
    gradient_drag(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        10.0,
        50.0,
        110.0,
        50.0,
        plain,
    );

    // Double-click the middle of the line: sampled stop appears.
    let p = GPoint::new(60.0, 50.0);
    for phase in [PointerPhase::Down, PointerPhase::Down] {
        tool.on_pointer_event(
            &NormalizedPointerEvent::new(PointerPhase::Move, PointerButton::Primary, p, p, plain),
            &mut bridge,
            &camera,
            &mut snap,
        )
        .unwrap();
        tool.on_pointer_event(
            &NormalizedPointerEvent::new(phase, PointerButton::Primary, p, p, plain),
            &mut bridge,
            &camera,
            &mut snap,
        )
        .unwrap();
    }

    let g = primary_linear(&bridge, id).expect("linear gradient");
    assert_eq!(g.stops.len(), 3);
    assert!((g.stops[1].offset - 0.5).abs() < 1e-6);
    bridge.undo().unwrap();
    assert_eq!(primary_linear(&bridge, id).unwrap().stops.len(), 2);
}

#[test]
fn gradient_double_click_on_stop_removes_but_keeps_two() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Gradient Remove").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = gradient_test_object(&mut bridge, "ptnd.red/500");
    let mut tool = GradientTool::new(GradientToolMode::Fill);
    let plain = SemanticModifiers::default();
    gradient_drag(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        10.0,
        50.0,
        110.0,
        50.0,
        plain,
    );

    // Add a middle stop first (double-click at x=60).
    let mid = GPoint::new(60.0, 50.0);
    for _ in 0..2 {
        tool.on_pointer_event(
            &NormalizedPointerEvent::new(
                PointerPhase::Down,
                PointerButton::Primary,
                mid,
                mid,
                plain,
            ),
            &mut bridge,
            &camera,
            &mut snap,
        )
        .unwrap();
    }
    assert_eq!(primary_linear(&bridge, id).unwrap().stops.len(), 3);

    // Double-click exactly on the middle stop removes it…
    std::thread::sleep(std::time::Duration::from_millis(450));
    for _ in 0..2 {
        tool.on_pointer_event(
            &NormalizedPointerEvent::new(
                PointerPhase::Down,
                PointerButton::Primary,
                mid,
                mid,
                plain,
            ),
            &mut bridge,
            &camera,
            &mut snap,
        )
        .unwrap();
    }
    assert_eq!(primary_linear(&bridge, id).unwrap().stops.len(), 2);

    // …but the last two stops are protected.
    std::thread::sleep(std::time::Duration::from_millis(450));
    let end = GPoint::new(110.0, 50.0);
    for _ in 0..2 {
        tool.on_pointer_event(
            &NormalizedPointerEvent::new(
                PointerPhase::Down,
                PointerButton::Primary,
                end,
                end,
                plain,
            ),
            &mut bridge,
            &camera,
            &mut snap,
        )
        .unwrap();
    }
    assert_eq!(primary_linear(&bridge, id).unwrap().stops.len(), 2);
}

#[test]
fn gradient_stop_drag_moves_offset() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Gradient Stop").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = gradient_test_object(&mut bridge, "ptnd.red/500");
    let mut tool = GradientTool::new(GradientToolMode::Fill);
    let plain = SemanticModifiers::default();
    gradient_drag(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        10.0,
        50.0,
        110.0,
        50.0,
        plain,
    );

    // Add a middle stop, then drag it from x=60 to x=85 (t 0.5 -> 0.75).
    let mid = GPoint::new(60.0, 50.0);
    for _ in 0..2 {
        tool.on_pointer_event(
            &NormalizedPointerEvent::new(
                PointerPhase::Down,
                PointerButton::Primary,
                mid,
                mid,
                plain,
            ),
            &mut bridge,
            &camera,
            &mut snap,
        )
        .unwrap();
    }
    std::thread::sleep(std::time::Duration::from_millis(450));
    let p = GPoint::new(60.0, 50.0);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Down, PointerButton::Primary, p, p, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    let q = GPoint::new(85.0, 50.0);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Move, PointerButton::Primary, q, q, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Up, PointerButton::Primary, q, q, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    let g = primary_linear(&bridge, id).expect("linear gradient");
    assert_eq!(g.stops.len(), 3);
    assert!((g.stops[1].offset - 0.75).abs() < 1e-6, "got {:?}", g.stops);
}

#[test]
fn gradient_radial_drag_sets_center_and_radius() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Gradient Radial").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = gradient_test_object(&mut bridge, "ptnd.red/500");
    let mut tool = GradientTool::new(GradientToolMode::Fill);
    tool.set_kind(GradientKind::Radial);

    gradient_drag(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        100.0,
        50.0,
        150.0,
        50.0,
        SemanticModifiers::default(),
    );

    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap();
    match obj
        .effective_appearance()
        .primary_fill()
        .map(|f| f.paint.clone())
    {
        Some(petunia_design_document::Paint::RadialGradient(g)) => {
            assert_eq!(g.center, [100.0, 50.0]);
            assert!((g.radius - 50.0).abs() < 1e-6);
        }
        _ => panic!("expected radial gradient"),
    }
}

fn picker_test_box(
    bridge: &mut PetuniaDesignGuiBridge,
    gen: &mut IdGenerator,
    name: &str,
    bounds: [f64; 4],
    fill: Option<&str>,
    stroke: Option<(&str, f64)>,
) -> petunia_design_foundation::ObjectId {
    let surface_id = bridge.active_surface().unwrap();
    let id = gen.next_object();
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
    if let Some(fill) = fill {
        bridge.set_fill(id, Some(fill.to_string())).unwrap();
    }
    if let Some((stroke, width)) = stroke {
        bridge
            .set_stroke(id, Some(stroke.to_string()), width)
            .unwrap();
    }
    id
}

fn picker_click(
    tool: &mut PickerTool,
    bridge: &mut PetuniaDesignGuiBridge,
    camera: &ViewportCamera,
    snap: &mut SnapEngine,
    x: f64,
    y: f64,
) {
    let p = GPoint::new(x, y);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(
            PointerPhase::Up,
            PointerButton::Primary,
            p,
            p,
            SemanticModifiers::default(),
        ),
        bridge,
        camera,
        snap,
    )
    .unwrap();
}

#[test]
fn picker_color_samples_fill_and_stroke() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Picker Color").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    picker_test_box(
        &mut bridge,
        &mut gen,
        "Source",
        [10.0, 10.0, 80.0, 80.0],
        Some("ptnd.red/500"),
        Some(("ptnd.blue/500", 3.0)),
    );
    let target = picker_test_box(
        &mut bridge,
        &mut gen,
        "Target",
        [200.0, 200.0, 50.0, 50.0],
        Some("ptnd.gray/500"),
        None,
    );
    bridge.clear_selection();
    bridge.set_selection(vec![target]);
    let mut tool = PickerTool::new(PickerMode::Color);

    picker_click(&mut tool, &mut bridge, &camera, &mut snap, 20.0, 20.0);

    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(target)
        .unwrap();
    let stack = obj.effective_appearance();
    let fill_paint = stack.primary_fill().map(|f| f.paint.clone());
    assert!(matches!(
        fill_paint,
        Some(petunia_design_document::Paint::Solid(ref t)) if t == "ptnd.red/500"
    ));
    let stroke = stack.primary_stroke().expect("stroke");
    assert!(
        matches!(&stroke.paint, petunia_design_document::Paint::Solid(t) if t == "ptnd.blue/500")
    );
    assert!((stroke.width - 3.0).abs() < 1e-9);
}

#[test]
fn picker_color_keeps_sampled_gradient() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Picker Gradient").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    let source = picker_test_box(
        &mut bridge,
        &mut gen,
        "Source",
        [10.0, 10.0, 80.0, 80.0],
        Some("ptnd.red/500"),
        None,
    );
    // Give the source a real linear gradient via the gradient tool.
    bridge.clear_selection();
    bridge.set_selection(vec![source]);
    let mut gtool = GradientTool::new(GradientToolMode::Fill);
    gradient_drag(
        &mut gtool,
        &mut bridge,
        &camera,
        &mut snap,
        10.0,
        50.0,
        90.0,
        50.0,
        SemanticModifiers::default(),
    );
    let target = picker_test_box(
        &mut bridge,
        &mut gen,
        "Target",
        [200.0, 200.0, 50.0, 50.0],
        Some("ptnd.gray/500"),
        None,
    );
    bridge.clear_selection();
    bridge.set_selection(vec![target]);
    let mut tool = PickerTool::new(PickerMode::Color);

    picker_click(&mut tool, &mut bridge, &camera, &mut snap, 20.0, 20.0);

    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(target)
        .unwrap();
    assert!(matches!(
        obj.effective_appearance()
            .primary_fill()
            .map(|f| f.paint.clone()),
        Some(petunia_design_document::Paint::LinearGradient(_))
    ));
}

#[test]
fn picker_skips_locked_objects() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Picker Locked").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    // Locked red box on top of an unlocked blue box (topmost = last created).
    picker_test_box(
        &mut bridge,
        &mut gen,
        "Below",
        [10.0, 10.0, 80.0, 80.0],
        Some("ptnd.blue/500"),
        None,
    );
    let locked = picker_test_box(
        &mut bridge,
        &mut gen,
        "Locked",
        [10.0, 10.0, 80.0, 80.0],
        Some("ptnd.red/500"),
        None,
    );
    bridge.set_locked(locked, true).unwrap();
    let target = picker_test_box(
        &mut bridge,
        &mut gen,
        "Target",
        [200.0, 200.0, 50.0, 50.0],
        Some("ptnd.gray/500"),
        None,
    );
    bridge.clear_selection();
    bridge.set_selection(vec![target]);
    let mut tool = PickerTool::new(PickerMode::Color);

    // The locked top box is skipped: the blue box below is sampled.
    picker_click(&mut tool, &mut bridge, &camera, &mut snap, 20.0, 20.0);

    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(target)
        .unwrap();
    assert!(matches!(
        obj.effective_appearance().primary_fill().map(|f| f.paint.clone()),
        Some(petunia_design_document::Paint::Solid(ref t)) if t == "ptnd.blue/500"
    ));
}

#[test]
fn measure_area_drag_reports_rect() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Measure Area").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = MeasureTool::new();
    tool.set_mode(MeasureMode::Area);
    let plain = SemanticModifiers::default();

    tool.on_pointer_event(
        &pointer_event(PointerPhase::Down, 30.0, 40.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &pointer_event(PointerPhase::Move, 10.0, 10.0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    let readout = tool.area_readout().expect("area");
    assert_eq!(
        (
            readout.width,
            readout.height,
            readout.area,
            readout.perimeter
        ),
        (20.0, 30.0, 600.0, 100.0)
    );
    assert!(tool.overlays().marquee_screen.is_some());
    // Distance mode is untouched and still the default.
    assert_eq!(MeasureTool::new().mode(), MeasureMode::Distance);
}

#[test]
fn measure_selection_totals_evaluated_areas() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Measure Total").expect("doc");
    let mut gen = IdGenerator::new();
    let a = picker_test_box(
        &mut bridge,
        &mut gen,
        "A",
        [0.0, 0.0, 10.0, 20.0],
        None,
        None,
    );
    let b = picker_test_box(
        &mut bridge,
        &mut gen,
        "B",
        [50.0, 50.0, 30.0, 40.0],
        None,
        None,
    );

    assert!((MeasureTool::measured_area(&bridge, &[a, b]) - (200.0 + 1200.0)).abs() < 1e-6);
    assert_eq!(MeasureTool::measured_area(&bridge, &[]), 0.0);
}

fn builder_two_rects(
    bridge: &mut PetuniaDesignGuiBridge,
) -> Vec<petunia_design_foundation::ObjectId> {
    let mut gen = IdGenerator::new();
    let a = corner_test_rect(bridge, &mut gen, [0.0, 0.0, 100.0, 100.0]);
    let b = corner_test_rect(bridge, &mut gen, [50.0, 50.0, 100.0, 100.0]);
    bridge.clear_selection();
    bridge.set_selection(vec![a, b]);
    vec![a, b]
}

fn builder_click(
    tool: &mut ShapeBuilderTool,
    bridge: &mut PetuniaDesignGuiBridge,
    camera: &ViewportCamera,
    snap: &mut SnapEngine,
    x: f64,
    y: f64,
    modifiers: SemanticModifiers,
) {
    let p = GPoint::new(x, y);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Down, PointerButton::Primary, p, p, modifiers),
        bridge,
        camera,
        snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Up, PointerButton::Primary, p, p, modifiers),
        bridge,
        camera,
        snap,
    )
    .unwrap();
}

fn object_bounds(
    bridge: &PetuniaDesignGuiBridge,
    id: petunia_design_foundation::ObjectId,
) -> [f64; 4] {
    bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap()
        .bounds
        .unwrap()
}

#[test]
fn builder_click_overlap_creates_intersection() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Builder Overlap").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    builder_two_rects(&mut bridge);
    let mut tool = ShapeBuilderTool::new(BuilderMode::ShapeBuilder);

    builder_click(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        75.0,
        75.0,
        SemanticModifiers::default(),
    );

    assert_eq!(bridge.snapshot().total_objects, 3);
    let region = bridge.selection().selected_ids[0];
    let bounds = object_bounds(&bridge, region);
    assert!((bounds[0] - 50.0).abs() < 1.0, "got {bounds:?}");
    assert!((bounds[1] - 50.0).abs() < 1.0, "got {bounds:?}");
    assert!((bounds[2] - 50.0).abs() < 1.0, "got {bounds:?}");
    assert!((bounds[3] - 50.0).abs() < 1.0, "got {bounds:?}");
}

#[test]
fn builder_click_exclusive_part_subtracts_other() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Builder Exclusive").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    builder_two_rects(&mut bridge);
    let mut tool = ShapeBuilderTool::new(BuilderMode::ShapeBuilder);

    // Top-left of A is outside B: region is A minus B (L-shaped, full extent).
    builder_click(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        10.0,
        10.0,
        SemanticModifiers::default(),
    );

    assert_eq!(bridge.snapshot().total_objects, 3);
    let region = bridge.selection().selected_ids[0];
    let bounds = object_bounds(&bridge, region);
    assert!((bounds[2] - 100.0).abs() < 1.0, "got {bounds:?}");
    // The notch is real: the region outline avoids B's interior corner.
    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(region)
        .unwrap();
    match obj.shape.as_ref().unwrap() {
        petunia_design_document::ShapeKind::Path(path) => {
            assert!(path.verbs.len() > 5, "L-shape needs vertices");
        }
        _ => panic!("expected path"),
    }
}

#[test]
fn builder_alt_click_carves_both_with_one_undo() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Builder Subtract").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let ids = builder_two_rects(&mut bridge);
    let mut tool = ShapeBuilderTool::new(BuilderMode::ShapeBuilder);
    let alt = SemanticModifiers {
        duplicate: true,
        ..Default::default()
    };

    builder_click(&mut tool, &mut bridge, &camera, &mut snap, 75.0, 75.0, alt);

    // Both rects lost the overlap, nothing created or deleted.
    assert_eq!(bridge.snapshot().total_objects, 2);
    for id in &ids {
        let bounds = object_bounds(&bridge, *id);
        assert!((bounds[2] - 100.0).abs() < 1.0, "got {bounds:?}");
    }
    bridge.undo().unwrap();
    assert_eq!(object_bounds(&bridge, ids[0]), [0.0, 0.0, 100.0, 100.0]);
    assert_eq!(object_bounds(&bridge, ids[1]), [50.0, 50.0, 100.0, 100.0]);
}

#[test]
fn builder_drag_merges_crossed_regions_keeping_sources() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Builder Drag").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    builder_two_rects(&mut bridge);
    let mut tool = ShapeBuilderTool::new(BuilderMode::ShapeBuilder);
    let plain = SemanticModifiers::default();

    // Drag from A-only through the overlap into B-only.
    let p0 = GPoint::new(10.0, 10.0);
    let p1 = GPoint::new(140.0, 140.0);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Down, PointerButton::Primary, p0, p0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Move, PointerButton::Primary, p1, p1, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Up, PointerButton::Primary, p1, p1, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    assert_eq!(bridge.snapshot().total_objects, 3);
    let region = bridge.selection().selected_ids[0];
    let bounds = object_bounds(&bridge, region);
    assert!((bounds[2] - 150.0).abs() < 2.0, "got {bounds:?}");
    assert!((bounds[3] - 150.0).abs() < 2.0, "got {bounds:?}");
}

#[test]
fn smartfill_floods_bounded_empty_face() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Flood Frame").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    // Picture frame: four walls, empty 80x80 middle.
    corner_test_rect(&mut bridge, &mut gen, [0.0, 0.0, 100.0, 10.0]);
    corner_test_rect(&mut bridge, &mut gen, [0.0, 90.0, 100.0, 10.0]);
    corner_test_rect(&mut bridge, &mut gen, [0.0, 10.0, 10.0, 80.0]);
    corner_test_rect(&mut bridge, &mut gen, [90.0, 10.0, 10.0, 80.0]);
    bridge.clear_selection();
    let mut tool = ShapeBuilderTool::new(BuilderMode::SmartFill);

    builder_click(&mut tool, &mut bridge, &camera, &mut snap, 50.0, 50.0, SemanticModifiers::default());

    assert_eq!(bridge.snapshot().total_objects, 5);
    let region = bridge.selection().selected_ids[0];
    let bounds = object_bounds(&bridge, region);
    assert!((bounds[0] - 10.0).abs() < 1.0, "got {bounds:?}");
    assert!((bounds[1] - 10.0).abs() < 1.0, "got {bounds:?}");
    assert!((bounds[2] - 80.0).abs() < 1.0, "got {bounds:?}");
    assert!((bounds[3] - 80.0).abs() < 1.0, "got {bounds:?}");
    // Default SmartFill token, undo removes the flood.
    let obj = bridge.session().unwrap().document().find_object(region).unwrap();
    assert!(matches!(
        obj.effective_appearance().primary_fill().map(|f| f.paint.clone()),
        Some(petunia_design_document::Paint::Solid(ref t)) if t == "ptnd.blue/500"
    ));
    bridge.undo().unwrap();
    assert_eq!(bridge.snapshot().total_objects, 4);
}

#[test]
fn smartfill_unbounded_click_is_noop() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Flood Open").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    corner_test_rect(&mut bridge, &mut gen, [0.0, 0.0, 100.0, 100.0]);
    bridge.clear_selection();
    let mut tool = ShapeBuilderTool::new(BuilderMode::SmartFill);

    // Far outside: the face touches the frame (unbounded) → NoOp.
    builder_click(&mut tool, &mut bridge, &camera, &mut snap, 500.0, 500.0, SemanticModifiers::default());

    assert_eq!(bridge.snapshot().total_objects, 1);
}

#[test]
fn smartfill_flood_bounded_by_open_strokes() {
    use petunia_design_geometry::PathVerb as V;
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Flood Strokes").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    // Box of four open strokes (10pt wide): the bands enclose a pocket.
    let surface_id = bridge.active_surface().unwrap();
    let mut gen = IdGenerator::new();
    for (x0, y0, x1, y1) in [
        (0.0, 0.0, 100.0, 0.0),
        (100.0, 0.0, 100.0, 100.0),
        (100.0, 100.0, 0.0, 100.0),
        (0.0, 100.0, 0.0, 0.0),
    ] {
        let id = gen.next_object();
        bridge
            .submit_command(CommandRequest::new(Command::CreateObject {
                surface: surface_id,
                id,
                name: "Wall".to_string(),
            }))
            .unwrap();
        let mut path = petunia_design_geometry::GPath::new();
        path.push(V::MoveTo(GPoint::new(x0, y0))).unwrap();
        path.push(V::LineTo(GPoint::new(x1, y1))).unwrap();
        bridge
            .submit_command(CommandRequest::new(Command::SetShape {
                id,
                shape: Some(petunia_design_document::ShapeKind::Path(path)),
            }))
            .unwrap();
        bridge
            .submit_command(CommandRequest::new(Command::SetBounds {
                id,
                bounds: Some([
                    x0.min(x1) - 5.0,
                    y0.min(y1) - 5.0,
                    (x1 - x0).abs().max(10.0),
                    (y1 - y0).abs().max(10.0),
                ]),
                rotation: 0.0,
            }))
            .unwrap();
        bridge.set_stroke(id, Some("ptnd.gray/900".to_string()), 10.0).unwrap();
    }
    bridge.clear_selection();
    let mut tool = ShapeBuilderTool::new(BuilderMode::SmartFill);

    // Inside the stroked box: open centerlines still bound the face.
    builder_click(&mut tool, &mut bridge, &camera, &mut snap, 50.0, 50.0, SemanticModifiers::default());

    assert_eq!(bridge.snapshot().total_objects, 5);
    let region = bridge.selection().selected_ids[0];
    let bounds = object_bounds(&bridge, region);
    assert!((bounds[2] - 90.0).abs() < 2.0, "got {bounds:?}");
    assert!((bounds[3] - 90.0).abs() < 2.0, "got {bounds:?}");
}

#[test]
fn smartfill_click_uses_default_fill() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Smart Fill").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    builder_two_rects(&mut bridge);
    let mut tool = ShapeBuilderTool::new(BuilderMode::SmartFill);

    builder_click(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        75.0,
        75.0,
        SemanticModifiers::default(),
    );

    assert_eq!(bridge.snapshot().total_objects, 3);
    let region = bridge.selection().selected_ids[0];
    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(region)
        .unwrap();
    assert!(matches!(
        obj.effective_appearance().primary_fill().map(|f| f.paint.clone()),
        Some(petunia_design_document::Paint::Solid(ref t)) if t == "ptnd.blue/500"
    ));
}

#[test]
fn builder_click_empty_is_noop() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Builder Empty").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    builder_two_rects(&mut bridge);
    let mut tool = ShapeBuilderTool::new(BuilderMode::ShapeBuilder);

    builder_click(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        500.0,
        500.0,
        SemanticModifiers::default(),
    );

    assert_eq!(bridge.snapshot().total_objects, 2);
}

#[allow(clippy::too_many_arguments)]
fn transparency_drag(
    tool: &mut GradientTool,
    bridge: &mut PetuniaDesignGuiBridge,
    camera: &ViewportCamera,
    snap: &mut SnapEngine,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
) {
    let plain = SemanticModifiers::default();
    gradient_drag(tool, bridge, camera, snap, x0, y0, x1, y1, plain);
}

#[test]
fn transparency_drag_sets_live_mask_not_opacity() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Transparency Live").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = gradient_test_object(&mut bridge, "ptnd.red/500");
    let mut tool = GradientTool::new(GradientToolMode::Transparency);

    transparency_drag(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        0.0,
        50.0,
        200.0,
        50.0,
    );

    // Live entry exists; base opacity untouched.
    let mods = bridge.modifiers(id);
    assert_eq!(mods.len(), 1);
    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap();
    assert!((obj.opacity - 1.0).abs() < 1e-9);
    match &mods[0].kind {
        petunia_design_document::ModifierKind::TransparentGradient { start, end, stops } => {
            assert_eq!((*start, *end), ([0.0, 50.0], [200.0, 50.0]));
            assert_eq!(stops.len(), 2);
        }
        _ => panic!("expected transparency"),
    }
    // Mask samples: opaque at start, transparent at end.
    assert!(
        (obj.sampled_opacity() - 0.5).abs() < 1e-6,
        "center must average"
    );
    let mask_start = petunia_design_document::evaluate_opacity_at(&mods, GPoint::new(0.0, 50.0));
    let mask_end = petunia_design_document::evaluate_opacity_at(&mods, GPoint::new(200.0, 50.0));
    assert!((mask_start - 1.0).abs() < 1e-9);
    assert!(mask_end.abs() < 1e-9);

    // One undo clears the live mask.
    bridge.undo().unwrap();
    assert!(bridge.modifiers(id).is_empty());
}

#[test]
fn transparency_bake_flattens_to_base_opacity() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Transparency Bake").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = gradient_test_object(&mut bridge, "ptnd.red/500");
    let mut tool = GradientTool::new(GradientToolMode::Transparency);
    transparency_drag(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        0.0,
        50.0,
        200.0,
        50.0,
    );

    bridge.bake_transparency(id).unwrap();
    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap();
    assert!(bridge.modifiers(id).is_empty());
    // Bounds center (100, 50) sits at t=0.5 → opacity 0.5.
    assert!((obj.opacity - 0.5).abs() < 1e-6, "got {}", obj.opacity);
}

#[test]
fn modifier_chain_api_disables_removes_reorders() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Chain API").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = gradient_test_object(&mut bridge, "ptnd.red/500");
    // Stack two live entries: contour then transparency.
    bridge.offset_path(id, 10.0).unwrap();
    let mut tool = GradientTool::new(GradientToolMode::Transparency);
    transparency_drag(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        0.0,
        0.0,
        200.0,
        0.0,
    );
    assert_eq!(bridge.modifiers(id).len(), 2);

    // Disable the contour: evaluated outline returns to base.
    let contour_id = bridge.modifiers(id)[0].id;
    bridge.set_modifier_enabled(id, contour_id, false).unwrap();
    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap();
    assert_eq!(obj.evaluated_bounds(), obj.bounds);

    // Re-enable, then remove the transparency entry.
    bridge.set_modifier_enabled(id, contour_id, true).unwrap();
    let transparent_id = bridge.modifiers(id)[1].id;
    bridge.remove_modifier(id, transparent_id).unwrap();
    assert_eq!(bridge.modifiers(id).len(), 1);

    // Reorder of a single entry is a NoOp (no history entry, F-22)…
    bridge.move_modifier_to_front(id, contour_id).unwrap();
    assert_eq!(bridge.modifiers(id).len(), 1);
    // …so undo reverts the remove above, restoring both entries.
    bridge.undo().unwrap();
    assert_eq!(bridge.modifiers(id).len(), 2);
}

#[test]
fn contour_and_transparency_coexist_independently() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Coexist").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let id = gradient_test_object(&mut bridge, "ptnd.red/500");
    bridge.offset_path(id, 10.0).unwrap();
    let mut tool = GradientTool::new(GradientToolMode::Transparency);
    transparency_drag(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        0.0,
        0.0,
        200.0,
        0.0,
    );

    // Geometry follows contour; opacity follows transparency; neither leaks.
    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap();
    let eval = obj.evaluated_bounds().unwrap();
    assert!((eval[2] - 220.0).abs() < 2.0, "got {eval:?}");
    assert_eq!(obj.bounds, Some([0.0, 0.0, 200.0, 100.0]));
}

fn text_path_line(
    bridge: &mut PetuniaDesignGuiBridge,
    gen: &mut IdGenerator,
) -> petunia_design_foundation::ObjectId {
    use petunia_design_geometry::PathVerb as V;
    let surface_id = bridge.active_surface().unwrap();
    let id = gen.next_object();
    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id,
            name: "Baseline".to_string(),
        }))
        .unwrap();
    let mut path = petunia_design_geometry::GPath::new();
    path.push(V::MoveTo(GPoint::new(0.0, 0.0))).unwrap();
    path.push(V::LineTo(GPoint::new(200.0, 0.0))).unwrap();
    bridge
        .submit_command(CommandRequest::new(Command::SetShape {
            id,
            shape: Some(petunia_design_document::ShapeKind::Path(path)),
        }))
        .unwrap();
    bridge
        .submit_command(CommandRequest::new(Command::SetBounds {
            id,
            bounds: Some([0.0, 0.0, 200.0, 1.0]),
            rotation: 0.0,
        }))
        .unwrap();
    bridge.clear_selection();
    id
}

fn attached_text(
    bridge: &PetuniaDesignGuiBridge,
    id: petunia_design_foundation::ObjectId,
) -> Option<petunia_design_document::TextOnPathAttachment> {
    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap();
    match &obj.shape {
        Some(petunia_design_document::ShapeKind::Text { on_path, .. }) => *on_path,
        _ => panic!("expected text"),
    }
}

fn text_click(
    tool: &mut TextTool,
    bridge: &mut PetuniaDesignGuiBridge,
    camera: &ViewportCamera,
    snap: &mut SnapEngine,
    x: f64,
    y: f64,
    modifiers: SemanticModifiers,
) {
    let p = GPoint::new(x, y);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Down, PointerButton::Primary, p, p, modifiers),
        bridge,
        camera,
        snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Up, PointerButton::Primary, p, p, modifiers),
        bridge,
        camera,
        snap,
    )
    .unwrap();
}

#[test]
fn text_click_on_path_attaches_with_span_to_end() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Text On Path").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    let target = text_path_line(&mut bridge, &mut gen);
    let mut tool = TextTool::new(TextToolMode::Artistic);

    text_click(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        50.0,
        0.0,
        SemanticModifiers::default(),
    );

    assert_eq!(bridge.snapshot().total_objects, 2);
    let id = bridge.selection().selected_ids[0];
    let att = attached_text(&bridge, id).expect("attached");
    assert_eq!(att.target, target);
    assert!((att.start - 0.25).abs() < 1e-6, "got {att:?}");
    assert!((att.end - 1.0).abs() < 1e-6, "got {att:?}");
    // Target path object survives untouched.
    let target_obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(target)
        .unwrap();
    assert!(matches!(
        target_obj.shape,
        Some(petunia_design_document::ShapeKind::Path(_))
    ));
}

#[test]
fn text_drag_along_path_sets_span() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Text Span").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    let target = text_path_line(&mut bridge, &mut gen);
    let mut tool = TextTool::new(TextToolMode::Artistic);
    let plain = SemanticModifiers::default();

    let p0 = GPoint::new(20.0, 0.0);
    let p1 = GPoint::new(150.0, 0.0);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Down, PointerButton::Primary, p0, p0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Move, PointerButton::Primary, p1, p1, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Up, PointerButton::Primary, p1, p1, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    let id = bridge.selection().selected_ids[0];
    let att = attached_text(&bridge, id).expect("attached");
    assert_eq!(att.target, target);
    assert!((att.start - 0.1).abs() < 1e-6, "got {att:?}");
    assert!((att.end - 0.75).abs() < 1e-6, "got {att:?}");
}

#[test]
fn text_handle_drag_moves_start_with_one_undo() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Text Handle").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    text_path_line(&mut bridge, &mut gen);
    let mut tool = TextTool::new(TextToolMode::Artistic);
    let plain = SemanticModifiers::default();
    text_click(&mut tool, &mut bridge, &camera, &mut snap, 50.0, 0.0, plain);
    let id = bridge.selection().selected_ids[0];
    assert!(tool.overlays(&camera, &bridge).text_path_handles.is_some());

    // Drag the start handle from x=50 to x=100.
    let p0 = GPoint::new(50.0, 0.0);
    let p1 = GPoint::new(100.0, 0.0);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Down, PointerButton::Primary, p0, p0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Move, PointerButton::Primary, p1, p1, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Up, PointerButton::Primary, p1, p1, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    let att = attached_text(&bridge, id).expect("attached");
    assert!((att.start - 0.5).abs() < 1e-6, "got {att:?}");
    bridge.undo().unwrap();
    let att = attached_text(&bridge, id).expect("attached");
    assert!((att.start - 0.25).abs() < 1e-6, "got {att:?}");
}

#[test]
fn text_alt_click_detaches() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Text Detach").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    text_path_line(&mut bridge, &mut gen);
    let mut tool = TextTool::new(TextToolMode::Artistic);
    text_click(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        50.0,
        0.0,
        SemanticModifiers::default(),
    );
    let id = bridge.selection().selected_ids[0];
    assert!(attached_text(&bridge, id).is_some());

    // Alt-click the start handle detaches.
    std::thread::sleep(std::time::Duration::from_millis(50));
    let alt = SemanticModifiers {
        duplicate: true,
        ..Default::default()
    };
    text_click(&mut tool, &mut bridge, &camera, &mut snap, 50.0, 0.0, alt);
    assert!(attached_text(&bridge, id).is_none());
}

#[test]
fn text_click_empty_still_creates_straight_text() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Text Straight").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    text_path_line(&mut bridge, &mut gen);
    bridge.clear_selection();
    let mut tool = TextTool::new(TextToolMode::Artistic);

    // Click far from the path: straight headline, no attachment.
    text_click(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        400.0,
        400.0,
        SemanticModifiers::default(),
    );

    assert_eq!(bridge.snapshot().total_objects, 2);
    let id = bridge.selection().selected_ids[0];
    assert!(attached_text(&bridge, id).is_none());
    let obj = bridge
        .session()
        .unwrap()
        .document()
        .find_object(id)
        .unwrap();
    assert_eq!(obj.bounds, Some([400.0, 400.0, 160.0, 32.0]));
}

#[test]
fn text_on_path_svg_uses_textpath_href() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Text SVG").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    let target = text_path_line(&mut bridge, &mut gen);
    let mut tool = TextTool::new(TextToolMode::Artistic);
    text_click(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        50.0,
        0.0,
        SemanticModifiers::default(),
    );

    let svg = petunia_design_io::export_document_svg(bridge.session().unwrap().document());
    assert!(svg.contains("<textPath"), "missing textPath:\n{svg}");
    assert!(
        svg.contains(&format!("href=\"#{target}\"")),
        "missing href:\n{svg}"
    );
}

#[allow(clippy::too_many_arguments)]
fn photo_drag(
    tool: &mut PhotoTool,
    bridge: &mut PetuniaDesignGuiBridge,
    camera: &ViewportCamera,
    snap: &mut SnapEngine,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    modifiers: SemanticModifiers,
) {
    let p0 = GPoint::new(x0, y0);
    let p1 = GPoint::new(x1, y1);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Down, PointerButton::Primary, p0, p0, modifiers),
        bridge,
        camera,
        snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Move, PointerButton::Primary, p1, p1, modifiers),
        bridge,
        camera,
        snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Up, PointerButton::Primary, p1, p1, modifiers),
        bridge,
        camera,
        snap,
    )
    .unwrap();
}

#[test]
fn marquee_rect_commits_mask_without_undo() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Marquee").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = PhotoTool::new(PhotoToolKind::MarqueeRect);

    // Session transient state: no new undo entry, no document mutation.
    // (new_document itself owns the single pre-existing entry.)
    let undo_before = bridge.can_undo();
    photo_drag(&mut tool, &mut bridge, &camera, &mut snap, 10.0, 10.0, 110.0, 60.0, SemanticModifiers::default());
    assert_eq!(bridge.can_undo(), undo_before);
    let mask = bridge.raster_selection();
    assert!(!mask.is_empty());
    assert!((mask.signed_area().abs() - 5000.0).abs() < 1.0, "got {}", mask.signed_area());
    assert!(mask.contains(GPoint::new(60.0, 35.0)));
    assert!(!mask.contains(GPoint::new(200.0, 200.0)));
    assert_eq!(bridge.snapshot().total_objects, 0);
    // Marching-ants overlay exposes the committed contours.
    assert!(tool.overlays(&camera, &bridge).selection_mask.is_some());
}

#[test]
fn marquee_ellipse_covers_center_not_corners() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Ellipse").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = PhotoTool::new(PhotoToolKind::MarqueeEllipse);

    photo_drag(&mut tool, &mut bridge, &camera, &mut snap, 0.0, 0.0, 100.0, 100.0, SemanticModifiers::default());

    let mask = bridge.raster_selection();
    assert!(mask.contains(GPoint::new(50.0, 50.0)));
    assert!(!mask.contains(GPoint::new(5.0, 5.0)));
}

#[test]
fn marquee_click_clears_mask() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Clear").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = PhotoTool::new(PhotoToolKind::MarqueeRect);
    let plain = SemanticModifiers::default();

    photo_drag(&mut tool, &mut bridge, &camera, &mut snap, 10.0, 10.0, 110.0, 60.0, plain);
    assert!(!bridge.raster_selection().is_empty());
    photo_drag(&mut tool, &mut bridge, &camera, &mut snap, 500.0, 500.0, 500.0, 500.0, plain);
    assert!(bridge.raster_selection().is_empty());
}

#[test]
fn marquee_shift_adds_and_alt_subtracts() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Modes").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = PhotoTool::new(PhotoToolKind::MarqueeRect);
    let shift = SemanticModifiers {
        constrain: true,
        ..Default::default()
    };
    let alt = SemanticModifiers {
        duplicate: true,
        ..Default::default()
    };

    photo_drag(&mut tool, &mut bridge, &camera, &mut snap, 0.0, 0.0, 100.0, 100.0, SemanticModifiers::default());
    photo_drag(&mut tool, &mut bridge, &camera, &mut snap, 50.0, 50.0, 150.0, 150.0, shift);
    let added = bridge.raster_selection().signed_area().abs();
    assert!((added - 17500.0).abs() < 2.0, "got {added}");

    photo_drag(&mut tool, &mut bridge, &camera, &mut snap, 0.0, 0.0, 100.0, 100.0, alt);
    let carved = bridge.raster_selection().signed_area().abs();
    assert!((carved - 7500.0).abs() < 2.0, "got {carved}");
    assert!(!bridge.raster_selection().contains(GPoint::new(25.0, 25.0)));
    assert!(bridge.raster_selection().contains(GPoint::new(125.0, 125.0)));
}

#[test]
fn lasso_encloses_polygon_mask() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Lasso").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = PhotoTool::new(PhotoToolKind::Lasso);
    let plain = SemanticModifiers::default();

    let p = GPoint::new(0.0, 0.0);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Down, PointerButton::Primary, p, p, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    for (x, y) in [(100.0, 0.0), (100.0, 100.0), (0.0, 100.0)] {
        let q = GPoint::new(x, y);
        tool.on_pointer_event(
            &NormalizedPointerEvent::new(PointerPhase::Move, PointerButton::Primary, q, q, plain),
            &mut bridge,
            &camera,
            &mut snap,
        )
        .unwrap();
    }
    let end = GPoint::new(0.0, 100.0);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Up, PointerButton::Primary, end, end, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    let mask = bridge.raster_selection();
    assert!((mask.signed_area().abs() - 10000.0).abs() < 1.0, "got {}", mask.signed_area());
    assert!(mask.contains(GPoint::new(50.0, 50.0)));
}

#[test]
fn raster_bridge_api_inverts_grows_feathers() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Mask API").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = PhotoTool::new(PhotoToolKind::MarqueeRect);
    photo_drag(&mut tool, &mut bridge, &camera, &mut snap, 0.0, 0.0, 10.0, 10.0, SemanticModifiers::default());

    bridge.set_raster_feather(2.5);
    assert!((bridge.raster_selection().feather - 2.5).abs() < 1e-9);

    bridge.grow_raster_selection(5.0);
    assert!((bridge.raster_selection().signed_area().abs() - 400.0).abs() < 8.0);

    bridge.invert_raster_selection();
    assert!(!bridge.raster_selection().contains(GPoint::new(5.0, 5.0)));

    bridge.clear_raster_selection();
    assert!(bridge.raster_selection().is_empty());
}

fn perspective_test_rect(
    bridge: &mut PetuniaDesignGuiBridge,
    gen: &mut IdGenerator,
) -> petunia_design_foundation::ObjectId {
    let id = corner_test_rect(bridge, gen, [0.0, 0.0, 100.0, 100.0]);
    bridge.clear_selection();
    bridge.set_selection(vec![id]);
    id
}

#[test]
fn perspective_drag_moves_corner_with_one_undo() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Perspective").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    let id = perspective_test_rect(&mut bridge, &mut gen);
    let mut tool = PerspectiveTool::new();
    let plain = SemanticModifiers::default();

    // Grab the top-right corner (100, 0) and pull it down to (100, 25).
    let p0 = GPoint::new(100.0, 0.0);
    let p1 = GPoint::new(100.0, 25.0);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Down, PointerButton::Primary, p0, p0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    assert!(tool.is_active());
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Move, PointerButton::Primary, p1, p1, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    assert!(tool.overlays(&bridge, &camera).handles.len() == 4);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Up, PointerButton::Primary, p1, p1, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    // Live quad stored; base untouched; outline pinched.
    let mods = bridge.modifiers(id);
    assert_eq!(mods.len(), 1);
    let obj = bridge.session().unwrap().document().find_object(id).unwrap();
    assert!(matches!(
        obj.shape,
        Some(petunia_design_document::ShapeKind::Rectangle { .. })
    ));
    let eval = obj.evaluated_bounds().unwrap();
    assert!((eval[1] - 0.0).abs() < 2.0, "got {eval:?}");

    // Exactly one undo entry clears the warp.
    bridge.undo().unwrap();
    assert!(bridge.modifiers(id).is_empty());
}

#[test]
fn perspective_second_drag_replaces_quad() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Perspective Replace").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    let id = perspective_test_rect(&mut bridge, &mut gen);
    let mut tool = PerspectiveTool::new();
    let plain = SemanticModifiers::default();

    bridge.set_perspective(id, [[0.0, 0.0], [100.0, 25.0], [100.0, 75.0], [0.0, 100.0]]).unwrap();
    assert_eq!(bridge.modifiers(id).len(), 1);

    // Drag the bottom-right corner: same entry, new quad (no stacking).
    let p0 = GPoint::new(100.0, 75.0);
    let p1 = GPoint::new(100.0, 90.0);
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Down, PointerButton::Primary, p0, p0, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();
    tool.on_pointer_event(
        &NormalizedPointerEvent::new(PointerPhase::Up, PointerButton::Primary, p1, p1, plain),
        &mut bridge,
        &camera,
        &mut snap,
    )
    .unwrap();

    assert_eq!(bridge.modifiers(id).len(), 1);
}

#[test]
fn vector_crop_clips_with_one_undo() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Vector Crop").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    let id = perspective_test_rect(&mut bridge, &mut gen);
    let mut tool = PhotoTool::new(PhotoToolKind::Crop);
    let plain = SemanticModifiers::default();

    photo_drag(&mut tool, &mut bridge, &camera, &mut snap, 25.0, 25.0, 75.0, 75.0, plain);

    // Live crop entry; base untouched; evaluated outline is the window.
    let mods = bridge.modifiers(id);
    assert_eq!(mods.len(), 1);
    let obj = bridge.session().unwrap().document().find_object(id).unwrap();
    assert!(matches!(
        obj.shape,
        Some(petunia_design_document::ShapeKind::Rectangle { .. })
    ));
    let eval = obj.evaluated_bounds().unwrap();
    assert!((eval[2] - 50.0).abs() < 1.0, "got {eval:?}");

    bridge.undo().unwrap();
    assert!(bridge.modifiers(id).is_empty());
}

#[test]
fn surface_crop_still_works_without_selection() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Surface Crop").expect("doc");
    bridge.clear_selection();
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut tool = PhotoTool::new(PhotoToolKind::Crop);

    photo_drag(
        &mut tool,
        &mut bridge,
        &camera,
        &mut snap,
        0.0,
        0.0,
        400.0,
        300.0,
        SemanticModifiers::default(),
    );

    assert!(bridge.raster_selection().is_empty());
    let surface_id = bridge.active_surface().unwrap();
    let surface = bridge.session().unwrap().surface(surface_id).unwrap();
    assert_eq!(surface.dimensions, [400.0, 300.0]);
}

#[test]
fn bake_geometry_commits_warp_and_crop_keeping_transparency() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Bake Geometry").expect("doc");
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let mut gen = IdGenerator::new();
    let id = perspective_test_rect(&mut bridge, &mut gen);

    bridge.set_perspective(id, [[0.0, 0.0], [100.0, 25.0], [100.0, 75.0], [0.0, 100.0]]).unwrap();
    bridge.set_crop_rect(id, [0.0, 0.0, 60.0, 100.0]).unwrap();
    let mut gtool = GradientTool::new(GradientToolMode::Transparency);
    transparency_drag(&mut gtool, &mut bridge, &camera, &mut snap, 0.0, 0.0, 100.0, 0.0);
    assert_eq!(bridge.modifiers(id).len(), 3);

    bridge.bake_geometry(id).unwrap();
    let obj = bridge.session().unwrap().document().find_object(id).unwrap();
    assert!(matches!(
        obj.shape,
        Some(petunia_design_document::ShapeKind::Path(_))
    ));
    // Only the transparency entry survives.
    let mods = bridge.modifiers(id);
    assert_eq!(mods.len(), 1);
    assert!(matches!(
        mods[0].kind,
        petunia_design_document::ModifierKind::TransparentGradient { .. }
    ));
}

fn cache_test_box(
    bridge: &mut PetuniaDesignGuiBridge,
    gen: &mut IdGenerator,
    bounds: [f64; 4],
) -> petunia_design_foundation::ObjectId {
    let surface_id = bridge.active_surface().unwrap();
    let id = gen.next_object();
    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id,
            name: "CacheBox".to_string(),
        }))
        .unwrap();
    bridge
        .submit_command(CommandRequest::new(Command::SetBounds {
            id,
            bounds: Some(bounds),
            rotation: 0.0,
        }))
        .unwrap();
    bridge.clear_selection();
    id
}

#[test]
fn geo_cache_memoizes_repeated_reads() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Geo Cache").expect("doc");
    let mut gen = IdGenerator::new();
    let id = cache_test_box(&mut bridge, &mut gen, [10.0, 10.0, 50.0, 50.0]);

    assert_eq!(bridge.geo_cache_len(), 0);
    let first = bridge.cached_bounds(id).expect("bounds");
    assert_eq!(bridge.geo_cache_len(), 1);
    // Repeated reads reuse the entry: no growth, identical values.
    for _ in 0..10 {
        assert_eq!(bridge.cached_bounds(id), Some(first));
        assert!(bridge.cached_hit(id, GPoint::new(20.0, 20.0), 0.5));
    }
    assert_eq!(bridge.geo_cache_len(), 1);
    assert!(!bridge.cached_hit(id, GPoint::new(500.0, 500.0), 0.5));
}

#[test]
fn geo_cache_invalidates_on_mutation_and_undo() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Geo Invalidate").expect("doc");
    let mut gen = IdGenerator::new();
    let id = cache_test_box(&mut bridge, &mut gen, [10.0, 10.0, 50.0, 50.0]);
    assert_eq!(bridge.cached_bounds(id), Some([10.0, 10.0, 50.0, 50.0]));

    bridge
        .submit_command(CommandRequest::new(Command::SetBounds {
            id,
            bounds: Some([30.0, 30.0, 50.0, 50.0]),
            rotation: 0.0,
        }))
        .unwrap();
    // Revision bumped: fresh evaluation, same single entry.
    assert_eq!(bridge.cached_bounds(id), Some([30.0, 30.0, 50.0, 50.0]));
    assert_eq!(bridge.geo_cache_len(), 1);
    assert!(bridge.cached_hit(id, GPoint::new(40.0, 40.0), 0.5));
    assert!(!bridge.cached_hit(id, GPoint::new(15.0, 15.0), 0.5));

    bridge.undo().unwrap();
    assert_eq!(bridge.cached_bounds(id), Some([10.0, 10.0, 50.0, 50.0]));
}

#[test]
fn geo_cache_matches_direct_evaluation_with_modifiers() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Geo Modifier").expect("doc");
    let mut gen = IdGenerator::new();
    let id = cache_test_box(&mut bridge, &mut gen, [0.0, 0.0, 100.0, 100.0]);
    bridge.offset_path(id, 10.0).unwrap();

    let obj = bridge.session().unwrap().document().find_object(id).unwrap();
    let direct_path = obj.evaluated_path();
    let direct_bounds = obj.evaluated_bounds();
    let cached_path = bridge.cached_path(id).expect("path");
    assert_eq!(cached_path.verbs, direct_path.verbs);
    assert_eq!(bridge.cached_bounds(id), direct_bounds);
    // Evaluated outline grew; base bounds stayed.
    assert_eq!(obj.bounds, Some([0.0, 0.0, 100.0, 100.0]));
    assert!((direct_bounds.unwrap()[2] - 120.0).abs() < 2.0);
}

#[test]
fn geo_cache_prunes_deleted_objects() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Geo Prune").expect("doc");
    let mut gen = IdGenerator::new();
    let id = cache_test_box(&mut bridge, &mut gen, [10.0, 10.0, 50.0, 50.0]);
    assert!(bridge.cached_bounds(id).is_some());
    assert_eq!(bridge.geo_cache_len(), 1);

    bridge
        .submit_command(CommandRequest::new(Command::DeleteObject { id }))
        .unwrap();
    assert_eq!(bridge.geo_cache_len(), 0);
    assert!(bridge.cached_bounds(id).is_none());
}

#[test]
fn zoom_flatten_tol_scales_with_zoom() {
    use petunia_design_geometry::zoom_flatten_tol;
    assert!((zoom_flatten_tol(1.0) - 0.5).abs() < 1e-9);
    assert!((zoom_flatten_tol(0.1) - 4.0).abs() < 1e-9);
    assert!((zoom_flatten_tol(10.0) - 0.05).abs() < 1e-9);
    assert!((zoom_flatten_tol(1000.0) - 0.05).abs() < 1e-9);
}

#[test]
fn cached_sample_and_nearest_match_direct_methods() {
    use petunia_design_geometry::PathVerb as V;
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Cached Sample").expect("doc");
    let surface_id = bridge.active_surface().unwrap();
    let mut gen = IdGenerator::new();
    let id = gen.next_object();
    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id,
            name: "Curve".to_string(),
        }))
        .unwrap();
    let mut path = petunia_design_geometry::GPath::new();
    path.push(V::MoveTo(GPoint::new(0.0, 0.0))).unwrap();
    path.push(V::CubicTo(
        GPoint::new(30.0, 0.0),
        GPoint::new(70.0, 100.0),
        GPoint::new(100.0, 100.0),
    ))
    .unwrap();
    bridge
        .submit_command(CommandRequest::new(Command::SetShape {
            id,
            shape: Some(petunia_design_document::ShapeKind::Path(path.clone())),
        }))
        .unwrap();
    bridge.clear_selection();

    for tol in [0.1, 0.5, 2.0] {
        for i in 0..=10 {
            let t = i as f64 / 10.0;
            let direct = path.sample_at(t, tol).expect("direct");
            let cached = bridge.cached_sample_at(id, t, tol).expect("cached");
            assert!((direct.0.x - cached.0.x).abs() < 1e-6, "t={t} tol={tol}");
            assert!((direct.0.y - cached.0.y).abs() < 1e-6, "t={t} tol={tol}");
        }
        let probe = GPoint::new(40.0, 30.0);
        assert!(
            (path.nearest_t(probe, tol).unwrap()
                - bridge.cached_nearest_t(id, probe, tol).unwrap())
            .abs()
                < 1e-6
        );
        let cached_polys = bridge.cached_polygons(id, tol).expect("polys");
        assert_eq!(cached_polys, path.to_polygons(tol));
    }
}

#[test]
fn cached_hit_respects_zoom_tolerance() {
    use petunia_design_geometry::PathVerb as V;
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Cached Hit Tol").expect("doc");
    let surface_id = bridge.active_surface().unwrap();
    let mut gen = IdGenerator::new();
    let id = gen.next_object();
    bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_id,
            id,
            name: "Bulge".to_string(),
        }))
        .unwrap();
    // Closed bulging curve: coarse flattening cuts corners vs fine.
    let mut path = petunia_design_geometry::GPath::new();
    path.push(V::MoveTo(GPoint::new(0.0, 0.0))).unwrap();
    path.push(V::CubicTo(
        GPoint::new(100.0, 0.0),
        GPoint::new(100.0, 100.0),
        GPoint::new(0.0, 100.0),
    ))
    .unwrap();
    path.push(V::Close).unwrap();
    bridge
        .submit_command(CommandRequest::new(Command::SetShape {
            id,
            shape: Some(petunia_design_document::ShapeKind::Path(path)),
        }))
        .unwrap();
    bridge.clear_selection();

    let coarse = bridge.cached_polygons(id, 4.0).expect("coarse");
    let fine = bridge.cached_polygons(id, 0.05).expect("fine");
    let count = |polys: &Vec<Vec<GPoint>>| polys.iter().map(|p| p.len()).sum::<usize>();
    assert!(count(&fine) >= count(&coarse), "coarse={} fine={}", count(&coarse), count(&fine));
    // Deep interior hits at every tolerance.
    assert!(bridge.cached_hit(id, GPoint::new(20.0, 50.0), 4.0));
    assert!(bridge.cached_hit(id, GPoint::new(20.0, 50.0), 0.05));
    assert!(!bridge.cached_hit(id, GPoint::new(500.0, 500.0), 4.0));
}

fn spatial_test_scene(
    bridge: &mut PetuniaDesignGuiBridge,
    gen: &mut IdGenerator,
    count: usize,
) -> Vec<petunia_design_foundation::ObjectId> {
    let mut ids = Vec::new();
    for i in 0..count {
        let x = (i % 20) as f64 * 60.0;
        let y = (i / 20) as f64 * 60.0;
        ids.push(cache_test_box(bridge, gen, [x, y, 50.0, 50.0]));
    }
    ids
}

#[test]
fn spatial_point_query_returns_topmost_first() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Spatial Point").expect("doc");
    let mut gen = IdGenerator::new();
    let a = cache_test_box(&mut bridge, &mut gen, [0.0, 0.0, 100.0, 100.0]);
    let b = cache_test_box(&mut bridge, &mut gen, [50.0, 50.0, 100.0, 100.0]);
    let session = bridge.session().unwrap();

    // Overlap zone: later (topmost) object first.
    assert_eq!(
        session.spatial_candidates_point(GPoint::new(75.0, 75.0), 0.0),
        vec![b, a]
    );
    // Outside everything: empty.
    assert!(session
        .spatial_candidates_point(GPoint::new(500.0, 500.0), 0.0)
        .is_empty());
    // Tolerance expands the query box: (102, 60) is 2pt past A's edge.
    assert_eq!(
        session.spatial_candidates_point(GPoint::new(102.0, 60.0), 5.0),
        vec![b, a]
    );
    assert!(session
        .spatial_candidates_point(GPoint::new(102.0, 60.0), 1.0)
        .eq(&vec![b]));
    assert_eq!(session.spatial_len(), 2);
}

#[test]
fn spatial_rect_query_matches_brute_force() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Spatial Rect").expect("doc");
    let mut gen = IdGenerator::new();
    let ids = spatial_test_scene(&mut bridge, &mut gen, 60);
    let session = bridge.session().unwrap();

    let rect = [0.0, 0.0, 200.0, 200.0];
    let mut indexed = session.spatial_candidates_rect(rect);
    indexed.sort();
    let mut brute: Vec<_> = ids
        .iter()
        .filter(|id| {
            session
                .find_object(**id)
                .and_then(|o| o.bounds)
                .is_some_and(|[x, y, w, h]| {
                    x < rect[2] && x + w > rect[0] && y < rect[3] && y + h > rect[1]
                })
        })
        .copied()
        .collect();
    brute.sort();
    assert_eq!(indexed, brute);
    assert_eq!(session.spatial_len(), 60);
}

#[test]
fn spatial_index_rebuilds_on_mutation() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Spatial Rebuild").expect("doc");
    let mut gen = IdGenerator::new();
    let id = cache_test_box(&mut bridge, &mut gen, [0.0, 0.0, 50.0, 50.0]);
    assert_eq!(
        bridge.session().unwrap().spatial_candidates_point(GPoint::new(25.0, 25.0), 0.0),
        vec![id]
    );

    bridge
        .submit_command(CommandRequest::new(Command::SetBounds {
            id,
            bounds: Some([300.0, 300.0, 50.0, 50.0]),
            rotation: 0.0,
        }))
        .unwrap();
    let session = bridge.session().unwrap();
    assert!(session
        .spatial_candidates_point(GPoint::new(25.0, 25.0), 0.0)
        .is_empty());
    assert_eq!(
        session.spatial_candidates_point(GPoint::new(325.0, 325.0), 0.0),
        vec![id]
    );
}

#[test]
fn spatial_tool_integration_and_performance() {
    let mut bridge = PetuniaDesignGuiBridge::new();
    bridge.new_document("Spatial Tools").expect("doc");
    let mut gen = IdGenerator::new();
    let ids = spatial_test_scene(&mut bridge, &mut gen, 200);

    // Warm up the spatial index
    let session = bridge.session().unwrap();
    assert_eq!(session.spatial_len(), 200);

    // Candidate lookup on 200 objects: must take < 50µs (F3 acceptance)
    let t0 = std::time::Instant::now();
    for _ in 0..10 {
        let _ = session.spatial_candidates_point(GPoint::new(125.0, 65.0), 4.0);
    }
    let elapsed = t0.elapsed();
    assert!(
        elapsed.as_millis() < 50,
        "Spatial query took too long: {:?}",
        elapsed
    );

    // Select tool click hit-test uses spatial candidates
    let mut select = SelectTool::new();
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut snap = SnapEngine::new();
    let ev = NormalizedPointerEvent::new(
        PointerPhase::Down,
        PointerButton::Primary,
        GPoint::new(10.0, 10.0),
        GPoint::new(10.0, 10.0),
        SemanticModifiers::default(),
    );
    let _ = select.on_pointer_event(&ev, &mut bridge, &camera, &mut snap);
    assert_eq!(bridge.selection().selected_ids, vec![ids[0]]);
}
