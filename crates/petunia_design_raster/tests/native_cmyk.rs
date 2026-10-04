use petunia_design_color::{cmyk_to_rgb, IccProfile, IccTransformOptions};
use petunia_design_raster::{
    AlphaMode, BitDepth, InkChannel, PixelFormat, RasterDisplaySampler, RasterLayer, TileCoord,
    TileMap,
};
use std::sync::Arc;
fn press() -> IccProfile {
    IccProfile::new(
        "Synthetic press".into(),
        Arc::new(include_bytes!("../../../fixtures/color/synthetic-cmyk.icc").to_vec()),
    )
    .unwrap()
}
#[test]
fn black_is_not_alpha_and_hidden_ink_survives_commit() {
    for (format, bytes) in [
        (
            PixelFormat::Cmyka8,
            vec![255, 0, 0, 0, 255, 0, 0, 0, 0, 255, 0, 0, 0, 255, 0],
        ),
        (
            PixelFormat::Cmyka16,
            [
                65535_u16, 0, 0, 0, 65535, 0, 0, 0, 0, 65535, 0, 0, 0, 65535, 0,
            ]
            .into_iter()
            .flat_map(u16::to_le_bytes)
            .collect(),
        ),
    ] {
        let layer = RasterLayer::from_cmyka_bytes(3, 1, format, press(), &bytes).unwrap();
        assert_eq!(layer.cmyka_pixel(0, 0).unwrap(), [1., 0., 0., 0., 1.]);
        assert_eq!(layer.cmyka_pixel(1, 0).unwrap(), [0., 0., 0., 0., 1.]);
        assert_eq!(layer.cmyka_pixel(2, 0).unwrap(), [0., 0., 0., 1., 0.]);
        assert_eq!(layer.cmyka_bytes(&|| false).unwrap(), bytes);
    }
}
#[test]
fn raw_eight_and_sixteen_bit_samples_survive_tile_boundaries() {
    for format in [PixelFormat::Cmyka8, PixelFormat::Cmyka16] {
        let bytes: Vec<_> = (0..129 * 3 * format.bytes_per_pixel())
            .map(|n| ((n * 43 + 17) % 256) as u8)
            .collect();
        let layer = RasterLayer::from_cmyka_bytes(129, 3, format, press(), &bytes).unwrap();
        assert_eq!(layer.cmyka_bytes(&|| false).unwrap(), bytes);
        assert!(layer.validate().is_ok());
        let tile = layer.tiles().get_tile(TileCoord::new(1, 0)).unwrap();
        assert!(
            tile.data[format.bytes_per_pixel()..128 * format.bytes_per_pixel()]
                .iter()
                .all(|v| *v == 0)
        );
    }
}
#[test]
fn rgb_access_and_premultiplication_cannot_reinterpret_native_ink() {
    let mut layer = RasterLayer::new_cmyk(4, 4, BitDepth::Sixteen, press()).unwrap();
    assert!(layer.pixel(0, 0).is_err());
    assert!(layer.set_pixel(0, 0, [1.; 4]).is_err());
    assert!(TileMap::new(PixelFormat::Cmyka8, AlphaMode::Premultiplied)
        .validate()
        .is_err());
    let before = layer.clone();
    assert!(layer.set_cmyka_pixel(0, 0, [f32::NAN; 5]).is_err());
    assert_eq!(before, layer);
    assert!(RasterLayer::new_cmyk(4, 4, BitDepth::Eight, IccProfile::srgb().unwrap()).is_err());
}
#[test]
fn assignment_and_identity_conversion_share_exact_native_tiles() {
    let mut layer = RasterLayer::new_cmyk(130, 2, BitDepth::Sixteen, press()).unwrap();
    layer
        .set_cmyka_pixel(129, 0, [0.12345, 0.98765, 0.5, 0., 0.23456])
        .unwrap();
    layer.commit();
    let profile =
        IccProfile::new("Assigned name".into(), Arc::new(press().bytes().to_vec())).unwrap();
    let assigned = layer.assign_cmyk_profile(profile.clone()).unwrap();
    let converted = layer
        .convert_cmyk_profile(profile, IccTransformOptions::default(), &|| false)
        .unwrap();
    let coord = TileCoord::new(1, 0);
    for next in [&assigned, &converted] {
        assert!(Arc::ptr_eq(
            &layer.tiles().get_tile(coord).unwrap().data,
            &next.tiles().get_tile(coord).unwrap().data
        ));
        assert_eq!(
            layer.cmyka_bytes(&|| false).unwrap(),
            next.cmyka_bytes(&|| false).unwrap()
        );
    }
}
#[test]
fn conversion_and_cancellation_never_modify_the_source_or_alpha() {
    let mut layer = RasterLayer::new_cmyk(3, 1, BitDepth::Sixteen, press()).unwrap();
    layer
        .set_cmyka_pixel(0, 0, [0.1, 0.2, 0.3, 0.4, 0.12345])
        .unwrap();
    layer.set_cmyka_pixel(1, 0, [1., 0., 0., 0., 0.]).unwrap();
    layer.commit();
    let before = layer.cmyka_bytes(&|| false).unwrap();
    let mut different = press().bytes().to_vec();
    different[80..84].copy_from_slice(b"PTND");
    let different =
        IccProfile::new("Different profile identity".into(), Arc::new(different)).unwrap();
    let converted = layer
        .convert_cmyk_profile(different.clone(), IccTransformOptions::default(), &|| false)
        .unwrap();
    assert_eq!(converted.cmyk_profile().unwrap().id(), different.id());
    for x in 0..3 {
        assert_eq!(
            converted.cmyka_pixel(x, 0).unwrap()[4],
            layer.cmyka_pixel(x, 0).unwrap()[4]
        );
    }
    assert_eq!(layer.cmyka_bytes(&|| false).unwrap(), before);
    assert_eq!(
        layer
            .convert_cmyk_profile(different, IccTransformOptions::default(), &|| true)
            .unwrap_err()
            .code()
            .0,
        "ptnd.cancelled"
    );
}
#[test]
fn display_uses_icc_and_separate_alpha_without_changing_samples() {
    let mut layer = RasterLayer::new_cmyk(2, 1, BitDepth::Sixteen, press()).unwrap();
    layer
        .set_cmyka_pixel(0, 0, [0.23, 0.47, 0.71, 0.11, 0.53])
        .unwrap();
    let p = layer.cmyka_pixel(0, 0).unwrap();
    let expected = cmyk_to_rgb(
        &press(),
        &IccProfile::srgb().unwrap(),
        &[[p[0], p[1], p[2], p[3]]],
        Default::default(),
    )
    .unwrap()[0];
    let before = layer.clone();
    let mut sampler = RasterDisplaySampler::new(&layer).unwrap();
    for _ in 0..3 {
        let actual = sampler.pixel(0, 0).unwrap();
        for i in 0..3 {
            assert!((actual[i] - expected[i]).abs() < 0.00002);
        }
        assert_eq!(actual[3], p[4]);
    }
    assert_eq!(sampler.pixel(2, 0).unwrap(), [0.; 4]);
    assert_eq!(layer, before);
}
#[test]
fn tac_policy_and_layer_separations_do_not_count_alpha_as_black() {
    let mut layer = RasterLayer::new_cmyk(3, 1, BitDepth::Sixteen, press()).unwrap();
    layer.set_cmyka_pixel(0, 0, [1., 1., 1., 0., 0.5]).unwrap();
    layer.set_cmyka_pixel(1, 0, [0., 0., 0., 1., 1.]).unwrap();
    layer.set_cmyka_pixel(2, 0, [1., 1., 1., 1., 0.]).unwrap();
    let report = layer.ink_coverage(Some(250.), &|| false).unwrap();
    assert_eq!(report.maximum_percent, 300.);
    assert_eq!(report.visible_pixels, 2);
    assert_eq!(report.pixels_over_limit, 1);
    assert_eq!(
        layer.ink_coverage(None, &|| false).unwrap().limit_percent,
        None
    );
    assert!(layer.ink_coverage(Some(f32::NAN), &|| false).is_err());
    let cyan = layer.separation(InkChannel::Cyan, &|| false).unwrap();
    let black = layer.separation(InkChannel::Black, &|| false).unwrap();
    assert!((cyan.pixel(0, 0).unwrap()[3] - 0.5).abs() < 0.00002);
    assert_eq!(black.pixel(0, 0).unwrap()[3], 0.);
    assert_eq!(black.pixel(1, 0).unwrap()[3], 1.);
    assert_eq!(black.pixel(2, 0).unwrap()[3], 0.);
    assert_eq!(
        layer
            .separation(InkChannel::Black, &|| true)
            .unwrap_err()
            .code()
            .0,
        "ptnd.cancelled"
    );
}
