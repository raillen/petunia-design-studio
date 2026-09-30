//! Tests for ViewportCamera, coordinate projections, snapping, and overlays.

use petunia_design_geometry::{GPoint, GRect};
use petunia_design_shell::canvas::*;

#[test]
fn camera_screen_and_document_coordinate_roundtrip() {
    let mut camera = ViewportCamera::new(1920.0, 1080.0);
    camera.pan(250.0, 120.0);
    camera.set_zoom(2.5);

    let doc_origin = GPoint::new(100.0, 200.0);
    let screen = camera.doc_to_screen(doc_origin);
    let roundtrip = camera.screen_to_doc(screen);

    assert!((roundtrip.x - doc_origin.x).abs() < 1e-6);
    assert!((roundtrip.y - doc_origin.y).abs() < 1e-6);
}

#[test]
fn camera_pointer_centered_zoom_invariant() {
    // 08.6: The document point directly under the pointer must remain
    // stationary in screen coordinates before and after zoom.
    let mut camera = ViewportCamera::new(1000.0, 800.0);
    camera.pan(100.0, 50.0);
    camera.set_zoom(1.2);

    let screen_cursor = GPoint::new(450.0, 320.0);
    let doc_before = camera.screen_to_doc(screen_cursor);

    // Zoom in by factor 1.5 centered at screen_cursor
    camera.zoom_at(screen_cursor, 1.5);

    let screen_after = camera.doc_to_screen(doc_before);
    assert!(
        (screen_after.x - screen_cursor.x).abs() < 1e-6,
        "X drift: {} vs {}",
        screen_after.x,
        screen_cursor.x
    );
    assert!(
        (screen_after.y - screen_cursor.y).abs() < 1e-6,
        "Y drift: {} vs {}",
        screen_after.y,
        screen_cursor.y
    );
}

#[test]
fn camera_zoom_bounds_clamped() {
    let mut camera = ViewportCamera::new(800.0, 600.0);
    camera.set_zoom(0.00001);
    assert_eq!(camera.zoom, MIN_ZOOM);

    camera.set_zoom(9999.0);
    assert_eq!(camera.zoom, MAX_ZOOM);
}

#[test]
fn camera_fit_rect_centers_target_comfortably() {
    let mut camera = ViewportCamera::new(1000.0, 1000.0);
    let doc_rect = GRect::new(100.0, 100.0, 600.0, 600.0);

    camera.fit_rect(doc_rect, 50.0); // 900x900 available
                                     // scale should be 900 / 500 = 1.8
    assert!((camera.zoom - 1.8).abs() < 1e-4);

    let center_doc = GPoint::new(350.0, 350.0);
    let center_screen = camera.doc_to_screen(center_doc);
    assert!((center_screen.x - 500.0).abs() < 1e-4);
    assert!((center_screen.y - 500.0).abs() < 1e-4);
}

#[test]
fn snap_engine_grid_and_guides() {
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut engine = SnapEngine::new();
    engine.config.grid_spacing = 20.0;
    engine.config.tolerance_px = 8.0;

    // Point near grid line x = 40.0 (e.g. 42.0)
    let candidate = GPoint::new(42.0, 15.0);
    let res = engine.snap_point(candidate, &camera, &[]);
    assert!(res.snapped_x);
    assert_eq!(res.point.x, 40.0);
    assert_eq!(res.point.y, 20.0); // 15.0 rounds to 20.0
}

#[test]
fn snap_engine_hysteresis_prevents_jitter() {
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let mut engine = SnapEngine::new();
    engine.guides_x.push(100.0);
    engine.config.tolerance_px = 5.0;
    engine.config.hysteresis_px = 5.0; // total release = 10.0
    engine.config.grid_enabled = false;

    // Frame 1: Acquired snap (dist = 3.0 <= 5.0)
    let p1 = GPoint::new(103.0, 0.0);
    let r1 = engine.snap_point(p1, &camera, &[]);
    assert!(r1.snapped_x);
    assert_eq!(r1.point.x, 100.0);

    // Frame 2: Moved to dist = 7.0 (> tolerance 5.0, but <= tolerance + hysteresis 10.0)
    let p2 = GPoint::new(107.0, 0.0);
    let r2 = engine.snap_point(p2, &camera, &[]);
    assert!(r2.snapped_x, "Hysteresis should retain snap at 7.0px");
    assert_eq!(r2.point.x, 100.0);

    // Frame 3: Moved to dist = 12.0 (> 10.0) -> snap released
    let p3 = GPoint::new(112.0, 0.0);
    let r3 = engine.snap_point(p3, &camera, &[]);
    assert!(!r3.snapped_x, "Snap should release past hysteresis");
    assert_eq!(r3.point.x, 112.0);
}

#[test]
fn selection_handles_geometry_and_hit_testing() {
    let camera = ViewportCamera::new(1000.0, 1000.0);
    let bounds = GRect::new(100.0, 100.0, 300.0, 250.0);
    let handles = compute_selection_handles(bounds, &camera, 10.0);

    assert_eq!(handles.len(), 9); // 8 box handles + 1 rotation handle

    // Top-left handle is at (100, 100)
    let tl = &handles[0];
    assert_eq!(tl.kind, SelectionHandleKind::TopLeft);
    assert!(tl.hit_test(GPoint::new(100.0, 100.0)));
    assert!(tl.hit_test(GPoint::new(104.0, 104.0)));
    assert!(!tl.hit_test(GPoint::new(115.0, 115.0)));
}

#[test]
fn oriented_selection_handles_and_hit_testing() {
    let camera = ViewportCamera::new(1000.0, 1000.0);
    // Square 100x100 at origin (100, 100) rotated by 90 degrees (pi/2)
    let rot = std::f64::consts::FRAC_PI_2;
    let transform = petunia_design_geometry::GAffine::translate(100.0, 100.0)
        .after(petunia_design_geometry::GAffine::rotate(rot));
    let handles =
        compute_selection_handles_oriented([0.0, 0.0, 100.0, 100.0], transform, &camera, 10.0);

    assert_eq!(handles.len(), 9);
    // TopLeft local (0, 0) -> (100, 100)
    let tl = &handles[0];
    assert_eq!(tl.kind, SelectionHandleKind::TopLeft);
    assert!((tl.doc_point.x - 100.0).abs() < 1e-4);
    assert!((tl.doc_point.y - 100.0).abs() < 1e-4);

    // TopRight local (100, 0) rotated 90 deg:
    // (100*cos(90) - 0*sin(90) + 100, 100*sin(90) + 0*cos(90) + 100) = (100, 200)
    let tr = handles
        .iter()
        .find(|h| h.kind == SelectionHandleKind::TopRight)
        .unwrap();
    assert!((tr.doc_point.x - 100.0).abs() < 1e-4);
    assert!((tr.doc_point.y - 200.0).abs() < 1e-4);

    // Hit test oriented handle at (100, 200)
    let hit = hit_test_handle_or_border_oriented(
        [0.0, 0.0, 100.0, 100.0],
        transform,
        GPoint::new(100.0, 200.0),
        GPoint::new(100.0, 200.0),
        &camera,
        10.0,
        8.0,
    );
    assert_eq!(hit, Some(SelectionHandleKind::TopRight));
}
