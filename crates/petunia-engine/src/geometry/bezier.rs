//! Cubic Bézier evaluation, splitting, bounds and flattening.
//!
//! All computation stays on Petunia [`Point`] values; kurbo
//! conversions live in a future adapter, never in persisted types.
//! Flattening is deterministic for a given tolerance and feeds
//! boolean, hit-test and tessellation consumers.

use petunia_core::{Contour, PathNode, Point, SegmentKind, Tolerance};

/// Checked adaptive flattening cannot silently accept a depth-limited segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FlattenError {
    #[error("flatten tolerance/geometry is invalid")]
    InvalidInput,
    #[error("flatten depth limit reached before meeting tolerance")]
    DepthLimit,
    #[error("flatten point budget exceeded")]
    PointLimit,
}

pub fn try_flatten_contour(
    contour: &Contour,
    tolerance: Tolerance,
    max_points: usize,
) -> Result<Vec<Point>, FlattenError> {
    if !tolerance.0.is_finite() || tolerance.0 <= 0.0 {
        return Err(FlattenError::InvalidInput);
    }
    if contour.nodes.len() > max_points {
        return Err(FlattenError::PointLimit);
    }
    let mut output = Vec::new();
    let Some(first) = contour.nodes.first() else {
        return Ok(output);
    };
    if !first.point.x.is_finite() || !first.point.y.is_finite() {
        return Err(FlattenError::InvalidInput);
    }
    if max_points == 0 {
        return Err(FlattenError::PointLimit);
    }
    output.push(first.point);
    let segments = if contour.closed {
        contour.nodes.len()
    } else {
        contour.nodes.len().saturating_sub(1)
    };
    for index in 0..segments {
        let from = &contour.nodes[index];
        let to = &contour.nodes[(index + 1) % contour.nodes.len()];
        if let Some(cubic) = segment_bezier(from, to) {
            let mut stack = vec![(cubic, 0u32)];
            while let Some((cubic, depth)) = stack.pop() {
                if [cubic.p0, cubic.p1, cubic.p2, cubic.p3]
                    .iter()
                    .any(|point| !point.x.is_finite() || !point.y.is_finite())
                {
                    return Err(FlattenError::InvalidInput);
                }
                if flatness(cubic) <= tolerance.0 {
                    if output.len() >= max_points {
                        return Err(FlattenError::PointLimit);
                    }
                    output.push(cubic.p3);
                } else {
                    if depth >= 24 {
                        return Err(FlattenError::DepthLimit);
                    }
                    let (left, right) = cubic.split(0.5);
                    stack.push((right, depth + 1));
                    stack.push((left, depth + 1));
                }
            }
        } else {
            if output.len() >= max_points {
                return Err(FlattenError::PointLimit);
            }
            if !to.point.x.is_finite() || !to.point.y.is_finite() {
                return Err(FlattenError::InvalidInput);
            }
            output.push(to.point);
        }
    }
    if contour.closed && output.len() > 1 && output.first() == output.last() {
        output.pop();
    }
    Ok(output)
}

/// One cubic Bézier segment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicBez {
    pub p0: Point,
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
}

impl CubicBez {
    /// Build a segment from its anchors and handles.
    #[must_use]
    pub const fn new(p0: Point, p1: Point, p2: Point, p3: Point) -> Self {
        Self { p0, p1, p2, p3 }
    }

    /// Evaluate at `t` in `0..1` with de Casteljau.
    #[must_use]
    pub fn evaluate(self, t: f64) -> Point {
        let a = lerp_point(self.p0, self.p1, t);
        let b = lerp_point(self.p1, self.p2, t);
        let c = lerp_point(self.p2, self.p3, t);
        let d = lerp_point(a, b, t);
        let e = lerp_point(b, c, t);
        lerp_point(d, e, t)
    }

    /// Split at `t` into two cubics covering `[0, t]` and `[t, 1]`.
    #[must_use]
    pub fn split(self, t: f64) -> (Self, Self) {
        let a = lerp_point(self.p0, self.p1, t);
        let b = lerp_point(self.p1, self.p2, t);
        let c = lerp_point(self.p2, self.p3, t);
        let d = lerp_point(a, b, t);
        let e = lerp_point(b, c, t);
        let mid = lerp_point(d, e, t);
        (
            Self {
                p0: self.p0,
                p1: a,
                p2: d,
                p3: mid,
            },
            Self {
                p0: mid,
                p1: e,
                p2: c,
                p3: self.p3,
            },
        )
    }

