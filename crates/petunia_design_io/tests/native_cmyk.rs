use petunia_design_color::IccProfile;
use petunia_design_document::{Document, DocumentMutator, DocumentObject, ShapeKind};
use petunia_design_foundation::{ObjectId, SurfaceId, NATIVE_SCHEMA_VERSION};
use petunia_design_io::{
    export_cmyk_tiff, export_document_pdf, import_cmyk_tiff, open_package, save_package,
    write_cmyk_tiff, PdfExportOptions,
};
use petunia_design_raster::{
    decode_image, EncodedImage, ImageDecodeLimits, PixelFormat, RasterLayer,
};
use std::{
    fs::File,
    io::{Cursor, Read, Write},
    process::Command,
    sync::Arc,
};
use tiff::{
    encoder::{
        colortype::{CMYK8, CMYKA8},
        TiffEncoder,
    },
    tags::Tag,
};
fn press() -> IccProfile {
    IccProfile::new(
        "Synthetic press".into(),
        Arc::new(include_bytes!("../../../fixtures/color/synthetic-cmyk.icc").to_vec()),
    )
    .unwrap()
}
fn layer(format: PixelFormat) -> RasterLayer {
    let bytes: Vec<u8> = if format == PixelFormat::Cmyka8 {
        vec![255, 0, 0, 0, 255, 0, 0, 0, 200, 127, 1, 2, 3, 4, 0]
    } else {
        [
            65535_u16, 1, 257, 0, 65535, 0, 0, 0, 0x1234, 0x4567, 123, 456, 789, 1234, 0,
        ]
        .into_iter()
        .flat_map(u16::to_le_bytes)
        .collect()
    };
    RasterLayer::from_cmyka_bytes(3, 1, format, press(), &bytes).unwrap()
}
fn document(format: PixelFormat) -> Document {
    document_from_shape(ShapeKind::Raster {
        layer: Arc::new(layer(format)),
    })
}
fn encoded_shape(layer: &RasterLayer) -> ShapeKind {
    ShapeKind::Image {
        path: "native.tif".into(),
        data: Some(Arc::new(
            EncodedImage::new(export_cmyk_tiff(layer, 300., &|| false).unwrap()).unwrap(),
        )),
    }
}
fn document_from_shape(shape: ShapeKind) -> Document {
    let mut doc = Document::new();
    let mut object = DocumentObject::new(ObjectId::new(2), "CMYK");
    object.bounds = Some([0., 0., 3., 1.]);
    object.shape = Some(shape);
    let mut m = DocumentMutator::new(&mut doc);
    m.add_surface(SurfaceId::new(1), "Page").unwrap();
    m.set_surface_geometry(SurfaceId::new(1), [0.; 2], [3., 1.])
        .unwrap();
    m.add_object(SurfaceId::new(1), object).unwrap();
    doc
}
fn rewrite(path: &std::path::Path, change: impl Fn(&str, &mut Vec<u8>)) {
    let mut archive = zip::ZipArchive::new(File::open(path).unwrap()).unwrap();
    let mut files = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).unwrap();
        let name = entry.name().to_string();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        change(&name, &mut bytes);
        files.push((name, bytes));
    }
    drop(archive);
    let mut zip = zip::ZipWriter::new(File::create(path).unwrap());
    for (name, bytes) in files {
        zip.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(&bytes).unwrap();
    }
    zip.finish().unwrap();
}
#[test]
fn native_tiff_roundtrip_preserves_every_ink_alpha_sample_and_icc() {
    for format in [PixelFormat::Cmyka8, PixelFormat::Cmyka16] {
        let layer = layer(format);
        let encoded = export_cmyk_tiff(&layer, 300., &|| false).unwrap();
        let restored = import_cmyk_tiff(&encoded, EncodedImage::MAX_BYTES).unwrap();
        assert_eq!(restored.tiles().format, format);
        assert_eq!(
            restored.cmyka_bytes(&|| false).unwrap(),
            layer.cmyka_bytes(&|| false).unwrap()
        );
        assert_eq!(restored.cmyk_profile().unwrap().bytes(), press().bytes());
        let mut decoder = tiff::decoder::Decoder::new(Cursor::new(encoded)).unwrap();
        for tag in [Tag::XResolution, Tag::YResolution] {
            let tiff::decoder::ifd::Value::Rational(numerator, denominator) =
                decoder.get_tag(tag).unwrap()
            else {
                panic!("TIFF resolution must use a rational value");
            };
            assert_ne!(denominator, 0);
            assert_eq!(f64::from(numerator) / f64::from(denominator), 300.);
        }
        assert_eq!(decoder.get_tag_u32(Tag::ResolutionUnit).unwrap(), 2);
        assert_eq!(
            decoder.find_tag_unsigned::<u16>(Tag::Unknown(332)).unwrap(),
            Some(1)
        );
        assert_eq!(
            decoder.find_tag_unsigned::<u16>(Tag::Unknown(334)).unwrap(),
            Some(4)
        );
        assert_eq!(
            decoder
                .find_tag_unsigned_vec::<u16>(Tag::ExtraSamples)
                .unwrap(),
            Some(vec![2])
        );
    }
}
fn fixture(
    orientation: u16,
    profile: Option<&[u8]>,
    extra: Option<u16>,
    planar: Option<u16>,
) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    {
        let mut encoder = TiffEncoder::new(&mut out).unwrap();
        macro_rules! write {
            ($color:ty, $data:expr) => {{
                let mut image = encoder.new_image::<$color>(2, 3).unwrap();
                image
                    .encoder()
                    .write_tag(Tag::Orientation, orientation)
                    .unwrap();
                if let Some(bytes) = profile {
                    image.encoder().write_tag(Tag::IccProfile, bytes).unwrap();
                }
                if let Some(value) = extra {
                    image
                        .encoder()
                        .write_tag(Tag::ExtraSamples, &[value][..])
                        .unwrap();
                }
                if let Some(value) = planar {
                    image
                        .encoder()
                        .write_tag(Tag::PlanarConfiguration, value)
                        .unwrap();
                }
                image.write_data(&$data).unwrap();
            }};
        }
        if extra.is_some() {
            write!(
                CMYKA8,
                (1..=6_u8)
                    .flat_map(|v| [v, 0, 0, 0, 255])
                    .collect::<Vec<_>>()
            );
        } else {
            write!(
                CMYK8,
                (1..=6_u8).flat_map(|v| [v, 0, 0, 0]).collect::<Vec<_>>()
            );
        }
    }
    out.into_inner()
}
#[test]
fn all_tiff_orientations_reorder_exact_ink_samples_without_conversion() {
    let cases = [
        (2, 3, [1, 2, 3, 4, 5, 6]),
        (2, 3, [2, 1, 4, 3, 6, 5]),
        (2, 3, [6, 5, 4, 3, 2, 1]),
        (2, 3, [5, 6, 3, 4, 1, 2]),
        (3, 2, [1, 3, 5, 2, 4, 6]),
        (3, 2, [5, 3, 1, 6, 4, 2]),
        (3, 2, [6, 4, 2, 5, 3, 1]),
        (3, 2, [2, 4, 6, 1, 3, 5]),
    ];
    for (index, (w, h, samples)) in cases.into_iter().enumerate() {
        let bytes = fixture(index as u16 + 1, Some(press().bytes()), None, None);
        let image = import_cmyk_tiff(&bytes, EncodedImage::MAX_BYTES).unwrap();
        assert_eq!((image.width(), image.height()), (w, h));
        assert_eq!(
            image.cmyka_bytes(&|| false).unwrap(),
            samples
                .into_iter()
                .flat_map(|v| [v, 0, 0, 0, 255])
                .collect::<Vec<_>>()
        );
    }
}
#[test]
fn missing_wrong_icc_associated_alpha_planar_and_limits_fail_admission() {
    for bytes in [
        fixture(1, None, None, None),
        fixture(1, Some(IccProfile::srgb().unwrap().bytes()), None, None),
        fixture(1, Some(press().bytes()), Some(1), None),
        fixture(1, Some(press().bytes()), None, Some(2)),
    ] {
        assert!(decode_image(&bytes, Default::default()).is_err());
    }
    let bytes = fixture(1, Some(press().bytes()), None, None);
    assert!(decode_image(
        &bytes,
        ImageDecodeLimits {
            max_decoded_bytes: 20,
            ..Default::default()
        }
    )
    .is_err());
    assert!(decode_image(
        &bytes,
        ImageDecodeLimits {
            max_profile_bytes: 32,
            ..Default::default()
        }
    )
    .is_err());
    assert!(import_cmyk_tiff(&bytes, 8).is_err());
}
#[test]
fn native_tiff_cancellation_keeps_the_destination_and_reports_cancellation() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("image.tif");
    std::fs::write(&path, b"original").unwrap();
    let calls = std::cell::Cell::new(0);
    let error = write_cmyk_tiff(&layer(PixelFormat::Cmyka16), &path, 300., &|| {
        calls.set(calls.get() + 1);
        calls.get() > 1
    })
    .unwrap_err();
    assert_eq!(error.code().0, "ptnd.cancelled");
    assert_eq!(std::fs::read(path).unwrap(), b"original");
}
#[test]
fn schema_six_and_resource_index_three_keep_native_ink_and_hashed_profile() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cmyk.PTND");
    let doc = document(PixelFormat::Cmyka16);
    save_package(&doc, &path).unwrap();
    assert_eq!(NATIVE_SCHEMA_VERSION, 6);
    assert_eq!(open_package(&path).unwrap().document, doc);
    let mut archive = zip::ZipArchive::new(File::open(&path).unwrap()).unwrap();
    let mut text = String::new();
    archive
        .by_name("document/document.json")
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    assert!(!text.contains("\"bytes\""));
    assert!(text.contains("Cmyka16"));
    let index: serde_json::Value =
        serde_json::from_reader(archive.by_name("resources/index.json").unwrap()).unwrap();
    assert_eq!(index["version"], 3);
    assert!(index["bindings"][0]["profile"]["asset"].is_string());
}
#[test]
fn native_profiles_cannot_be_corrupted_or_relabelled_as_old_schema() {
    let directory = tempfile::tempdir().unwrap();
    for change in 0..3 {
        let path = directory.path().join(format!("bad-{change}.PTND"));
        save_package(&document(PixelFormat::Cmyka8), &path).unwrap();
        rewrite(&path, |name, bytes| {
            if change == 0 && name.ends_with(".bin") && bytes.get(36..40) == Some(&b"acsp"[..]) {
                bytes[0] ^= 1;
            }
            if change == 1 && name == "resources/index.json" {
                let mut value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
                value["version"] = 2.into();
                *bytes = serde_json::to_vec(&value).unwrap();
            }
            if change == 2 && (name == "manifest.json" || name == "document/document.json") {
                let mut value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
                value["schema_version"] = 5.into();
                *bytes = serde_json::to_vec(&value).unwrap();
            }
        });
        assert!(
            open_package(&path).is_err(),
            "case {change} must reject inconsistent ink resources"
        );
    }
}
#[test]
fn native_pdf_requires_one_common_profile_across_surfaces_layers_and_images() {
    let mut bytes = press().bytes().to_vec();
    bytes[80..84].copy_from_slice(b"PTND");
    let profile = IccProfile::new("Different press".into(), Arc::new(bytes)).unwrap();
    let different = layer(PixelFormat::Cmyka16)
        .assign_cmyk_profile(profile.clone())
        .unwrap();
    for source in 0..3 {
        let mut doc = document(PixelFormat::Cmyka8);
        let mut mutator = DocumentMutator::new(&mut doc);
        if source == 0 {
            mutator
                .set_surface_cmyk_profile(SurfaceId::new(1), Some(profile.clone()))
                .unwrap();
        } else {
            let mut object = DocumentObject::new(ObjectId::new(3), "Different ICC");
            object.bounds = Some([0., 0., 3., 1.]);
            object.shape = Some(if source == 1 {
                ShapeKind::Raster {
                    layer: Arc::new(different.clone()),
                }
            } else {
                encoded_shape(&different)
            });
            mutator.add_object(SurfaceId::new(1), object).unwrap();
        }
        let before = doc.clone();
        let error = export_document_pdf(&doc, &PdfExportOptions::default()).unwrap_err();
        assert!(error.to_string().contains("different press profiles"));
        assert_eq!(doc, before);
    }
}
#[test]
fn native_pdf_is_independently_readable_as_four_channel_icc_with_soft_mask() {
    for format in [PixelFormat::Cmyka8, PixelFormat::Cmyka16] {
        for encoded in [false, true] {
            let doc = if encoded {
                document_from_shape(encoded_shape(&layer(format)))
            } else {
                document(format)
            };
            let (bytes, report) = export_document_pdf(&doc, &PdfExportOptions::default()).unwrap();
            assert!(report.passed);
            assert!(report.degradations.is_empty());
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("native.pdf");
            std::fs::write(&path, &bytes).unwrap();
            let result = Command::new("pdfimages")
                .arg("-list")
                .arg(&path)
                .output()
                .expect("poppler-utils required");
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            let listing = String::from_utf8_lossy(&result.stdout);
            let channels = listing
                .lines()
                .filter(|s| s.contains("image") && !s.contains("smask"))
                .any(|line| {
                    let fields: Vec<_> = line.split_whitespace().collect();
                    fields.len() > 7
                        && matches!(fields[5], "icc" | "cmyk")
                        && fields[6] == "4"
                        && fields[7]
                            == if format == PixelFormat::Cmyka16 {
                                "16"
                            } else {
                                "8"
                            }
                });
            assert!(String::from_utf8_lossy(&bytes).contains("/ICCBased"));
            assert!(channels, "{listing}");
            assert!(listing.contains("smask"), "{listing}");
            if let Some(directory) = std::env::var_os("PETUNIA_CMYK_EVIDENCE_DIR") {
                let directory = std::path::PathBuf::from(directory);
                std::fs::create_dir_all(&directory).unwrap();
                let prefix = if encoded { "image" } else { "native" };
                std::fs::write(directory.join(format!("{prefix}-{format:?}.pdf")), bytes).unwrap();
                std::fs::write(
                    directory.join(format!("native-{format:?}.tif")),
                    export_cmyk_tiff(&layer(format), 300., &|| false).unwrap(),
                )
                .unwrap();
            }
        }
    }
}
