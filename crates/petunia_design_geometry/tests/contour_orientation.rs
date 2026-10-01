//! Glyph contour normalization support; regression sources not executed yet.
use petunia_design_geometry::boolean::FillRule;
use petunia_design_geometry::{GPath, GPoint, GRect, PathVerb};
#[test]
fn reversing_compound_contours_preserves_nonzero_counters() {
    let mut p = GPath::rect(GRect::new(0.0, 0.0, 20.0, 20.0), 0.0, 0.0);
    p.verbs.extend(
        GPath::rect(GRect::new(5.0, 5.0, 15.0, 15.0), 0.0, 0.0)
            .reversed_contours()
            .verbs,
    );
    let r = p.reversed_contours();
    assert!((p.signed_area() + r.signed_area()).abs() < 1e-9);
    for point in [
        GPoint::new(2.0, 2.0),
        GPoint::new(10.0, 10.0),
        GPoint::new(30.0, 30.0),
    ] {
        assert_eq!(
            p.contains_point_with_fill(point, 0.01, FillRule::NonZero),
            r.contains_point_with_fill(point, 0.01, FillRule::NonZero)
        );
    }
    assert!(!r.contains_point_with_fill(GPoint::new(10.0, 10.0), 0.01, FillRule::NonZero));
}
#[test]
fn curve_reversal_preserves_bounds_length_and_closure() {
    let p = GPath {
        verbs: vec![
            PathVerb::MoveTo(GPoint::new(0.0, 0.0)),
            PathVerb::QuadTo(GPoint::new(10.0, -5.0), GPoint::new(20.0, 0.0)),
            PathVerb::CubicTo(
                GPoint::new(20.0, 10.0),
                GPoint::new(0.0, 10.0),
                GPoint::new(0.0, 0.0),
            ),
            PathVerb::Close,
        ],
    };
    let r = p.reversed_contours();
    let a = p.bounding_box().unwrap();
    let b = r.bounding_box().unwrap();
    assert!([a.x0 - b.x0, a.y0 - b.y0, a.x1 - b.x1, a.y1 - b.y1]
        .iter()
        .all(|d| d.abs() < 1e-9));
    assert!((p.approx_length(0.001) - r.approx_length(0.001)).abs() < 1e-5);
    assert!((p.signed_area() + r.signed_area()).abs() < 1e-9);
    assert!(r.verbs.contains(&PathVerb::Close));
    assert!(r.verbs.iter().any(|v| matches!(v, PathVerb::QuadTo(..))));
    assert!(r.verbs.iter().any(|v| matches!(v, PathVerb::CubicTo(..))));
}
#[test]
fn reversing_open_subpaths_does_not_add_closing_edges() {
    let p = GPath {
        verbs: vec![
            PathVerb::MoveTo(GPoint::new(0.0, 0.0)),
            PathVerb::LineTo(GPoint::new(10.0, 10.0)),
            PathVerb::MoveTo(GPoint::new(30.0, 0.0)),
            PathVerb::LineTo(GPoint::new(40.0, 10.0)),
        ],
    };
    let r = p.reversed_contours();
    assert!(!r.verbs.contains(&PathVerb::Close));
    assert_eq!(r.verbs.len(), p.verbs.len());
    assert_eq!(p.bounding_box(), r.bounding_box());
}
