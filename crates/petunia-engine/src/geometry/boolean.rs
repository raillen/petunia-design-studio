//! Polygon boolean operations through an encapsulated adapter.
//!
//! Petunia [`Point`] values cross into the boolean library and back;
//! the external engine type never leaks into our API. Curves flatten
//! before booleans: exact topology comes from the integer kernel,
//! curve fidelity from the flatten tolerance.

use crate::geometry::bezier::{flatten_contour, CubicBez};
use crate::geometry::bounds::Bounds;
use i_overlay::core::fill_rule::FillRule as OverlayFillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::single::SingleFloatOverlay;
use petunia_core::{Contour, FillRule, PathNode, Point, Tolerance, VectorPath};

/// Boolean set operation over flattened polygons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp {
    Union,
    Intersection,
    Difference,
    Exclusion,
}

fn overlay_rule(op: BooleanOp) -> OverlayRule {
    match op {
        BooleanOp::Union => OverlayRule::Union,
        BooleanOp::Intersection => OverlayRule::Intersect,
        BooleanOp::Difference => OverlayRule::Difference,
        BooleanOp::Exclusion => OverlayRule::Xor,
    }
}

fn overlay_fill_rule(rule: FillRule) -> OverlayFillRule {
    match rule {
        FillRule::NonZero => OverlayFillRule::NonZero,
        FillRule::EvenOdd => OverlayFillRule::EvenOdd,
    }
}

/// Boolean combination of polygon rings, preserving holes: each
/// result shape holds its contours (outer plus holes). Rings are
/// plain point loops; closure is implied.
#[must_use]
pub fn boolean_rings(
    subject: &[Vec<Point>],
    clip: &[Vec<Point>],
    op: BooleanOp,
    rule: FillRule,
) -> Vec<Vec<Vec<Point>>> {
    let subject: Vec<Vec<[f64; 2]>> = subject
        .iter()
        .map(|ring| ring.iter().map(|point| [point.x, point.y]).collect())
        .collect();
    let clip: Vec<Vec<[f64; 2]>> = clip
        .iter()
        .map(|ring| ring.iter().map(|point| [point.x, point.y]).collect())
        .collect();
    subject
        .overlay(&clip, overlay_rule(op), overlay_fill_rule(rule))
        .into_iter()
        .map(|shape| {
            shape
                .into_iter()
                .map(|contour| {
                    contour
                        .into_iter()
                        .map(|point| Point::new(point[0], point[1]))
                        .collect()
                })
                .collect()
        })
        .collect()
}

/// Boolean combination of two paths: flatten with `tolerance`, run
/// the integer kernel, rebuild cusp-node contours. Result contours
/// are closed; open inputs flatten as polylines first.
#[must_use]
pub fn boolean_paths(
    subject: &VectorPath,
    clip: &VectorPath,
    op: BooleanOp,
    rule: FillRule,
    tolerance: Tolerance,
) -> VectorPath {
    let subject_rings: Vec<Vec<Point>> = subject
        .contours
        .iter()
        .map(|contour| flatten_contour(contour, tolerance))
        .filter(|ring| ring.len() >= 3)
        .collect();
    let clip_rings: Vec<Vec<Point>> = clip
        .contours
        .iter()
        .map(|contour| flatten_contour(contour, tolerance))
        .filter(|ring| ring.len() >= 3)
        .collect();
    let mut result = VectorPath::new();
    result.fill_rule = rule;
    for shape in boolean_rings(&subject_rings, &clip_rings, op, rule) {
        for ring in shape {
            if ring.len() < 3 {
                continue;
            }
            let mut contour = Contour::new(true);
            for point in ring {
                contour.push_node(PathNode::new(point, petunia_core::NodeKind::Cusp));
            }
            result.push_contour(contour);
        }
    }
    result
}

/// Bounds of one flattened ring, or `None` when degenerate.
#[must_use]
pub fn ring_bounds(ring: &[Point]) -> Option<Bounds> {
    Bounds::of_points(ring)
}

/// Flatten one cubic segment for consumers that need polylines.
#[must_use]
pub fn flatten_cubic(curve: CubicBez, tolerance: Tolerance) -> Vec<Point> {
    curve.flatten(tolerance)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tolerance() -> Tolerance {
        Tolerance::new(0.01).expect("valid")
    }

    fn square(x: f64, y: f64, size: f64) -> Vec<Point> {
        vec![
            Point::new(x, y),
            Point::new(x + size, y),
            Point::new(x + size, y + size),
            Point::new(x, y + size),
        ]
    }

    fn area(shapes: &[Vec<Vec<Point>>]) -> f64 {
        shapes
            .iter()
            .flat_map(|shape| shape.iter())
            .map(|ring| {
                (0..ring.len())
                    .map(|index| {
                        let a = ring[index];
                        let b = ring[(index + 1) % ring.len()];
                        a.x * b.y - b.x * a.y
                    })
                    .sum::<f64>()
                    .abs()
                    / 2.0
            })
            .sum()
    }

    #[test]
    fn union_merges_overlapping_squares() {
        let result = boolean_rings(
            &[square(0.0, 0.0, 10.0)],
            &[square(5.0, 0.0, 10.0)],
            BooleanOp::Union,
            FillRule::NonZero,
        );
        assert!((area(&result) - 150.0).abs() < 1e-6, "area {result:?}");
    }

    #[test]
    fn intersection_and_difference_match_areas() {
        let subject = vec![square(0.0, 0.0, 10.0)];
        let clip = vec![square(5.0, 0.0, 10.0)];
        let hit = boolean_rings(&subject, &clip, BooleanOp::Intersection, FillRule::NonZero);
        assert!((area(&hit) - 50.0).abs() < 1e-6);
        let rest = boolean_rings(&subject, &clip, BooleanOp::Difference, FillRule::NonZero);
        assert!((area(&rest) - 50.0).abs() < 1e-6);
        let xor = boolean_rings(&subject, &clip, BooleanOp::Exclusion, FillRule::NonZero);
        assert!((area(&xor) - 100.0).abs() < 1e-6);
    }

    #[test]
    fn disjoint_inputs_union_without_loss() {
        let result = boolean_rings(
            &[square(0.0, 0.0, 10.0)],
            &[square(20.0, 20.0, 10.0)],
            BooleanOp::Union,
            FillRule::NonZero,
        );
        assert!((area(&result) - 200.0).abs() < 1e-6);
    }

    #[test]
    fn path_boolean_rebuilds_closed_contours() {
        let mut subject = VectorPath::new();
        let mut contour = Contour::new(true);
        for point in square(0.0, 0.0, 10.0) {
            contour.push_node(PathNode::new(point, petunia_core::NodeKind::Cusp));
        }
        subject.push_contour(contour);
        let mut clip = VectorPath::new();
        let mut contour = Contour::new(true);
        for point in square(5.0, 5.0, 10.0) {
            contour.push_node(PathNode::new(point, petunia_core::NodeKind::Cusp));
        }
        clip.push_contour(contour);
        let result = boolean_paths(
            &subject,
            &clip,
            BooleanOp::Union,
            FillRule::NonZero,
            tolerance(),
        );
        assert!(!result.contours.is_empty());
        assert!(result.contours.iter().all(|contour| contour.closed));
    }
}
