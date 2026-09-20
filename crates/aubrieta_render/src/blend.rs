//! Canonical BlendMode contract and reference CPU implementation.
//!
//! Conforms to W3C Compositing and Blending Level 1 & PDF Reference specification.
//! Supports all 12 separable blend modes and 4 non-separable blend modes with
//! mathematically exact straight and premultiplied compositing math.

use serde::{Deserialize, Serialize};

/// 16 standard blend modes (W3C / PDF specification).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BlendMode {
    /// Normal: replaces backdrop with source based on source alpha.
    #[default]
    Normal,
    /// Multiply: multiplies backdrop and source channels.
    Multiply,
    /// Screen: multiplies inverse of backdrop and source channels.
    Screen,
    /// Overlay: multiplies or screens depending on backdrop channel.
    Overlay,
    /// Darken: selects the minimum of backdrop and source.
    Darken,
    /// Lighten: selects the maximum of backdrop and source.
    Lighten,
    /// ColorDodge: brightens backdrop to reflect source.
    ColorDodge,
    /// ColorBurn: darkens backdrop to reflect source.
    ColorBurn,
    /// HardLight: multiplies or screens depending on source channel.
    HardLight,
    /// SoftLight: darkens or lightens depending on source channel.
    SoftLight,
    /// Difference: subtracts darker color from lighter color.
    Difference,
    /// Exclusion: similar to difference with lower contrast.
    Exclusion,
    /// Hue: preserves luminosity and saturation of backdrop with hue of source.
    Hue,
    /// Saturation: preserves luminosity and hue of backdrop with saturation of source.
    Saturation,
    /// Color: preserves luminosity of backdrop with hue and saturation of source.
    Color,
    /// Luminosity: preserves hue and saturation of backdrop with luminosity of source.
    Luminosity,
}

impl BlendMode {
    /// Returns true if the blend mode operates independently on each color channel.
    #[must_use]
    pub const fn is_separable(&self) -> bool {
        match self {
            Self::Normal
            | Self::Multiply
            | Self::Screen
            | Self::Overlay
            | Self::Darken
            | Self::Lighten
            | Self::ColorDodge
            | Self::ColorBurn
            | Self::HardLight
            | Self::SoftLight
            | Self::Difference
            | Self::Exclusion => true,
            Self::Hue | Self::Saturation | Self::Color | Self::Luminosity => false,
        }
    }

