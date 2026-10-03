//! Schema/resource regression sources; no execution during MVP implementation.
use petunia_design_document::{Document, DocumentMutator, DocumentObject, ShapeKind};
use petunia_design_foundation::{ObjectId, SurfaceId, NATIVE_SCHEMA_VERSION};
use petunia_design_io::{open_package, save_package};
use petunia_design_raster::{BitDepth, RasterLayer, RasterLayerKind};
use std::{
    fs::File,
    io::{Read, Write},
    sync::Arc,
};
fn document() -> Document {
    let mut layer = RasterLayer::new(129, 3, RasterLayerKind::Pixels, BitDepth::Sixteen).unwrap();
    layer
        .set_pixel(128, 2, [0.12345, 0.5, 0.98765, 0.75])
        .unwrap();
    layer.commit();
    let mut object = DocumentObject::new(ObjectId::new(2), "Pixel");
    object.bounds = Some([10.0, 10.0, 129.0, 3.0]);
    object.shape = Some(ShapeKind::Raster {
        layer: Arc::new(layer),
    });
    let mut document = Document::new();
    let mut m = DocumentMutator::new(&mut document);
    m.add_surface(SurfaceId::new(1), "Page").unwrap();
    m.add_object(SurfaceId::new(1), object).unwrap();
    document
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
    let mut writer = zip::ZipWriter::new(File::create(path).unwrap());
    for (name, bytes) in files {
        writer
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap();
}
#[test]
fn sixteen_bit_binary_resources_roundtrip_without_json_pixel_arrays() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("r.PTND");
    let document = document();
    save_package(&document, &path).unwrap();
    assert_eq!(open_package(&path).unwrap().document, document);
    let mut archive = zip::ZipArchive::new(File::open(&path).unwrap()).unwrap();
    let mut text = String::new();
    archive
        .by_name("document/document.json")
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    assert!(text.contains("\"tiles\": []"));
    assert!(text.len() < 5000);
    assert!(archive.file_names().any(|name| name.ends_with(".bin")));
}
#[test]
fn altered_binary_content_fails_digest_admission() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("r.PTND");
    save_package(&document(), &path).unwrap();
    rewrite(&path, |name, bytes| {
        if name.ends_with(".bin") {
            bytes[0] ^= 1;
        }
    });
    assert!(open_package(&path)
        .unwrap_err()
        .to_string()
        .contains("digest"));
}
#[test]
fn conflicting_raster_descriptor_is_not_silently_repaired() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("r.PTND");
    save_package(&document(), &path).unwrap();
    rewrite(&path, |name, bytes| {
        if name == "resources/index.json" {
            let mut value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
            value["bindings"][0]["width"] = serde_json::json!(128);
            *bytes = serde_json::to_vec(&value).unwrap();
        }
    });
    assert!(open_package(&path).is_err());
}
#[test]
fn schema_four_requires_an_explicit_binary_resource_contract() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("r.PTND");
    save_package(&document(), &path).unwrap();
    rewrite(&path, |name, bytes| {
        if name == "manifest.json" {
            let mut value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
            value["binary_resources"] = serde_json::json!(false);
            *bytes = serde_json::to_vec(&value).unwrap();
        }
    });
    assert_eq!(NATIVE_SCHEMA_VERSION, 5);
    assert!(open_package(&path).is_err());
}
#[test]
fn shared_tiles_are_written_once_and_remain_shared_after_reopening() {
    let mut document = document();
    let mut duplicate = document.find_object(ObjectId::new(2)).unwrap().clone();
    duplicate.id = ObjectId::new(3);
    DocumentMutator::new(&mut document)
        .add_object(SurfaceId::new(1), duplicate)
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("r.PTND");
    save_package(&document, &path).unwrap();
    let archive = zip::ZipArchive::new(File::open(&path).unwrap()).unwrap();
    assert_eq!(
        archive.file_names().filter(|n| n.ends_with(".bin")).count(),
        1
    );
    let restored = open_package(&path).unwrap().document;
    let Some(ShapeKind::Raster { layer: a }) =
        &restored.find_object(ObjectId::new(2)).unwrap().shape
    else {
        unreachable!()
    };
    let Some(ShapeKind::Raster { layer: b }) =
        &restored.find_object(ObjectId::new(3)).unwrap().shape
    else {
        unreachable!()
    };
    assert!(Arc::ptr_eq(
        &a.tiles().tiles().next().unwrap().1.data,
        &b.tiles().tiles().next().unwrap().1.data
    ));
}
#[test]
fn empty_coverage_masks_have_an_explicit_resource_binding() {
    let mut document = Document::new();
    let mut m = DocumentMutator::new(&mut document);
    m.add_surface(SurfaceId::new(1), "Page").unwrap();
    let mut mask = DocumentObject::new(ObjectId::new(2), "Mask");
    mask.bounds = Some([0.0, 0.0, 10.0, 10.0]);
    mask.shape = Some(ShapeKind::Raster {
        layer: Arc::new(RasterLayer::new(10, 10, RasterLayerKind::Mask, BitDepth::Eight).unwrap()),
    });
    m.add_object(SurfaceId::new(1), mask).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("r.PTND");
    save_package(&document, &path).unwrap();
    assert_eq!(open_package(&path).unwrap().document, document);
}

