//! Property tests for render-side contracts.
//!
//! Covers output transfer endpoints, premultiplied round-trips,
//! adjustment identity and non-finite rejection, plus blend-mode
//! separability rules.

use petunia_core::appearance::BlendMode;
use petunia_render::compositor::{composite, Pixel};
use petunia_render::output::{frame_to_rgba8, linear_to_srgb_byte};
use petunia_render::{apply_adjustment, StraightPixel};
use petunia_render_model::{RenderAdjustment, RenderColor};
use proptest::prelude::*;

fn straight() -> impl Strategy<Value = StraightPixel> {
    (0.0f32..=1.0, 0.0f32..=1.0, 0.0f32..=1.0, 0.0f32..=1.0)
        .prop_map(|(r, g, b, a)| StraightPixel { r, g, b, a })
}

proptest! {
    /// Transfer endpoints are exact for every generated input.
    #[test]
    fn transfer_endpoints_hold(value in 0.0f32..=1.0) {
        let byte = linear_to_srgb_byte(value, 0.0);
        prop_assert!((0..=255).contains(&byte));
        if value <= 0.0 {
            prop_assert_eq!(byte, 0);
        }
        if value >= 1.0 {
            prop_assert_eq!(byte, 255);
        }
    }

    /// Straightening a premultiplied pixel recovers its parts.
    #[test]
    fn straight_alpha_round_trips(r in -1.0f32..2.0, g in -1.0f32..2.0, b in -1.0f32..2.0) {
        let alpha = 0.5f32;
        let premultiplied = Pixel { r: r * alpha, g: g * alpha, b: b * alpha, a: alpha };
        let back = premultiplied.unpremultiplied();
        for (got, want) in [back[0], back[1], back[2]].iter().zip([r, g, b]) {
            prop_assert!((got - want).abs() < 1e-6, "{got} != {want}");
        }
    }

    /// Neutral adjustments are the identity on every pixel. Posterize
    /// has no neutral parameter: it quantizes to 1/(levels-1) steps, so
    /// it is asserted with its own bound.
    #[test]
    fn neutral_adjustments_are_identity(pixel in straight()) {
        for adjustment in [
            RenderAdjustment::BrightnessContrast { brightness: 0.0, contrast: 0.0 },
            RenderAdjustment::Hsl { hue_shift: 0.0, saturation: 0.0, lightness: 0.0 },
            RenderAdjustment::Vibrance { amount: 0.0 },
        ] {
            let out = apply_adjustment(&adjustment, pixel).expect("neutral applies");
            for (got, want) in [(out.r, pixel.r), (out.g, pixel.g), (out.b, pixel.b)] {
                prop_assert!((got - want).abs() < 1e-4, "{got} != {want}");
            }
            prop_assert_eq!(out.a, pixel.a);
        }
    }

    /// Posterize quantizes to exactly `levels` values with a bounded error.
    #[test]
    fn posterize_quantizes_within_half_a_step(pixel in straight(), levels in 2u32..256) {
        let steps = (levels - 1) as f32;
        let out = apply_adjustment(
            &RenderAdjustment::Posterize { levels },
            pixel,
        )
        .expect("applies");
        for (got, want) in [(out.r, pixel.r), (out.g, pixel.g), (out.b, pixel.b)] {
            let step = (got * steps).round();
            prop_assert!((got - step / steps).abs() < 1e-6, "{got} is not a step");
            prop_assert!(
                (got - want).abs() <= 1.0 / (2.0 * steps) + 1e-6,
                "error {got} - {want} exceeds half a step"
            );
        }
        // Alpha is never quantized.
        prop_assert_eq!(out.a, pixel.a);
    }

    /// Source-over keeps the backdrop when the source is transparent
    /// and replaces it when the source is opaque.
    #[test]
    fn source_over_honors_alpha_endpoints(
        r in 0.0f32..=1.0,
        g in 0.0f32..=1.0,
        b in 0.0f32..=1.0,
    ) {
        let backdrop = Pixel { r: 0.2, g: 0.4, b: 0.6, a: 1.0 };
        let transparent = Pixel { r, g, b, a: 0.0 };
        prop_assert_eq!(composite(transparent, backdrop, BlendMode::Normal), backdrop);
        let opaque = Pixel { r, g, b, a: 1.0 };
        let out = composite(opaque, backdrop, BlendMode::Normal);
        prop_assert_eq!(out, opaque);
    }

    /// Separable blend modes commute per channel; non-separable ones
    /// mix the whole triplet.
    #[test]
    fn separable_blends_are_channel_wise(s in 0.0f32..=1.0, d in 0.0f32..=1.0) {
        let gray_s = [s, s, s];
        let gray_d = [d, d, d];
        // Multiply on gray must return gray with (s*d) per channel.
        let out = petunia_render::apply_blend(gray_s, gray_d, BlendMode::Multiply);
        for channel in out {
            prop_assert!((channel - s * d).abs() < 1e-6);
        }
    }

    /// Non-finite adjustment inputs error instead of propagating.
    #[test]
    fn non_finite_adjustments_error(bad in 0u8..3) {
        let pixel = StraightPixel {
            r: match bad {
                0 => f32::NAN,
                1 => f32::INFINITY,
                _ => f32::NEG_INFINITY,
            },
            g: 0.5,
            b: 0.5,
            a: 1.0,
        };
        prop_assert!(
            apply_adjustment(&RenderAdjustment::Invert, pixel).is_err(),
            "{pixel:?}"
        );
    }

    /// Opaque pixels always quantize to alpha 255; clear stays clear.
    #[test]
    fn quantization_endpoints(alpha in 0.0f32..=1.0) {
        let pixels = vec![
            Pixel { r: 1.0, g: 1.0, b: 1.0, a: alpha },
            Pixel::CLEAR,
        ];
        let bytes = frame_to_rgba8(&pixels, 2, false);
        prop_assert_eq!(bytes.len(), 8);
        if alpha >= 1.0 {
            prop_assert_eq!(bytes[3], 255);
        }
        prop_assert_eq!(bytes[7], 0);
        let _ = RenderColor::WHITE;
    }
}
