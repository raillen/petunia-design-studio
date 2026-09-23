use petunia_design_document::{Document, DocumentMutator, DocumentObject};
use petunia_design_foundation::IdGenerator;
use petunia_design_io::{export_document_pdf, PdfExportOptions};
use proptest::prelude::*;

proptest! {
    #[test]
    fn arbitrary_document_always_exports_valid_pdf_stream(
        surface_count in 1..4usize,
        objects_per_surface in 0..5usize,
        r in 0u8..=255,
        g in 0u8..=255,
        b in 0u8..=255,
    ) {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let mut mutator = DocumentMutator::new(&mut doc);

        let hex_color = format!("#{r:02x}{g:02x}{b:02x}");
        let mut total_expected_objects = 0;

        for s_idx in 0..surface_count {
            let s_id = gen.next_surface();
            mutator.add_surface(s_id, format!("Surface {s_idx}")).unwrap();

            for o_idx in 0..objects_per_surface {
                let mut obj = DocumentObject::new(gen.next_object(), format!("Obj {o_idx}"));
                obj.fill = Some(hex_color.clone());
                mutator.add_object(s_id, obj).unwrap();
                total_expected_objects += 1;
            }
        }

        let options = PdfExportOptions::default();
        let (bytes, report) = export_document_pdf(&doc, &options).expect("PDF export should succeed");

        prop_assert!(!bytes.is_empty(), "PDF bytes must not be empty");
        prop_assert!(bytes.starts_with(b"%PDF-"), "PDF stream must start with %PDF- magic bytes");
        prop_assert_eq!(report.surfaces, surface_count);
        prop_assert_eq!(report.objects, total_expected_objects);
        prop_assert!(report.passed);
    }
}
