//! SVG subset and precision regression sources for the deferred final batch.
use petunia_design_geometry::{FillRule, GPoint};
use petunia_design_io::{export_document_svg, export_path_d, import_svg, parse_path_d};
#[test]
fn compact_relative_and_smooth_paths_preserve_full_float_precision() {
    let d = "M.123456789-2e-3h4v3c1 2 3 4 5 6s7 8 9 10q1 2 3 4t5 6z";
    let path = parse_path_d(d).unwrap();
    assert!(path.is_finite());
    let exported = export_path_d(&path);
    assert!(exported.contains("0.123456789"));
    assert_eq!(parse_path_d(&exported).unwrap(), path);
}
#[test]
fn elliptical_arc_and_unknown_path_commands_fail_explicitly() {
    assert!(parse_path_d("M0 0 A10 10 0 0 1 20 20").is_err());
    assert!(parse_path_d("M0 0 R1 1").is_err());
}
#[test]
fn imported_svg_defaults_to_nonzero_winding_and_inherits_transforms() {
    let document=import_svg(r##"<svg xmlns="http://www.w3.org/2000/svg" width="40pt" height="40pt" viewBox="0 0 40 40"><g transform="translate(5 6)"><path fill="#f00" d="M0 0h10v10h-10z M2 2h6v6h-6z"/></g></svg>"##).unwrap();
    let object = document.surfaces()[0]
        .objects()
        .iter()
        .find(|o| o.shape.is_some())
        .unwrap();
    assert_eq!(object.fill_rule, FillRule::NonZero);
    let world = document.evaluated_path_world(object.id).unwrap();
    assert!(world.contains_point_with_fill(GPoint::new(10., 11.), 0.01, FillRule::NonZero));
    assert!(export_document_svg(&document)
        .unwrap()
        .contains("fill-rule=\"nonzero\""));
}
#[test]
fn external_resources_css_and_filters_do_not_silently_flatten() {
    for child in [
        r#"<image href="https://example.invalid/a.png"/>"#,
        r#"<path style="fill:red" d="M0 0h2v2z"/>"#,
        r#"<path filter="url(#blur)" d="M0 0h2v2z"/>"#,
    ] {
        assert!(import_svg(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10">{child}</svg>"#
        ))
        .is_err());
    }
}
#[test]
fn document_origin_and_physical_size_are_not_rounded_or_normalized_away() {
    use petunia_design_document::{Document, DocumentMutator, DocumentObject, ShapeKind};
    use petunia_design_foundation::{ObjectId, SurfaceId};
    let mut document = Document::new();
    let mut mutator = DocumentMutator::new(&mut document);
    let surface = SurfaceId::new(1);
    mutator.add_surface(surface, "Page").unwrap();
    mutator
        .set_surface_geometry(surface, [-1000., -20.], [123.456789, 30.])
        .unwrap();
    let mut object = DocumentObject::new(ObjectId::new(2), "Rect");
    object.bounds = Some([-999., -19., 1.123456789, 2.]);
    object.shape = Some(ShapeKind::Rectangle {
        corner_radii: [0.; 4],
    });
    object.fill = Some("#f00".into());
    mutator.add_object(surface, object).unwrap();
    let svg = export_document_svg(&document).unwrap();
    assert!(svg.contains("123.456789pt"));
    assert!(svg.contains("-1000 -20"));
    assert!(svg.contains("1.123456789"));
}

#[test]
fn rgba_hex_alpha_survives_cpu_and_svg_export_without_double_application() {
    use petunia_design_document::{Document, DocumentMutator, DocumentObject, ShapeKind};
    use petunia_design_foundation::{ObjectId, SurfaceId};
    let mut doc = Document::new();
    let mut m = DocumentMutator::new(&mut doc);
    m.add_surface(SurfaceId::new(1), "Page").unwrap();
    let mut object = DocumentObject::new(ObjectId::new(2), "Alpha");
    object.bounds = Some([0., 0., 10., 10.]);
    object.shape = Some(ShapeKind::Rectangle {
        corner_radii: [0.; 4],
    });
    object.fill = Some("#ff000080".into());
    m.add_object(SurfaceId::new(1), object).unwrap();
    let svg = export_document_svg(&doc).unwrap();
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let alpha = xml
        .descendants()
        .find(|n| n.has_tag_name("path") && n.attribute("fill-opacity").is_some())
        .unwrap()
        .attribute("fill-opacity")
        .unwrap()
        .parse::<f64>()
        .unwrap();
    assert!((alpha - 128. / 255.).abs() < 1e-7);
    let scene = petunia_design_render::RenderSurface::extract(&doc.surfaces()[0]).unwrap();
    let pixels = petunia_design_render::CpuRenderer::default()
        .render(
            &scene,
            petunia_design_render::RenderRequest {
                viewport: petunia_design_geometry::GRect::new(0., 0., 10., 10.),
                width: 10,
                height: 10,
                background: [0; 4],
            },
        )
        .unwrap();
    assert_eq!(pixels.get_pixel(5, 5).unwrap(), [255, 0, 0, 128]);
}
