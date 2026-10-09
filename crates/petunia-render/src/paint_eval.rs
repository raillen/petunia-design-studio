//! Paint evaluation: authorial paint plus geometry into samples.
//!
//! Gradients follow the shared contract: spread first, adjacent
//! stops, midpoint remap, interpolation-space conversion, lerp, then
//! back to compositing linear. Degenerate inputs error loudly;
//! midpoints outside `(0, 1)` are rejected, never silently clamped.

use crate::error::{RenderError, Result};
use petunia_core::paint::{GradientInterpolation, GradientSpread};
use petunia_render_model::{RenderColor, RenderGradient, RenderGradientStop};

/// Sample one gradient at parameter `t`.
pub fn sample_gradient(gradient: &RenderGradient, t: f64) -> Result<RenderColor> {
    if gradient.stops.is_empty() {
        return Err(RenderError::Draw(
            "gradient needs at least one stop".to_string(),
        ));
    }
    let spread = apply_spread(t, gradient.spread);
    let (left, right) = locate_stops(&gradient.stops, spread);
    interpolate_stops(left, right, spread, gradient.interpolation)
}

/// Evaluate a linear gradient at a document point.
pub fn sample_linear(gradient: &RenderGradient, point: (f64, f64)) -> Result<RenderColor> {
    let dx = gradient.end.0 - gradient.start.0;
    let dy = gradient.end.1 - gradient.start.1;
    let denom = dx * dx + dy * dy;
    if denom <= 0.0 {
        return Err(RenderError::Draw(
            "degenerate linear gradient: coincident endpoints".to_string(),
        ));
    }
    let t = ((point.0 - gradient.start.0) * dx + (point.1 - gradient.start.1) * dy) / denom;
    sample_gradient(gradient, t)
}

/// Evaluate a radial gradient at a document point under the local
/// unit-radius space the geometry resolved.
pub fn sample_radial(gradient: &RenderGradient, point: (f64, f64)) -> Result<RenderColor> {
    if !(gradient.radius.is_finite() && gradient.radius > 0.0) {
        return Err(RenderError::Draw(
            "degenerate radial gradient: non-positive radius".to_string(),
        ));
    }
    let dx = (point.0 - gradient.start.0) / gradient.radius;
    let dy = (point.1 - gradient.start.1) / gradient.radius;
    sample_gradient(gradient, dx.hypot(dy))
}

fn apply_spread(t: f64, spread: GradientSpread) -> f64 {
    if !t.is_finite() {
        return 0.0;
    }
    match spread {
        GradientSpread::Pad => t.clamp(0.0, 1.0),
        GradientSpread::Repeat => t - t.floor(),
        GradientSpread::Reflect => {
            let period = t.floor();
            let unit = t - period;
            if (period as i64) & 1 == 0 {
                unit
            } else {
                1.0 - unit
            }
        }
    }
}

fn locate_stops(stops: &[RenderGradientStop], t: f64) -> (RenderGradientStop, RenderGradientStop) {
    let mut left = stops[0];
    let mut right = stops[stops.len() - 1];
    for stop in stops {
        if f64::from(stop.offset) <= t {
            left = *stop;
        }
        if f64::from(stop.offset) >= t {
            right = *stop;
            break;
        }
    }
    (left, right)
}

fn interpolate_stops(
    left: RenderGradientStop,
    right: RenderGradientStop,
    t: f64,
    interpolation: GradientInterpolation,
) -> Result<RenderColor> {
    if (right.offset - left.offset).abs() <= f32::EPSILON {
        return Ok(mix_straight(left.color, right.color, 0.5, interpolation));
    }
    let unit =
        ((t - f64::from(left.offset)) / f64::from(right.offset - left.offset)).clamp(0.0, 1.0);
    // Midpoint remap: gamma = ln(0.5)/ln(m) sends u=m to exactly 0.5.
    // The left stop owns the interval's midpoint; endpoints 0/1
    // singularize the parametrization and are rejected.
    let midpoint = f64::from(left.midpoint);
    if !(midpoint > 0.0 && midpoint < 1.0) {
        return Err(RenderError::Draw(format!(
            "gradient midpoint outside (0, 1): {midpoint}"
        )));
    }
    let remapped = if (midpoint - 0.5).abs() <= 1e-9 {
        unit
    } else {
        (0.5f64.ln() / midpoint.ln() * unit.ln().max(-745.0)).exp()
    };
    Ok(mix_straight(
        left.color,
        right.color,
        remapped as f32,
        interpolation,
    ))
}

