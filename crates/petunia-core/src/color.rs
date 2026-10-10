//! Color representations and color space metadata.

use crate::error::{CoreError, Result};
use crate::id::{ResourceId, SpotColorId};
use serde::{Deserialize, Serialize};

/// RGBA color with normalized f32 channels [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorRgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorRgba {
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    #[must_use]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates an RGBA color from 8-bit integers (0-255).
    #[must_use]
    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: f32::from(r) / 255.0,
            g: f32::from(g) / 255.0,
            b: f32::from(b) / 255.0,
            a: f32::from(a) / 255.0,
        }
    }
}

/// Color spaces supported by the creative engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ColorSpace {
    #[default]
    Srgb,
    DisplayP3,
    LinearSrgb,
}

/// Authorial RGB channels in `f32`. Straight alpha is persistent;
/// premultiplied alpha is a compositing representation only.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub alpha: f32,
}

/// Authorial CMYK channels in `f32`, each in `0..1`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Cmyka {
    pub c: f32,
    pub m: f32,
    pub y: f32,
    pub k: f32,
    pub alpha: f32,
}

/// Authorial CIE Lab value. The `a`/`b` axes keep their own semantic
/// ranges and are never renormalized to fit `0..1`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Laba {
    pub l: f32,
    pub a_axis: f32,
    pub b_axis: f32,
    pub alpha: f32,
}

/// Authorial gray value plus straight alpha.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Graya {
    pub gray: f32,
    pub alpha: f32,
}

/// Process color channels: numeric value plus an explicit space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ProcessColorValue {
    Rgb(Rgba),
    Cmyk(Cmyka),
    Lab(Laba),
    Gray(Graya),
}

/// A process color: channels plus the space that interprets them.
/// A bare channel tuple without a space is never a complete color.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessColor {
    pub value: ProcessColorValue,
    pub space: ColorSpaceRef,
}

impl ProcessColor {
    /// Borrow RGB channels when the value holds them.
    #[must_use]
    pub fn as_rgb(&self) -> Option<Rgba> {
        if let ProcessColorValue::Rgb(channels) = &self.value {
            Some(*channels)
        } else {
            None
        }
    }

    /// All channels must be finite; alpha and CMYK channels must sit
    /// in `0..1`. Linear RGB may exceed `0..1` (HDR-ready, no clamp).
    pub fn validate(&self) -> Result<()> {
        let channels: &[f32] = match &self.value {
            ProcessColorValue::Rgb(c) => &[c.r, c.g, c.b, c.alpha],
            ProcessColorValue::Cmyk(c) => &[c.c, c.m, c.y, c.k, c.alpha],
            ProcessColorValue::Lab(c) => &[c.l, c.a_axis, c.b_axis, c.alpha],
            ProcessColorValue::Gray(c) => &[c.gray, c.alpha],
        };
        if channels.iter().any(|c| !c.is_finite()) {
            return Err(CoreError::InvariantViolation(
                "non-finite color channel rejected".to_string(),
            ));
        }
        let alpha = channels[channels.len() - 1];
        if !(0.0..=1.0).contains(&alpha) {
            return Err(CoreError::InvariantViolation(format!(
                "authorial alpha out of 0..=1: {alpha}"
            )));
        }
        if let ProcessColorValue::Cmyk(c) = &self.value {
            for channel in [c.c, c.m, c.y, c.k] {
                if !(0.0..=1.0).contains(&channel) {
                    return Err(CoreError::InvariantViolation(format!(
                        "CMYK channel out of 0..=1: {channel}"
                    )));
                }
            }
        }
        Ok(())
    }
}

/// Built-in working spaces known without an external profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuiltinColorSpace {
    Srgb,
    DisplayP3,
    LinearSrgb,
}

/// Explicit reference to the space interpreting a process color:
///
/// - built-in known space;
/// - embedded ICC profile (bytes live in resource storage);
/// - resolved external ICC profile;
/// - document-defined space.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColorSpaceRef {
    Builtin(BuiltinColorSpace),
    EmbeddedIcc(ResourceId),
    ExternalIcc { location: String },
    DocumentSpace { name: String },
}

/// Rendering intent for profile conversions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RenderingIntent {
    Perceptual,
    RelativeColorimetric,
    Saturation,
    AbsoluteColorimetric,
}

/// Document-wide color defaults. Existing colors are never silently
/// reinterpreted when these change; that takes explicit Commands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentColorSpec {
    pub rgb_working: ColorSpaceRef,
    pub cmyk_working: Option<ColorSpaceRef>,
    pub gray_working: Option<ColorSpaceRef>,
    pub rendering_intent: RenderingIntent,
}

