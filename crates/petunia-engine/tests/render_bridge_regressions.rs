use petunia_core::*;
use petunia_engine::{compile_document, headless_frame, DocumentRevision};
use petunia_render::{RenderOptions, SoftwareRenderer};
use petunia_render_model::*;

fn render(document: &Document) -> Vec<u8> {
    document.validate().unwrap();
    let (snapshot, warnings) =
        compile_document(document, DocumentRevision::GENESIS, RenderQuality::Export);
    assert!(warnings.is_empty(), "{warnings:?}");
    SoftwareRenderer::new(1 << 20, 1 << 20)
        .render(
            &headless_frame(snapshot, 32, 32, 1.),
            &RenderOptions::default(),
        )
        .unwrap()
        .0
}
fn pixel(bytes: &[u8], x: usize, y: usize) -> &[u8] {
    &bytes[(y * 32 + x) * 4..(y * 32 + x) * 4 + 4]
}
fn insert(document: &mut Document, path: VectorPath) -> ObjectId {
    let page = document.scene.default_page();
    let node = SceneNode::new_path("path", path, ParentRef::Page(page));
    let id = node.id;
    document.scene.insert_root(page, node).unwrap();
    id
}

#[test]
fn group_opacity_applies_after_child_overlap_and_does_not_double_composite() {
    let mut document = Document::new("opacity");
    let page = document.scene.default_page();
    let mut group = SceneNode::new_path("group", VectorPath::new(), ParentRef::Page(page));
    group.item = SceneItem::Group(vec![]);
    group.opacity = 0.5;
    let id = group.id;
    document.scene.insert_root(page, group).unwrap();
    for x in [2., 6.] {
        document
            .scene
            .insert_child(
                id,
                SceneNode::new_path(
                    "child",
                    VectorPath::rect(x, 2., 10., 10.),
                    ParentRef::Object(id),
                ),
                None,
            )
            .unwrap();
    }
    let pixels = render(&document);
    assert_eq!(pixel(&pixels, 4, 6), pixel(&pixels, 8, 6));
    assert!((180..=190).contains(&pixel(&pixels, 8, 6)[0]));
}

#[test]
fn authorial_luminance_mask_and_blur_reach_pixels() {
    let mut document = Document::new("mask");
    let target = insert(&mut document, VectorPath::rect(2., 2., 26., 26.));
    let source = insert(&mut document, VectorPath::rect(2., 2., 8., 8.));
    document.scene.get_node_mut(target).unwrap().mask = Some(MaskBinding {
        source,
        mode: MaskMode::Luminance,
    });
    // Opaque black has zero luminance: the large target disappears; the source
    // remains an ordinary visible scene object in its own small rectangle.
    let pixels = render(&document);
    assert_eq!(pixel(&pixels, 20, 20), &[255, 255, 255, 255]);
    assert_eq!(pixel(&pixels, 5, 5), &[0, 0, 0, 255]);
    document.scene.get_node_mut(target).unwrap().mask = None;
    document
        .scene
        .get_node_mut(source)
        .unwrap()
        .post_effects
        .items
        .push(
            EffectInstance::new(
                true,
                1.,
                BlendMode::Normal,
                None,
                PostPaintEffect::GaussianBlur(BlurParams::new(1., 1.).unwrap()),
            )
            .unwrap(),
        );
    document.scene.get_node_mut(target).unwrap().visible = false;
    let pixels = render(&document);
    assert!(
        pixel(&pixels, 10, 5)[0] < 255,
        "blur must extend coverage beyond x=10"
    );
    assert_eq!(pixel(&pixels, 20, 20), &[255, 255, 255, 255]);
}

#[test]
fn renderer_rejects_target_and_kernel_that_exceed_working_limits_before_allocation() {
    let document = Document::new("limits");
    let (snapshot, _) =
        compile_document(&document, DocumentRevision::GENESIS, RenderQuality::Export);
    let mut frame = headless_frame(snapshot, u32::MAX, u32::MAX, 1.);
    let mut renderer = SoftwareRenderer::new(1 << 20, 1 << 20);
    assert!(renderer.render(&frame, &RenderOptions::default()).is_err());
    frame.target = RenderTarget {
        width: 32,
        height: 32,
    };
    frame.snapshot.pages[0]
        .primitives
        .push(RenderPrimitive::Group(RenderGroup {
            source: ObjectId::new_v4(),
            children: vec![],
            opacity: 1.,
            blend_mode: BlendMode::Normal,
            mask: None,
            clip: None,
            effects: vec![RenderEffect::Blur {
                sigma_x: 1e300,
                sigma_y: 1.,
            }],
            isolation: IsolationMode::Isolated,
            bounds: Rect::new(0., 0., 32., 32.),
        }));
    assert!(renderer.render(&frame, &RenderOptions::default()).is_err());
}

