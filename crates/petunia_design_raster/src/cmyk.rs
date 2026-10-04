//! Native process-ink operations. Display samples are disposable derivatives;
//! assignment, conversion, separation and alpha have distinct contracts.
use crate::{BitDepth, PixelFormat, RasterLayer, RasterLayerKind, TileCoord, TILE_SIZE};
use petunia_design_color::icc::{CmykDisplayTransform, CmykInkTransform};
use petunia_design_color::{IccProfile, IccTransformOptions};
use petunia_design_foundation::PetuniaError;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InkChannel {
    Cyan,
    Magenta,
    Yellow,
    Black,
}
impl InkChannel {
    pub fn index(self) -> usize {
        match self {
            Self::Cyan => 0,
            Self::Magenta => 1,
            Self::Yellow => 2,
            Self::Black => 3,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct InkCoverageReport {
    /// Literal process ink sum; alpha never masquerades as black.
    pub maximum_percent: f32,
    pub channel_maxima: [f32; 4],
    pub visible_pixels: u64,
    pub pixels_over_limit: u64,
    /// Caller-supplied press policy. No universal TAC ceiling is invented.
    pub limit_percent: Option<f32>,
}
fn invalid(reason: &'static str) -> PetuniaError {
    PetuniaError::invalid_input(reason)
}
fn check(cancelled: &dyn Fn() -> bool) -> Result<(), PetuniaError> {
    if cancelled() {
        Err(PetuniaError::cancelled("native ink operation cancelled"))
    } else {
        Ok(())
    }
}
impl RasterLayer {
    /// Imports exact little-endian/native 8-bit samples into bounded COW tiles.
    pub fn from_cmyka_bytes(
        width: u32,
        height: u32,
        format: PixelFormat,
        profile: IccProfile,
        data: &[u8],
    ) -> Result<Self, PetuniaError> {
        if !format.is_cmyk() {
            return Err(invalid("native CMYK sample layout"));
        }
        let count = (width as usize)
            .checked_mul(height as usize)
            .and_then(|n| n.checked_mul(format.bytes_per_pixel()))
            .ok_or_else(|| invalid("native CMYK dimensions overflow"))?;
        if data.len() != count || count > crate::tile::MAX_TILE_BYTES {
            return Err(invalid("native CMYK byte budget/layout"));
        }
        let mut layer = Self::new_cmyk(width, height, format.bit_depth(), profile)?;
        let bpp = format.bytes_per_pixel();
        for y in 0..height as usize {
            for tile_x in 0..width.div_ceil(TILE_SIZE as u32) as usize {
                let x = tile_x * TILE_SIZE;
                let samples = (width as usize - x).min(TILE_SIZE);
                let tile = layer
                    .tiles_mut()
                    .get_or_create_tile(TileCoord {
                        x: tile_x as i32,
                        y: (y / TILE_SIZE) as i32,
                    })
                    .ok_or_else(|| invalid("native CMYK resident tile budget"))?;
                let source = (y * width as usize + x) * bpp;
                let target = y % TILE_SIZE * TILE_SIZE * bpp;
                std::sync::Arc::make_mut(&mut tile.data)[target..target + samples * bpp]
                    .copy_from_slice(&data[source..source + samples * bpp]);
            }
        }
        layer.commit();
        layer.validate()?;
        Ok(layer)
    }
    /// Exact native ink/alpha bytes, never a display-derived RGB round trip.
    pub fn cmyka_bytes(&self, cancelled: &dyn Fn() -> bool) -> Result<Vec<u8>, PetuniaError> {
        check(cancelled)?;
        self.validate()?;
        if !self.is_cmyk() || self.cmyk_profile().is_none() {
            return Err(invalid("native CMYK profile missing"));
        }
        let bpp = self.tiles().format.bytes_per_pixel();
        let count = self.width() as usize * self.height() as usize * bpp;
        if count > crate::tile::MAX_TILE_BYTES {
            return Err(invalid("native CMYK dense output budget"));
        }
        let mut data = Vec::new();
        data.try_reserve_exact(count)
            .map_err(|_| invalid("native CMYK output allocation"))?;
        data.resize(count, 0);
        for y in 0..self.height() as usize {
            check(cancelled)?;
            for x in 0..self.width().div_ceil(TILE_SIZE as u32) as usize {
                let coord = TileCoord {
                    x: x as i32,
                    y: (y / TILE_SIZE) as i32,
                };
                if let Some(tile) = self.tiles().get_tile(coord) {
                    let columns = (self.width() as usize - x * TILE_SIZE).min(TILE_SIZE);
                    let from = y % TILE_SIZE * TILE_SIZE * bpp;
                    let to = (y * self.width() as usize + x * TILE_SIZE) * bpp;
                    data[to..to + columns * bpp]
                        .copy_from_slice(&tile.data[from..from + columns * bpp]);
                }
            }
        }
        Ok(data)
    }
    pub fn convert_cmyk_profile(
        &self,
        profile: IccProfile,
        options: IccTransformOptions,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<Self, PetuniaError> {
        check(cancelled)?;
        let source = self
            .cmyk_profile()
            .ok_or_else(|| invalid("native CMYK source profile missing"))?;
        if source.id() == profile.id() {
            return self.assign_cmyk_profile(profile);
        }
        let transform = CmykInkTransform::new(source, &profile, options)?;
        let mut next = self.assign_cmyk_profile(profile)?;
        for (coord, tile) in self.tiles().tiles() {
            check(cancelled)?;
            let rows = (self.height() as usize - coord.y as usize * TILE_SIZE).min(TILE_SIZE);
            let cols = (self.width() as usize - coord.x as usize * TILE_SIZE).min(TILE_SIZE);
            let target = next
                .tiles_mut()
                .get_or_create_tile(*coord)
                .ok_or_else(|| invalid("CMYK conversion tile budget"))?;
            for y in 0..rows {
                check(cancelled)?;
                let mut ink = Vec::with_capacity(cols);
                let mut alpha = Vec::with_capacity(cols);
                for x in 0..cols {
                    let p = tile.get_cmyka(x, y)?;
                    ink.push([p[0], p[1], p[2], p[3]]);
                    alpha.push(p[4]);
                }
                let mut converted = vec![[0.; 4]; cols];
                transform.convert(&ink, &mut converted)?;
                for (x, (p, a)) in converted.into_iter().zip(alpha).enumerate() {
                    target.set_cmyka(x, y, [p[0], p[1], p[2], p[3], a])?;
                }
            }
        }
        next.commit();
        next.validate()?;
        check(cancelled)?;
        Ok(next)
    }
    pub fn ink_coverage(
        &self,
        limit_percent: Option<f32>,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<InkCoverageReport, PetuniaError> {
        check(cancelled)?;
        if !self.is_cmyk()
            || limit_percent.is_some_and(|v| !v.is_finite() || !(0.0..=400.).contains(&v))
        {
            return Err(invalid("invalid CMYK TAC inspection policy"));
        }
        let mut report = InkCoverageReport {
            maximum_percent: 0.,
            channel_maxima: [0.; 4],
            visible_pixels: 0,
            pixels_over_limit: 0,
            limit_percent,
        };
        // Missing tiles are transparent. Inspect resident finite samples only.
        for (coord, tile) in self.tiles().tiles() {
            check(cancelled)?;
            let rows = (self.height() as usize - coord.y as usize * TILE_SIZE).min(TILE_SIZE);
            let cols = (self.width() as usize - coord.x as usize * TILE_SIZE).min(TILE_SIZE);
            for y in 0..rows {
                check(cancelled)?;
                for x in 0..cols {
                    let p = tile.get_cmyka(x, y)?;
                    if p[4] == 0. {
                        continue;
                    }
                    report.visible_pixels += 1;
                    let total = p[..4].iter().sum::<f32>() * 100.;
                    report.maximum_percent = report.maximum_percent.max(total);
                    for (maximum, value) in report.channel_maxima.iter_mut().zip(&p[..4]) {
                        *maximum = maximum.max(*value * 100.);
                    }
                    if limit_percent.is_some_and(|limit| total > limit) {
                        report.pixels_over_limit += 1;
                    }
                }
            }
        }
        Ok(report)
    }
    /// Process plate coverage = literal ink times independent alpha. This is
    /// an inspection derivative, not compositing/separation of a whole page.
    pub fn separation(
        &self,
        channel: InkChannel,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<Self, PetuniaError> {
        check(cancelled)?;
        if !self.is_cmyk() {
            return Err(invalid("process separation requires native CMYK"));
        }
        let mut plate = Self::new(
            self.width(),
            self.height(),
            RasterLayerKind::Mask,
            BitDepth::Sixteen,
        )?;
        for (coord, tile) in self.tiles().tiles() {
            let rows = (self.height() as usize - coord.y as usize * TILE_SIZE).min(TILE_SIZE);
            let cols = (self.width() as usize - coord.x as usize * TILE_SIZE).min(TILE_SIZE);
            for y in 0..rows {
                check(cancelled)?;
                for x in 0..cols {
                    let p = tile.get_cmyka(x, y)?;
                    plate.set_pixel(
                        coord.x as i64 * TILE_SIZE as i64 + x as i64,
                        coord.y as i64 * TILE_SIZE as i64 + y as i64,
                        [1., 1., 1., p[channel.index()] * p[4]],
                    )?;
                }
            }
        }
        plate.commit();
        Ok(plate)
    }
}

/// Per-operation LRU of at most eight ICC-derived tiles (2 MiB). Its lifetime
/// and transform belong to one worker, and never enter serialized documents.
pub struct RasterDisplaySampler<'a> {
    layer: &'a RasterLayer,
    transform: Option<CmykDisplayTransform>,
    cache: VecDeque<(TileCoord, Vec<[f32; 4]>)>,
}
impl<'a> RasterDisplaySampler<'a> {
    pub const MAX_CACHE_BYTES: usize = 8 * TILE_SIZE * TILE_SIZE * 16;
    pub fn width(&self) -> u32 {
        self.layer.width()
    }
    pub fn height(&self) -> u32 {
        self.layer.height()
    }
    pub fn new(layer: &'a RasterLayer) -> Result<Self, PetuniaError> {
        layer.validate()?;
        let transform = if layer.is_cmyk() {
            Some(CmykDisplayTransform::new(
                layer
                    .cmyk_profile()
                    .ok_or_else(|| invalid("CMYK display requires an ICC profile"))?,
                Default::default(),
            )?)
        } else {
            None
        };
        Ok(Self {
            layer,
            transform,
            cache: VecDeque::new(),
        })
    }
    pub fn pixel(&mut self, x: i64, y: i64) -> Result<[f32; 4], PetuniaError> {
        if x < 0
            || y < 0
            || x >= i64::from(self.layer.width())
            || y >= i64::from(self.layer.height())
        {
            return Ok([0.; 4]);
        }
        let Some(transform) = &self.transform else {
            return self.layer.pixel(x, y);
        };
        let coord = TileCoord::from_pixel(x, y).ok_or_else(|| invalid("display tile address"))?;
        let Some(tile) = self.layer.tiles().get_tile(coord) else {
            return Ok([0.; 4]);
        };
        let index = y.rem_euclid(TILE_SIZE as i64) as usize * TILE_SIZE
            + x.rem_euclid(TILE_SIZE as i64) as usize;
        if let Some(position) = self.cache.iter().position(|(key, _)| *key == coord) {
            let hit = self.cache.remove(position).expect("located cache entry");
            let pixel = hit.1[index];
            self.cache.push_back(hit);
            return Ok(pixel);
        }
        // Evict before allocating, so live derivative bytes never exceed eight tiles.
        if self.cache.len() == 8 {
            self.cache.pop_front();
        }
        let mut colors = Vec::new();
        colors
            .try_reserve_exact(TILE_SIZE * TILE_SIZE)
            .map_err(|_| invalid("ICC display tile allocation"))?;
        for row in 0..TILE_SIZE {
            let mut input = [[0.; 4]; TILE_SIZE];
            let mut alpha = [0.; TILE_SIZE];
            for x in 0..TILE_SIZE {
                let p = tile.get_cmyka(x, row)?;
                input[x] = [p[0], p[1], p[2], p[3]];
                alpha[x] = p[4];
            }
            let mut rgb = [[0.; 3]; TILE_SIZE];
            transform.convert_batch(&input, &mut rgb)?;
            colors.extend(
                rgb.into_iter()
                    .zip(alpha)
                    .map(|(rgb, alpha)| [rgb[0], rgb[1], rgb[2], alpha]),
            );
        }
        let pixel = colors[index];
        self.cache.push_back((coord, colors));
        Ok(pixel)
    }
}
