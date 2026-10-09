//! Effect, mask and adjustment descriptors for the compositor.
//!
//! Parameters mirror the authorial contracts; the render side owns
//! the math. Every adjustment declares its color space and preserves
//! alpha unless it explicitly owns coverage.

use petunia_core::{ObjectId, Rect};
use serde::{Deserialize, Serialize};

/// Post-paint effect descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderEffect {
    Blur { sigma_x: f64, sigma_y: f64 },
    DropShadow(Box<ShadowEffect>),
    Adjustment(RenderAdjustment),
}

/// Drop shadow pipeline: coverage, offset, blur, colorize, composite
/// behind the content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowEffect {
    pub offset: (f64, f64),
    pub sigma: (f64, f64),
    pub color: crate::paint::RenderColor,
    pub spread: f64,
    pub opacity: f32,
}

/// Alpha or luminance coverage resolved by render; luminance uses the
/// contract-defined transfer, never a naive RGB mean.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderMask {
    Alpha(ObjectId),
    Luminance(ObjectId),
}

/// Clip geometry for one group or primitive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderClip {
    Rect(Rect),
    Polygon(Vec<(f64, f64)>),
}

/// Adjustment domain: each adjustment names the space its math is
/// defined in. Backends never pick implicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdjustmentSpace {
    WorkingEncodedRgb,
    WorkingLinearRgb,
    PerceptualOklab,
}

/// Typed adjustment operations with validated parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderAdjustment {
    Invert,
    Grayscale,
    Posterize {
        levels: u32,
    },
    BrightnessContrast {
        brightness: f32,
        contrast: f32,
    },
    Levels {
        channels: LevelsChannels,
    },
    Hsl {
        hue_shift: f32,
        saturation: f32,
        lightness: f32,
    },
    Vibrance {
        amount: f32,
    },
    ColorToAlpha {
        reference: crate::paint::RenderColor,
    },
    Curves {
        points: Vec<CurvePoint>,
        monotonic: bool,
    },
}

/// Per-channel levels with master applied first.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelsChannels {
    pub master: LevelsChannel,
    pub red: Option<LevelsChannel>,
    pub green: Option<LevelsChannel>,
    pub blue: Option<LevelsChannel>,
}

/// One levels mapping: input range, gamma and output range.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LevelsChannel {
    pub input_black: f32,
    pub input_white: f32,
    pub gamma: f32,
    pub output_black: f32,
    pub output_white: f32,
}

/// Monotone-by-contract curve control point in 0..1.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CurvePoint {
    pub x: f32,
    pub y: f32,
}
