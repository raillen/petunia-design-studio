//! Encapsulated color management boundary.
//!
//! ICC parsing and full CMM transforms belong to the Color Management
//! Engine. This module owns the authorial side of that boundary:
//! transform descriptors with explicit source, destination and intent,
//! cache keys, and the exact built-in sRGB transfer functions. Any
//! pair needing an ICC profile or a not-yet-implemented space fails
//! with a typed error instead of guessing.

use crate::color::{
    BuiltinColorSpace, ColorSpaceRef, ProcessColor, ProcessColorValue, RenderingIntent, Rgba,
};
use crate::error::{CoreError, Result};
use serde::{Deserialize, Serialize};

/// One requested conversion: explicit source, destination and intent.
/// Assign-vs-convert semantics stay at the Command layer; this type
/// only describes the numeric mapping.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColorTransform {
    pub source: ColorSpaceRef,
    pub destination: ColorSpaceRef,
    pub intent: RenderingIntent,
}

impl ColorTransform {
    #[must_use]
    pub fn new(source: ColorSpaceRef, destination: ColorSpaceRef, intent: RenderingIntent) -> Self {
        Self {
            source,
            destination,
            intent,
        }
    }

    /// Stable key for transform caches: same triple, same mapping.
    #[must_use]
    pub fn cache_key(&self) -> (ColorSpaceRef, ColorSpaceRef, RenderingIntent) {
        (self.source.clone(), self.destination.clone(), self.intent)
    }

    /// Apply the mapping to a process color whose space matches the
    /// declared source. Built-in sRGB transfers are exact; anything
    /// needing ICC data or an unimplemented space errors explicitly.
    pub fn apply(&self, color: &ProcessColor) -> Result<ProcessColor> {
        if color.space != self.source {
            return Err(CoreError::InvariantViolation(format!(
                "transform source mismatch: color is {:?}, transform expects {:?}",
                color.space, self.source
            )));
        }
        let ProcessColorValue::Rgb(channels) = &color.value else {
            return Err(CoreError::UnsupportedTransform(
                "only RGB values convert through built-in transfers".to_string(),
            ));
        };
        let converted = match (&self.source, &self.destination) {
            (
                ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
                ColorSpaceRef::Builtin(BuiltinColorSpace::LinearSrgb),
            ) => Rgba {
                r: encoded_to_linear(channels.r),
                g: encoded_to_linear(channels.g),
                b: encoded_to_linear(channels.b),
                alpha: channels.alpha,
            },
            (
                ColorSpaceRef::Builtin(BuiltinColorSpace::LinearSrgb),
                ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
            ) => Rgba {
                r: linear_to_encoded(channels.r),
                g: linear_to_encoded(channels.g),
                b: linear_to_encoded(channels.b),
                alpha: channels.alpha,
            },
            (from, to) if from == to => *channels,
            _ => {
                return Err(CoreError::UnsupportedTransform(format!(
                    "no built-in mapping from {from:?} to {to:?}; ICC-backed pairs belong to the color engine",
                    from = self.source,
                    to = self.destination,
                )));
            }
        };
        Ok(ProcessColor {
            value: ProcessColorValue::Rgb(converted),
            space: self.destination.clone(),
        })
    }
}

/// Exact sRGB electro-optical transfer function (encoded → linear).
fn encoded_to_linear(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

/// Exact inverse transfer function (linear → encoded).
fn linear_to_encoded(channel: f32) -> f32 {
    if channel <= 0.0031308 {
        12.92 * channel
    } else {
        1.055 * channel.powf(1.0 / 2.4) - 0.055
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn srgb(r: f32, g: f32, b: f32) -> ProcessColor {
        ProcessColor {
            value: ProcessColorValue::Rgb(Rgba {
                r,
                g,
                b,
                alpha: 1.0,
            }),
            space: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
        }
    }

    fn transform(from: BuiltinColorSpace, to: BuiltinColorSpace) -> ColorTransform {
        ColorTransform::new(
            ColorSpaceRef::Builtin(from),
            ColorSpaceRef::Builtin(to),
            RenderingIntent::RelativeColorimetric,
        )
    }

    #[test]
    fn srgb_transfers_round_trip() {
        let forward = transform(BuiltinColorSpace::Srgb, BuiltinColorSpace::LinearSrgb);
        let linear = forward.apply(&srgb(0.5, 0.25, 1.0)).expect("supported");
        assert!((linear.as_rgb().expect("rgb").r - 0.214).abs() < 0.001);

        let back = transform(BuiltinColorSpace::LinearSrgb, BuiltinColorSpace::Srgb);
        let round_tripped = back.apply(&linear).expect("supported");
        let back_rgb = round_tripped.as_rgb().expect("rgb");
        for (a, b) in [(back_rgb.r, 0.5), (back_rgb.g, 0.25), (back_rgb.b, 1.0)] {
            assert!((a - b).abs() < 1e-5, "{a} != {b}");
        }
    }

    #[test]
    fn unsupported_pairs_fail_with_typed_errors() {
        let icc = ColorTransform::new(
            ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
            ColorSpaceRef::ExternalIcc {
                location: "printer.icc".to_string(),
            },
            RenderingIntent::Perceptual,
        );
        assert!(matches!(
            icc.apply(&srgb(1.0, 0.0, 0.0)),
            Err(CoreError::UnsupportedTransform(_))
        ));

        let mismatched = transform(BuiltinColorSpace::Srgb, BuiltinColorSpace::LinearSrgb);
        let linear_color = ProcessColor {
            space: ColorSpaceRef::Builtin(BuiltinColorSpace::LinearSrgb),
            ..srgb(1.0, 0.0, 0.0)
        };
        assert!(matches!(
            mismatched.apply(&linear_color),
            Err(CoreError::InvariantViolation(_))
        ));
    }

    #[test]
    fn cache_key_covers_source_destination_and_intent() {
        let first = transform(BuiltinColorSpace::Srgb, BuiltinColorSpace::LinearSrgb);
        let second = ColorTransform::new(
            ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
            ColorSpaceRef::Builtin(BuiltinColorSpace::LinearSrgb),
            RenderingIntent::Perceptual,
        );
        assert_eq!(first.cache_key(), first.clone().cache_key());
        assert_ne!(first.cache_key(), second.cache_key());
    }
}