#[test]
fn opaque_sparse_mask_background_and_zero_coverage_binary_tile_roundtrip() {
    let mut mask = RasterLayer::opaque_mask(129, 3, BitDepth::Sixteen).unwrap();
    mask.set_pixel(128, 2, [1., 1., 1., 0.]).unwrap();
    mask.commit();
    let mut document = Document::new();
    let surface = SurfaceId::new(1);
    let mut object = DocumentObject::new(ObjectId::new(2), "Mask");
    object.bounds = Some([0., 0., 129., 3.]);
    object.shape = Some(ShapeKind::Raster {
        layer: Arc::new(mask),
    });
    {
        let mut mutator = DocumentMutator::new(&mut document);
        mutator.add_surface(surface, "Page").unwrap();
        mutator.add_object(surface, object).unwrap();
    }
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("mask.PTND");
    save_package(&document, &path).unwrap();
    let restored = open_package(&path).unwrap().document;
    assert_eq!(restored, document);
    let Some(ShapeKind::Raster { layer }) = &restored.find_object(ObjectId::new(2)).unwrap().shape
    else {
        panic!("mask source missing")
    };
    assert_eq!(layer.pixel(128, 2)[3], 0.);
    assert_eq!(layer.pixel(127, 2)[3], 1.);
}

fn press_profile() -> petunia_design_color::IccProfile {
    petunia_design_color::IccProfile::new(
        "Synthetic press".into(),
        Arc::new(include_bytes!("../../../fixtures/color/synthetic-cmyk.icc").to_vec()),
    )
    .unwrap()
}
#[test]
fn icc_assignment_is_reversible_and_roundtrips_as_a_hashed_binary_resource() {
    let mut doc = document();
    let baseline = doc.clone();
    let profile = press_profile();
    let change = DocumentMutator::new(&mut doc)
        .set_surface_cmyk_profile(SurfaceId::new(1), Some(profile.clone()))
        .unwrap();
    DocumentMutator::new(&mut doc).revert(&change).unwrap();
    assert_eq!(doc, baseline);
    DocumentMutator::new(&mut doc)
        .set_surface_cmyk_profile(SurfaceId::new(1), Some(profile.clone()))
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("icc.PTND");
    save_package(&doc, &path).unwrap();
    assert_eq!(open_package(&path).unwrap().document, doc);
    let mut archive = zip::ZipArchive::new(File::open(&path).unwrap()).unwrap();
    let mut metadata = String::new();
    archive
        .by_name("document/document.json")
        .unwrap()
        .read_to_string(&mut metadata)
        .unwrap();
    assert!(!metadata.contains("cmyk_profile"));
    let mut index = String::new();
    archive
        .by_name("resources/index.json")
        .unwrap()
        .read_to_string(&mut index)
        .unwrap();
    let value: serde_json::Value = serde_json::from_str(&index).unwrap();
    let asset = value["profiles"][0]["asset"].as_str().unwrap();
    let mut bytes = Vec::new();
    archive
        .by_name(&format!("resources/{asset}.bin"))
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    assert_eq!(bytes, profile.bytes());
    drop(archive);
    rewrite(&path, |name, bytes| {
        if name == format!("resources/{asset}.bin") {
            bytes[36] ^= 1;
        }
    });
    assert!(open_package(&path).is_err());
}
#[test]
fn old_schema_four_resource_index_migrates_without_inventing_an_icc_profile() {
    let doc = document();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("v4.PTND");
    save_package(&doc, &path).unwrap();
    rewrite(&path, |name, bytes| {
        if name == "manifest.json" || name == "document/document.json" {
            let mut value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
            value["schema_version"] = 4.into();
            *bytes = serde_json::to_vec(&value).unwrap();
        }
        if name == "resources/index.json" {
            let mut value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
            value["version"] = 1.into();
            value.as_object_mut().unwrap().remove("profiles");
            *bytes = serde_json::to_vec(&value).unwrap();
        }
    });
    let restored = open_package(&path).unwrap().document;
    assert_eq!(restored, doc);
    assert!(restored.surfaces()[0].cmyk_profile.is_none());
}
