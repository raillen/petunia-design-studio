//! Authorial appearance: how geometry is painted.
//!
//! Appearance is an ordered list so fills and strokes interleave.
//! Effect *parameters* are the source of truth; evaluated results,
//! expanded outlines and raster caches are derived state owned by
//! the Engine and Render stages.

use crate::color::ColorValue;
use crate::error::{CoreError, Result};
use crate::id::{AppearanceItemId, ObjectId, ResourceId, StyleId};
use crate::math::Transform2D;
use crate::paint::{ColorSource, Gradient, PaintSpace};
use serde::{Deserialize, Serialize};

/// Ordered appearance stack of one paintable object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Appearance {
    pub items: Vec<AppearanceItem>,
}

impl Appearance {
    /// The historical new-path default, now explicit: one enabled
    /// solid sRGB black fill. New paths keep rendering exactly as
    /// before; any other look is an explicit edit.
    #[must_use]
    pub fn solid_black() -> Self {
        use crate::color::{
            BuiltinColorSpace, ColorSpaceRef, ProcessColor, ProcessColorValue, Rgba,
        };
        use crate::id::AppearanceItemId;
        Self {
            items: vec![AppearanceItem {
                id: AppearanceItemId::new_v4(),
                enabled: true,
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
                kind: AppearanceKind::Fill(Paint::Solid(ColorSource::Value(ColorValue::Process(
                    ProcessColor {
                        value: ProcessColorValue::Rgb(Rgba {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            alpha: 1.0,
                        }),
                        space: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
                    },
                )))),
            }],
        }
    }
}

impl Appearance {
    /// Re-check item invariants on any instance, including
    /// deserialized ones: opacity ranges and stroke payloads.
    pub fn validate(&self) -> Result<()> {
        for item in &self.items {
            if !item.opacity.is_finite() || !(0.0..=1.0).contains(&item.opacity) {
                return Err(CoreError::InvariantViolation(format!(
                    "appearance opacity out of 0..=1: {}",
                    item.opacity
                )));
            }
            if let AppearanceKind::Stroke(style) = &item.kind {
                style.validate()?;
            }
        }
        Ok(())
    }
}

/// One paint entry with its own identity, opacity and blend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppearanceItem {
    pub id: AppearanceItemId,
    pub enabled: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub kind: AppearanceKind,
}

impl AppearanceItem {
    /// Opacity must sit in `0..1`; every opacity level in the pipeline
    /// keeps its own semantics and is never collapsed into one field.
    pub fn new(
        enabled: bool,
        opacity: f32,
        blend_mode: BlendMode,
        kind: AppearanceKind,
    ) -> Result<Self> {
        if !opacity.is_finite() || !(0.0..=1.0).contains(&opacity) {
            return Err(CoreError::InvariantViolation(format!(
                "appearance opacity out of 0..=1: {opacity}"
            )));
        }
        Ok(Self {
            id: AppearanceItemId::new_v4(),
            enabled,
            opacity,
            blend_mode,
            kind,
        })
    }
}

/// Fill or stroke payload of one appearance item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AppearanceKind {
    Fill(Paint),
    Stroke(StrokeStyle),
}

/// Authorial paint: solid, gradients or a reusable pattern. Gradient
/// stops, interpolation, spread and space persist here; conical
/// gradients extend this contract and mesh gradients stay out of v0.1
/// until they own a model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Paint {
    Solid(ColorSource),
    LinearGradient(Gradient),
    RadialGradient(Gradient),
    Pattern(PatternPaint),
}

/// A reusable paint pattern: source, authorial transform and repeat
/// semantics. Rasterized pattern tiles are cache, never source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternPaint {
    pub source: PatternSource,
    pub transform: Transform2D,
    pub space: PaintSpace,
    pub repeat_x: PatternRepeat,
    pub repeat_y: PatternRepeat,
}

/// Pattern artwork: a raster resource or an authorized vector definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternSource {
    Raster(ResourceId),
    Vector(ObjectId),
}

/// Initial repeat vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PatternRepeat {
    #[default]
    Repeat,
    Clamp,
    Mirror,
}

