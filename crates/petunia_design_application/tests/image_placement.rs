//! Atomic image-placement and persistence regression sources; not executed yet.
use petunia_design_application::{ActionId, ActionRequest, DocumentSession};
use petunia_design_document::{Document, DocumentMutator, ShapeKind};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_io::{export_raster, RasterExportOptions, RawRasterImage};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

struct SourceFile(PathBuf);
impl SourceFile {
    fn new(bytes: &[u8]) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "petunia-image-place-{}-{}.png",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        file.write_all(bytes).unwrap();
        Self(path)
    }
}
impl Drop for SourceFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
fn session() -> DocumentSession {
    let mut document = Document::new();
    DocumentMutator::new(&mut document)
        .add_surface(SurfaceId::new(1), "Page")
        .unwrap();
    DocumentSession::with_document("Images", document)
}
fn png() -> Vec<u8> {
    export_raster(
        &RawRasterImage::from_rgba8(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 255]).unwrap(),
        &RasterExportOptions::default(),
    )
    .unwrap()
    .0
}
fn place(
    session: &mut DocumentSession,
    path: &std::path::Path,
) -> Result<petunia_design_document::ChangeSet, petunia_design_foundation::PetuniaError> {
    session.dispatch_action(ActionRequest::new(
        ActionId::new("ptnd.action.file.place"),
        serde_json::json!({"path":path}),
    ))
}

#[test]
fn missing_and_corrupt_sources_leave_document_history_selection_and_ids_unchanged() {
    for bytes in [None, Some(b"not a PNG".as_slice())] {
        let mut session = session();
        let file = SourceFile::new(bytes.unwrap_or(b""));
        if bytes.is_none() {
            std::fs::remove_file(&file.0).unwrap();
        }
        let document = session.document().clone();
        let selection = session.selection.clone();
        assert!(place(&mut session, &file.0).is_err());
        assert_eq!(session.document(), &document);
        assert_eq!(session.selection, selection);
        assert_eq!(session.current_revision(), 0);
        assert_eq!(session.history().undo_len(), 0);
        assert!(!session.is_dirty());
        assert_eq!(session.next_object_id(), ObjectId::new(2));
    }
}

#[test]
fn null_path_does_not_fabricate_a_sample_image() {
    let mut session = session();
    let before = session.document().clone();
    assert!(session
        .dispatch_action(ActionRequest::new(
            ActionId::new("ptnd.action.file.place"),
            serde_json::Value::Null
        ))
        .is_err());
    assert_eq!(session.document(), &before);
    assert_eq!(session.history().undo_len(), 0);
}

#[test]
fn placing_embeds_exact_source_and_has_one_reversible_command() {
    let bytes = png();
    let file = SourceFile::new(&bytes);
    let mut session = session();
    let before = session.document().clone();
    place(&mut session, &file.0).unwrap();
    let id = session.selection.selected_ids[0];
    let object = session.find_object(id).unwrap();
    assert_eq!(object.bounds, Some([100.0, 100.0, 2.0, 1.0]));
    let Some(ShapeKind::Image {
        data: Some(source), ..
    }) = &object.shape
    else {
        panic!("embedded image");
    };
    assert_eq!(source.as_slice(), bytes);
    let after = session.document().clone();
    assert_eq!(session.history().undo_len(), 1);
    assert!(session.undo().unwrap());
    assert_eq!(session.document(), &before);
    assert!(session.redo().unwrap());
    assert_eq!(session.document(), &after);
}

#[test]
fn native_round_trip_and_png_export_survive_original_file_removal() {
    use petunia_design_application::export_service::{
        export_document, ExportFormat, ExportRequest,
    };
    let bytes = png();
    let file = SourceFile::new(&bytes);
    let mut session = session();
    place(&mut session, &file.0).unwrap();
    std::fs::remove_file(&file.0).unwrap();
    let package = SourceFile::new(b"");
    let package_path = package.0.with_extension("PTND");
    petunia_design_io::save_package(session.document(), &package_path).unwrap();
    let opened = petunia_design_io::open_package(&package_path).unwrap();
    std::fs::remove_file(&package_path).unwrap();
    assert_eq!(&opened.document, session.document());
    let destination = SourceFile::new(b"");
    export_document(
        &opened.document,
        &ExportRequest::new(ExportFormat::Png, &destination.0),
    )
    .unwrap();
    let raw =
        petunia_design_io::import_raster(&std::fs::read(&destination.0).unwrap(), 32 * 1024 * 1024)
            .unwrap();
    let index = (100 * raw.width as usize + 100) * 4;
    assert_eq!(&raw.data[index..index + 4], &[255, 0, 0, 255]);
}
