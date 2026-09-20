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
