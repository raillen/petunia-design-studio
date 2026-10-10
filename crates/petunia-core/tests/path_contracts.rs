use petunia_core::{Document, NodeKind, ParentRef, PathNode, Point, SceneNode, VectorPath};

#[test]
fn path_ids_and_coordinates_are_validated_at_scene_boundary() {
    let mut document = Document::new("validated path");
    let page = document.scene.default_page();
    let mut path = VectorPath::rect(0.0, 0.0, 5.0, 5.0);
    let duplicate = path.contours[0].nodes[0].clone();
    path.contours[0].nodes.push(duplicate);
    assert!(path.validate().is_err());
    assert!(document
        .scene
        .insert_root(
            page,
            SceneNode::new_path("duplicate", path, ParentRef::Page(page))
        )
        .is_err());
    let mut path = VectorPath::rect(0.0, 0.0, 5.0, 5.0);
    path.contours[0].nodes[0].handle_out = Some(Point::new(f64::NAN, 0.0));
    assert!(path.validate().is_err());
    assert!(document
        .scene
        .insert_root(
            page,
            SceneNode::new_path("NaN", path, ParentRef::Page(page))
        )
        .is_err());
    assert_eq!(document.scene.len(), 0);
}

#[test]
fn degenerate_geometry_remains_valid() {
    let mut path = VectorPath::new();
    let mut contour = petunia_core::Contour::new(true);
    contour.push_node(PathNode::new(Point::new(0.0, 0.0), NodeKind::Cusp));
    path.push_contour(contour);
    assert!(path.validate().is_ok());
}
