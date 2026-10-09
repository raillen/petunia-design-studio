//! Groups, frames, views and statistics.
//!
//! Groups preserve composition semantics (isolation, masks, effects)
//!Simple subtrees may flatten when provably identical. Frames bind a
//! snapshot to a view, target and statistics for one render call.

use crate::effect::{RenderClip, RenderEffect, RenderMask};
use crate::primitive::RenderPrimitive;
use crate::snapshot::RenderSnapshot;
use petunia_core::{BlendMode, ObjectId, Rect};
use serde::{Deserialize, Serialize};

/// Whether a group composites in place or through an isolated
/// intermediate surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationMode {
    Isolated,
    Flattened,
}

/// A subtree with its own composition semantics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderGroup {
    pub source: ObjectId,
    pub children: Vec<RenderPrimitive>,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub mask: Option<RenderMask>,
    pub clip: Option<RenderClip>,
    pub effects: Vec<RenderEffect>,
    pub isolation: IsolationMode,
    pub bounds: Rect,
}

/// Document-to-device mapping for one frame.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ViewTransform {
    pub scale: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}

impl ViewTransform {
    /// Map a document point into device pixels.
    #[must_use]
    pub fn apply(&self, x: f64, y: f64) -> (f64, f64) {
        (
            x * self.scale + self.offset_x,
            y * self.scale + self.offset_y,
        )
    }
}

/// Offscreen render target descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RenderTarget {
    pub width: u32,
    pub height: u32,
}

/// Everything one render call needs: snapshot, view and target.
/// Backend options (quality, DPR policy) travel alongside in the
/// render crate, keeping this contract backend-free.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderFrame {
    pub snapshot: RenderSnapshot,
    pub view: ViewTransform,
    pub target: RenderTarget,
}

/// Per-frame statistics for tests and profiling.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct RenderStats {
    pub primitives_drawn: usize,
    pub tiles_processed: usize,
    pub pixels_written: u64,
}
