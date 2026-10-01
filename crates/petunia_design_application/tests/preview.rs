//! Latest-request preview regression sources; execution deferred to final MVP gate.
use petunia_design_application::{
    preview::{PreviewController, PreviewFrame, PreviewRequest, PreviewSource},
    view_camera::ViewportCamera,
};
use petunia_design_document::{ContainerRole, DocumentObject, ShapeKind, Surface};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_jobs::JobFailure;
use petunia_design_render::{CpuRenderer, PixelBufferRgba8, RenderSurface};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
fn surface(color: &str) -> Surface {
    let mut o = DocumentObject::new(ObjectId::new(2), "Art");
    o.bounds = Some([0.0, 0.0, 16.0, 16.0]);
    o.shape = Some(ShapeKind::Rectangle {
        corner_radii: [0.0; 4],
    });
    o.fill = Some(color.into());
    let mut surface = Surface::with_objects(SurfaceId::new(1), "Page", vec![o]);
    surface.dimensions = [32.0, 32.0];
    surface
}
fn request(s: &Surface, rev: u64) -> PreviewRequest {
    PreviewRequest::from_camera(
        PreviewSource::capture(s, rev),
        &ViewportCamera::new(32.0, 32.0),
        0,
        false,
    )
    .unwrap()
}
fn finish(c: &mut PreviewController) -> Arc<PreviewFrame> {
    let end = Instant::now() + Duration::from_secs(10);
    loop {
        c.poll();
        assert!(c.failure().is_none(), "{:?}", c.failure());
        if !c.is_pending() {
            return c.frame().unwrap();
        }
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(1));
    }
}
#[test]
fn preview_and_cpu_export_compose_the_same_snapshot_against_the_same_backdrop() {
    let s = surface("#ff0000");
    let r = request(&s, 1);
    let mut c = PreviewController::new().unwrap();
    c.request(Some(r.clone()));
    let frame = finish(&mut c);
    let scene = RenderSurface::extract(&s).unwrap();
    let backdrop = PixelBufferRgba8::with_fill(32, 32, [0xe2, 0xe4, 0xe8, 255]);
    let expected = CpuRenderer::default()
        .render_over(&scene, r.render, Some(&backdrop))
        .unwrap();
    assert_eq!(frame.pixels, expected);
}
#[test]
fn same_revision_and_surface_id_in_different_tabs_never_publish_old_pixels() {
    let a = request(&surface("#ff0000"), 0);
    let b = request(&surface("#0000ff"), 0);
    assert_ne!(a.source.id(), b.source.id());
    let mut c = PreviewController::new().unwrap();
    c.request(Some(a));
    finish(&mut c);
    c.request(Some(b.clone()));
    assert!(c.frame().is_none());
    let frame = finish(&mut c);
    assert_eq!(frame.request, b);
    assert_eq!(frame.pixels.get_pixel(5, 5).unwrap(), [0, 0, 255, 255]);
}
#[test]
fn pan_keeps_old_world_region_only_until_latest_camera_pixels_are_ready() {
    let mut r = request(&surface("#ff0000"), 5);
    let mut c = PreviewController::new().unwrap();
    c.request(Some(r.clone()));
    let previous = finish(&mut c);
    r.render.viewport = petunia_design_geometry::GRect::new(8.0, 0.0, 40.0, 32.0);
    c.request(Some(r.clone()));
    assert!(Arc::ptr_eq(&c.frame().unwrap(), &previous));
    let frame = finish(&mut c);
    assert_eq!(frame.request, r);
    assert!(!Arc::ptr_eq(&frame, &previous));
    assert_eq!(frame.pixels.get_pixel(25, 5).unwrap()[3], 0);
}
#[test]
fn channel_isolation_happens_after_composition_and_clears_old_display_mode() {
    let mut r = request(&surface("#123456"), 1);
    let mut c = PreviewController::new().unwrap();
    c.request(Some(r.clone()));
    finish(&mut c);
    r.channel = 2;
    c.request(Some(r));
    assert!(c.frame().is_none());
    assert_eq!(
        finish(&mut c).pixels.get_pixel(5, 5).unwrap(),
        [0x34, 0x34, 0x34, 255]
    );
}
#[test]
fn missing_icc_engine_is_a_presentation_reason_and_clear_cancels_work() {
    let mut r = request(&surface("#ff0000"), 1);
    r.soft_proof = true;
    let mut c = PreviewController::new().unwrap();
    c.request(Some(r));
    let end = Instant::now() + Duration::from_secs(5);
    while c.is_pending() {
        c.poll();
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(matches!(c.failure(),Some(JobFailure::Failed(reason)) if reason.contains("ICC")));
    assert!(c.frame().is_none());
    c.clear();
    assert!(!c.is_pending());
    assert!(c.failure().is_none());
}
#[test]
fn camera_admission_rejects_nonfinite_inputs_and_pixel_pressure() {
    let source = PreviewSource::capture(&surface("#ff0000"), 1);
    for (w, h, z) in [
        (f64::NAN, 32.0, 1.0),
        (32.0, 32.0, 0.0),
        (32.0, f64::INFINITY, 1.0),
        (8192.0, 8192.0, 1.0),
    ] {
        let mut camera = ViewportCamera::new(32.0, 32.0);
        camera.viewport_width = w;
        camera.viewport_height = h;
        camera.zoom = z;
        assert!(PreviewRequest::from_camera(source.clone(), &camera, 0, false).is_err());
    }
    assert!(
        PreviewRequest::from_camera(source, &ViewportCamera::new(32.0, 32.0), 5, false).is_err()
    );
}
#[test]
fn rapid_replacements_coalesce_to_the_last_request() {
    let source = PreviewSource::capture(&surface("#00ff00"), 1);
    let mut c = PreviewController::new().unwrap();
    let mut last = None;
    for i in 0..100 {
        let mut camera = ViewportCamera::new(32.0, 32.0);
        camera.pan_x = f64::from(i);
        let r = PreviewRequest::from_camera(source.clone(), &camera, 0, false).unwrap();
        c.request(Some(r.clone()));
        last = Some(r);
    }
    assert_eq!(finish(&mut c).request, last.unwrap());
}
#[test]
fn grouped_opacity_uses_the_shared_isolated_composition() {
    let mut group = DocumentObject::new(ObjectId::new(2), "Group");
    group.role = Some(ContainerRole::Group);
    group.bounds = Some([0.0, 0.0, 32.0, 32.0]);
    group.opacity = 0.5;
    group.children = vec![ObjectId::new(3), ObjectId::new(4)];
    let mut a = DocumentObject::new(ObjectId::new(3), "A");
    a.bounds = Some([0.0, 0.0, 20.0, 20.0]);
    a.shape = Some(ShapeKind::Rectangle {
        corner_radii: [0.0; 4],
    });
    a.fill = Some("#ff0000".into());
    a.parent = Some(group.id);
    let mut b = a.clone();
    b.id = ObjectId::new(4);
    b.bounds = Some([10.0, 0.0, 20.0, 20.0]);
    let s = Surface::with_objects(SurfaceId::new(1), "Page", vec![group, a, b]);
    let mut c = PreviewController::new().unwrap();
    c.request(Some(request(&s, 0)));
    let p = finish(&mut c);
    assert_eq!(p.pixels.get_pixel(5, 5), p.pixels.get_pixel(15, 5));
}
