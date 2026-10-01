//! Immutable encoded sources and bounded codec admission. No filesystem access.
use std::fmt;
use std::io::Cursor;
use std::sync::Arc;

use crate::PixelFormat;
use image::{DynamicImage, ExtendedColorType, ImageDecoder, ImageReader};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};

/// A typed content key, independent of file paths, pointers and document IDs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImageContentKey([u8; 32]);

/// Encoded source with a digest calculated once. Native serde remains an array
/// of bytes; imported precision/profiles/metadata remain in these original bytes.
#[derive(Clone, PartialEq, Eq)]
pub struct EncodedImage {
    bytes: Vec<u8>,
    key: ImageContentKey,
}

impl fmt::Debug for EncodedImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EncodedImage")
            .field("byte_len", &self.bytes.len())
            .field("key", &self.key)
            .finish()
    }
}
impl EncodedImage {
    /// Per-source bound at the native deserialization boundary.
    pub const MAX_BYTES: usize = 32 * 1024 * 1024;
    pub fn new(bytes: Vec<u8>) -> Result<Self, ImageAssetError> {
        if bytes.len() > Self::MAX_BYTES {
            return Err(ImageAssetError::Limit("encoded source bytes"));
        }
        let digest = Sha256::digest(&bytes);
        let mut key = [0; 32];
        key.copy_from_slice(&digest);
        Ok(Self {
            bytes,
            key: ImageContentKey(key),
        })
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }
    pub fn resident_bytes(&self) -> usize {
        self.bytes.capacity()
    }
    pub fn byte_len(&self) -> usize {
        self.bytes.len()
    }
    pub fn content_key(&self) -> ImageContentKey {
        self.key
    }
}
impl AsRef<[u8]> for EncodedImage {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}
impl Serialize for EncodedImage {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.bytes.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for EncodedImage {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = EncodedImage;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("an encoded image byte array within the source quota")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                let hint = seq.size_hint().unwrap_or(0);
                if hint > EncodedImage::MAX_BYTES {
                    return Err(serde::de::Error::custom(
                        "encoded image exceeds source quota",
                    ));
                }
                let mut bytes = Vec::new();
                bytes
                    .try_reserve(hint.min(64 * 1024))
                    .map_err(serde::de::Error::custom)?;
                while let Some(byte) = seq.next_element::<u8>()? {
                    if bytes.len() == EncodedImage::MAX_BYTES {
                        return Err(serde::de::Error::custom(
                            "encoded image exceeds source quota",
                        ));
                    }
                    if bytes.len() == bytes.capacity() {
                        let target = bytes
                            .capacity()
                            .saturating_mul(2)
                            .max(1)
                            .min(EncodedImage::MAX_BYTES);
                        bytes
                            .try_reserve_exact(target - bytes.len())
                            .map_err(serde::de::Error::custom)?;
                    }
                    bytes.push(byte);
                }
                EncodedImage::new(bytes).map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_seq(Visitor)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ImageAssetError {
    #[error("invalid image: {0}")]
    Invalid(String),
    #[error("image resource limit: {0}")]
    Limit(&'static str),
    #[error("unavailable image capability: {0}")]
    Unsupported(&'static str),
    #[error("image preparation cancelled")]
    Cancelled,
}

/// Explicit limits applied to codec setup, output and conversions.
#[derive(Clone, Copy, Debug)]
pub struct ImageDecodeLimits {
    pub max_encoded_bytes: usize,
    pub max_dimension: u32,
    pub max_pixels: u64,
    pub max_decoded_bytes: u64,
    pub max_working_bytes: u64,
    pub max_profile_bytes: usize,
}
impl Default for ImageDecodeLimits {
    fn default() -> Self {
        Self {
            max_encoded_bytes: EncodedImage::MAX_BYTES,
            max_dimension: 32_768,
            max_pixels: 16_777_216,
            max_decoded_bytes: 128 * 1024 * 1024,
            max_working_bytes: 512 * 1024 * 1024,
            max_profile_bytes: 4 * 1024 * 1024,
        }
    }
}

/// Oriented straight-alpha RGBA8 or little-endian RGBA16. Source bytes remain
/// untouched; profiles are retained, never silently interpreted as sRGB.
#[derive(Debug)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub data: Vec<u8>,
    pub icc_profile: Option<Arc<Vec<u8>>>,
}

/// Decodes supported RGB/gray PNG/JPEG/TIFF/WebP. CMYK/HDR are explicit missing
/// capabilities. Library allocation limits are supplemented by checked output
/// admission; codec-private allocation accounting remains best effort.
pub fn decode_image(
    bytes: &[u8],
    limits: ImageDecodeLimits,
) -> Result<DecodedImage, ImageAssetError> {
    decode_with_policy(bytes, limits, true)
}
pub(crate) fn decode_display_image(
    bytes: &[u8],
    limits: ImageDecodeLimits,
) -> Result<DecodedImage, ImageAssetError> {
    decode_with_policy(bytes, limits, false)
}
fn decode_with_policy(
    bytes: &[u8],
    limits: ImageDecodeLimits,
    allow_icc: bool,
) -> Result<DecodedImage, ImageAssetError> {
    if bytes.len() > limits.max_encoded_bytes {
        return Err(ImageAssetError::Limit("encoded file bytes"));
    }
    let format = image::guess_format(bytes).map_err(codec_error)?;
    if !matches!(
        format,
        image::ImageFormat::Png
            | image::ImageFormat::Jpeg
            | image::ImageFormat::Tiff
            | image::ImageFormat::WebP
    ) {
        return Err(ImageAssetError::Unsupported("image format"));
    }
    admit_display_metadata(bytes, format)?;
    if format == image::ImageFormat::Jpeg {
        match jpeg_components(bytes)? {
            1 | 3 => {}
            4 => {
                return Err(ImageAssetError::Unsupported(
                    "CMYK JPEG requires an ICC conversion",
                ));
            }
            _ => return Err(ImageAssetError::Unsupported("JPEG component layout")),
        }
    }
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut codec_limits = image::Limits::default();
    codec_limits.max_image_width = Some(limits.max_dimension);
    codec_limits.max_image_height = Some(limits.max_dimension);
    codec_limits.max_alloc = Some(limits.max_working_bytes);
    reader.limits(codec_limits);
    let mut decoder = reader.into_decoder().map_err(codec_error)?;
    if matches!(
        decoder.original_color_type(),
        ExtendedColorType::Cmyk8 | ExtendedColorType::Cmyk16
    ) {
        return Err(ImageAssetError::Unsupported(
            "CMYK TIFF requires an ICC conversion",
        ));
    }
    let color = decoder.color_type();
    if !matches!(
        color,
        image::ColorType::L8
            | image::ColorType::La8
            | image::ColorType::Rgb8
            | image::ColorType::Rgba8
            | image::ColorType::L16
            | image::ColorType::La16
            | image::ColorType::Rgb16
            | image::ColorType::Rgba16
    ) {
        return Err(ImageAssetError::Unsupported("floating point/HDR image"));
    }
    let (width, height) = decoder.dimensions();
    let pixels = u64::from(width)
        .checked_mul(u64::from(height))
        .ok_or(ImageAssetError::Limit("pixel count overflow"))?;
    let sixteen = color.bits_per_pixel() / u16::from(color.channel_count()) == 16;
    let output = pixels
        .checked_mul(if sixteen { 8 } else { 4 })
        .ok_or(ImageAssetError::Limit("output size overflow"))?;
    if width == 0
        || height == 0
        || width > limits.max_dimension
        || height > limits.max_dimension
        || pixels > limits.max_pixels
        || output > limits.max_decoded_bytes
    {
        return Err(ImageAssetError::Limit("decoded dimensions/bytes"));
    }
    // Original decoder buffer, orientation copy, normalized RGBA and sample
    // serialization may overlap. Reserve a conservative explicit working bound.
    let working = decoder
        .total_bytes()
        .checked_mul(2)
        .and_then(|v| output.checked_mul(2).and_then(|o| v.checked_add(o)))
        .ok_or(ImageAssetError::Limit("decode working size overflow"))?;
    if working > limits.max_working_bytes {
        return Err(ImageAssetError::Limit("decode/conversion working bytes"));
    }
    let icc_profile = decoder.icc_profile().map_err(codec_error)?;
    if icc_profile
        .as_ref()
        .is_some_and(|p| p.len() > limits.max_profile_bytes)
    {
        return Err(ImageAssetError::Limit("ICC profile bytes"));
    }
    if !allow_icc && icc_profile.is_some() {
        return Err(ImageAssetError::Unsupported(
            "ICC image conversion requires a color-management module",
        ));
    }
    let orientation = decoder.orientation().map_err(codec_error)?;
    let mut image = DynamicImage::from_decoder(decoder).map_err(codec_error)?;
    image.apply_orientation(orientation);
    let width = image.width();
    let height = image.height();
    let data = if sixteen {
        let rgba = image.into_rgba16().into_raw();
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(
                usize::try_from(output)
                    .map_err(|_| ImageAssetError::Limit("RGBA16 address space"))?,
            )
            .map_err(|_| ImageAssetError::Limit("RGBA16 allocation"))?;
        for sample in rgba {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        bytes
    } else {
        image.into_rgba8().into_raw()
    };
    Ok(DecodedImage {
        width,
        height,
        format: if sixteen {
            PixelFormat::Rgba16
        } else {
            PixelFormat::Rgba8
        },
        data,
        icc_profile: icc_profile.map(Arc::new),
    })
}

fn codec_error(error: image::ImageError) -> ImageAssetError {
    match error {
        image::ImageError::Limits(_) => ImageAssetError::Limit("codec dimensions/allocation"),
        image::ImageError::Unsupported(_) => {
            ImageAssetError::Unsupported("codec color layout/feature")
        }
        other => ImageAssetError::Invalid(other.to_string().chars().take(1024).collect()),
    }
}

// The generic codec API does not expose PNG CICP/gamma/chromaticities. Reject
// declared non-sRGB metadata rather than relabeling decoded samples as sRGB.
fn admit_display_metadata(bytes: &[u8], format: image::ImageFormat) -> Result<(), ImageAssetError> {
    if format == image::ImageFormat::WebP
        && bytes.get(12..16) == Some(b"VP8X")
        && bytes.get(20).is_some_and(|flags| flags & 2 != 0)
    {
        return Err(ImageAssetError::Unsupported("animated WebP"));
    }
    if format != image::ImageFormat::Png {
        return Ok(());
    }
    let mut offset = 8_usize;
    let mut declared_srgb = false;
    let mut non_srgb_gamma = false;
    let mut non_srgb_chromaticity = false;
    while offset < bytes.len() {
        let header = bytes
            .get(offset..offset + 8)
            .ok_or_else(|| ImageAssetError::Invalid("truncated PNG chunk header".into()))?;
        let length = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
        let end = offset
            .checked_add(12)
            .and_then(|v| v.checked_add(length))
            .filter(|v| *v <= bytes.len())
            .ok_or_else(|| ImageAssetError::Invalid("truncated PNG chunk".into()))?;
        let chunk = &bytes[offset + 8..end - 4];
        match &header[4..8] {
            b"IDAT" | b"IEND" => break,
            b"acTL" => return Err(ImageAssetError::Unsupported("animated PNG")),
            b"sRGB" => declared_srgb = true,
            b"cICP" if chunk != [1, 13, 0, 1] => {
                return Err(ImageAssetError::Unsupported("PNG CICP color conversion"));
            }
            b"gAMA" if chunk.len() == 4 => non_srgb_gamma = chunk != 45455_u32.to_be_bytes(),
            b"cHRM" if chunk.len() == 32 => {
                let srgb = [31270_u32, 32900, 64000, 33000, 30000, 60000, 15000, 6000];
                non_srgb_chromaticity = chunk
                    .chunks_exact(4)
                    .zip(srgb)
                    .any(|(actual, expected)| actual != expected.to_be_bytes());
            }
            _ => {}
        }
        offset = end;
    }
    if !declared_srgb && (non_srgb_gamma || non_srgb_chromaticity) {
        return Err(ImageAssetError::Unsupported(
            "PNG declared color conversion",
        ));
    }
    Ok(())
}

// SOF component count precedes the library's lossy CMYK-to-RGB preview. JPEG
// frame headers are marker segments; never search arbitrary payload for bytes.
fn jpeg_components(bytes: &[u8]) -> Result<u8, ImageAssetError> {
    let invalid = || ImageAssetError::Invalid("malformed JPEG frame header".into());
    let mut offset = 2;
    while offset < bytes.len() {
        if bytes[offset] != 0xff {
            return Err(invalid());
        }
        while offset < bytes.len() && bytes[offset] == 0xff {
            offset += 1;
        }
        let marker = *bytes.get(offset).ok_or_else(invalid)?;
        offset += 1;
        if marker == 0xda || marker == 0xd9 {
            return Err(invalid());
        }
        if marker == 0x01 || (0xd0..=0xd7).contains(&marker) {
            continue;
        }
        let len_bytes = bytes.get(offset..offset + 2).ok_or_else(invalid)?;
        let len = usize::from(u16::from_be_bytes([len_bytes[0], len_bytes[1]]));
        if len < 2 || offset.checked_add(len).is_none_or(|end| end > bytes.len()) {
            return Err(invalid());
        }
        if matches!(marker, 0xc0..=0xc3 | 0xc5..=0xc7 | 0xc9..=0xcb | 0xcd..=0xcf) {
            if len < 8 {
                return Err(invalid());
            }
            return Ok(bytes[offset + 7]);
        }
        offset += len;
    }
    Err(invalid())
}
