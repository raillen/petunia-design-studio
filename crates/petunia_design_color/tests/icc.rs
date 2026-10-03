use petunia_design_color::{
    cmyk_to_cmyk, cmyk_to_rgb, parse_cmyk_token, IccProfile, IccProofSettings, IccTransformOptions,
};
use std::sync::Arc;
fn press() -> IccProfile {
    IccProfile::new(
        "Synthetic CMYK test profile".into(),
        Arc::new(include_bytes!("../../../fixtures/color/synthetic-cmyk.icc").to_vec()),
    )
    .unwrap()
}
#[test]
fn profile_identity_is_content_addressed_and_serde_cannot_bypass_admission() {
    let profile = press();
    assert!(profile.is_press_profile());
    assert!(!profile.is_monitor_profile());
    let renamed = IccProfile::new("Renamed".into(), Arc::new(profile.bytes().to_vec())).unwrap();
    assert_eq!(profile.id(), renamed.id());
    let encoded = serde_json::to_vec(&profile).unwrap();
    let decoded: IccProfile = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(profile, decoded);
    let mut malformed: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    malformed["bytes"][36] = 0.into();
    assert!(serde_json::from_value::<IccProfile>(malformed).is_err());
}
#[test]
fn truncated_signature_size_version_and_tag_ranges_are_rejected() {
    let profile = press();
    for (at, value) in [(0, 255), (8, 9), (36, 0), (136, 255), (140, 255)] {
        let mut bytes = profile.bytes().to_vec();
        bytes[at] = value;
        assert!(IccProfile::new("Bad".into(), Arc::new(bytes)).is_err());
    }
    for length in [0, 40, 128, 131] {
        assert!(IccProfile::new(
            "Truncated".into(),
            Arc::new(profile.bytes()[..length].to_vec())
        )
        .is_err());
    }
}
#[test]
fn native_cmyk_transform_uses_percentage_units_and_preserves_identical_profile_inks() {
    let profile = press();
    let rgb = IccProfile::srgb().unwrap();
    let options = IccTransformOptions::default();
    let inks = [
        [0., 0., 0., 0.],
        [0., 0., 0., 1.],
        [1., 0., 0., 0.],
        [0.2, 0.4, 0.6, 0.8],
    ];
    let mapped = cmyk_to_rgb(&profile, &rgb, &inks, options).unwrap();
    assert!(mapped[0].iter().all(|v| *v > 0.9), "{:?}", mapped);
    assert!(mapped[1].iter().all(|v| *v < 0.1), "{:?}", mapped);
    assert!(mapped[2][0] < mapped[2][1] && mapped[2][0] < mapped[2][2]);
    assert_eq!(
        cmyk_to_cmyk(&profile, &profile, &inks, options).unwrap(),
        inks
    );
    assert!(cmyk_to_rgb(&rgb, &profile, &inks, options).is_err());
    assert!(cmyk_to_rgb(&profile, &rgb, &[[f32::NAN, 0., 0., 0.]], options).is_err());
}
#[test]
fn proof_is_derived_preserves_alpha_and_obeys_cancellation() {
    let profile = press();
    let original = profile.bytes().to_vec();
    let monitor = IccProfile::srgb().unwrap();
    assert!(monitor.is_monitor_profile());
    let settings = IccProofSettings {
        proof: profile.clone(),
        monitor,
        options: Default::default(),
        proof_intent: Default::default(),
    };
    let mut pixels = vec![255, 80, 30, 128, 12, 120, 240, 0, 220, 40, 10, 255];
    settings.apply_rgba8(&mut pixels, &|| false).unwrap();
    assert_eq!([pixels[3], pixels[7], pixels[11]], [128, 0, 255]);
    assert_eq!(profile.bytes(), original);
    let baseline = pixels.clone();
    assert!(settings.apply_rgba8(&mut pixels, &|| true).is_err());
    assert_eq!(pixels, baseline);
    assert!(settings.apply_rgba8(&mut pixels[..3], &|| false).is_err());
}
#[test]
fn cmyk_syntax_is_unambiguous_bounded_and_finite() {
    assert_eq!(
        parse_cmyk_token("CMYK(100%, 0%, 25%, 50%)").unwrap(),
        Some([1., 0., 0.25, 0.5])
    );
    assert_eq!(
        parse_cmyk_token("cmyk(1,0,0.25,0.5)").unwrap(),
        Some([1., 0., 0.25, 0.5])
    );
    assert_eq!(parse_cmyk_token("#ff00ff").unwrap(), None);
    for bad in [
        "cmyk(100,0,0,0)",
        "cmyk(NaN,0,0,0)",
        "cmyk(-0.1,0,0,0)",
        "cmyk(0,0,0)",
        "cmyk(0,0,0,0,0)",
        "cmyk(0,0,0,0",
    ] {
        assert!(parse_cmyk_token(bad).is_err(), "{bad}");
    }
}
