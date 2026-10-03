use petunia_design_document::{Document, DocumentMutator, DocumentObject, ShapeKind};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_io::{export_document_pdf, PdfExportOptions};

#[test]
fn pdf_uses_enabled_surfaces_and_local_page_dimensions() {
    let mut doc = Document::new();
    let mut m = DocumentMutator::new(&mut doc);
    m.add_surface(SurfaceId::new(1), "Selected").unwrap();
    m.add_surface(SurfaceId::new(2), "Excluded").unwrap();
    m.set_surface_geometry(SurfaceId::new(1), [-300.0, 800.0], [320.0, 200.0])
        .unwrap();
    m.set_surface_export_enabled(SurfaceId::new(2), false)
        .unwrap();
    let (bytes, report) = export_document_pdf(&doc, &PdfExportOptions::default()).unwrap();
    assert_eq!(report.surfaces, 1);
    // Independent parser checks the emitted file, including object streams.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("pages.pdf");
    std::fs::write(&path, &bytes).unwrap();
    let output = std::process::Command::new("pdfinfo")
        .arg(&path)
        .output()
        .expect("install poppler-utils to run PDF interoperability tests");
    assert!(
        output.status.success(),
        "pdfinfo rejected exported document"
    );
    let info = String::from_utf8(output.stdout).unwrap();
    assert!(
        info.lines()
            .any(|line| line.starts_with("Page size:") && line.contains("320 x 200 pts")),
        "{info}"
    );
    assert!(
        info.lines()
            .any(|line| line.starts_with("Pages:") && line.split_whitespace().last() == Some("1")),
        "{info}"
    );
    DocumentMutator::new(&mut doc)
        .set_surface_export_enabled(SurfaceId::new(1), false)
        .unwrap();
    assert!(export_document_pdf(&doc, &PdfExportOptions::default()).is_err());
}

#[test]
fn strict_pdf_export_rejects_approximation_and_missing_color_values() {
    let mut doc = Document::new();
    let mut m = DocumentMutator::new(&mut doc);
    m.add_surface(SurfaceId::new(1), "Page").unwrap();
    let mut object = DocumentObject::new(ObjectId::new(2), "Unsupported color");
    object.bounds = Some([10.0, 10.0, 50.0, 50.0]);
    object.shape = Some(ShapeKind::Rectangle {
        corner_radii: [0.0; 4],
    });
    object.fill = Some("unregistered.color".into());
    m.add_object(SurfaceId::new(1), object).unwrap();
    let strict = PdfExportOptions {
        allow_degradations: false,
        ..PdfExportOptions::default()
    };
    assert!(export_document_pdf(&doc, &strict).is_err());
    let permissive = PdfExportOptions {
        allow_degradations: true,
        ..Default::default()
    };
    let (_, report) = export_document_pdf(&doc, &permissive).unwrap();
    assert!(report
        .degradations
        .iter()
        .any(|d| d.code == "COLOR_TOKEN_SUBSTITUTED"));
}