    /// Tight axis bounds from endpoints plus interior extrema of the
    /// derivative quadratics.
    #[must_use]
    pub fn bounds(self) -> (Point, Point) {
        let mut min_x = self.p0.x.min(self.p3.x);
        let mut max_x = self.p0.x.max(self.p3.x);
        let mut min_y = self.p0.y.min(self.p3.y);
        let mut max_y = self.p0.y.max(self.p3.y);
        for t in extrema(self.p0.x, self.p1.x, self.p2.x, self.p3.x)
            .into_iter()
            .chain(extrema(self.p0.y, self.p1.y, self.p2.y, self.p3.y))
        {
            let point = self.evaluate(t);
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }
        (Point::new(min_x, min_y), Point::new(max_x, max_y))
    }

    /// Adaptive subdivision flattening: split until the control
    /// polygon stays within `tolerance` of the chord. Endpoints are
    /// included; the caller closes rings when needed.
    #[must_use]
    pub fn flatten(self, tolerance: Tolerance) -> Vec<Point> {
        let mut points = vec![self.p0];
        flatten_into(self, tolerance.0, &mut points, 0);
        points.push(self.p3);
        points
    }

    /// Chord-length approximation over the flattened curve.
    #[must_use]
    pub fn approx_length(self, tolerance: Tolerance) -> f64 {
        self.flatten(tolerance)
            .windows(2)
            .map(|pair| distance(pair[0], pair[1]))
            .sum()
    }
}

fn lerp_point(a: Point, b: Point, t: f64) -> Point {
    Point::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

fn distance(a: Point, b: Point) -> f64 {
    (b.x - a.x).hypot(b.y - a.y)
}

/// Roots of the cubic derivative in `(0, 1)` for one component.
fn extrema(p0: f64, p1: f64, p2: f64, p3: f64) -> Vec<f64> {
    // Derivative coefficients of the Bernstein form, divided by 3.
    let a = -p0 + 3.0 * p1 - 3.0 * p2 + p3;
    let b = 2.0 * (p0 - 2.0 * p1 + p2);
    let c = -p0 + p1;
    let mut roots = Vec::new();
    if a.abs() < f64::EPSILON {
        if b.abs() > f64::EPSILON {
            let t = -c / b;
            if t > 0.0 && t < 1.0 {
                roots.push(t);
            }
        }
        return roots;
    }
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return roots;
    }
    let root = discriminant.sqrt();
    for t in [(-b - root) / (2.0 * a), (-b + root) / (2.0 * a)] {
        if t > 0.0 && t < 1.0 {
            roots.push(t);
        }
    }
    roots
}

fn flatness(cubic: CubicBez) -> f64 {
    // Max distance of inner controls from the chord, scaled: the
    // standard subdivision estimate (times 0.75 is folded into the
    // caller threshold by using the raw value against tolerance).
    let ux = 3.0 * cubic.p1.x - 2.0 * cubic.p0.x - cubic.p3.x;
    let uy = 3.0 * cubic.p1.y - 2.0 * cubic.p0.y - cubic.p3.y;
    let vx = 3.0 * cubic.p2.x - 2.0 * cubic.p3.x - cubic.p0.x;
    let vy = 3.0 * cubic.p2.y - 2.0 * cubic.p3.y - cubic.p0.y;
    ux.hypot(uy).max(vx.hypot(vy))
}

fn flatten_into(cubic: CubicBez, tolerance: f64, out: &mut Vec<Point>, depth: u32) {
    if depth > 24 || flatness(cubic) <= tolerance {
        return;
    }
    let (left, right) = cubic.split(0.5);
    flatten_into(left, tolerance, out, depth + 1);
    out.push(left.p3);
    flatten_into(right, tolerance, out, depth + 1);
}

