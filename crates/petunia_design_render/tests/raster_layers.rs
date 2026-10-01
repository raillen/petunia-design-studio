//! Shared-scene raster/mask regression sources; final batch execution deferred.
use petunia_design_document::{
    Document, DocumentMutator, DocumentObject, MaskMode, ShapeKind, Surface,
};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_geometry::GRect;
use petunia_design_raster::{BitDepth, RasterLayer, RasterLayerKind};
use petunia_design_render::{CpuRenderer, RenderRequest, RenderSurface};
use std::sync::Arc;
fn raster(id: u64, kind: RasterLayerKind, rgba: [f32; 4]) -> DocumentObject {
    let mut layer = RasterLayer::new(16, 16, kind, BitDepth::Sixteen).unwrap();
    for y in 0..16 {
        for x in 0..16 {
            layer.set_pixel(x, y, rgba).unwrap();
        }
    }
    layer.commit();
    let mut object = DocumentObject::new(ObjectId::new(id), "Raster");
    object.bounds = Some([0.0, 0.0, 16.0, 16.0]);
    object.shape = Some(ShapeKind::Raster {
        layer: Arc::new(layer),
    });
    object
}
fn request() -> RenderRequest {
    RenderRequest {
        viewport: GRect::new(0.0, 0.0, 16.0, 16.0),
        width: 16,
        height: 16,
        background: [0; 4],
    }
}
fn render(objects: Vec<DocumentObject>) -> petunia_design_render::PixelBufferRgba8 {
    let scene =
        RenderSurface::extract(&Surface::with_objects(SurfaceId::new(1), "Page", objects)).unwrap();
    CpuRenderer::default().render(&scene, request()).unwrap()
}
#[test]
fn persistent_sixteen_bit_pixels_reach_the_shared_scene() {
    assert_eq!(
        render(vec![raster(
            2,
            RasterLayerKind::Pixels,
            [1.0, 0.0, 0.0, 1.0]
        )])
        .get_pixel(8, 8)
        .unwrap(),
        [255, 0, 0, 255]
    );
}
#[test]
fn gray_coverage_masks_render_as_white_alpha_without_fake_color_fills() {
    let pixel = render(vec![raster(2, RasterLayerKind::Mask, [0.0, 0.0, 0.0, 0.5])])
        .get_pixel(8, 8)
        .unwrap();
    assert_eq!(pixel[..3], [255; 3]);
    assert!((i32::from(pixel[3]) - 128).abs() <= 1);
}
#[test]
fn direct_pixel_mask_samples_coverage_once() {
    let mut content = raster(2, RasterLayerKind::Pixels, [1.0, 0.0, 0.0, 1.0]);
    content.clip_mask_id = Some(ObjectId::new(3));
    let mut mask = raster(3, RasterLayerKind::Mask, [1.0, 1.0, 1.0, 0.5]);
    mask.is_clip_mask = true;
    mask.mask_mode = MaskMode::Alpha;
    let pixel = render(vec![content, mask]).get_pixel(8, 8).unwrap();
    assert!((i32::from(pixel[3]) - 128).abs() <= 1);
}
#[test]
fn canonical_clip_group_does_not_square_coverage() {
    let mut document = Document::new();
    let mut m = DocumentMutator::new(&mut document);
    m.add_surface(SurfaceId::new(1), "Page").unwrap();
    m.add_object(
        SurfaceId::new(1),
        raster(2, RasterLayerKind::Pixels, [1.0, 0.0, 0.0, 1.0]),
    )
    .unwrap();
    m.add_object(
        SurfaceId::new(1),
        raster(3, RasterLayerKind::Mask, [1.0, 1.0, 1.0, 0.5]),
    )
    .unwrap();
    m.create_clip_group(
        SurfaceId::new(1),
        ObjectId::new(4),
        ObjectId::new(3),
        vec![ObjectId::new(2)],
    )
    .unwrap();
    document.validate().unwrap();
    assert_eq!(
        document.find_object(ObjectId::new(3)).unwrap().mask_mode,
        MaskMode::Alpha
    );
    let scene = RenderSurface::extract(document.surface(SurfaceId::new(1)).unwrap()).unwrap();
    let pixel = CpuRenderer::default()
        .render(&scene, request())
        .unwrap()
        .get_pixel(8, 8)
        .unwrap();
    assert!((i32::from(pixel[3]) - 128).abs() <= 1);
}
#[test]
fn raster_object_opacity_is_applied_once() {
    let mut object = raster(2, RasterLayerKind::Pixels, [1.0, 0.0, 0.0, 0.5]);
    object.opacity = 0.5;
    let pixel = render(vec![object]).get_pixel(8, 8).unwrap();
    assert!((i32::from(pixel[3]) - 64).abs() <= 1);
}
#[test]
fn draft_replaces_derived_pixels_without_mutating_source_descriptors() {
    let object = raster(2, RasterLayerKind::Pixels, [1.0, 0.0, 0.0, 1.0]);
    let surface = Surface::with_objects(SurfaceId::new(1), "Page", vec![object]);
    let scene = RenderSurface::extract(&surface).unwrap();
    let Some(ShapeKind::Raster { layer }) =
        raster(2, RasterLayerKind::Pixels, [0.0, 1.0, 0.0, 1.0]).shape
    else {
        unreachable!()
    };
    let draft = scene.with_raster_preview(ObjectId::new(2), layer).unwrap();
    assert_eq!(
        CpuRenderer::default()
            .render(&draft, request())
            .unwrap()
            .get_pixel(8, 8)
            .unwrap(),
        [0, 255, 0, 255]
    );
    assert_eq!(
        CpuRenderer::default()
            .render(&scene, request())
            .unwrap()
            .get_pixel(8, 8)
            .unwrap(),
        [255, 0, 0, 255]
    );
}
