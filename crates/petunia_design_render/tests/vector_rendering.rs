//! Pixel regressions prepared during implementation; execution is deferred.
use petunia_design_document::{
    AppearanceStack, ContainerRole, DocumentObject, EffectItem, EffectKind, FillItem, GradientStop,
    LinearGradient, MaskMode, ModifierItem, ModifierKind, ModifierSpace, OpacityStop, ShapeKind,
    StrokeItem, Surface,
};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_geometry::{GPath, GRect};
use petunia_design_render::{
    CpuRenderer, RenderError, RenderLimits, RenderRequest, RenderSurface, SoftwarePixelCompositor,
};
use std::sync::Arc;

fn rect(id: u64, bounds: [f64; 4], fill: &str) -> DocumentObject {
    let mut o = DocumentObject::new(ObjectId::new(id), "vector");
    o.bounds = Some(bounds);
    o.shape = Some(ShapeKind::Rectangle {
        corner_radii: [0.0; 4],
    });
    o.fill = Some(fill.into());
    o
}
fn snapshot(objects: Vec<DocumentObject>) -> RenderSurface {
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

#[test]
fn ellipse_has_transparent_corners_and_antialiased_edges() {
    let mut o = rect(2, [10.0, 10.0, 20.0, 20.0], "#ff0000");
    o.shape = Some(ShapeKind::Ellipse);
    let pixels = CpuRenderer::default()
        .render(&snapshot(vec![o]), request())
        .unwrap();
    assert_eq!(pixels.get_pixel(10, 10).unwrap()[3], 0);
    assert_eq!(pixels.get_pixel(20, 20).unwrap(), [255, 0, 0, 255]);
    assert!(pixels.data.chunks_exact(4).any(|p| p[3] > 0 && p[3] < 255));
}

#[test]
fn shape_less_objects_do_not_invent_rectangles() {
    let mut o = rect(2, [0.0, 0.0, 30.0, 30.0], "#ff0000");
    o.shape = None;
    let pixels = CpuRenderer::default()
        .render(&snapshot(vec![o]), request())
        .unwrap();
    assert!(pixels.data.iter().all(|v| *v == 0));
}

#[test]
fn compound_geometry_preserves_even_odd_holes() {
    let mut path = GPath::rect(GRect::new(0.0, 0.0, 30.0, 30.0), 0.0, 0.0);
    path.verbs
        .extend(GPath::rect(GRect::new(10.0, 10.0, 20.0, 20.0), 0.0, 0.0).verbs);
    let mut o = rect(2, [0.0, 0.0, 30.0, 30.0], "#0000ff");
    o.shape = Some(ShapeKind::LocalPath {
        path: Arc::new(path),
        reference_size: [30.0, 30.0],
    });
    let pixels = CpuRenderer::default()
        .render(&snapshot(vec![o]), request())
        .unwrap();
    assert_eq!(pixels.get_pixel(5, 5).unwrap(), [0, 0, 255, 255]);
    assert_eq!(pixels.get_pixel(15, 15).unwrap()[3], 0);
}

#[test]
fn color_gradient_changes_across_the_shape() {
    let mut o = rect(2, [0.0, 0.0, 20.0, 20.0], "#000000");
    let mut a = AppearanceStack::new();
    a.add_fill(FillItem::linear_gradient(
        1,
        LinearGradient::new(
            [0.0, 0.0],
            [20.0, 0.0],
            vec![
                GradientStop::new(0.0, "#ff0000"),
                GradientStop::new(1.0, "#0000ff"),
            ],
        ),
    ));
    o.appearance = Some(a);
    let pixels = CpuRenderer::default()
        .render(&snapshot(vec![o]), request())
        .unwrap();
    let left = pixels.get_pixel(2, 10).unwrap();
    let right = pixels.get_pixel(17, 10).unwrap();
    assert!(left[0] > left[2] && right[2] > right[0]);
}

#[test]
fn stroke_does_not_fill_its_interior() {
    let mut o = rect(2, [10.0, 10.0, 20.0, 20.0], "#ff0000");
    o.fill = None;
    o.stroke = Some("#0000ff".into());
    o.stroke_width = 4.0;
    let pixels = CpuRenderer::default()
        .render(&snapshot(vec![o]), request())
        .unwrap();
    assert_eq!(pixels.get_pixel(20, 20).unwrap()[3], 0);
    assert_eq!(pixels.get_pixel(10, 20).unwrap(), [0, 0, 255, 255]);
}

#[test]
fn multiple_fills_receive_object_opacity_once_after_composition() {
    let mut o = rect(2, [0.0, 0.0, 20.0, 20.0], "#000000");
    let mut a = AppearanceStack::new();
    a.opacity = 0.5;
    a.add_fill(FillItem::solid(1, "#ff0000"));
    a.add_fill(FillItem::solid(2, "#0000ff"));
    o.appearance = Some(a);
    assert_eq!(
        CpuRenderer::default()
            .render(&snapshot(vec![o]), request())
            .unwrap()
            .get_pixel(10, 10)
            .unwrap(),
        [0, 0, 255, 128]
    );
}

#[test]
fn group_opacity_does_not_accumulate_at_child_overlaps() {
    let mut parent = DocumentObject::new(ObjectId::new(2), "Group");
    parent.role = Some(ContainerRole::Group);
    parent.opacity = 0.5;
    parent.children = vec![ObjectId::new(3), ObjectId::new(4)];
    let mut a = rect(3, [5.0, 5.0, 20.0, 20.0], "#ff0000");
    a.parent = Some(parent.id);
    let mut b = rect(4, [15.0, 5.0, 20.0, 20.0], "#0000ff");
    b.parent = Some(parent.id);
    let pixels = CpuRenderer::default()
        .render(&snapshot(vec![parent, a, b]), request())
        .unwrap();
    assert_eq!(pixels.get_pixel(10, 10).unwrap()[3], 128);
    assert_eq!(pixels.get_pixel(20, 10).unwrap(), [0, 0, 255, 128]);
}

#[test]
fn rotated_parent_places_child_geometry_in_world_coordinates() {
    let mut parent = DocumentObject::new(ObjectId::new(2), "Group");
    parent.bounds = Some([30.0, 10.0, 30.0, 30.0]);
    parent.rotation = std::f64::consts::FRAC_PI_2;
    parent.role = Some(ContainerRole::Group);
    parent.children = vec![ObjectId::new(3)];
    let mut child = rect(3, [0.0, 0.0, 20.0, 10.0], "#ff0000");
    child.parent = Some(parent.id);
    let pixels = CpuRenderer::default()
        .render(&snapshot(vec![parent, child]), request())
        .unwrap();
    assert_eq!(pixels.get_pixel(25, 20).unwrap(), [255, 0, 0, 255]);
    assert_eq!(pixels.get_pixel(5, 5).unwrap()[3], 0);
}

#[test]
fn elliptical_vector_mask_clips_geometry_instead_of_bounds() {
    let mut mask = rect(2, [10.0, 10.0, 20.0, 20.0], "#ffffff");
    mask.is_clip_mask = true;
    mask.shape = Some(ShapeKind::Ellipse);
    let mut content = rect(3, [0.0, 0.0, 40.0, 40.0], "#ff0000");
    content.clip_mask_id = Some(mask.id);
    let pixels = CpuRenderer::default()
        .render(&snapshot(vec![mask, content]), request())
        .unwrap();
    assert_eq!(pixels.get_pixel(10, 10).unwrap()[3], 0);
    assert_eq!(pixels.get_pixel(20, 20).unwrap(), [255, 0, 0, 255]);
}

#[test]
fn alpha_and_luminance_masks_use_distinct_sample_semantics() {
    let mut mask = rect(2, [0.0, 0.0, 20.0, 20.0], "#000000");
    mask.is_clip_mask = true;
    mask.opacity = 0.5;
    let mut content = rect(3, [0.0, 0.0, 20.0, 20.0], "#ff0000");
    content.clip_mask_id = Some(mask.id);
    content.mask_mode = MaskMode::Alpha;
    let alpha = CpuRenderer::default()
        .render(&snapshot(vec![mask.clone(), content.clone()]), request())
        .unwrap();
    assert_eq!(alpha.get_pixel(10, 10).unwrap()[3], 128);
    content.mask_mode = MaskMode::Luminance;
    let luma = CpuRenderer::default()
        .render(&snapshot(vec![mask, content]), request())
        .unwrap();
    assert_eq!(luma.get_pixel(10, 10).unwrap()[3], 0);
}

#[test]
fn transparency_modifier_is_sampled_per_pixel() {
    let mut o = rect(2, [0.0, 0.0, 20.0, 20.0], "#ff0000");
    let mut modifier = ModifierItem::new(
        1,
        ModifierKind::TransparentGradient {
            start: [0.0, 0.0],
            end: [20.0, 0.0],
            stops: vec![OpacityStop::new(0.0, 0.0), OpacityStop::new(1.0, 1.0)],
        },
    );
    modifier.space = ModifierSpace::Local {
        reference_size: [20.0, 20.0],
    };
    o.modifiers.push(modifier);
    let pixels = CpuRenderer::default()
        .render(&snapshot(vec![o]), request())
        .unwrap();
    assert!(pixels.get_pixel(2, 10).unwrap()[3] < 50);
    assert!(pixels.get_pixel(17, 10).unwrap()[3] > 200);
}

#[test]
fn gaussian_blur_has_soft_edges_and_no_dark_color_fringe() {
    let mut o = rect(2, [20.0, 20.0, 10.0, 10.0], "#ff0000");
    let mut a = o.effective_appearance();
    a.effects.push(EffectItem {
        id: 1,
        visible: true,
        kind: EffectKind::GaussianBlur { radius: 2.0 },
    });
    o.appearance = Some(a);
    let pixels = CpuRenderer::default()
        .render(&snapshot(vec![o]), request())
        .unwrap();
    let edge = pixels.get_pixel(19, 25).unwrap();
    assert!(edge[3] > 0 && edge[3] < 255);
    assert_eq!(edge[0], 255);
    assert_eq!(pixels.get_pixel(5, 5).unwrap()[3], 0);
}

#[test]
fn cropped_blur_matches_full_render_without_edge_seams() {
    let mut o = rect(2, [20.0, 20.0, 10.0, 10.0], "#ff0000");
    let mut a = o.effective_appearance();
    a.effects.push(EffectItem {
        id: 1,
        visible: true,
        kind: EffectKind::GaussianBlur { radius: 2.0 },
    });
    o.appearance = Some(a);
    let scene = snapshot(vec![o]);
    let full = CpuRenderer::default().render(&scene, request()).unwrap();
    let crop = CpuRenderer::default()
        .render(
            &scene,
            RenderRequest {
                viewport: GRect::new(25.0, 15.0, 35.0, 35.0),
                width: 10,
                height: 20,
                background: [0; 4],
            },
        )
        .unwrap();
    for y in 0..20 {
        for x in 0..10 {
            assert_eq!(crop.get_pixel(x, y), full.get_pixel(x + 25, y + 15));
        }
    }
}

#[test]
fn drop_shadow_is_an_offset_blurred_silhouette() {
    let mut o = rect(2, [10.0, 10.0, 10.0, 10.0], "#ff0000");
    let mut a = o.effective_appearance();
    a.effects.push(EffectItem {
        id: 1,
        visible: true,
        kind: EffectKind::DropShadow {
            offset: [10.0, 0.0],
            blur: 1.0,
            color: "#0000ff".into(),
            opacity: 0.5,
        },
    });
    o.appearance = Some(a);
    let pixels = CpuRenderer::default()
        .render(&snapshot(vec![o]), request())
        .unwrap();
    assert_eq!(pixels.get_pixel(15, 15).unwrap(), [255, 0, 0, 255]);
    let shadow = pixels.get_pixel(25, 15).unwrap();
    assert_eq!(shadow[2], 255);
    assert!(shadow[3] > 100 && shadow[3] <= 128);
}

#[test]
fn artboard_origin_does_not_inflate_allocation_and_dpi_controls_size() {
    for origin in [[-1_000_000.0, -1_000_000.0], [1_000_000.0, 1_000_000.0]] {
        let o = rect(2, [origin[0], origin[1], 10.0, 10.0], "#ff0000");
        let mut surface = Surface::with_objects(SurfaceId::new(1), "Far away", vec![o]);
        surface.origin = origin;
        surface.dimensions = [10.0, 10.0];
        let scene = RenderSurface::extract(&surface).unwrap();
        let pixels = CpuRenderer::default()
            .render(&scene, RenderRequest::for_surface(&scene, 144.0).unwrap())
            .unwrap();
        assert_eq!(
            (pixels.width, pixels.height, pixels.data.len()),
            (20, 20, 1600)
        );
        assert_eq!(pixels.get_pixel(10, 10).unwrap(), [255, 0, 0, 255]);
    }
}

#[test]
fn output_and_live_intermediate_budgets_reject_before_large_allocation() {
    let scene = snapshot(vec![rect(2, [0.0, 0.0, 64.0, 64.0], "#ff0000")]);
    let renderer = CpuRenderer::new(RenderLimits {
        max_output_pixels: 10,
        ..RenderLimits::default()
    });
    assert!(matches!(
        renderer.render(&scene, request()),
        Err(RenderError::Limit(_))
    ));
    let renderer = CpuRenderer::new(RenderLimits {
        max_working_bytes: 64 * 64 * 4,
        ..RenderLimits::default()
    });
    assert!(matches!(
        renderer.render(&scene, request()),
        Err(RenderError::Limit(_))
    ));
}

#[test]
fn unsupported_content_and_invalid_hierarchy_return_reasons() {
    let mut image = rect(2, [0.0, 0.0, 20.0, 20.0], "#ff0000");
    image.shape = Some(ShapeKind::Image {
        path: "image.png".into(),
        data: None,
    });
    let image_scene = RenderSurface::extract(&Surface::with_objects(
        SurfaceId::new(1),
        "Page",
        vec![image],
    ))
    .unwrap();
    assert!(matches!(
        CpuRenderer::default().render(&image_scene, request()),
        Err(RenderError::Unsupported { .. })
    ));
    let mut cycle = rect(2, [0.0, 0.0, 20.0, 20.0], "#ff0000");
    cycle.parent = Some(cycle.id);
    cycle.children.push(cycle.id);
    assert!(RenderSurface::extract(&Surface::with_objects(
        SurfaceId::new(1),
        "Page",
        vec![cycle]
    ))
    .is_err());
}

#[test]
fn damage_retains_previous_effect_reach_and_mask_dependants() {
    let mut mask = rect(2, [10.0, 10.0, 10.0, 10.0], "#ffffff");
    mask.is_clip_mask = true;
    let mut content = rect(3, [0.0, 0.0, 60.0, 60.0], "#ff0000");
    content.clip_mask_id = Some(mask.id);
    let before = snapshot(vec![mask.clone(), content.clone()]);
    mask.bounds = Some([40.0, 40.0, 10.0, 10.0]);
    let after = snapshot(vec![mask, content]);
    let damage = before.damage_to(&after).unwrap().unwrap();
    assert!(damage.x0 <= 0.0 && damage.y0 <= 0.0 && damage.x1 >= 60.0 && damage.y1 >= 60.0);
    assert!(before.damage_to(&before).unwrap().is_none());
}

#[test]
fn failed_dirty_render_keeps_destination_unchanged() {
    let mut o = rect(2, [0.0, 0.0, 20.0, 20.0], "#ff0000");
    let mut a = o.effective_appearance();
    a.effects.push(EffectItem {
        id: 1,
        visible: true,
        kind: EffectKind::Noise {
            amount: 0.5,
            monochrome: true,
        },
    });
    o.appearance = Some(a);
    let surface = Surface::with_objects(SurfaceId::new(1), "Unsupported", vec![o]);
    let mut pixels = petunia_design_render::PixelBufferRgba8::with_fill(64, 64, [0, 255, 0, 255]);
    let before = pixels.clone();
    assert!(SoftwarePixelCompositor::render_surface_dirty_rgba8(
        &surface,
        &mut pixels,
        request().viewport,
        Some([0; 4])
    )
    .is_err());
    assert_eq!(pixels, before);
}

#[test]
fn open_stroke_with_outside_alignment_is_not_silently_centered() {
    let mut o = rect(2, [0.0, 0.0, 20.0, 20.0], "#ff0000");
    let mut path = GPath::rect(GRect::new(0.0, 0.0, 20.0, 20.0), 0.0, 0.0);
    path.verbs.pop();
    o.shape = Some(ShapeKind::LocalPath {
        path: Arc::new(path),
        reference_size: [20.0, 20.0],
    });
    let mut a = AppearanceStack::new();
    let mut stroke = StrokeItem::solid(1, "#ff0000", 2.0);
    stroke.alignment = petunia_design_document::StrokeAlignment::Outside;
    a.add_stroke(stroke);
    o.appearance = Some(a);
    assert!(matches!(
        CpuRenderer::default().render(&snapshot(vec![o]), request()),
        Err(RenderError::Unsupported { .. })
    ));
}

#[test]
fn cancelled_render_returns_no_partial_image() {
    let scene = snapshot(vec![rect(2, [0.0, 0.0, 64.0, 64.0], "#ff0000")]);
    let token = petunia_design_jobs::CancellationToken::new();
    token.cancel();
    assert!(matches!(
        CpuRenderer::default().render_cancellable(&scene, request(), &token),
        Err(RenderError::Cancelled)
    ));
}

#[test]
fn gradient_coordinates_are_localized_before_f32_conversion() {
    let origin = 1_000_000.0;
    let size = 0.001;
    let mut object = rect(2, [origin, origin, size, size], "#000000");
    let mut appearance = AppearanceStack::new();
    appearance.add_fill(FillItem::linear_gradient(
        1,
        LinearGradient::new(
            [origin, origin],
            [origin + size, origin],
            vec![
                GradientStop::new(0.0, "#ff0000"),
                GradientStop::new(1.0, "#0000ff"),
            ],
        ),
    ));
    object.appearance = Some(appearance);
    let mut surface = Surface::with_objects(SurfaceId::new(1), "Tiny distant art", vec![object]);
    surface.origin = [origin, origin];
    surface.dimensions = [size, size];
    let scene = RenderSurface::extract(&surface).unwrap();
    let pixels = CpuRenderer::default()
        .render(
            &scene,
            RenderRequest {
                viewport: GRect::new(origin, origin, origin + size, origin + size),
                width: 20,
                height: 20,
                background: [0; 4],
            },
        )
        .unwrap();
    let left = pixels.get_pixel(2, 10).unwrap();
    let right = pixels.get_pixel(17, 10).unwrap();
    assert!(left[0] > left[2] && right[2] > right[0]);
}
