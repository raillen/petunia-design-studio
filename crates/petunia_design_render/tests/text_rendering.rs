//! Shared text composition regression sources; not executed yet.
use petunia_design_document::{
    ContainerRole, DocumentObject, EffectItem, EffectKind, ModifierItem, ModifierKind,
    ModifierSpace, ShapeKind, Surface,
};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_geometry::GRect;
use petunia_design_render::{CpuRenderer, RenderError, RenderRequest, RenderSurface};
fn text(content: &str) -> DocumentObject {
    let mut o = DocumentObject::new(ObjectId::new(2), "Text");
    o.bounds = Some([5.0, 5.0, 100.0, 50.0]);
    o.fill = Some("#ff0000".into());
    o.shape = Some(ShapeKind::Text {
        content: content.into(),
        font_family: "DejaVu Sans".into(),
        font_size: 32.0,
        line_height: 1.2,
        letter_spacing: 0.0,
        on_path: None,
    });
    o
}
fn surface(objects: Vec<DocumentObject>) -> Surface {
    Surface::with_objects(SurfaceId::new(1), "Page", objects)
}
fn request() -> RenderRequest {
    RenderRequest {
        viewport: GRect::new(0.0, 0.0, 128.0, 128.0),
        width: 128,
        height: 128,
        background: [0; 4],
    }
}
#[test]
fn text_is_shaped_ink_and_native_source_remains_editable() {
    let o = text("O");
    let original = o.clone();
    let scene = RenderSurface::extract(&surface(vec![o.clone()])).unwrap();
    let node = scene.node(o.id).unwrap();
    let bounds = node.visual_bounds().unwrap();
    assert!(bounds.width() < 100.0 && bounds.height() < 50.0);
    let pixels = CpuRenderer::default().render(&scene, request()).unwrap();
    assert!(pixels.data.as_chunks::<4>().0.iter().any(|p| p[3] > 0));
    assert_eq!(
        pixels
            .get_pixel(
                ((bounds.x0 + bounds.x1) * 0.5) as u32,
                ((bounds.y0 + bounds.y1) * 0.5) as u32
            )
            .unwrap()[3],
        0
    );
    assert_eq!(o, original);
}
#[test]
fn empty_and_whitespace_text_has_no_scene_damage_or_pixels() {
    for content in ["", " \n "] {
        let scene = RenderSurface::extract(&surface(vec![text(content)])).unwrap();
        assert!(scene
            .node(ObjectId::new(2))
            .unwrap()
            .visual_bounds()
            .is_none());
        let p = CpuRenderer::default().render(&scene, request()).unwrap();
        assert!(p.data.iter().all(|v| *v == 0));
    }
}
#[test]
fn group_rotation_and_opacity_apply_to_glyph_ink_once() {
    let mut group = DocumentObject::new(ObjectId::new(3), "Group");
    group.role = Some(ContainerRole::Group);
    group.bounds = Some([70.0, 10.0, 100.0, 50.0]);
    group.rotation = std::f64::consts::FRAC_PI_2;
    group.opacity = 0.5;
    group.children = vec![ObjectId::new(2)];
    let mut o = text("H");
    o.parent = Some(group.id);
    let scene = RenderSurface::extract(&surface(vec![group, o])).unwrap();
    let p = CpuRenderer::default().render(&scene, request()).unwrap();
    assert!(p.data.as_chunks::<4>().0.iter().any(|v| v[3] == 128));
    assert!(p.data.as_chunks::<4>().0.iter().all(|v| v[3] <= 128));
}
#[test]
fn live_crop_and_blur_fold_over_glyphs_without_baking() {
    let mut o = text("HELLO");
    let source = o.shape.clone();
    let mut crop = ModifierItem::enabled(
        1,
        ModifierKind::CropRect {
            rect: [0.0, 0.0, 20.0, 50.0],
        },
    );
    crop.space = ModifierSpace::Local {
        reference_size: [100.0, 50.0],
    };
    o.modifiers.push(crop);
    let scene = RenderSurface::extract(&surface(vec![o.clone()])).unwrap();
    let p = CpuRenderer::default().render(&scene, request()).unwrap();
    assert_eq!(p.get_pixel(30, 20).unwrap()[3], 0);
    assert!(p.data.as_chunks::<4>().0.iter().any(|p| p[3] > 0));
    let mut app = o.effective_appearance();
    app.effects.push(EffectItem {
        id: 1,
        kind: EffectKind::GaussianBlur { radius: 2.0 },
        visible: true,
    });
    o.appearance = Some(app);
    let blurred = CpuRenderer::default()
        .render(
            &RenderSurface::extract(&surface(vec![o.clone()])).unwrap(),
            request(),
        )
        .unwrap();
    assert_ne!(blurred, p);
    assert_eq!(o.shape, source);
}
#[test]
fn glyph_vector_mask_retains_the_counter_hole() {
    let mut mask = text("O");
    mask.is_clip_mask = true;
    let mut art = DocumentObject::new(ObjectId::new(3), "Art");
    art.bounds = Some([0.0, 0.0, 128.0, 128.0]);
    art.shape = Some(ShapeKind::Rectangle {
        corner_radii: [0.0; 4],
    });
    art.fill = Some("#0000ff".into());
    art.clip_mask_id = Some(mask.id);
    let scene = RenderSurface::extract(&surface(vec![mask, art])).unwrap();
    let b = scene
        .node(ObjectId::new(2))
        .unwrap()
        .visual_bounds()
        .unwrap();
    let p = CpuRenderer::default().render(&scene, request()).unwrap();
    assert!(p
        .data
        .as_chunks::<4>()
        .0
        .iter()
        .any(|p| p[2] == 255 && p[3] > 0));
    assert_eq!(
        p.get_pixel(((b.x0 + b.x1) * 0.5) as u32, ((b.y0 + b.y1) * 0.5) as u32)
            .unwrap()[3],
        0
    );
}
#[test]
fn scene_extraction_can_cancel_before_glyph_preparation() {
    assert!(matches!(
        RenderSurface::extract_cancellable(&surface(vec![text("cancel")]), &|| true),
        Err(RenderError::Cancelled)
    ));
}
#[test]
fn invisible_text_does_not_admit_unsupported_glyphs() {
    let mut o = text("\u{10ffff}");
    o.visible = false;
    let scene = RenderSurface::extract(&surface(vec![o])).unwrap();
    assert!(CpuRenderer::default()
        .render(&scene, request())
        .unwrap()
        .data
        .iter()
        .all(|v| *v == 0));
}

#[test]
fn attached_text_has_an_explicit_capability_reason_without_flattening_source() {
    let mut o = text("attached");
    if let Some(ShapeKind::Text { on_path, .. }) = &mut o.shape {
        *on_path = Some(petunia_design_document::TextOnPathAttachment::new(
            ObjectId::new(3),
            0.0,
            1.0,
        ));
    }
    let original = o.shape.clone();
    assert!(matches!(
        RenderSurface::extract(&surface(vec![o.clone()])),
        Err(RenderError::Unsupported {
            feature: "shaped text along a path",
            ..
        })
    ));
    assert_eq!(o.shape, original);
}
