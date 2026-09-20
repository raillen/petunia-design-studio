//! Canonical Appearance Stack model conforming to 10.4.
//!
//! An ordered, non-destructive stack containing multiple fills, multiple strokes,
//! opacity/blend modes, and typed effect chain entries.

use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_one() -> f64 {
    1.0
}

/// 16 standard blend modes conforming to W3C Compositing and Blending Level 1 / PDF.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BlendMode {
    /// Normal: replaces backdrop with source based on source alpha.
    #[default]
    Normal,
    /// Multiply: multiplies backdrop and source channels.
    Multiply,
    /// Screen: multiplies inverse of backdrop and source channels.
    Screen,
    /// Overlay: multiplies or screens depending on backdrop channel.
    Overlay,
    /// Darken: selects the minimum of backdrop and source.
    Darken,
    /// Lighten: selects the maximum of backdrop and source.
    Lighten,
    /// ColorDodge: brightens backdrop to reflect source.
    ColorDodge,
    /// ColorBurn: darkens backdrop to reflect source.
    ColorBurn,
    /// HardLight: multiplies or screens depending on source channel.
    HardLight,
    /// SoftLight: darkens or lightens depending on source channel.
    SoftLight,
    /// Difference: subtracts darker color from lighter color.
    Difference,
    /// Exclusion: similar to difference with lower contrast.
    Exclusion,
    /// Hue: preserves luminosity and saturation of backdrop with hue of source.
    Hue,
    /// Saturation: preserves luminosity and hue of backdrop with saturation of source.
    Saturation,
    /// Color: preserves luminosity of backdrop with hue and saturation of source.
    Color,
    /// Luminosity: preserves hue and saturation of backdrop with luminosity of source.
    Luminosity,
}

/// A color stop in a gradient.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GradientStop {
    /// Normalized position along gradient vector in [0.0, 1.0].
    pub offset: f64,
    /// Semantic token reference or literal color value.
    pub color: String,
    /// Individual stop opacity in [0.0, 1.0].
    #[serde(default = "default_one")]
    pub opacity: f64,
}

impl GradientStop {
    /// Creates a new gradient stop.
    #[must_use]
    pub fn new(offset: f64, color: impl Into<String>) -> Self {
        Self {
            offset: offset.clamp(0.0, 1.0),
            color: color.into(),
            opacity: 1.0,
        }
    }

    /// Sets the opacity of this stop.
    #[must_use]
    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
}

/// Linear gradient definition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinearGradient {
    /// Start point in normalized or document coordinates [x, y].
    pub start: [f64; 2],
    /// End point in normalized or document coordinates [x, y].
    pub end: [f64; 2],
    /// Ordered color stops.
    pub stops: Vec<GradientStop>,
}

impl LinearGradient {
    /// Creates a linear gradient between two points with given stops.
    #[must_use]
    pub fn new(start: [f64; 2], end: [f64; 2], stops: Vec<GradientStop>) -> Self {
        Self { start, end, stops }
    }

    /// Sample interpolated color value at normalized t in [0.0, 1.0].
    #[must_use]
    pub fn sample_stop(&self, t: f64) -> Option<(&GradientStop, &GradientStop, f64)> {
        if self.stops.is_empty() {
            return None;
        }
        if self.stops.len() == 1 {
            return Some((&self.stops[0], &self.stops[0], 0.0));
        }
        let t = t.clamp(0.0, 1.0);
        for i in 0..self.stops.len() - 1 {
            let s0 = &self.stops[i];
            let s1 = &self.stops[i + 1];
            if t >= s0.offset && t <= s1.offset {
                let range = s1.offset - s0.offset;
                let factor = if range > f64::EPSILON {
                    (t - s0.offset) / range
                } else {
                    0.0
                };
                return Some((s0, s1, factor));
            }
        }
        let last = self.stops.last().unwrap();
        Some((last, last, 0.0))
    }
}

/// Radial gradient definition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RadialGradient {
    /// Center point [x, y].
    pub center: [f64; 2],
    /// Radius in document points.
    pub radius: f64,
    /// Ordered color stops.
    pub stops: Vec<GradientStop>,
}

impl RadialGradient {
    /// Creates a radial gradient.
    #[must_use]
    pub fn new(center: [f64; 2], radius: f64, stops: Vec<GradientStop>) -> Self {
        Self {
            center,
            radius: radius.max(0.001),
            stops,
        }
    }
}

/// Paint variant supported by the V1 Appearance Stack.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Paint {
    /// No paint (transparent).
    #[default]
    None,
    /// Solid color token reference (e.g. `aubrieta.blue/500`) or hex string.
    Solid(String),
    /// Linear gradient paint.
    LinearGradient(LinearGradient),
    /// Radial gradient paint.
    RadialGradient(RadialGradient),
}

