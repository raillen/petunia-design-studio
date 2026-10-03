//! Lossless native process-ink TIFF interchange. This exports one authored
//! pixel layer; it never silently bakes a page's effects/masks or RGB preview.
use petunia_design_color::IccProfile;
use petunia_design_foundation::PetuniaError;
use petunia_design_raster::{
    decode_image, EncodedImage, ImageDecodeLimits, PixelFormat, RasterLayer,
};
use std::{
    io::{self, Cursor, Seek, SeekFrom, Write},
    path::Path,
};
use tiff::{
    encoder::{
        colortype::{ColorType, CMYKA8},
        Compression, DeflateLevel, Rational, TiffEncoder,
    },
    tags::{PhotometricInterpretation, SampleFormat, Tag},
};

struct Cmyka16;
impl ColorType for Cmyka16 {
    type Inner = u16;
    const TIFF_VALUE: PhotometricInterpretation = PhotometricInterpretation::CMYK;
    const BITS_PER_SAMPLE: &'static [u16] = &[16; 5];
    const SAMPLE_FORMAT: &'static [SampleFormat] = &[SampleFormat::Uint; 5];
    fn horizontal_predict(row: &[u16], output: &mut Vec<u16>) {
        output.extend_from_slice(&row[..row.len().min(5)]);
        for index in 5..row.len() {
            output.push(row[index].wrapping_sub(row[index - 5]));
        }
    }
}
fn invalid(message: impl Into<String>) -> PetuniaError {
    PetuniaError::invalid_input(message)
}
fn check(cancelled: &dyn Fn() -> bool) -> Result<(), PetuniaError> {
    if cancelled() {
        Err(PetuniaError::cancelled("CMYK TIFF operation cancelled"))
    } else {
        Ok(())
    }
}
pub fn import_cmyk_tiff(
    bytes: &[u8],
    max_encoded_bytes: usize,
) -> Result<RasterLayer, PetuniaError> {
    if !matches!(image::guess_format(bytes), Ok(image::ImageFormat::Tiff)) {
        return Err(invalid("native CMYK import requires TIFF"));
    }
    let decoded = decode_image(
        bytes,
        ImageDecodeLimits {
            max_encoded_bytes,
            ..Default::default()
        },
    )
    .map_err(|e| invalid(e.to_string()))?;
    if !decoded.format.is_cmyk() {
        return Err(invalid("TIFF source is not native CMYK"));
    }
    let profile = IccProfile::new(
        "Embedded CMYK TIFF profile".into(),
        decoded
            .icc_profile
            .ok_or_else(|| invalid("CMYK TIFF ICC profile missing"))?,
    )?;
    RasterLayer::from_cmyka_bytes(
        decoded.width,
        decoded.height,
        decoded.format,
        profile,
        &decoded.data,
    )
}