fn mix_straight(
    left: RenderColor,
    right: RenderColor,
    t: f32,
    interpolation: GradientInterpolation,
) -> RenderColor {
    let (a, b) = (
        to_interpolation(left, interpolation),
        to_interpolation(right, interpolation),
    );
    let mixed = [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ];
    let rgb = from_interpolation(mixed, interpolation);
    RenderColor {
        r: rgb[0],
        g: rgb[1],
        b: rgb[2],
        a: left.a + (right.a - left.a) * t,
    }
}

fn to_interpolation(color: RenderColor, interpolation: GradientInterpolation) -> [f32; 3] {
    match interpolation {
        GradientInterpolation::LinearRgb => [color.r, color.g, color.b],
        GradientInterpolation::EncodedRgb => {
            let linear = [color.r, color.g, color.b];
            // Already linear in the compositor: re-encode for mixing.
            [
                linear_to_encoded(linear[0]),
                linear_to_encoded(linear[1]),
                linear_to_encoded(linear[2]),
            ]
        }
        GradientInterpolation::Lab => linear_to_lab([color.r, color.g, color.b]),
        GradientInterpolation::Oklab => linear_to_oklab([color.r, color.g, color.b]),
    }
}

fn from_interpolation(mixed: [f32; 3], interpolation: GradientInterpolation) -> [f32; 3] {
    match interpolation {
        GradientInterpolation::LinearRgb => mixed,
        GradientInterpolation::EncodedRgb => [
            encoded_to_linear(mixed[0]),
            encoded_to_linear(mixed[1]),
            encoded_to_linear(mixed[2]),
        ],
        GradientInterpolation::Lab => lab_to_linear(mixed),
        GradientInterpolation::Oklab => oklab_to_linear(mixed),
    }
}

