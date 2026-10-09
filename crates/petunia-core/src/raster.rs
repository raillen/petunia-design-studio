//! Authorial raster model: pixel layers, surfaces and placed images.
//!
//! Editable surfaces and placed sources are document state. Brush
//! caches, decoded previews, mipmaps and thumbnails are derived state.
//! Surfaces use straight alpha canonically, matching `ColorValue`.

use crate::color::ColorSpaceRef;
use crate::crop::ImageSourceRect;
use crate::error::{CoreError, Result};
use crate::id::ResourceId;
use serde::{Deserialize, Serialize};

/// Pixel dimensions of a surface in whole pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelSize {
    pub width: u32,
    pub height: u32,
}

/// Authorial raster storage formats. Gray/CMYK storage waits for a
/// complete pipeline preserving their semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PixelFormat {
    Rgba8Unorm,
    Rgba16Unorm,
    Rgba16Float,
    Rgba32Float,
}

/// Stable integer tile coordinates inside a surface. The last row and
/// column may cover a smaller logical extent; algorithms never read
/// uninitialized padding as authorial pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileCoord {
    pub x: u32,
    pub y: u32,
}

/// Reference to surface bytes living in resource storage, never inside
/// a scene node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PixelSurfaceRef {
    pub resource: ResourceId,
}

/// Logical surface descriptor: size, storage format and color space.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PixelSurfaceDescriptor {
    pub size: PixelSize,
    pub format: PixelFormat,
    pub color_space: ColorSpaceRef,
}

impl PixelSurfaceDescriptor {
    /// Surfaces need at least one pixel in each dimension.
    pub fn new(size: PixelSize, format: PixelFormat, color_space: ColorSpaceRef) -> Result<Self> {
        if size.width == 0 || size.height == 0 {
            return Err(CoreError::InvariantViolation(
                "pixel surface needs non-zero dimensions".to_string(),
            ));
        }
        Ok(Self {
            size,
            format,
            color_space,
        })
    }
}

/// An editable raster surface. Tile storage stays logical
/// (`TileCoord` space); physical container layout may evolve without
/// changing this semantic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelLayer {
    pub surface: PixelSurfaceRef,
}

/// Sampling policy of a placed image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ImageSamplingPolicy {
    #[default]
    Bilinear,
    Nearest,
    Bicubic,
}

/// A placed immutable image source. The scene transform positions it;
/// local geometry derives from placed source plus crop. Resetting the
/// crop removes intent (`source_rect = None`), never resource bytes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ImageObject {
    pub resource: ResourceId,
    pub source_rect: Option<ImageSourceRect>,
    pub sampling: ImageSamplingPolicy,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::BuiltinColorSpace;

    #[test]
    fn surface_descriptor_requires_dimensions() {
        let space = ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb);
        assert!(PixelSurfaceDescriptor::new(
            PixelSize {
                width: 64,
                height: 64
            },
            PixelFormat::Rgba8Unorm,
            space.clone(),
        )
        .is_ok());
        assert!(PixelSurfaceDescriptor::new(
            PixelSize {
                width: 0,
                height: 64
            },
            PixelFormat::Rgba8Unorm,
            space,
        )
        .is_err());
    }

    #[test]
    fn raster_serialization_round_trip() {
        let layer = PixelLayer {
            surface: PixelSurfaceRef {
                resource: ResourceId::new_v4(),
            },
        };
        let json = serde_json::to_string(&layer).expect("serializable");
        let back: PixelLayer = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(back, layer);
    }
}
