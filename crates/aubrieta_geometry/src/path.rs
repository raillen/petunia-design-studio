//! Semantic vector path. Kurbo lives behind [`GPath`] methods only.

use serde::{Deserialize, Serialize};

use crate::{GAffine, GPoint, GRect};

/// Single path verb with explicit coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PathVerb {
    /// Start a new subpath.
    MoveTo(GPoint),
    /// Straight segment.
    LineTo(GPoint),
    /// Quadratic Bézier with one control point.
    QuadTo(GPoint, GPoint),
    /// Cubic Bézier with two control points.
    CubicTo(GPoint, GPoint, GPoint),
    /// Close the current subpath with a straight segment.
    Close,
}

/// Owned vector path: ordered verbs forming zero or more subpaths.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GPath {
    /// Verbs in draw order.
    pub verbs: Vec<PathVerb>,
}

impl GPath {
    /// Creates an empty path.
    #[must_use]
    pub fn new() -> Self {
        Self { verbs: Vec::new() }
    }

    /// True when there are no verbs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.verbs.is_empty()
    }

    /// Number of `MoveTo` verbs (subpath starts).
    #[must_use]
    pub fn subpath_count(&self) -> usize {
        self.verbs
            .iter()
            .filter(|v| matches!(v, PathVerb::MoveTo(_)))
            .count()
    }

    /// Appends a verb, rejecting non-finite coordinates.
    pub fn push(&mut self, verb: PathVerb) -> Result<(), String> {
        // Validate coordinates explicitly; affines preserve finiteness.
        let mut scratch = [GPoint::ORIGIN; 3];
        let points: &[GPoint] = match &verb {
            PathVerb::MoveTo(p) | PathVerb::LineTo(p) => std::slice::from_ref(p),
            PathVerb::QuadTo(c, p) => {
                scratch[0] = *c;
                scratch[1] = *p;
                &scratch[..2]
            }
            PathVerb::CubicTo(c1, c2, p) => {
                scratch[0] = *c1;
                scratch[1] = *c2;
                scratch[2] = *p;
                &scratch[..3]
            }
            PathVerb::Close => &[],
        };
        for point in points {
            if !point.is_finite() {
                return Err(format!("non-finite coordinate in {verb:?}"));
            }
        }
        self.verbs.push(verb);
        Ok(())
    }

    /// Tight-ish bounding box via the Kurbo adapter. `None` when empty.
    #[must_use]
    pub fn bounding_box(&self) -> Option<GRect> {
        kurbo_adapter::bounding_box(self)
    }

    /// Approximate total length within `tolerance`.
    #[must_use]
    pub fn approx_length(&self, tolerance: f64) -> f64 {
        kurbo_adapter::approx_length(self, tolerance)
    }

    /// Returns the path with `affine` applied to every coordinate.
    /// Exact for all verbs: affines preserve Bézier degree.
    #[must_use]
    pub fn transformed(&self, affine: GAffine) -> Self {
        let verbs = self
            .verbs
            .iter()
            .map(|verb| match *verb {
                PathVerb::MoveTo(p) => PathVerb::MoveTo(affine.apply(p)),
                PathVerb::LineTo(p) => PathVerb::LineTo(affine.apply(p)),
                PathVerb::QuadTo(c, p) => PathVerb::QuadTo(affine.apply(c), affine.apply(p)),
                PathVerb::CubicTo(c1, c2, p) => {
                    PathVerb::CubicTo(affine.apply(c1), affine.apply(c2), affine.apply(p))
                }
                PathVerb::Close => PathVerb::Close,
            })
            .collect();
        Self { verbs }
    }

    /// Flattens curves to polygons within `tolerance`, one contour per
    /// subpath. Closure points are not duplicated.
    #[must_use]
    pub fn to_polygons(&self, tolerance: f64) -> Vec<Vec<GPoint>> {
        kurbo_adapter::flatten_to_polygons(self, tolerance)
    }
}

