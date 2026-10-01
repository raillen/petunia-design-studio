//! Semantic 2D affine transform. Kurbo stays inside the adapter.

use serde::{Deserialize, Serialize};

use crate::GPoint;

/// Row-major 2D affine transform:
///
/// ```text
/// | a c e |
/// | b d f |
/// | 0 0 1 |
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GAffine {
    /// Matrix coefficients `[a, b, c, d, e, f]`.
    pub coeffs: [f64; 6],
}

impl GAffine {
    /// Identity transform.
    pub const IDENTITY: Self = Self {
        coeffs: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    };

    /// Creates a transform from raw coefficients.
    #[must_use]
    pub const fn new(coeffs: [f64; 6]) -> Self {
        Self { coeffs }
    }

    /// Translation by (`dx`, `dy`).
    #[must_use]
    pub const fn translate(dx: f64, dy: f64) -> Self {
        Self {
            coeffs: [1.0, 0.0, 0.0, 1.0, dx, dy],
        }
    }

    /// Uniform or non-uniform scale.
    #[must_use]
    pub const fn scale(sx: f64, sy: f64) -> Self {
        Self {
            coeffs: [sx, 0.0, 0.0, sy, 0.0, 0.0],
        }
    }

    /// Pure rotation around the origin by `radians`.
    #[must_use]
    pub fn rotate(radians: f64) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self {
            coeffs: [cos, sin, -sin, cos, 0.0, 0.0],
        }
    }

    /// Computes the determinant of the 2x2 linear portion (`a * d - b * c`).
    #[must_use]
    pub fn determinant(&self) -> f64 {
        let [a, b, c, d, _, _] = self.coeffs;
        a * d - b * c
    }

    /// Computes the inverse affine transform, or returns `None` if singular.
    #[must_use]
    pub fn inverse(&self) -> Option<Self> {
        let det = self.determinant();
        if det.abs() < 1e-12 {
            return None;
        }
        let inv_det = 1.0 / det;
        let [a, b, c, d, e, f] = self.coeffs;
        Some(Self {
            coeffs: [
                d * inv_det,
                -b * inv_det,
                -c * inv_det,
                a * inv_det,
                (c * f - d * e) * inv_det,
                (b * e - a * f) * inv_det,
            ],
        })
    }

    /// Applies the transform to a point.
    #[must_use]
    pub fn apply(self, point: GPoint) -> GPoint {
        let [a, b, c, d, e, f] = self.coeffs;
        GPoint {
            x: a.mul_add(point.x, c.mul_add(point.y, e)),
            y: b.mul_add(point.x, d.mul_add(point.y, f)),
        }
    }

    /// Axis-aligned footprint of all four transformed corners.
    #[must_use]
    pub fn transform_rect(self, rect: crate::GRect) -> crate::GRect {
        let points = [
            GPoint::new(rect.x0, rect.y0),
            GPoint::new(rect.x1, rect.y0),
            GPoint::new(rect.x1, rect.y1),
            GPoint::new(rect.x0, rect.y1),
        ]
        .map(|point| self.apply(point));
        crate::GRect::new(
            points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min),
            points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min),
            points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max),
            points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max),
        )
    }
    /// Composes so `other` applies first: `apply(p) == self.apply(other.apply(p))`.
    #[must_use]
    pub fn after(self, other: Self) -> Self {
        let [a1, b1, c1, d1, e1, f1] = self.coeffs;
        let [a2, b2, c2, d2, e2, f2] = other.coeffs;
        Self {
            coeffs: [
                a1 * a2 + c1 * b2,
                b1 * a2 + d1 * b2,
                a1 * c2 + c1 * d2,
                b1 * c2 + d1 * d2,
                a1 * e2 + c1 * f2 + e1,
                b1 * e2 + d1 * f2 + f1,
            ],
        }
    }
}