#[test]
fn expanded_symbol_instances_keep_distinct_selectable_sources() {
    let mut document = Document::new("symbols");
    let page = document.scene.default_page();
    let source = SceneNode::new_path(
        "definition path",
        VectorPath::rect(0., 0., 6., 6.),
        ParentRef::Page(page),
    );
    let source_id = source.id;
    let definition = SymbolDefinition::new(
        "box",
        vec![source_id],
        std::collections::BTreeMap::from([(source_id, source)]),
    )
    .unwrap();
    let symbol = definition.id;
    document.symbols.insert(definition);
    let mut ids = Vec::new();
    for x in [2., 20.] {
        let mut node = SceneNode::new_path("instance", VectorPath::new(), ParentRef::Page(page));
        node.item = SceneItem::SymbolInstance(SymbolInstance {
            definition: symbol,
            overrides: vec![],
        });
        node.transform = Transform2D::translation(x, 2.);
        ids.push(node.id);
        document.scene.insert_root(page, node).unwrap();
    }
    document.validate().unwrap();
    let (snapshot, warnings) =
        compile_document(&document, DocumentRevision::GENESIS, RenderQuality::Export);
    assert!(warnings.is_empty(), "{warnings:?}");
    use petunia_engine::spatial::*;
    let index = RStarIndex::bulk_load(entries_for_snapshot(&snapshot));
    for (id, x) in ids.into_iter().zip([4., 22.]) {
        let hits = hit_test_with_snapshot(
            &document,
            &snapshot,
            &index,
            HitTestRequest {
                page,
                point_document: Point::new(x, 4.),
                tolerance_px: 0.5,
                mode: HitTestMode::Fill,
            },
            Transform2D::IDENTITY,
        );
        assert_eq!(
            hits.iter().map(|hit| hit.object).collect::<Vec<_>>(),
            vec![id]
        );
    }
}

#[test]
fn transparent_shadow_preserves_coverage_and_checked_flatten_rejects_excess_work() {
    use petunia_engine::geometry::bezier::{try_flatten_contour, FlattenError};
    let mut document = Document::new("shadow alpha");
    let target = insert(&mut document, VectorPath::rect(2., 2., 6., 6.));
    let transparent = ColorSource::Value(ColorValue::Process(ProcessColor {
        space: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
        value: ProcessColorValue::Rgb(Rgba {
            r: 0.,
            g: 0.,
            b: 0.,
            alpha: 0.,
        }),
    }));
    document
        .scene
        .get_node_mut(target)
        .unwrap()
        .post_effects
        .items
        .push(
            EffectInstance::new(
                true,
                1.,
                BlendMode::Normal,
                None,
                PostPaintEffect::DropShadow(
                    ShadowParams::new(Vec2::new(12., 0.), Vec2::new(0., 0.), transparent, 0.)
                        .unwrap(),
                ),
            )
            .unwrap(),
        );
    let pixels = render(&document);
    assert_eq!(pixel(&pixels, 5, 5), &[0, 0, 0, 255]);
    assert_eq!(pixel(&pixels, 17, 5), &[255, 255, 255, 255]);
    let mut a = PathNode::new(Point::new(0., 0.), NodeKind::Cusp);
    a.handle_out = Some(Point::new(0., 1e100));
    let mut contour = Contour::new(false);
    contour.nodes = vec![a, PathNode::new(Point::new(10., 0.), NodeKind::Cusp)];
    assert!(matches!(
        try_flatten_contour(&contour, Tolerance(0.01), 65536),
        Err(FlattenError::DepthLimit | FlattenError::PointLimit)
    ));
    assert_eq!(
        try_flatten_contour(&contour, Tolerance(0.), 8),
        Err(FlattenError::InvalidInput)
    );
}
