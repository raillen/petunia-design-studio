//! Sparse 128x128 CPU tile storage (09.6).

use std::collections::HashMap;

use aubrieta_geometry::GRect;
use serde::{Deserialize, Serialize};

use crate::pixel::{AlphaMode, PixelFormat};

/// Canonical logical CPU tile dimension: 128x128 pixels (09.6).
pub const TILE_SIZE: usize = 128;

/// 2D tile coordinate in layer space. Negative coordinates supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TileCoord {
    pub x: i32,
    pub y: i32,
}

impl TileCoord {
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Derives the tile coordinate containing a continuous pixel location.
    #[must_use]
    pub fn from_pixel(px: i64, py: i64) -> Self {
        let size = TILE_SIZE as i64;
        let tx = px.div_euclid(size) as i32;
        let ty = py.div_euclid(size) as i32;
        Self::new(tx, ty)
    }

    /// Pixel bounding box of this tile in layer space.
    #[must_use]
    pub fn bounds(self) -> GRect {
        let size = TILE_SIZE as f64;
        let x0 = f64::from(self.x) * size;
        let y0 = f64::from(self.y) * size;
        GRect::new(x0, y0, x0 + size, y0 + size)
    }
}

/// Lifecycle state of a raster tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileState {
    ResidentClean,
    ResidentWorkingDirty,
    ResidentCommittedDirty,
}

/// A resident logical CPU tile of 128x128 pixels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tile {
    pub coord: TileCoord,
    pub format: PixelFormat,
    pub alpha_mode: AlphaMode,
    pub state: TileState,
    pub data: Vec<u8>,
}

impl Tile {
    /// Allocates a new empty (transparent black) tile.
    #[must_use]
    pub fn new_empty(coord: TileCoord, format: PixelFormat, alpha_mode: AlphaMode) -> Self {
        let total_bytes = TILE_SIZE * TILE_SIZE * format.bytes_per_pixel();
        Self {
            coord,
            format,
            alpha_mode,
            state: TileState::ResidentWorkingDirty,
            data: vec![0u8; total_bytes],
        }
    }

    /// Reads a pixel as normalized RGBA `[0.0, 1.0]`.
    #[must_use]
    pub fn get_pixel_normalized(&self, lx: usize, ly: usize) -> [f32; 4] {
        if lx >= TILE_SIZE || ly >= TILE_SIZE {
            return [0.0, 0.0, 0.0, 0.0];
        }
        let bpp = self.format.bytes_per_pixel();
        let offset = (ly * TILE_SIZE + lx) * bpp;
        if offset + bpp > self.data.len() {
            return [0.0, 0.0, 0.0, 0.0];
        }

        match self.format {
            PixelFormat::Rgba8 => {
                let r = f32::from(self.data[offset]) / 255.0;
                let g = f32::from(self.data[offset + 1]) / 255.0;
                let b = f32::from(self.data[offset + 2]) / 255.0;
                let a = f32::from(self.data[offset + 3]) / 255.0;
                [r, g, b, a]
            }
            PixelFormat::Rgba16 => {
                let r_raw = u16::from_ne_bytes([self.data[offset], self.data[offset + 1]]);
                let g_raw = u16::from_ne_bytes([self.data[offset + 2], self.data[offset + 3]]);
                let b_raw = u16::from_ne_bytes([self.data[offset + 4], self.data[offset + 5]]);
                let a_raw = u16::from_ne_bytes([self.data[offset + 6], self.data[offset + 7]]);
                [
                    f32::from(r_raw) / 65535.0,
                    f32::from(g_raw) / 65535.0,
                    f32::from(b_raw) / 65535.0,
                    f32::from(a_raw) / 65535.0,
                ]
            }
            PixelFormat::Gray8 => {
                let v = f32::from(self.data[offset]) / 255.0;
                [v, v, v, 1.0]
            }
            PixelFormat::Gray16 => {
                let v_raw = u16::from_ne_bytes([self.data[offset], self.data[offset + 1]]);
                let v = f32::from(v_raw) / 65535.0;
                [v, v, v, 1.0]
            }
        }
    }

