//! Re-evaluable effect parameters, ordered by pipeline phase.
//!
//! Geometry effects run before appearance; post-paint effects consume
//! painted results. Every instance carries a stable identity for
//! reorder, history and cache; each effect kind carries a persisted
//! schema version so documents migrate explicitly.

use crate::appearance::BlendMode;
use crate::error::{CoreError, Result};
use crate::id::{EffectId, ObjectId};
use crate::math::Vec2;
use crate::paint::ColorSource;
use serde::{Deserialize, Serialize};

/// One instance in a stack: identity, switches and the typed operation.
/// `mask` scopes only this operation, avoiding artificial groups for
/// localized effects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectInstance<T> {
    pub id: EffectId,
    pub enabled: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub mask: Option<ObjectId>,
    pub operation: T,
}

impl<T> EffectInstance<T> {
    /// Opacity must sit in `0..1`.
    pub fn new(
        enabled: bool,
        opacity: f32,
        blend_mode: BlendMode,
        mask: Option<ObjectId>,
        operation: T,
    ) -> Result<Self> {
        if !opacity.is_finite() || !(0.0..=1.0).contains(&opacity) {
            return Err(CoreError::InvariantViolation(format!(
                "effect opacity out of 0..=1: {opacity}"
            )));
        }
        Ok(Self {
            id: EffectId::new_v4(),
            enabled,
            opacity,
            blend_mode,
            mask,
            operation,
        })
    }
}

/// Ordered geometry effects: source geometry first, appearance after.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GeometryEffectStack {
    pub items: Vec<GeometryEffectInstance>,
}

/// One geometry effect instance.
pub type GeometryEffectInstance = EffectInstance<GeometryEffect>;

/// Built-in geometry operations as typed structs, never
/// `name + HashMap<String, Value>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GeometryEffect {
    Offset(OffsetParams),
    Corners(CornerParams),
    LiveBoolean(LiveBooleanParams),
}

impl GeometryEffect {
    /// Persisted schema version of geometry effect parameters.
    pub const SCHEMA_VERSION: u32 = 1;
}

/// Contour offset intent.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OffsetParams {
    pub distance: Vec2,
}

/// Live corner rounding intent (shared engine effect).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CornerParams {
    pub radius: f64,
}

impl CornerParams {
    /// Radius must be finite and non-negative.
    pub fn new(radius: f64) -> Result<Self> {
        if !radius.is_finite() || radius < 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid corner radius rejected: {radius}"
            )));
        }
        Ok(Self { radius })
    }
}

/// Boolean set operation vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BooleanOperation {
    Union,
    Intersection,
    Difference,
    Exclusion,
}

/// Live boolean intent evaluated by the geometry engine.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LiveBooleanParams {
    pub operation: BooleanOperation,
}

/// Ordered post-paint effects over already painted results.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PostPaintEffectStack {
    pub items: Vec<PostPaintEffectInstance>,
}

/// One post-paint effect instance.
pub type PostPaintEffectInstance = EffectInstance<PostPaintEffect>;

/// Built-in post-paint operations. Adjustment kinds gain their own
/// schemas when implemented; the reference stays an opaque object ID
/// until then.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PostPaintEffect {
    GaussianBlur(BlurParams),
    DropShadow(ShadowParams),
    InnerShadow(ShadowParams),
    Glow(GlowParams),
    Adjustment(ObjectId),
}

impl PostPaintEffect {
    /// Persisted schema version of post-paint effect parameters.
    pub const SCHEMA_VERSION: u32 = 1;
}

/// Gaussian blur persists sigma, never UI radius. Zero sigma is a
/// valid passthrough.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BlurParams {
    pub sigma_x: f64,
    pub sigma_y: f64,
}

impl BlurParams {
    /// Sigmas must be finite and non-negative.
    pub fn new(sigma_x: f64, sigma_y: f64) -> Result<Self> {
        for sigma in [sigma_x, sigma_y] {
            if !sigma.is_finite() || sigma < 0.0 {
                return Err(CoreError::InvariantViolation(format!(
                    "invalid blur sigma rejected: {sigma}"
                )));
            }
        }
        Ok(Self { sigma_x, sigma_y })
    }
}

