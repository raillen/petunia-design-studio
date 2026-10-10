//! Premultiplied-linear compositor with one blend contract.
//!
//! The working format is linear premultiplied RGBA f32. Blend modes
//! evaluate on unpremultiplied components and rejoin coverage
//! through Porter-Duff; alpha zero never divides by zero, and
//! NaN/Inf never propagates into the frame.

use petunia_core::appearance::BlendMode;

/// One premultiplied linear pixel.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pixel {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Pixel {
    /// Transparent black: the neutral element of source-over.
    pub const CLEAR: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// Straighten components, treating fully transparent pixels as
    /// canonical zero instead of dividing by zero.
    #[must_use]
    pub fn unpremultiplied(self) -> [f32; 3] {
        if self.a <= 0.0 {
            [0.0, 0.0, 0.0]
        } else {
            [self.r / self.a, self.g / self.a, self.b / self.a]
        }
    }

    /// True when every lane is finite.
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.r.is_finite() && self.g.is_finite() && self.b.is_finite() && self.a.is_finite()
    }
}

/// Source-over with a blend mode: `Cout = blend + Cb × (1 − αs)`,
/// applied per the single shared contract. Non-finite inputs fall
/// back to the backdrop instead of poisoning the frame.
#[must_use]
pub fn composite(src: Pixel, dst: Pixel, mode: BlendMode) -> Pixel {
    if !src.is_finite() || !dst.is_finite() {
        return dst;
    }
    if src.a <= 0.0 {
        return dst;
    }
    let source = src.unpremultiplied();
    let backdrop = dst.unpremultiplied();
    let blended = apply_blend(source, backdrop, mode);
    let out_a = (src.a + dst.a * (1.0 - src.a)).clamp(0.0, 1.0);
    let out = Pixel {
        r: source[0] * src.a * (1.0 - dst.a) + blended[0] * src.a * dst.a + dst.r * (1.0 - src.a),
        g: source[1] * src.a * (1.0 - dst.a) + blended[1] * src.a * dst.a + dst.g * (1.0 - src.a),
        b: source[2] * src.a * (1.0 - dst.a) + blended[2] * src.a * dst.a + dst.b * (1.0 - src.a),
        a: out_a,
    };
    if out.is_finite() {
        out
    } else {
        dst
    }
}

pub use petunia_render_model::apply_blend;

#[cfg(test)]
mod tests {
    use super::*;

    fn luminosity(rgb: [f32; 3]) -> f32 {
        0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
    }

    fn pixel(r: f32, g: f32, b: f32, a: f32) -> Pixel {
        Pixel { r, g, b, a }
    }

    #[test]
    fn source_over_honors_alpha_endpoints() {
        let red = pixel(1.0, 0.0, 0.0, 1.0);
        let blue = pixel(0.0, 0.0, 1.0, 1.0);
        // Opaque source replaces the destination under Normal.
        assert_eq!(composite(red, blue, BlendMode::Normal), red);
        // Transparent source leaves the destination alone.
        assert_eq!(composite(Pixel::CLEAR, blue, BlendMode::Multiply), blue);
        // Transparent black stays neutral.
        assert_eq!(composite(Pixel::CLEAR, blue, BlendMode::Normal), blue);
    }

    #[test]
    fn semitransparent_source_blends_linearly() {
        let half_red = pixel(0.5, 0.0, 0.0, 0.5);
        let blue = pixel(0.0, 0.0, 1.0, 1.0);
        let out = composite(half_red, blue, BlendMode::Normal);
        assert!((out.a - 1.0).abs() < 1e-6);
        assert!((out.r - 0.5).abs() < 1e-6);
        assert!((out.b - 0.5).abs() < 1e-6);
    }

    #[test]
    fn blend_modes_match_reference_values() {
        let white = [1.0, 1.0, 1.0];
        let half = [0.5, 0.5, 0.5];
        assert_eq!(
            apply_blend(white, half, BlendMode::Multiply),
            [0.5, 0.5, 0.5]
        );
        assert_eq!(apply_blend(white, half, BlendMode::Screen), [1.0, 1.0, 1.0]);
        let difference = apply_blend([0.2, 0.4, 0.6], [0.8, 0.6, 0.4], BlendMode::Difference);
        for (got, want) in difference.iter().zip([0.6, 0.2, 0.2]) {
            assert!((got - want).abs() < 1e-6, "{difference:?}");
        }
        // Overlay at mid backdrop behaves like 2*s*d per channel.
        let overlay = apply_blend([0.5, 0.5, 0.5], [0.25, 0.25, 0.25], BlendMode::Overlay);
        assert!((overlay[0] - 0.25).abs() < 1e-6);
    }

    #[test]
    fn non_separable_modes_keep_whole_triplets() {
        // Luminosity carries the source luminance onto the backdrop hue.
        let out = apply_blend([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], BlendMode::Luminosity);
        assert!((luminosity(out) - luminosity([1.0, 0.0, 0.0])).abs() < 1e-5);
        assert!(out[2] > out[0]);
        // Hue carries the source hue onto backdrop saturation/luminance.
        let hue = apply_blend([1.0, 0.0, 0.0], [0.0, 0.5, 0.0], BlendMode::Hue);
        assert!(hue[0] >= hue[1] && hue[0] >= hue[2]);
    }

    #[test]
    fn non_finite_inputs_never_poison_frames() {
        let nan = pixel(f32::NAN, 0.0, 0.0, 0.5);
        let blue = pixel(0.0, 0.0, 1.0, 1.0);
        assert_eq!(composite(nan, blue, BlendMode::Normal), blue);
        let inf = Pixel {
            r: f32::INFINITY,
            ..blue
        };
        assert_eq!(composite(blue, inf, BlendMode::Screen), inf);
    }
}
