//! Property-based tests for Viewport Camera math, snapping determinism, and headless MockGuiAdapter.

use petunia_design_document::Document;
use petunia_design_geometry::{GPoint, GRect};
use petunia_design_shell::canvas::{SnapEngine, ViewportCamera};
use petunia_design_shell::shell::MockGuiAdapter;
use proptest::prelude::*;

proptest! {
    #[test]
    fn camera_coordinate_transformation_always_invertible(
        screen_w in 100.0..3840.0f64,
        screen_h in 100.0..2160.0f64,
        pan_x in -5000.0..5000.0f64,
        pan_y in -5000.0..5000.0f64,
        zoom in 0.01..50.0f64,
        doc_x in -10000.0..10000.0f64,
        doc_y in -10000.0..10000.0f64,
    ) {
        let mut camera = ViewportCamera::new(screen_w, screen_h);
        camera.pan_x = pan_x;
        camera.pan_y = pan_y;
        camera.zoom = zoom;

        let doc_pt = GPoint::new(doc_x, doc_y);
        let screen_pt = camera.doc_to_screen(doc_pt);
        let roundtrip = camera.screen_to_doc(screen_pt);

        prop_assert!((roundtrip.x - doc_pt.x).abs() < 1e-4);
        prop_assert!((roundtrip.y - doc_pt.y).abs() < 1e-4);
    }

    #[test]
    fn snapping_produces_finite_coordinates_and_valid_guides(
        cand_x in -1000.0..1000.0f64,
        cand_y in -1000.0..1000.0f64,
        grid_spacing in 5.0..100.0f64,
    ) {
        let camera = ViewportCamera::new(1920.0, 1080.0);
        let mut engine = SnapEngine::new();
        engine.config.grid_spacing = grid_spacing;

        let pt = GPoint::new(cand_x, cand_y);
        let res = engine.snap_point(pt, &camera, &[GRect::new(50.0, 50.0, 150.0, 150.0)]);

        prop_assert!(res.point.x.is_finite());
        prop_assert!(res.point.y.is_finite());

        for g in res.guides {
            prop_assert!(g.position.is_finite());
            prop_assert!(g.span_start.is_finite());
            prop_assert!(g.span_end.is_finite());
        }
    }

    #[test]
    fn mock_gui_adapter_always_headless_compliant_on_arbitrary_doc(
        num_surfaces in 1..5usize,
    ) {
        let mut adapter = MockGuiAdapter::new();
        let mut doc = Document::default();
        let mut gen = petunia_design_foundation::IdGenerator::new();

        for s in 0..num_surfaces {
            let surface = petunia_design_document::Surface::new(
                gen.next_surface(),
                format!("Surface {s}"),
            );
            petunia_design_document::DocumentMutator::new(&mut doc)
                .add_surface(surface.id, surface.name.clone())
                .unwrap();
        }

        adapter.open_session("Test Doc", doc).unwrap();
        prop_assert!(adapter.is_headless_compliant());

        let snap = adapter.resolve_snapshot();
        prop_assert_eq!(snap.surface_count, num_surfaces);

        let actions = adapter.enumerate_actions();
        prop_assert!(!actions.states.is_empty());
    }
}
