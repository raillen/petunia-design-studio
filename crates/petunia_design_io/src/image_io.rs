use crate::pdf::{DegradationItem, FidelityGrade};
use image::{codecs::jpeg::JpegEncoder, codecs::png::PngEncoder, ExtendedColorType, ImageEncoder};
use petunia_design_foundation::PetuniaError;
use petunia_design_raster::{PixelFormat, Tile, TileCoord, TILE_SIZE};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

/// Supported raster image file formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RasterFormat {
    /// Portable Network Graphics (supports 8-bit and 16-bit RGBA).
    Png,
    /// Joint Photographic Experts Group (8-bit RGB only, no alpha).
    Jpeg,
    /// WebP image format (8-bit RGBA).
    WebP,
    /// Tagged Image File Format (supports 8-bit and 16-bit RGBA).
    Tiff,
}

impl RasterFormat {
    /// Detects format from file extension.
    #[must_use]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "webp" => Some(Self::WebP),
            "tif" | "tiff" => Some(Self::Tiff),
            _ => None,
        }
    }

    /// Sniffs file format from magic header bytes.
    #[must_use]
    pub fn from_magic(bytes: &[u8]) -> Option<Self> {
        if bytes.len() >= 8 && bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Some(Self::Png);
        }
        if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
            return Some(Self::Jpeg);
        }
        if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
            return Some(Self::WebP);
        }
        if bytes.len() >= 4
            && ((bytes[0] == b'I' && bytes[1] == b'I' && bytes[2] == 0x2A && bytes[3] == 0x00)
                || (bytes[0] == b'M' && bytes[1] == b'M' && bytes[2] == 0x00 && bytes[3] == 0x2A))
        {
            return Some(Self::Tiff);
        }
        None
    }

    /// Returns whether this format supports native 16-bit per channel depth.
    #[must_use]
    pub const fn supports_16bit(&self) -> bool {
        match self {
            Self::Png | Self::Tiff => true,
            Self::Jpeg | Self::WebP => false,
        }
    }

    /// Returns whether this format supports transparency / alpha channel.
    #[must_use]
    pub const fn supports_alpha(&self) -> bool {
        match self {
            Self::Png | Self::WebP | Self::Tiff => true,
            Self::Jpeg => false,
        }
    }

    /// Standard MIME type string.
    #[must_use]
    pub const fn mime_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::WebP => "image/webp",
            Self::Tiff => "image/tiff",
        }
    }
}

/// In-memory raw raster image buffer with explicit bit depth and format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawRasterImage {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Pixel format (Rgba8 or Rgba16).
    pub format: PixelFormat,
    /// Straight-alpha bytes; 16-bit samples are little-endian on every platform.
    pub data: Vec<u8>,
}

impl RawRasterImage {
    /// Creates an 8-bit RGBA image from byte slice.
    pub fn from_rgba8(width: u32, height: u32, data: Vec<u8>) -> Result<Self, PetuniaError> {
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|px| px.checked_mul(4))
            .ok_or_else(|| PetuniaError::invalid_input("Image dimensions overflow"))?;

        if data.len() != expected {
            return Err(PetuniaError::invalid_input(format!(
                "Invalid RGBA8 byte length: expected {expected}, got {}",
                data.len()
            )));
        }

