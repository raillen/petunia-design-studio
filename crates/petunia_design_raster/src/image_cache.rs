//! Content-addressed CPU image preparation. The cache counts live image owners,
//! not just entries, so eviction cannot hide pinned allocations from its budget.
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::{
    DecodedImage, EncodedImage, ImageAssetError, ImageContentKey, ImageDecodeLimits, PixelFormat,
};

#[derive(Clone, Copy, Debug)]
pub struct ImageCacheLimits {
    pub max_bytes: usize,
    pub max_entries: usize,
    pub decode: ImageDecodeLimits,
}
impl Default for ImageCacheLimits {
    fn default() -> Self {
        Self {
            max_bytes: 128 * 1024 * 1024,
            max_entries: 256,
            decode: ImageDecodeLimits::default(),
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct ImageCacheStats {
    pub entries: usize,
    pub live_bytes: usize,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
}

/// Premultiplied, encoded sRGB RGBA8 mip level, ready for a raster backend.
#[derive(Debug)]
pub struct ImageLevel {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}
impl ImageLevel {
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn premultiplied_rgba8(&self) -> &[u8] {
        &self.pixels
    }
}

#[derive(Debug)]
struct Residency {
    account: Arc<AtomicUsize>,
    bytes: usize,
}
impl Drop for Residency {
    fn drop(&mut self) {
        self.account.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}

/// Source precision is retained in the document's encoded bytes. These levels
/// are rebuildable RGBA8 display/export derivatives, not editable pixel storage.
#[derive(Debug)]
pub struct PreparedImage {
    key: ImageContentKey,
    source_format: PixelFormat,
    levels: Vec<ImageLevel>,
    profile: Option<Arc<Vec<u8>>>,
    _residency: Residency,
}
impl PreparedImage {
    pub fn content_key(&self) -> ImageContentKey {
        self.key
    }
    pub fn source_format(&self) -> PixelFormat {
        self.source_format
    }
    pub fn width(&self) -> u32 {
        self.levels[0].width
    }
    pub fn height(&self) -> u32 {
        self.levels[0].height
    }
    pub fn levels(&self) -> &[ImageLevel] {
        &self.levels
    }
    pub fn icc_profile(&self) -> Option<&[u8]> {
        self.profile.as_deref().map(Vec::as_slice)
    }
    /// Selects a conservative isotropic LOD by the largest device footprint.
    /// Strong anisotropy needs a separate filtering capability; no EWA claim.
    pub fn level_for_scale(&self, largest_texel_scale: f64) -> usize {
        if !largest_texel_scale.is_finite() || largest_texel_scale >= 1.0 {
            return 0;
        }
        let mut chosen = 0;
        for (index, level) in self.levels.iter().enumerate().skip(1) {
            let ratio = (f64::from(self.width()) / f64::from(level.width))
                .max(f64::from(self.height()) / f64::from(level.height));
            if largest_texel_scale * ratio > 1.0 {
                break;
            }
            chosen = index;
        }
        chosen
    }
}

struct Entry {
    value: Result<Arc<PreparedImage>, ImageAssetError>,
    touched: u64,
}
#[derive(Default)]
struct State {
    entries: HashMap<ImageContentKey, Entry>,
    clock: u64,
    hits: u64,
    misses: u64,
    evictions: u64,
}

/// Shared cache with LRU eviction and a single active cold preparation. Warm
/// reads do not hold the decoder lock. Serial cold decoding bounds overlapping
/// codec/scratch allocations and deduplicates concurrent misses.
pub struct ImageCache {
    limits: ImageCacheLimits,
    state: Mutex<State>,
    preparation: Mutex<()>,
    live_bytes: Arc<AtomicUsize>,
}
impl std::fmt::Debug for ImageCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageCache")
            .field("limits", &self.limits)
            .field("stats", &self.stats())
            .finish()
    }
}
impl ImageCache {
    pub fn new(limits: ImageCacheLimits) -> Result<Self, ImageAssetError> {
        if limits.max_entries == 0 || limits.max_bytes == 0 {
            return Err(ImageAssetError::Limit("empty image cache budget"));
        }
        Ok(Self {
            limits,
            state: Mutex::new(State::default()),
            preparation: Mutex::new(()),
            live_bytes: Arc::new(AtomicUsize::new(0)),
        })
    }
    /// Default cache shared by display, CPU export and worker renderer owners.
    pub fn shared() -> Arc<Self> {
        static CACHE: OnceLock<Arc<ImageCache>> = OnceLock::new();
        CACHE
            .get_or_init(|| {
                Arc::new(Self {
                    limits: ImageCacheLimits::default(),
                    state: Mutex::new(State::default()),
                    preparation: Mutex::new(()),
                    live_bytes: Arc::new(AtomicUsize::new(0)),
                })
            })
            .clone()
    }
    pub fn stats(&self) -> ImageCacheStats {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        ImageCacheStats {
            entries: state.entries.len(),
            live_bytes: self.live_bytes.load(Ordering::Acquire),
            hits: state.hits,
            misses: state.misses,
            evictions: state.evictions,
        }
    }
    fn hit(&self, key: ImageContentKey) -> Option<Result<Arc<PreparedImage>, ImageAssetError>> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.clock = state.clock.saturating_add(1);
        let clock = state.clock;
        let value = state.entries.get_mut(&key).map(|entry| {
            entry.touched = clock;
            entry.value.clone()
        });
        if value.is_some() {
            state.hits = state.hits.saturating_add(1);
        }
        value
    }
    pub fn prepare(
        &self,
        source: &EncodedImage,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<Arc<PreparedImage>, ImageAssetError> {
        if cancelled() {
            return Err(ImageAssetError::Cancelled);
        }
        if let Some(hit) = self.hit(source.content_key()) {
            return hit;
        }
        let _single_flight = self.preparation.lock().unwrap_or_else(|e| e.into_inner());
        if cancelled() {
            return Err(ImageAssetError::Cancelled);
        }
        if let Some(hit) = self.hit(source.content_key()) {
            return hit;
        }
        {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            state.misses = state.misses.saturating_add(1);
        }
        let result = self.prepare_cold(source, cancelled);
        if matches!(
            &result,
            Err(ImageAssetError::Cancelled | ImageAssetError::Limit(_))
        ) {
            return result;
        }
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        while state.entries.len() >= self.limits.max_entries {
            if !evict_one(&mut state) {
                return Err(ImageAssetError::Limit(
                    "image cache entries pinned by active owners",
                ));
            }
        }
        state.clock = state.clock.saturating_add(1);
        let touched = state.clock;
        state.entries.insert(
            source.content_key(),
            Entry {
                value: result.clone(),
                touched,
            },
        );
        result
    }

    fn prepare_cold(
        &self,
        source: &EncodedImage,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<Arc<PreparedImage>, ImageAssetError> {
        let mut decoded = crate::decode_image(source.as_slice(), self.limits.decode)?;
        if cancelled() {
            return Err(ImageAssetError::Cancelled);
        }
        let mut bytes = decoded.icc_profile.as_ref().map_or(0, |p| p.len());
        let mut dimensions = Vec::new();
        let (mut w, mut h) = (decoded.width, decoded.height);
        loop {
            let size = (w as usize)
                .checked_mul(h as usize)
                .and_then(|v| v.checked_mul(4))
                .ok_or(ImageAssetError::Limit("mip dimensions"))?;
            bytes = bytes
                .checked_add(size)
                .ok_or(ImageAssetError::Limit("mip bytes"))?;
            dimensions.push((w, h));
            if w == 1 && h == 1 {
                break;
            }
            w = (w / 2).max(1);
            h = (h / 2).max(1);
        }
        let pixels = u64::from(decoded.width) * u64::from(decoded.height);
        let next_pixels = dimensions
            .get(1)
            .map_or(0, |(w, h)| u64::from(*w) * u64::from(*h));
        let working = pixels
            .checked_add(next_pixels)
            .and_then(|n| n.checked_mul(16))
            .and_then(|n| n.checked_add(decoded.data.len() as u64))
            .and_then(|n| n.checked_add(bytes as u64))
            .ok_or(ImageAssetError::Limit("mip preparation working bytes"))?;
        if working > self.limits.decode.max_working_bytes || bytes > self.limits.max_bytes {
            return Err(ImageAssetError::Limit(
                "image pyramid working/resident bytes",
            ));
        }
        {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            while self
                .live_bytes
                .load(Ordering::Acquire)
                .checked_add(bytes)
                .is_none_or(|n| n > self.limits.max_bytes)
            {
                if !evict_one(&mut state) {
                    return Err(ImageAssetError::Limit(
                        "image bytes pinned by active owners",
                    ));
                }
            }
            self.live_bytes.fetch_add(bytes, Ordering::AcqRel);
        }
        let residency = Residency {
            account: self.live_bytes.clone(),
            bytes,
        };
        if let Some(profile) = decoded.icc_profile.as_ref() {
            petunia_design_color::rgb_profiles::convert_rgba_to_srgb(
                &mut decoded.data,
                decoded.format == PixelFormat::Rgba16,
                decoded.width,
                profile,
                cancelled,
            )
            .map_err(|e| {
                if cancelled() {
                    ImageAssetError::Cancelled
                } else {
                    ImageAssetError::Invalid(e.to_string())
                }
            })?;
        }
        let source_format = decoded.format;
        let mut current = linear_premultiplied(&decoded, cancelled)?;
        let profile = decoded.icc_profile;
        // Release the native decoded buffer before subsequent mip allocations.
        drop(decoded.data);
        let mut levels = Vec::with_capacity(dimensions.len());
        for (index, &(w, h)) in dimensions.iter().enumerate() {
            levels.push(display_level(w, h, &current, cancelled)?);
            if let Some(&(next_w, next_h)) = dimensions.get(index + 1) {
                current = area_downsample(&current, w, h, next_w, next_h, cancelled)?;
            }
        }
        Ok(Arc::new(PreparedImage {
            key: source.content_key(),
            source_format,
            levels,
            profile,
            _residency: residency,
        }))
    }
}

fn evict_one(state: &mut State) -> bool {
    let victim = state
        .entries
        .iter()
        .filter(|(_, entry)| match &entry.value {
            Err(_) => true,
            Ok(image) => Arc::strong_count(image) == 1,
        })
        .min_by_key(|(_, entry)| entry.touched)
        .map(|(key, _)| *key);
    if let Some(victim) = victim {
        state.entries.remove(&victim);
        state.evictions = state.evictions.saturating_add(1);
        true
    } else {
        false
    }
}

fn float_buffer(pixels: usize) -> Result<Vec<[f32; 4]>, ImageAssetError> {
    let mut out = Vec::new();
    out.try_reserve_exact(pixels)
        .map_err(|_| ImageAssetError::Limit("linear image scratch allocation"))?;
    out.resize(pixels, [0.0; 4]);
    Ok(out)
}
fn linear_premultiplied(
    image: &DecodedImage,
    cancelled: &dyn Fn() -> bool,
) -> Result<Vec<[f32; 4]>, ImageAssetError> {
    let mut output = float_buffer(image.width as usize * image.height as usize)?;
    for (i, out) in output.iter_mut().enumerate() {
        if i % image.width as usize == 0 && cancelled() {
            return Err(ImageAssetError::Cancelled);
        }
        let rgba: [f32; 4] = if image.format == PixelFormat::Rgba16 {
            std::array::from_fn(|c| {
                f32::from(u16::from_le_bytes([
                    image.data[i * 8 + c * 2],
                    image.data[i * 8 + c * 2 + 1],
                ])) / 65535.0
            })
        } else {
            std::array::from_fn(|c| f32::from(image.data[i * 4 + c]) / 255.0)
        };
        *out = [
            srgb_to_linear(rgba[0]) * rgba[3],
            srgb_to_linear(rgba[1]) * rgba[3],
            srgb_to_linear(rgba[2]) * rgba[3],
            rgba[3],
        ];
    }
    Ok(output)
}
fn display_level(
    width: u32,
    height: u32,
    linear: &[[f32; 4]],
    cancelled: &dyn Fn() -> bool,
) -> Result<ImageLevel, ImageAssetError> {
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(linear.len() * 4)
        .map_err(|_| ImageAssetError::Limit("display mip allocation"))?;
    for (i, rgba) in linear.iter().enumerate() {
        if i % width as usize == 0 && cancelled() {
            return Err(ImageAssetError::Cancelled);
        }
        let alpha = rgba[3].clamp(0.0, 1.0);
        let mut value = [0; 4];
        if alpha > 0.0 {
            for c in 0..3 {
                value[c] = (linear_to_srgb((rgba[c] / alpha).clamp(0.0, 1.0)) * alpha * 255.0)
                    .round() as u8;
            }
        }
        value[3] = (alpha * 255.0).round() as u8;
        pixels.extend_from_slice(&value);
    }
    Ok(ImageLevel {
        width,
        height,
        pixels,
    })
}

// Exact area footprints handle odd dimensions; a 3x1 source becoming 1x1
// weights all three source pixels equally rather than discarding its edge.
fn area_downsample(
    source: &[[f32; 4]],
    width: u32,
    height: u32,
    next_w: u32,
    next_h: u32,
    cancelled: &dyn Fn() -> bool,
) -> Result<Vec<[f32; 4]>, ImageAssetError> {
    let mut output = float_buffer(next_w as usize * next_h as usize)?;
    let sx = f64::from(width) / f64::from(next_w);
    let sy = f64::from(height) / f64::from(next_h);
    for y in 0..next_h {
        if cancelled() {
            return Err(ImageAssetError::Cancelled);
        }
        for x in 0..next_w {
            let x0 = f64::from(x) * sx;
            let x1 = f64::from(x + 1) * sx;
            let y0 = f64::from(y) * sy;
            let y1 = f64::from(y + 1) * sy;
            let mut sum = [0.0; 4];
            for iy in y0.floor() as u32..(y1.ceil() as u32).min(height) {
                for ix in x0.floor() as u32..(x1.ceil() as u32).min(width) {
                    let weight = ((x1.min(f64::from(ix + 1)) - x0.max(f64::from(ix)))
                        * (y1.min(f64::from(iy + 1)) - y0.max(f64::from(iy)))
                        / (sx * sy)) as f32;
                    for c in 0..4 {
                        sum[c] += source[iy as usize * width as usize + ix as usize][c] * weight;
                    }
                }
            }
            output[y as usize * next_w as usize + x as usize] = sum;
        }
    }
    Ok(output)
}
fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
fn linear_to_srgb(v: f32) -> f32 {
    if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}
