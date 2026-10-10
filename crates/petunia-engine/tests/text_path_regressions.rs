use petunia_core::*;
use petunia_engine::text::{evaluate_text_on_path, FontRegistry};

#[test]
fn loaded_font_bytes_remain_shared_and_available_without_the_source_file() {
    let path = std::env::temp_dir().join(format!("petunia-font-snapshot-{}", DocumentId::new_v4()));
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(
        path.join("font.ttf"),
        include_bytes!("fixtures/fonts/DejaVuSans.ttf"),
    )
    .unwrap();
    let mut fonts = FontRegistry::new();
    fonts.load_dir(&path);
    std::fs::remove_dir_all(&path).unwrap();
    let query = petunia_engine::text::FontQuery {
        families: vec!["DejaVu Sans".into()],
        weight: 400,
        italic: false,
    };
    let first = fonts.resolve(&query).unwrap();
    let second = fonts.resolve(&query).unwrap();
    assert!(std::sync::Arc::ptr_eq(&first.bytes, &second.bytes));
    assert!(fonts.resolve_cluster(&query, "مرحبا").is_some());
}

#[test]
fn linked_frames_consume_whole_lines_and_compile_each_frame_once() {
    let mut fonts = FontRegistry::new();
    fonts.load_data(include_bytes!("fixtures/fonts/DejaVuSans.ttf").to_vec());
    let character = CharacterStyle::new(
        FontRef::new("DejaVu Sans", None, None, vec![]).unwrap(),
        12.,
        ColorSource::Value(ColorValue::Process(ProcessColor {
            space: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
            value: ProcessColorValue::Rgb(Rgba {
                r: 0.,
                g: 0.,
                b: 0.,
                alpha: 1.,
            }),
        })),
        0.,
        0.,
        "en",
    )
    .unwrap();
    let paragraph = ParagraphStyle::new(TextAlignment::Left, 16., 0., 0.).unwrap();
    let mut document = Document::new("flow");
    let page = document.scene.default_page();
    let mut first = SceneNode::new_path("first", VectorPath::new(), ParentRef::Page(page));
    let mut second = SceneNode::new_path("second", VectorPath::new(), ParentRef::Page(page));
    let a = first.id;
    let b = second.id;
    let container = TextContainer::Frame(TextFrameSpec {
        size: Size2::new(100., 16.).unwrap(),
        overflow: TextOverflow::Clip,
    });
    let mut source = TextObject::new(
        "First\nSecond".into(),
        vec![],
        vec![],
        container.clone(),
        TextFlow { next: Some(b) },
    )
    .unwrap();
    let tail = TextObject::new(
        String::new(),
        vec![],
        vec![],
        container,
        TextFlow { next: None },
    )
    .unwrap();
    let original = source.clone();
    let layouts = petunia_engine::text::evaluate_text_flow(
        &[(a, &source), (b, &tail)],
        &document.styles,
        &fonts,
        &character,
        &paragraph,
    )
    .unwrap();
    assert_eq!(source, original);
    assert_eq!(layouts[&a].glyphs.len(), 5);
    assert_eq!(layouts[&b].glyphs.len(), 6);
    assert_eq!(layouts[&b].glyphs[0].source_byte, 6);
    assert!(!layouts[&a].overflow && !layouts[&b].overflow);
    first.item = SceneItem::Text(source.clone());
    second.item = SceneItem::Text(tail.clone());
    second.transform = Transform2D::translation(120., 0.);
    document.scene.insert_root(page, first).unwrap();
    document.scene.insert_root(page, second).unwrap();
    document.validate().unwrap();
    let providers = petunia_engine::compile::CompileProviders {
        fonts: Some(&fonts),
        default_character: Some(&character),
        default_paragraph: Some(&paragraph),
        ..Default::default()
    };
    let (snapshot, warnings) = petunia_engine::compile::compile_document_with_providers(
        &document,
        petunia_engine::DocumentRevision::GENESIS,
        petunia_render_model::RenderQuality::Export,
        &providers,
    );
    assert!(warnings.is_empty(), "{warnings:?}");
    let counts: Vec<_> = snapshot.pages[0]
        .primitives
        .iter()
        .map(|primitive| match primitive {
            petunia_render_model::RenderPrimitive::Group(group) => group.children.len(),
            _ => panic!("expected clipped text frame"),
        })
        .collect();
    assert_eq!(counts, vec![5, 6]);
    source.flow.next = Some(a);
    assert!(petunia_engine::text::evaluate_text_flow(
        &[(a, &source), (b, &tail)],
        &document.styles,
        &fonts,
        &character,
        &paragraph
    )
    .is_err());
}

#[test]
fn text_follows_path_tangent_and_start_offset_without_mutating_authorial_content() {
    let mut fonts = FontRegistry::new();
    fonts.load_data(include_bytes!("fixtures/fonts/DejaVuSans.ttf").to_vec());
    let character = CharacterStyle::new(
        FontRef::new("DejaVu Sans", None, None, vec![]).unwrap(),
        12.,
        ColorSource::Value(ColorValue::Process(ProcessColor {
            space: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
            value: ProcessColorValue::Rgb(Rgba {
                r: 0.,
                g: 0.,
                b: 0.,
                alpha: 1.,
            }),
        })),
        0.,
        0.,
        "en",
    )
    .unwrap();
    let paragraph = ParagraphStyle::new(TextAlignment::Left, 16., 0., 0.).unwrap();
    let text = TextObject::new(
        "Ab".into(),
        vec![],
        vec![],
        TextContainer::OnPath(TextPathRef::new(ObjectId::new_v4(), 5.).unwrap()),
        TextFlow { next: None },
    )
    .unwrap();
    let original = text.clone();
    let mut contour = Contour::new(false);
    contour.nodes = vec![
        PathNode::line(Point::new(20., 30.), NodeKind::Cusp),
        PathNode::line(Point::new(20., 130.), NodeKind::Cusp),
    ];
    let mut path = VectorPath::new();
    path.push_contour(contour);
    let layout = evaluate_text_on_path(
        &text,
        &path,
        &StyleRegistry::new(),
        &fonts,
        &character,
        &paragraph,
    )
    .unwrap();
    assert_eq!(text, original);
    assert_eq!(layout.glyphs.len(), 2);
    assert!((layout.glyphs[0].origin.x - 20.).abs() < 1e-6);
    assert!((layout.glyphs[0].origin.y - 35.).abs() < 1e-6);
    assert!(layout.glyphs[1].origin.y > layout.glyphs[0].origin.y);
    assert!(!layout.overflow);
    assert!(layout
        .outlines
        .iter()
        .flat_map(|outline| &outline.path.contours)
        .flat_map(|contour| &contour.nodes)
        .any(|node| node.point.x > 20.));
    path.contours[0].nodes[1].point = Point::new(20., 36.);
    assert!(
        evaluate_text_on_path(
            &text,
            &path,
            &StyleRegistry::new(),
            &fonts,
            &character,
            &paragraph
        )
        .unwrap()
        .overflow
    );
}
