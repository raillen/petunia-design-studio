//! Conservative bounds and point-in-polygon queries.
//!
//! Bounds are broad phase: cheap, conservative rectangles over
//! derived geometry. Exact containment honors the fill rule over
//! flattened rings.

use petunia_core::{FillRule, Point};

/// Axis-aligned bounds over derived geometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub min: Point,
    pub max: Point,
}

impl Bounds {
    /// Bounds of one point.
    #[must_use]
    pub fn point(point: Point) -> Self {
        Self {
            min: point,
            max: point,
        }
    }

    /// Smallest bounds holding every point, or `None` when empty.
    #[must_use]
    pub fn of_points(points: &[Point]) -> Option<Self> {
        let first = points.first()?;
        let mut bounds = Self::point(*first);
        for point in &points[1..] {
            bounds = bounds.union(&Self::point(*point));
        }
        Some(bounds)
    }

    /// Smallest bounds holding both.
    #[must_use]
    pub fn union(self, other: &Self) -> Self {
        Self {
            min: Point::new(self.min.x.min(other.min.x), self.min.y.min(other.min.y)),
            max: Point::new(self.max.x.max(other.max.x), self.max.y.max(other.max.y)),
        }
    }

    /// Padded bounds for stroke/effect-aware queries.
    #[must_use]
    pub fn padded(self, padding: f64) -> Self {
        Self {
            min: Point::new(self.min.x - padding, self.min.y - padding),
            max: Point::new(self.max.x + padding, self.max.y + padding),
        }
    }

    /// Inclusive containment.
    #[must_use]
    pub fn contains(self, point: Point) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// True when the rectangles touch or overlap.
    #[must_use]
    pub fn overlaps(self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }
}

/// Exact point-in-ring test over a flattened polygon. `NonZero` uses
/// the winding number, `EvenOdd` the crossing parity. The ring is
/// treated cyclically; a duplicated closing anchor is harmless.
#[must_use]
pub fn point_in_polygon(point: Point, ring: &[Point], rule: FillRule) -> bool {
    if ring.len() < 3 {
        return false;
    }
    let mut winding = 0i32;
    for index in 0..ring.len() {
        let a = ring[index];
        let b = ring[(index + 1) % ring.len()];
        if a.y <= point.y {
            if b.y > point.y && cross(a, b, point) > 0.0 {
                winding += 1;
            }
        } else if b.y <= point.y && cross(a, b, point) < 0.0 {
            winding -= 1;
        }
    }
    match rule {
        FillRule::NonZero => winding != 0,
        FillRule::EvenOdd => winding % 2 != 0,
    }
}

fn cross(a: Point, b: Point, point: Point) -> f64 {
    (b.x - a.x) * (point.y - a.y) - (point.x - a.x) * (b.y - a.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Vec<Point> {
        vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ]
    }

    #[test]
    fn bounds_union_and_overlap() {
        let bounds = Bounds::of_points(&square()).expect("non-empty");
        assert_eq!(bounds.min, Point::new(0.0, 0.0));
        assert_eq!(bounds.max, Point::new(10.0, 10.0));
        assert!(bounds.contains(Point::new(5.0, 5.0)));
        assert!(!bounds.contains(Point::new(11.0, 5.0)));
        let far = Bounds::of_points(&[Point::new(50.0, 50.0)]).expect("point");
        assert!(!bounds.overlaps(&far));
        assert!(bounds.overlaps(&bounds.padded(100.0)));
    }

    #[test]
    fn fill_rules_agree_on_simple_rings() {
        let ring = square();
        for rule in [FillRule::NonZero, FillRule::EvenOdd] {
            assert!(point_in_polygon(Point::new(5.0, 5.0), &ring, rule));
            assert!(!point_in_polygon(Point::new(20.0, 5.0), &ring, rule));
        }
    }

    #[test]
    fn winding_distinguishes_double_covered_region() {
        // The same square traced twice: NonZero winding is 2
        // (inside), EvenOdd parity is 0 (outside).
        let mut doubled = square();
        doubled.extend(square());
        let center = Point::new(5.0, 5.0);
        assert!(point_in_polygon(center, &doubled, FillRule::NonZero));
        assert!(!point_in_polygon(center, &doubled, FillRule::EvenOdd));
    }
}