/// Persistent blend vocabulary. The math belongs to Render; names are
/// never localized in storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

/// Line cap vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StrokeCap {
    #[default]
    Butt,
    Round,
    Square,
}

/// Line join vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StrokeJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

/// Stroke placement on closed contours. Open paths only define
/// Center in v0.1; inside/outside there must not gain an ambiguous
/// silent semantic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StrokeAlignment {
    #[default]
    Center,
    Inside,
    Outside,
}

/// Dash lengths stay validated and normalized (see [`DashPattern`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashPattern {
    pub lengths: Vec<f64>,
    pub offset: f64,
}

impl DashPattern {
    /// Validation rules: negative lengths error, a zero total errors,
    /// an empty list means a continuous stroke, an odd count duplicates
    /// the sequence deterministically, and the offset normalizes
    /// modularly over the pattern total.
    pub fn new(lengths: Vec<f64>, offset: f64) -> Result<Self> {
        if !offset.is_finite() {
            return Err(CoreError::InvariantViolation(format!(
                "non-finite dash offset rejected: {offset}"
            )));
        }
        let mut normalized = lengths;
        Self::normalize_input(&mut normalized)?;
        if normalized.len() % 2 == 1 {
            let doubled = normalized.clone();
            normalized.extend(doubled);
        }
        // The offset wraps over the final repeating period, which
        // includes the deterministic duplication above.
        let period: f64 = normalized.iter().sum();
        let offset = if period > 0.0 {
            offset.rem_euclid(period)
        } else {
            0.0
        };
        Ok(Self {
            lengths: normalized,
            offset,
        })
    }

    fn normalize_input(lengths: &mut [f64]) -> Result<()> {
        for length in lengths.iter() {
            if !length.is_finite() || *length < 0.0 {
                return Err(CoreError::InvariantViolation(format!(
                    "invalid dash length rejected: {length}"
                )));
            }
        }
        let total: f64 = lengths.iter().sum();
        if !lengths.is_empty() && total <= 0.0 {
            return Err(CoreError::InvariantViolation(
                "dash pattern with zero total rejected".to_string(),
            ));
        }
        Ok(())
    }

    /// Re-check the canonical persisted form: inputs valid, an even
    /// (normalized) count or an empty list, and an offset inside the
    /// repeating period.
    pub fn validate(&self) -> Result<()> {
        let mut lengths = self.lengths.clone();
        Self::normalize_input(&mut lengths)?;
        if self.lengths.len() % 2 == 1 {
            return Err(CoreError::InvariantViolation(
                "dash pattern must persist normalized (even-count) lengths".to_string(),
            ));
        }
        if !self.offset.is_finite() {
            return Err(CoreError::InvariantViolation(format!(
                "non-finite dash offset rejected: {}",
                self.offset
            )));
        }
        if self.lengths.is_empty() {
            if self.offset != 0.0 {
                return Err(CoreError::InvariantViolation(format!(
                    "continuous dash must persist offset 0, got {}",
                    self.offset
                )));
            }
            return Ok(());
        }
        let period: f64 = self.lengths.iter().sum();
        if !(0.0..period).contains(&self.offset) {
            return Err(CoreError::InvariantViolation(format!(
                "dash offset outside its period: {}",
                self.offset
            )));
        }
        Ok(())
    }

    /// True for a continuous stroke.
    #[must_use]
    pub fn is_continuous(&self) -> bool {
        self.lengths.is_empty()
    }
}

/// One variable-width sample at a normalized arc-length position.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WidthPoint {
    pub position: f32,
    pub scale: f32,
}

/// Width profile over normalized arc length. Absolute distances stay
/// out so editing the path length reparametrizes the distribution
/// instead of shifting control points arbitrarily.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableWidthProfile {
    pub points: Vec<WidthPoint>,
}

