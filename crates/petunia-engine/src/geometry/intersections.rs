//! Segment predicates and polyline intersections.
//!
//! Orientation drives every topological question here. Results are
//! discrete (`Clockwise`, `CounterClockwise`, `Collinear`); callers
//! near zero treat the outcome as fragile and prefer the robust
//! boolean library downstream for topology.

use petunia_core::Point;

/// Discrete orientation of the turn `a → b → c`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Clockwise,
    CounterClockwise,
    Collinear,
}

/// Orientation of three points by signed area.
#[must_use]
pub fn orient(a: Point, b: Point, c: Point) -> Orientation {
    let area = (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y);
    if area > 0.0 {
        Orientation::CounterClockwise
    } else if area < 0.0 {
        Orientation::Clockwise
    } else {
        Orientation::Collinear
    }
}

/// How two closed segments meet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SegmentIntersection {
    /// A single meeting point (crossing, touch or T-junction).
    Point(Point),
    /// Collinear overlap along more than a point.
    Overlap(Point, Point),
}

/// Intersection of two closed segments, or `None` when disjoint.
#[must_use]
pub fn segments_intersect(
    p1: Point,
    p2: Point,
    p3: Point,
    p4: Point,
) -> Option<SegmentIntersection> {
    let o1 = orient(p1, p2, p3);
    let o2 = orient(p1, p2, p4);
    let o3 = orient(p3, p4, p1);
    let o4 = orient(p3, p4, p2);

    // Proper crossing.
    if o1 != o2
        && o3 != o4
        && o1 != Orientation::Collinear
        && o2 != Orientation::Collinear
        && o3 != Orientation::Collinear
        && o4 != Orientation::Collinear
    {
        return Some(SegmentIntersection::Point(crossing_point(p1, p2, p3, p4)));
    }

    // Endpoint touches and collinear overlaps.
    let mut touches = Vec::new();
    for (point, on_a, on_b) in [
        (p3, on_segment(p1, p2, p3), true),
        (p4, on_segment(p1, p2, p4), true),
        (p1, true, on_segment(p3, p4, p1)),
        (p2, true, on_segment(p3, p4, p2)),
    ] {
        if on_a && on_b {
            touches.push(point);
        }
    }
    touches.dedup_by(|a, b| a.x == b.x && a.y == b.y);
    match touches.len() {
        0 => None,
        1 => Some(SegmentIntersection::Point(touches[0])),
        _ => {
            let (mut first, mut second) = (touches[0], touches[1]);
            if second.x < first.x || (second.x == first.x && second.y < first.y) {
                std::mem::swap(&mut first, &mut second);
            }
            if first == second {
                Some(SegmentIntersection::Point(first))
            } else {
                Some(SegmentIntersection::Overlap(first, second))
            }
        }
    }
}

fn on_segment(a: Point, b: Point, point: Point) -> bool {
    point.x >= a.x.min(b.x)
        && point.x <= a.x.max(b.x)
        && point.y >= a.y.min(b.y)
        && point.y <= a.y.max(b.y)
        && orient(a, b, point) == Orientation::Collinear
}

fn crossing_point(p1: Point, p2: Point, p3: Point, p4: Point) -> Point {
    let denom = (p1.x - p2.x) * (p3.y - p4.y) - (p1.y - p2.y) * (p3.x - p4.x);
    if denom == 0.0 {
        return Point::new((p1.x + p3.x) / 2.0, (p1.y + p3.y) / 2.0);
    }
    let x = ((p1.x * p2.y - p1.y * p2.x) * (p3.x - p4.x)
        - (p1.x - p2.x) * (p3.x * p4.y - p3.y * p4.x))
        / denom;
    let y = ((p1.x * p2.y - p1.y * p2.x) * (p3.y - p4.y)
        - (p1.y - p2.y) * (p3.x * p4.y - p3.y * p4.x))
        / denom;
    Point::new(x, y)
}

/// All pairwise intersections between two polylines, in scan order.
#[must_use]
pub fn polyline_intersections(a: &[Point], b: &[Point]) -> Vec<SegmentIntersection> {
    let mut hits = Vec::new();
    for pair_a in a.windows(2) {
        for pair_b in b.windows(2) {
            if let Some(hit) = segments_intersect(pair_a[0], pair_a[1], pair_b[0], pair_b[1]) {
                hits.push(hit);
            }
        }
    }
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(x: f64, y: f64) -> Point {
        Point::new(x, y)
    }

    #[test]
    fn orientation_reports_turns() {
        assert_eq!(
            orient(pt(0.0, 0.0), pt(1.0, 0.0), pt(1.0, 1.0)),
            Orientation::CounterClockwise
        );
        assert_eq!(
            orient(pt(0.0, 0.0), pt(1.0, 0.0), pt(1.0, -1.0)),
            Orientation::Clockwise
        );
        assert_eq!(
            orient(pt(0.0, 0.0), pt(1.0, 1.0), pt(2.0, 2.0)),
            Orientation::Collinear
        );
    }

    #[test]
    fn crossing_touching_and_parallel_cases() {
        // Proper crossing at (1, 1).
        match segments_intersect(pt(0.0, 0.0), pt(2.0, 2.0), pt(0.0, 2.0), pt(2.0, 0.0)) {
            Some(SegmentIntersection::Point(hit)) => {
                assert!((hit.x - 1.0).abs() < 1e-9 && (hit.y - 1.0).abs() < 1e-9);
            }
            other => panic!("expected crossing point, got {other:?}"),
        }
        // Shared endpoint.
        assert!(matches!(
            segments_intersect(pt(0.0, 0.0), pt(1.0, 1.0), pt(1.0, 1.0), pt(2.0, 0.0)),
            Some(SegmentIntersection::Point(_))
        ));
        // Strictly parallel disjoint segments.
        assert_eq!(
            segments_intersect(pt(0.0, 0.0), pt(1.0, 0.0), pt(0.0, 1.0), pt(1.0, 1.0)),
            None
        );
    }

    #[test]
    fn collinear_overlap_reports_span() {
        match segments_intersect(pt(0.0, 0.0), pt(4.0, 0.0), pt(2.0, 0.0), pt(6.0, 0.0)) {
            Some(SegmentIntersection::Overlap(a, b)) => {
                assert_eq!((a.x, b.x), (2.0, 4.0));
            }
            other => panic!("expected overlap span, got {other:?}"),
        }
    }
}