        Ok(Self {
            width,
            height,
            format: PixelFormat::Rgba8,
            data,
        })
    }

    /// Creates a 16-bit RGBA image from u16 slice (deep color).
    pub fn from_rgba16(width: u32, height: u32, data: &[u16]) -> Result<Self, PetuniaError> {
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|px| px.checked_mul(4))
            .ok_or_else(|| PetuniaError::invalid_input("Image dimensions overflow"))?;

        if data.len() != expected {
            return Err(PetuniaError::invalid_input(format!(
                "Invalid RGBA16 sample length: expected {expected}, got {}",
                data.len()
            )));
        }

        let mut byte_data = Vec::with_capacity(data.len() * 2);
        for &sample in data {
            byte_data.extend_from_slice(&sample.to_le_bytes());
        }

        Ok(Self {
            width,
            height,
            format: PixelFormat::Rgba16,
            data: byte_data,
        })
    }

    /// Creates a RawRasterImage directly from an `petunia_design_raster::Tile`.
    #[must_use]
    pub fn from_tile(tile: &Tile) -> Self {
        let mut data = tile.data.as_ref().clone();
        if tile.alpha_mode == petunia_design_raster::AlphaMode::Premultiplied {
            match tile.format {
                PixelFormat::Rgba8 => {
                    for pixel in data.chunks_exact_mut(4) {
                        let alpha = u32::from(pixel[3]);
                        for channel in &mut pixel[..3] {
                            *channel = if alpha == 0 {
                                0
                            } else {
                                ((u32::from(*channel) * 255 + alpha / 2) / alpha).min(255) as u8
                            };
                        }
                    }
                }
                PixelFormat::Rgba16 => {
                    for pixel in data.chunks_exact_mut(8) {
                        let alpha = u32::from(u16::from_le_bytes([pixel[6], pixel[7]]));
                        for channel in pixel[..6].chunks_exact_mut(2) {
                            let sample = u32::from(u16::from_le_bytes([channel[0], channel[1]]));
                            let value = if alpha == 0 {
                                0
                            } else {
                                ((sample * 65535 + alpha / 2) / alpha).min(65535) as u16
                            };
                            channel.copy_from_slice(&value.to_le_bytes());
                        }
                    }
                }
                PixelFormat::Gray8 | PixelFormat::Gray16 => {}
            }
        }
        Self {
            width: TILE_SIZE as u32,
            height: TILE_SIZE as u32,
            format: tile.format,
            data,
        }
    }

    /// Converts this RawRasterImage into an `petunia_design_raster::Tile` if dimensions are 128x128.
    pub fn to_tile(&self, coord: TileCoord) -> Result<Tile, PetuniaError> {
        let tile_dim = TILE_SIZE as u32;
        if self.width != tile_dim || self.height != tile_dim {
            return Err(PetuniaError::invalid_input(format!(
                "Tile requires {tile_dim}x{tile_dim} dimensions, got {}x{}",
                self.width, self.height
            )));
        }
        Ok(Tile {
            coord,
            format: self.format,
            alpha_mode: petunia_design_raster::AlphaMode::Straight,
            state: petunia_design_raster::TileState::ResidentWorkingDirty,
            data: self.data.clone().into(),
        })
    }

    /// Converts data into a vector of u16 samples regardless of source bit depth.
    #[must_use]
    pub fn to_rgba16(&self) -> Vec<u16> {
        match self.format {
            PixelFormat::Rgba16 => {
                let mut out = Vec::with_capacity(self.data.len() / 2);
                for chunk in self.data.as_chunks::<2>().0 {
                    out.push(u16::from_le_bytes([chunk[0], chunk[1]]));
                }
                out
            }
            PixelFormat::Rgba8 => {
                let mut out = Vec::with_capacity(self.data.len());
                for &b in &self.data {
                    // Scale 0..255 up to 0..65535
                    out.push(((b as u32 * 65535 + 127) / 255) as u16);
                }
                out
            }
            PixelFormat::Gray16 => {
                let mut out = Vec::with_capacity(self.data.len() * 2);
                for chunk in self.data.as_chunks::<2>().0 {
                    let g = u16::from_le_bytes([chunk[0], chunk[1]]);
                    out.extend_from_slice(&[g, g, g, 65535]);
                }
                out
            }
            PixelFormat::Gray8 => {
                let mut out = Vec::with_capacity(self.data.len() * 4);
                for &b in &self.data {
                    let g16 = ((b as u32 * 65535 + 127) / 255) as u16;
                    out.extend_from_slice(&[g16, g16, g16, 65535]);
                }
                out
            }
        }
    }

    /// Converts data into a vector of u8 samples, downsampling if source is 16-bit.
    #[must_use]
    pub fn to_rgba8(&self) -> Vec<u8> {
        match self.format {
            PixelFormat::Rgba8 => self.data.clone(),
            PixelFormat::Rgba16 => {
                let mut out = Vec::with_capacity(self.data.len() / 2);
                for chunk in self.data.as_chunks::<2>().0 {
                    let val = u16::from_le_bytes([chunk[0], chunk[1]]);
                    out.push((val >> 8) as u8);
                }
                out
            }
            PixelFormat::Gray8 => {
                let mut out = Vec::with_capacity(self.data.len() * 4);
                for &b in &self.data {
                    out.extend_from_slice(&[b, b, b, 255]);
                }
                out
            }
            PixelFormat::Gray16 => {
                let mut out = Vec::with_capacity(self.data.len() * 2);
                for chunk in self.data.as_chunks::<2>().0 {
                    let g = (u16::from_le_bytes([chunk[0], chunk[1]]) >> 8) as u8;
                    out.extend_from_slice(&[g, g, g, 255]);
                }
                out
            }
        }
    }
}

