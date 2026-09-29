//! Canonical Tonal Adjustments Model conforming to Spec 10.10.
//!
//! Ordered, non-destructive tonal adjustment pipeline including Levels,
//! Curves, HSL, Exposure, and White Balance. Each adjustment declares its
//! parameters and evaluated transfer function in normalized RGB space [0.0, 1.0].

use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_one() -> f64 {
    1.0
}

/// Channel target for multi-channel adjustments (Levels, Curves).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdjustmentChannel {
    #[default]
    Master,
    Red,
    Green,
    Blue,
}

/// Black point, white point, gamma, and output range for a single channel.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChannelLevels {
    /// Input black point normalized to [0.0, 1.0] (default 0.0).
    pub input_black: f64,
    /// Input white point normalized to [0.0, 1.0] (default 1.0).
    pub input_white: f64,
    /// Midpoint gamma / power exponent in [0.1, 10.0] (default 1.0).
    pub gamma: f64,
    /// Output black point normalized to [0.0, 1.0] (default 0.0).
    pub output_black: f64,
    /// Output white point normalized to [0.0, 1.0] (default 1.0).
    pub output_white: f64,
}

impl Default for ChannelLevels {
    fn default() -> Self {
        Self {
            input_black: 0.0,
            input_white: 1.0,
            gamma: 1.0,
            output_black: 0.0,
            output_white: 1.0,
        }
    }
}

impl ChannelLevels {
    /// Evaluates transfer function for a single normalized channel value in [0.0, 1.0].
    #[must_use]
    pub fn evaluate(&self, x: f32) -> f32 {
        let x = x as f64;
        let in_black = self.input_black.clamp(0.0, 1.0);
        let in_white = self.input_white.clamp(0.0, 1.0);
        let out_black = self.output_black.clamp(0.0, 1.0);
        let out_white = self.output_white.clamp(0.0, 1.0);
        let gamma = self.gamma.clamp(0.1, 10.0);

        if in_white <= in_black || (x - in_black) <= 1e-6 {
            return out_black as f32;
        }
        if (in_white - x) <= 1e-6 {
            return out_white as f32;
        }

        let normalized = ((x - in_black) / (in_white - in_black)).clamp(0.0, 1.0);
        let adjusted = if (gamma - 1.0).abs() < 1e-6 {
            normalized
        } else {
            normalized.powf(1.0 / gamma)
        };
        let out = out_black + adjusted * (out_white - out_black);
        out.clamp(0.0, 1.0) as f32
    }
}

/// Typed V1 non-destructive tonal adjustment kinds (Spec 10.10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AdjustmentKind {
    /// Levels adjustment: input/output clipping and gamma midpoint per channel.
    Levels {
        master: ChannelLevels,
        #[serde(default)]
        red: Option<ChannelLevels>,
        #[serde(default)]
        green: Option<ChannelLevels>,
        #[serde(default)]
        blue: Option<ChannelLevels>,
    },
    /// Curves adjustment: arbitrary spline transfer curve per channel.
    Curves {
        /// Master RGB spline control points [x, y] in [0.0, 1.0].
        master_points: Vec<[f64; 2]>,
        #[serde(default)]
        red_points: Option<Vec<[f64; 2]>>,
        #[serde(default)]
        green_points: Option<Vec<[f64; 2]>>,
        #[serde(default)]
        blue_points: Option<Vec<[f64; 2]>>,
    },
    /// Hue, Saturation, and Lightness adjustment.
    Hsl {
        /// Hue rotation in degrees [-180.0, 180.0] (default 0.0).
        hue_shift: f64,
        /// Saturation adjustment in [-1.0, 1.0] (default 0.0).
        saturation: f64,
        /// Lightness adjustment in [-1.0, 1.0] (default 0.0).
        lightness: f64,
    },
    /// Exposure adjustment: EV stops, black offset, and gamma.
    Exposure {
        /// Exposure value (EV stops) in [-5.0, 5.0] (default 0.0).
        exposure: f64,
        /// Black level offset in [-0.5, 0.5] (default 0.0).
        offset: f64,
        /// Gamma power exponent in [0.1, 5.0] (default 1.0).
        gamma: f64,
    },
    /// White Balance adjustment: color temperature and tint.
    WhiteBalance {
        /// Temperature shift in [-1.0, 1.0] (-1 cool/blue, +1 warm/orange).
        temperature: f64,
        /// Tint shift in [-1.0, 1.0] (-1 green, +1 magenta).
        tint: f64,
    },
}

impl AdjustmentKind {
    /// Creates a default neutral Levels adjustment.
    #[must_use]
    pub fn default_levels() -> Self {
        Self::Levels {
            master: ChannelLevels::default(),
            red: None,
            green: None,
            blue: None,
        }
    }

