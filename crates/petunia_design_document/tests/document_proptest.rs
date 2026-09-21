//! Property invariants for the document: persistence roundtrip, undo
//! inverse, duplicate rejection over arbitrary trees.

use petunia_design_document::{Document, DocumentMutator, DocumentObject, Surface};
use petunia_design_foundation::IdGenerator;
use proptest::prelude::*;

fn arb_name() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 _-]{1,24}".prop_map(|s: String| s)
}

fn arb_fill() -> impl Strategy<Value = Option<String>> {
    prop::option::of("[a-z]+\\.[a-z]+/[0-9]{3}".prop_map(|s: String| s))
}

proptest! {
    /// Save/reopen preserves any generated document.
    #[test]
    fn document_json_roundtrip(
        surfaces in prop::collection::vec((arb_name(), arb_name(), arb_fill()), 0..4usize),
    ) {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        for (surface_name, object_name, fill) in surfaces {
            let surface_id = gen.next_surface();
            {
                let mut mutator = DocumentMutator::new(&mut doc);
                mutator.add_surface(surface_id, surface_name).unwrap();
                let mut object = DocumentObject::new(gen.next_object(), object_name);
                object.fill = fill;
                mutator.add_object(surface_id, object).unwrap();
            }
        }
        let json = doc.to_json().unwrap();
        let reopened = Document::from_json(&json).unwrap();
        prop_assert_eq!(reopened, doc);
    }

    /// revert() is the exact inverse of add_object for any generated object.
    #[test]
    fn add_then_revert_restores_document(name in arb_name(), fill in arb_fill()) {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let surface_id = gen.next_surface();
        DocumentMutator::new(&mut doc)
            .add_surface(surface_id, "Page")
            .unwrap();
        let before = doc.clone();
        let mut object = DocumentObject::new(gen.next_object(), name);
        object.fill = fill;
        let mut mutator = DocumentMutator::new(&mut doc);
        let changes = mutator.add_object(surface_id, object).unwrap();
        mutator.revert(&changes).unwrap();
        prop_assert_eq!(&doc, &before);
    }

    /// Duplicate object IDs are always rejected, regardless of payload.
    #[test]
    fn duplicate_object_id_rejected(name_a in arb_name(), name_b in arb_name()) {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let surface_id = gen.next_surface();
        let mut mutator = DocumentMutator::new(&mut doc);
        mutator.add_surface(surface_id, "Page").unwrap();
        let id = gen.next_object();
        mutator
            .add_object(surface_id, DocumentObject::new(id, name_a))
            .unwrap();
        prop_assert!(mutator
            .add_object(surface_id, DocumentObject::new(id, name_b))
            .is_err());
    }

    /// Surface count is preserved through serialization.
    #[test]
    fn surface_list_roundtrip(names in prop::collection::vec(arb_name(), 0..6usize)) {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        for name in &names {
            let id = gen.next_surface();
            DocumentMutator::new(&mut doc).add_surface(id, name.clone()).unwrap();
        }
        let reopened = Document::from_json(&doc.to_json().unwrap()).unwrap();
        prop_assert_eq!(reopened.surfaces().len(), names.len());
        let _ = Surface::new(gen.next_surface(), "unused-construction-check");
    }
}