/// Configuration options for raster image export.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RasterExportOptions {
    /// Target image format.
    pub format: RasterFormat,
    /// JPEG compression quality (1 to 100).
    pub jpeg_quality: u8,
    /// Whether export is allowed when degradations occur.
    pub allow_degradations: bool,
}

impl Default for RasterExportOptions {
    fn default() -> Self {
        Self {
            format: RasterFormat::Png,
            jpeg_quality: 90,
            allow_degradations: true,
        }
    }
}

/// Encodes straight RGBA8/sRGB with explicit PNG physical resolution metadata.
/// Pixel dimensions are chosen by the renderer; this function does not resample.
pub fn export_png_rgba8_at_dpi(image: &RawRasterImage, dpi: f64) -> Result<Vec<u8>, PetuniaError> {
    if image.format != PixelFormat::Rgba8 || image.width == 0 || image.height == 0 {
        return Err(PetuniaError::invalid_input(
            "PNG density export requires nonempty straight RGBA8",
        ));
    }
    let bytes = (image.width as usize)
        .checked_mul(image.height as usize)
        .and_then(|v| v.checked_mul(4));
    if bytes != Some(image.data.len()) {
        return Err(PetuniaError::invalid_input("invalid PNG pixel data length"));
    }
    let ppm = (dpi / 0.0254).round();
    if !dpi.is_finite() || dpi <= 0.0 || ppm < 1.0 || ppm > f64::from(u32::MAX) {
        return Err(PetuniaError::invalid_input(
            "DPI is outside PNG physical-resolution range",
        ));
    }
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, image.width, image.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    encoder.set_pixel_dims(Some(png::PixelDimensions {
        xppu: ppm as u32,
        yppu: ppm as u32,
        unit: png::Unit::Meter,
    }));
    let mut writer = encoder
        .write_header()
        .map_err(|e| PetuniaError::io(format!("PNG header: {e}")))?;
    writer
        .write_image_data(&image.data)
        .map_err(|e| PetuniaError::io(format!("PNG pixels: {e}")))?;
    writer
        .finish()
        .map_err(|e| PetuniaError::io(format!("PNG finish: {e}")))?;
    Ok(bytes)
}

