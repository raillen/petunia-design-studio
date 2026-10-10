//! Behavior contracts for headless text and raster editing.
use petunia_core::{
    styles::{CharacterStyle, ParagraphStyle, StyleDefinition, StyleRegistry, TextAlignment},
    text::{
        CharacterStyleRef, FontRef, TextContainer, TextFlow, TextFrameSpec, TextObject,
        TextOverflow, TextRange, TextRun,
    },
    BuiltinColorSpace, ColorSource, ColorSpaceRef, ColorValue, PixelFormat, Point, ProcessColor,
    ProcessColorValue, Rgba, Size2, TileCoord,
};
use petunia_engine::{
    brush::{DabSpec, RasterStroke, RoundBrush},
    filter::{EdgeMode, FilterContext, GaussianBlur, ImageFilter, RectI, Surface},
    text::{evaluate_text, FontRegistry},
    tiles::TileStore,
};
fn fonts() -> FontRegistry {
    let mut registry = FontRegistry::new();
    registry.load_data(include_bytes!("fixtures/fonts/DejaVuSans.ttf").to_vec());
    registry.load_data(include_bytes!("fixtures/fonts/LiberationSans-Regular.ttf").to_vec());
    registry
}
fn character(family: &str, size: f64) -> CharacterStyle {
    CharacterStyle::new(
        FontRef::new(family, None, None, vec![]).unwrap(),
        size,
        ColorSource::Value(ColorValue::Process(ProcessColor {
            value: ProcessColorValue::Rgb(Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                alpha: 1.0,
            }),
            space: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
        })),
        0.0,
        0.0,
        "ar",
    )
    .unwrap()
}
fn paragraph() -> ParagraphStyle {
    ParagraphStyle::new(TextAlignment::Left, 16.0, 0.0, 0.0).unwrap()
}
fn text(source: &str, container: TextContainer) -> TextObject {
    TextObject::new(
        source.into(),
        vec![],
        vec![],
        container,
        TextFlow { next: None },
    )
    .unwrap()
}
#[test]
fn mixed_bidi_fallback_shapes_connected_arabic_and_keeps_source_clusters() {
    let object = text("Latin مرحبا world", TextContainer::Artistic);
    let original = object.clone();
    let result = evaluate_text(
        &object,
        &StyleRegistry::new(),
        &fonts(),
        &character("Liberation Sans", 12.0),
        &paragraph(),
    )
    .unwrap();
    assert_eq!(object, original);
    assert!(result.glyphs.iter().all(|glyph| glyph.glyph_id != 0));
    assert!(result
        .glyphs
        .iter()
        .any(|glyph| glyph.family == "Liberation Sans"));
    let arabic: Vec<_> = result
        .glyphs
        .iter()
        .filter(|glyph| glyph.family == "DejaVu Sans")
        .collect();
    assert_eq!(arabic.len(), 5);
    assert!(arabic
        .windows(2)
        .all(|pair| pair[0].source_byte > pair[1].source_byte));
    let ids: Vec<_> = arabic.iter().map(|glyph| glyph.glyph_id).collect();
    let isolated = evaluate_text(
        &text("م ر ح ب ا", TextContainer::Artistic),
        &StyleRegistry::new(),
        &fonts(),
        &character("DejaVu Sans", 12.0),
        &paragraph(),
    )
    .unwrap();
    assert_ne!(
        ids,
        isolated
            .glyphs
            .iter()
            .filter(|glyph| glyph.family == "DejaVu Sans")
            .map(|glyph| glyph.glyph_id)
            .collect::<Vec<_>>()
    );
    assert!(!result.outlines.is_empty());
}
#[test]
fn framed_styled_text_wraps_with_grapheme_safe_ranges_and_reports_overflow() {
    let mut object = text(
        "one e\u{301} two three four five",
        TextContainer::Frame(TextFrameSpec {
            size: Size2::new(55.0, 20.0).unwrap(),
            overflow: TextOverflow::Clip,
        }),
    );
    let id = petunia_core::StyleId::new_v4();
    let mut styles = StyleRegistry::new();
    styles.insert(
        id,
        StyleDefinition::Character(character("DejaVu Sans", 18.0)),
    );
    object.runs.push(TextRun {
        range: TextRange::new(0, 3, &object.text).unwrap(),
        style: CharacterStyleRef { style: id },
    });
    let result = evaluate_text(
        &object,
        &styles,
        &fonts(),
        &character("DejaVu Sans", 10.0),
        &paragraph(),
    )
    .unwrap();
    assert!(result.lines.len() > 2);
    assert!(result.overflow);
    assert_eq!(result.lines.first().unwrap().source.start, 0);
    assert_eq!(result.lines.last().unwrap().source.end, object.text.len());
    assert!(result
        .lines
        .iter()
        .all(|line| object.text.is_char_boundary(line.source.start)
            && object.text.is_char_boundary(line.source.end)));
    assert!(result
        .glyphs
        .windows(2)
        .all(|pair| pair[0].origin.y <= pair[1].origin.y));
    let mut doubled = character("DejaVu Sans", 20.0);
    doubled.baseline_shift = 3.0;
    let small = evaluate_text(
        &text("Hello", TextContainer::Artistic),
        &styles,
        &fonts(),
        &character("DejaVu Sans", 10.0),
        &paragraph(),
    )
    .unwrap();
    let big = evaluate_text(
        &text("Hello", TextContainer::Artistic),
        &styles,
        &fonts(),
        &doubled,
        &paragraph(),
    )
    .unwrap();
    assert!((big.lines[0].width / small.lines[0].width - 2.0).abs() < 1e-9);
    assert!((small.glyphs[0].origin.y - big.glyphs[0].origin.y - 3.0).abs() < 1e-9);
}
fn brush() -> RoundBrush {
    RoundBrush {
        color: [1.0, 0.0, 0.0, 1.0],
        hardness: 1.0,
        flow: 0.4,
        opacity: 0.5,
        blend: petunia_core::BlendMode::Normal,
    }
}
fn dab() -> DabSpec {
    DabSpec {
        position: Point::new(6.0, 6.0),
        diameter: 6.0,
        opacity: 1.0,
        jitter: 0.0,
    }
}
#[test]
fn round_dabs_touch_only_intersected_tiles_and_undo_replays_versions() {
    let mut source = TileStore::new(8, 64).unwrap();
    source
        .write_tile(TileCoord { x: 3, y: 3 }, 8, 8, PixelFormat::Rgba8Unorm)
        .unwrap()[0] = 77;
    source.clear_dirty();
    let untouched = source.get(TileCoord { x: 3, y: 3 }).unwrap().bytes.clone();
    let mut stroke = RasterStroke::begin(&source, 32, 32, brush()).unwrap();
    stroke.dab(dab(), None).unwrap();
    assert_eq!(source.len(), 1);
    assert_eq!(source.dirty_tiles(), vec![]);
    assert!(!stroke
        .preview()
        .dirty_tiles()
        .contains(&TileCoord { x: 3, y: 3 }));
    assert!(std::sync::Arc::ptr_eq(
        &untouched,
        &stroke
            .preview()
            .get(TileCoord { x: 3, y: 3 })
            .unwrap()
            .bytes
    ));
    let first = stroke
        .preview()
        .get(TileCoord { x: 0, y: 0 })
        .unwrap()
        .bytes[(6 * 8 + 6) * 4 + 3];
    for _ in 0..20 {
        stroke.dab(dab(), None).unwrap();
    }
    let alpha = stroke
        .preview()
        .get(TileCoord { x: 0, y: 0 })
        .unwrap()
        .bytes[(6 * 8 + 6) * 4 + 3];
    assert!(alpha > first);
    assert!(alpha <= 128);
    let transaction = stroke.finish();
    transaction.apply(&mut source);
    let committed = source.get(TileCoord { x: 0, y: 0 }).unwrap().bytes.clone();
    transaction.undo(&mut source);
    assert!(source.get(TileCoord { x: 0, y: 0 }).is_none());
    transaction.apply(&mut source);
    assert!(std::sync::Arc::ptr_eq(
        &committed,
        &source.get(TileCoord { x: 0, y: 0 }).unwrap().bytes
    ));
}
#[test]
fn selection_and_cancel_keep_source_unchanged_and_materialization_is_lossless_png() {
    let source = TileStore::new(8, 4).unwrap();
    let mut stroke = RasterStroke::begin(&source, 8, 8, brush()).unwrap();
    let mask = Surface::transparent(8, 8, 256).unwrap();
    stroke.dab(dab(), Some(&mask)).unwrap();
    assert!(stroke.preview().is_empty());
    stroke.dab(dab(), None).unwrap();
    let edit = stroke
        .materialize(petunia_core::ObjectId::new_v4())
        .unwrap();
    let decoded = image::load_from_memory(&edit.png).unwrap().to_rgba8();
    assert_eq!(decoded.dimensions(), (8, 8));
    assert_eq!(decoded.get_pixel(6, 6).0[0], 255);
    assert!(decoded.get_pixel(6, 6).0[3] > 0);
    assert_eq!(edit.request.operations.len(), 2);
    drop(stroke);
    assert!(source.is_empty());
}
#[test]
fn roi_matches_full_filter_in_all_edge_modes_without_transparent_color_halos() {
    let mut source = Surface::transparent(7, 5, 140).unwrap();
    for y in 0..5 {
        for x in 0..7 {
            source.set(x, y, 0, if x < 3 { 1.0 } else { 0.0 });
            source.set(x, y, 2, 1.0);
            source.set(x, y, 3, if x < 3 { 1.0 } else { 0.0 });
        }
    }
    let blur = GaussianBlur::new(1.0, 0.6).unwrap();
    for edge in [
        EdgeMode::Transparent,
        EdgeMode::Clamp,
        EdgeMode::Repeat,
        EdgeMode::Mirror,
    ] {
        let ctx = FilterContext {
            edge,
            max_pixels: 4096,
        };
        let mut full = Surface::transparent(7, 5, 140).unwrap();
        blur.evaluate(&ctx, &source, &mut full).unwrap();
        let mut roi = Surface::transparent(3, 3, 36).unwrap();
        blur.evaluate_region(&ctx, &source, RectI::new(0, 0, 3, 3).unwrap(), &mut roi)
            .unwrap();
        for y in 0..3 {
            for x in 0..3 {
                for channel in 0..4 {
                    assert!(
                        (roi.sample(x, y, channel, edge) - full.sample(x, y, channel, edge)).abs()
                            < 1e-6
                    );
                }
            }
        }
        for rgba in full
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|rgba| rgba[3] > 0.0)
        {
            assert!((rgba[0] - 1.0).abs() < 1e-6);
        }
    }
    assert_eq!(
        RectI::new(0, 0, 4, 4).unwrap().expanded(3),
        RectI::new(0, 0, 7, 7).unwrap()
    );
}
