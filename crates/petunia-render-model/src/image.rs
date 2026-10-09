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
