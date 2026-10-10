//! 2D mathematical primitives: points, vectors, bounding boxes, and affine transforms.

use crate::error::{CoreError, Result};
use serde::{Deserialize, Serialize};

/// A 2D point with f64 precision.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn distance_to(&self, other: Self) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

/// A 2D displacement vector.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub dx: f64,
    pub dy: f64,
}

impl Vec2 {
    pub const ZERO: Self = Self { dx: 0.0, dy: 0.0 };

    #[must_use]
    pub const fn new(dx: f64, dy: f64) -> Self {
        Self { dx, dy }
    }

    #[must_use]
    pub fn length(&self) -> f64 {
        (self.dx.powi(2) + self.dy.powi(2)).sqrt()
    }
}

/// An axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub fn contains_point(&self, p: Point) -> bool {
        p.x >= self.x && p.x <= self.x + self.width && p.y >= self.y && p.y <= self.y + self.height
    }

    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

/// A 2D affine transformation matrix:
/// [ a  c  tx ]
/// [ b  d  ty ]
/// [ 0  0  1  ]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform2D {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub tx: f64,
    pub ty: f64,
}

impl Transform2D {
    /// Identity matrix.
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    #[must_use]
    pub const fn translation(tx: f64, ty: f64) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            tx,
            ty,
        }
    }

    #[must_use]
    pub const fn scale(sx: f64, sy: f64) -> Self {
        Self {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            tx: 0.0,
            ty: 0.0,
        }
    }

    #[must_use]
    pub fn transform_point(&self, p: Point) -> Point {
        Point {
            x: self.a * p.x + self.c * p.y + self.tx,
            y: self.b * p.x + self.d * p.y + self.ty,
        }
    }

    /// Compose two transforms: `self.concat(other)` applies `other`
    /// first and then `self`, matching the documented `A * B * p`
    /// convention.
    #[must_use]
    pub fn concat(self, other: Self) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            tx: self.a * other.tx + self.c * other.ty + self.tx,
            ty: self.b * other.tx + self.d * other.ty + self.ty,
        }
    }

    /// Invert this transform. Singular or numerically unsafe matrices
    /// return `None`: the Core refuses to hand out an inverse that
    /// would silently corrupt geometry.
    #[must_use]
    pub fn inverse(&self) -> Option<Self> {
        let det = self.a * self.d - self.b * self.c;
        if !det.is_finite() || det == 0.0 {
            return None;
        }
        // `a`, `b`, `c`, `d` below are the entries of the *inverse*
        // 2×2 block; the translation follows from `-(A⁻¹ · t)`.
        let a = self.d / det;
        let d = self.a / det;
        let b = -self.b / det;
        let c = -self.c / det;
        let candidate = Self {
            a,
            c,
            b,
            d,
            tx: -(a * self.tx + c * self.ty),
            ty: -(b * self.tx + d * self.ty),
        };
        if !candidate.is_numerically_safe() {
            return None;
        }
        Some(candidate)
    }

    /// True when every coefficient is finite and scales stay in a
    /// range that survives typical document coordinate arithmetic.
    #[must_use]
    pub fn is_numerically_safe(&self) -> bool {
        const LIMIT: f64 = 1e12;
        [self.a, self.b, self.c, self.d, self.tx, self.ty]
            .iter()
            .all(|coefficient| coefficient.is_finite() && coefficient.abs() <= LIMIT)
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// A 2D extent (width/height) with its own semantics: unlike [`Vec2`],
/// a size is never negative and has no direction.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size2 {
    pub width: f64,
    pub height: f64,
}

impl Size2 {
    /// Build a size, rejecting negative or non-finite components.
    /// Zero is allowed: a degenerate but valid extent.
    pub fn new(width: f64, height: f64) -> Result<Self> {
        if !width.is_finite() || !height.is_finite() {
            return Err(CoreError::InvariantViolation(format!(
                "non-finite size rejected: {width}x{height}"
            )));
        }
        if width < 0.0 || height < 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "negative size rejected: {width}x{height}"
            )));
        }
        Ok(Self { width, height })
    }

    /// True when either extent is zero.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.width == 0.0 || self.height == 0.0
    }
}

/// An angle in radians. Finite values only; normalization for display
/// or evaluation is the caller's responsibility.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Angle(pub f64);

impl Angle {
    /// Build an angle in radians, rejecting non-finite values.
    pub fn new(radians: f64) -> Result<Self> {
        if !radians.is_finite() {
            return Err(CoreError::InvariantViolation(format!(
                "non-finite angle rejected: {radians}"
            )));
        }
        Ok(Self(radians + 0.0))
    }

    /// Build an angle from degrees.
    pub fn from_degrees(degrees: f64) -> Result<Self> {
        Self::new(degrees.to_radians())
    }

    /// The angle in radians.
    #[must_use]
    pub fn radians(self) -> f64 {
        self.0
    }
}

/// A context-specific comparison tolerance.
///
/// There is no universal `EPSILON`: coincidence, flattening, boolean,
/// hit-test, snapping and inverse-transform comparisons each carry
/// their own tolerance. A tolerance only decides whether two values
/// count as coincident for one algorithm; it never mutates geometry.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Tolerance(pub f64);

impl Tolerance {
    /// Build a tolerance; it must be finite and non-negative.
    pub fn new(value: f64) -> Result<Self> {
        if !value.is_finite() || value < 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid tolerance rejected: {value}"
            )));
        }
        Ok(Self(value))
    }

    /// True when `|a - b| <= tolerance`.
    #[must_use]
    pub fn close_enough(self, a: f64, b: f64) -> bool {
        (a - b).abs() <= self.0
    }
}

