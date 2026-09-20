use aubrieta_io::image_io::{
    export_raster, import_raster, RasterExportOptions, RasterFormat, RawRasterImage,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn arbitrary_8bit_rgba_roundtrips_through_png(
        r in 0u8..=255,
        g in 0u8..=255,
        b in 0u8..=255,
        a in 0u8..=255,
    ) {
        let pixels = vec![r, g, b, a, r, g, b, a, r, g, b, a, r, g, b, a];
        let raw = RawRasterImage::from_rgba8(2, 2, pixels.clone()).unwrap();

        let options = RasterExportOptions {
            format: RasterFormat::Png,
            ..Default::default()
        };

        let (encoded, degradations) = export_raster(&raw, &options).expect("PNG export failed");
        prop_assert!(degradations.is_empty());

        let imported = import_raster(&encoded, 1024 * 1024).expect("PNG import failed");
        prop_assert_eq!(imported.width, 2);
        prop_assert_eq!(imported.height, 2);
        prop_assert_eq!(imported.data, pixels);
    }

    #[test]
    fn magic_sniffing_identifies_png_and_jpeg_reliably(
        r in 0u8..=255,
        g in 0u8..=255,
        b in 0u8..=255,
    ) {
        let pixels = vec![r, g, b, 255, r, g, b, 255];
        let raw = RawRasterImage::from_rgba8(2, 1, pixels).unwrap();

        // Test PNG sniffing
        let (png_bytes, _) = export_raster(&raw, &RasterExportOptions {
            format: RasterFormat::Png,
            ..Default::default()
        }).unwrap();
        prop_assert_eq!(RasterFormat::from_magic(&png_bytes), Some(RasterFormat::Png));

        // Test JPEG sniffing
        let (jpeg_bytes, _) = export_raster(&raw, &RasterExportOptions {
            format: RasterFormat::Jpeg,
            ..Default::default()
        }).unwrap();
        prop_assert_eq!(RasterFormat::from_magic(&jpeg_bytes), Some(RasterFormat::Jpeg));
    }
}
