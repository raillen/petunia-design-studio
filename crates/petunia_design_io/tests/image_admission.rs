//! Codec admission regression sources; validation is deferred.
use petunia_design_io::{import_raster, RawRasterImage};
use petunia_design_raster::{decode_image, ImageAssetError, ImageDecodeLimits, PixelFormat};

fn png_gamma(gamma: f32) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, 1, 1);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_gamma(png::ScaledFloat::new(gamma));
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&[100, 100, 100, 255])
        .unwrap();
    bytes
}

#[test]
fn declared_non_srgb_png_gamma_has_a_conversion_reason() {
    assert!(matches!(
        decode_image(&png_gamma(1.0), ImageDecodeLimits::default()),
        Err(ImageAssetError::Unsupported(_))
    ));
    assert!(decode_image(&png_gamma(0.45455), ImageDecodeLimits::default()).is_ok());
}

#[test]
fn gray16_png_import_and_tile_adapter_share_little_endian_samples() {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, 1, 1);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Sixteen);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&0x1234_u16.to_be_bytes())
        .unwrap();
    let raw = import_raster(&bytes, 1024 * 1024).unwrap();
    assert_eq!(raw.format, PixelFormat::Rgba16);
    assert_eq!(
        raw.to_rgba16().unwrap(),
        vec![0x1234, 0x1234, 0x1234, 65535]
    );
    let tile_image = RawRasterImage::from_rgba16(128, 128, &[0x1234; 128 * 128 * 4]).unwrap();
    let tile = tile_image
        .to_tile(petunia_design_raster::TileCoord::new(0, 0))
        .unwrap();
    assert_eq!(&tile.data[..2], &[0x34, 0x12]);
    assert_eq!(RawRasterImage::from_tile(&tile), tile_image);
}

#[test]
fn unsupported_magic_and_encoded_quota_fail_before_raw_publication() {
    let bytes = png_gamma(0.45455);
    assert!(import_raster(&bytes, bytes.len() - 1).is_err());
    assert!(import_raster(b"GIF89a", 1024).is_err());
    assert!(import_raster(b"not an image", 1024).is_err());
}

#[test]
fn animated_png_is_not_silently_flattened_to_its_first_frame() {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, 1, 1);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_animated(1, 0).unwrap();
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&[255, 0, 0, 255])
        .unwrap();
    assert!(matches!(
        decode_image(&bytes, ImageDecodeLimits::default()),
        Err(ImageAssetError::Unsupported("animated PNG"))
    ));
}

#[test]
fn premultiplied_tiles_export_straight_alpha_without_darkening() {
    use petunia_design_raster::{AlphaMode, Tile, TileCoord};
    for format in [PixelFormat::Rgba8, PixelFormat::Rgba16] {
        let mut tile = Tile::new_empty(TileCoord::new(0, 0), format, AlphaMode::Premultiplied);
        tile.set_pixel_normalized(0, 0, [1.0, 0.5, 0.0, 0.5])
            .unwrap();
        let raw = RawRasterImage::from_tile(&tile);
        assert_eq!(&raw.to_rgba8().unwrap()[..4], &[255, 128, 0, 128]);
        let straight = raw.to_tile(TileCoord::new(0, 0)).unwrap();
        assert_eq!(straight.alpha_mode, AlphaMode::Straight);
        assert!((straight.get_pixel_normalized(0, 0).unwrap()[0] - 1.0).abs() < 0.0001);
    }
}
