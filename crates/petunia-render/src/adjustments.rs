//! Adjustment math: exact spec formulas over straight alpha.
//!
//! Adjustments never touch premultiplied buffers directly: callers
//! straighten first, apply here, and premultiply back. Neutral
//! parameters are always identity; NaN/Inf inputs error instead of
//! propagating.

use crate::error::{RenderError, Result};
use petunia_render_model::{CurvePoint, LevelsChannel, RenderAdjustment};

/// One straight-alpha linear pixel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StraightPixel {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl StraightPixel {
    /// Reject non-finite lanes before any math runs.
    pub fn check(self) -> Result<Self> {
        if self.r.is_finite() && self.g.is_finite() && self.b.is_finite() && self.a.is_finite() {
            Ok(self)
        } else {
            Err(RenderError::Draw(
                "adjustment refuses non-finite input".to_string(),
            ))
        }
    }
}

/// Apply one adjustment to a straight-alpha pixel.
pub fn apply_adjustment(op: &RenderAdjustment, pixel: StraightPixel) -> Result<StraightPixel> {
    let pixel = pixel.check()?;
    match op {
        RenderAdjustment::Invert => Ok(StraightPixel {
            r: 1.0 - pixel.r.clamp(0.0, 1.0),
            g: 1.0 - pixel.g.clamp(0.0, 1.0),
            b: 1.0 - pixel.b.clamp(0.0, 1.0),
            a: pixel.a,
        }),
        RenderAdjustment::Grayscale => {
            let luminance = 0.2126 * pixel.r + 0.7152 * pixel.g + 0.0722 * pixel.b;
            Ok(StraightPixel {
                r: luminance,
                g: luminance,
                b: luminance,
                a: pixel.a,
            })
        }
        RenderAdjustment::Posterize { levels } => {
            if *levels < 2 {
                return Err(RenderError::Draw(format!(
                    "posterize needs levels >= 2, got {levels}"
                )));
            }
            let steps = (*levels - 1) as f32;
            Ok(StraightPixel {
                r: (pixel.r * steps).round() / steps,
                g: (pixel.g * steps).round() / steps,
                b: (pixel.b * steps).round() / steps,
                a: pixel.a,
            })
        }
        RenderAdjustment::BrightnessContrast {
            brightness,
            contrast,
        } => {
            validate_unit_signed("brightness", *brightness)?;
            validate_unit_signed("contrast", *contrast)?;
            let gain = 2.0f32.powf(*contrast);
            let map = |channel: f32| (((channel + brightness - 0.5) * gain) + 0.5).clamp(0.0, 1.0);
            Ok(StraightPixel {
                r: map(pixel.r),
                g: map(pixel.g),
                b: map(pixel.b),
                a: pixel.a,
            })
        }
        RenderAdjustment::Levels { channels } => {
            let mut rgb = [pixel.r, pixel.g, pixel.b];
            let maps = [
                &channels.master,
                channels.red.as_ref().unwrap_or(&channels.master),
                channels.green.as_ref().unwrap_or(&channels.master),
                channels.blue.as_ref().unwrap_or(&channels.master),
            ];
            // Master first, then per-channel: one versioned order.
            for channel in 0..3 {
                validate_levels(maps[0])?;
                validate_levels(maps[channel + 1])?;
                rgb[channel] = apply_levels_channel(rgb[channel], maps[0]);
                rgb[channel] = apply_levels_channel(rgb[channel], maps[channel + 1]);
            }
            Ok(StraightPixel {
                r: rgb[0],
                g: rgb[1],
                b: rgb[2],
                a: pixel.a,
            })
        }
        RenderAdjustment::Hsl {
            hue_shift,
            saturation,
            lightness,
        } => {
            validate_unit_signed("saturation", *saturation)?;
            validate_unit_signed("lightness", *lightness)?;
            if !hue_shift.is_finite() {
                return Err(RenderError::Draw("non-finite hue shift".to_string()));
            }
            let hsl = rgb_to_hsl([pixel.r, pixel.g, pixel.b]);
            let hue = (hsl[0] + hue_shift).rem_euclid(1.0);
            let saturation = (hsl[1] + saturation).clamp(0.0, 1.0);
            let lightness = (hsl[2] + lightness).clamp(0.0, 1.0);
            let rgb = hsl_to_rgb([hue, saturation, lightness]);
            Ok(StraightPixel {
                r: rgb[0],
                g: rgb[1],
                b: rgb[2],
                a: pixel.a,
            })
        }
        RenderAdjustment::Vibrance { amount } => {
            validate_unit_signed("vibrance", *amount)?;
            let hsl = rgb_to_hsl([pixel.r, pixel.g, pixel.b]);
            let (h, s, l) = (hsl[0], hsl[1], hsl[2]);
            let adjusted = if *amount >= 0.0 {
                s + amount * (1.0 - s)
            } else {
                s * (1.0 + amount)
            };
            let rgb = hsl_to_rgb([h, adjusted.clamp(0.0, 1.0), l]);
            Ok(StraightPixel {
                r: rgb[0],
                g: rgb[1],
                b: rgb[2],
                a: pixel.a,
            })
        }
        RenderAdjustment::ColorToAlpha { reference } => {
            color_to_alpha(pixel, [reference.r, reference.g, reference.b])
        }
        RenderAdjustment::Curves { points, monotonic } => {
            let lut = build_curve_lut(points, *monotonic)?;
            let map = |channel: f32| {
                let position = channel.clamp(0.0, 1.0) * (lut.len() - 1) as f32;
                let index = position.floor() as usize;
                let fraction = position - index as f32;
                if index + 1 >= lut.len() {
                    lut[lut.len() - 1]
                } else {
                    lut[index] + fraction * (lut[index + 1] - lut[index])
                }
            };
            Ok(StraightPixel {
                r: map(pixel.r),
                g: map(pixel.g),
                b: map(pixel.b),
                a: pixel.a,
            })
        }
    }
}

