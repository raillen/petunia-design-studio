use petunia_design_document::{Document, DocumentMutator, DocumentObject};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_io::{open_package, save_package, PackageManifest};
use std::io::Write;
use std::sync::{Arc, Barrier};

fn fixture(name: &str) -> Document {
    let mut doc = Document::new();
    let mut m = DocumentMutator::new(&mut doc);
    m.add_surface(SurfaceId::new(1), name).unwrap();
    m.add_object(
        SurfaceId::new(1),
        DocumentObject::new(ObjectId::new(2), name),
    )
    .unwrap();
    doc
}

#[test]
fn concurrent_saves_publish_a_complete_package_without_temporary_collisions() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("concurrent.PTND");
    let barrier = Arc::new(Barrier::new(4));
    let docs: Vec<_> = (0..4).map(|i| fixture(&format!("writer {i}"))).collect();
    std::thread::scope(|scope| {
        for doc in &docs {
            let barrier = barrier.clone();
            let path = &path;
            scope.spawn(move || {
                barrier.wait();
                save_package(doc, path).unwrap();
            });
        }
    });
    assert!(docs.contains(&open_package(&path).unwrap().document));
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn invalid_save_preserves_previous_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("preserved.PTND");
    save_package(&fixture("valid"), &path).unwrap();
    let before = std::fs::read(&path).unwrap();
    // Raw serde decoding is deliberately exercised as an untrusted input:
    // even a caller bypassing Document::from_json cannot save invalid content.
    let mut value = serde_json::to_value(fixture("invalid")).unwrap();
    value["surfaces"][0]["dimensions"] = serde_json::json!([-1.0, 1.0]);
    let invalid: Document = serde_json::from_value(value).unwrap();
    assert!(save_package(&invalid, &path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

fn write_raw_package(path: &std::path::Path, manifest: &[u8], document: &[u8]) {
    let mut zip = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("manifest.json", options).unwrap();
    zip.write_all(manifest).unwrap();
    zip.start_file("document/document.json", options).unwrap();
    zip.write_all(document).unwrap();
    zip.finish().unwrap();
}

#[test]
fn compressed_oversized_manifest_and_schema_mismatch_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("invalid.PTND");
    let doc = fixture("valid").to_json().unwrap();
    write_raw_package(&path, &vec![b' '; 65_537], doc.as_bytes());
    assert!(open_package(&path).is_err());
    let mut manifest = PackageManifest::current();
    manifest.schema_version = 1;
    write_raw_package(
        &path,
        &serde_json::to_vec(&manifest).unwrap(),
        doc.as_bytes(),
    );
    assert!(open_package(&path).is_err());
}

#[test]
fn schema_one_package_opens_and_is_written_as_schema_three() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("migrated.PTND");
    let mut manifest = PackageManifest::current();
    manifest.schema_version = 1;
    let mut value = serde_json::to_value(fixture("old")).unwrap();
    value["schema_version"] = 1.into();
    write_raw_package(
        &path,
        &serde_json::to_vec(&manifest).unwrap(),
        &serde_json::to_vec(&value).unwrap(),
    );
    let opened = open_package(&path).unwrap();
    assert_eq!(opened.document.schema_version(), 3);
    save_package(&opened.document, &path).unwrap();
    assert_eq!(open_package(&path).unwrap().document, opened.document);
}

#[test]
fn schema_two_modifier_package_migrates_and_preserves_editable_parameters() {
    use petunia_design_document::{ModifierSpace, ShapeKind};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("schema-two.PTND");
    let mut doc = fixture("schema two");
    let id = ObjectId::new(2);
    let mut m = DocumentMutator::new(&mut doc);
    m.set_bounds(id, Some([100.0, 200.0, 100.0, 100.0]), 0.0)
        .unwrap();
    m.set_shape(
        id,
        Some(ShapeKind::Rectangle {
            corner_radii: [0.0; 4],
        }),
    )
    .unwrap();
    let mut wire = serde_json::to_value(doc).unwrap();
    wire["schema_version"] = 2.into();
    wire["surfaces"][0]["objects"][0]["modifiers"] = serde_json::json!([{"id":1,"kind":{"type":"CropRect","rect":[110,210,50,30]},"enabled":true}]);
    let mut manifest = PackageManifest::current();
    manifest.schema_version = 2;
    write_raw_package(
        &path,
        &serde_json::to_vec(&manifest).unwrap(),
        &serde_json::to_vec(&wire).unwrap(),
    );
    let opened = open_package(&path).unwrap();
    assert_eq!(opened.document.schema_version(), 3);
    assert!(matches!(
        opened.document.find_object(id).unwrap().modifiers[0].space,
        ModifierSpace::Local {
            reference_size: [100.0, 100.0]
        }
    ));
    save_package(&opened.document, &path).unwrap();
    assert_eq!(open_package(&path).unwrap().document, opened.document);
}
