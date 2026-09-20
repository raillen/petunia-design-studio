//! Headless MVP flow: document -> geometry -> color -> evaluation ->
//! scene -> native package -> reopen. No GUI types here by construction.
//!
//! Minimum E2E from 07: create document → draw path/shape → apply semantic
//! color → undo/redo → save/reopen → export summary.

use aubrieta_application::{Command, CommandRequest, History};
use aubrieta_color::{convert_for_display, ColorValue, Lab, RenderingIntent};
use aubrieta_document::Document;
use aubrieta_evaluation::Evaluator;
use aubrieta_foundation::IdGenerator;
use aubrieta_geometry::{boolean_op, BooleanInput, BooleanOp, GAffine, GPath, GPoint, PathVerb};
use aubrieta_platform::{
    ClipboardService, EnvironmentService, FileFilter, HeadlessClipboard, HeadlessEnvironment,
};
use aubrieta_raster::{AlphaMode, BlendMode, BrushDab, PixelFormat, TileMap};
use aubrieta_render::{HeadlessSummaryBackend, RenderBackend, Scene};
use aubrieta_resources::{
    IconId, Locale, ResourcePack, TextId, ThemeMode, ID_ACTION_EXPORT, ID_EXPORT_SUMMARY,
};
use aubrieta_text::{TextLayout, TextOffset, TextStory};
use std::collections::HashMap;

