use petunia_core::*;
use petunia_engine::persistence::{save_snapshot, SavePolicy};
use petunia_engine::ptnd::PackageLimits;
use petunia_engine::{CommandId, DocumentOp, HistoryDescription, TransactionRequest};
use petunia_ui::{StudioSession, ToolSession, UserAction};

fn request(operations: Vec<DocumentOp>) -> TransactionRequest {
    TransactionRequest {
        command_id: CommandId::new_v4(),
        operations,
        merge_key: None,
    }
}
fn directory() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("petunia-session-{}", DocumentId::new_v4()));
    std::fs::create_dir_all(&path).unwrap();
    path
}
fn image(session: &mut StudioSession) -> ObjectId {
    let bytes = include_bytes!("../../petunia-engine/tests/fixtures/red.png").to_vec();
    let resource = ResourceId::new_v4();
    let record = ResourceRecord {
        id: resource,
        kind: ResourceKind::Image,
        source: ResourceSource::Embedded {
            entry: format!("{resource}.png"),
        },
        content_hash: Some(ContentHash::new(&bytes)),
        metadata: ResourceMetadata::default(),
    };
    let mut node = SceneNode::new_path(
        "red",
        VectorPath::new(),
        ParentRef::Page(session.active_page()),
    );
    let id = node.id;
    node.item = SceneItem::Image(petunia_engine::io::place_image(
        resource,
        ImageSamplingPolicy::Nearest,
    ));
    session
        .apply_edit(
            request(vec![
                DocumentOp::Registry(petunia_engine::transaction::RegistryOp::Resource {
                    id: resource,
                    value: Some(Box::new(record)),
                }),
                DocumentOp::InsertRoot {
                    index: 0,
                    node: Box::new(node),
                },
            ]),
            HistoryDescription::InsertObjects,
        )
        .unwrap();
    session.set_resource_bytes(resource, bytes).unwrap();
    id
}