/// Shadow intent: coverage spread applies before blur, never as
/// "larger sigma". Semantic order is fixed in the compositor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowParams {
    pub offset: Vec2,
    pub sigma: Vec2,
    pub color: ColorSource,
    pub spread: f64,
}

impl ShadowParams {
    /// Spread must be finite; sigma components finite and non-negative.
    pub fn new(offset: Vec2, sigma: Vec2, color: ColorSource, spread: f64) -> Result<Self> {
        if !sigma.dx.is_finite() || !sigma.dy.is_finite() || sigma.dx < 0.0 || sigma.dy < 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid shadow sigma rejected: {sigma:?}"
            )));
        }
        if !spread.is_finite() {
            return Err(CoreError::InvariantViolation(format!(
                "non-finite shadow spread rejected: {spread}"
            )));
        }
        Ok(Self {
            offset,
            sigma,
            color,
            spread,
        })
    }
}

/// Glow intent: like a shadow without an offset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlowParams {
    pub sigma: Vec2,
    pub color: ColorSource,
    pub spread: f64,
}

impl GlowParams {
    /// Same range rules as [`ShadowParams`].
    pub fn new(sigma: Vec2, color: ColorSource, spread: f64) -> Result<Self> {
        if !sigma.dx.is_finite() || !sigma.dy.is_finite() || sigma.dx < 0.0 || sigma.dy < 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid glow sigma rejected: {sigma:?}"
            )));
        }
        if !spread.is_finite() {
            return Err(CoreError::InvariantViolation(format!(
                "non-finite glow spread rejected: {spread}"
            )));
        }
        Ok(Self {
            sigma,
            color,
            spread,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::appearance::BlendMode;
    use crate::color::{BuiltinColorSpace, ColorSpaceRef, ColorValue, ProcessColor};
    use crate::color::{ProcessColorValue, Rgba};

    fn color() -> ColorSource {
        ColorSource::Value(ColorValue::Process(ProcessColor {
            value: ProcessColorValue::Rgb(Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                alpha: 0.8,
            }),
            space: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
        }))
    }

    #[test]
    fn geometry_and_post_stacks_keep_phase_separate() {
        let geometry = GeometryEffectInstance::new(
            true,
            1.0,
            BlendMode::Normal,
            None,
            GeometryEffect::Offset(OffsetParams {
                distance: Vec2::new(4.0, 0.0),
            }),
        )
        .expect("valid");
        let post = PostPaintEffectInstance::new(
            true,
            0.5,
            BlendMode::Multiply,
            None,
            PostPaintEffect::GaussianBlur(BlurParams::new(2.0, 2.0).expect("valid")),
        )
        .expect("valid");
        assert_ne!(geometry.id, post.id);
        assert!(
            EffectInstance::new(true, 2.0, BlendMode::Normal, None, geometry.operation).is_err()
        );
    }

    #[test]
    fn blur_sigma_zero_passes_through() {
        assert!(BlurParams::new(0.0, 0.0).is_ok());
        assert!(BlurParams::new(-1.0, 0.0).is_err());
        assert!(CornerParams::new(f64::NAN).is_err());
    }

    #[test]
    fn shadow_keeps_coverage_before_blur() {
        let shadow = ShadowParams::new(Vec2::new(2.0, 2.0), Vec2::new(4.0, 4.0), color(), 2.0)
            .expect("valid");
        assert_eq!(shadow.spread, 2.0);
        assert!(
            ShadowParams::new(Vec2::new(0.0, 0.0), Vec2::new(-1.0, 0.0), color(), 0.0).is_err()
        );
    }

    #[test]
    fn effect_serialization_round_trip() {
        let stack = GeometryEffectStack {
            items: vec![GeometryEffectInstance::new(
                true,
                1.0,
                BlendMode::Normal,
                None,
                GeometryEffect::LiveBoolean(LiveBooleanParams {
                    operation: BooleanOperation::Union,
                }),
            )
            .expect("valid")],
        };
        let json = serde_json::to_string(&stack).expect("serializable");
        let back: GeometryEffectStack = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(back, stack);
    }
}
