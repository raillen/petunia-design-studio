use petunia_design_document::{Document, DocumentMutator, DocumentObject, ShapeKind};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_geometry::{GPath, GRect};

fn legacy_document() -> Document {
    let mut doc = Document::new();
    let mut m = DocumentMutator::new(&mut doc);
    m.add_surface(SurfaceId::new(1), "Page").unwrap();
    let mut object = DocumentObject::new(ObjectId::new(2), "Legacy");
    object.bounds = Some([100.0, -20.0, 30.0, 40.0]);
    object.shape = Some(ShapeKind::Path(GPath::rect(
        GRect::new(100.0, -20.0, 130.0, 20.0),
        0.0,
        0.0,
    )));
    m.add_object(SurfaceId::new(1), object).unwrap();
    doc
}

#[test]
fn legacy_schema_migrates_without_changing_world_geometry() {
    // Independent schema-1 payload, not one serialized by the new writer.
    let mut value: serde_json::Value =
        serde_json::from_str(&legacy_document().to_json().unwrap()).unwrap();
    value["schema_version"] = 1.into();
    let old = GPath::rect(GRect::new(100.0, -20.0, 130.0, 20.0), 0.0, 0.0);
    value["surfaces"][0]["objects"][0]["shape"] =
        serde_json::to_value(ShapeKind::Path(old.clone())).unwrap();
    let migrated = Document::from_json(&value.to_string()).unwrap();
    assert_eq!(migrated.schema_version(), 3);
    assert_eq!(
        migrated.evaluated_path_world(ObjectId::new(2)).unwrap(),
        old
    );
    assert!(matches!(
        migrated.find_object(ObjectId::new(2)).unwrap().shape,
        Some(ShapeKind::LocalPath { .. })
    ));
    assert_eq!(
        Document::from_json(&migrated.to_json().unwrap()).unwrap(),
        migrated
    );
}

#[test]
fn moving_and_resizing_preserve_source_points_and_revert_exactly() {
    let mut doc = legacy_document();
    let before = doc.clone();
    let source = doc.find_object(ObjectId::new(2)).unwrap().shape.clone();
    let mut m = DocumentMutator::new(&mut doc);
    let changes = m
        .set_bounds(ObjectId::new(2), Some([200.0, 50.0, 60.0, 20.0]), 0.0)
        .unwrap();
    assert_eq!(
        m.document().find_object(ObjectId::new(2)).unwrap().shape,
        source
    );
    let rect = m
        .document()
        .evaluated_path_world(ObjectId::new(2))
        .unwrap()
        .bounding_box()
        .unwrap();
    assert_eq!(rect, GRect::new(200.0, 50.0, 260.0, 70.0));
    m.revert(&changes).unwrap();
    assert_eq!(doc, before);
}

#[test]
fn atomic_path_edit_updates_both_frames_and_undo_restores_source() {
    let mut doc = legacy_document();
    let before = doc.clone();
    let next = GPath::rect(GRect::new(-50.0, 80.0, 10.0, 100.0), 0.0, 0.0);
    let mut m = DocumentMutator::new(&mut doc);
    let changes = m
        .set_path(
            ObjectId::new(2),
            next.clone(),
            [-50.0, 80.0, 60.0, 20.0],
            0.0,
        )
        .unwrap();
    assert_eq!(
        m.document().evaluated_path_world(ObjectId::new(2)).unwrap(),
        next
    );
    let snapshot = m.document().clone();
    assert!(m
        .set_path(
            ObjectId::new(2),
            GPath::new(),
            [0.0, 0.0, f64::NAN, 10.0],
            0.0
        )
        .is_err());
    assert_eq!(m.document(), &snapshot);
    m.revert(&changes).unwrap();
    assert_eq!(doc, before);
}