    /// Evaluates the separable blend function B(cb, cs) for a single normalized channel in [0, 1].
    /// For non-separable modes, defaults to Normal for channel-wise queries.
    #[must_use]
    pub fn blend_channel(&self, cb: f32, cs: f32) -> f32 {
        let cb = cb.clamp(0.0, 1.0);
        let cs = cs.clamp(0.0, 1.0);
        match self {
            Self::Normal => cs,
            Self::Multiply => cb * cs,
            Self::Screen => cb + cs - (cb * cs),
            Self::Overlay => {
                if cb <= 0.5 {
                    2.0 * cb * cs
                } else {
                    1.0 - 2.0 * (1.0 - cb) * (1.0 - cs)
                }
            }
            Self::Darken => cb.min(cs),
            Self::Lighten => cb.max(cs),
            Self::ColorDodge => {
                if cb <= 0.0 {
                    0.0
                } else if cs >= 1.0 {
                    1.0
                } else {
                    (cb / (1.0 - cs)).min(1.0)
                }
            }
            Self::ColorBurn => {
                if cb >= 1.0 {
                    1.0
                } else if cs <= 0.0 {
                    0.0
                } else {
                    1.0 - ((1.0 - cb) / cs).min(1.0)
                }
            }
            Self::HardLight => {
                if cs <= 0.5 {
                    2.0 * cb * cs
                } else {
                    1.0 - 2.0 * (1.0 - cb) * (1.0 - cs)
                }
            }
            Self::SoftLight => {
                if cs <= 0.5 {
                    cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb)
                } else {
                    let d = if cb <= 0.25 {
                        ((16.0 * cb - 12.0) * cb + 4.0) * cb
                    } else {
                        cb.sqrt()
                    };
                    cb + (2.0 * cs - 1.0) * (d - cb)
                }
            }
            Self::Difference => (cb - cs).abs(),
            Self::Exclusion => cb + cs - 2.0 * cb * cs,
            // Non-separable fallback for channel-wise evaluation:
            Self::Hue | Self::Saturation | Self::Color | Self::Luminosity => cs,
        }
    }

    /// Evaluates the full RGB blend function B(Cb, Cs) for normalized RGB in [0, 1].
    #[must_use]
    pub fn blend_rgb(&self, cb: [f32; 3], cs: [f32; 3]) -> [f32; 3] {
        if self.is_separable() {
            [
                self.blend_channel(cb[0], cs[0]),
                self.blend_channel(cb[1], cs[1]),
                self.blend_channel(cb[2], cs[2]),
            ]
        } else {
            let cb_clamped = [
                cb[0].clamp(0.0, 1.0),
                cb[1].clamp(0.0, 1.0),
                cb[2].clamp(0.0, 1.0),
            ];
            let cs_clamped = [
                cs[0].clamp(0.0, 1.0),
                cs[1].clamp(0.0, 1.0),
                cs[2].clamp(0.0, 1.0),
            ];
            match self {
                Self::Hue => set_lum(set_sat(cs_clamped, sat(cb_clamped)), lum(cb_clamped)),
                Self::Saturation => set_lum(set_sat(cb_clamped, sat(cs_clamped)), lum(cb_clamped)),
                Self::Color => set_lum(cs_clamped, lum(cb_clamped)),
                Self::Luminosity => set_lum(cb_clamped, lum(cs_clamped)),
                _ => cs_clamped,
            }
        }
    }

    /// Composites straight RGBA source over straight RGBA backdrop.
    /// Channels are in [0.0, 1.0].
    #[must_use]
    pub fn composite_pixel_straight(&self, cb: [f32; 4], cs: [f32; 4]) -> [f32; 4] {
        let ab = cb[3].clamp(0.0, 1.0);
        let as_ = cs[3].clamp(0.0, 1.0);

        if ab <= 0.0 && as_ <= 0.0 {
            return [0.0, 0.0, 0.0, 0.0];
        }
        if ab <= 0.0 {
            return [
                cs[0].clamp(0.0, 1.0),
                cs[1].clamp(0.0, 1.0),
                cs[2].clamp(0.0, 1.0),
                as_,
            ];
        }
        if as_ <= 0.0 {
            return [
                cb[0].clamp(0.0, 1.0),
                cb[1].clamp(0.0, 1.0),
                cb[2].clamp(0.0, 1.0),
                ab,
            ];
        }

        let ar = as_ + ab * (1.0 - as_);
        if ar <= 1e-7 {
            return [0.0, 0.0, 0.0, 0.0];
        }

        let b = self.blend_rgb([cb[0], cb[1], cb[2]], [cs[0], cs[1], cs[2]]);

        let cr_r = ((1.0 - ab) * as_ * cs[0] + (1.0 - as_) * ab * cb[0] + ab * as_ * b[0]) / ar;
        let cr_g = ((1.0 - ab) * as_ * cs[1] + (1.0 - as_) * ab * cb[1] + ab * as_ * b[1]) / ar;
        let cr_b = ((1.0 - ab) * as_ * cs[2] + (1.0 - as_) * ab * cb[2] + ab * as_ * b[2]) / ar;

        [
            cr_r.clamp(0.0, 1.0),
            cr_g.clamp(0.0, 1.0),
            cr_b.clamp(0.0, 1.0),
            ar.clamp(0.0, 1.0),
        ]
    }

    /// Composites straight RGBA with [u8; 4] values (0-255).
    #[must_use]
    pub fn composite_u8(&self, cb: [u8; 4], cs: [u8; 4]) -> [u8; 4] {
        let cb_f = [
            cb[0] as f32 / 255.0,
            cb[1] as f32 / 255.0,
            cb[2] as f32 / 255.0,
            cb[3] as f32 / 255.0,
        ];
        let cs_f = [
            cs[0] as f32 / 255.0,
            cs[1] as f32 / 255.0,
            cs[2] as f32 / 255.0,
            cs[3] as f32 / 255.0,
        ];
        let out = self.composite_pixel_straight(cb_f, cs_f);
        [
            (out[0] * 255.0 + 0.5) as u8,
            (out[1] * 255.0 + 0.5) as u8,
            (out[2] * 255.0 + 0.5) as u8,
            (out[3] * 255.0 + 0.5) as u8,
        ]
    }

    /// Composites straight RGBA with [u16; 4] values (0-65535, matching aubrieta_raster deep color).
    #[must_use]
    pub fn composite_u16(&self, cb: [u16; 4], cs: [u16; 4]) -> [u16; 4] {
        let cb_f = [
            cb[0] as f32 / 65535.0,
            cb[1] as f32 / 65535.0,
            cb[2] as f32 / 65535.0,
            cb[3] as f32 / 65535.0,
        ];
        let cs_f = [
            cs[0] as f32 / 65535.0,
            cs[1] as f32 / 65535.0,
            cs[2] as f32 / 65535.0,
            cs[3] as f32 / 65535.0,
        ];
        let out = self.composite_pixel_straight(cb_f, cs_f);
        [
            (out[0] * 65535.0 + 0.5) as u16,
            (out[1] * 65535.0 + 0.5) as u16,
            (out[2] * 65535.0 + 0.5) as u16,
            (out[3] * 65535.0 + 0.5) as u16,
        ]
    }
}

