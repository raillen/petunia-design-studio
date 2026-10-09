//! Reusable paint resources: swatches and gradients.
//!
//! A swatch is a document-level reusable value. Gradient stops,
//! interpolation, spread and coordinate space persist here; the
//! renderer never infers them.

use crate::color::ColorValue;
use crate::error::{CoreError, Result};
use crate::id::SwatchId;
use crate::math::Point;
use serde::{Deserialize, Serialize};

/// Linked-versus-local paint choice: a literal value or a live
/// reference to a swatch. The choice is explicit; a local copy stops
/// depending on the swatch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColorSource {
    Value(ColorValue),
    Swatch(SwatchId),
}

/// A reusable document swatch: a color or a gradient under a stable ID.
/// Linked uses reference the ID so edits propagate; local copies stop
/// depending on the swatch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Swatch {
    pub id: SwatchId,
    pub name: String,
    pub value: SwatchValue,
}

/// What a swatch holds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SwatchValue {
    Color(ColorValue),
    Gradient(Gradient),
}

/// One gradient stop. Offset is normalized in `0..1`; midpoint pins
/// the perceptual middle between adjacent stops and is defined in
/// `0..1` as well.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradientStop {
    pub offset: f32,
    pub color: ColorSource,
    pub midpoint: f32,
}

impl GradientStop {
    /// Offsets and midpoints must be finite values in `0..1`.
    pub fn new(offset: f32, color: ColorSource, midpoint: f32) -> Result<Self> {
        for (label, value) in [("offset", offset), ("midpoint", midpoint)] {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(CoreError::InvariantViolation(format!(
                    "gradient {label} out of 0..=1: {value}"
                )));
            }
        }
        Ok(Self {
            offset,
            color,
            midpoint,
        })
    }
}

/// Persisted interpolation policy. `LinearRgb` is the initial technical
/// default; perceptual modes never change the stop structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GradientInterpolation {
    #[default]
    LinearRgb,
    EncodedRgb,
    Lab,
    Oklab,
}

/// What happens outside the stop range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GradientSpread {
    #[default]
    Pad,
    Repeat,
    Reflect,
}

/// Which coordinate space the gradient geometry lives in. Object space
/// follows the object; document space stays document-referenced, so a
/// transform never has to guess whether the gradient follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PaintSpace {
    #[default]
    Object,
    Document,
}

/// Linear and radial gradient geometry in the declared space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GradientGeometry {
    Linear { start: Point, end: Point },
    Radial { center: Point, radius: f64 },
}

impl GradientGeometry {
    /// Radial radii must be finite and non-negative.
    pub fn radial(center: Point, radius: f64) -> Result<Self> {
        if !radius.is_finite() || radius < 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid radial gradient radius rejected: {radius}"
            )));
        }
        Ok(Self::Radial { center, radius })
    }
}

/// A gradient value: stops plus the policies a renderer needs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gradient {
    pub stops: Vec<GradientStop>,
    pub interpolation: GradientInterpolation,
    pub spread: GradientSpread,
    pub space: PaintSpace,
    pub geometry: GradientGeometry,
}

impl Gradient {
    /// At least one stop is required; stop order stays authorial.
    pub fn new(
        stops: Vec<GradientStop>,
        interpolation: GradientInterpolation,
        spread: GradientSpread,
        space: PaintSpace,
        geometry: GradientGeometry,
    ) -> Result<Self> {
        if stops.is_empty() {
            return Err(CoreError::InvariantViolation(
                "gradient needs at least one stop".to_string(),
            ));
        }
        Ok(Self {
            stops,
            interpolation,
            spread,
            space,
            geometry,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{BuiltinColorSpace, ColorSpaceRef, ProcessColor, ProcessColorValue, Rgba};

    fn solid() -> ColorSource {
        ColorSource::Value(ColorValue::Process(ProcessColor {
            value: ProcessColorValue::Rgb(Rgba {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                alpha: 1.0,
            }),
            space: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
        }))
    }

    fn stop(offset: f32) -> GradientStop {
        GradientStop::new(offset, solid(), 0.5).expect("valid test stop")
    }

    #[test]
    fn stop_rejects_out_of_range_positions() {
        assert!(GradientStop::new(0.5, solid(), 0.5).is_ok());
        assert!(GradientStop::new(1.5, solid(), 0.5).is_err());
        assert!(GradientStop::new(0.5, solid(), f32::NAN).is_err());
    }

    #[test]
    fn gradient_needs_stops_and_valid_geometry() {
        let geometry = GradientGeometry::Linear {
            start: Point::new(0.0, 0.0),
            end: Point::new(100.0, 0.0),
        };
        assert!(Gradient::new(
            vec![stop(0.0), stop(1.0)],
            GradientInterpolation::LinearRgb,
            GradientSpread::Pad,
            PaintSpace::Object,
            geometry,
        )
        .is_ok());
        assert!(Gradient::new(
            vec![],
            GradientInterpolation::LinearRgb,
            GradientSpread::Pad,
            PaintSpace::Object,
            geometry,
        )
        .is_err());
        assert!(GradientGeometry::radial(Point::new(0.0, 0.0), -5.0).is_err());
    }

    #[test]
    fn swatch_round_trip_preserves_gradient() {
        let swatch = Swatch {
            id: SwatchId::new_v4(),
            name: "Brand".to_string(),
            value: SwatchValue::Gradient(
                Gradient::new(
                    vec![stop(0.0), stop(1.0)],
                    GradientInterpolation::Lab,
                    GradientSpread::Reflect,
                    PaintSpace::Document,
                    GradientGeometry::Linear {
                        start: Point::new(0.0, 0.0),
                        end: Point::new(50.0, 50.0),
                    },
                )
                .expect("valid"),
            ),
        };
        let json = serde_json::to_string(&swatch).expect("serializable");
        let back: Swatch = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(back, swatch);
    }
}
