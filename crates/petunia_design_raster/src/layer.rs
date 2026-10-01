//! Canonical, bounded sparse pixel and coverage-mask layers. Cloning a working
//! layer shares all resident tiles until a pixel in that tile changes.
use crate::{AlphaMode, BitDepth, BlendMode, BrushDab, PixelFormat, TileMap, TILE_SIZE};
use petunia_design_foundation::PetuniaError;
use petunia_design_geometry::{GAffine, GPoint};
use serde::{Deserialize, Serialize};

pub const MAX_LAYER_PIXELS: u64 = 16_777_216;
pub const MAX_LAYER_DIMENSION: u32 = 32_768;
pub const MAX_DAB_WORK: usize = 270_400;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RasterLayerKind {
    Pixels,
    Mask,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "LayerWire")]
pub struct RasterLayer {
    width: u32,
    height: u32,
    kind: RasterLayerKind,
    #[serde(default)]
    default_coverage: u16,
    tiles: TileMap,
}
#[derive(Deserialize)]
struct LayerWire {
    width: u32,
    height: u32,
    kind: RasterLayerKind,
    #[serde(default)]
    default_coverage: u16,
    tiles: TileMap,
}
impl TryFrom<LayerWire> for RasterLayer {
    type Error = PetuniaError;
    fn try_from(wire: LayerWire) -> Result<Self, Self::Error> {
        let layer = Self {
            width: wire.width,
            height: wire.height,
            kind: wire.kind,
            default_coverage: wire.default_coverage,
            tiles: wire.tiles,
        };
        layer.validate()?;
        layer.validate_padding()?;
        Ok(layer)
    }
}
impl RasterLayer {
    pub fn new(
        width: u32,
        height: u32,
        kind: RasterLayerKind,
        depth: BitDepth,
    ) -> Result<Self, PetuniaError> {
        let format = match (kind, depth) {
            (RasterLayerKind::Pixels, BitDepth::Eight) => PixelFormat::Rgba8,
            (RasterLayerKind::Pixels, BitDepth::Sixteen) => PixelFormat::Rgba16,
            (RasterLayerKind::Mask, BitDepth::Eight) => PixelFormat::Gray8,
            (RasterLayerKind::Mask, BitDepth::Sixteen) => PixelFormat::Gray16,
        };
        let layer = Self {
            width,
            height,
            kind,
            default_coverage: 0,
            tiles: TileMap::new(format, AlphaMode::Straight),
        };
        layer.validate()?;
        Ok(layer)
    }
    pub fn from_tiles(
        width: u32,
        height: u32,
        kind: RasterLayerKind,
        tiles: TileMap,
    ) -> Result<Self, PetuniaError> {
        let layer = Self {
            width,
            height,
            kind,
            default_coverage: 0,
            tiles,
        };
        layer.validate()?;
        layer.validate_padding()?;
        Ok(layer)
    }
    /// An opaque mask needs no resident tiles until edited. Missing mask tiles
    /// reveal the complete plane; pixels beyond its finite frame remain absent.
    pub fn opaque_mask(width: u32, height: u32, depth: BitDepth) -> Result<Self, PetuniaError> {
        let mut layer = Self::new(width, height, RasterLayerKind::Mask, depth)?;
        layer.default_coverage = u16::MAX;
        Ok(layer)
    }
    /// Attach verified binary tiles without losing sparse mask background metadata.
    pub fn with_tiles(&self, tiles: TileMap) -> Result<Self, PetuniaError> {
        let layer = Self {
            tiles,
            ..self.without_tiles()
        };
        layer.validate()?;
        layer.validate_padding()?;
        Ok(layer)
    }
    pub fn default_coverage(&self) -> u16 {
        self.default_coverage
    }
    /// Descriptor-only copy for the native binary resource index.
    pub fn without_tiles(&self) -> Self {
        Self {
            width: self.width,
            height: self.height,
            kind: self.kind,
            default_coverage: self.default_coverage,
            tiles: TileMap::new(self.tiles.format, self.tiles.alpha_mode),
        }
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn kind(&self) -> RasterLayerKind {
        self.kind
    }
    pub fn tiles(&self) -> &TileMap {
        &self.tiles
    }
    pub fn resident_bytes(&self) -> usize {
        self.tiles.resident_bytes()
    }
    pub fn commit(&mut self) {
        if self.default_coverage == 0 {
            self.tiles.commit();
        } else {
            self.tiles.commit_retaining_tiles();
        }
    }
    pub fn validate(&self) -> Result<(), PetuniaError> {
        if self.default_coverage != 0
            && (self.kind != RasterLayerKind::Mask || self.default_coverage != u16::MAX)
        {
            return Err(PetuniaError::invalid_input(
                "invalid sparse mask background",
            ));
        }
        if self.width == 0
            || self.height == 0
            || self.width > MAX_LAYER_DIMENSION
            || self.height > MAX_LAYER_DIMENSION
            || u64::from(self.width) * u64::from(self.height) > MAX_LAYER_PIXELS
        {
            return Err(PetuniaError::invalid_input(
                "raster layer dimensions exceed the pixel budget",
            ));
        }
        if (self.kind == RasterLayerKind::Pixels) != (self.tiles.format.channels() == 4) {
            return Err(PetuniaError::invalid_input(
                "pixel/mask layer format mismatch",
            ));
        }
        if self.kind == RasterLayerKind::Mask && self.tiles.alpha_mode != AlphaMode::Straight {
            return Err(PetuniaError::invalid_input(
                "coverage masks require straight storage",
            ));
        }
        self.tiles.validate()?;
        for (coord, tile) in self.tiles.tiles() {
            if coord.x < 0
                || coord.y < 0
                || coord.x as u32 >= self.width.div_ceil(TILE_SIZE as u32)
                || coord.y as u32 >= self.height.div_ceil(TILE_SIZE as u32)
            {
                return Err(PetuniaError::invalid_input("tile outside raster layer"));
            }
        }
        Ok(())
    }
    // Check bytes once at input admission. Thereafter fields are private and
    // every edit clips to the finite layer, so document validation stays O(tiles).
    fn validate_padding(&self) -> Result<(), PetuniaError> {
        for (coord, tile) in self.tiles.tiles() {
            // Padding is never artwork and must not carry hidden source data.
            if (coord.x as u32 + 1) * TILE_SIZE as u32 > self.width
                || (coord.y as u32 + 1) * TILE_SIZE as u32 > self.height
            {
                let bpp = tile.format.bytes_per_pixel();
                for y in 0..TILE_SIZE {
                    for x in 0..TILE_SIZE {
                        if (coord.x as usize * TILE_SIZE + x >= self.width as usize
                            || coord.y as usize * TILE_SIZE + y >= self.height as usize)
                            && tile.data[(y * TILE_SIZE + x) * bpp..(y * TILE_SIZE + x + 1) * bpp]
                                .iter()
                                .any(|v| *v != 0)
                        {
                            return Err(PetuniaError::invalid_input("nonzero raster edge padding"));
                        }
                    }
                }
            }
        }
        Ok(())
    }
    /// Straight RGBA for pixels; white with coverage alpha for masks.
    pub fn pixel(&self, x: i64, y: i64) -> [f32; 4] {
        if x < 0 || y < 0 || x >= i64::from(self.width) || y >= i64::from(self.height) {
            return [0.0; 4];
        }
        let coord = crate::TileCoord::from_pixel(x, y).expect("admitted plane coordinates");
        if self.kind == RasterLayerKind::Mask && self.tiles.get_tile(coord).is_none() {
            return [1., 1., 1., f32::from(self.default_coverage) / 65535.];
        }
        let pixel = self.tiles.get_pixel(x, y);
        if self.kind == RasterLayerKind::Mask {
            [1.0, 1.0, 1.0, pixel[0]]
        } else {
            pixel
        }
    }
    /// Bounded edit of one canonical pixel or coverage value. No tile is
    /// allocated to write zero into already absent storage.
    pub fn set_pixel(&mut self, x: i64, y: i64, color: [f32; 4]) -> Result<bool, PetuniaError> {
        if x < 0 || y < 0 || x >= i64::from(self.width) || y >= i64::from(self.height) {
            return Ok(false);
        }
        if color
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return Err(PetuniaError::invalid_input("invalid raster pixel"));
        }
        let color = if self.kind == RasterLayerKind::Mask {
            [color[3], color[3], color[3], 1.0]
        } else {
            color
        };
        let current = self.pixel(x, y);
        let current = if self.kind == RasterLayerKind::Mask {
            [current[3], current[3], current[3], 1.]
        } else {
            current
        };
        if current == color
            || (self.kind == RasterLayerKind::Pixels && current[3] == 0.0 && color[3] == 0.0)
            || (self.kind == RasterLayerKind::Mask && current[0] == color[0])
        {
            return Ok(false);
        }
        if self.default_coverage != 0 {
            let coord = crate::TileCoord::from_pixel(x, y).expect("admitted plane coordinates");
            if self.tiles.get_tile(coord).is_none() {
                let tile = self.tiles.get_or_create_tile(coord).ok_or_else(|| {
                    PetuniaError::invalid_input("mask tile allocation budget exceeded")
                })?;
                // Initialize only the finite part; native admission requires exterior padding to stay zero.
                let bytes = std::sync::Arc::make_mut(&mut tile.data);
                let bpp = tile.format.bytes_per_pixel();
                let rows = (i64::from(self.height) - i64::from(coord.y) * TILE_SIZE as i64)
                    .clamp(0, TILE_SIZE as i64) as usize;
                let cols = (i64::from(self.width) - i64::from(coord.x) * TILE_SIZE as i64)
                    .clamp(0, TILE_SIZE as i64) as usize;
                for row in 0..rows {
                    bytes[row * TILE_SIZE * bpp..(row * TILE_SIZE + cols) * bpp].fill(255);
                }
            }
        }
        if !self.tiles.set_pixel(x, y, color) {
            return Err(PetuniaError::invalid_input(
                "raster tile allocation budget exceeded",
            ));
        }
        Ok(true)
    }
    /// Stamp a circle in world coordinates through the complete layer affine.
    /// This preserves the brush shape under rotation/nonuniform object scaling.
    /// The caller supplies a prepared selection stencil, never polygon tests
    /// per pixel. Failures affect only the caller's disposable working layer.
    pub fn stamp(
        &mut self,
        dab: &BrushDab,
        pixels_to_world: GAffine,
        original: &RasterLayer,
        accumulation: &mut RasterLayer,
        master_opacity: f32,
        remaining_work: &mut usize,
        mut selection: impl FnMut(i64, i64) -> Result<f32, PetuniaError>,
    ) -> Result<bool, PetuniaError> {
        if self.width != original.width
            || self.height != original.height
            || self.kind != original.kind
            || self.width != accumulation.width
            || self.height != accumulation.height
            || accumulation.kind != RasterLayerKind::Mask
            || !master_opacity.is_finite()
            || !(0.0..=1.0).contains(&master_opacity)
        {
            return Err(PetuniaError::invalid_input(
                "invalid brush accumulation frame",
            ));
        }
        if !pixels_to_world.coeffs.iter().all(|v| v.is_finite())
            || ![dab.center_x, dab.center_y, dab.radius]
                .iter()
                .all(|v| v.is_finite())
            || !dab.hardness.is_finite()
            || !dab.opacity.is_finite()
            || dab
                .color
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            || !(0.0..=1.0).contains(&dab.hardness)
            || !(0.0..=1.0).contains(&dab.opacity)
        {
            return Err(PetuniaError::invalid_input("invalid raster dab"));
        }
        if dab.radius <= 0.0 || dab.opacity == 0.0 {
            return Ok(false);
        }
        let inverse = pixels_to_world
            .inverse()
            .filter(|v| v.coeffs.iter().all(|v| v.is_finite()))
            .ok_or_else(|| PetuniaError::invalid_input("singular brush frame"))?;
        let center = inverse.apply(GPoint::new(dab.center_x, dab.center_y));
        let [a, b, c, d, _, _] = inverse.coeffs;
        let rx = dab.radius * a.hypot(c);
        let ry = dab.radius * b.hypot(d);
        let x0 = (center.x - rx).floor().clamp(0.0, f64::from(self.width)) as i64;
        let y0 = (center.y - ry).floor().clamp(0.0, f64::from(self.height)) as i64;
        let x1 = (center.x + rx).ceil().clamp(0.0, f64::from(self.width)) as i64;
        let y1 = (center.y + ry).ceil().clamp(0.0, f64::from(self.height)) as i64;
        let work = ((x1 - x0) * (y1 - y0)) as usize;
        if work > MAX_DAB_WORK || work > *remaining_work {
            return Err(PetuniaError::invalid_input(
                "brush gesture pixel-work budget exceeded",
            ));
        }
        *remaining_work -= work;
        let mut changed = false;
        let [a, b, _, _, _, _] = pixels_to_world.coeffs;
        for y in y0..y1 {
            let mut world = pixels_to_world.apply(GPoint::new(x0 as f64 + 0.5, y as f64 + 0.5));
            for x in x0..x1 {
                let distance = (world.x - dab.center_x).hypot(world.y - dab.center_y) / dab.radius;
                world.x += a;
                world.y += b;
                if distance >= 1.0 {
                    continue;
                }
                let falloff = if distance <= f64::from(dab.hardness) {
                    1.0
                } else {
                    ((1.0 - distance) / (1.0 - f64::from(dab.hardness))) as f32
                };
                let coverage = (falloff * dab.opacity * selection(x, y)?).clamp(0.0, 1.0);
                if coverage == 0.0 {
                    continue;
                }
                let source_alpha = if dab.blend_mode == BlendMode::DestinationOut {
                    1.0
                } else {
                    dab.color[3]
                };
                let old_coverage = accumulation.pixel(x, y)[3];
                let cumulative = 1.0 - (1.0 - old_coverage) * (1.0 - coverage * source_alpha);
                accumulation.set_pixel(x, y, [1.0, 1.0, 1.0, cumulative.clamp(0.0, 1.0)])?;
                let coverage = cumulative * master_opacity;
                let dst = original.pixel(x, y);
                let result = if self.kind == RasterLayerKind::Mask {
                    let value = if dab.blend_mode == BlendMode::DestinationOut {
                        dst[3] * (1.0 - coverage)
                    } else {
                        let luminance =
                            0.2126 * dab.color[0] + 0.7152 * dab.color[1] + 0.0722 * dab.color[2];
                        dst[3] + (luminance - dst[3]) * coverage
                    };
                    [1.0, 1.0, 1.0, value.clamp(0.0, 1.0)]
                } else {
                    let mut src = dab.color;
                    src[3] = coverage;
                    dab.blend_mode.blend(src, dst)
                };
                changed |= self.set_pixel(x, y, result)?;
            }
        }
        Ok(changed)
    }
}