fn encoded_to_linear(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_encoded(channel: f32) -> f32 {
    if channel <= 0.0031308 {
        12.92 * channel
    } else {
        1.055 * channel.powf(1.0 / 2.4) - 0.055
    }
}

const SRGB_TO_XYZ: [[f64; 3]; 3] = [
    [0.4124564, 0.3575761, 0.1804375],
    [0.2126729, 0.7151522, 0.0721750],
    [0.0193339, 0.1191920, 0.9503041],
];

const XYZ_TO_SRGB: [[f64; 3]; 3] = [
    [3.2404542, -1.5371385, -0.4985314],
    [-0.9692660, 1.8760108, 0.0415560],
    [0.0556434, -0.2040259, 1.0572252],
];

fn mat_vec(matrix: [[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    [
        matrix[0][0] * v[0] + matrix[0][1] * v[1] + matrix[0][2] * v[2],
        matrix[1][0] * v[0] + matrix[1][1] * v[1] + matrix[1][2] * v[2],
        matrix[2][0] * v[0] + matrix[2][1] * v[1] + matrix[2][2] * v[2],
    ]
}

fn linear_to_lab(rgb: [f32; 3]) -> [f32; 3] {
    const EPSILON: f64 = 216.0 / 24389.0;
    const KAPPA: f64 = 24389.0 / 27.0;
    let xyz = mat_vec(
        SRGB_TO_XYZ,
        [f64::from(rgb[0]), f64::from(rgb[1]), f64::from(rgb[2])],
    );
    // D65 reference white.
    let white = [0.95047, 1.0, 1.08883];
    let mut f = [0.0; 3];
    for channel in 0..3 {
        let t = xyz[channel] / white[channel];
        f[channel] = if t > EPSILON {
            t.cbrt()
        } else {
            (KAPPA * t + 16.0) / 116.0
        };
    }
    [
        (116.0 * f[1] - 16.0) as f32,
        (500.0 * (f[0] - f[1])) as f32,
        (200.0 * (f[1] - f[2])) as f32,
    ]
}

fn lab_to_linear(lab: [f32; 3]) -> [f32; 3] {
    const EPSILON: f64 = 216.0 / 24389.0;
    const KAPPA: f64 = 24389.0 / 27.0;
    let fy = (f64::from(lab[0]) + 16.0) / 116.0;
    let fx = fy + f64::from(lab[1]) / 500.0;
    let fz = fy - f64::from(lab[2]) / 200.0;
    let white = [0.95047, 1.0, 1.08883];
    let f = [fx, fy, fz];
    let mut xyz = [0.0; 3];
    for channel in 0..3 {
        let cube = f[channel].powi(3);
        xyz[channel] = white[channel]
            * if cube > EPSILON {
                cube
            } else {
                (116.0 * f[channel] - 16.0) / KAPPA
            };
    }
    let rgb = mat_vec(XYZ_TO_SRGB, xyz);
    [rgb[0] as f32, rgb[1] as f32, rgb[2] as f32]
}

const M2: [[f64; 3]; 3] = [
    [0.2104542553, 0.7936177850, -0.0040720468],
    [1.9779984951, -2.4285922050, 0.4505937099],
    [0.0259040371, 0.7827717662, -0.8086757660],
];

const M2_INV: [[f64; 3]; 3] = [
    [1.0, 0.3963377774, 0.2158037573],
    [1.0, -0.1055613458, -0.0638541728],
    [1.0, -0.0894841775, -1.2914855480],
];

/// Direct linear-sRGB to LMS matrix (Ottosson, sRGB gamut).
/// Rows sum to exactly 1 so D65 white maps to unit LMS.
const M1_SRGB: [[f64; 3]; 3] = [
    [0.4122214708, 0.5363325363, 0.0514459929],
    [0.2119034982, 0.6806995451, 0.1073969566],
    [0.0883024619, 0.2817188376, 0.6299787005],
];

/// LMS to linear sRGB (Ottosson, inverse of `M1_SRGB`).
const M1_INV_SRGB: [[f64; 3]; 3] = [
    [4.0767416621, -3.3077115913, 0.2309699292],
    [-1.2684380046, 2.6097574011, -0.3413193965],
    [-0.0041960863, -0.7034186147, 1.7076147010],
];

fn linear_to_oklab(rgb: [f32; 3]) -> [f32; 3] {
    let lms = mat_vec(
        M1_SRGB,
        [f64::from(rgb[0]), f64::from(rgb[1]), f64::from(rgb[2])],
    );
    let lms = [lms[0].cbrt(), lms[1].cbrt(), lms[2].cbrt()];
    let lab = mat_vec(M2, lms);
    [lab[0] as f32, lab[1] as f32, lab[2] as f32]
}

fn oklab_to_linear(lab: [f32; 3]) -> [f32; 3] {
    let lms = mat_vec(
        M2_INV,
        [f64::from(lab[0]), f64::from(lab[1]), f64::from(lab[2])],
    );
    let lms = [lms[0].powi(3), lms[1].powi(3), lms[2].powi(3)];
    let rgb = mat_vec(M1_INV_SRGB, lms);
    [rgb[0] as f32, rgb[1] as f32, rgb[2] as f32]
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_render_model::RenderGradientStop;

    fn two_stop(midpoint: f32) -> RenderGradient {
        RenderGradient {
            stops: vec![
                RenderGradientStop {
                    offset: 0.0,
                    color: RenderColor::BLACK,
                    midpoint,
                },
                RenderGradientStop {
                    offset: 1.0,
                    color: RenderColor::WHITE,
                    midpoint: 0.5,
                },
            ],
            interpolation: GradientInterpolation::LinearRgb,
            spread: GradientSpread::Pad,
            space: petunia_core::paint::PaintSpace::Object,
            start: (0.0, 0.0),
            end: (100.0, 0.0),
            radius: 1.0,
        }
    }

    #[test]
    fn stops_endpoints_and_midpoint_identity() {
        let gradient = two_stop(0.5);
        let at_zero = sample_linear(&gradient, (0.0, 0.0)).expect("samples");
        assert_eq!((at_zero.r, at_zero.g, at_zero.b), (0.0, 0.0, 0.0));
        let at_one = sample_linear(&gradient, (100.0, 0.0)).expect("samples");
        assert_eq!((at_one.r, at_one.g, at_one.b), (1.0, 1.0, 1.0));
        let at_half = sample_linear(&gradient, (50.0, 0.0)).expect("samples");
        assert!((at_half.r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn shifted_midpoint_moves_the_halfway_point() {
        // Midpoint 0.25 pulls the 50% mix toward the start.
        let gradient = two_stop(0.25);
        let early = sample_linear(&gradient, (25.0, 0.0)).expect("samples");
        assert!((early.r - 0.5).abs() < 1e-5, "got {}", early.r);
        let late = sample_linear(&gradient, (75.0, 0.0)).expect("samples");
        assert!(late.r > 0.5 && late.r < 1.0, "got {}", late.r);
    }

    #[test]
    fn singular_midpoints_error_loudly() {
        let mut gradient = two_stop(0.5);
        gradient.stops[0].midpoint = 0.0;
        assert!(sample_linear(&gradient, (50.0, 0.0)).is_err());
    }

    #[test]
    fn repeat_reflect_handle_negative_t() {
        let mut gradient = two_stop(0.5);
        gradient.spread = GradientSpread::Repeat;
        let wrapped = sample_linear(&gradient, (-25.0, 0.0)).expect("samples");
        let plain = sample_linear(&gradient, (75.0, 0.0)).expect("samples");
        assert!((wrapped.r - plain.r).abs() < 1e-6);
        gradient.spread = GradientSpread::Reflect;
        let mirrored = sample_linear(&gradient, (125.0, 0.0)).expect("samples");
        let expected = sample_linear(&gradient, (75.0, 0.0)).expect("samples");
        assert!((mirrored.r - expected.r).abs() < 1e-6);
    }

    #[test]
    fn degenerate_linear_errors() {
        let mut gradient = two_stop(0.5);
        gradient.end = (0.0, 0.0);
        assert!(sample_linear(&gradient, (10.0, 0.0)).is_err());
    }

    #[test]
    fn lab_midpoint_of_black_white_is_mid_gray() {
        let mut gradient = two_stop(0.5);
        gradient.interpolation = GradientInterpolation::Lab;
        let mid = sample_linear(&gradient, (50.0, 0.0)).expect("samples");
        // L must sit at 50 by perceptual construction; a/b near zero.
        let lab = linear_to_lab([mid.r, mid.g, mid.b]);
        assert!((lab[0] - 50.0).abs() < 1.0, "L={}", lab[0]);
        assert!(lab[1].abs() < 1.0 && lab[2].abs() < 1.0);
    }

    #[test]
    fn oklab_matches_independent_implementation() {
        use palette::{convert::FromColorUnclamped, LinSrgb, Oklab};
        // Cross-check every channel against the palette crate across
        // primaries, grays and saturated mixes.
        for rgb in [
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.4, 0.02, 0.7],
            [0.13, 0.55, 0.21],
        ] {
            let expected = Oklab::from_color_unclamped(LinSrgb::new(rgb[0], rgb[1], rgb[2]));
            let got = linear_to_oklab(rgb);
            for (got, want) in [got[0], got[1], got[2]]
                .iter()
                .zip([expected.l, expected.a, expected.b])
            {
                assert!((got - want).abs() < 1e-3, "{rgb:?} got {got:?}");
            }
            let back = oklab_to_linear(got);
            for (got, want) in back.iter().zip(rgb) {
                assert!((got - want).abs() < 1e-3, "{rgb:?} got {back:?}");
            }
        }
    }
}
