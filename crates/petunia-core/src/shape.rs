//! Authorial parametric vector shapes.
//!
//! A [`ParametricShape`] describes high-level intent (size, sides, radii,
//! arcs) and stays parametric until an explicit `Convert to Curves`
//! command materializes it. Evaluation into a path is Engine
//! work; this module persists only valid finite parameters.

use crate::error::{CoreError, Result};
use crate::math::{Angle, Size2, Vec2};
use serde::{Deserialize, Serialize};

/// Per-corner radii of a rectangle, each with non-negative finite X/Y.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CornerRadii {
    pub top_left: Vec2,
    pub top_right: Vec2,
    pub bottom_right: Vec2,
    pub bottom_left: Vec2,
}

impl CornerRadii {
    /// Uniform radii for all four corners.
    pub fn uniform(radius: Vec2) -> Result<Self> {
        Self::new(radius, radius, radius, radius)
    }

    /// Independent radii per corner; each component must be finite
    /// and non-negative.
    pub fn new(
        top_left: Vec2,
        top_right: Vec2,
        bottom_right: Vec2,
        bottom_left: Vec2,
    ) -> Result<Self> {
        for corner in [top_left, top_right, bottom_right, bottom_left] {
            if !corner.dx.is_finite() || !corner.dy.is_finite() {
                return Err(CoreError::InvariantViolation(format!(
                    "non-finite corner radius rejected: {corner:?}"
                )));
            }
            if corner.dx < 0.0 || corner.dy < 0.0 {
                return Err(CoreError::InvariantViolation(format!(
                    "negative corner radius rejected: {corner:?}"
                )));
            }
        }
        Ok(Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        })
    }
}

/// Rectangle intent: size plus corner radii. A rounded rectangle is a
/// rectangle with radii, not a separate persistent type.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RectangleSpec {
    pub size: Size2,
    pub corners: CornerRadii,
}

impl RectangleSpec {
    pub fn new(size: Size2, corners: CornerRadii) -> Self {
        Self { size, corners }
    }
}

/// How an elliptical arc closes (or stays open).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EllipseArc {
    /// Full ellipse; a circle is an ellipse with equal radii.
    Full,
    /// Open arc between start and start + sweep.
    Open { start: Angle, sweep: Angle },
    /// Arc closed by a straight segment between endpoints.
    Chord { start: Angle, sweep: Angle },
    /// Arc closed through the center.
    Pie { start: Angle, sweep: Angle },
}

/// Ellipse intent: radii plus arc mode, in local space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EllipseSpec {
    pub radii: Vec2,
    pub arc: EllipseArc,
}

impl EllipseSpec {
    /// Radii must be finite and non-negative; angles are validated
    /// by [`Angle`] itself.
    pub fn new(radii: Vec2, arc: EllipseArc) -> Result<Self> {
        if !radii.dx.is_finite() || !radii.dy.is_finite() {
            return Err(CoreError::InvariantViolation(format!(
                "non-finite ellipse radii rejected: {radii:?}"
            )));
        }
        if radii.dx < 0.0 || radii.dy < 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "negative ellipse radii rejected: {radii:?}"
            )));
        }
        Ok(Self { radii, arc })
    }

    /// True for a full ellipse with equal radii.
    #[must_use]
    pub fn is_circle(self) -> bool {
        matches!(self.arc, EllipseArc::Full) && self.radii.dx == self.radii.dy
    }
}

/// Regular polygon intent: vertices spread uniformly over the
/// circumscribed circle (`angle_i = rotation + i × 2π / sides`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PolygonSpec {
    pub sides: u32,
    pub radius: f64,
    pub rotation: Angle,
}

impl PolygonSpec {
    /// Requires `sides >= 3` and a finite non-negative radius.
    pub fn new(sides: u32, radius: f64, rotation: Angle) -> Result<Self> {
        if sides < 3 {
            return Err(CoreError::InvariantViolation(format!(
                "polygon needs at least 3 sides, got {sides}"
            )));
        }
        if !radius.is_finite() || radius < 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid polygon radius rejected: {radius}"
            )));
        }
        Ok(Self {
            sides,
            radius,
            rotation,
        })
    }
}

/// Star intent: vertices alternate outer/inner radius. Ratios above
/// one stay mathematically valid and are not clamped.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StarSpec {
    pub points: u32,
    pub outer_radius: f64,
    pub inner_ratio: f64,
    pub rotation: Angle,
}