impl VariableWidthProfile {
    /// Points must sort by position, stay in finite `0..1` positions
    /// with finite non-negative scales; duplicate positions merge by
    /// keeping the last scale.
    pub fn new(mut points: Vec<WidthPoint>) -> Result<Self> {
        Self::check_points(&points)?;
        points.sort_by(|a, b| {
            a.position
                .partial_cmp(&b.position)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        points.dedup_by(|later, earlier| {
            if later.position == earlier.position {
                *earlier = *later;
                true
            } else {
                false
            }
        });
        if points.is_empty() {
            return Err(CoreError::InvariantViolation(
                "width profile needs at least one point".to_string(),
            ));
        }
        Ok(Self { points })
    }

    fn check_points(points: &[WidthPoint]) -> Result<()> {
        for point in points {
            if !point.position.is_finite()
                || !(0.0..=1.0).contains(&point.position)
                || !point.scale.is_finite()
                || point.scale < 0.0
            {
                return Err(CoreError::InvariantViolation(format!(
                    "invalid width point rejected: {point:?}"
                )));
            }
        }
        Ok(())
    }

    /// Re-check the canonical persisted form: valid ranges, sorted
    /// positions with no duplicates, and at least one point.
    pub fn validate(&self) -> Result<()> {
        Self::check_points(&self.points)?;
        if self.points.is_empty() {
            return Err(CoreError::InvariantViolation(
                "width profile needs at least one point".to_string(),
            ));
        }
        for pair in self.points.windows(2) {
            if pair[0].position >= pair[1].position {
                return Err(CoreError::InvariantViolation(format!(
                    "width profile positions must be sorted and unique: {:?}",
                    self.points
                )));
            }
        }
        Ok(())
    }

    /// Linear interpolation between points, per the initial policy.
    #[must_use]
    pub fn sample(&self, position: f32) -> f32 {
        let first = self.points.first().expect("non-empty profile");
        let last = self.points.last().expect("non-empty profile");
        if position <= first.position {
            return first.scale;
        }
        if position >= last.position {
            return last.scale;
        }
        let window = self
            .points
            .windows(2)
            .find(|pair| position <= pair[1].position)
            .expect("covering window");
        let (a, b) = (window[0], window[1]);
        let span = b.position - a.position;
        if span <= 0.0 {
            return b.scale;
        }
        let t = (position - a.position) / span;
        a.scale + t * (b.scale - a.scale)
    }
}

/// Stroke intent; expanded outlines stay derived until Expand Stroke.
/// The stroke carries its own paint: a stroke item never borrows the
/// fill paint implicitly, so the rendered color stays explicit in the
/// stack order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrokeStyle {
    pub width: f64,
    pub cap: StrokeCap,
    pub join: StrokeJoin,
    pub miter_limit: f64,
    pub alignment: StrokeAlignment,
    pub dash: DashPattern,
    pub paint: ColorSource,
    pub variable_width: Option<VariableWidthProfile>,
    pub start_marker: Option<ObjectId>,
    pub end_marker: Option<ObjectId>,
}

impl StrokeStyle {
    /// Width must be finite and non-negative; miter limit finite.
    /// Markers persist reference plus parameters only; placement is
    /// derived from path tangent and never mutates path nodes.
    pub fn new(width: f64, cap: StrokeCap, join: StrokeJoin, paint: ColorSource) -> Result<Self> {
        if !width.is_finite() || width < 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid stroke width rejected: {width}"
            )));
        }
        Ok(Self {
            width,
            cap,
            join,
            miter_limit: 4.0,
            alignment: StrokeAlignment::Center,
            dash: DashPattern::new(Vec::new(), 0.0)?,
            paint,
            variable_width: None,
            start_marker: None,
            end_marker: None,
        })
    }

    /// Re-check the construction invariants on any instance,
    /// including deserialized ones.
    pub fn validate(&self) -> Result<()> {
        if !self.width.is_finite() || self.width < 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid stroke width rejected: {}",
                self.width
            )));
        }
        if !self.miter_limit.is_finite() {
            return Err(CoreError::InvariantViolation(format!(
                "non-finite miter limit rejected: {}",
                self.miter_limit
            )));
        }
        self.dash.validate()?;
        if let Some(profile) = &self.variable_width {
            profile.validate()?;
        }
        Ok(())
    }
}

/// Where an appearance comes from: local value, linked style, or
/// linked style with typed overrides. The three states stay distinct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AppearanceSource {
    Local(Appearance),
    Style(StyleId),
    StyleWithOverrides {
        style: StyleId,
        overrides: AppearanceOverrides,
    },
}

