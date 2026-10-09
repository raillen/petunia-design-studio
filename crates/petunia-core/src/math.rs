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
}
