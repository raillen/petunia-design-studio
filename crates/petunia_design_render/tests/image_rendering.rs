//! Image composition regression sources; execution is deferred.
use image::ImageEncoder;
use petunia_design_document::{
    ContainerRole, DocumentObject, ModifierItem, ModifierKind, ModifierSpace, OpacityStop,
    ShapeKind, Surface,
};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_geometry::GRect;
use petunia_design_raster::{EncodedImage, ImageCache, ImageCacheLimits};
use petunia_design_render::{CpuRenderer, RenderError, RenderLimits, RenderRequest, RenderSurface};
use std::sync::Arc;

fn placed(id: u64, bounds: [f64; 4], pixel: [u8; 4]) -> DocumentObject {
    let mut bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut bytes)
        .write_image(&pixel, 1, 1, image::ExtendedColorType::Rgba8)
        .unwrap();
    let mut object = DocumentObject::new(ObjectId::new(id), "Photo");
    object.bounds = Some(bounds);
    object.shape = Some(ShapeKind::Image {
        path: "/nonexistent/original.png".into(),
        data: Some(Arc::new(EncodedImage::new(bytes).unwrap())),
    });
    object
}
fn scene(objects: Vec<DocumentObject>) -> RenderSurface {
    RenderSurface::extract(&Surface::with_objects(SurfaceId::new(1), "Page", objects)).unwrap()
}
fn request() -> RenderRequest {
    RenderRequest {
        viewport: GRect::new(0.0, 0.0, 64.0, 64.0),
        width: 64,
        height: 64,
        background: [0; 4],
    }
}
fn modifier(kind: ModifierKind, size: [f64; 2]) -> ModifierItem {
    let mut value = ModifierItem::new(1, kind);
    value.space = ModifierSpace::Local {
        reference_size: size,
    };
    value
}

#[test]
fn embedded_image_renders_without_opening_its_original_path() {
    let pixels = CpuRenderer::default()
        .render(
            &scene(vec![placed(2, [10.0, 10.0, 20.0, 10.0], [255, 0, 0, 255])]),
            request(),
        )
        .unwrap();
    assert_eq!(pixels.get_pixel(15, 15).unwrap(), [255, 0, 0, 255]);
    assert_eq!(pixels.get_pixel(5, 5).unwrap()[3], 0);
}

#[test]
fn image_alpha_and_object_opacity_are_applied_once() {
    let mut object = placed(2, [10.0, 10.0, 20.0, 20.0], [255, 0, 0, 128]);
    object.opacity = 0.5;
    let pixels = CpuRenderer::default()
        .render(&scene(vec![object]), request())
        .unwrap();
    assert_eq!(pixels.get_pixel(20, 20).unwrap(), [255, 0, 0, 64]);
}

#[test]
fn ancestor_rotation_and_group_isolation_apply_to_images() {
    let mut parent = DocumentObject::new(ObjectId::new(2), "Group");
    parent.role = Some(ContainerRole::Group);
    parent.bounds = Some([30.0, 10.0, 30.0, 30.0]);
    parent.rotation = std::f64::consts::FRAC_PI_2;
    parent.opacity = 0.5;
    parent.children = vec![ObjectId::new(3), ObjectId::new(4)];
    let mut a = placed(3, [0.0, 0.0, 20.0, 10.0], [255, 0, 0, 255]);
    a.parent = Some(parent.id);
    let mut b = placed(4, [10.0, 0.0, 20.0, 10.0], [0, 0, 255, 255]);
    b.parent = Some(parent.id);
    let pixels = CpuRenderer::default()
        .render(&scene(vec![parent, a, b]), request())
        .unwrap();
    assert_eq!(pixels.get_pixel(25, 15).unwrap(), [255, 0, 0, 128]);
    assert_eq!(pixels.get_pixel(25, 25).unwrap(), [0, 0, 255, 128]);
    assert_eq!(pixels.get_pixel(35, 25).unwrap()[3], 0);
}