/// Kurbo adapter. The only module allowed to name `kurbo` types.
mod kurbo_adapter {
    use kurbo::{Affine, BezPath, PathEl, Point, Shape as _};

    use super::{GPath, PathVerb};
    use crate::{GPoint, GRect};

    fn to_kurbo_point(point: GPoint) -> Point {
        Point::new(point.x, point.y)
    }

    fn to_kurbo(path: &GPath) -> BezPath {
        let mut bez = BezPath::new();
        for verb in &path.verbs {
            match *verb {
                PathVerb::MoveTo(p) => bez.move_to(to_kurbo_point(p)),
                PathVerb::LineTo(p) => bez.line_to(to_kurbo_point(p)),
                PathVerb::QuadTo(c, p) => {
                    bez.quad_to(to_kurbo_point(c), to_kurbo_point(p));
                }
                PathVerb::CubicTo(c1, c2, p) => {
                    bez.curve_to(to_kurbo_point(c1), to_kurbo_point(c2), to_kurbo_point(p));
                }
                PathVerb::Close => bez.close_path(),
            }
        }
        bez
    }

    pub(super) fn bounding_box(path: &GPath) -> Option<GRect> {
        if path.is_empty() {
            return None;
        }
        let rect = to_kurbo(path).bounding_box();
        Some(GRect::new(rect.x0, rect.y0, rect.x1, rect.y1))
    }

    pub(super) fn approx_length(path: &GPath, tolerance: f64) -> f64 {
        to_kurbo(path).perimeter(tolerance.max(0.001))
    }

    pub(super) fn flatten_to_polygons(path: &GPath, tolerance: f64) -> Vec<Vec<GPoint>> {
        let bez = to_kurbo(path);
        let mut contours: Vec<Vec<GPoint>> = Vec::new();
        let mut current: Vec<GPoint> = Vec::new();
        kurbo::flatten(bez.iter(), tolerance.max(0.001), |el| match el {
            PathEl::MoveTo(p) => {
                if !current.is_empty() {
                    contours.push(std::mem::take(&mut current));
                }
                current.push(GPoint::new(p.x, p.y));
            }
            PathEl::LineTo(p) => current.push(GPoint::new(p.x, p.y)),
            PathEl::ClosePath => {
                if !current.is_empty() {
                    contours.push(std::mem::take(&mut current));
                }
            }
            // flatten() only emits MoveTo/LineTo/ClosePath.
            PathEl::QuadTo(..) | PathEl::CurveTo(..) => {}
        });
        if !current.is_empty() {
            contours.push(current);
        }
        contours
    }

    #[allow(dead_code)]
    pub(super) fn to_kurbo_affine(affine: crate::GAffine) -> Affine {
        Affine::new(affine.coeffs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> GPath {
        let mut path = GPath::new();
        for verb in [
            PathVerb::MoveTo(GPoint::new(0.0, 0.0)),
            PathVerb::LineTo(GPoint::new(4.0, 0.0)),
            PathVerb::LineTo(GPoint::new(0.0, 3.0)),
            PathVerb::Close,
        ] {
            path.push(verb).expect("finite");
        }
        path
    }

    #[test]
    fn bounding_box_matches_triangle() {
        let bounds = triangle().bounding_box().expect("bounds");
        assert_eq!(bounds, GRect::new(0.0, 0.0, 4.0, 3.0));
    }

    #[test]
    fn empty_path_has_no_bounds() {
        assert_eq!(GPath::new().bounding_box(), None);
    }

    #[test]
    fn transform_moves_bounds_exactly() {
        let moved = triangle().transformed(GAffine::translate(10.0, 0.0));
        let bounds = moved.bounding_box().expect("bounds");
        assert_eq!(bounds, GRect::new(10.0, 0.0, 14.0, 3.0));
    }

    #[test]
    fn non_finite_coordinates_rejected() {
        let mut path = GPath::new();
        assert!(path
            .push(PathVerb::LineTo(GPoint::new(f64::NAN, 0.0)))
            .is_err());
    }
}
