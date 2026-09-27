//! Composition hierarchy, isolation groups, and clip/opacity propagation.
//!
//! Controls group isolation semantics, cumulative opacity multiplication,
//! and recursive clipping boundaries.

use crate::blend::BlendMode;
use petunia_design_geometry::GRect;
use serde::{Deserialize, Serialize};

/// Group isolation and appearance properties for composition nodes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IsolationGroup {
    /// Human-readable label for debugging/diagnostics.
    pub name: String,
    /// When true, children composite onto an initially transparent surface
    /// before being blended into the parent backdrop.
    pub isolated: bool,
    /// Group opacity in [0.0, 1.0].
    pub opacity: f32,
    /// Blend mode applied when compositing this group into its parent backdrop.
    pub blend_mode: BlendMode,
    /// Optional rectangular clipping mask applied to this group.
    pub clip_bounds: Option<GRect>,
}

impl Default for IsolationGroup {
    fn default() -> Self {
        Self {
            name: "DefaultGroup".to_string(),
            isolated: true,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            clip_bounds: None,
        }
    }
}

impl IsolationGroup {
    /// Creates a new isolated group with custom opacity and blend mode.
    #[must_use]
    pub fn new(name: impl Into<String>, opacity: f32, blend_mode: BlendMode) -> Self {
        Self {
            name: name.into(),
            isolated: true,
            opacity: opacity.clamp(0.0, 1.0),
            blend_mode,
            clip_bounds: None,
        }
    }

    /// Creates a non-isolated pass-through group.
    #[must_use]
    pub fn pass_through(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            isolated: false,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            clip_bounds: None,
        }
    }

    /// Sets clipping bounds for this group.
    #[must_use]
    pub fn with_clip(mut self, clip: GRect) -> Self {
        self.clip_bounds = Some(clip);
        self
    }
}

/// Evaluates cumulative opacity and effective clip along a composition ancestor chain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EffectiveContext {
    /// Multiplied opacity along all ancestors.
    pub cumulative_opacity: f32,
    /// Intersected clipping rectangle, or None if unclipped.
    pub effective_clip: Option<GRect>,
}

impl Default for EffectiveContext {
    fn default() -> Self {
        Self {
            cumulative_opacity: 1.0,
            effective_clip: None,
        }
    }
}

impl EffectiveContext {
    /// Combines current context with a child group.
    #[must_use]
    pub fn child_of(&self, group: &IsolationGroup) -> Self {
        let cumulative_opacity = (self.cumulative_opacity * group.opacity).clamp(0.0, 1.0);
        let effective_clip = match (self.effective_clip, group.clip_bounds) {
            (None, None) => None,
            (Some(c), None) => Some(c),
            (None, Some(c)) => Some(c),
            (Some(c1), Some(c2)) => c1.intersection(c2),
        };
        Self {
            cumulative_opacity,
            effective_clip,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_opacity_multiplies_down_hierarchy() {
        let parent = IsolationGroup::new("Parent", 0.5, BlendMode::Normal);
        let child = IsolationGroup::new("Child", 0.5, BlendMode::Multiply);

        let root_ctx = EffectiveContext::default();
        let parent_ctx = root_ctx.child_of(&parent);
        assert!((parent_ctx.cumulative_opacity - 0.5).abs() < 1e-6);

        let child_ctx = parent_ctx.child_of(&child);
        assert!((child_ctx.cumulative_opacity - 0.25).abs() < 1e-6);
    }

    #[test]
    fn clipping_intersects_properly() {
        let rect1 = GRect::new(0.0, 0.0, 100.0, 100.0);
        let rect2 = GRect::new(50.0, 50.0, 150.0, 150.0);

        let g1 = IsolationGroup::default().with_clip(rect1);
        let g2 = IsolationGroup::default().with_clip(rect2);

        let ctx = EffectiveContext::default().child_of(&g1).child_of(&g2);
        let clip = ctx.effective_clip.expect("Should have intersected clip");
        assert_eq!(clip.x0, 50.0);
        assert_eq!(clip.y0, 50.0);
        assert_eq!(clip.x1, 100.0);
        assert_eq!(clip.y1, 100.0);
    }
}