impl StarSpec {
    /// Requires `points >= 2`, finite non-negative outer radius and
    /// finite non-negative ratio.
    pub fn new(points: u32, outer_radius: f64, inner_ratio: f64, rotation: Angle) -> Result<Self> {
        if points < 2 {
            return Err(CoreError::InvariantViolation(format!(
                "star needs at least 2 points, got {points}"
            )));
        }
        if !outer_radius.is_finite() || outer_radius < 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid star outer radius rejected: {outer_radius}"
            )));
        }
        if !inner_ratio.is_finite() || inner_ratio < 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid star inner ratio rejected: {inner_ratio}"
            )));
        }
        Ok(Self {
            points,
            outer_radius,
            inner_ratio,
            rotation,
        })
    }
}

/// Authorial shape intent. The object transform lives on the scene
/// node, not in the spec; derived paths, bounds and tessellation are
/// never persisted here.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ParametricShape {
    Rectangle(RectangleSpec),
    Ellipse(EllipseSpec),
    Polygon(PolygonSpec),
    Star(StarSpec),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Angle;

    fn angle(degrees: f64) -> Angle {
        Angle::from_degrees(degrees).expect("valid test angle")
    }

    fn radii(dx: f64, dy: f64) -> Vec2 {
        Vec2::new(dx, dy)
    }

    #[test]
    fn rectangle_accepts_valid_specs() {
        let size = Size2::new(100.0, 50.0).expect("valid");
        let corners = CornerRadii::uniform(radii(8.0, 8.0)).expect("valid");
        let shape = ParametricShape::Rectangle(RectangleSpec::new(size, corners));
        assert!(matches!(shape, ParametricShape::Rectangle(_)));
    }

    #[test]
    fn corner_radii_reject_negative_and_non_finite() {
        assert!(CornerRadii::uniform(radii(-1.0, 0.0)).is_err());
        assert!(CornerRadii::uniform(radii(f64::NAN, 0.0)).is_err());
        assert!(CornerRadii::new(
            radii(1.0, 1.0),
            radii(1.0, 1.0),
            radii(1.0, f64::INFINITY),
            radii(1.0, 1.0),
        )
        .is_err());
    }

    #[test]
    fn ellipse_circle_is_full_with_equal_radii() {
        let circle = EllipseSpec::new(radii(10.0, 10.0), EllipseArc::Full).expect("valid");
        assert!(circle.is_circle());
        let ellipse = EllipseSpec::new(radii(10.0, 5.0), EllipseArc::Full).expect("valid");
        assert!(!ellipse.is_circle());
        let pie = EllipseSpec::new(
            radii(10.0, 10.0),
            EllipseArc::Pie {
                start: angle(0.0),
                sweep: angle(90.0),
            },
        )
        .expect("valid");
        assert!(!pie.is_circle());
        assert!(EllipseSpec::new(radii(-1.0, 1.0), EllipseArc::Full).is_err());
    }

    #[test]
    fn polygon_enforces_minimum_sides_and_radius() {
        assert!(PolygonSpec::new(5, 10.0, angle(0.0)).is_ok());
        assert!(PolygonSpec::new(2, 10.0, angle(0.0)).is_err());
        assert!(PolygonSpec::new(5, -1.0, angle(0.0)).is_err());
        assert!(PolygonSpec::new(5, f64::NAN, angle(0.0)).is_err());
    }

    #[test]
    fn star_enforces_minimum_points_without_ratio_clamp() {
        // Ratios above one stay mathematically valid per spec.
        assert!(StarSpec::new(5, 10.0, 1.5, angle(0.0)).is_ok());
        assert!(StarSpec::new(1, 10.0, 0.5, angle(0.0)).is_err());
        assert!(StarSpec::new(5, -1.0, 0.5, angle(0.0)).is_err());
        assert!(StarSpec::new(5, 10.0, -0.5, angle(0.0)).is_err());
    }

    #[test]
    fn shape_serialization_round_trip_preserves_params() {
        let shape = ParametricShape::Star(StarSpec::new(5, 10.0, 0.5, angle(0.0)).expect("valid"));
        let json = serde_json::to_string(&shape).expect("serializable");
        let back: ParametricShape = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(back, shape);
    }
}