/// Stroke alignment relative to the path outline.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StrokeAlignment {
    /// Centered on the path geometry (default).
    #[default]
    Center,
    /// Aligned inside closed path geometry.
    Inside,
    /// Aligned outside closed path geometry.
    Outside,
}

/// Stroke cap style for line endpoints.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StrokeCap {
    /// Flat square cap ending exactly at vertex.
    #[default]
    Butt,
    /// Semicircular rounded cap.
    Round,
    /// Square cap extending half width past vertex.
    Square,
}

/// Stroke join style for path vertices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StrokeJoin {
    /// Sharp corner with miter limit clipping.
    #[default]
    Miter,
    /// Rounded corner.
    Round,
    /// Beveled corner.
    Bevel,
}

/// Single fill layer in the Appearance Stack.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FillItem {
    /// Local entry identity within the appearance stack.
    pub id: u32,
    /// Paint style (solid, linear gradient, radial gradient, none).
    pub paint: Paint,
    /// Layer-specific opacity in [0.0, 1.0].
    #[serde(default = "default_one")]
    pub opacity: f64,
    /// Layer-specific blend mode.
    #[serde(default)]
    pub blend_mode: BlendMode,
    /// Whether this fill layer is active.
    #[serde(default = "default_true")]
    pub visible: bool,
}

impl FillItem {
    /// Creates a solid fill item.
    #[must_use]
    pub fn solid(id: u32, color: impl Into<String>) -> Self {
        Self {
            id,
            paint: Paint::Solid(color.into()),
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            visible: true,
        }
    }

    /// Creates a linear gradient fill item.
    #[must_use]
    pub fn linear_gradient(id: u32, gradient: LinearGradient) -> Self {
        Self {
            id,
            paint: Paint::LinearGradient(gradient),
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            visible: true,
        }
    }

    /// Creates a radial gradient fill item.
    #[must_use]
    pub fn radial_gradient(id: u32, gradient: RadialGradient) -> Self {
        Self {
            id,
            paint: Paint::RadialGradient(gradient),
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            visible: true,
        }
    }
}

/// Single stroke layer in the Appearance Stack.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrokeItem {
    /// Local entry identity within the appearance stack.
    pub id: u32,
    /// Paint style for the stroke outline.
    pub paint: Paint,
    /// Stroke thickness in document points.
    #[serde(default = "default_one")]
    pub width: f64,
    /// Inside, Center, or Outside path alignment.
    #[serde(default)]
    pub alignment: StrokeAlignment,
    /// End cap style.
    #[serde(default)]
    pub cap: StrokeCap,
    /// Corner join style.
    #[serde(default)]
    pub join: StrokeJoin,
    /// Miter angle clipping threshold.
    #[serde(default = "default_miter")]
    pub miter_limit: f64,
    /// Dash pattern intervals in points.
    #[serde(default)]
    pub dash_array: Vec<f64>,
    /// Dash phase offset.
    #[serde(default)]
    pub dash_offset: f64,
    /// Layer-specific opacity in [0.0, 1.0].
    #[serde(default = "default_one")]
    pub opacity: f64,
    /// Layer-specific blend mode.
    #[serde(default)]
    pub blend_mode: BlendMode,
    /// Whether this stroke layer is active.
    #[serde(default = "default_true")]
    pub visible: bool,
}

fn default_miter() -> f64 {
    4.0
}

impl StrokeItem {
    /// Creates a simple solid stroke item.
    #[must_use]
    pub fn solid(id: u32, color: impl Into<String>, width: f64) -> Self {
        Self {
            id,
            paint: Paint::Solid(color.into()),
            width: width.max(0.0),
            alignment: StrokeAlignment::Center,
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
            miter_limit: 4.0,
            dash_array: Vec::new(),
            dash_offset: 0.0,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            visible: true,
        }
    }
}

/// Typed V1 non-destructive effects (10.4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EffectKind {
    /// Outer drop shadow effect.
    DropShadow {
        /// Offset vector [dx, dy] in document points.
        offset: [f64; 2],
        /// Gaussian blur radius in points.
        blur: f64,
        /// Shadow color token.
        color: String,
        /// Shadow opacity in [0.0, 1.0].
        opacity: f64,
    },
    /// Inner shadow effect clipped to object fill.
    InnerShadow {
        /// Offset vector [dx, dy] in document points.
        offset: [f64; 2],
        /// Gaussian blur radius in points.
        blur: f64,
        /// Shadow color token.
        color: String,
        /// Shadow opacity in [0.0, 1.0].
        opacity: f64,
    },
    /// Direct Gaussian blur filter.
    GaussianBlur {
        /// Blur radius in document points.
        radius: f64,
    },
}

