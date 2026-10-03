use petunia_design_application::{
    raster_edit::{RasterBrush, RasterStroke},
    ActionId, ActionRequest, Command, CommandRequest, DocumentSession,
};
use petunia_design_color::IccProfile;
use petunia_design_document::{Document, DocumentMutator, DocumentObject, ShapeKind};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_geometry::GPoint;
use petunia_design_raster::{BitDepth, RasterLayer};
use std::sync::Arc;
fn press() -> IccProfile {
    IccProfile::new(
        "Synthetic press".into(),
        Arc::new(include_bytes!("../../../fixtures/color/synthetic-cmyk.icc").to_vec()),
    )
    .unwrap()
}
fn session(with_layer: bool) -> DocumentSession {
    let mut document = Document::new();
    let mut m = DocumentMutator::new(&mut document);
    m.add_surface(SurfaceId::new(1), "Page").unwrap();
    m.set_surface_geometry(SurfaceId::new(1), [0.; 2], [16.; 2])
        .unwrap();
    if with_layer {
        let mut object = DocumentObject::new(ObjectId::new(2), "Ink");
        object.bounds = Some([0., 0., 16., 16.]);
        object.shape = Some(ShapeKind::Raster {
            layer: Arc::new(RasterLayer::new_cmyk(16, 16, BitDepth::Sixteen, press()).unwrap()),
        });
        m.add_object(SurfaceId::new(1), object).unwrap();
    }
    let mut session = DocumentSession::with_document("Ink", document);
    if with_layer {
        session.selection.select_exact(vec![ObjectId::new(2)]);
    }
    session
}
fn ink(session: &DocumentSession) -> Arc<RasterLayer> {
    let Some(ShapeKind::Raster { layer }) = &session.document().surfaces()[0].objects()[0].shape
    else {
        panic!("native layer")
    };
    layer.clone()
}
fn brush() -> RasterBrush {
    RasterBrush {
        radius: 2.,
        hardness: 1.,
        flow: 1.,
        opacity: 0.5,
        color: [0., 0., 0., 1.],
        ink: Some([0., 0., 0., 1.]),
        erase: false,
    }
}
#[test]
fn native_layer_creation_is_one_reversible_action_and_requires_icc() {
    let mut session = session(false);
    let action = ActionRequest::without_payload(ActionId::new(ActionId::RASTER_CREATE_CMYK));
    assert!(session.dispatch_action(action.clone()).is_err());
    assert!(session.history().undo_entries().is_empty());
    session
        .execute_command(CommandRequest::new(Command::SetSurfaceCmykProfile {
            surface: SurfaceId::new(1),
            profile: Some(press()),
        }))
        .unwrap();
    let before = session.document().clone();
    session.dispatch_action(action).unwrap();
    assert!(ink(&session).is_cmyk());
    assert_eq!(ink(&session).tiles().format.bit_depth(), BitDepth::Sixteen);
    let after = session.document().clone();
    assert_eq!(session.selection.selected_ids.len(), 1);
    session.undo().unwrap();
    assert_eq!(session.document(), &before);
    session.redo().unwrap();
    assert_eq!(session.document(), &after);
}
#[test]
fn pure_black_paint_and_eraser_keep_separate_alpha_and_one_gesture_history() {
    let mut session = session(true);
    let before = session.document().clone();
    let mut stroke = RasterStroke::begin(&session, brush()).unwrap();
    stroke.sample(GPoint::new(4., 4.), 1.).unwrap();
    stroke.sample(GPoint::new(8., 4.), 1.).unwrap();
    assert_eq!(session.document(), &before);
    stroke.commit(&mut session).unwrap();
    assert_eq!(session.history().undo_entries().len(), 1);
    let painted = ink(&session).cmyka_pixel(5, 4).unwrap();
    assert_eq!(&painted[..4], &[0., 0., 0., 1.]);
    assert!((painted[4] - 0.5).abs() < 0.00002);
    let mut eraser = brush();
    eraser.erase = true;
    let mut stroke = RasterStroke::begin(&session, eraser).unwrap();
    stroke.sample(GPoint::new(5., 4.), 1.).unwrap();
    stroke.commit(&mut session).unwrap();
    let erased = ink(&session).cmyka_pixel(5, 4).unwrap();
    assert_eq!(&erased[..4], &painted[..4]);
    assert!((erased[4] - 0.25).abs() < 0.00003);
    session.undo().unwrap();
    assert_eq!(ink(&session).cmyka_pixel(5, 4).unwrap(), painted);
    session.undo().unwrap();
    assert_eq!(session.document(), &before);
}
#[test]
fn flood_fill_preserves_process_black_and_cancelled_fill_never_publishes() {
    let mut session = session(true);
    let before = session.document().clone();
    let mut cancelled = RasterStroke::begin(&session, brush()).unwrap();
    assert_eq!(
        cancelled
            .flood_fill(GPoint::new(1., 1.), 0., &|| true)
            .unwrap_err()
            .code()
            .0,
        "ptnd.cancelled"
    );
    assert!(cancelled.commit(&mut session).is_err());
    assert_eq!(session.document(), &before);
    let mut stroke = RasterStroke::begin(&session, brush()).unwrap();
    stroke
        .flood_fill(GPoint::new(1., 1.), 0., &|| false)
        .unwrap();
    stroke.commit(&mut session).unwrap();
    for (x, y) in [(0, 0), (15, 15)] {
        let p = ink(&session).cmyka_pixel(x, y).unwrap();
        assert_eq!(&p[..4], &[0., 0., 0., 1.]);
        assert!((p[4] - 0.5).abs() < 0.00002);
    }
    assert_eq!(session.history().undo_entries().len(), 1);
    session.undo().unwrap();
    assert_eq!(session.document(), &before);
}
#[test]
fn profile_assignment_undo_preserves_tiles_and_stale_strokes_are_rejected() {
    let mut session = session(true);
    let mut stroke = RasterStroke::begin(&session, brush()).unwrap();
    stroke.sample(GPoint::new(4., 4.), 1.).unwrap();
    stroke.commit(&mut session).unwrap();
    let original = ink(&session);
    let mut stale = RasterStroke::begin(&session, brush()).unwrap();
    stale.sample(GPoint::new(8., 4.), 1.).unwrap();
    let profile = IccProfile::new(
        "Assigned profile name".into(),
        Arc::new(press().bytes().to_vec()),
    )
    .unwrap();
    session
        .execute_command(CommandRequest::new(Command::AssignRasterCmykProfile {
            id: ObjectId::new(2),
            profile,
        }))
        .unwrap();
    let assigned = ink(&session);
    assert!(Arc::ptr_eq(
        &original.tiles().tiles().next().unwrap().1.data,
        &assigned.tiles().tiles().next().unwrap().1.data
    ));
    assert_eq!(
        original.cmyka_bytes(&|| false).unwrap(),
        assigned.cmyka_bytes(&|| false).unwrap()
    );
    assert!(stale.commit(&mut session).is_err());
    session.undo().unwrap();
    assert_eq!(ink(&session), original);
}
#[test]
fn rgb_brush_is_converted_once_and_native_process_proof_is_explicitly_blocked() {
    let session = session(true);
    let mut b = brush();
    b.ink = None;
    b.color = [0.6, 0.3, 0.1, 1.];
    assert!(RasterStroke::begin(&session, b).is_ok());
    assert!(petunia_design_application::preview::has_native_ink(
        &session.document().surfaces()[0]
    ));
    use petunia_design_application::preview::{PreviewController, PreviewRequest, PreviewSource};
    let camera = petunia_design_application::view_camera::ViewportCamera::new(16., 16.);
    let mut request = PreviewRequest::from_camera(
        PreviewSource::capture(&session.document().surfaces()[0], 0),
        &camera,
        0,
        true,
    )
    .unwrap();
    request.proof_settings = Some(petunia_design_color::IccProofSettings {
        proof: press(),
        monitor: IccProfile::srgb().unwrap(),
        options: Default::default(),
        proof_intent: petunia_design_color::RenderingIntent::RelativeColorimetric,
    });
    let mut controller = PreviewController::new().unwrap();
    controller.request(Some(request));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while controller.is_pending() {
        controller.poll();
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert!(controller.failure().is_some());
    assert!(controller.frame().is_none());
    assert!(format!("{:?}", controller.failure()).contains("native ink proof"));
}
