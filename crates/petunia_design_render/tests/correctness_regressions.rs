use petunia_design_document::{Document, DocumentMutator, DocumentObject, ShapeKind};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_render::SoftwarePixelCompositor;

#[test]
fn object_opacity_is_applied_exactly_once() {
    let mut document = Document::new();
    let mut object = DocumentObject::new(ObjectId::new(2), "half-transparent");
    object.bounds = Some([10.0, 10.0, 20.0, 20.0]);
    object.shape = Some(ShapeKind::Rectangle {
        corner_radii: [0.0; 4],
    });
    object.fill = Some("#ff0000".into());
    object.opacity = 0.5;
    let mut mutator = DocumentMutator::new(&mut document);
    mutator.add_surface(SurfaceId::new(1), "page").unwrap();
    mutator.add_object(SurfaceId::new(1), object).unwrap();
    let pixels =
        SoftwarePixelCompositor::render_surface_rgba8(&document.surfaces()[0], 40, 40, [0; 4]);
    assert_eq!(pixels.get_pixel(15, 15).unwrap(), [255, 0, 0, 128]);
}