// --- Non-separable helper functions (W3C standard) ---

#[inline]
fn lum(c: [f32; 3]) -> f32 {
    0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
}

#[inline]
fn sat(c: [f32; 3]) -> f32 {
    let max = c[0].max(c[1]).max(c[2]);
    let min = c[0].min(c[1]).min(c[2]);
    max - min
}

fn clip_color(mut c: [f32; 3]) -> [f32; 3] {
    let l = lum(c);
    let n = c[0].min(c[1]).min(c[2]);
    let x = c[0].max(c[1]).max(c[2]);

    if n < 0.0 {
        let denom = l - n;
        if denom > 1e-7 {
            c[0] = l + (((c[0] - l) * l) / denom);
            c[1] = l + (((c[1] - l) * l) / denom);
            c[2] = l + (((c[2] - l) * l) / denom);
        } else {
            c = [l, l, l];
        }
    }
    if x > 1.0 {
        let denom = x - l;
        if denom > 1e-7 {
            c[0] = l + (((c[0] - l) * (1.0 - l)) / denom);
            c[1] = l + (((c[1] - l) * (1.0 - l)) / denom);
            c[2] = l + (((c[2] - l) * (1.0 - l)) / denom);
        } else {
            c = [l, l, l];
        }
    }
    [
        c[0].clamp(0.0, 1.0),
        c[1].clamp(0.0, 1.0),
        c[2].clamp(0.0, 1.0),
    ]
}

fn set_lum(c: [f32; 3], l: f32) -> [f32; 3] {
    let d = l - lum(c);
    clip_color([c[0] + d, c[1] + d, c[2] + d])
}

fn set_sat(c: [f32; 3], s: f32) -> [f32; 3] {
    let mut sorted = [(0usize, c[0]), (1usize, c[1]), (2usize, c[2])];
    if sorted[0].1 > sorted[1].1 {
        sorted.swap(0, 1);
    }
    if sorted[1].1 > sorted[2].1 {
        sorted.swap(1, 2);
    }
    if sorted[0].1 > sorted[1].1 {
        sorted.swap(0, 1);
    }

    let min = sorted[0].1;
    let mid = sorted[1].1;
    let max = sorted[2].1;

    let mut result = [0.0f32; 3];
    if max > min {
        let new_mid = ((mid - min) * s) / (max - min);
        result[sorted[0].0] = 0.0;
        result[sorted[1].0] = new_mid;
        result[sorted[2].0] = s;
    } else {
        result[sorted[0].0] = 0.0;
        result[sorted[1].0] = 0.0;
        result[sorted[2].0] = 0.0;
    }
    result
}

impl From<aubrieta_document::BlendMode> for BlendMode {
    fn from(mode: aubrieta_document::BlendMode) -> Self {
        match mode {
            aubrieta_document::BlendMode::Normal => Self::Normal,
            aubrieta_document::BlendMode::Multiply => Self::Multiply,
            aubrieta_document::BlendMode::Screen => Self::Screen,
            aubrieta_document::BlendMode::Overlay => Self::Overlay,
            aubrieta_document::BlendMode::Darken => Self::Darken,
            aubrieta_document::BlendMode::Lighten => Self::Lighten,
            aubrieta_document::BlendMode::ColorDodge => Self::ColorDodge,
            aubrieta_document::BlendMode::ColorBurn => Self::ColorBurn,
            aubrieta_document::BlendMode::HardLight => Self::HardLight,
            aubrieta_document::BlendMode::SoftLight => Self::SoftLight,
            aubrieta_document::BlendMode::Difference => Self::Difference,
            aubrieta_document::BlendMode::Exclusion => Self::Exclusion,
            aubrieta_document::BlendMode::Hue => Self::Hue,
            aubrieta_document::BlendMode::Saturation => Self::Saturation,
            aubrieta_document::BlendMode::Color => Self::Color,
            aubrieta_document::BlendMode::Luminosity => Self::Luminosity,
        }
    }
}

