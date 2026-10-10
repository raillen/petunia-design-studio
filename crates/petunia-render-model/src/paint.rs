//! Evaluated paint: colors, gradients and strokes ready to sample.
//!
//! Authorial parameters live in Core; these types carry values
//! already resolved for the compositor. Colors are linear-space
//! straight-alpha floats; the compositor premultiplies at the last
//! step.

use petunia_core::{
    GradientInterpolation, GradientSpread, PaintSpace, PatternRepeat, Rect, ResourceId, Transform2D,
};
use serde::{Deserialize, Serialize};

/// Linear-space straight-alpha color for composition input.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RenderColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl RenderColor {
    /// Transparent black: the neutral element of source-over.
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// Opaque black.
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// Opaque white.
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    /// Straight alpha preserved through evaluation.
    #[must_use]
    pub fn with_alpha(self, alpha: f32) -> Self {
        Self { a: alpha, ..self }
    }
}

/// One evaluated gradient stop. The midpoint owns the interval to
/// the next stop: `0.5` is linear, shifted values remap exponentially,
/// and `0`/`1` are invalid (singular parametrization).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RenderGradientStop {
    pub offset: f32,
    pub color: RenderColor,
    pub midpoint: f32,
}

/// Evaluated gradient: stops plus the persisted policies the
/// evaluator resolved from authorial parameters. Endpoints live in
/// plain document pairs to keep the contract free of geometry helpers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderGradient {
    pub stops: Vec<RenderGradientStop>,
    pub interpolation: GradientInterpolation,
    pub spread: GradientSpread,
    pub space: PaintSpace,
    pub start: (f64, f64),
    pub end: (f64, f64),
    pub radius: f64,
    pub start_angle: f64,
}

/// Evaluated pattern paint ready to sample from decoded image resources.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderPattern {
    pub resource: ResourceId,
    pub width: u32,
    pub height: u32,
    pub repeat_x: PatternRepeat,
    pub repeat_y: PatternRepeat,
    pub transform: Transform2D,
}

/// Paint ready to sample: solid, linear, radial, conical gradient or pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderPaint {
    Solid(RenderColor),
    LinearGradient(RenderGradient),
    RadialGradient(RenderGradient),
    ConicalGradient(RenderGradient),
    Pattern(RenderPattern),
}

/// Stroke ready to rasterize: paint plus resolved width.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderStroke {
    pub paint: RenderPaint,
    pub width: f64,
}

/// What fills and strokes one vector primitive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderAppearance {
    pub fill: Option<RenderPaint>,
    pub stroke: Option<RenderStroke>,
    pub opacity: f32,
}

/// Bounds helper shared by primitives.
#[must_use]
pub fn rect_of(x: f64, y: f64, width: f64, height: f64) -> Rect {
    Rect::new(x, y, width, height)
}