fn main() {
    if let Err(error) = run() {
        eprintln!("aubrieta-cli: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut ids = IdGenerator::new();
    let mut document = Document::new();
    let mut history = History::new(100);
    let mut evaluator = Evaluator::new();

    // 1. Document: surface + shape.
    let surface = ids.next_surface();
    let object = ids.next_object();
    for request in [
        CommandRequest::new(Command::CreateSurface {
            id: surface,
            name: "Page 1".to_string(),
        }),
        CommandRequest::new(Command::CreateObject {
            surface,
            id: object,
            name: "Rect".to_string(),
        }),
        CommandRequest::new(Command::SetFill {
            id: object,
            fill: Some("aubrieta.red/500".to_string()),
        }),
    ] {
        let changes = history
            .execute(&mut document, &request)
            .map_err(|e| format!("execute: {e}"))?;
        evaluator.note_changes(&changes);
    }

    // 2. Geometry: triangle path, bounds, translated bounds, boolean area.
    let mut triangle = GPath::new();
    for verb in [
        PathVerb::MoveTo(GPoint::new(0.0, 0.0)),
        PathVerb::LineTo(GPoint::new(4.0, 0.0)),
        PathVerb::LineTo(GPoint::new(0.0, 3.0)),
        PathVerb::Close,
    ] {
        triangle.push(verb).map_err(|e| format!("path: {e}"))?;
    }
    let bounds = triangle.bounding_box().ok_or("triangle bounds")?;
    assert_eq!((bounds.x1, bounds.y1), (4.0, 3.0));
    let moved = triangle.transformed(GAffine::translate(10.0, 0.0));
    let moved_bounds = moved.bounding_box().ok_or("moved bounds")?;
    assert_eq!((moved_bounds.x0, moved_bounds.y0), (10.0, 0.0));
    let union = boolean_op(
        &BooleanInput::single(vec![
            GPoint::new(0.0, 0.0),
            GPoint::new(2.0, 0.0),
            GPoint::new(2.0, 2.0),
            GPoint::new(0.0, 2.0),
        ]),
        &BooleanInput::single(vec![
            GPoint::new(1.0, 1.0),
            GPoint::new(3.0, 1.0),
            GPoint::new(3.0, 3.0),
            GPoint::new(1.0, 3.0),
        ]),
        BooleanOp::Union,
    );
    assert_eq!(union.len(), 1, "union must be one contour");

    // 3. Color: semantic fill → display sRGB; Lab white landmark.
    let preview = convert_for_display(
        &ColorValue::Lab(Lab {
            l: 100.0,
            a: 0.0,
            b: 0.0,
        }),
        RenderingIntent::Perceptual,
    );
    assert!(
        (preview.r - 1.0).abs() < 0.01
            && (preview.g - 1.0).abs() < 0.01
            && (preview.b - 1.0).abs() < 0.01,
        "Lab white must preview white, got {preview:?}"
    );

    // 4. Evaluation cache + undo/redo through history.
    let summary = evaluator.evaluate(&document);
    assert_eq!((summary.surfaces, summary.objects), (1, 1));
    assert!(history
        .undo(&mut document)
        .map_err(|e| format!("undo: {e}"))?);
    assert!(history
        .redo(&mut document)
        .map_err(|e| format!("redo: {e}"))?);

    // 5. Scene render summary (headless reference).
    let scene = Scene::extract(&document);
    let rendered = HeadlessSummaryBackend
        .render(&scene)
        .map_err(|e| format!("render: {e}"))?;
    assert!(rendered.contains("aubrieta.red/500"));

    // 6. Native package save/reopen (atomic), both suffixes.
    let dir = std::env::temp_dir().join("aubrieta-cli");
    std::fs::create_dir_all(&dir).map_err(|e| format!("temp dir: {e}"))?;
    for suffix in ["aubrieta", "aubri"] {
        let path = dir.join(format!("mvp.{suffix}"));
        aubrieta_io::save_package(&document, &path).map_err(|e| format!("save: {e}"))?;
        let reopened = aubrieta_io::open_package(&path).map_err(|e| format!("reopen: {e}"))?;
        assert_eq!(document, reopened, "package roundtrip must preserve");
        std::fs::remove_file(&path).map_err(|e| format!("cleanup: {e}"))?;
    }

    // 7. Typography (aubrieta_text): story, runs, line wrap, hit test.
    let text_story_id = ids.next_text_story();
    let mut story =
        TextStory::with_content(text_story_id, "Aubrieta Creative Suite\nTypography Engine");
    story
        .insert_str(TextOffset::new(8), " Professional")
        .map_err(|e| format!("text insert: {e}"))?;
    let text_layout = TextLayout::layout(&story, Some(250.0));
    assert_eq!(text_layout.line_count, 3);
    let hit_offset = text_layout.hit_test(GPoint::new(10.0, 10.0));
    assert!(hit_offset.0 <= story.content.len());

    // 8. Raster Engine (aubrieta_raster): sparse 128x128 tiles, 16-bit depth, brush dab.
    let mut tile_map = TileMap::new(PixelFormat::Rgba16, AlphaMode::Straight);
    let dab = BrushDab {
        center_x: 64.0,
        center_y: 64.0,
        radius: 12.0,
        hardness: 0.8,
        opacity: 0.9,
        color: [1.0, 0.2, 0.1, 1.0],
        blend_mode: BlendMode::Normal,
    };
    dab.stamp_onto(&mut tile_map);
    tile_map.commit();
    let raster_bounds = tile_map.bounds().ok_or("raster bounds")?;
    assert_eq!(tile_map.resident_tile_count(), 1);

    // 9. SVG vector export (aubrieta_io): document envelope and path roundtrip.
    let svg_content = aubrieta_io::export_document_svg(&document);
    assert!(svg_content.contains("<svg"));
    let path_d = aubrieta_io::export_path_d(&triangle);
    let parsed_path = aubrieta_io::parse_path_d(&path_d).map_err(|e| format!("svg parse: {e}"))?;
    assert_eq!(parsed_path.verbs.len(), triangle.verbs.len());

    // 10. Resource Pack, DTCG Tokens & i18n (aubrieta_resources).
    let core_pack = ResourcePack::core_pack();
    let light_token = core_pack
        .tokens
        .resolve("surface.canvas", Some(ThemeMode::Light))
        .map_err(|e| format!("token light: {e}"))?;
    let dark_token = core_pack
        .tokens
        .resolve("surface.canvas", Some(ThemeMode::Dark))
        .map_err(|e| format!("token dark: {e}"))?;
    assert_ne!(light_token, dark_token);

    let mut i18n_params = HashMap::new();
    i18n_params.insert("count".to_string(), "1".to_string());
    i18n_params.insert("format".to_string(), "SVG".to_string());
    let en_msg =
        core_pack
            .localization
            .format(&TextId::new(ID_EXPORT_SUMMARY), &Locale::EnUs, &i18n_params);
    let pt_msg =
        core_pack
            .localization
            .format(&TextId::new(ID_EXPORT_SUMMARY), &Locale::PtBr, &i18n_params);
    assert!(en_msg.contains("Exported 1 items"));
    assert!(pt_msg.contains("Exportados 1 itens"));
    assert!(core_pack
        .icons
        .get(&IconId::new(ID_ACTION_EXPORT))
        .is_some());

    // 11. Platform Services & In-Memory Clipboard (aubrieta_platform).
    let mut clipboard = HeadlessClipboard::new();
    clipboard
        .set_text(&svg_content)
        .map_err(|e| format!("clipboard set: {e}"))?;
    let pasted_svg = clipboard
        .get_text()
        .map_err(|e| format!("clipboard get: {e}"))?
        .ok_or("clipboard empty")?;
    assert_eq!(pasted_svg, svg_content);

    let filter = FileFilter::AubrietaPackage;
    assert!(filter.matches(std::path::Path::new("mvp.aubrieta")));

    let platform_env = HeadlessEnvironment::new()
        .get_environment()
        .map_err(|e| format!("env query: {e}"))?;
    assert_eq!(platform_env.scale_factor, 1.0);

    println!(
        "OK surfaces={} objects={} gen={} bounds=({},{}) union_contours={} white=({:.2},{:.2},{:.2}) lines={} raster_tiles={} svg_len={} tokens={} locales=2 clip_len={}",
        summary.surfaces,
        summary.objects,
        summary.generation,
        bounds.x1,
        bounds.y1,
        union.len(),
        preview.r,
        preview.g,
        preview.b,
        text_layout.line_count,
        tile_map.resident_tile_count(),
        svg_content.len(),
        core_pack.tokens.len(),
        pasted_svg.len(),
    );
    println!("i18n en-US: \"{en_msg}\"");
    println!("i18n pt-BR: \"{pt_msg}\"");
    println!(
        "Raster bounds: ({:.0},{:.0}) to ({:.0},{:.0})",
        raster_bounds.x0, raster_bounds.y0, raster_bounds.x1, raster_bounds.y1
    );
    println!("{rendered}");
    Ok(())
}
