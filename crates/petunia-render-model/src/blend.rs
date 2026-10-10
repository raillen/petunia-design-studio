//! Shared linear RGB blend mathematics used by reference rendering and raster editing.
use petunia_core::BlendMode;

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
