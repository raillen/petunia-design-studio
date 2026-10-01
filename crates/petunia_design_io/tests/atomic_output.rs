//! Prepared regression sources; final MVP gates own execution.
use petunia_design_io::atomic_output::{write_atomic, OutputLease};
#[test]
fn competing_writer_is_rejected_until_the_same_inode_is_unlocked() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("out.svg");
    let lease = OutputLease::acquire(&path).unwrap();
    assert!(OutputLease::acquire(&path).is_err());
    drop(lease);
    write_atomic(&path, b"first", &|| false).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"first");
    write_atomic(&path, b"second", &|| false).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"second");
}
#[test]
fn cancellation_preserves_the_existing_artifact() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("out.png");
    std::fs::write(&path, b"original").unwrap();
    assert!(write_atomic(&path, b"replacement", &|| true).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"original");
}
#[cfg(unix)]
#[test]
fn symlink_output_is_rejected_without_touching_its_target() {
    let directory = tempfile::tempdir().unwrap();
    let original = directory.path().join("original");
    let path = directory.path().join("out");
    std::fs::write(&original, b"original").unwrap();
    std::os::unix::fs::symlink(&original, &path).unwrap();
    assert!(write_atomic(&path, b"replacement", &|| false).is_err());
    assert_eq!(std::fs::read(&original).unwrap(), b"original");
}