    /// Sets a pixel with normalized RGBA `[0.0, 1.0]`.
    pub fn set_pixel_normalized(&mut self, lx: usize, ly: usize, color: [f32; 4]) {
        if lx >= TILE_SIZE || ly >= TILE_SIZE {
            return;
        }
        let bpp = self.format.bytes_per_pixel();
        let offset = (ly * TILE_SIZE + lx) * bpp;
        if offset + bpp > self.data.len() {
            return;
        }

        self.state = TileState::ResidentWorkingDirty;

        match self.format {
            PixelFormat::Rgba8 => {
                self.data[offset] = (color[0].clamp(0.0, 1.0) * 255.0).round() as u8;
                self.data[offset + 1] = (color[1].clamp(0.0, 1.0) * 255.0).round() as u8;
                self.data[offset + 2] = (color[2].clamp(0.0, 1.0) * 255.0).round() as u8;
                self.data[offset + 3] = (color[3].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            PixelFormat::Rgba16 => {
                let r = (color[0].clamp(0.0, 1.0) * 65535.0).round() as u16;
                let g = (color[1].clamp(0.0, 1.0) * 65535.0).round() as u16;
                let b = (color[2].clamp(0.0, 1.0) * 65535.0).round() as u16;
                let a = (color[3].clamp(0.0, 1.0) * 65535.0).round() as u16;
                self.data[offset..offset + 2].copy_from_slice(&r.to_ne_bytes());
                self.data[offset + 2..offset + 4].copy_from_slice(&g.to_ne_bytes());
                self.data[offset + 4..offset + 6].copy_from_slice(&b.to_ne_bytes());
                self.data[offset + 6..offset + 8].copy_from_slice(&a.to_ne_bytes());
            }
            PixelFormat::Gray8 => {
                self.data[offset] = (color[0].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            PixelFormat::Gray16 => {
                let v = (color[0].clamp(0.0, 1.0) * 65535.0).round() as u16;
                self.data[offset..offset + 2].copy_from_slice(&v.to_ne_bytes());
            }
        }
    }
}

/// Sparse map of tiles composing a raster layer (09.6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileMap {
    pub format: PixelFormat,
    pub alpha_mode: AlphaMode,
    tiles: HashMap<TileCoord, Tile>,
}

impl TileMap {
    /// Creates an empty sparse tile map.
    #[must_use]
    pub fn new(format: PixelFormat, alpha_mode: AlphaMode) -> Self {
        Self {
            format,
            alpha_mode,
            tiles: HashMap::new(),
        }
    }

    /// Number of allocated resident tiles. Absent tiles consume no memory.
    #[must_use]
    pub fn resident_tile_count(&self) -> usize {
        self.tiles.len()
    }

    /// Accesses an existing tile if resident.
    #[must_use]
    pub fn get_tile(&self, coord: TileCoord) -> Option<&Tile> {
        self.tiles.get(&coord)
    }

    /// Obtains or allocates a working tile at the coordinate.
    pub fn get_or_create_tile(&mut self, coord: TileCoord) -> &mut Tile {
        let format = self.format;
        let alpha_mode = self.alpha_mode;
        self.tiles
            .entry(coord)
            .or_insert_with(|| Tile::new_empty(coord, format, alpha_mode))
    }

    /// Reads a global pixel coordinate. Returns transparent if tile is absent.
    #[must_use]
    pub fn get_pixel(&self, px: i64, py: i64) -> [f32; 4] {
        let coord = TileCoord::from_pixel(px, py);
        let Some(tile) = self.tiles.get(&coord) else {
            return [0.0, 0.0, 0.0, 0.0];
        };
        let size = TILE_SIZE as i64;
        let lx = px.rem_euclid(size) as usize;
        let ly = py.rem_euclid(size) as usize;
        tile.get_pixel_normalized(lx, ly)
    }

    /// Writes a pixel at global layer coordinates, allocating tile if absent.
    pub fn set_pixel(&mut self, px: i64, py: i64, color: [f32; 4]) {
        let coord = TileCoord::from_pixel(px, py);
        let size = TILE_SIZE as i64;
        let lx = px.rem_euclid(size) as usize;
        let ly = py.rem_euclid(size) as usize;
        let tile = self.get_or_create_tile(coord);
        tile.set_pixel_normalized(lx, ly, color);
    }

    /// Computes the bounding box enclosing all resident non-empty tiles.
    #[must_use]
    pub fn bounds(&self) -> Option<GRect> {
        if self.tiles.is_empty() {
            return None;
        }
        let mut bounds: Option<GRect> = None;
        for coord in self.tiles.keys() {
            let tb = coord.bounds();
            bounds = match bounds {
                None => Some(tb),
                Some(b) => b.union(tb),
            };
        }
        bounds
    }

    /// Commits all working dirty tiles to committed state.
    pub fn commit(&mut self) {
        for tile in self.tiles.values_mut() {
            if tile.state == TileState::ResidentWorkingDirty {
                tile.state = TileState::ResidentCommittedDirty;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_tile_map_absent_tiles_are_transparent() {
        let map = TileMap::new(PixelFormat::Rgba8, AlphaMode::Straight);
        assert_eq!(map.resident_tile_count(), 0);
        assert_eq!(map.get_pixel(10, 10), [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn writing_pixel_allocates_tile_and_roundtrips() {
        let mut map = TileMap::new(PixelFormat::Rgba8, AlphaMode::Straight);
        map.set_pixel(130, 10, [1.0, 0.5, 0.0, 1.0]); // Falls into TileCoord { x: 1, y: 0 }
        assert_eq!(map.resident_tile_count(), 1);

        let px = map.get_pixel(130, 10);
        assert!((px[0] - 1.0).abs() < 0.01);
        assert!((px[1] - 0.5).abs() < 0.01);
        assert!((px[2] - 0.0).abs() < 0.01);
        assert!((px[3] - 1.0).abs() < 0.01);
    }

    #[test]
    fn deep_color_16bit_precision_preserved() {
        let mut map = TileMap::new(PixelFormat::Rgba16, AlphaMode::Straight);
        map.set_pixel(0, 0, [0.123_45, 0.678_91, 0.999_99, 1.0]);
        let px = map.get_pixel(0, 0);
        assert!((px[0] - 0.123_45).abs() < 0.0001);
        assert!((px[1] - 0.678_91).abs() < 0.0001);
    }
}