fn validate_unit_signed(name: &str, value: f32) -> Result<()> {
    if !value.is_finite() || !(-1.0..=1.0).contains(&value) {
        return Err(RenderError::Draw(format!("{name} outside -1..=1: {value}")));
    }
    Ok(())
}

fn validate_levels(channel: &LevelsChannel) -> Result<()> {
    if !(0.0..=1.0).contains(&channel.input_black)
        || !(0.0..=1.0).contains(&channel.input_white)
        || channel.input_black >= channel.input_white
    {
        return Err(RenderError::Draw(format!(
            "levels need 0 <= black < white <= 1, got {}..{}",
            channel.input_black, channel.input_white
        )));
    }
    if !(channel.gamma.is_finite() && channel.gamma > 0.0) {
        return Err(RenderError::Draw(format!(
            "levels gamma must be positive finite, got {}",
            channel.gamma
        )));
    }
    if !(0.0..=1.0).contains(&channel.output_black)
        || !(0.0..=1.0).contains(&channel.output_white)
        || channel.output_black > channel.output_white
    {
        return Err(RenderError::Draw("levels output range invalid".to_string()));
    }
    Ok(())
}

fn apply_levels_channel(x: f32, channel: &LevelsChannel) -> f32 {
    let normalized =
        ((x - channel.input_black) / (channel.input_white - channel.input_black)).clamp(0.0, 1.0);
    let graded = normalized.powf(1.0 / channel.gamma);
    channel.output_black + graded * (channel.output_white - channel.output_black)
}

fn rgb_to_hsl(rgb: [f32; 3]) -> [f32; 3] {
    let max = rgb[0].max(rgb[1]).max(rgb[2]);
    let min = rgb[0].min(rgb[1]).min(rgb[2]);
    let lightness = (max + min) / 2.0;
    if (max - min).abs() < f32::EPSILON {
        return [0.0, 0.0, lightness];
    }
    let delta = max - min;
    let saturation = if lightness > 0.5 {
        delta / (2.0 - max - min)
    } else {
        delta / (max + min)
    };
    let hue = if (max - rgb[0]).abs() < f32::EPSILON {
        (rgb[1] - rgb[2]) / delta + if rgb[1] < rgb[2] { 6.0 } else { 0.0 }
    } else if (max - rgb[1]).abs() < f32::EPSILON {
        (rgb[2] - rgb[0]) / delta + 2.0
    } else {
        (rgb[0] - rgb[1]) / delta + 4.0
    } / 6.0;
    [
        hue.rem_euclid(1.0),
        saturation.clamp(0.0, 1.0),
        lightness.clamp(0.0, 1.0),
    ]
}

fn hsl_to_rgb(hsl: [f32; 3]) -> [f32; 3] {
    let (h, s, l) = (
        hsl[0].rem_euclid(1.0),
        hsl[1].clamp(0.0, 1.0),
        hsl[2].clamp(0.0, 1.0),
    );
    if s <= 0.0 {
        return [l, l, l];
    }
    // Standard hue-to-rgb with chroma formulation.
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match (h * 6.0).floor() as u32 % 6 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [r + m, g + m, b + m]
}

