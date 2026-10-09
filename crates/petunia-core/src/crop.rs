//! Non-destructive crop and clipping relations.
//!
//! Cropping changes display intent; source pixels, paths and
//! resources stay preserved until an explicit destructive command.
//! Placed images crop through a normalized `source_rect`; generic
//! content reuses [`ClipBinding`] against a real scene object.

use crate::error::{CoreError, Result};
use crate::id::ObjectId;
use serde::{Deserialize, Serialize};

/// A point in normalized source space (`0..=1` on both axes).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NormalizedPoint {
    pub x: f64,
    pub y: f64,
}

impl NormalizedPoint {
    /// Both components must lie in `0..=1` so the crop survives
    /// resolution changes and relinked sources.
    pub fn new(x: f64, y: f64) -> Result<Self> {
        for component in [x, y] {
            if !component.is_finite() || !(0.0..=1.0).contains(&component) {
                return Err(CoreError::InvariantViolation(format!(
                    "normalized coordinate out of 0..=1: {component}"
                )));
            }
        }
        Ok(Self { x, y })
    }
}

/// Normalized crop rectangle of a placed image source.
/// `None` on the image means the full source.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ImageSourceRect {
    pub min: NormalizedPoint,
    pub max: NormalizedPoint,
}

impl ImageSourceRect {
    /// Enforces `0 <= min < max <= 1` on both axes.
    pub fn new(min: NormalizedPoint, max: NormalizedPoint) -> Result<Self> {
        if !(min.x < max.x && min.y < max.y) {
            return Err(CoreError::InvariantViolation(format!(
                "image source rect needs min < max, got {min:?}..{max:?}"
            )));
        }
        Ok(Self { min, max })
    }
}

/// How a clip source participates in painting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingSourceUse {
    /// The source exists only to feed the binding and paints nothing.
    BindingOnly,
    /// The source also paints normally.
    AlsoVisible,
}

/// A clip relation between scene objects: `target` stays whole while
/// evaluation intersects it with `source`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClipBinding {
    pub target: ObjectId,
    pub source: ObjectId,
    pub use_as: BindingSourceUse,
}

impl ClipBinding {
    /// Target and source must differ; a self-clip is rejected.
    pub fn new(target: ObjectId, source: ObjectId, use_as: BindingSourceUse) -> Result<Self> {
        if target == source {
            return Err(CoreError::InvariantViolation(
                "clip target and source must differ".to_string(),
            ));
        }
        Ok(Self {
            target,
            source,
            use_as,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: f64, y: f64) -> NormalizedPoint {
        NormalizedPoint::new(x, y).expect("valid test point")
    }

    #[test]
    fn source_rect_enforces_normalized_ordering() {
        assert!(ImageSourceRect::new(point(0.1, 0.1), point(0.9, 0.8)).is_ok());
        // Reversed corners are rejected.
        assert!(ImageSourceRect::new(point(0.9, 0.1), point(0.1, 0.8)).is_err());
        // Degenerate (zero-area) rects are rejected.
        assert!(ImageSourceRect::new(point(0.5, 0.5), point(0.5, 0.5)).is_err());
        // Out-of-range coordinates never enter the domain.
        assert!(NormalizedPoint::new(1.5, 0.5).is_err());
        assert!(NormalizedPoint::new(0.5, f64::NAN).is_err());
    }

    #[test]
    fn clip_binding_rejects_self_clip() {
        let id = ObjectId::new_v4();
        let other = ObjectId::new_v4();
        assert!(ClipBinding::new(id, other, BindingSourceUse::BindingOnly).is_ok());
        assert!(ClipBinding::new(id, id, BindingSourceUse::BindingOnly).is_err());
    }

    #[test]
    fn crop_serialization_round_trip() {
        let rect = ImageSourceRect::new(point(0.0, 0.0), point(1.0, 1.0)).expect("valid");
        let json = serde_json::to_string(&rect).expect("serializable");
        let back: ImageSourceRect = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(back, rect);
    }
}
