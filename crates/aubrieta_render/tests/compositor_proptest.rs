use aubrieta_render::blend::BlendMode;
use proptest::prelude::*;

const ALL_BLEND_MODES: &[BlendMode] = &[
    BlendMode::Normal,
    BlendMode::Multiply,
    BlendMode::Screen,
    BlendMode::Overlay,
    BlendMode::Darken,
    BlendMode::Lighten,
    BlendMode::ColorDodge,
    BlendMode::ColorBurn,
    BlendMode::HardLight,
    BlendMode::SoftLight,
    BlendMode::Difference,
    BlendMode::Exclusion,
    BlendMode::Hue,
    BlendMode::Saturation,
    BlendMode::Color,
    BlendMode::Luminosity,
];

proptest! {
    #[test]
    fn composite_pixel_always_within_bounds_and_finite(
        mode_idx in 0..16usize,
        cb_r in -2.0f32..2.0f32,
        cb_g in -2.0f32..2.0f32,
        cb_b in -2.0f32..2.0f32,
        cb_a in -1.0f32..2.0f32,
        cs_r in -2.0f32..2.0f32,
        cs_g in -2.0f32..2.0f32,
        cs_b in -2.0f32..2.0f32,
        cs_a in -1.0f32..2.0f32,
    ) {
        let mode = ALL_BLEND_MODES[mode_idx];
        let cb = [cb_r, cb_g, cb_b, cb_a];
        let cs = [cs_r, cs_g, cs_b, cs_a];

        let out = mode.composite_pixel_straight(cb, cs);

        for (i, &val) in out.iter().enumerate() {
            prop_assert!(val.is_finite(), "Output channel {i} was not finite: {val}");
            prop_assert!((0.0..=1.0).contains(&val), "Output channel {i} out of bounds: {val}");
        }
    }

    #[test]
    fn transparent_backdrop_passes_source(
        mode_idx in 0..16usize,
        cs_r in 0.0f32..1.0f32,
        cs_g in 0.0f32..1.0f32,
        cs_b in 0.0f32..1.0f32,
        cs_a in 0.0f32..1.0f32,
    ) {
        let mode = ALL_BLEND_MODES[mode_idx];
        let cb = [0.5, 0.5, 0.5, 0.0];
        let cs = [cs_r, cs_g, cs_b, cs_a];

        let out = mode.composite_pixel_straight(cb, cs);

        prop_assert!((out[0] - cs_r).abs() < 1e-5);
        prop_assert!((out[1] - cs_g).abs() < 1e-5);
        prop_assert!((out[2] - cs_b).abs() < 1e-5);
        prop_assert!((out[3] - cs_a).abs() < 1e-5);
    }

    #[test]
    fn transparent_source_leaves_backdrop(
        mode_idx in 0..16usize,
        cb_r in 0.0f32..1.0f32,
        cb_g in 0.0f32..1.0f32,
        cb_b in 0.0f32..1.0f32,
        cb_a in 0.0f32..1.0f32,
    ) {
        let mode = ALL_BLEND_MODES[mode_idx];
        let cb = [cb_r, cb_g, cb_b, cb_a];
        let cs = [0.2, 0.3, 0.4, 0.0];

        let out = mode.composite_pixel_straight(cb, cs);

        prop_assert!((out[0] - cb_r).abs() < 1e-5);
        prop_assert!((out[1] - cb_g).abs() < 1e-5);
        prop_assert!((out[2] - cb_b).abs() < 1e-5);
        prop_assert!((out[3] - cb_a).abs() < 1e-5);
    }
}