struct BoundedOutput<'a> {
    buffer: Cursor<Vec<u8>>,
    cancelled: &'a dyn Fn() -> bool,
}
impl Write for BoundedOutput<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if (self.cancelled)() {
            return Err(io::Error::other("CMYK TIFF cancelled"));
        }
        if self
            .buffer
            .position()
            .checked_add(bytes.len() as u64)
            .is_none_or(|n| n > EncodedImage::MAX_BYTES as u64)
        {
            return Err(io::Error::other("CMYK TIFF encoded byte budget exceeded"));
        }
        self.buffer.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.buffer.flush()
    }
}
impl Seek for BoundedOutput<'_> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        if (self.cancelled)() {
            return Err(io::Error::other("CMYK TIFF cancelled"));
        }
        let next = self.buffer.seek(position)?;
        if next > EncodedImage::MAX_BYTES as u64 {
            return Err(io::Error::other("CMYK TIFF seek budget exceeded"));
        }
        Ok(next)
    }
}
fn encode<C: ColorType>(
    layer: &RasterLayer,
    samples: &[C::Inner],
    dpi: f64,
    output: &mut BoundedOutput<'_>,
) -> Result<(), PetuniaError>
where
    [C::Inner]: tiff::encoder::TiffValue,
{
    let mut encoder = TiffEncoder::new(output)
        .map_err(|e| invalid(e.to_string()))?
        .with_compression(Compression::Deflate(DeflateLevel::Balanced));
    let mut image = encoder
        .new_image::<C>(layer.width(), layer.height())
        .map_err(|e| invalid(e.to_string()))?;
    image
        .rows_per_strip(64)
        .map_err(|e| invalid(e.to_string()))?;
    let profile = layer
        .cmyk_profile()
        .ok_or_else(|| invalid("CMYK TIFF output profile missing"))?;
    let dpi = Rational {
        n: (dpi * 10000.).round() as u32,
        d: 10000,
    };
    for (tag, value) in [
        (Tag::Unknown(332), 1_u16),
        (Tag::Unknown(334), 4),
        (Tag::Orientation, 1),
        (Tag::ResolutionUnit, 2),
    ] {
        image
            .encoder()
            .write_tag(tag, value)
            .map_err(|e| invalid(e.to_string()))?;
    }
    image
        .encoder()
        .write_tag(Tag::ExtraSamples, &[2_u16][..])
        .map_err(|e| invalid(e.to_string()))?;
    image
        .encoder()
        .write_tag(Tag::IccProfile, profile.bytes())
        .map_err(|e| invalid(e.to_string()))?;
    image
        .encoder()
        .write_tag(Tag::XResolution, &dpi)
        .map_err(|e| invalid(e.to_string()))?;
    image
        .encoder()
        .write_tag(Tag::YResolution, &dpi)
        .map_err(|e| invalid(e.to_string()))?;
    // write_data initializes the configured compressor and writes bounded
    // strips. write_strip alone leaves the writer uncompressed in this codec.
    image
        .write_data(samples)
        .map_err(|e| invalid(e.to_string()))
}
pub fn export_cmyk_tiff(
    layer: &RasterLayer,
    dpi: f64,
    cancelled: &dyn Fn() -> bool,
) -> Result<Vec<u8>, PetuniaError> {
    check(cancelled)?;
    if !dpi.is_finite() || !(1.0..=10000.).contains(&dpi) {
        return Err(invalid("CMYK TIFF DPI must be 1–10000"));
    }
    let data = layer.cmyka_bytes(cancelled)?;
    let mut output = BoundedOutput {
        buffer: Cursor::new(Vec::new()),
        cancelled,
    };
    let result = match layer.tiles().format {
        PixelFormat::Cmyka8 => encode::<CMYKA8>(layer, &data, dpi, &mut output),
        PixelFormat::Cmyka16 => {
            let mut samples = Vec::new();
            samples
                .try_reserve_exact(data.len() / 2)
                .map_err(|_| invalid("CMYK TIFF sample allocation"))?;
            for p in data.as_chunks::<2>().0 {
                samples.push(u16::from_le_bytes([p[0], p[1]]));
            }
            encode::<Cmyka16>(layer, &samples, dpi, &mut output)
        }
        _ => return Err(invalid("native CMYK TIFF export requires CMYKA")),
    };
    check(cancelled)?;
    result?;
    Ok(output.buffer.into_inner())
}
pub fn write_cmyk_tiff(
    layer: &RasterLayer,
    path: &Path,
    dpi: f64,
    cancelled: &dyn Fn() -> bool,
) -> Result<(), PetuniaError> {
    let lease = crate::atomic_output::OutputLease::acquire(path)?;
    let mut temporary = lease.temporary()?;
    let bytes = export_cmyk_tiff(layer, dpi, cancelled)?;
    check(cancelled)?;
    temporary
        .write_all(&bytes)
        .map_err(|e| PetuniaError::io(e.to_string()))?;
    lease.publish(temporary, cancelled)
}