#[test]
fn malformed_ids_frames_references_and_cycles_are_rejected() {
    let value: serde_json::Value =
        serde_json::from_str(&legacy_document().to_json().unwrap()).unwrap();
    let mut duplicate = value.clone();
    let object = duplicate["surfaces"][0]["objects"][0].clone();
    duplicate["surfaces"][0]["objects"]
        .as_array_mut()
        .unwrap()
        .push(object);
    assert!(Document::from_json(&duplicate.to_string()).is_err());
    let mut missing = value.clone();
    missing["surfaces"][0]["objects"][0]["parent"] = 999.into();
    assert!(Document::from_json(&missing.to_string()).is_err());
    let mut cycle = value.clone();
    cycle["surfaces"][0]["objects"][0]["parent"] = 2.into();
    cycle["surfaces"][0]["objects"][0]["children"] = serde_json::json!([2]);
    assert!(Document::from_json(&cycle.to_string()).is_err());
    let mut invalid = value;
    // Low-level serde is untrusted. Publication validation must reject inputs
    // which bypassed the canonical loader and its schema/path migration.
    let mut unknown = invalid.clone();
    unknown["schema_version"] = 999.into();
    let raw: Document = serde_json::from_value(unknown).unwrap();
    assert!(raw.validate().is_err());
    let mut unnormalized = invalid.clone();
    unnormalized["surfaces"][0]["objects"][0]["shape"] = serde_json::to_value(ShapeKind::Path(
        GPath::rect(GRect::new(100.0, -20.0, 130.0, 20.0), 0.0, 0.0),
    ))
    .unwrap();
    let raw: Document = serde_json::from_value(unnormalized).unwrap();
    assert!(raw.validate().is_err());
    invalid["surfaces"][0]["objects"][0]["shape"]["reference_size"] = serde_json::json!([0, 40]);
    assert!(Document::from_json(&invalid.to_string()).is_err());
}

#[test]
fn deleting_a_path_detaches_text_and_bindings_and_undo_restores_references() {
    use petunia_design_document::{
        BindingId, DataBinding, DataSourceId, DataSourceParser, MissingValuePolicy, TargetProperty,
        TextOnPathAttachment, ValueFormatter,
    };
    let mut doc = legacy_document();
    let source =
        DataSourceParser::parse_json(DataSourceId::new(1), "source", r#"[{"name":"Blue"}]"#)
            .unwrap();
    let field = source.schema.fields[0].id;
    let mut text = DocumentObject::new(ObjectId::new(3), "Attached");
    text.bounds = Some([100.0, -20.0, 100.0, 20.0]);
    text.shape = Some(ShapeKind::Text {
        content: "Text".into(),
        font_family: "sans-serif".into(),
        font_size: 12.0,
        line_height: 1.2,
        letter_spacing: 0.0,
        on_path: Some(TextOnPathAttachment::new(ObjectId::new(2), 0.0, 1.0)),
    });
    let mut m = DocumentMutator::new(&mut doc);
    m.add_object(SurfaceId::new(1), text).unwrap();
    m.add_data_source(source).unwrap();
    m.add_data_binding(DataBinding {
        id: BindingId::new(1),
        source_id: DataSourceId::new(1),
        field_id: field,
        target_object: ObjectId::new(2),
        target_property: TargetProperty::FillColor,
        formatter: ValueFormatter::None,
        missing_policy: MissingValuePolicy::Skip,
    })
    .unwrap();
    let before = m.document().clone();
    before.validate().unwrap();
    let changes = m.remove_object(ObjectId::new(2)).unwrap();
    m.document().validate().unwrap();
    assert!(m.document().bindings().is_empty());
    assert!(matches!(
        m.document().find_object(ObjectId::new(3)).unwrap().shape,
        Some(ShapeKind::Text { on_path: None, .. })
    ));
    m.revert(&changes).unwrap();
    m.document().validate().unwrap();
    assert_eq!(m.document(), &before);
}

#[test]
fn path_source_is_shared_by_snapshots_and_replacement_does_not_modify_them() {
    let mut doc = legacy_document();
    let snapshot = doc.clone();
    let source = match &doc.find_object(ObjectId::new(2)).unwrap().shape {
        Some(ShapeKind::LocalPath { path, .. }) => path.clone(),
        _ => panic!("canonical path required"),
    };
    let saved = match &snapshot.find_object(ObjectId::new(2)).unwrap().shape {
        Some(ShapeKind::LocalPath { path, .. }) => path,
        _ => panic!("canonical path required"),
    };
    assert!(std::sync::Arc::ptr_eq(&source, saved));
    let expected = snapshot.to_json().unwrap();
    let mut m = DocumentMutator::new(&mut doc);
    m.set_path(
        ObjectId::new(2),
        GPath::rect(GRect::new(0.0, 0.0, 20.0, 20.0), 0.0, 0.0),
        [0.0, 0.0, 20.0, 20.0],
        0.0,
    )
    .unwrap();
    assert_eq!(snapshot.to_json().unwrap(), expected);
    assert_ne!(
        doc.find_object(ObjectId::new(2)).unwrap().shape,
        snapshot.find_object(ObjectId::new(2)).unwrap().shape
    );
}
