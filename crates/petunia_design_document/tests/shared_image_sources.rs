use petunia_design_document::ShapeKind;
use std::sync::Arc;

#[test]
fn cloning_image_descriptor_shares_its_source_without_changing_wire_bytes() {
    let source = Arc::new(vec![1, 2, 3, 4]);
    let shape = ShapeKind::Image {
        path: "source.png".into(),
        data: Some(source.clone()),
    };
    let clone = shape.clone();
    let ShapeKind::Image {
        data: Some(clone_source),
        ..
    } = clone
    else {
        panic!("image descriptor");
    };
    assert!(Arc::ptr_eq(&source, &clone_source));
    let wire = serde_json::to_value(&shape).unwrap();
    assert_eq!(wire["data"], serde_json::json!([1, 2, 3, 4]));
    assert_eq!(serde_json::from_value::<ShapeKind>(wire).unwrap(), shape);
}
