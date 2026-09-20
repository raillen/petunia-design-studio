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
}
