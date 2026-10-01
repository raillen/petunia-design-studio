//! Bounded ICC RGB input conversion. The caller owns an expendable decoded
//! buffer; original encoded pixels/profile bytes remain canonical and unchanged.
use moxcms::{
    ColorProfile, DataColorSpace, Layout, ParsingOptions, RenderingIntent, TransformOptions,
};
use petunia_design_foundation::PetuniaError;
fn invalid(reason: impl Into<String>) -> PetuniaError {
    PetuniaError::invalid_input(reason)
}
fn options() -> TransformOptions {
    TransformOptions {
        rendering_intent: RenderingIntent::RelativeColorimetric,
        ..Default::default()
    }
}
pub fn convert_rgba_to_srgb(
    data: &mut [u8],
    sixteen: bool,
    width: u32,
    profile: &[u8],
    cancelled: &dyn Fn() -> bool,
) -> Result<(), PetuniaError> {
    if profile.len() > 4 * 1024 * 1024 {
        return Err(invalid("ICC profile exceeds 4 MiB"));
    }
    if width == 0 || width > 32768 {
        return Err(invalid("ICC scanline dimensions"));
    }
    let profile = ColorProfile::new_from_slice_with_options(
        profile,
        ParsingOptions {
            max_profile_size: 4 * 1024 * 1024,
            max_allowed_clut_size: 1024 * 1024,
            max_allowed_trc_size: 65536,
        },
    )
    .map_err(|e| invalid(format!("invalid or over-budget ICC profile: {e}")))?;
    if profile.color_space != DataColorSpace::Rgb {
        return Err(PetuniaError::capability_unavailable(
            "MVP ICC input conversion requires an RGB profile; Gray/CMYK are V1 capabilities",
        ));
    }
    let target = ColorProfile::new_srgb();
    let row_samples = width as usize * 4;
    let row_bytes = row_samples * if sixteen { 2 } else { 1 };
    if data.len() % row_bytes != 0 {
        return Err(invalid("ICC pixel buffer layout"));
    }
    if cancelled() {
        return Err(invalid("ICC conversion cancelled"));
    }
    if sixteen {
        let transform = profile
            .create_transform_16bit(Layout::Rgba, &target, Layout::Rgba, options())
            .map_err(|e| invalid(format!("ICC RGB transform unavailable: {e}")))?;
        let mut source = vec![0u16; row_samples];
        let mut output = vec![0u16; row_samples];
        for row in data.chunks_exact_mut(row_bytes) {
            if cancelled() {
                return Err(invalid("ICC conversion cancelled"));
            }
            for (sample, bytes) in source.iter_mut().zip(row.chunks_exact(2)) {
                *sample = u16::from_le_bytes([bytes[0], bytes[1]]);
            }
            transform
                .transform(&source, &mut output)
                .map_err(|e| invalid(format!("ICC RGB conversion: {e}")))?;
            // Alpha is coverage, not an ink channel or a profile component.
            for pixel in 0..width as usize {
                output[pixel * 4 + 3] = source[pixel * 4 + 3];
            }
            for (bytes, sample) in row.chunks_exact_mut(2).zip(output.iter()) {
                bytes.copy_from_slice(&sample.to_le_bytes());
            }
        }
    } else {
        let transform = profile
            .create_transform_8bit(Layout::Rgba, &target, Layout::Rgba, options())
            .map_err(|e| invalid(format!("ICC RGB transform unavailable: {e}")))?;
        let mut output = vec![0u8; row_bytes];
        for row in data.chunks_exact_mut(row_bytes) {
            if cancelled() {
                return Err(invalid("ICC conversion cancelled"));
            }
            transform
                .transform(row, &mut output)
                .map_err(|e| invalid(format!("ICC RGB conversion: {e}")))?;
            for pixel in 0..width as usize {
                output[pixel * 4 + 3] = row[pixel * 4 + 3];
            }
            row.copy_from_slice(&output);
        }
    }
    Ok(())
}