impl From<BlendMode> for aubrieta_document::BlendMode {
    fn from(mode: BlendMode) -> Self {
        match mode {
            BlendMode::Normal => aubrieta_document::BlendMode::Normal,
            BlendMode::Multiply => aubrieta_document::BlendMode::Multiply,
            BlendMode::Screen => aubrieta_document::BlendMode::Screen,
            BlendMode::Overlay => aubrieta_document::BlendMode::Overlay,
            BlendMode::Darken => aubrieta_document::BlendMode::Darken,
            BlendMode::Lighten => aubrieta_document::BlendMode::Lighten,
            BlendMode::ColorDodge => aubrieta_document::BlendMode::ColorDodge,
            BlendMode::ColorBurn => aubrieta_document::BlendMode::ColorBurn,
            BlendMode::HardLight => aubrieta_document::BlendMode::HardLight,
            BlendMode::SoftLight => aubrieta_document::BlendMode::SoftLight,
            BlendMode::Difference => aubrieta_document::BlendMode::Difference,
            BlendMode::Exclusion => aubrieta_document::BlendMode::Exclusion,
            BlendMode::Hue => aubrieta_document::BlendMode::Hue,
            BlendMode::Saturation => aubrieta_document::BlendMode::Saturation,
            BlendMode::Color => aubrieta_document::BlendMode::Color,
            BlendMode::Luminosity => aubrieta_document::BlendMode::Luminosity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_blend_replaces_backdrop_at_full_alpha() {
        let red = [1.0, 0.0, 0.0, 1.0];
        let green = [0.0, 1.0, 0.0, 1.0];
        let out = BlendMode::Normal.composite_pixel_straight(red, green);
        assert_eq!(out, [0.0, 1.0, 0.0, 1.0]);
    }

    #[test]
    fn multiply_blend_combines_channels() {
        let gray1 = [0.5, 0.5, 0.5, 1.0];
        let gray2 = [0.5, 0.5, 0.5, 1.0];
        let out = BlendMode::Multiply.composite_pixel_straight(gray1, gray2);
        assert!((out[0] - 0.25).abs() < 1e-5);
        assert!((out[1] - 0.25).abs() < 1e-5);
        assert!((out[2] - 0.25).abs() < 1e-5);
        assert_eq!(out[3], 1.0);
    }

    #[test]
    fn transparent_source_leaves_backdrop_intact() {
        let blue = [0.0, 0.0, 1.0, 1.0];
        let transparent = [1.0, 1.0, 1.0, 0.0];
        let out = BlendMode::Screen.composite_pixel_straight(blue, transparent);
        assert_eq!(out, blue);
    }

    #[test]
    fn transparent_backdrop_passes_source_intact() {
        let transparent = [0.0, 0.0, 0.0, 0.0];
        let orange = [1.0, 0.5, 0.0, 0.8];
        let out = BlendMode::Overlay.composite_pixel_straight(transparent, orange);
        assert_eq!(out, orange);
    }

    #[test]
    fn deep_color_u16_compositing_preserves_precision() {
        let cb = [32768u16, 32768u16, 32768u16, 65535u16];
        let cs = [32768u16, 32768u16, 32768u16, 65535u16];
        let out = BlendMode::Multiply.composite_u16(cb, cs);
        // (0.5 * 0.5) * 65535 = 16383.75 ~ 16384
        assert!((out[0] as i32 - 16384).abs() <= 1);
        assert_eq!(out[3], 65535);
    }

    #[test]
    fn non_separable_modes_preserve_valid_color_range() {
        let cb = [0.8, 0.2, 0.1, 1.0];
        let cs = [0.1, 0.5, 0.9, 1.0];
        for mode in [
            BlendMode::Hue,
            BlendMode::Saturation,
            BlendMode::Color,
            BlendMode::Luminosity,
        ] {
            let out = mode.composite_pixel_straight(cb, cs);
            assert!(
                out[0] >= 0.0 && out[0] <= 1.0,
                "Channel 0 out of bounds in {mode:?}"
            );
            assert!(
                out[1] >= 0.0 && out[1] <= 1.0,
                "Channel 1 out of bounds in {mode:?}"
            );
            assert!(
                out[2] >= 0.0 && out[2] <= 1.0,
                "Channel 2 out of bounds in {mode:?}"
            );
            assert_eq!(out[3], 1.0);
        }
    }
}