    /// Creates a default neutral Curves adjustment (linear 0->1 ramp).
    #[must_use]
    pub fn default_curves() -> Self {
        Self::Curves {
            master_points: vec![[0.0, 0.0], [1.0, 1.0]],
            red_points: None,
            green_points: None,
            blue_points: None,
        }
    }

    /// Creates a default neutral HSL adjustment.
    #[must_use]
    pub fn default_hsl() -> Self {
        Self::Hsl {
            hue_shift: 0.0,
            saturation: 0.0,
            lightness: 0.0,
        }
    }

    /// Creates a default neutral Exposure adjustment.
    #[must_use]
    pub fn default_exposure() -> Self {
        Self::Exposure {
            exposure: 0.0,
            offset: 0.0,
            gamma: 1.0,
        }
    }

    /// Creates a default neutral White Balance adjustment.
    #[must_use]
    pub fn default_white_balance() -> Self {
        Self::WhiteBalance {
            temperature: 0.0,
            tint: 0.0,
        }
    }

    /// Evaluates this adjustment on a normalized linear RGB triple in [0.0, 1.0].
    #[must_use]
    pub fn evaluate_rgb(&self, rgb: [f32; 3]) -> [f32; 3] {
        match self {
            Self::Levels {
                master,
                red,
                green,
                blue,
            } => {
                let r = red.as_ref().map_or(rgb[0], |l| l.evaluate(rgb[0]));
                let g = green.as_ref().map_or(rgb[1], |l| l.evaluate(rgb[1]));
                let b = blue.as_ref().map_or(rgb[2], |l| l.evaluate(rgb[2]));
                [master.evaluate(r), master.evaluate(g), master.evaluate(b)]
            }
            Self::Curves {
                master_points,
                red_points,
                green_points,
                blue_points,
            } => {
                let r = red_points
                    .as_deref()
                    .map_or(rgb[0], |pts| evaluate_curve(pts, rgb[0]));
                let g = green_points
                    .as_deref()
                    .map_or(rgb[1], |pts| evaluate_curve(pts, rgb[1]));
                let b = blue_points
                    .as_deref()
                    .map_or(rgb[2], |pts| evaluate_curve(pts, rgb[2]));
                [
                    evaluate_curve(master_points, r),
                    evaluate_curve(master_points, g),
                    evaluate_curve(master_points, b),
                ]
            }
            Self::Hsl {
                hue_shift,
                saturation,
                lightness,
            } => apply_hsl(rgb, *hue_shift, *saturation, *lightness),
            Self::Exposure {
                exposure,
                offset,
                gamma,
            } => apply_exposure(rgb, *exposure, *offset, *gamma),
            Self::WhiteBalance { temperature, tint } => {
                apply_white_balance(rgb, *temperature, *tint)
            }
        }
    }
}

/// Evaluates a 2D spline curve at input `x` in [0.0, 1.0] using Monotone Cubic Hermite interpolation.
#[must_use]
pub fn evaluate_curve(points: &[[f64; 2]], x: f32) -> f32 {
    if points.is_empty() {
        return x;
    }
    if points.len() == 1 {
        return points[0][1].clamp(0.0, 1.0) as f32;
    }
    let x_f64 = (x as f64).clamp(0.0, 1.0);

    // If before first point, extrapolate/clamp to first point Y
    if x_f64 <= points[0][0] {
        return points[0][1].clamp(0.0, 1.0) as f32;
    }
    // If after last point, clamp to last point Y
    if x_f64 >= points[points.len() - 1][0] {
        return points[points.len() - 1][1].clamp(0.0, 1.0) as f32;
    }

    // 2-point curve is purely linear
    if points.len() == 2 {
        let p0 = points[0];
        let p1 = points[1];
        let span = p1[0] - p0[0];
        if span.abs() < 1e-6 {
            return p0[1].clamp(0.0, 1.0) as f32;
        }
        let t = (x_f64 - p0[0]) / span;
        let y = p0[1] + t * (p1[1] - p0[1]);
        return y.clamp(0.0, 1.0) as f32;
    }

    let n = points.len();
    // Calculate secants
    let mut deltas = Vec::with_capacity(n - 1);
    let mut hs = Vec::with_capacity(n - 1);
    for w in points.windows(2) {
        let h = w[1][0] - w[0][0];
        let delta = if h.abs() > 1e-6 {
            (w[1][1] - w[0][1]) / h
        } else {
            0.0
        };
        hs.push(h);
        deltas.push(delta);
    }

    // Initial tangents (Fritsch-Carlson)
    let mut d = Vec::with_capacity(n);
    d.push(deltas[0]);
    for i in 1..n - 1 {
        d.push((deltas[i - 1] + deltas[i]) * 0.5);
    }
    d.push(deltas[n - 2]);

    // Monotonicity enforcement
    for i in 0..n - 1 {
        if deltas[i].abs() < 1e-6 {
            d[i] = 0.0;
            d[i + 1] = 0.0;
        } else {
            let alpha = d[i] / deltas[i];
            let beta = d[i + 1] / deltas[i];
            let dist_sq = alpha * alpha + beta * beta;
            if dist_sq > 9.0 {
                let tau = 3.0 / dist_sq.sqrt();
                d[i] = tau * alpha * deltas[i];
                d[i + 1] = tau * beta * deltas[i];
            }
        }
    }

    // Locate segment and evaluate cubic Hermite
    for i in 0..n - 1 {
        let x0 = points[i][0];
        let x1 = points[i + 1][0];
        if x_f64 >= x0 && x_f64 <= x1 {
            let h = hs[i];
            if h.abs() < 1e-6 {
                return points[i][1].clamp(0.0, 1.0) as f32;
            }
            let t = (x_f64 - x0) / h;
            let t2 = t * t;
            let t3 = t2 * t;

            let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
            let h10 = t3 - 2.0 * t2 + t;
            let h01 = -2.0 * t3 + 3.0 * t2;
            let h11 = t3 - t2;

            let y =
                h00 * points[i][1] + h10 * h * d[i] + h01 * points[i + 1][1] + h11 * h * d[i + 1];
            return y.clamp(0.0, 1.0) as f32;
        }
    }

    x
}

