//! Sparse 128x128 CPU tile storage (09.6).

use std::collections::BTreeMap;
use std::sync::Arc;

use petunia_design_geometry::GRect;
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
    pub fn from_pixel(px: i64, py: i64) -> Option<Self> {
        let size = TILE_SIZE as i64;
        let tx = i32::try_from(px.div_euclid(size)).ok()?;
        let ty = i32::try_from(py.div_euclid(size)).ok()?;
        Some(Self::new(tx, ty))
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
    #[serde(deserialize_with = "bounded_tile_bytes")]
    pub data: Arc<Vec<u8>>,
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
            data: Arc::new(vec![0u8; total_bytes]),
        }
    }

    /// Reads straight RGBA `[0.0, 1.0]`, converting the storage alpha mode.
    /// Sixteen-bit channels use canonical little-endian bytes.
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

        let mut color = match self.format {
            PixelFormat::Rgba8 => {
                let r = f32::from(self.data[offset]) / 255.0;
                let g = f32::from(self.data[offset + 1]) / 255.0;
                let b = f32::from(self.data[offset + 2]) / 255.0;
                let a = f32::from(self.data[offset + 3]) / 255.0;
                [r, g, b, a]
            }
            PixelFormat::Rgba16 => {
                let r_raw = u16::from_le_bytes([self.data[offset], self.data[offset + 1]]);
                let g_raw = u16::from_le_bytes([self.data[offset + 2], self.data[offset + 3]]);
                let b_raw = u16::from_le_bytes([self.data[offset + 4], self.data[offset + 5]]);
                let a_raw = u16::from_le_bytes([self.data[offset + 6], self.data[offset + 7]]);
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
                let v_raw = u16::from_le_bytes([self.data[offset], self.data[offset + 1]]);
                let v = f32::from(v_raw) / 65535.0;
                [v, v, v, 1.0]
            }
        };
        if self.alpha_mode == AlphaMode::Premultiplied && self.format.channels() == 4 {
            if color[3] == 0.0 {
                return [0.0; 4];
            }
            for channel in 0..3 {
                color[channel] = (color[channel] / color[3]).clamp(0.0, 1.0);
            }
        }
        color
    }

    /// Writes straight RGBA, converting to the declared storage alpha mode.
    pub fn set_pixel_normalized(&mut self, lx: usize, ly: usize, color: [f32; 4]) {
        if lx >= TILE_SIZE || ly >= TILE_SIZE {
            return;
        }
        let bpp = self.format.bytes_per_pixel();
        let offset = (ly * TILE_SIZE + lx) * bpp;
        if offset + bpp > self.data.len() {
            return;
        }

        let mut color = color.map(|v| {
            if v.is_finite() {
                v.clamp(0.0, 1.0)
            } else {
                0.0
            }
        });
        if self.alpha_mode == AlphaMode::Premultiplied && self.format.channels() == 4 {
            for channel in 0..3 {
                color[channel] *= color[3];
            }
        }
        self.state = TileState::ResidentWorkingDirty;

        let data = Arc::make_mut(&mut self.data);
        match self.format {
            PixelFormat::Rgba8 => {
                data[offset] = (color[0].clamp(0.0, 1.0) * 255.0).round() as u8;
                data[offset + 1] = (color[1].clamp(0.0, 1.0) * 255.0).round() as u8;
                data[offset + 2] = (color[2].clamp(0.0, 1.0) * 255.0).round() as u8;
                data[offset + 3] = (color[3].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            PixelFormat::Rgba16 => {
                let r = (color[0].clamp(0.0, 1.0) * 65535.0).round() as u16;
                let g = (color[1].clamp(0.0, 1.0) * 65535.0).round() as u16;
                let b = (color[2].clamp(0.0, 1.0) * 65535.0).round() as u16;
                let a = (color[3].clamp(0.0, 1.0) * 65535.0).round() as u16;
                data[offset..offset + 2].copy_from_slice(&r.to_le_bytes());
                data[offset + 2..offset + 4].copy_from_slice(&g.to_le_bytes());
                data[offset + 4..offset + 6].copy_from_slice(&b.to_le_bytes());
                data[offset + 6..offset + 8].copy_from_slice(&a.to_le_bytes());
            }
            PixelFormat::Gray8 => {
                data[offset] = (color[0].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            PixelFormat::Gray16 => {
                let v = (color[0].clamp(0.0, 1.0) * 65535.0).round() as u16;
                data[offset..offset + 2].copy_from_slice(&v.to_le_bytes());
            }
        }
    }
}

/// Sparse map of tiles composing a raster layer (09.6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileMap {
    pub format: PixelFormat,
    pub alpha_mode: AlphaMode,
    #[serde(with = "tile_entries")]
    tiles: BTreeMap<TileCoord, Arc<Tile>>,
}

impl TileMap {
    /// Creates an empty sparse tile map.
    #[must_use]
    pub fn new(format: PixelFormat, alpha_mode: AlphaMode) -> Self {
        Self {
            format,
            alpha_mode,
            tiles: BTreeMap::new(),
        }
    }

    /// Admits immutable binary tiles with duplicate, format and quota checks.
    pub fn from_tiles(
        format: PixelFormat,
        alpha_mode: AlphaMode,
        tiles: impl IntoIterator<Item = Arc<Tile>>,
    ) -> Result<Self, petunia_design_foundation::PetuniaError> {
        let mut map = Self::new(format, alpha_mode);
        let mut bytes = 0usize;
        for tile in tiles {
            bytes = bytes.saturating_add(tile.data.len());
            if map.tiles.len() >= MAX_RESIDENT_TILES
                || bytes > MAX_TILE_BYTES
                || map.tiles.insert(tile.coord, tile).is_some()
            {
                return Err(petunia_design_foundation::PetuniaError::invalid_input(
                    "duplicate or oversized tile resource",
                ));
            }
        }
        map.validate()?;
        Ok(map)
    }

    /// Number of allocated resident tiles. Absent tiles consume no memory.
    #[must_use]
    pub fn resident_tile_count(&self) -> usize {
        self.tiles.len()
    }

    /// Accesses an existing tile if resident.
    #[must_use]
    pub fn get_tile(&self, coord: TileCoord) -> Option<&Tile> {
        self.tiles.get(&coord).map(Arc::as_ref)
    }

    /// Obtains or allocates a working tile at the coordinate.
    pub fn get_or_create_tile(&mut self, coord: TileCoord) -> Option<&mut Tile> {
        let bytes = TILE_SIZE * TILE_SIZE * self.format.bytes_per_pixel();
        if !self.tiles.contains_key(&coord)
            && (self.tiles.len() >= MAX_RESIDENT_TILES
                || self.resident_bytes().checked_add(bytes)? > MAX_TILE_BYTES)
        {
            return None;
        }
        let format = self.format;
        let alpha_mode = self.alpha_mode;
        Some(Arc::make_mut(self.tiles.entry(coord).or_insert_with(
            || Arc::new(Tile::new_empty(coord, format, alpha_mode)),
        )))
    }

    /// Deterministic resident tile iteration. Snapshots share these immutable tiles.
    pub fn tiles(&self) -> impl Iterator<Item = (&TileCoord, &Arc<Tile>)> {
        self.tiles.iter()
    }

    /// Canonical pixel bytes, excluding the small sparse-map index.
    pub fn resident_bytes(&self) -> usize {
        self.tiles.values().map(|tile| tile.data.capacity()).sum()
    }

    /// Checks the storage contract before admitting a layer to the document.
    pub fn validate(&self) -> Result<(), petunia_design_foundation::PetuniaError> {
        if self.tiles.len() > MAX_RESIDENT_TILES || self.resident_bytes() > MAX_TILE_BYTES {
            return Err(petunia_design_foundation::PetuniaError::invalid_input(
                "raster tile budget exceeded",
            ));
        }
        let expected = TILE_SIZE * TILE_SIZE * self.format.bytes_per_pixel();
        if self.tiles.iter().any(|(coord, tile)| {
            tile.coord != *coord
                || tile.format != self.format
                || tile.alpha_mode != self.alpha_mode
                || tile.data.len() != expected
        }) {
            return Err(petunia_design_foundation::PetuniaError::invalid_input(
                "invalid raster tile storage",
            ));
        }
        Ok(())
    }

    /// Bytes newly retained by replacing this map with `next`; shared tiles are free.
    pub fn changed_retained_bytes(&self, next: &Self) -> usize {
        let previous = self
            .tiles
            .iter()
            .filter(|(coord, tile)| {
                !next
                    .tiles
                    .get(coord)
                    .is_some_and(|other| Arc::ptr_eq(&tile.data, &other.data))
            })
            .map(|(_, tile)| tile.data.capacity())
            .sum::<usize>();
        let added = next
            .tiles
            .iter()
            .filter(|(coord, tile)| {
                !self
                    .tiles
                    .get(coord)
                    .is_some_and(|other| Arc::ptr_eq(&tile.data, &other.data))
            })
            .map(|(_, tile)| tile.data.capacity())
            .sum::<usize>();
        previous
            .saturating_add(added)
            .saturating_add((self.tiles.len() + next.tiles.len()) * 64)
    }

    /// Reads a global pixel coordinate. Returns transparent if tile is absent.
    #[must_use]
    pub fn get_pixel(&self, px: i64, py: i64) -> [f32; 4] {
        let Some(coord) = TileCoord::from_pixel(px, py) else {
            return [0.0; 4];
        };
        let Some(tile) = self.tiles.get(&coord) else {
            return [0.0, 0.0, 0.0, 0.0];
        };
        let size = TILE_SIZE as i64;
        let lx = px.rem_euclid(size) as usize;
        let ly = py.rem_euclid(size) as usize;
        tile.get_pixel_normalized(lx, ly)
    }

    /// Writes a pixel at global layer coordinates, allocating tile if absent.
    pub fn set_pixel(&mut self, px: i64, py: i64, color: [f32; 4]) -> bool {
        let Some(coord) = TileCoord::from_pixel(px, py) else {
            return false;
        };
        let size = TILE_SIZE as i64;
        let lx = px.rem_euclid(size) as usize;
        let ly = py.rem_euclid(size) as usize;
        let Some(tile) = self.get_or_create_tile(coord) else {
            return false;
        };
        tile.set_pixel_normalized(lx, ly, color);
        true
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
        // Sparse zero coverage is represented by absence. Inspect only working
        // tiles; do not rescan immutable artwork on every completed gesture.
        self.tiles.retain(|_, tile| {
            if tile.state != TileState::ResidentWorkingDirty {
                return true;
            }
            match tile.format {
                PixelFormat::Gray8 | PixelFormat::Gray16 => {
                    tile.data.iter().any(|value| *value != 0)
                }
                PixelFormat::Rgba8 => tile.data.chunks_exact(4).any(|pixel| pixel[3] != 0),
                PixelFormat::Rgba16 => tile
                    .data
                    .chunks_exact(8)
                    .any(|pixel| pixel[6] != 0 || pixel[7] != 0),
            }
        });
        self.commit_retaining_tiles();
    }
    /// Coverage planes with a nonzero sparse background must retain zero tiles.
    pub fn commit_retaining_tiles(&mut self) {
        for tile in self.tiles.values_mut() {
            if tile.state == TileState::ResidentWorkingDirty {
                Arc::make_mut(tile).state = TileState::ResidentCommittedDirty;
            }
        }
    }
}

/// Per-layer ceilings also apply during deserialization, before document admission.
pub const MAX_RESIDENT_TILES: usize = 2048;
pub const MAX_TILE_BYTES: usize = 128 * 1024 * 1024;

fn bounded_tile_bytes<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Arc<Vec<u8>>, D::Error> {
    struct Bytes;
    impl<'de> serde::de::Visitor<'de> for Bytes {
        type Value = Vec<u8>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a bounded tile byte array")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<u8>, A::Error> {
            let max = TILE_SIZE * TILE_SIZE * 8;
            let mut bytes = Vec::with_capacity(seq.size_hint().unwrap_or(0).min(max));
            while let Some(byte) = seq.next_element::<u8>()? {
                if bytes.len() == max {
                    return Err(serde::de::Error::custom("tile byte limit"));
                }
                bytes.push(byte);
            }
            Ok(bytes)
        }
    }
    deserializer.deserialize_seq(Bytes).map(Arc::new)
}

// JSON object keys cannot be struct coordinates. A sorted tile sequence is
// portable, deterministic, and lets the reader reject duplicates and quotas.
mod tile_entries {
    use super::*;
    use serde::ser::SerializeSeq;
    pub fn serialize<S: serde::Serializer>(
        tiles: &BTreeMap<TileCoord, Arc<Tile>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(tiles.len()))?;
        for tile in tiles.values() {
            seq.serialize_element(tile)?;
        }
        seq.end()
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeMap<TileCoord, Arc<Tile>>, D::Error> {
        struct Tiles;
        impl<'de> serde::de::Visitor<'de> for Tiles {
            type Value = BTreeMap<TileCoord, Arc<Tile>>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("bounded unique raster tiles")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                let mut tiles = BTreeMap::new();
                let mut bytes = 0usize;
                while let Some(tile) = seq.next_element::<Tile>()? {
                    bytes = bytes.saturating_add(tile.data.len());
                    if tiles.len() >= MAX_RESIDENT_TILES || bytes > MAX_TILE_BYTES {
                        return Err(serde::de::Error::custom("raster tile budget exceeded"));
                    }
                    if tile.data.len() != TILE_SIZE * TILE_SIZE * tile.format.bytes_per_pixel()
                        || tiles.insert(tile.coord, Arc::new(tile)).is_some()
                    {
                        return Err(serde::de::Error::custom("invalid or duplicate raster tile"));
                    }
                }
                Ok(tiles)
            }
        }
        deserializer.deserialize_seq(Tiles)
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
