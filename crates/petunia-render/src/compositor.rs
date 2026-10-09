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
        r: blended[0] * src.a + dst.r * (1.0 - src.a),
        g: blended[1] * src.a + dst.g * (1.0 - src.a),
        b: blended[2] * src.a + dst.b * (1.0 - src.a),
        a: out_a,
    };
    if out.is_finite() {
        out
    } else {
        dst
    }
}

/// Blend two straight-alpha triplets under one shared contract.
/// Exposed so property tests and future backends verify the same
/// formulas the compositor uses.
pub fn apply_blend(source: [f32; 3], backdrop: [f32; 3], mode: BlendMode) -> [f32; 3] {
    let mut out = [0.0; 3];
    for channel in 0..3 {
        out[channel] = blend_channel(
            source[channel],
            backdrop[channel],
            mode,
            source,
            backdrop,
            channel,
        );
    }
    out
}

fn blend_channel(
    s: f32,
    d: f32,
    mode: BlendMode,
    source: [f32; 3],
    backdrop: [f32; 3],
    channel: usize,
) -> f32 {
    match mode {
        BlendMode::Normal => s,
        BlendMode::Multiply => s * d,
        BlendMode::Screen => s + d - s * d,
        BlendMode::Overlay => {
            if d <= 0.5 {
                2.0 * s * d
            } else {
                1.0 - 2.0 * (1.0 - s) * (1.0 - d)
            }
        }
        BlendMode::Darken => s.min(d),
        BlendMode::Lighten => s.max(d),
        BlendMode::ColorDodge => {
            if d == 0.0 {
                0.0
            } else if s >= 1.0 {
                1.0
            } else {
                (d / (1.0 - s)).min(1.0)
            }
        }
        BlendMode::ColorBurn => {
            if d >= 1.0 {
                1.0
            } else if s <= 0.0 {
                0.0
            } else {
                1.0 - ((1.0 - d) / s).min(1.0)
            }
        }
        BlendMode::HardLight => {
            if s <= 0.5 {
                2.0 * s * d
            } else {
                1.0 - 2.0 * (1.0 - s) * (1.0 - d)
            }
        }
        BlendMode::SoftLight => {
            if s <= 0.5 {
                d - (1.0 - 2.0 * s) * d * (1.0 - d)
            } else {
                let root = if d <= 0.0 { 0.0 } else { d.sqrt() };
                d + (2.0 * s - 1.0) * (root - d)
            }
        }
        BlendMode::Difference => (s - d).abs(),
        BlendMode::Exclusion => s + d - 2.0 * s * d,
        // Non-separable modes combine whole triplets; the per-channel
        // loop below only selects which triplet each lane carries.
        BlendMode::Hue | BlendMode::Saturation | BlendMode::Color | BlendMode::Luminosity => {
            non_separable(source, backdrop, mode)[channel]
        }
    }
}

/// Whole-triplet result for hue, saturation, color and luminosity.
fn non_separable(source: [f32; 3], backdrop: [f32; 3], mode: BlendMode) -> [f32; 3] {
    match mode {
        BlendMode::Hue => set_lum(set_sat(source, saturation(backdrop)), luminosity(backdrop)),
        BlendMode::Saturation => {
            set_lum(set_sat(backdrop, saturation(source)), luminosity(backdrop))
        }
        BlendMode::Color => set_lum(source, luminosity(backdrop)),
        BlendMode::Luminosity => set_lum(backdrop, luminosity(source)),
        _ => unreachable!("separable modes never reach here"),
    }
}

fn luminosity(rgb: [f32; 3]) -> f32 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}

fn saturation(rgb: [f32; 3]) -> f32 {
    rgb[0].max(rgb[1]).max(rgb[2]) - rgb[0].min(rgb[1]).min(rgb[2])
}

fn clip_color(mut color: [f32; 3]) -> [f32; 3] {
    let lum = luminosity(color);
    let min = color[0].min(color[1]).min(color[2]);
    let max = color[0].max(color[1]).max(color[2]);
    if min < 0.0 {
        for channel in &mut color {
            *channel = lum + ((*channel - lum) * lum) / (lum - min);
        }
    }
    if max > 1.0 {
        for channel in &mut color {
            *channel = lum + ((*channel - lum) * (1.0 - lum)) / (max - lum);
        }
    }
    color
}

fn set_lum(color: [f32; 3], lum: f32) -> [f32; 3] {
    let delta = lum - luminosity(color);
    clip_color([color[0] + delta, color[1] + delta, color[2] + delta])
}

fn set_sat(color: [f32; 3], saturation: f32) -> [f32; 3] {
    let max = color[0].max(color[1]).max(color[2]);
    let min = color[0].min(color[1]).min(color[2]);
    if max <= min {
        return [0.0, 0.0, 0.0];
    }
    let mut out = [0.0; 3];
    for (channel, value) in color.iter().enumerate() {
        out[channel] = (value - min) * saturation / (max - min);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

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
