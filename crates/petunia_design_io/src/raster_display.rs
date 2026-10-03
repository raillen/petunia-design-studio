//! Explicit RGB derivatives for RGB-only interchange. Authored ink resources
//! are read without mutation; native TIFF/PDF paths use the original samples.
use crate::RawRasterImage;
use petunia_design_foundation::PetuniaError;
use petunia_design_raster::{BitDepth, PixelFormat, RasterDisplaySampler, RasterLayer};
pub fn display_raster_layer(
    layer: &RasterLayer,
    cancelled: &dyn Fn() -> bool,
) -> Result<RawRasterImage, PetuniaError> {
    let sixteen = layer.tiles().format.bit_depth() == BitDepth::Sixteen;
    let count = layer.width() as usize * layer.height() as usize * if sixteen { 8 } else { 4 };
    if count > 128 * 1024 * 1024 {
        return Err(PetuniaError::invalid_input(
            "RGB display derivative byte budget",
        ));
    }
    let mut data = Vec::new();
    data.try_reserve_exact(count)
        .map_err(|_| PetuniaError::invalid_input("RGB display derivative allocation"))?;
    let mut sampler = RasterDisplaySampler::new(layer)?;
    for y in 0..layer.height() {
        if cancelled() {
            return Err(PetuniaError::cancelled("RGB display derivative cancelled"));
        }
        for x in 0..layer.width() {
            for value in sampler.pixel(i64::from(x), i64::from(y))? {
                if sixteen {
                    data.extend_from_slice(
                        &((value.clamp(0., 1.) * 65535.).round() as u16).to_le_bytes(),
                    );
                } else {
                    data.push((value.clamp(0., 1.) * 255.).round() as u8);
                }
            }
        }
    }
    Ok(RawRasterImage {
        width: layer.width(),
        height: layer.height(),
        format: if sixteen {
            PixelFormat::Rgba16
        } else {
            PixelFormat::Rgba8
        },
        data,
    })
}
pub fn import_display_raster(
    bytes: &[u8],
    max_bytes: usize,
) -> Result<RawRasterImage, PetuniaError> {
    let native = petunia_design_raster::decode_image(
        bytes,
        petunia_design_raster::ImageDecodeLimits {
            max_encoded_bytes: max_bytes.min(petunia_design_raster::EncodedImage::MAX_BYTES),
            ..Default::default()
        },
    )
    .map_err(|e| PetuniaError::invalid_input(e.to_string()))?;
    if !native.format.is_cmyk() {
        return crate::import_raster(bytes, max_bytes);
    }
    let profile = petunia_design_color::IccProfile::new(
        "Embedded CMYK display profile".into(),
        native
            .icc_profile
            .ok_or_else(|| PetuniaError::invalid_input("CMYK display ICC profile missing"))?,
    )?;
    let layer = RasterLayer::from_cmyka_bytes(
        native.width,
        native.height,
        native.format,
        profile,
        &native.data,
    )?;
    display_raster_layer(&layer, &|| false)
}