/// Typed per-item differences over a base style. Built-ins never use
/// arbitrary string properties.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AppearanceOverrides {
    pub items: Vec<AppearanceOverride>,
}

/// One override targets a stable item identity with a typed value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppearanceOverride {
    pub target: AppearanceItemId,
    pub value: AppearanceOverrideValue,
}

/// Typed override payloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AppearanceOverrideValue {
    Enabled(bool),
    Opacity(f32),
    BlendMode(BlendMode),
    Fill(Paint),
    Stroke(StrokeStyle),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::{Gradient, GradientGeometry, GradientInterpolation, GradientSpread};

    fn paint() -> Paint {
        Paint::Solid(ColorSource::Swatch(crate::id::SwatchId::new_v4()))
    }

    #[test]
    fn item_validates_opacity_range() {
        assert!(AppearanceItem::new(
            true,
            0.5,
            BlendMode::Multiply,
            AppearanceKind::Fill(paint())
        )
        .is_ok());
        assert!(
            AppearanceItem::new(true, 1.5, BlendMode::Normal, AppearanceKind::Fill(paint()))
                .is_err()
        );
    }

    #[test]
    fn dash_normalization_rules() {
        // Empty means continuous.
        let continuous = DashPattern::new(Vec::new(), 0.0).expect("valid");
        assert!(continuous.is_continuous());
        // Odd counts duplicate deterministically; offset wraps modularly.
        let odd = DashPattern::new(vec![4.0, 2.0, 1.0], 10.0).expect("valid");
        assert_eq!(odd.lengths, vec![4.0, 2.0, 1.0, 4.0, 2.0, 1.0]);
        assert_eq!(odd.offset, 10.0f64.rem_euclid(14.0));
        // Negative lengths and zero totals error.
        assert!(DashPattern::new(vec![-1.0], 0.0).is_err());
        assert!(DashPattern::new(vec![0.0, 0.0], 0.0).is_err());
    }

    #[test]
    fn width_profile_samples_linearly_between_points() {
        let profile = VariableWidthProfile::new(vec![
            WidthPoint {
                position: 0.0,
                scale: 1.0,
            },
            WidthPoint {
                position: 1.0,
                scale: 3.0,
            },
        ])
        .expect("valid");
        assert_eq!(profile.sample(0.0), 1.0);
        assert_eq!(profile.sample(0.5), 2.0);
        assert_eq!(profile.sample(1.0), 3.0);
    }

    #[test]
    fn stroke_style_carries_its_own_paint() {
        let paint = ColorSource::Swatch(crate::id::SwatchId::new_v4());
        let style = StrokeStyle::new(2.0, StrokeCap::Round, StrokeJoin::Round, paint.clone())
            .expect("valid");
        assert_eq!(style.width, 2.0);
        assert!(style.validate().is_ok());
        assert!(StrokeStyle::new(-1.0, StrokeCap::Butt, StrokeJoin::Miter, paint).is_err());
    }

    #[test]
    fn solid_black_matches_the_historical_default() {
        let appearance = Appearance::solid_black();
        assert_eq!(appearance.items.len(), 1);
        let AppearanceKind::Fill(Paint::Solid(ColorSource::Value(color))) =
            &appearance.items[0].kind
        else {
            panic!("default is one solid fill");
        };
        assert!(appearance.validate().is_ok());
        let _ = color;
    }

    #[test]
    fn appearance_source_states_stay_distinct() {
        let style = StyleId::new_v4();
        let local = AppearanceSource::Local(Appearance { items: Vec::new() });
        let linked = AppearanceSource::Style(style);
        assert_ne!(local, linked);
        // An empty stop list is not a gradient.
        assert!(Gradient::new(
            vec![],
            GradientInterpolation::LinearRgb,
            GradientSpread::Pad,
            crate::paint::PaintSpace::Object,
            GradientGeometry::Linear {
                start: crate::math::Point::new(0.0, 0.0),
                end: crate::math::Point::new(1.0, 1.0),
            },
        )
        .is_err());
    }
}
