//! Copy-on-write tile store for raster surfaces.
//!
//! Tiles are small independent pixel blocks addressed by [`TileCoord`].
//! Snapshots share tile bytes through reference counting; a write
//! clones first, so undo swaps references instead of copying whole
//! canvases. Only intersected tiles are ever touched.

use crate::error::{EngineError, Result};
use petunia_core::{PixelFormat, TileCoord};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Bytes per pixel for authorial storage formats.
#[must_use]
pub fn bytes_per_pixel(format: PixelFormat) -> usize {
    match format {
        PixelFormat::Rgba8Unorm => 4,
        PixelFormat::Rgba16Unorm | PixelFormat::Rgba16Float => 8,
        PixelFormat::Rgba32Float => 16,
    }
}

/// One tile version: dimensions, format and shared bytes.
#[derive(Debug, Clone)]
pub struct TileVersion {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub bytes: Arc<Vec<u8>>,
}

impl TileVersion {
    /// Allocate a zeroed tile.
    pub fn zeroed(width: u32, height: u32, format: PixelFormat) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(EngineError::Execution(
                "tile needs non-zero dimensions".to_string(),
            ));
        }
        let byte_count = (width as usize)
            .checked_mul(height as usize)
            .and_then(|pixels| pixels.checked_mul(bytes_per_pixel(format)))
            .filter(|bytes| *bytes <= 16 << 20)
            .ok_or_else(|| EngineError::Execution("tile allocation exceeds 16 MiB guard".into()))?;
        Ok(Self {
            width,
            height,
            format,
            bytes: Arc::new(vec![0u8; byte_count]),
        })
    }

    /// True when no other owner shares these bytes.
    #[must_use]
    pub fn is_unique(&self) -> bool {
        Arc::strong_count(&self.bytes) == 1
    }
}

/// Logical tile map with copy-on-write edits and a dirty set for
/// renderer uploads. Tile size stays a measured runtime choice, not
/// an architectural constant frozen here.
#[derive(Debug, Clone, Default)]
pub struct TileStore {
    tiles: HashMap<TileCoord, TileVersion>,
    dirty: HashSet<TileCoord>,
    tile_size: u32,
    max_tiles: usize,
}

impl TileStore {
    /// Create a store with nominal tile size and a cap on tracked
    /// tiles. Both are guarded before any allocation happens.
    pub fn new(tile_size: u32, max_tiles: usize) -> Result<Self> {
        if tile_size == 0 || tile_size > 1024 {
            return Err(EngineError::Execution(format!(
                "tile size out of 1..=1024: {tile_size}"
            )));
        }
        if max_tiles == 0 {
            return Err(EngineError::Execution(
                "tile store needs a positive tile cap".to_string(),
            ));
        }
        Ok(Self {
            tiles: HashMap::new(),
            dirty: HashSet::new(),
            tile_size,
            max_tiles,
        })
    }

    /// Read a tile version without marking anything dirty.
    #[must_use]
    pub fn get(&self, coord: TileCoord) -> Option<&TileVersion> {
        self.tiles.get(&coord)
    }

    /// Number of tracked tiles.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    /// Nominal tile edge used for candidate sizing.
    #[must_use]
    pub fn tile_size(&self) -> u32 {
        self.tile_size
    }

    /// True when no tile is tracked.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    /// Tiles changed since the last [`Self::clear_dirty`].
    #[must_use]
    pub fn dirty_tiles(&self) -> Vec<TileCoord> {
        let mut dirty: Vec<TileCoord> = self.dirty.iter().copied().collect();
        dirty.sort_by_key(|coord| (coord.x, coord.y));
        dirty
    }

    /// Clear the dirty set after uploads.
    pub fn clear_dirty(&mut self) {
        self.dirty.clear();
    }

    /// Ensure a writable tile: clone shared bytes first (COW),
    /// allocate missing tiles zeroed, then hand out the bytes.
    /// Marks the tile dirty.
    pub fn write_tile(
        &mut self,
        coord: TileCoord,
        width: u32,
        height: u32,
        format: PixelFormat,
    ) -> Result<&mut Vec<u8>> {
        if !self.tiles.contains_key(&coord) {
            if self.tiles.len() >= self.max_tiles {
                return Err(EngineError::Execution(format!(
                    "tile cap {} reached at {coord:?}",
                    self.max_tiles
                )));
            }
            self.tiles
                .insert(coord, TileVersion::zeroed(width, height, format)?);
        }
        let tile = self
            .tiles
            .get_mut(&coord)
            .ok_or_else(|| EngineError::Execution("tile missing after allocation".into()))?;
        if tile.width != width || tile.height != height || tile.format != format {
            return Err(EngineError::Execution(
                "tile dimensions/format mismatch".into(),
            ));
        }
        if !tile.is_unique() {
            let cloned: Vec<u8> = tile.bytes.as_ref().clone();
            tile.bytes = Arc::new(cloned);
        }
        self.dirty.insert(coord);
        Ok(Arc::make_mut(&mut tile.bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coord(x: u32, y: u32) -> TileCoord {
        TileCoord { x, y }
    }

    #[test]
    fn cow_shares_until_write() {
        let mut store = TileStore::new(64, 16).expect("valid");
        store
            .write_tile(coord(0, 0), 64, 64, PixelFormat::Rgba8Unorm)
            .expect("allocates")[0] = 9;
        let shared = store.get(coord(0, 0)).expect("present").bytes.clone();
        assert_eq!(shared[0], 9);
        // A second write clones first: the old snapshot keeps 9.
        store
            .write_tile(coord(0, 0), 64, 64, PixelFormat::Rgba8Unorm)
            .expect("writes")[0] = 42;
        assert_eq!(shared[0], 9);
        assert_eq!(store.get(coord(0, 0)).expect("present").bytes[0], 42);
        assert_eq!(store.dirty_tiles(), vec![coord(0, 0)]);
        store.clear_dirty();
        assert!(store.dirty_tiles().is_empty());
    }

    #[test]
    fn guards_reject_absurd_stores() {
        assert!(TileStore::new(0, 16).is_err());
        assert!(TileStore::new(2048, 16).is_err());
        assert!(TileStore::new(64, 0).is_err());
        let mut store = TileStore::new(64, 1).expect("valid");
        store
            .write_tile(coord(0, 0), 64, 64, PixelFormat::Rgba8Unorm)
            .expect("first fits");
        assert!(store
            .write_tile(coord(1, 0), 64, 64, PixelFormat::Rgba8Unorm)
            .is_err());
    }
}

/// Atomic version exchange. Arc-backed copies preserve untouched tiles and let
/// cancel/undo/replay exchange references without rerunning brush dynamics.
#[derive(Debug, Clone)]
pub struct TileTransaction {
    before: TileStore,
    after: TileStore,
}
impl TileTransaction {
    #[must_use]
    pub fn new(before: TileStore, after: TileStore) -> Self {
        Self { before, after }
    }
    pub fn apply(&self, destination: &mut TileStore) {
        *destination = self.after.clone();
    }
    pub fn undo(&self, destination: &mut TileStore) {
        *destination = self.before.clone();
    }
    #[must_use]
    pub fn dirty_tiles(&self) -> Vec<TileCoord> {
        self.after.dirty_tiles()
    }
}
