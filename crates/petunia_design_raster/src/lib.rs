#![forbid(unsafe_code)]

//! Raster engine: tiles, brushes, masks, pixel storage (09.6).
//!
//! Sparse 128x128 CPU tiles form the canonical pixel layer representation.
//! 8-bit and 16-bit depths are V1 architectural requirements.

pub mod brush;
pub mod cmyk;
mod cmyk_tiff;
pub mod image_assets;
mod image_cache;
pub mod layer;
pub mod pixel;
pub mod tile;

pub use brush::{BlendMode, BrushDab, BrushInputSample};
pub use image_assets::{
    decode_image, DecodedImage, EncodedImage, ImageAssetError, ImageContentKey, ImageDecodeLimits,
};
pub use image_cache::{ImageCache, ImageCacheLimits, ImageCacheStats, ImageLevel, PreparedImage};
pub use pixel::{AlphaMode, BitDepth, PixelFormat};
pub use tile::{Tile, TileCoord, TileMap, TileState, TILE_SIZE};

pub use cmyk::{InkChannel, InkCoverageReport, RasterDisplaySampler};
pub use layer::{RasterLayer, RasterLayerKind};