/// Rotates `point` around `pivot` by `delta_angle` radians (Table B).
/// Returns the rotated point; degenerate pivots (NaN) pass through.
#[must_use]
pub fn rotate_point_around(point: GPoint, pivot: GPoint, delta_angle: f64) -> GPoint {
    if !delta_angle.is_finite() || !pivot.is_finite() || !point.is_finite() {
        return point;
    }
    let transform = GAffine::translate(pivot.x, pivot.y)
        .after(GAffine::rotate(delta_angle))
        .after(GAffine::translate(-pivot.x, -pivot.y));
    transform.apply(point)
}

/// Computes the signed angular delta from `p0` to `p1` around `pivot`
/// (Table B). Returns `None` when either vector is degenerate, so callers
/// never apply an `atan2(0,0)` phantom rotation.
#[must_use]
pub fn pivot_angle_delta(p0: GPoint, p1: GPoint, pivot: GPoint) -> Option<f64> {
    let v0x = p0.x - pivot.x;
    let v0y = p0.y - pivot.y;
    let v1x = p1.x - pivot.x;
    let v1y = p1.y - pivot.y;
    if v0x * v0x + v0y * v0y < 1e-12 || v1x * v1x + v1y * v1y < 1e-12 {
        return None;
    }
    let a0 = v0y.atan2(v0x);
    let a1 = v1y.atan2(v1x);
    Some(a1 - a0)
}

/// Computes the radial scale factor from `p0` to `p1` around `pivot`
/// (Table B): `|p1 - pivot| / |p0 - pivot|`. Returns `None` when either
/// radius is degenerate, so callers never divide by zero.
#[must_use]
pub fn scale_factor_around(p0: GPoint, p1: GPoint, pivot: GPoint) -> Option<f64> {
    let r0 = ((p0.x - pivot.x).powi(2) + (p0.y - pivot.y).powi(2)).sqrt();
    let r1 = ((p1.x - pivot.x).powi(2) + (p1.y - pivot.y).powi(2)).sqrt();
    if !r0.is_finite() || !r1.is_finite() || r0 < 1e-9 {
        return None;
    }
    Some(r1 / r0)
}

/// Scales `[x, y, w, h]` bounds by `k` about `pivot` (Table B).
#[must_use]
pub fn scale_bounds_about(bounds: [f64; 4], pivot: GPoint, k: f64) -> Option<[f64; 4]> {
    if !k.is_finite() || k <= 1e-9 {
        return None;
    }
    let (x, y, w, h) = (bounds[0], bounds[1], bounds[2], bounds[3]);
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    Some([
        pivot.x + (x - pivot.x) * k,
        pivot.y + (y - pivot.y) * k,
        (w * k).max(1.0),
        (h * k).max(1.0),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_then_scale_composes() {
        let composed = GAffine::scale(2.0, 2.0).after(GAffine::translate(1.0, 1.0));
        let point = composed.apply(GPoint::ORIGIN);
        assert_eq!(point, GPoint::new(2.0, 2.0));
    }

    #[test]
    fn rotate_90_degrees_transforms_axis() {
        use std::f64::consts::FRAC_PI_2;
        let rot = GAffine::rotate(FRAC_PI_2);
        let p = rot.apply(GPoint::new(1.0, 0.0));
        assert!((p.x).abs() < 1e-10);
        assert!((p.y - 1.0).abs() < 1e-10);
    }

    #[test]
    fn inverse_cancels_transform() {
        let t = GAffine::translate(15.0, -25.0);
        let r = GAffine::rotate(0.4);
        let s = GAffine::scale(2.5, 1.8);
        let composed = t.after(r).after(s);

        let inv = composed.inverse().expect("invertible");
        let roundtrip = composed.after(inv);
        let p = roundtrip.apply(GPoint::new(42.0, -7.0));
        assert!((p.x - 42.0).abs() < 1e-10);
        assert!((p.y - (-7.0)).abs() < 1e-10);
    }
}
