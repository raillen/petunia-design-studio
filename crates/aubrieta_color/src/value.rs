//! Owned semantic color values with exact display math.

use serde::{Deserialize, Serialize};

/// Display sRGB triple, components in 0..=1.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Srgb {
    /// Red channel.
    pub r: f32,
    /// Green channel.
    pub g: f32,
    /// Blue channel.
    pub b: f32,
}

impl Srgb {
    /// Black.
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };

    /// Validates range and finiteness.
    pub fn try_new(r: f32, g: f32, b: f32) -> Result<Self, String> {
        for (name, v) in [("r", r), ("g", g), ("b", b)] {
            if !v.is_finite() || !(0.0..=1.0).contains(&v) {
                return Err(format!("sRGB channel `{name}` out of range: {v}"));
            }
        }
        Ok(Self { r, g, b })
    }

    /// Clamps adapter output into range.
    #[must_use]
    pub fn clamped(r: f32, g: f32, b: f32) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
        }
    }
}

/// CMYK press values, components in 0..=1.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cmyk {
    /// Cyan.
    pub c: f32,
    /// Magenta.
    pub m: f32,
    /// Yellow.
    pub y: f32,
    /// Black key.
    pub k: f32,
}

/// CIELAB D50-derived values carried in documents (display via D65 path).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Lab {
    /// Lightness 0..=100.
    pub l: f32,
    /// Green–red axis.
    pub a: f32,
    /// Blue–yellow axis.
    pub b: f32,
}

/// Semantic color stored in documents and styles.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ColorValue {
    /// Display-oriented sRGB.
    Rgb(Srgb),
    /// Single-channel gray 0..=1 (0 = black).
    Gray(f32),
    /// Press CMYK. Display conversion is preview-only, never stored back.
    Cmyk(Cmyk),
    /// Device-independent Lab.
    Lab(Lab),
    /// Named spot ink with a process fallback for preview.
    Spot {
        /// Ink name, e.g. `PANTONE 185 C`.
        name: String,
        /// Process fallback used for on-screen preview.
        fallback: Box<ColorValue>,
    },
    /// Registration mark color (all plates). Displays as black.
    Registration,
}

impl ColorValue {
    /// One-way conversion to display sRGB. Lossy by contract; never write
    /// the result back as the stored value.
    #[must_use]
    pub fn to_srgb(&self) -> Srgb {
        match self {
            Self::Rgb(rgb) => *rgb,
            Self::Gray(k) => Srgb::clamped(*k, *k, *k),
            Self::Cmyk(cmyk) => cmyk_naive_to_srgb(*cmyk),
            Self::Lab(lab) => lab_to_srgb(*lab),
            Self::Spot { fallback, .. } => fallback.to_srgb(),
            Self::Registration => Srgb::BLACK,
        }
    }
}

/// Naive CMYK→sRGB for on-screen preview only.
fn cmyk_naive_to_srgb(cmyk: Cmyk) -> Srgb {
    let r = 1.0 - (cmyk.c + cmyk.k).min(1.0);
    let g = 1.0 - (cmyk.m + cmyk.k).min(1.0);
    let b = 1.0 - (cmyk.y + cmyk.k).min(1.0);
    Srgb::clamped(r, g, b)
}

/// Closed-form CIELAB→sRGB (D65) owned by Aubrieta; the moxcms-assisted path
/// in `transform` cross-checks this differentially.
fn lab_to_srgb(lab: Lab) -> Srgb {
    const XN: f64 = 0.950_47;
    const ZN: f64 = 1.088_83;
    const DELTA: f64 = 6.0 / 29.0;

    let fy = (f64::from(lab.l) + 16.0) / 116.0;
    let fx = fy + f64::from(lab.a) / 500.0;
    let fz = fy - f64::from(lab.b) / 200.0;
    let cube = |t: f64| {
        if t > DELTA {
            t.powi(3)
        } else {
            3.0 * DELTA * DELTA * (t - 4.0 / 29.0)
        }
    };
    let x = XN * cube(fx);
    let y = cube(fy);
    let z = ZN * cube(fz);

    // XYZ D65 → linear sRGB.
    let r_lin = 3.240_6 * x - 1.537_2 * y - 0.498_6 * z;
    let g_lin = -0.968_9 * x + 1.875_8 * y + 0.041_5 * z;
    let b_lin = 0.055_7 * x - 0.204_0 * y + 1.057_0 * z;

    Srgb::clamped(
        gamma_encode(r_lin) as f32,
        gamma_encode(g_lin) as f32,
        gamma_encode(b_lin) as f32,
    )
}

fn gamma_encode(linear: f64) -> f64 {
    if linear <= 0.003_130_8 {
        12.92 * linear
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srgb_rejects_out_of_range() {
        assert!(Srgb::try_new(2.0, 0.0, 0.0).is_err());
        assert!(Srgb::try_new(0.5, 0.5, 0.5).is_ok());
    }

    #[test]
    fn white_and_black_lab_landmarks() {
        let white = ColorValue::Lab(Lab {
            l: 100.0,
            a: 0.0,
            b: 0.0,
        })
        .to_srgb();
        assert!((white.r - 1.0).abs() < 0.01, "{white:?}");
        assert!((white.g - 1.0).abs() < 0.01, "{white:?}");
        assert!((white.b - 1.0).abs() < 0.01, "{white:?}");
        let black = ColorValue::Lab(Lab {
            l: 0.0,
            a: 0.0,
            b: 0.0,
        })
        .to_srgb();
        assert_eq!(black, Srgb::BLACK);
    }

    #[test]
    fn cmyk_black_previews_black() {
        let black = ColorValue::Cmyk(Cmyk {
            c: 0.0,
            m: 0.0,
            y: 0.0,
            k: 1.0,
        })
        .to_srgb();
        assert_eq!(black, Srgb::BLACK);
    }

    #[test]
    fn spot_uses_fallback_for_preview() {
        let spot = ColorValue::Spot {
            name: "PANTONE 185 C".to_string(),
            fallback: Box::new(ColorValue::Rgb(Srgb::try_new(1.0, 0.0, 0.0).unwrap())),
        };
        assert_eq!(spot.to_srgb(), Srgb::try_new(1.0, 0.0, 0.0).unwrap());
    }
}
