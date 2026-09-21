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

/// A color stop in a gradient (F-11).
/// `id` is the stable local identity for reorder/reverse (10.4); serde
/// default keeps old payloads readable.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GradientStop {
    /// Stable local identity within the gradient.
    #[serde(default)]
    pub id: u32,
    /// Normalized position along gradient vector in [0.0, 1.0].
    pub offset: f64,
    /// Semantic token reference or literal color value.
    /// Accepted: `#RRGGBB`, `ptnd.*` tokens, `rgb(r,g,b)`,
    /// `gray(v)`, `cmyk(c,m,y,k)`.
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
            id: 0,
            offset: offset.clamp(0.0, 1.0),
            color: color.into(),
            opacity: 1.0,
        }
    }

    /// Creates a stop with an explicit stable id.
    #[must_use]
    pub fn with_id(offset: f64, color: impl Into<String>, id: u32) -> Self {
        Self {
            id,
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

    /// Resolves the stop color to linear 0..=1 sRGB triple (F-05).
    /// Supports `#RRGGBB`, `rgb()/gray()/cmyk()` literals and the known
    /// `ptnd.*` semantic tokens; unknown tokens fall back to mid-gray
    /// so callers never panic on user content.
    #[must_use]
    pub fn resolved_rgb(&self) -> [f32; 3] {
        resolve_color_to_rgb(&self.color)
    }
}

/// Parses a color token/literal into 0..=1 sRGB (F-05, dependency-free so
/// `petunia_design_document` stays UI-agnostic and cycle-free).
/// Legacy `aubrieta.*` identifiers are accepted on read and normalized to
/// the current `ptnd.*` namespace (15.A).
#[must_use]
pub fn resolve_color_to_rgb(token: &str) -> [f32; 3] {
    let owned;
    let t = match petunia_design_foundation::normalize_legacy_namespace(token) {
        Some(normalized) => {
            owned = normalized;
            owned.trim()
        }
        None => token.trim(),
    };
    if let Some(hex) = t.strip_prefix('#') {
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return [
                    f32::from(r) / 255.0,
                    f32::from(g) / 255.0,
                    f32::from(b) / 255.0,
                ];
            }
        }
        if hex.len() == 3 {
            let exp = |c: char| u8::from_str_radix(&format!("{c}{c}"), 16).unwrap_or(128);
            let chars: Vec<char> = hex.chars().collect();
            if chars.len() == 3 {
                return [
                    f32::from(exp(chars[0])) / 255.0,
                    f32::from(exp(chars[1])) / 255.0,
                    f32::from(exp(chars[2])) / 255.0,
                ];
            }
        }
    }
    let lower = t.to_lowercase();
    if let Some(inner) = lower.strip_prefix("gray(").and_then(|s| s.strip_suffix(')')) {
        if let Ok(v) = inner.trim().parse::<f32>() {
            let v = v.clamp(0.0, 1.0);
            return [v, v, v];
        }
    }
    if let Some(inner) = lower.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 {
            let parse = |s: &str| {
                let s = s.trim();
                if let Some(pct) = s.strip_suffix('%') {
                    pct.trim().parse::<f32>().map(|v| v / 100.0).ok()
                } else if let Ok(v) = s.parse::<f32>() {
                    Some(if v > 1.0 { v / 255.0 } else { v })
                } else {
                    None
                }
            };
            if let (Some(r), Some(g), Some(b)) = (parse(parts[0]), parse(parts[1]), parse(parts[2]))
            {
                return [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
            }
        }
    }
    if let Some(inner) = lower.strip_prefix("cmyk(").and_then(|s| s.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 4 {
            let vals: Option<Vec<f32>> = parts
                .iter()
                .map(|s| s.trim().trim_end_matches('%').parse::<f32>().ok().map(|v| {
                    if s.trim().ends_with('%') {
                        v / 100.0
                    } else {
                        v
                    }
                }))
                .collect();
            if let Some(v) = vals {
                let (c, m, y, k) = (v[0].clamp(0.0, 1.0), v[1].clamp(0.0, 1.0), v[2].clamp(0.0, 1.0), v[3].clamp(0.0, 1.0));
                // Naive preview-only conversion (matches `petunia_design_color`).
                return [1.0 - (c + k).min(1.0), 1.0 - (m + k).min(1.0), 1.0 - (y + k).min(1.0)];
            }
        }
    }
    match t {
        "ptnd.red/500" => [0.937, 0.267, 0.267],
        "ptnd.blue/500" => [0.231, 0.510, 0.965],
        "ptnd.green/500" => [0.133, 0.773, 0.369],
        "ptnd.yellow/500" => [0.918, 0.702, 0.031],
        "ptnd.gray/900" => [0.067, 0.067, 0.067],
        "ptnd.gray/500" => [0.42, 0.42, 0.42],
        "ptnd.white" => [1.0, 1.0, 1.0],
        "ptnd.black" => [0.0, 0.0, 0.0],
        "ptnd.purple/500" => [0.55, 0.30, 0.85],
        "ptnd.cyan/500" => [0.15, 0.75, 0.85],
        _ => [0.5, 0.5, 0.5],
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
        // Sort a view by offset so unordered stops do not hide content (F-11).
        let mut order: Vec<usize> = (0..self.stops.len()).collect();
        order.sort_by(|a, b| {
            self.stops[*a]
                .offset
                .partial_cmp(&self.stops[*b].offset)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for w in order.windows(2) {
            let s0 = &self.stops[w[0]];
            let s1 = &self.stops[w[1]];
            if t >= s0.offset && t <= s1.offset {
                let range = s1.offset - s0.offset;
                let factor = if range > f64::EPSILON {
                    (t - s0.offset) / range
                } else {
                    0.0
                };
                return Some((s0, s1, factor.clamp(0.0, 1.0)));
            }
        }
        // t outside [first,last]: clamp to the nearest end instead of
        // returning the last stop twice regardless of direction.
        if t <= self.stops[order[0]].offset {
            let s = &self.stops[order[0]];
            return Some((s, s, 0.0));
        }
        let s = &self.stops[order[order.len() - 1]];
        Some((s, s, 0.0))
    }

    /// Interpolates an sRGB color + opacity at `t` in linear-light-correct
    /// order: resolve stops to sRGB, lerp channels and alpha (F-11).
    /// Interpolation space is sRGB (documented policy until ICC gradients
    /// land in 09.9).
    #[must_use]
    pub fn sample_rgba(&self, t: f64) -> Option<([f32; 3], f32)> {
        let (s0, s1, f) = self.sample_stop(t)?;
        let c0 = s0.resolved_rgb();
        let c1 = s1.resolved_rgb();
        let f = f as f32;
        let rgb = [
            c0[0] + (c1[0] - c0[0]) * f,
            c0[1] + (c1[1] - c0[1]) * f,
            c0[2] + (c1[2] - c0[2]) * f,
        ];
        let a = (s0.opacity + (s1.opacity - s0.opacity) * f64::from(f)) as f32;
        Some((rgb, a.clamp(0.0, 1.0)))
    }

    /// Reorders stops by offset (stable) and reassigns sequential ids.
    pub fn sort_and_reindex(&mut self) {
        self.stops.sort_by(|a, b| {
            a.offset
                .partial_cmp(&b.offset)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (i, s) in self.stops.iter_mut().enumerate() {
            s.id = i as u32 + 1;
        }
    }

    /// Reverses stop order deterministically, mirroring offsets (10.4).
    pub fn reverse(&mut self) {
        for s in &mut self.stops {
            s.offset = 1.0 - s.offset;
        }
        self.sort_and_reindex();
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

    /// Sample stop pair at normalized radial distance `t` (F-11).
    #[must_use]
    pub fn sample_stop(&self, t: f64) -> Option<(&GradientStop, &GradientStop, f64)> {
        if self.stops.is_empty() {
            return None;
        }
        if self.stops.len() == 1 {
            return Some((&self.stops[0], &self.stops[0], 0.0));
        }
        let t = t.clamp(0.0, 1.0);
        let mut order: Vec<usize> = (0..self.stops.len()).collect();
        order.sort_by(|a, b| {
            self.stops[*a]
                .offset
                .partial_cmp(&self.stops[*b].offset)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for w in order.windows(2) {
            let s0 = &self.stops[w[0]];
            let s1 = &self.stops[w[1]];
            if t >= s0.offset && t <= s1.offset {
                let range = s1.offset - s0.offset;
                let factor = if range > f64::EPSILON {
                    (t - s0.offset) / range
                } else {
                    0.0
                };
                return Some((s0, s1, factor.clamp(0.0, 1.0)));
            }
        }
        if t <= self.stops[order[0]].offset {
            let s = &self.stops[order[0]];
            return Some((s, s, 0.0));
        }
        let s = &self.stops[order[order.len() - 1]];
        Some((s, s, 0.0))
    }

    /// Interpolates sRGB + opacity at radial `t` (F-11).
    #[must_use]
    pub fn sample_rgba(&self, t: f64) -> Option<([f32; 3], f32)> {
        let (s0, s1, f) = self.sample_stop(t)?;
        let c0 = s0.resolved_rgb();
        let c1 = s1.resolved_rgb();
        let f = f as f32;
        let rgb = [
            c0[0] + (c1[0] - c0[0]) * f,
            c0[1] + (c1[1] - c0[1]) * f,
            c0[2] + (c1[2] - c0[2]) * f,
        ];
        let a = (s0.opacity + (s1.opacity - s0.opacity) * f64::from(f)) as f32;
        Some((rgb, a.clamp(0.0, 1.0)))
    }
}

/// Paint variant supported by the V1 Appearance Stack.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Paint {
    /// No paint (transparent).
    #[default]
    None,
    /// Solid color token reference (e.g. `ptnd.blue/500`) or hex string.
    Solid(String),
    /// Linear gradient paint.
    LinearGradient(LinearGradient),
    /// Radial gradient paint.
    RadialGradient(RadialGradient),
}

impl Paint {
    /// Resolves a solid paint to sRGB, if applicable (F-05).
    #[must_use]
    pub fn solid_rgb(&self) -> Option<[f32; 3]> {
        match self {
            Self::Solid(token) => Some(resolve_color_to_rgb(token)),
            _ => None,
        }
    }

    /// Samples a paint at normalized gradient position `t`.
    /// Solids ignore `t`; gradients interpolate; `None` has no color.
    #[must_use]
    pub fn sample_rgba(&self, t: f64) -> Option<([f32; 3], f32)> {
        match self {
            Self::None => None,
            Self::Solid(token) => Some((resolve_color_to_rgb(token), 1.0)),
            Self::LinearGradient(g) => g.sample_rgba(t),
            Self::RadialGradient(g) => g.sample_rgba(t),
        }
    }
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