#[test]
fn resource_render_clipboard_save_open_and_undo_share_one_document_contract() {
    let mut session = StudioSession::new("resources");
    session.view_mut().viewport = Rect::new(0., 0., 8., 8.);
    let original = image(&mut session);
    session.select_objects(vec![original]);
    let copied = session.copy_selection().unwrap();
    session.paste(&copied).unwrap();
    let pasted = session.selection().objects()[0];
    assert_ne!(original, pasted);
    let (pixels, _) = session
        .render_headless(&petunia_render::RenderOptions::default())
        .unwrap();
    assert_eq!(&pixels[..4], &[255, 0, 0, 255]);
    session.dispatch_action(UserAction::Undo).unwrap();
    assert!(session.document().scene.get_node(pasted).is_none());
    session.dispatch_action(UserAction::Redo).unwrap();
    assert!(session.document().scene.get_node(pasted).is_some());
    let directory = directory();
    let path = directory.join("image.ptnd");
    session.save_as(&path, SavePolicy::CreateNew).unwrap();
    assert!(!session.is_dirty());
    let mut opened = StudioSession::new("old");
    opened.view_mut().viewport = session.view().viewport;
    opened.open(&path).unwrap();
    assert_eq!(opened.document().id, session.document().id);
    assert_eq!(opened.document().resources.len(), 2);
    assert_eq!(
        opened
            .render_headless(&petunia_render::RenderOptions::default())
            .unwrap()
            .0,
        pixels
    );
    assert!(!opened.is_dirty());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn old_save_completion_keeps_new_edits_dirty_and_external_conflicts_keep_disk_intact() {
    let directory = directory();
    let path = directory.join("drawing.ptnd");
    let mut session = StudioSession::new("snapshot");
    session
        .insert_path("box", VectorPath::rect(2., 2., 4., 4.))
        .unwrap();
    let snapshot = session.save_snapshot();
    session
        .insert_path("new", VectorPath::rect(10., 10., 4., 4.))
        .unwrap();
    let completion = save_snapshot(
        &snapshot,
        &path,
        &SavePolicy::CreateNew,
        &PackageLimits::default(),
    )
    .unwrap();
    session.acknowledge_save(completion).unwrap();
    assert!(session.is_dirty());
    session.dispatch_action(UserAction::Undo).unwrap();
    assert!(!session.is_dirty());
    session.dispatch_action(UserAction::Redo).unwrap();
    std::fs::write(&path, b"external edit").unwrap();
    assert!(session.save().is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"external edit");
    let identity = session.document().id;
    let revision = session.revision();
    assert!(session.open(&path).is_err());
    assert_eq!(session.document().id, identity);
    assert_eq!(session.revision(), revision);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn view_rotation_pan_and_dpr_match_render_and_logical_pointer_coordinates() {
    let mut session = StudioSession::new("view");
    session
        .insert_path("box", VectorPath::rect(2., 2., 4., 4.))
        .unwrap();
    let object = session.document().scene.root_order()[0];
    let view = session.view_mut();
    view.viewport = Rect::new(0., 0., 16., 16.);
    view.dpr = 2.;
    view.rotation = std::f64::consts::FRAC_PI_2;
    view.pan_x = 14.;
    view.pan_y = 2.;
    let (pixels, _) = session
        .render_headless(&petunia_render::RenderOptions::default())
        .unwrap();
    assert_eq!(pixels.len(), 32 * 32 * 4);
    assert_eq!(
        &pixels[(12 * 32 + 20) * 4..(12 * 32 + 20) * 4 + 4],
        &[0, 0, 0, 255]
    );
    assert_eq!(
        &pixels[(4 * 32 + 4) * 4..(4 * 32 + 4) * 4 + 4],
        &[255, 255, 255, 255]
    );
    assert!(session
        .hit_test(Point::new(10., 6.))
        .iter()
        .any(|hit| matches!(hit,petunia_ui::HitTarget::Fill {object:id} if *id==object)));
}

#[test]
fn snapping_uses_document_coordinates_and_group_hits_follow_edit_context() {
    use petunia_ui::ToolServices;
    let mut session = StudioSession::new("spatial session");
    let page = session.active_page();
    let grid = GridDefinition::new(
        GridScope::Page(page),
        Point::new(0., 0.),
        GridSpec::Affine(
            AffineGridSpec::new(
                AffineGridKind::Cartesian,
                Vec2::new(10., 0.),
                Vec2::new(0., 10.),
                1,
                1,
                Tolerance(1e-6),
            )
            .unwrap(),
        ),
    );
    session
        .apply_edit(
            request(vec![DocumentOp::Registry(
                petunia_engine::transaction::RegistryOp::Grid {
                    id: grid.id,
                    value: Some(Box::new(grid)),
                },
            )]),
            HistoryDescription::EditObjects,
        )
        .unwrap();
    session.view_mut().scale = 2.;
    session.view_mut().pan_x = 100.;
    session.view_mut().rotation = 0.4;
    assert_eq!(session.snap(Point::new(9., 9.)).0, Point::new(10., 10.));
    session.view_mut().scale = 1.;
    session.view_mut().pan_x = 0.;
    session.view_mut().rotation = 0.;
    let mut group = SceneNode::new_path("group", VectorPath::new(), ParentRef::Page(page));
    group.item = SceneItem::Group(vec![]);
    group.transform = Transform2D::translation(16., 0.);
    let parent = group.id;
    let child = SceneNode::new_path(
        "child",
        VectorPath::rect(2., 2., 8., 8.),
        ParentRef::Object(parent),
    );
    let object = child.id;
    session
        .apply_edit(
            request(vec![
                DocumentOp::InsertRoot {
                    index: 0,
                    node: Box::new(group),
                },
                DocumentOp::InsertNode {
                    parent,
                    index: 0,
                    node: Box::new(child),
                },
            ]),
            HistoryDescription::InsertObjects,
        )
        .unwrap();
    assert!(
        matches!(session.hit_test(Point::new(21.,5.)).first(),Some(petunia_ui::HitTarget::Fill {object:id}) if *id==parent)
    );
    assert_eq!(
        session.marquee_select(Point::new(17., 1.), Point::new(27., 11.)),
        vec![parent]
    );
    session.on_double_click(Point::new(21., 5.));
    assert!(
        matches!(session.hit_test(Point::new(21.,5.)).first(),Some(petunia_ui::HitTarget::Fill {object:id}) if *id==object)
    );
    session
        .apply_edit(
            request(vec![DocumentOp::SetVisibility {
                object: parent,
                visible: false,
            }]),
            HistoryDescription::SetVisibility,
        )
        .unwrap();
    assert!(!session.is_editable(object));
    assert!(session.hit_test(Point::new(21., 5.)).is_empty());
}