/// Canonicalize a float for stable representation: `-0.0` becomes
/// `0.0`. Non-finite values pass through unchanged; rejecting them
/// stays the caller's boundary responsibility.
#[must_use]
pub fn canonicalize(value: f64) -> f64 {
    if value == 0.0 {
        0.0
    } else {
        value
    }
}

/// Convert a screen-space tolerance (pixels) into document units for
/// the current view scale, so handles stay clickable at any zoom:
///
/// ```text
/// document_tolerance = screen_tolerance_px / view_scale
/// ```
///
/// Returns `None` for a non-finite, zero or negative view scale.
#[must_use]
pub fn document_tolerance(screen_tolerance_px: f64, view_scale: f64) -> Option<f64> {
    if !screen_tolerance_px.is_finite() || !view_scale.is_finite() || view_scale <= 0.0 {
        return None;
    }
    Some(screen_tolerance_px / view_scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_translation_and_bounds() {
        let t = Transform2D::translation(10.0, 20.0);
        let p = Point::new(5.0, 5.0);
        let transformed = t.transform_point(p);
        assert_eq!(transformed, Point::new(15.0, 25.0));

        let rect = Rect::new(0.0, 0.0, 50.0, 50.0);
        assert!(rect.contains_point(Point::new(25.0, 25.0)));
        assert!(!rect.contains_point(Point::new(100.0, 100.0)));
    }

    #[test]
    fn test_size_rejects_negative_and_non_finite() {
        assert!(Size2::new(10.0, 20.0).is_ok());
        assert!(Size2::new(0.0, 0.0).is_ok());
        assert!(Size2::new(-1.0, 5.0).is_err());
        assert!(Size2::new(5.0, f64::NAN).is_err());
        assert!(Size2::new(f64::INFINITY, 5.0).is_err());
        assert!(Size2::new(10.0, 0.0).expect("zero height").is_empty());
        assert!(!Size2::new(10.0, 5.0).expect("valid").is_empty());
    }

    #[test]
    fn test_angle_rejects_non_finite() {
        assert_eq!(
            Angle::from_degrees(180.0).expect("valid").radians(),
            std::f64::consts::PI
        );
        assert!(Angle::new(f64::NAN).is_err());
        assert!(Angle::new(f64::INFINITY).is_err());
    }

    #[test]
    fn test_tolerance_is_contextual_not_universal() {
        let coincidence = Tolerance::new(1e-5).expect("valid");
        // Spec example: algorithmic coincidence without mutating geometry.
        assert!(coincidence.close_enough(10.0, 10.000005));
        assert!(!coincidence.close_enough(10.0, 10.001));
        assert!(Tolerance::new(-1.0).is_err());
        assert!(Tolerance::new(f64::NAN).is_err());
    }

    #[test]
    fn test_canonicalize_folds_negative_zero() {
        assert_eq!(canonicalize(-0.0), 0.0);
        assert!(canonicalize(-0.0).is_sign_positive());
        assert_eq!(canonicalize(3.25), 3.25);
    }

    #[test]
    fn test_document_tolerance_scales_with_zoom() {
        assert_eq!(document_tolerance(4.0, 2.0), Some(2.0));
        assert_eq!(document_tolerance(4.0, 0.0), None);
        assert_eq!(document_tolerance(4.0, -1.0), None);
        assert_eq!(document_tolerance(f64::NAN, 1.0), None);
    }

    #[test]
    fn concat_applies_other_first() {
        let shift = Transform2D::translation(10.0, 0.0);
        let scale = Transform2D::scale(2.0, 2.0);
        let point = Point::new(1.0, 1.0);
        // scale.concat(shift): shift first, then scale.
        let composed = scale.concat(shift).transform_point(point);
        assert_eq!(composed, Point::new(22.0, 2.0));
        assert_eq!(Transform2D::IDENTITY.concat(shift), shift);
    }

    #[test]
    fn inverse_round_trips_and_rejects_singular() {
        let transform = Transform2D {
            a: 2.0,
            c: 1.5,
            b: -0.5,
            d: 3.0,
            tx: 12.0,
            ty: -7.0,
        };
        let inverse = transform.inverse().expect("invertible");
        let point = Point::new(5.0, 9.0);
        let restored = inverse.transform_point(transform.transform_point(point));
        assert!((restored.x - point.x).abs() < 1e-9, "{restored:?}");
        assert!((restored.y - point.y).abs() < 1e-9, "{restored:?}");
        // Collinear columns: no inverse, no silent corruption.
        let singular = Transform2D {
            a: 2.0,
            c: 2.0,
            b: 3.0,
            d: 3.0,
            tx: 0.0,
            ty: 0.0,
        };
        assert!(singular.inverse().is_none());
        // Identity is its own inverse.
        assert_eq!(Transform2D::IDENTITY.inverse(), Some(Transform2D::IDENTITY));
    }

    #[test]
    fn numerical_safety_rejects_non_finite() {
        assert!(Transform2D::IDENTITY.is_numerically_safe());
        let unsafe_matrix = Transform2D {
            a: f64::INFINITY,
            ..Transform2D::IDENTITY
        };
        assert!(!unsafe_matrix.is_numerically_safe());
        assert!(unsafe_matrix.inverse().is_none());
    }
}
