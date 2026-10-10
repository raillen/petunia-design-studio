//! Image and raster references: handles, never pixels.
//!
//! Decoded bytes live in render caches keyed by resource identity;
//! primitives carry the handle plus sampling and bounds.

use petunia_core::{ImageSamplingPolicy, ImageSourceRect, ObjectId, Rect, ResourceId, TileCoord};
use serde::{Deserialize, Serialize};

/// Placed image: resource handle, optional source crop, transform,
/// explicit sampling policy, opacity and conservative bounds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImagePrimitive {
    pub source: ObjectId,
    pub resource: ResourceId,
    pub source_rect: Option<ImageSourceRect>,
    pub transform: petunia_core::Transform2D,
    pub sampling: ImageSamplingPolicy,
    pub opacity: f32,
    pub bounds: Rect,
}

/// Pixel surface reference: surface handle plus an optional tile for
/// partial updates. The model never exposes mutable surface storage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RasterPrimitive {
    pub source: ObjectId,
    pub surface: ResourceId,
    pub tile: Option<TileCoord>,
    pub opacity: f32,
    pub bounds: Rect,
}

/// Immutable decoded linear RGB pixels shared by snapshot resources.
/// Resource loading/color conversion is performed before compilation.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<crate::paint::RenderColor>,
}
impl ResolvedImage {
    /// Checked image extent prevents invalid buffers entering the renderer.
    pub fn new(width: u32, height: u32, pixels: Vec<crate::paint::RenderColor>) -> Option<Self> {
        if width == 0
            || height == 0
            || (width as usize).checked_mul(height as usize)? != pixels.len()
        {
            return None;
        }
        if pixels.iter().any(|pixel| {
            !pixel.r.is_finite()
                || !pixel.g.is_finite()
                || !pixel.b.is_finite()
                || !pixel.a.is_finite()
                || !(0.0..=1.0).contains(&pixel.a)
        }) {
            return None;
        }
        Some(Self {
            width,
            height,
            pixels,
        })
    }
}