fn color_to_alpha(pixel: StraightPixel, reference: [f32; 3]) -> Result<StraightPixel> {
    let source = [pixel.r, pixel.g, pixel.b];
    let mut alpha = 0.0f32;
    for channel in 0..3 {
        let (source_channel, reference_channel) = (source[channel], reference[channel]);
        let candidate = if (source_channel - reference_channel).abs() < f32::EPSILON {
            0.0
        } else if source_channel > reference_channel {
            let denom = 1.0 - reference_channel;
            if denom <= 0.0 {
                1.0
            } else {
                (source_channel - reference_channel) / denom
            }
        } else {
            if reference_channel <= 0.0 {
                1.0
            } else {
                (reference_channel - source_channel) / reference_channel
            }
        };
        alpha = alpha.max(candidate.clamp(0.0, 1.0));
    }
    if alpha <= 0.0 {
        return Ok(StraightPixel {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        });
    }
    let foreground = [
        (source[0] - (1.0 - alpha) * reference[0]) / alpha,
        (source[1] - (1.0 - alpha) * reference[1]) / alpha,
        (source[2] - (1.0 - alpha) * reference[2]) / alpha,
    ];
    Ok(StraightPixel {
        r: foreground[0].clamp(0.0, 1.0),
        g: foreground[1].clamp(0.0, 1.0),
        b: foreground[2].clamp(0.0, 1.0),
        a: (pixel.a * alpha).clamp(0.0, 1.0),
    })
}

fn build_curve_lut(points: &[CurvePoint], monotonic: bool) -> Result<Vec<f32>> {
    if points.len() < 2 {
        return Err(RenderError::Draw(
            "curves need at least two control points".to_string(),
        ));
    }
    for point in points {
        if !point.x.is_finite() || !point.y.is_finite() {
            return Err(RenderError::Draw("non-finite curve point".to_string()));
        }
        if !(0.0..=1.0).contains(&point.x) {
            return Err(RenderError::Draw(format!(
                "curve x outside 0..=1: {}",
                point.x
            )));
        }
    }
    for pair in points.windows(2) {
        if pair[1].x <= pair[0].x {
            return Err(RenderError::Draw(
                "curve points must order strictly by x".to_string(),
            ));
        }
    }
    let non_monotonic = points.windows(2).any(|pair| pair[1].y < pair[0].y);
    if non_monotonic && monotonic {
        return Err(RenderError::Draw(
            "monotonic curve rejects decreasing control points".to_string(),
        ));
    }
    // Monotone cubic (Fritsch-Carlson) when the data is monotone,
    // linear segments otherwise; either way LUT has 256 entries.
    let monotone_data = !non_monotonic;
    let tangents = fritsch_carlson(points, monotone_data);
    const STEPS: usize = 256;
    let mut lut = Vec::with_capacity(STEPS);
    for step in 0..STEPS {
        let x = step as f32 / (STEPS - 1) as f32;
        lut.push(sample_curve(points, &tangents, x, monotone_data));
    }
    Ok(lut)
}

fn fritsch_carlson(points: &[CurvePoint], monotone: bool) -> Vec<f32> {
    let count = points.len();
    if count < 2 {
        return vec![0.0; count];
    }
    if count == 2 || !monotone {
        // Straight secant: two points or explicitly free curves.
        let slope = (points[1].y - points[0].y) / (points[1].x - points[0].x).max(f32::EPSILON);
        return vec![slope; count];
    }
    let mut deltas = Vec::with_capacity(count - 1);
    let mut h = Vec::with_capacity(count - 1);
    for pair in points.windows(2) {
        let dx = pair[1].x - pair[0].x;
        h.push(dx);
        deltas.push((pair[1].y - pair[0].y) / dx);
    }
    let mut tangents = vec![0.0; count];
    tangents[0] = deltas[0];
    tangents[count - 1] = deltas[count - 2];
    for index in 1..count - 1 {
        if deltas[index - 1] == 0.0 || deltas[index] == 0.0 {
            tangents[index] = 0.0;
        } else {
            let w1 = 2.0 * h[index] + h[index - 1];
            let w2 = h[index] + 2.0 * h[index - 1];
            tangents[index] = (w1 + w2) / (w1 / deltas[index - 1] + w2 / deltas[index]);
        }
    }
    tangents
}