/// Encodes an in-memory image with explicit degradation analysis.
pub fn export_raster(
    image: &RawRasterImage,
    options: &RasterExportOptions,
) -> Result<(Vec<u8>, Vec<DegradationItem>), PetuniaError> {
    let mut degradations = Vec::new();

    // Check 16-bit downsampling degradation
    if image.format == PixelFormat::Rgba16 && !options.format.supports_16bit() {
        degradations.push(DegradationItem {
            code: "DEGRADATION_DEPTH_16_TO_8".to_string(),
            description: format!(
                "Target format {:?} does not support 16-bit depth; precision quantized to 8-bit",
                options.format
            ),
            grade: FidelityGrade::DestructiveDegradation,
        });
    }

    // Check alpha loss degradation
    if !options.format.supports_alpha() {
        degradations.push(DegradationItem {
            code: "DEGRADATION_ALPHA_STRIPPED".to_string(),
            description: format!(
                "Target format {:?} does not support transparency; alpha channel flattened",
                options.format
            ),
            grade: FidelityGrade::DestructiveDegradation,
        });
    }

    if !options.allow_degradations && !degradations.is_empty() {
        return Err(PetuniaError::invalid_input(
            "Raster export cancelled due to destructive degradation constraints",
        ));
    }

    let mut out = Vec::new();

    match options.format {
        RasterFormat::Png => {
            let cursor = Cursor::new(&mut out);
            let encoder = PngEncoder::new(cursor);
            let (color_type, byte_slice) = match image.format {
                PixelFormat::Rgba16 => (ExtendedColorType::Rgba16, image.data.as_slice()),
                PixelFormat::Rgba8 => (ExtendedColorType::Rgba8, image.data.as_slice()),
                PixelFormat::Gray16 => (ExtendedColorType::L16, image.data.as_slice()),
                PixelFormat::Gray8 => (ExtendedColorType::L8, image.data.as_slice()),
            };
            let native_bytes;
            let byte_slice = if cfg!(target_endian = "big")
                && matches!(image.format, PixelFormat::Rgba16 | PixelFormat::Gray16)
            {
                native_bytes = byte_slice
                    .chunks_exact(2)
                    .flat_map(|p| u16::from_le_bytes([p[0], p[1]]).to_ne_bytes())
                    .collect::<Vec<_>>();
                native_bytes.as_slice()
            } else {
                byte_slice
            };
            encoder
                .write_image(byte_slice, image.width, image.height, color_type)
                .map_err(|e| PetuniaError::io(format!("PNG encoding error: {e}")))?;
        }
        RasterFormat::Jpeg => {
            let cursor = Cursor::new(&mut out);
            let mut encoder =
                JpegEncoder::new_with_quality(cursor, options.jpeg_quality.clamp(1, 100));
            // JPEG requires RGB8 (no alpha)
            let rgba8 = image.to_rgba8();
            let mut rgb8 = Vec::with_capacity((image.width as usize) * (image.height as usize) * 3);
            for chunk in rgba8.as_chunks::<4>().0 {
                rgb8.push(chunk[0]);
                rgb8.push(chunk[1]);
                rgb8.push(chunk[2]);
            }
            encoder
                .encode(
                    &rgb8,
                    image.width,
                    image.height,
                    image::ExtendedColorType::Rgb8,
                )
                .map_err(|e| PetuniaError::io(format!("JPEG encoding error: {e}")))?;
        }
        RasterFormat::WebP => {
            let rgba8 = image.to_rgba8();
            let dyn_img = image::DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(image.width, image.height, rgba8).ok_or_else(|| {
                    PetuniaError::invalid_input("Failed to assemble WebP image buffer")
                })?,
            );
            let mut cursor = Cursor::new(&mut out);
            dyn_img
                .write_to(&mut cursor, image::ImageFormat::WebP)
                .map_err(|e| PetuniaError::io(format!("WebP encoding error: {e}")))?;
        }
        RasterFormat::Tiff => {
            let mut cursor = Cursor::new(&mut out);
            match image.format {
                PixelFormat::Rgba16 | PixelFormat::Gray16 => {
                    let u16_data = image.to_rgba16();
                    let dyn_img = image::DynamicImage::ImageRgba16(
                        image::ImageBuffer::from_raw(image.width, image.height, u16_data)
                            .ok_or_else(|| {
                                PetuniaError::invalid_input("Failed to assemble TIFF 16-bit buffer")
                            })?,
                    );
                    dyn_img
                        .write_to(&mut cursor, image::ImageFormat::Tiff)
                        .map_err(|e| {
                            PetuniaError::io(format!("TIFF 16-bit encoding error: {e}"))
                        })?;
                }
                PixelFormat::Rgba8 | PixelFormat::Gray8 => {
                    let rgba8_data = image.to_rgba8();
                    let dyn_img = image::DynamicImage::ImageRgba8(
                        image::RgbaImage::from_raw(image.width, image.height, rgba8_data)
                            .ok_or_else(|| {
                                PetuniaError::invalid_input("Failed to assemble TIFF 8-bit buffer")
                            })?,
                    );
                    dyn_img
                        .write_to(&mut cursor, image::ImageFormat::Tiff)
                        .map_err(|e| PetuniaError::io(format!("TIFF 8-bit encoding error: {e}")))?;
                }
            }
        }
    }

    Ok((out, degradations))
}

