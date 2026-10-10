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
        // `-0.0 + 0.0` is `0.0`: construction canonicalizes the only
        // float with two representations.
        Self {
            x: x + 0.0,
            y: y + 0.0,
        }
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
        Self {
            dx: dx + 0.0,
            dy: dy + 0.0,
        }
    }

    #[must_use]
    pub fn length(&self) -> f64 {
        (self.dx.powi(2) + self.dy.powi(2)).sqrt()
    }
}

/// An axis-aligned bounding box.
///
/// Storage is origin plus non-negative extents, always canonical: a
/// negative extent flips to the canonical corner, so drag-inverted
/// input never becomes a second implicit representation. Every
/// constructor normalizes; use [`Rect::is_finite`] at domain
/// boundaries since non-finite values stay representable here and
/// invalid there.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    /// Build a rect, canonicalizing negative extents and `-0.0`.
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        // Const-compatible canonicalization: flip inverted extents,
        // fold `-0.0` into `0.0`.
        let (x, width) = if width < 0.0 {
            (x + width + 0.0, -width)
        } else {
            (x + 0.0, width + 0.0)
        };
        let (y, height) = if height < 0.0 {
            (y + height + 0.0, -height)
        } else {
            (y + 0.0, height + 0.0)
        };
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Smallest rect containing every point; empty for empty input.
    #[must_use]
    pub fn from_points(points: &[Point]) -> Self {
        let mut iter = points.iter();
        let Some(first) = iter.next() else {
            return Self::new(0.0, 0.0, 0.0, 0.0);
        };
        let (mut min_x, mut min_y) = (first.x, first.y);
        let (mut max_x, mut max_y) = (first.x, first.y);
        for point in iter {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }
        Self::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// Rect at an origin with the given size, canonicalized.
    #[must_use]
    pub fn from_origin_size(origin: Point, size: Size2) -> Self {
        Self::new(origin.x, origin.y, size.width, size.height)
    }

    /// Canonical minimum corner.
    #[must_use]
    pub fn min(self) -> Point {
        Point::new(self.x, self.y)
    }

    /// Canonical maximum corner.
    #[must_use]
    pub fn max(self) -> Point {
        Point::new(self.x + self.width, self.y + self.height)
    }

    /// Center point.
    #[must_use]
    pub fn center(self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Smallest rect containing both.
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        let min_x = self.x.min(other.x);
        let min_y = self.y.min(other.y);
        let max_x = (self.x + self.width).max(other.x + other.width);
        let max_y = (self.y + self.height).max(other.y + other.height);
        Self::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// Overlap area, or an empty rect at the overlap origin.
    #[must_use]
    pub fn intersection(self, other: Self) -> Self {
        let min_x = self.x.max(other.x);
        let min_y = self.y.max(other.y);
        let max_x = (self.x + self.width).min(other.x + other.width);
        let max_y = (self.y + self.height).min(other.y + other.height);
        Self::new(
            min_x,
            min_y,
            (max_x - min_x).max(0.0),
            (max_y - min_y).max(0.0),
        )
    }

    /// Grow by `delta` on every side; negative deltas shrink.
    #[must_use]
    pub fn expand(self, delta: f64) -> Self {
        Self::new(
            self.x - delta,
            self.y - delta,
            self.width + delta * 2.0,
            self.height + delta * 2.0,
        )
    }

    #[must_use]
    pub fn contains_point(&self, p: Point) -> bool {
        p.x >= self.x && p.x <= self.x + self.width && p.y >= self.y && p.y <= self.y + self.height
    }

    /// True when `other` lies fully inside, edges inclusive.
    #[must_use]
    pub fn contains_rect(&self, other: &Self) -> bool {
        self.contains_point(other.min()) && self.contains_point(other.max())
    }

    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// True when either extent is zero.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.width == 0.0 || self.height == 0.0
    }

    /// True when every component is finite. Non-finite rects are
    /// representable but invalid at domain boundaries.
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.width.is_finite()
            && self.height.is_finite()
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
            tx: tx + 0.0,
            ty: ty + 0.0,
        }
    }

    #[must_use]
    pub const fn scale(sx: f64, sy: f64) -> Self {
        Self {
            a: sx + 0.0,
            b: 0.0,
            c: 0.0,
            d: sy + 0.0,
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

    /// Invert with an explicit tolerance: the only strict inverse.
    /// `Singular` for a zero determinant, `NearSingular` when the
    /// determinant is finite but within `tolerance` of zero, and
    /// `NonFinite` before any division happens. Never returns NaN.
    pub fn try_inverse(&self, tolerance: Tolerance) -> std::result::Result<Self, TransformError> {
        let det = self.a * self.d - self.b * self.c;
        if !det.is_finite()
            || !self.a.is_finite()
            || !self.b.is_finite()
            || !self.c.is_finite()
            || !self.d.is_finite()
            || !self.tx.is_finite()
            || !self.ty.is_finite()
        {
            return Err(TransformError::NonFinite);
        }
        if det == 0.0 {
            return Err(TransformError::Singular);
        }
        if det.abs() <= tolerance.0 {
            return Err(TransformError::NearSingular);
        }
        let a = self.d / det;
        let d = self.a / det;
        let b = -self.b / det;
        let c = -self.c / det;
        Ok(Self {
            a,
            c,
            b,
            d,
            tx: -(a * self.tx + c * self.ty),
            ty: -(b * self.tx + d * self.ty),
        })
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

/// Why a transform has no inverse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformError {
    /// A coefficient is NaN or infinite: checked before any division.
    NonFinite,
    /// Zero determinant: at least one dimension collapsed.
    Singular,
    /// Finite but within the caller's tolerance of zero: the inverse
    /// exists mathematically but is numerically unsafe.
    NearSingular,
}

/// Translation, rotation, scale and shear separated from one matrix.
///
/// Deterministic policy: rotation comes from the first column
/// (`atan2(b, a)`), `scale_x` is its non-negative length, and any
/// reflection lands on a negative `scale_y`. Shear is the x-shear
/// factor applied before rotation. [`Transform2D::recompose`] inverts
/// this exact policy, so decompose/recompose round-trips every
/// well-conditioned matrix.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecomposedTransform {
    pub translation: Vec2,
    pub rotation: Angle,
    pub scale: Vec2,
    pub shear_x: f64,
}

impl Transform2D {
    /// Split into translation, rotation, scale and shear following
    /// the documented policy. Fails on non-finite, singular and
    /// near-degenerate matrices instead of guessing.
    pub fn decompose(&self) -> std::result::Result<DecomposedTransform, TransformError> {
        let Transform2D { a, b, c, d, tx, ty } = *self;
        for coefficient in [a, b, c, d, tx, ty] {
            if !coefficient.is_finite() {
                return Err(TransformError::NonFinite);
            }
        }
        let det = a * d - b * c;
        if det == 0.0 {
            return Err(TransformError::Singular);
        }
        let scale_x = a.hypot(b);
        if scale_x == 0.0 || !scale_x.is_finite() {
            return Err(TransformError::Singular);
        }
        let rotation = Angle(b.atan2(a) + 0.0);
        let (sin, cos) = rotation.radians().sin_cos();
        // First row of R^-1 * M is [scale_x, shear * scale_y]; only
        // the second component carries new information.
        let row_y = c * cos + d * sin;
        let scale_y = det / scale_x;
        if !scale_y.is_finite() {
            return Err(TransformError::NearSingular);
        }
        let shear_x = if scale_y == 0.0 { 0.0 } else { row_y / scale_y };
        if !shear_x.is_finite() || !row_y.is_finite() {
            return Err(TransformError::NearSingular);
        }
        Ok(DecomposedTransform {
            translation: Vec2::new(tx, ty),
            rotation,
            scale: Vec2::new(scale_x, scale_y),
            shear_x,
        })
    }

    /// Rebuild the matrix this decomposition came from: translate
    /// after rotating the sheared, scaled axes.
    #[must_use]
    pub fn recompose(parts: &DecomposedTransform) -> Self {
        let (sin, cos) = parts.rotation.radians().sin_cos();
        let sx = parts.scale.dx;
        let sy = parts.scale.dy;
        let k = parts.shear_x;
        Self {
            a: cos * sx,
            b: sin * sx,
            c: cos * k * sy - sin * sy,
            d: sin * k * sy + cos * sy,
            tx: parts.translation.dx,
            ty: parts.translation.dy,
        }
    }
}

/// Explicit seed for authorial randomness: brush jitter, scatter,
/// noise and procedural patterns reproduce from input, parameters
/// and seed. System time never seeds a static authored result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RandomSeed(pub u64);

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

/// Per-side expansion or margin. The unit comes from context; never
/// mix screen pixels and document units implicitly.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Insets {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl Insets {
    /// All sides zero.
    pub const ZERO: Self = Self {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
    };

    /// Build insets, rejecting non-finite components. Negative sides
    /// stay representable here; owners with non-negative semantics
    /// (page margins, bleed) reject them at their own boundary.
    pub fn new(left: f64, top: f64, right: f64, bottom: f64) -> Result<Self> {
        for side in [left, top, right, bottom] {
            if !side.is_finite() {
                return Err(CoreError::InvariantViolation(format!(
                    "non-finite inset rejected: {side}"
                )));
            }
        }
        Ok(Self {
            left,
            top,
            right,
            bottom,
        })
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
    /// `-0.0` canonicalizes to `0.0`.
    pub fn new(value: f64) -> Result<Self> {
        if !value.is_finite() || value < 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid tolerance rejected: {value}"
            )));
        }
        Ok(Self(value + 0.0))
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
    fn test_insets_reject_non_finite() {
        assert_eq!(Insets::ZERO, Insets::new(0.0, 0.0, 0.0, 0.0).expect("zero"));
        assert!(Insets::new(1.0, f64::NAN, 0.0, 0.0).is_err());
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
    fn rect_constructors_canonicalize() {
        // Negative extents flip to the canonical corner.
        let rect = Rect::new(10.0, 10.0, -4.0, -6.0);
        assert_eq!(
            (rect.x, rect.y, rect.width, rect.height),
            (6.0, 4.0, 4.0, 6.0)
        );
        assert_eq!(Rect::new(1.0, 2.0, 3.0, 4.0).min(), Point::new(1.0, 2.0));
        assert_eq!(Rect::new(1.0, 2.0, 3.0, 4.0).max(), Point::new(4.0, 6.0));
        assert_eq!(Rect::new(1.0, 2.0, 3.0, 4.0).center(), Point::new(2.5, 4.0));
        assert!(Rect::new(0.0, 0.0, 0.0, 5.0).is_empty());
        assert!(!Rect::new(0.0, 0.0, 1.0, 1.0).is_empty());
    }

    #[test]
    fn rect_set_operations_match_their_contract() {
        let outer = Rect::new(0.0, 0.0, 10.0, 10.0);
        let inner = Rect::new(2.0, 2.0, 3.0, 3.0);
        let overlap = Rect::new(8.0, 8.0, 5.0, 5.0);
        assert!(outer.contains_rect(&inner));
        assert!(!inner.contains_rect(&outer));
        assert_eq!(outer.union(inner), outer);
        let cut = outer.intersection(overlap);
        assert_eq!((cut.x, cut.y, cut.width, cut.height), (8.0, 8.0, 2.0, 2.0));
        let grown = inner.expand(1.0);
        assert_eq!(
            (grown.x, grown.y, grown.width, grown.height),
            (1.0, 1.0, 5.0, 5.0)
        );
        assert_eq!(
            Rect::from_points(&[Point::new(3.0, 1.0), Point::new(1.0, 4.0)]),
            Rect::new(1.0, 1.0, 2.0, 3.0)
        );
        assert!(Rect::new(0.0, 0.0, f64::NAN, 1.0).is_finite() == false);
    }

    #[test]
    fn try_inverse_names_its_failure() {
        let singular = Transform2D {
            a: 2.0,
            c: 2.0,
            b: 3.0,
            d: 3.0,
            tx: 0.0,
            ty: 0.0,
        };
        assert_eq!(
            singular.try_inverse(Tolerance::new(1e-9).expect("valid")),
            Err(TransformError::Singular)
        );
        let tiny = Transform2D::scale(1e-12, 1.0);
        assert_eq!(
            tiny.try_inverse(Tolerance::new(1e-9).expect("valid")),
            Err(TransformError::NearSingular)
        );
        let wild = Transform2D {
            a: f64::INFINITY,
            ..Transform2D::IDENTITY
        };
        assert_eq!(
            wild.try_inverse(Tolerance::new(1e-9).expect("valid")),
            Err(TransformError::NonFinite)
        );
        let fine = Transform2D::translation(3.0, -2.0);
        let back = fine
            .try_inverse(Tolerance::new(1e-12).expect("valid"))
            .expect("invertible");
        assert_eq!(
            back.transform_point(fine.transform_point(Point::new(1.0, 1.0))),
            Point::new(1.0, 1.0)
        );
    }

    #[test]
    fn decompose_recompose_round_trips() {
        let matrix = Transform2D {
            a: 2.0,
            c: 1.5,
            b: -0.5,
            d: 3.0,
            tx: 12.0,
            ty: -7.0,
        };
        let parts = matrix.decompose().expect("decomposes");
        let rebuilt = Transform2D::recompose(&parts);
        for (got, want) in [
            (rebuilt.a, matrix.a),
            (rebuilt.b, matrix.b),
            (rebuilt.c, matrix.c),
            (rebuilt.d, matrix.d),
            (rebuilt.tx, matrix.tx),
            (rebuilt.ty, matrix.ty),
        ] {
            assert!((got - want).abs() < 1e-9, "{got} != {want}");
        }
        // Reflection lands on a negative y scale, by policy.
        let mirror = Transform2D::scale(-2.0, 3.0);
        let parts = mirror.decompose().expect("decomposes");
        assert!(parts.scale.dy < 0.0, "{parts:?}");
        assert!(Transform2D {
            a: 0.0,
            c: 0.0,
            b: 0.0,
            d: 0.0,
            tx: 0.0,
            ty: 0.0
        }
        .decompose()
        .is_err());
    }

    #[test]
    fn random_seed_round_trips() {
        let seed = RandomSeed(0x9E37_79B9);
        let json = serde_json::to_string(&seed).expect("serializes");
        let back: RandomSeed = serde_json::from_str(&json).expect("parses");
        assert_eq!(back, seed);
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
