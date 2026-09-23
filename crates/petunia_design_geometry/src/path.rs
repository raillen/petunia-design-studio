//! Semantic vector path. Kurbo lives behind [`GPath`] methods only.

use serde::{Deserialize, Serialize};

use crate::{GAffine, GPoint, GRect};
use crate::boolean::FillRule;

/// Cross-product sign of edge `a->b` relative to `p`: >0 when `p` is left.
fn is_left(a: GPoint, b: GPoint, p: GPoint) -> f64 {
    (b.x - a.x) * (p.y - a.y) - (p.x - a.x) * (b.y - a.y)
}

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

    /// Returns the path scaled about `center` by `(sx, sy)` (Table B).
    /// Used by path offsetting so curves track their bounds. Exact for all
    /// verbs; callers guarantee finite factors near 1.0.
    #[must_use]
    pub fn scaled_about(&self, center: GPoint, sx: f64, sy: f64) -> Self {
        let map = |p: GPoint| {
            GPoint::new(center.x + (p.x - center.x) * sx, center.y + (p.y - center.y) * sy)
        };
        let verbs = self
            .verbs
            .iter()
            .map(|verb| match *verb {
                PathVerb::MoveTo(p) => PathVerb::MoveTo(map(p)),
                PathVerb::LineTo(p) => PathVerb::LineTo(map(p)),
                PathVerb::QuadTo(c, p) => PathVerb::QuadTo(map(c), map(p)),
                PathVerb::CubicTo(c1, c2, p) => {
                    PathVerb::CubicTo(map(c1), map(c2), map(p))
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

    /// Arc-length of the flattened outline within `tolerance` (F-21).
    /// Used by text-on-path placement; curve-exact length stays POST_V1.
    #[must_use]
    pub fn outline_length(&self, tolerance: f64) -> f64 {
        self.to_polygons(tolerance.max(0.001))
            .iter()
            .map(|contour| {
                contour
                    .windows(2)
                    .map(|w| w[0].distance_to(w[1]))
                    .sum::<f64>()
            })
            .sum()
    }

    /// Samples the outline at normalized fraction `t` in `[0.0, 1.0]`.
    /// Returns `(point, tangent_angle_radians)` walking subpaths in order.
    /// `None` on empty outlines.
    #[must_use]
    pub fn sample_at(&self, t: f64, tolerance: f64) -> Option<(GPoint, f64)> {
        let contours = self.to_polygons(tolerance.max(0.001));
        let total: f64 = contours
            .iter()
            .map(|c| c.windows(2).map(|w| w[0].distance_to(w[1])).sum::<f64>())
            .sum();
        if total < 1e-9 {
            // Degenerate (single point or empty): report the first point.
            let pt = contours.iter().flatten().next().copied()?;
            return Some((pt, 0.0));
        }
        let mut target = t.clamp(0.0, 1.0) * total;
        for contour in &contours {
            for w in contour.windows(2) {
                let seg = w[0].distance_to(w[1]);
                if target <= seg {
                    let f = if seg < 1e-12 { 0.0 } else { target / seg };
                    let pt = GPoint::new(
                        w[0].x + (w[1].x - w[0].x) * f,
                        w[0].y + (w[1].y - w[0].y) * f,
                    );
                    return Some((pt, (w[1].y - w[0].y).atan2(w[1].x - w[0].x)));
                }
                target -= seg;
            }
            // Gap between subpaths consumes no length; continue into next.
        }
        let last = contours.iter().flatten().next_back().copied()?;
        Some((last, 0.0))
    }

    /// Normalized fraction `t` of the outline point nearest `pt`.
    /// Returns `None` on empty outlines.
    #[must_use]
    pub fn nearest_t(&self, pt: GPoint, tolerance: f64) -> Option<f64> {
        let contours = self.to_polygons(tolerance.max(0.001));
        let total: f64 = contours
            .iter()
            .map(|c| c.windows(2).map(|w| w[0].distance_to(w[1])).sum::<f64>())
            .sum();
        if total < 1e-9 {
            return None;
        }
        let mut best = (f64::INFINITY, 0.0);
        let mut acc = 0.0;
        for contour in &contours {
            for w in contour.windows(2) {
                let (abx, aby) = (w[1].x - w[0].x, w[1].y - w[0].y);
                let len2 = (abx * abx + aby * aby).max(1e-12);
                let f = (((pt.x - w[0].x) * abx + (pt.y - w[0].y) * aby) / len2).clamp(0.0, 1.0);
                let proj = GPoint::new(w[0].x + abx * f, w[0].y + aby * f);
                let d = proj.distance_to(pt);
                if d < best.0 {
                    let seg = abx.hypot(aby);
                    best = (d, (acc + seg * f) / total);
                }
                acc += abx.hypot(aby);
            }
        }
        Some(best.1.clamp(0.0, 1.0))
    }

    /// Creates a rectangle path, with optional corner radii.
    #[must_use]
    pub fn rect(rect: GRect, rx: f64, ry: f64) -> Self {
        let mut path = Self::new();
        let x0 = rect.x0;
        let y0 = rect.y0;
        let x1 = rect.x1;
        let y1 = rect.y1;
        let w = rect.width();
        let h = rect.height();
        let rx = rx.abs().min(w / 2.0);
        let ry = ry.abs().min(h / 2.0);

        if rx <= 1e-6 || ry <= 1e-6 {
            let _ = path.push(PathVerb::MoveTo(GPoint::new(x0, y0)));
            let _ = path.push(PathVerb::LineTo(GPoint::new(x1, y0)));
            let _ = path.push(PathVerb::LineTo(GPoint::new(x1, y1)));
            let _ = path.push(PathVerb::LineTo(GPoint::new(x0, y1)));
            let _ = path.push(PathVerb::Close);
        } else {
            const KAPPA: f64 = 0.5522847498307936;
            let kx = rx * KAPPA;
            let ky = ry * KAPPA;

            let _ = path.push(PathVerb::MoveTo(GPoint::new(x0 + rx, y0)));
            let _ = path.push(PathVerb::LineTo(GPoint::new(x1 - rx, y0)));
            let _ = path.push(PathVerb::CubicTo(
                GPoint::new(x1 - rx + kx, y0),
                GPoint::new(x1, y0 + ry - ky),
                GPoint::new(x1, y0 + ry),
            ));
            let _ = path.push(PathVerb::LineTo(GPoint::new(x1, y1 - ry)));
            let _ = path.push(PathVerb::CubicTo(
                GPoint::new(x1, y1 - ry + ky),
                GPoint::new(x1 - rx + kx, y1),
                GPoint::new(x1 - rx, y1),
            ));
            let _ = path.push(PathVerb::LineTo(GPoint::new(x0 + rx, y1)));
            let _ = path.push(PathVerb::CubicTo(
                GPoint::new(x0 + rx - kx, y1),
                GPoint::new(x0, y1 - ry + ky),
                GPoint::new(x0, y1 - ry),
            ));
            let _ = path.push(PathVerb::LineTo(GPoint::new(x0, y0 + ry)));
            let _ = path.push(PathVerb::CubicTo(
                GPoint::new(x0, y0 + ry - ky),
                GPoint::new(x0 + rx - kx, y0),
                GPoint::new(x0 + rx, y0),
            ));
            let _ = path.push(PathVerb::Close);
        }
        path
    }

    /// Creates an ellipse path centered at `center` with radii `rx` and `ry`.
    #[must_use]
    pub fn ellipse(center: GPoint, rx: f64, ry: f64) -> Self {
        let mut path = Self::new();
        let rx = rx.abs().max(1e-6);
        let ry = ry.abs().max(1e-6);
        const KAPPA: f64 = 0.5522847498307936;
        let kx = rx * KAPPA;
        let ky = ry * KAPPA;
        let cx = center.x;
        let cy = center.y;

        let _ = path.push(PathVerb::MoveTo(GPoint::new(cx, cy - ry)));
        let _ = path.push(PathVerb::CubicTo(
            GPoint::new(cx + kx, cy - ry),
            GPoint::new(cx + rx, cy - ky),
            GPoint::new(cx + rx, cy),
        ));
        let _ = path.push(PathVerb::CubicTo(
            GPoint::new(cx + rx, cy + ky),
            GPoint::new(cx + kx, cy + ry),
            GPoint::new(cx, cy + ry),
        ));
        let _ = path.push(PathVerb::CubicTo(
            GPoint::new(cx - kx, cy + ry),
            GPoint::new(cx - rx, cy + ky),
            GPoint::new(cx - rx, cy),
        ));
        let _ = path.push(PathVerb::CubicTo(
            GPoint::new(cx - rx, cy - ky),
            GPoint::new(cx - kx, cy - ry),
            GPoint::new(cx, cy - ry),
        ));
        let _ = path.push(PathVerb::Close);
        path
    }

    /// Creates a circle path centered at `center` with `radius`.
    #[must_use]
    pub fn circle(center: GPoint, radius: f64) -> Self {
        Self::ellipse(center, radius, radius)
    }

    /// Creates a regular polygon with `sides` vertices centered at `center`.
    #[must_use]
    pub fn regular_polygon(center: GPoint, radius: f64, sides: usize) -> Self {
        let mut path = Self::new();
        let sides = sides.max(3);
        let step = std::f64::consts::TAU / (sides as f64);
        let start_angle = -std::f64::consts::FRAC_PI_2;

        for i in 0..sides {
            let angle = start_angle + (i as f64) * step;
            let pt = GPoint::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            );
            if i == 0 {
                let _ = path.push(PathVerb::MoveTo(pt));
            } else {
                let _ = path.push(PathVerb::LineTo(pt));
            }
        }
        let _ = path.push(PathVerb::Close);
        path
    }

    /// Creates a star polygon with `points` points.
    #[must_use]
    pub fn star(center: GPoint, outer_radius: f64, inner_radius: f64, points: usize) -> Self {
        let mut path = Self::new();
        let points = points.max(3);
        let total_vertices = points * 2;
        let step = std::f64::consts::TAU / (total_vertices as f64);
        let start_angle = -std::f64::consts::FRAC_PI_2;

        for i in 0..total_vertices {
            let angle = start_angle + (i as f64) * step;
            let r = if i % 2 == 0 {
                outer_radius
            } else {
                inner_radius
            };
            let pt = GPoint::new(center.x + r * angle.cos(), center.y + r * angle.sin());
            if i == 0 {
                let _ = path.push(PathVerb::MoveTo(pt));
            } else {
                let _ = path.push(PathVerb::LineTo(pt));
            }
        }
        let _ = path.push(PathVerb::Close);
        path
    }

    /// Creates a rectangle with one radius per corner:
    /// `[top-left, top-right, bottom-right, bottom-left]` (clockwise).
    /// Each radius clamps to half the shortest edge so corners never
    /// overlap. Zero radii stay sharp lines.
    #[must_use]
    pub fn rect_corners(rect: GRect, radii: [f64; 4]) -> Self {
        const KAPPA: f64 = 0.5522847498307936;
        let (x0, y0, x1, y1) = (rect.x0, rect.y0, rect.x1, rect.y1);
        let limit = rect.width().max(0.0).min(rect.height().max(0.0)) / 2.0;
        let [tl, tr, br, bl] = [
            radii[0].abs().min(limit),
            radii[1].abs().min(limit),
            radii[2].abs().min(limit),
            radii[3].abs().min(limit),
        ];
        let mut path = Self::new();
        let edge_to = |path: &mut Self, p: GPoint| {
            if path.is_empty() {
                let _ = path.push(PathVerb::MoveTo(p));
            } else {
                let _ = path.push(PathVerb::LineTo(p));
            }
        };
        // Quarter-circle arc as one cubic through tangent points.
        let corner_arc =
            |path: &mut Self, start: GPoint, c1: GPoint, c2: GPoint, end: GPoint| {
                edge_to(path, start);
                let _ = path.push(PathVerb::CubicTo(c1, c2, end));
            };
        // Clockwise from the left of the top edge.
        if tl > 1e-9 {
            let (k, cx, cy) = (tl * KAPPA, x0 + tl, y0 + tl);
            corner_arc(
                &mut path,
                GPoint::new(x0, cy),
                GPoint::new(x0, cy - k),
                GPoint::new(cx - k, y0),
                GPoint::new(cx, y0),
            );
        } else {
            edge_to(&mut path, GPoint::new(x0, y0));
        }
        if tr > 1e-9 {
            let (k, cx, cy) = (tr * KAPPA, x1 - tr, y0 + tr);
            corner_arc(
                &mut path,
                GPoint::new(cx, y0),
                GPoint::new(cx + k, y0),
                GPoint::new(x1, cy - k),
                GPoint::new(x1, cy),
            );
        } else {
            edge_to(&mut path, GPoint::new(x1, y0));
        }
        if br > 1e-9 {
            let (k, cx, cy) = (br * KAPPA, x1 - br, y1 - br);
            corner_arc(
                &mut path,
                GPoint::new(x1, cy),
                GPoint::new(x1, cy + k),
                GPoint::new(cx + k, y1),
                GPoint::new(cx, y1),
            );
        } else {
            edge_to(&mut path, GPoint::new(x1, y1));
        }
        if bl > 1e-9 {
            let (k, cx, cy) = (bl * KAPPA, x0 + bl, y1 - bl);
            corner_arc(
                &mut path,
                GPoint::new(cx, y1),
                GPoint::new(cx - k, y1),
                GPoint::new(x0, cy + k),
                GPoint::new(x0, cy),
            );
        } else {
            edge_to(&mut path, GPoint::new(x0, y1));
        }
        let _ = path.push(PathVerb::Close);
        path
    }

    /// Creates a straight line segment from `p1` to `p2`.    #[must_use]
    pub fn line(p1: GPoint, p2: GPoint) -> Self {
        let mut path = Self::new();
        let _ = path.push(PathVerb::MoveTo(p1));
        let _ = path.push(PathVerb::LineTo(p2));
        path
    }

    /// Builds a path from closed polygon contours (e.g. from boolean operations).
    #[must_use]
    pub fn from_polygons(contours: &[Vec<GPoint>]) -> Self {
        let mut path = Self::new();
        for contour in contours {
            if contour.is_empty() {
                continue;
            }
            let _ = path.push(PathVerb::MoveTo(contour[0]));
            for pt in &contour[1..] {
                let _ = path.push(PathVerb::LineTo(*pt));
            }
            let _ = path.push(PathVerb::Close);
        }
        path
    }

    /// Converts this path to standard SVG path data (e.g. `M 10 20 L 30 40 Z`).
    #[must_use]
    pub fn to_svg_path_data(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        for verb in &self.verbs {
            match verb {
                PathVerb::MoveTo(p) => {
                    let _ = write!(out, "M {:.3} {:.3} ", p.x, p.y);
                }
                PathVerb::LineTo(p) => {
                    let _ = write!(out, "L {:.3} {:.3} ", p.x, p.y);
                }
                PathVerb::QuadTo(c, p) => {
                    let _ = write!(out, "Q {:.3} {:.3} {:.3} {:.3} ", c.x, c.y, p.x, p.y);
                }
                PathVerb::CubicTo(c1, c2, p) => {
                    let _ = write!(
                        out,
                        "C {:.3} {:.3} {:.3} {:.3} {:.3} {:.3} ",
                        c1.x, c1.y, c2.x, c2.y, p.x, p.y
                    );
                }
                PathVerb::Close => {
                    out.push_str("Z ");
                }
            }
        }
        out.trim_end().to_string()
    }

    /// Hit-tests whether `point` is contained inside the path using even-odd rule.
    #[must_use]
    pub fn contains_point(&self, point: GPoint, tolerance: f64) -> bool {
        self.contains_point_with_fill(point, tolerance, FillRule::EvenOdd)
    }

    /// Hit-tests with an explicit fill rule (F-17, F-21).
    /// `EvenOdd` flips on every crossing; `NonZero` counts winding direction.
    #[must_use]
    pub fn contains_point_with_fill(
        &self,
        point: GPoint,
        tolerance: f64,
        fill_rule: FillRule,
    ) -> bool {
        let polygons = self.to_polygons(tolerance);
        match fill_rule {
            FillRule::EvenOdd => {
                let mut inside = false;
                for contour in polygons {
                    if contour.len() < 3 {
                        continue;
                    }
                    let n = contour.len();
                    for i in 0..n {
                        let j = (i + 1) % n;
                        let pi = contour[i];
                        let pj = contour[j];
                        if ((pi.y > point.y) != (pj.y > point.y))
                            && (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x)
                        {
                            inside = !inside;
                        }
                    }
                }
                inside
            }
            FillRule::NonZero => {
                let mut winding = 0i32;
                for contour in polygons {
                    if contour.len() < 3 {
                        continue;
                    }
                    let n = contour.len();
                    for i in 0..n {
                        let pi = contour[i];
                        let pj = contour[(i + 1) % n];
                        if pi.y <= point.y {
                            if pj.y > point.y
                                && is_left(pi, pj, point) > 0.0
                            {
                                winding += 1;
                            }
                        } else if pj.y <= point.y && is_left(pi, pj, point) < 0.0 {
                            winding -= 1;
                        }
                    }
                }
                winding != 0
            }
        }
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
    fn sample_at_walks_line_by_fraction() {
        let line = GPath::line(GPoint::new(0.0, 0.0), GPoint::new(100.0, 0.0));
        let (p0, a0) = line.sample_at(0.0, 0.5).expect("sample");
        let (p1, a1) = line.sample_at(1.0, 0.5).expect("sample");
        let (mid, _) = line.sample_at(0.5, 0.5).expect("sample");
        assert!((p0.x).abs() < 1e-6 && (p1.x - 100.0).abs() < 1e-6);
        assert!((mid.x - 50.0).abs() < 1e-6);
        assert!(a0.abs() < 1e-9 && a1.abs() < 1e-9);
        assert!((line.outline_length(0.5) - 100.0).abs() < 1e-6);
    }

    #[test]
    fn nearest_t_roundtrips_on_line() {
        let line = GPath::line(GPoint::new(0.0, 0.0), GPoint::new(100.0, 0.0));
        let t = line.nearest_t(GPoint::new(30.0, 4.0), 0.5).expect("t");
        assert!((t - 0.3).abs() < 1e-6, "got {t}");
        assert!(GPath::new().nearest_t(GPoint::ORIGIN, 0.5).is_none());
    }

    #[test]
    fn rect_corners_all_sharp_matches_plain_rect() {
        let sharp = GPath::rect_corners(GRect::new(0.0, 0.0, 100.0, 60.0), [0.0; 4]);
        assert_eq!(sharp.bounding_box(), Some(GRect::new(0.0, 0.0, 100.0, 60.0)));
        assert!(sharp.verbs.iter().all(|v| matches!(
            v,
            PathVerb::MoveTo(_) | PathVerb::LineTo(_) | PathVerb::Close
        )));
    }

    #[test]
    fn rect_corners_single_round_keeps_bounds() {
        let one = GPath::rect_corners(GRect::new(0.0, 0.0, 100.0, 60.0), [20.0, 0.0, 0.0, 0.0]);
        assert_eq!(one.bounding_box(), Some(GRect::new(0.0, 0.0, 100.0, 60.0)));
        assert_eq!(
            one.verbs.iter().filter(|v| matches!(v, PathVerb::CubicTo(_, _, _))).count(),
            1
        );
    }

    #[test]
    fn rect_corners_clamp_huge_radii() {
        let big = GPath::rect_corners(GRect::new(0.0, 0.0, 100.0, 60.0), [500.0; 4]);
        let bounds = big.bounding_box().expect("bounds");
        assert!((bounds.width() - 100.0).abs() < 1e-6, "got {bounds:?}");
        assert!(big.verbs.iter().all(|v| match v {
            PathVerb::MoveTo(p) | PathVerb::LineTo(p) => p.is_finite(),
            PathVerb::QuadTo(c, p) => c.is_finite() && p.is_finite(),
            PathVerb::CubicTo(c1, c2, p) => c1.is_finite() && c2.is_finite() && p.is_finite(),
            PathVerb::Close => true,
        }));
    }

    #[test]
    fn non_finite_coordinates_rejected() {
        let mut path = GPath::new();
        assert!(path
            .push(PathVerb::LineTo(GPoint::new(f64::NAN, 0.0)))
            .is_err());
    }
}