/// Reads one encoded source with a quota before allocating or decoding it.
/// A raced file growth is bounded by `take`; the renderer never calls this API.
pub fn read_encoded_image(
    path: &std::path::Path,
) -> Result<petunia_design_raster::EncodedImage, PetuniaError> {
    use std::io::Read;
    let file = std::fs::File::open(path)
        .map_err(|e| PetuniaError::io(format!("Cannot read image: {e}")))?;
    let metadata = file
        .metadata()
        .map_err(|e| PetuniaError::io(format!("Cannot inspect image: {e}")))?;
    if !metadata.is_file() || metadata.len() > petunia_design_raster::EncodedImage::MAX_BYTES as u64
    {
        return Err(PetuniaError::invalid_input(
            "Image must be a regular file within the encoded-source quota",
        ));
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(metadata.len() as usize)
        .map_err(|_| {
            PetuniaError::invalid_input("Image source allocation exceeds available memory")
        })?;
    let bound = petunia_design_raster::EncodedImage::MAX_BYTES + 1;
    let mut bounded = file.take(bound as u64);
    let mut chunk = [0; 32 * 1024];
    loop {
        let count = match bounded.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => count,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(PetuniaError::io(format!("Cannot read image: {error}"))),
        };
        let needed = bytes.len() + count;
        if needed > bytes.capacity() {
            let target = needed.max(bytes.capacity().saturating_mul(2).min(bound));
            bytes.try_reserve_exact(target - bytes.len()).map_err(|_| {
                PetuniaError::invalid_input("Image source allocation exceeds available memory")
            })?;
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
    petunia_design_raster::EncodedImage::new(bytes)
        .map_err(|e| PetuniaError::invalid_input(e.to_string()))
}

/// Imports untagged RGB/gray PNG/JPEG/TIFF/WebP with codec admission before
/// pixel decoding. Tagged ICC images require a CMM; this raw API has no profile
/// field and cannot silently discard that color contract.
pub fn import_raster(bytes: &[u8], max_bytes: usize) -> Result<RawRasterImage, PetuniaError> {
    let mut limits = petunia_design_raster::ImageDecodeLimits::default();
    limits.max_encoded_bytes = limits.max_encoded_bytes.min(max_bytes);
    limits.max_decoded_bytes = limits
        .max_decoded_bytes
        .min((max_bytes as u64).saturating_mul(4));
    let mut decoded = petunia_design_raster::decode_image(bytes, limits)
        .map_err(|e| PetuniaError::invalid_input(e.to_string()))?;
    if let Some(profile) = decoded.icc_profile.as_ref() {
        petunia_design_color::rgb_profiles::convert_rgba_to_srgb(
            &mut decoded.data,
            decoded.format == PixelFormat::Rgba16,
            decoded.width,
            profile,
            &|| false,
        )?;
    }
    Ok(RawRasterImage {
        width: decoded.width,
        height: decoded.height,
        format: decoded.format,
        data: decoded.data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_8bit_roundtrip() {
        let mut pixels = Vec::with_capacity(16 * 16 * 4);
        for i in 0..16 * 16 {
            pixels.push((i % 256) as u8);
            pixels.push(100);
            pixels.push(200);
            pixels.push(255);
        }

        let raw = RawRasterImage::from_rgba8(16, 16, pixels).unwrap();
        let options = RasterExportOptions {
            format: RasterFormat::Png,
            ..Default::default()
        };

        let (encoded, degradations) = export_raster(&raw, &options).unwrap();
        assert!(degradations.is_empty());
        assert!(encoded.starts_with(b"\x89PNG\r\n\x1a\n"));

        let imported = import_raster(&encoded, 1024 * 1024).unwrap();
        assert_eq!(imported.width, 16);
        assert_eq!(imported.height, 16);
        assert_eq!(imported.format, PixelFormat::Rgba8);
        assert_eq!(imported.data, raw.data);
    }

    #[test]
    fn png_16bit_depth_preserved_without_truncation() {
        let mut samples = Vec::with_capacity(8 * 8 * 4);
        for _ in 0..8 * 8 {
            samples.push(12345u16);
            samples.push(54321u16);
            samples.push(32000u16);
            samples.push(65535u16);
        }

        let raw = RawRasterImage::from_rgba16(8, 8, &samples).unwrap();
        let options = RasterExportOptions {
            format: RasterFormat::Png,
            ..Default::default()
        };

        let (encoded, degradations) = export_raster(&raw, &options).unwrap();
        assert!(degradations.is_empty());

        let imported = import_raster(&encoded, 1024 * 1024).unwrap();
        assert_eq!(imported.width, 8);
        assert_eq!(imported.height, 8);
        assert_eq!(imported.format, PixelFormat::Rgba16);
        assert_eq!(imported.to_rgba16(), samples);
    }

    #[test]
    fn jpeg_export_records_destructive_degradation_for_alpha() {
        let pixels = vec![
            255, 0, 0, 128, 0, 255, 0, 128, 0, 0, 255, 128, 255, 255, 255, 128,
        ];
        let raw = RawRasterImage::from_rgba8(2, 2, pixels).unwrap();
        let options = RasterExportOptions {
            format: RasterFormat::Jpeg,
            ..Default::default()
        };

        let (encoded, degradations) = export_raster(&raw, &options).unwrap();
        assert_eq!(degradations.len(), 1);
        assert_eq!(degradations[0].code, "DEGRADATION_ALPHA_STRIPPED");
        assert_eq!(degradations[0].grade, FidelityGrade::DestructiveDegradation);
        assert_eq!(RasterFormat::from_magic(&encoded), Some(RasterFormat::Jpeg));
    }

    #[test]
    fn raster_tile_conversion_roundtrip() {
        let coord = TileCoord::new(2, 3);
        let mut data = Vec::with_capacity(128 * 128 * 4);
        for _ in 0..128 * 128 {
            data.extend_from_slice(&[10, 20, 30, 255]);
        }
        let tile = Tile {
            coord,
            format: PixelFormat::Rgba8,
            alpha_mode: petunia_design_raster::AlphaMode::Straight,
            state: petunia_design_raster::TileState::ResidentWorkingDirty,
            data: data.clone(),
        };

        let image = RawRasterImage::from_tile(&tile);
        assert_eq!(image.width, 128);
        assert_eq!(image.height, 128);

        let roundtrip_tile = image.to_tile(coord).unwrap();
        assert_eq!(roundtrip_tile.coord, coord);
        assert_eq!(roundtrip_tile.format, PixelFormat::Rgba8);
        assert_eq!(roundtrip_tile.data, tile.data);
    }
}