/// Single effect entry in the Appearance Stack.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EffectItem {
    /// Local identity within the stack.
    pub id: u32,
    /// The typed effect definition.
    pub kind: EffectKind,
    /// Whether the effect is active.
    #[serde(default = "default_true")]
    pub visible: bool,
}

/// Ordered, non-destructive Appearance Stack for a DocumentObject (10.4).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AppearanceStack {
    /// Ordered fill entries rendered from back to front.
    #[serde(default)]
    pub fills: Vec<FillItem>,
    /// Ordered stroke entries rendered from back to front.
    #[serde(default)]
    pub strokes: Vec<StrokeItem>,
    /// Ordered non-destructive post-render effects.
    #[serde(default)]
    pub effects: Vec<EffectItem>,
    /// Overall object opacity in [0.0, 1.0].
    #[serde(default = "default_one")]
    pub opacity: f64,
    /// Overall object blend mode with backdrop.
    #[serde(default)]
    pub blend_mode: BlendMode,
}

impl AppearanceStack {
    /// Creates an empty appearance stack.
    #[must_use]
    pub fn new() -> Self {
        Self {
            fills: Vec::new(),
            strokes: Vec::new(),
            effects: Vec::new(),
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
        }
    }

    /// Builder helper to add a primary solid fill.
    #[must_use]
    pub fn with_fill(mut self, color: impl Into<String>) -> Self {
        let next_id = self.fills.iter().map(|f| f.id).max().unwrap_or(0) + 1;
        self.fills.push(FillItem::solid(next_id, color));
        self
    }

    /// Builder helper to add a primary solid stroke.
    #[must_use]
    pub fn with_stroke(mut self, color: impl Into<String>, width: f64) -> Self {
        let next_id = self.strokes.iter().map(|s| s.id).max().unwrap_or(0) + 1;
        self.strokes.push(StrokeItem::solid(next_id, color, width));
        self
    }

    /// Returns the primary active fill item, if any.
    #[must_use]
    pub fn primary_fill(&self) -> Option<&FillItem> {
        self.fills
            .iter()
            .find(|f| f.visible && f.paint != Paint::None)
    }

    /// Returns the primary active stroke item, if any.
    #[must_use]
    pub fn primary_stroke(&self) -> Option<&StrokeItem> {
        self.strokes
            .iter()
            .find(|s| s.visible && s.paint != Paint::None)
    }

    /// Appends a new fill entry.
    pub fn add_fill(&mut self, fill: FillItem) {
        self.fills.push(fill);
    }

    /// Removes a fill entry by local id.
    pub fn remove_fill(&mut self, fill_id: u32) -> bool {
        if let Some(pos) = self.fills.iter().position(|f| f.id == fill_id) {
            self.fills.remove(pos);
            true
        } else {
            false
        }
    }

    /// Appends a new stroke entry.
    pub fn add_stroke(&mut self, stroke: StrokeItem) {
        self.strokes.push(stroke);
    }

    /// Removes a stroke entry by local id.
    pub fn remove_stroke(&mut self, stroke_id: u32) -> bool {
        if let Some(pos) = self.strokes.iter().position(|s| s.id == stroke_id) {
            self.strokes.remove(pos);
            true
        } else {
            false
        }
    }

    /// Appends a new effect entry.
    pub fn add_effect(&mut self, effect: EffectItem) {
        self.effects.push(effect);
    }

    /// Removes an effect entry by local id.
    pub fn remove_effect(&mut self, effect_id: u32) -> bool {
        if let Some(pos) = self.effects.iter().position(|e| e.id == effect_id) {
            self.effects.remove(pos);
            true
        } else {
            false
        }
    }

    /// Calculates expanded bounding box inflation required by active effects.
    #[must_use]
    pub fn bounds_inflation(&self) -> f64 {
        let mut max_inf: f64 = 0.0;
        for eff in &self.effects {
            if !eff.visible {
                continue;
            }
            match &eff.kind {
                EffectKind::DropShadow { offset, blur, .. } => {
                    let off_dist = (offset[0].powi(2) + offset[1].powi(2)).sqrt();
                    let reach = off_dist + blur * 2.0;
                    max_inf = max_inf.max(reach);
                }
                EffectKind::GaussianBlur { radius } => {
                    max_inf = max_inf.max(radius * 2.5);
                }
                EffectKind::InnerShadow { .. } => {
                    // Inner shadow does not expand outer bounds
                }
            }
        }
        max_inf
    }
}
