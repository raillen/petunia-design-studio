//! Profile conversion sources; no run before all MVP features are implemented.
use petunia_design_color::rgb_profiles::convert_rgba_to_srgb;
#[test]
fn srgb_profile_preserves_coverage_and_nearly_preserves_rgb8() {
    let bytes = moxcms::ColorProfile::new_srgb().encode().unwrap();
    let original = [240u8, 30, 120, 17, 1, 200, 2, 0];
    let mut pixels = original;
    convert_rgba_to_srgb(&mut pixels, false, 2, &bytes, &|| false).unwrap();
    for i in [0, 1, 2, 4, 5, 6] {
        assert!(pixels[i].abs_diff(original[i]) <= 1);
    }
    assert_eq!(pixels[3], 17);
    assert_eq!(pixels[7], 0);
}
#[test]
fn rgba16_profile_conversion_keeps_deep_samples_and_alpha() {
    let bytes = moxcms::ColorProfile::new_srgb().encode().unwrap();
    let samples = [12345u16, 54321, 30000, 1537];
    let mut pixels: Vec<_> = samples.iter().flat_map(|n| n.to_le_bytes()).collect();
    convert_rgba_to_srgb(&mut pixels, true, 1, &bytes, &|| false).unwrap();
    let result: Vec<_> = pixels
        .as_chunks::<2>()
        .0
        .iter()
        .map(|v| u16::from_le_bytes([v[0], v[1]]))
        .collect();
    assert_eq!(result[3], 1537);
    for i in 0..3 {
        assert!(result[i].abs_diff(samples[i]) < 32);
    }
}
#[test]
fn invalid_profile_or_cancelled_transform_never_publishes_a_derived_buffer() {
    let mut pixels = [1u8, 2, 3, 4];
    let original = pixels;
    assert!(convert_rgba_to_srgb(&mut pixels, false, 1, b"invalid", &|| false).is_err());
    assert_eq!(pixels, original);
    let bytes = moxcms::ColorProfile::new_srgb().encode().unwrap();
    assert!(convert_rgba_to_srgb(&mut pixels, false, 1, &bytes, &|| true).is_err());
    assert_eq!(pixels, original);
}
