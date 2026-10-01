//! Recovery regression sources; execution joins the final MVP gates.
use petunia_design_document::{Document, DocumentMutator};
use petunia_design_foundation::SurfaceId;
use petunia_design_io::{
    open_package,
    recovery::{RecoveryMetadata, RecoveryStore},
    save_package,
};
fn document() -> Document {
    let mut doc = Document::new();
    DocumentMutator::new(&mut doc)
        .add_surface(SurfaceId::new(1), "Page")
        .unwrap();
    doc
}
#[test]
fn recovery_copy_and_metadata_are_one_atomic_native_package() {
    let dir = tempfile::tempdir().unwrap();
    let store = RecoveryStore::new(dir.path().to_path_buf()).unwrap();
    let doc = document();
    let meta = RecoveryMetadata::new("Unsaved".into(), None, 5);
    let key = store.write(&doc, &meta, None).unwrap();
    assert_eq!(
        open_package(key.path()).unwrap().recovery,
        Some(meta.clone())
    );
    assert_eq!(store.restore(&key).unwrap(), doc);
    assert_eq!(store.list().unwrap()[0].metadata, meta);
}
#[test]
fn recovery_writes_never_replace_the_original_project() {
    let dir = tempfile::tempdir().unwrap();
    let original = dir.path().join("original.PTND");
    let doc = document();
    save_package(&doc, &original).unwrap();
    let original_bytes = std::fs::read(&original).unwrap();
    let store = RecoveryStore::new(dir.path().join("recovery")).unwrap();
    let meta = RecoveryMetadata::new("Original".into(), Some(original.clone()), 2);
    store.write(&doc, &meta, None).unwrap();
    assert_eq!(std::fs::read(&original).unwrap(), original_bytes);
}
#[test]
fn failed_recovery_replacement_retains_the_preceding_valid_copy() {
    let dir = tempfile::tempdir().unwrap();
    let store = RecoveryStore::new(dir.path().to_path_buf()).unwrap();
    let doc = document();
    let meta = RecoveryMetadata::new("First".into(), None, 1);
    let key = store.write(&doc, &meta, None).unwrap();
    let before = std::fs::read(key.path()).unwrap();
    let bad = RecoveryMetadata::new("x".repeat(4097), None, 2);
    assert!(store.write(&doc, &bad, Some(&key)).is_err());
    assert_eq!(std::fs::read(key.path()).unwrap(), before);
}
#[test]
fn repeated_snapshots_reuse_one_copy_and_publish_the_latest_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let store = RecoveryStore::new(dir.path().to_path_buf()).unwrap();
    let doc = document();
    let key = store
        .write(&doc, &RecoveryMetadata::new("First".into(), None, 1), None)
        .unwrap();
    let meta = RecoveryMetadata::new("Latest".into(), None, 8);
    let next = store.write(&doc, &meta, Some(&key)).unwrap();
    assert_eq!(key, next);
    assert_eq!(store.list().unwrap().len(), 1);
    assert_eq!(store.list().unwrap()[0].metadata, meta);
}
#[test]
fn foreign_store_keys_cannot_be_written_restored_or_deleted() {
    let dir = tempfile::tempdir().unwrap();
    let a = RecoveryStore::new(dir.path().join("a")).unwrap();
    let b = RecoveryStore::new(dir.path().join("b")).unwrap();
    let doc = document();
    let meta = RecoveryMetadata::new("A".into(), None, 1);
    let key = a.write(&doc, &meta, None).unwrap();
    assert!(b.write(&doc, &meta, Some(&key)).is_err());
    assert!(b.restore(&key).is_err());
    assert!(b.remove(&key).is_err());
    assert_eq!(a.restore(&key).unwrap(), doc);
}
#[test]
fn saved_copy_cleanup_removes_only_the_owned_recovery_package() {
    let dir = tempfile::tempdir().unwrap();
    let store = RecoveryStore::new(dir.path().to_path_buf()).unwrap();
    let original = dir.path().join("original.PTND");
    let doc = document();
    save_package(&doc, &original).unwrap();
    let key = store
        .write(
            &doc,
            &RecoveryMetadata::new("A".into(), Some(original.clone()), 1),
            None,
        )
        .unwrap();
    store.remove(&key).unwrap();
    assert!(original.exists());
    assert!(store.list().unwrap().is_empty());
}
