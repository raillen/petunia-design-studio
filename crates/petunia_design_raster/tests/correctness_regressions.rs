use petunia_design_raster::{AlphaMode, BlendMode, BrushDab, PixelFormat, TileMap};

#[test]
fn eraser_removes_coverage_independently_of_its_color_alpha() {
    for format in [PixelFormat::Rgba8, PixelFormat::Rgba16] {
        let mut tiles = TileMap::new(format, AlphaMode::Straight);
        tiles.set_pixel(0, 0, [1.0, 0.0, 0.0, 1.0]);
        let mut eraser = BrushDab::eraser_dab(0.0, 0.0);
        eraser.opacity = 0.5;
        eraser.stamp_onto(&mut tiles).unwrap();
        let pixel = tiles.get_pixel(0, 0).unwrap();
        assert!((pixel[3] - 0.5).abs() < 0.005);
        assert!((pixel[0] - 1.0).abs() < 0.005);
        eraser.opacity = 1.0;
        eraser.stamp_onto(&mut tiles).unwrap();
        assert_eq!(tiles.get_pixel(0, 0).unwrap(), [0.0; 4]);
    }
}

#[test]
fn photo_blending_preserves_source_over_transparent_backdrops() {
    for mode in [BlendMode::Normal, BlendMode::Multiply, BlendMode::Screen] {
        for alpha in [0.25, 0.5, 1.0] {
            let source = [1.0, 0.0, 0.0, alpha];
            assert_eq!(mode.blend(source, [0.0; 4]), source);
        }
    }
}

#[test]
fn straight_and_premultiplied_storage_have_the_same_blending_result() {
    for format in [PixelFormat::Rgba8, PixelFormat::Rgba16] {
        let mut straight = TileMap::new(format, AlphaMode::Straight);
        let mut premul = TileMap::new(format, AlphaMode::Premultiplied);
        for map in [&mut straight, &mut premul] {
            map.set_pixel(0, 0, [0.8, 0.4, 0.2, 0.5]);
        }
        let mut dab = BrushDab::paint_dab(0.0, 0.0);
        dab.color = [0.2, 0.8, 0.4, 0.6];
        dab.opacity = 0.5;
        dab.stamp_onto(&mut straight).unwrap();
        dab.stamp_onto(&mut premul).unwrap();
        for (a, b) in straight
            .get_pixel(0, 0)
            .unwrap()
            .into_iter()
            .zip(premul.get_pixel(0, 0).unwrap())
        {
            assert!((a - b).abs() < 0.015);
        }
        let mut eraser = BrushDab::eraser_dab(0.0, 0.0);
        eraser.opacity = 0.5;
        eraser.stamp_onto(&mut straight).unwrap();
        eraser.stamp_onto(&mut premul).unwrap();
        for (a, b) in straight
            .get_pixel(0, 0)
            .unwrap()
            .into_iter()
            .zip(premul.get_pixel(0, 0).unwrap())
        {
            assert!((a - b).abs() < 0.015);
        }
    }
}

#[test]
fn sixteen_bit_channels_are_little_endian_and_zero_alpha_has_no_hidden_premul_color() {
    let mut tile = petunia_design_raster::Tile::new_empty(
        petunia_design_raster::TileCoord::new(0, 0),
        PixelFormat::Rgba16,
        AlphaMode::Straight,
    );
    tile.set_pixel_normalized(0, 0, [f32::from(0x1234_u16) / 65535.0, 0.0, 0.0, 1.0])
        .unwrap();
    assert_eq!(&tile.data[..2], &[0x34, 0x12]);
    tile.alpha_mode = AlphaMode::Premultiplied;
    tile.set_pixel_normalized(0, 0, [1.0, 1.0, 1.0, 0.0])
        .unwrap();
    assert_eq!(&tile.data[..8], &[0; 8]);
    assert_eq!(tile.get_pixel_normalized(0, 0).unwrap(), [0.0; 4]);
}

#[test]
fn invalid_or_unbounded_dabs_and_coordinate_wrap_cannot_modify_tiles() {
    let mut tiles = TileMap::new(PixelFormat::Rgba8, AlphaMode::Straight);
    tiles.set_pixel(0, 0, [1.0, 0.0, 0.0, 1.0]);
    let expected = tiles.clone();
    for radius in [f64::NAN, f64::INFINITY, 1e20] {
        let mut dab = BrushDab::paint_dab(0.0, 0.0);
        dab.radius = radius;
        assert!(dab.stamp_onto(&mut tiles).is_err());
        assert_eq!(tiles, expected);
    }
    assert!(!tiles.set_pixel(i64::MAX, 0, [0.0, 1.0, 0.0, 1.0]));
    assert_eq!(tiles.get_pixel(i64::MAX, 0).unwrap(), [0.0; 4]);
    assert_eq!(tiles, expected);
}
