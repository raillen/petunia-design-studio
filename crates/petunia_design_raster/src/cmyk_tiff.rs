//! Checked native TIFF admission. The generic image facade converts CMYK to
//! RGB, so native process ink is read with the TIFF codec directly.
use crate::{DecodedImage, ImageAssetError, ImageDecodeLimits, PixelFormat};
use petunia_design_color::IccProfile;
use std::{io::Cursor, sync::Arc};
use tiff::{
    decoder::{Decoder, DecodingResult, Limits},
    tags::Tag,
    ColorType,
};
fn codec(error: tiff::TiffError) -> ImageAssetError {
    if matches!(error, tiff::TiffError::LimitsExceeded) {
        ImageAssetError::Limit("TIFF codec allocation")
    } else {
        ImageAssetError::Invalid(error.to_string().chars().take(1024).collect())
    }
}
pub(crate) fn decode(
    bytes: &[u8],
    limits: ImageDecodeLimits,
) -> Result<Option<DecodedImage>, ImageAssetError> {
    let mut codec_limits = Limits::default();
    codec_limits.decoding_buffer_size =
        usize::try_from(limits.max_decoded_bytes).unwrap_or(usize::MAX);
    codec_limits.intermediate_buffer_size =
        usize::try_from(limits.max_working_bytes.min(128 * 1024 * 1024)).unwrap_or(usize::MAX);
    codec_limits.ifd_value_size = limits.max_profile_bytes.min(4 * 1024 * 1024);
    let mut decoder = Decoder::new(Cursor::new(bytes))
        .map_err(codec)?
        .with_limits(codec_limits);
    let color = decoder.colortype().map_err(codec)?;
    let (bits, alpha) = match color {
        ColorType::CMYK(bits) => (bits, false),
        ColorType::CMYKA(bits) => (bits, true),
        _ => return Ok(None),
    };
    if !matches!(bits, 8 | 16) {
        return Err(ImageAssetError::Unsupported(
            "native CMYK TIFF integer depth",
        ));
    }
    if decoder.more_images() {
        return Err(ImageAssetError::Unsupported("multipage CMYK TIFF"));
    }
    if decoder
        .find_tag_unsigned::<u16>(Tag::PlanarConfiguration)
        .map_err(codec)?
        .unwrap_or(1)
        != 1
    {
        return Err(ImageAssetError::Unsupported("planar CMYK TIFF"));
    }
    // TIFF 6.0 defaults: InkSet=CMYK (332=1), NumberOfInks (334)=4.
    if decoder
        .find_tag_unsigned::<u16>(Tag::Unknown(332))
        .map_err(codec)?
        .unwrap_or(1)
        != 1
        || decoder
            .find_tag_unsigned::<u16>(Tag::Unknown(334))
            .map_err(codec)?
            .unwrap_or(4)
            != 4
    {
        return Err(ImageAssetError::Unsupported("non-process TIFF ink set"));
    }
    let extras = decoder
        .find_tag_unsigned_vec::<u16>(Tag::ExtraSamples)
        .map_err(codec)?
        .unwrap_or_default();
    if (alpha && extras != [2]) || (!alpha && !extras.is_empty()) {
        return Err(ImageAssetError::Unsupported(
            "CMYK TIFF requires explicit unassociated alpha",
        ));
    }
    let (width, height) = decoder.dimensions().map_err(codec)?;
    let pixels = u64::from(width) * u64::from(height);
    let bpc = usize::from(bits / 8);
    let output_bytes = pixels
        .checked_mul(5 * bpc as u64)
        .ok_or(ImageAssetError::Limit("CMYK TIFF size overflow"))?;
    let source_channels = if alpha { 5 } else { 4 };
    let working = output_bytes
        .checked_mul(2)
        .and_then(|n| n.checked_add(pixels * source_channels as u64 * bpc as u64 * 2))
        .ok_or(ImageAssetError::Limit("CMYK TIFF working size overflow"))?;
    if width == 0
        || height == 0
        || width > limits.max_dimension
        || height > limits.max_dimension
        || pixels > limits.max_pixels
        || output_bytes > limits.max_decoded_bytes
        || working > limits.max_working_bytes
    {
        return Err(ImageAssetError::Limit("CMYK TIFF dimensions/working bytes"));
    }
    let profile = decoder
        .find_tag(Tag::IccProfile)
        .map_err(codec)?
        .ok_or(ImageAssetError::Unsupported(
            "CMYK TIFF requires an embedded ICC profile or explicit assignment",
        ))?
        .into_u8_vec()
        .map_err(codec)?;
    if profile.len() > limits.max_profile_bytes {
        return Err(ImageAssetError::Limit("CMYK TIFF ICC profile bytes"));
    }
    let profile = Arc::new(profile);
    let checked = IccProfile::new("Embedded CMYK TIFF profile".into(), profile.clone())
        .map_err(|e| ImageAssetError::Invalid(e.to_string()))?;
    if !checked.is_press_profile() {
        return Err(ImageAssetError::Unsupported(
            "CMYK TIFF requires a CMYK output profile",
        ));
    }
    let orientation = decoder
        .find_tag_unsigned::<u16>(Tag::Orientation)
        .map_err(codec)?
        .unwrap_or(1);
    if !(1..=8).contains(&orientation) {
        return Err(ImageAssetError::Invalid("invalid TIFF orientation".into()));
    }
    let raw = match (bits, decoder.read_image().map_err(codec)?) {
        (8, DecodingResult::U8(samples)) => samples,
        (16, DecodingResult::U16(samples)) => {
            let mut raw = Vec::new();
            raw.try_reserve_exact(samples.len() * 2)
                .map_err(|_| ImageAssetError::Limit("CMYK TIFF sample allocation"))?;
            for sample in samples {
                raw.extend_from_slice(&sample.to_le_bytes());
            }
            raw
        }
        _ => {
            return Err(ImageAssetError::Unsupported(
                "CMYK TIFF sample representation",
            ))
        }
    };
    if raw.len() != pixels as usize * source_channels * bpc {
        return Err(ImageAssetError::Invalid(
            "CMYK TIFF decoded sample layout".into(),
        ));
    }
    let (out_width, out_height) = if orientation >= 5 {
        (height, width)
    } else {
        (width, height)
    };
    let mut data = Vec::new();
    data.try_reserve_exact(output_bytes as usize)
        .map_err(|_| ImageAssetError::Limit("CMYK TIFF output allocation"))?;
    for y in 0..out_height {
        for x in 0..out_width {
            let (sx, sy) = match orientation {
                1 => (x, y),
                2 => (width - 1 - x, y),
                3 => (width - 1 - x, height - 1 - y),
                4 => (x, height - 1 - y),
                5 => (y, x),
                6 => (y, height - 1 - x),
                7 => (width - 1 - y, height - 1 - x),
                8 => (width - 1 - y, x),
                _ => unreachable!("checked TIFF orientation"),
            };
            let offset = (sy as usize * width as usize + sx as usize) * source_channels * bpc;
            data.extend_from_slice(&raw[offset..offset + source_channels * bpc]);
            if !alpha {
                data.extend(std::iter::repeat_n(255, bpc));
            }
        }
    }
    Ok(Some(DecodedImage {
        width: out_width,
        height: out_height,
        format: if bits == 8 {
            PixelFormat::Cmyka8
        } else {
            PixelFormat::Cmyka16
        },
        data,
        icc_profile: Some(profile),
    }))
}