/// Converts RGB [0..1] to HSL [0..360, 0..1, 0..1].
fn rgb_to_hsl(rgb: [f32; 3]) -> [f32; 3] {
    let r = rgb[0];
    let g = rgb[1];
    let b = rgb[2];
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let l = (max + min) / 2.0;

    if delta < 1e-5 {
        return [0.0, 0.0, l];
    }

    let s = if l > 0.5 {
        delta / (2.0 - max - min)
    } else {
        delta / (max + min)
    };

    let mut h = if (max - r).abs() < 1e-5 {
        (g - b) / delta + (if g < b { 6.0 } else { 0.0 })
    } else if (max - g).abs() < 1e-5 {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    h *= 60.0;

    [h, s, l]
}

/// Helper for HSL to RGB conversion.
fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

/// Converts HSL [0..360, 0..1, 0..1] to RGB [0..1].
fn hsl_to_rgb(hsl: [f32; 3]) -> [f32; 3] {
    let h = hsl[0] / 360.0;
    let s = hsl[1].clamp(0.0, 1.0);
    let l = hsl[2].clamp(0.0, 1.0);

    if s < 1e-5 {
        return [l, l, l];
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    [
        hue_to_rgb(p, q, h + 1.0 / 3.0).clamp(0.0, 1.0),
        hue_to_rgb(p, q, h).clamp(0.0, 1.0),
        hue_to_rgb(p, q, h - 1.0 / 3.0).clamp(0.0, 1.0),
    ]
}

/// Applies HSL shift to RGB triple.
#[must_use]
pub fn apply_hsl(rgb: [f32; 3], hue_shift_deg: f64, sat_shift: f64, light_shift: f64) -> [f32; 3] {
    let mut hsl = rgb_to_hsl(rgb);
    hsl[0] = (hsl[0] + hue_shift_deg as f32).rem_euclid(360.0);
    if sat_shift >= 0.0 {
        hsl[1] += (sat_shift as f32) * (1.0 - hsl[1]);
    } else {
        hsl[1] += (sat_shift as f32) * hsl[1];
    }
    if light_shift >= 0.0 {
        hsl[2] += (light_shift as f32) * (1.0 - hsl[2]);
    } else {
        hsl[2] += (light_shift as f32) * hsl[2];
    }
    hsl_to_rgb(hsl)
}

/// Applies exposure (EV), offset, and gamma correction to RGB.
#[must_use]
pub fn apply_exposure(rgb: [f32; 3], ev: f64, offset: f64, gamma: f64) -> [f32; 3] {
    let factor = 2.0_f64.powf(ev);
    let g = gamma.clamp(0.1, 5.0);
    let adjust = |c: f32| {
        let v = (c as f64 * factor + offset).clamp(0.0, 1.0);
        if (g - 1.0).abs() < 1e-6 {
            v as f32
        } else {
            v.powf(1.0 / g) as f32
        }
    };
    [adjust(rgb[0]), adjust(rgb[1]), adjust(rgb[2])]
}

/// Applies temperature and tint white balance to RGB.
#[must_use]
pub fn apply_white_balance(rgb: [f32; 3], temperature: f64, tint: f64) -> [f32; 3] {
    let temp = temperature.clamp(-1.0, 1.0);
    let t = tint.clamp(-1.0, 1.0);

    let mut r = rgb[0] as f64 + temp * 0.20 + t * 0.08;
    let mut g = rgb[1] as f64 - t * 0.16;
    let mut b = rgb[2] as f64 - temp * 0.20 + t * 0.08;

    r = r.clamp(0.0, 1.0);
    g = g.clamp(0.0, 1.0);
    b = b.clamp(0.0, 1.0);

    [r as f32, g as f32, b as f32]
}

/// Single adjustment entry in an object's appearance stack (Spec 10.10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdjustmentItem {
    /// Local identity within the appearance stack.
    pub id: u32,
    /// The typed adjustment definition and parameters.
    pub kind: AdjustmentKind,
    /// Blend opacity of this adjustment in [0.0, 1.0].
    #[serde(default = "default_one")]
    pub opacity: f64,
    /// Whether the adjustment is active.
    #[serde(default = "default_true")]
    pub visible: bool,
}

impl AdjustmentItem {
    /// Creates a new adjustment item.
    #[must_use]
    pub fn new(id: u32, kind: AdjustmentKind) -> Self {
        Self {
            id,
            kind,
            opacity: 1.0,
            visible: true,
        }
    }

    /// Evaluates this adjustment with opacity blend on input RGB.
    #[must_use]
    pub fn apply(&self, rgb: [f32; 3]) -> [f32; 3] {
        if !self.visible || self.opacity <= 0.0 {
            return rgb;
        }
        let adjusted = self.kind.evaluate_rgb(rgb);
        let alpha = self.opacity.clamp(0.0, 1.0) as f32;
        if (alpha - 1.0).abs() < 1e-4 {
            adjusted
        } else {
            [
                rgb[0] + (adjusted[0] - rgb[0]) * alpha,
                rgb[1] + (adjusted[1] - rgb[1]) * alpha,
                rgb[2] + (adjusted[2] - rgb[2]) * alpha,
            ]
        }
    }
}

/// Folds an ordered slice of adjustment items across an input color.
#[must_use]
pub fn apply_adjustment_chain(mut rgb: [f32; 3], adjustments: &[AdjustmentItem]) -> [f32; 3] {
    for adj in adjustments {
        rgb = adj.apply(rgb);
    }
    rgb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_evaluation_preserves_identity_with_default() {
        let levels = ChannelLevels::default();
        assert!((levels.evaluate(0.0) - 0.0).abs() < 1e-4);
        assert!((levels.evaluate(0.5) - 0.5).abs() < 1e-4);
        assert!((levels.evaluate(1.0) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn levels_clips_black_and_white_points() {
        let levels = ChannelLevels {
            input_black: 0.2,
            input_white: 0.8,
            gamma: 1.0,
            output_black: 0.0,
            output_white: 1.0,
        };
        assert_eq!(levels.evaluate(0.1), 0.0);
        assert_eq!(levels.evaluate(0.2), 0.0);
        assert_eq!(levels.evaluate(0.8), 1.0);
        assert_eq!(levels.evaluate(0.9), 1.0);
        assert!((levels.evaluate(0.5) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn curves_evaluation_identity() {
        let curve = vec![[0.0, 0.0], [1.0, 1.0]];
        assert!((evaluate_curve(&curve, 0.25) - 0.25).abs() < 1e-3);
        assert!((evaluate_curve(&curve, 0.50) - 0.50).abs() < 1e-3);
    }

    #[test]
    fn hsl_identity_and_shifts() {
        let white = [1.0, 1.0, 1.0];
        let res = apply_hsl(white, 0.0, 0.0, 0.0);
        assert!((res[0] - 1.0).abs() < 1e-4);

        let red = [1.0, 0.0, 0.0];
        let green = apply_hsl(red, 120.0, 0.0, 0.0);
        assert!((green[0] - 0.0).abs() < 1e-2);
        assert!((green[1] - 1.0).abs() < 1e-2);
        assert!((green[2] - 0.0).abs() < 1e-2);
    }

    #[test]
    fn exposure_ev_doubles_and_halves() {
        let mid = [0.25, 0.25, 0.25];
        let plus_one = apply_exposure(mid, 1.0, 0.0, 1.0);
        assert!((plus_one[0] - 0.50).abs() < 1e-4);
        let minus_one = apply_exposure(mid, -1.0, 0.0, 1.0);
        assert!((minus_one[0] - 0.125).abs() < 1e-4);
    }
}