/// Either a process color or a named spot ink. Spot is its own
/// identity, never "special CMYK".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColorValue {
    Process(ProcessColor),
    Spot(SpotColorRef),
}

/// A spot ink definition: identity, name and fallback preview.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpotColor {
    pub id: SpotColorId,
    pub name: String,
    pub alternate: ProcessColor,
}

/// Lookup of spot ink definitions by identity. Inks live here;
/// uses reference them by [`SpotColorId`] and carry their own tint.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SpotRegistry {
    entries: std::collections::BTreeMap<SpotColorId, SpotColor>,
}

impl SpotRegistry {
    /// Remove one record by identity; callers validate remaining references.
    pub fn remove(&mut self, id: SpotColorId) -> Option<SpotColor> {
        self.entries.remove(&id)
    }

    /// Empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace a spot ink definition.
    pub fn insert(&mut self, spot: SpotColor) {
        self.entries.insert(spot.id, spot);
    }

    /// Look up an ink by identity.
    #[must_use]
    pub fn get(&self, id: SpotColorId) -> Option<&SpotColor> {
        self.entries.get(&id)
    }

    /// Iterate entries in deterministic order; traversal only.
    pub fn iter(&self) -> impl Iterator<Item = (SpotColorId, &SpotColor)> {
        self.entries.iter().map(|(id, spot)| (*id, spot))
    }

    /// Number of tracked inks.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when no ink is tracked.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Use of one spot ink at a given tint. The tint belongs to the use,
/// so the same ink serves at several percentages.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpotColorRef {
    pub id: SpotColorId,
    pub tint: f32,
}

impl SpotColorRef {
    /// Tint must sit in `0..1`.
    pub fn new(id: SpotColorId, tint: f32) -> Result<Self> {
        if !tint.is_finite() || !(0.0..=1.0).contains(&tint) {
            return Err(CoreError::InvariantViolation(format!(
                "spot tint out of 0..=1: {tint}"
            )));
        }
        Ok(Self { id, tint })
    }
}

/// RGB after the space transfer function (e.g. sRGB encoded).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EncodedRgba(pub Rgba);

/// RGB proportional to represented luminance. Conversion depends on
/// the space/profile and belongs to the color engine.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LinearRgba(pub Rgba);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_conversions() {
        let white = ColorRgba::from_u8(255, 255, 255, 255);
        assert!((white.r - 1.0).abs() < f32::EPSILON);
        assert!((white.g - 1.0).abs() < f32::EPSILON);
        assert!((white.b - 1.0).abs() < f32::EPSILON);
        assert!((white.a - 1.0).abs() < f32::EPSILON);
    }

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

    #[test]
    fn test_process_color_validation() {
        assert!(srgb(1.0, 0.0, 0.0).validate().is_ok());
        // HDR-ready linear values above one are not clamped here.
        let mut hdr = srgb(2.5, 0.0, 0.0);
        hdr.space = ColorSpaceRef::Builtin(BuiltinColorSpace::LinearSrgb);
        assert!(hdr.validate().is_ok());
        // Alpha outside 0..1 is rejected.
        let mut bad_alpha = srgb(1.0, 0.0, 0.0);
        if let ProcessColorValue::Rgb(channels) = &mut bad_alpha.value {
            channels.alpha = 1.5;
        }
        assert!(bad_alpha.validate().is_err());
        // CMYK channels outside 0..1 are rejected.
        let bad_cmyk = ProcessColor {
            value: ProcessColorValue::Cmyk(Cmyka {
                c: 2.0,
                m: 0.0,
                y: 0.0,
                k: 0.0,
                alpha: 1.0,
            }),
            space: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
        };
        assert!(bad_cmyk.validate().is_err());
    }

    #[test]
    fn test_spot_tint_range() {
        let id = SpotColorId::new_v4();
        assert!(SpotColorRef::new(id, 0.5).is_ok());
        assert!(SpotColorRef::new(id, 1.5).is_err());
        assert!(SpotColorRef::new(id, f32::NAN).is_err());
    }

    #[test]
    fn test_managed_color_round_trip() {
        let spot = ColorValue::Spot(SpotColorRef::new(SpotColorId::new_v4(), 0.75).expect("valid"));
        let json = serde_json::to_string(&spot).expect("serializable");
        let back: ColorValue = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(back, spot);
    }
}