/// Flatten one contour into polylines: straight runs pass through,
/// cubic runs use endpoint anchors for absent control handles.
#[must_use]
pub fn flatten_contour(contour: &Contour, tolerance: Tolerance) -> Vec<Point> {
    let mut points = Vec::new();
    if contour.nodes.is_empty() {
        return points;
    }
    let segments = contour.nodes.len() - 1;
    for (index, node) in contour.nodes.iter().enumerate() {
        if index == 0 {
            points.push(node.point);
        }
        if index >= segments && !contour.closed {
            break;
        }
        let next = &contour.nodes[(index + 1) % contour.nodes.len()];
        match segment_bezier(node, next) {
            Some(cubic) => {
                let mut flat = cubic.flatten(tolerance);
                flat.remove(0);
                points.extend(flat);
            }
            _ => points.push(next.point),
        }
        if index >= segments {
            break;
        }
    }
    // Drop a duplicated closing anchor so rings stay clean.
    if contour.closed && points.len() > 1 && points.first() == points.last() {
        points.pop();
    }
    points
}

/// Build a segment between two anchors honoring one-sided handles.
#[must_use]
pub fn segment_bezier(from: &PathNode, to: &PathNode) -> Option<CubicBez> {
    match from.outgoing {
        SegmentKind::Cubic => Some(CubicBez {
            p0: from.point,
            p1: from.handle_out.unwrap_or(from.point),
            p2: to.handle_in.unwrap_or(to.point),
            p3: to.point,
        }),
        SegmentKind::Line => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tolerance() -> Tolerance {
        Tolerance::new(0.01).expect("valid")
    }

    #[test]
    fn explicit_line_ignores_handles_and_one_sided_cubic_keeps_curvature() {
        let mut a = PathNode::new(Point::new(0.0, 0.0), petunia_core::NodeKind::Cusp);
        a.handle_out = Some(Point::new(0.0, 10.0));
        let b = PathNode::new(Point::new(10.0, 0.0), petunia_core::NodeKind::Cusp);
        let mut contour = Contour::new(false);
        contour.nodes = vec![a.clone(), b.clone()];
        let curved = flatten_contour(&contour, tolerance());
        assert!(curved.iter().any(|p| p.y > 1.0));
        a.outgoing = SegmentKind::Line;
        contour.nodes[0] = a;
        assert_eq!(
            flatten_contour(&contour, tolerance()),
            vec![contour.nodes[0].point, b.point]
        );
    }

    fn s_curve() -> CubicBez {
        CubicBez {
            p0: Point::new(0.0, 0.0),
            p1: Point::new(1.0, 0.0),
            p2: Point::new(0.0, 1.0),
            p3: Point::new(1.0, 1.0),
        }
    }

    #[test]
    fn evaluate_matches_endpoints_and_midpoint() {
        let curve = s_curve();
        assert_eq!(curve.evaluate(0.0), curve.p0);
        assert_eq!(curve.evaluate(1.0), curve.p3);
        let mid = curve.evaluate(0.5);
        assert!((mid.x - 0.5).abs() < 1e-12);
        assert!((mid.y - 0.5).abs() < 1e-12);
    }

    #[test]
    fn split_covers_both_halves_continuously() {
        let curve = s_curve();
        let (left, right) = curve.split(0.5);
        assert_eq!(left.p3, right.p0);
        assert_eq!(left.p0, curve.p0);
        assert_eq!(right.p3, curve.p3);
        assert_eq!(left.evaluate(1.0), curve.evaluate(0.5));
    }

    #[test]
    fn bounds_contain_tight_extrema() {
        let (min, max) = s_curve().bounds();
        assert!(min.x <= 0.0 && max.x >= 1.0);
        assert!(min.y <= 0.0 && max.y >= 1.0);
        let flat = CubicBez {
            p0: Point::new(0.0, 0.0),
            p1: Point::new(3.0, 0.0),
            p2: Point::new(3.0, 0.0),
            p3: Point::new(6.0, 0.0),
        };
        let (min, max) = flat.bounds();
        assert_eq!((min.x, max.x), (0.0, 6.0));
    }

    #[test]
    fn flatten_respects_tolerance_deterministically() {
        let fine = s_curve().flatten(Tolerance::new(0.001).expect("valid"));
        let coarse = s_curve().flatten(Tolerance::new(0.1).expect("valid"));
        assert!(fine.len() > coarse.len());
        assert_eq!(s_curve().flatten(tolerance()).first(), Some(&s_curve().p0));
    }
}
