//! Output transform: linear working buffers to device bytes.
//!
//! SDR output quantizes through the display transfer with optional
//! ordered dithering against banding. Thumbnails reuse the pipeline
//! with overlays off; headless targets never touch Qt.

use crate::compositor::Pixel;

/// Ordered 4×4 Bayer thresholds in `0..1`.
const BAYER_4X4: [[f32; 4]; 4] = [
    [0.0 / 16.0, 8.0 / 16.0, 2.0 / 16.0, 10.0 / 16.0],
    [12.0 / 16.0, 4.0 / 16.0, 14.0 / 16.0, 6.0 / 16.0],
    [3.0 / 16.0, 11.0 / 16.0, 1.0 / 16.0, 9.0 / 16.0],
    [15.0 / 16.0, 7.0 / 16.0, 13.0 / 16.0, 5.0 / 16.0],
];

/// Linear working value to sRGB-encoded byte with optional ordered
/// dither. Dither adds low-amplitude controlled noise before
/// quantization; it never alters the document.
#[must_use]
pub fn linear_to_srgb_byte(value: f32, dither: f32) -> u8 {
    let biased = (value + dither).clamp(0.0, 1.0);
    let encoded = if biased <= 0.0031308 {
        12.92 * biased
    } else {
        1.055 * biased.powf(1.0 / 2.4) - 0.055
    };
    (encoded.clamp(0.0, 1.0) * 255.0).round().clamp(0.0, 255.0) as u8
}

/// Convert one premultiplied linear frame to RGBA8. Straight alpha
/// returns on the way out; fully transparent pixels canonicalize to
/// zero color.
#[must_use]
pub fn frame_to_rgba8(pixels: &[Pixel], width: u32, dither: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(pixels.len() * 4);
    for (index, pixel) in pixels.iter().enumerate() {
        let (x, y) = (index as u32 % width, index as u32 / width);
        let amount = if dither {
            (BAYER_4X4[(y % 4) as usize][(x % 4) as usize] - 0.5) / 255.0
        } else {
            0.0
        };
        let (r, g, b) = if pixel.a <= 0.0 {
            (0.0, 0.0, 0.0)
        } else {
            (pixel.r / pixel.a, pixel.g / pixel.a, pixel.b / pixel.a)
        };
        out.push(linear_to_srgb_byte(r, amount));
        out.push(linear_to_srgb_byte(g, amount));
        out.push(linear_to_srgb_byte(b, amount));
        out.push((pixel.a.clamp(0.0, 1.0) * 255.0).round() as u8);
    }
    out
}

/// Encode RGBA8 bytes as PNG through the image crate.
#[must_use]
pub fn encode_png_rgba8(width: u32, height: u32, rgba8: &[u8]) -> Option<Vec<u8>> {
    use image::codecs::png::PngEncoder;
    use image::{ExtendedColorType, ImageEncoder};
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes)
        .write_image(rgba8, width, height, ExtendedColorType::Rgba8)
        .ok()?;
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_endpoints_and_midpoint() {
        assert_eq!(linear_to_srgb_byte(0.0, 0.0), 0);
        assert_eq!(linear_to_srgb_byte(1.0, 0.0), 255);
        // Linear ~0.214 encodes to sRGB 50% (≈128).
        let mid = linear_to_srgb_byte(0.214, 0.0);
        assert!((126..=130).contains(&mid), "{mid}");
    }

    #[test]
    fn transparent_pixels_canonicalize() {
        let out = frame_to_rgba8(&[Pixel::CLEAR], 1, false);
        assert_eq!(out, vec![0, 0, 0, 0]);
        let half = frame_to_rgba8(
            &[Pixel {
                r: 0.5,
                g: 0.0,
                b: 0.0,
                a: 0.5,
            }],
            1,
            false,
        );
        // Straightened red at full encoded intensity.
        assert_eq!(half[0], 255);
        assert_eq!(half[3], 128);
    }

    #[test]
    fn png_round_trip_preserves_bytes() {
        let rgba8 = vec![255u8, 0, 0, 255];
        let png = encode_png_rgba8(1, 1, &rgba8).expect("encodes");
        let decoded = image::load_from_memory(&png).expect("decodes").into_rgba8();
        assert_eq!(decoded.into_raw(), rgba8);
    }
}
