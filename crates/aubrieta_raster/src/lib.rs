#![forbid(unsafe_code)]

//! Raster engine: tiles, brushes, masks, pixel storage (09.6).
//!
//! Sparse 128x128 CPU tiles form the canonical pixel layer representation.
//! 8-bit and 16-bit depths are V1 architectural requirements.

pub mod brush;
pub mod pixel;
pub mod tile;

pub use brush::{BlendMode, BrushDab, BrushInputSample};
pub use pixel::{AlphaMode, BitDepth, PixelFormat};
pub use tile::{Tile, TileCoord, TileMap, TileState, TILE_SIZE};
