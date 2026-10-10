use petunia_core::{Point, VectorPath};
use petunia_ui::{PointerEvent, PointerSample, StudioSession, ToolKind, UserAction};

fn pointer(session: &mut StudioSession, phase: &str, x: f64, y: f64, shift: bool) {
    let position = Point::new(x, y);
    let event = match phase {
        "down" => PointerEvent::Down {
            position,
            pressure: 0.5,
        },
        "move" => PointerEvent::Move {
            position,
            pressure: 0.5,
        },
        _ => PointerEvent::Up { position },
    };
    session
        .dispatch_pointer_sample(
            event,
            PointerSample {
                position,
                pressure: 0.5,
                shift,
                ctrl: false,
                alt: false,
                timestamp_ms: 1,
            },
        )
        .unwrap();
}

#[test]
fn object_drag_commits_once_on_release_and_undo_restores_transform() {
    let mut session = StudioSession::new("drag");
    session
        .insert_path("box", VectorPath::rect(10., 10., 20., 20.))
        .unwrap();
    let id = session
        .document()
        .scene
        .page_roots(session.active_page())
        .unwrap()[0];
    let original = session.document().scene.get_node(id).unwrap().transform;
    let revision = session.revision();
    pointer(&mut session, "down", 15., 15., false);
    pointer(&mut session, "move", 35., 45., false);
    assert_eq!(session.revision(), revision);
    pointer(&mut session, "up", 35., 45., false);
    assert_eq!(session.revision().0, revision.0 + 1);
    assert_ne!(
        session.document().scene.get_node(id).unwrap().transform,
        original
    );
    session.dispatch_action(UserAction::Undo).unwrap();
    assert_eq!(
        session.document().scene.get_node(id).unwrap().transform,
        original
    );
}

#[test]
fn pen_draft_can_be_cancelled_after_release_or_finished_as_one_transaction() {
    let mut session = StudioSession::new("pen");
    session.select_tool(ToolKind::Pen);
    pointer(&mut session, "down", 10., 10., false);
    pointer(&mut session, "up", 10., 10., false);
    assert!(session.interaction_active());
    assert_eq!(session.revision().0, 0);
    session.on_escape();
    assert!(!session.interaction_active());
    assert_eq!(session.revision().0, 0);
    for (x, y) in [(10., 10.), (80., 30.)] {
        pointer(&mut session, "down", x, y, false);
        pointer(&mut session, "up", x, y, false);
    }
    session.finish_pen().unwrap();
    assert!(!session.interaction_active());
    assert_eq!(session.revision().0, 1);
    session.dispatch_action(UserAction::Undo).unwrap();
    assert!(session
        .document()
        .scene
        .page_roots(session.active_page())
        .unwrap()
        .is_empty());
}

#[test]
fn shift_selection_preserves_existing_selection_without_moving_objects() {
    let mut session = StudioSession::new("selection");
    session
        .insert_path("one", VectorPath::rect(10., 10., 20., 20.))
        .unwrap();
    session
        .insert_path("two", VectorPath::rect(50., 10., 20., 20.))
        .unwrap();
    let roots = session
        .document()
        .scene
        .page_roots(session.active_page())
        .unwrap();
    let (first, second) = (roots[0], roots[1]);
    session.set_selection(vec![first]).unwrap();
    let revision = session.revision();
    pointer(&mut session, "down", 55., 15., true);
    pointer(&mut session, "up", 55., 15., true);
    assert!(session.selection().contains(first));
    assert!(session.selection().contains(second));
    assert_eq!(session.revision(), revision);
}
