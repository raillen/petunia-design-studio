use petunia_design_io::{export_png_rgba8_at_dpi, RawRasterImage};
use std::io::Cursor;

#[test]
fn png_records_srgb_and_physical_density_without_changing_pixel_dimensions() {
    let image = RawRasterImage::from_rgba8(2, 1, vec![255, 0, 0, 128, 0, 0, 255, 255]).unwrap();
    let bytes = export_png_rgba8_at_dpi(&image, 300.0).unwrap();
    let reader = png::Decoder::new(Cursor::new(bytes)).read_info().unwrap();
    let info = reader.info();
    let density = info.pixel_dims.expect("pHYs metadata");
    assert_eq!((info.width, info.height), (2, 1));
    assert_eq!(
        (density.xppu, density.yppu, density.unit),
        (11811, 11811, png::Unit::Meter)
    );
    assert!(info.srgb.is_some());
}

#[test]
fn png_rejects_invalid_density_and_corrupt_pixel_buffer() {
    let mut image = RawRasterImage::from_rgba8(1, 1, vec![255, 0, 0, 255]).unwrap();
    for dpi in [0.0, -1.0, f64::NAN, f64::INFINITY, 1e20] {
        assert!(export_png_rgba8_at_dpi(&image, dpi).is_err());
    }
    image.data.clear();
    assert!(export_png_rgba8_at_dpi(&image, 72.0).is_err());
}