fn sample_curve(points: &[CurvePoint], tangents: &[f32], x: f32, monotone: bool) -> f32 {
    if x <= points[0].x {
        return points[0].y;
    }
    if x >= points[points.len() - 1].x {
        return points[points.len() - 1].y;
    }
    let index = points
        .windows(2)
        .position(|pair| x >= pair[0].x && x <= pair[1].x)
        .unwrap_or(0);
    let (p0, p1) = (points[index], points[index + 1]);
    let span = p1.x - p0.x;
    if !monotone {
        let t = (x - p0.x) / span;
        return p0.y + t * (p1.y - p0.y);
    }
    let t = (x - p0.x) / span;
    let (m0, m1) = (tangents[index] * span, tangents[index + 1] * span);
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * p0.y
        + (t3 - 2.0 * t2 + t) * m0
        + (-2.0 * t3 + 3.0 * t2) * p1.y
        + (t3 - t2) * m1
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_render_model::{LevelsChannel, LevelsChannels};

    fn pixel(r: f32, g: f32, b: f32) -> StraightPixel {
        StraightPixel { r, g, b, a: 1.0 }
    }

    fn neutral(op: &RenderAdjustment) {
        // Neutral parameters are identity on arbitrary input.
        let input = pixel(0.25, 0.5, 0.75);
        let out = apply_adjustment(op, input).expect("neutral applies");
        for (a, b) in [(out.r, input.r), (out.g, input.g), (out.b, input.b)] {
            assert!((a - b).abs() < 1e-5, "{out:?}");
        }
        assert_eq!(out.a, 1.0);
    }

    #[test]
    fn neutral_parameters_are_identity() {
        // Invert has no neutral parameters (it is its own inverse);
        // every other adjustment below must preserve its input.
        neutral(&RenderAdjustment::BrightnessContrast {
            brightness: 0.0,
            contrast: 0.0,
        });
        neutral(&RenderAdjustment::Hsl {
            hue_shift: 0.0,
            saturation: 0.0,
            lightness: 0.0,
        });
        neutral(&RenderAdjustment::Vibrance { amount: 0.0 });
    }

    #[test]
    fn invert_and_posterize_match_spec() {
        let out =
            apply_adjustment(&RenderAdjustment::Invert, pixel(0.25, 0.5, 1.0)).expect("inverts");
        assert_eq!((out.r, out.g, out.b), (0.75, 0.5, 0.0));
        let out = apply_adjustment(
            &RenderAdjustment::Posterize { levels: 2 },
            pixel(0.3, 0.7, 0.0),
        )
        .expect("posterizes");
        assert_eq!((out.r, out.g, out.b), (0.0, 1.0, 0.0));
        assert!(apply_adjustment(
            &RenderAdjustment::Posterize { levels: 1 },
            pixel(0.5, 0.5, 0.5)
        )
        .is_err());
    }

    #[test]
    fn grayscale_uses_luminance_not_mean() {
        let out =
            apply_adjustment(&RenderAdjustment::Grayscale, pixel(1.0, 0.0, 0.0)).expect("grays");
        assert!((out.r - 0.2126).abs() < 1e-4, "{out:?}");
        assert_eq!(out.r, out.g);
        assert_eq!(out.g, out.b);
    }

    #[test]
    fn levels_identity_and_gamma() {
        let identity = LevelsChannels {
            master: LevelsChannel {
                input_black: 0.0,
                input_white: 1.0,
                gamma: 1.0,
                output_black: 0.0,
                output_white: 1.0,
            },
            red: None,
            green: None,
            blue: None,
        };
        neutral(&RenderAdjustment::Levels {
            channels: identity.clone(),
        });
        assert!(apply_adjustment(
            &RenderAdjustment::Levels {
                channels: LevelsChannels {
                    master: LevelsChannel {
                        gamma: 0.0,
                        ..identity.master
                    },
                    ..identity
                },
            },
            pixel(0.5, 0.5, 0.5),
        )
        .is_err());
    }

    #[test]
    fn color_to_alpha_recomposes_over_reference() {
        use petunia_render_model::RenderColor;
        let reference = RenderColor {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };
        let original = pixel(1.0, 0.25, 0.25);
        let removed = apply_adjustment(&RenderAdjustment::ColorToAlpha { reference }, original)
            .expect("removes white");
        // Recomposite over white must approximate the original.
        let over = removed.a;
        let recomposed = [
            removed.r * over + 1.0 * (1.0 - over),
            removed.g * over + 1.0 * (1.0 - over),
            removed.b * over + 1.0 * (1.0 - over),
        ];
        for (a, b) in recomposed.iter().zip([1.0, 0.25, 0.25]) {
            assert!((a - b).abs() < 1e-4, "{recomposed:?}");
        }
    }

    #[test]
    fn curves_reject_bad_control_points() {
        let bad = RenderAdjustment::Curves {
            points: vec![CurvePoint { x: 0.0, y: 0.0 }, CurvePoint { x: 0.0, y: 1.0 }],
            monotonic: true,
        };
        assert!(apply_adjustment(&bad, pixel(0.5, 0.5, 0.5)).is_err());
        let good = RenderAdjustment::Curves {
            points: vec![CurvePoint { x: 0.0, y: 0.0 }, CurvePoint { x: 1.0, y: 1.0 }],
            monotonic: true,
        };
        let out = apply_adjustment(&good, pixel(0.3, 0.3, 0.3)).expect("identity curve");
        assert!((out.r - 0.3).abs() < 1e-3, "{out:?}");
    }

    #[test]
    fn non_finite_inputs_error_loudly() {
        let bad = StraightPixel {
            r: f32::NAN,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        assert!(apply_adjustment(&RenderAdjustment::Invert, bad).is_err());
    }
}