#[test]
fn crop_changes_coverage_without_stretching_the_source() {
    let mut object = placed(2, [10.0, 10.0, 20.0, 20.0], [0, 255, 0, 255]);
    let mut encoded = Vec::new();
    image::codecs::png::PngEncoder::new(&mut encoded)
        .write_image(
            &[255, 0, 0, 255, 0, 0, 255, 255],
            2,
            1,
            image::ExtendedColorType::Rgba8,
        )
        .unwrap();
    object.shape = Some(ShapeKind::Image {
        path: "missing.png".into(),
        data: Some(Arc::new(EncodedImage::new(encoded).unwrap())),
    });
    let original = object.shape.clone();
    object.modifiers.push(modifier(
        ModifierKind::CropRect {
            rect: [10.0, 0.0, 10.0, 20.0],
        },
        [20.0, 20.0],
    ));
    let pixels = CpuRenderer::default()
        .render(&scene(vec![object.clone()]), request())
        .unwrap();
    assert_eq!(pixels.get_pixel(15, 20).unwrap()[3], 0);
    let right = pixels.get_pixel(28, 20).unwrap();
    assert_eq!(right, [0, 0, 255, 255]);
    assert_eq!(object.shape, original);
}

#[test]
fn empty_crop_stays_empty_in_scene_bounds_and_pixels() {
    let mut object = placed(2, [10.0, 10.0, 20.0, 20.0], [255, 0, 0, 255]);
    object.modifiers.push(modifier(
        ModifierKind::CropRect {
            rect: [100.0, 100.0, 5.0, 5.0],
        },
        [20.0, 20.0],
    ));
    let scene = scene(vec![object]);
    assert!(scene
        .node(ObjectId::new(2))
        .unwrap()
        .visual_bounds()
        .is_none());
    let pixels = CpuRenderer::default().render(&scene, request()).unwrap();
    assert!(pixels.data.iter().all(|byte| *byte == 0));
}

#[test]
fn spatial_transparency_and_vector_masks_sample_image_pixels() {
    let mut object = placed(2, [10.0, 10.0, 20.0, 20.0], [255, 0, 0, 255]);
    object.modifiers.push(modifier(
        ModifierKind::TransparentGradient {
            start: [0.0, 0.0],
            end: [20.0, 0.0],
            stops: vec![OpacityStop::new(0.0, 0.0), OpacityStop::new(1.0, 1.0)],
        },
        [20.0, 20.0],
    ));
    let mut mask = DocumentObject::new(ObjectId::new(3), "Ellipse mask");
    mask.bounds = Some([10.0, 10.0, 20.0, 20.0]);
    mask.shape = Some(ShapeKind::Ellipse);
    mask.fill = Some("#ffffff".into());
    mask.is_clip_mask = true;
    object.clip_mask_id = Some(mask.id);
    let pixels = CpuRenderer::default()
        .render(&scene(vec![object, mask]), request())
        .unwrap();
    assert_eq!(pixels.get_pixel(10, 10).unwrap()[3], 0);
    assert!(pixels.get_pixel(12, 20).unwrap()[3] < 50);
    assert!(pixels.get_pixel(27, 20).unwrap()[3] > 200);
}

#[test]
fn repeated_cpu_renders_reuse_the_supplied_cache() {
    let cache = Arc::new(ImageCache::new(ImageCacheLimits::default()).unwrap());
    let renderer = CpuRenderer::with_image_cache(RenderLimits::default(), cache.clone());
    let scene = scene(vec![placed(2, [0.0, 0.0, 10.0, 10.0], [0, 0, 255, 255])]);
    renderer.render(&scene, request()).unwrap();
    renderer.render(&scene, request()).unwrap();
    assert_eq!(cache.stats().misses, 1);
    assert_eq!(cache.stats().hits, 1);
}

#[test]
fn missing_source_and_unimplemented_image_warps_have_capability_reasons() {
    let mut object = placed(2, [0.0, 0.0, 20.0, 20.0], [255, 0, 0, 255]);
    if let Some(ShapeKind::Image { data, .. }) = &mut object.shape {
        *data = None;
    }
    assert!(matches!(
        CpuRenderer::default().render(&scene(vec![object]), request()),
        Err(RenderError::Unsupported {
            feature: "embedded image source",
            ..
        })
    ));
    let mut object = placed(3, [0.0, 0.0, 20.0, 20.0], [255, 0, 0, 255]);
    object.modifiers.push(modifier(
        ModifierKind::Perspective {
            quad: [[0.0, 0.0], [20.0, 0.0], [18.0, 20.0], [0.0, 20.0]],
        },
        [20.0, 20.0],
    ));
    assert!(matches!(
        CpuRenderer::default().render(&scene(vec![object]), request()),
        Err(RenderError::Unsupported {
            feature: "image perspective/contour sampling",
            ..
        })
    ));
}
