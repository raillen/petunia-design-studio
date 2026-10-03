//! Automated project scenarios. Human task acceptance is a separate release gate.
use petunia_design_application::{
    export_service::{export_document, ExportFormat, ExportRequest},
    Command, CommandRequest, History, Transaction,
};
use petunia_design_document::{Document, ShapeKind};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_io::{open_package, save_package};
use petunia_design_raster::{BitDepth, RasterLayer, RasterLayerKind};
use std::sync::Arc;
fn execute(doc: &mut Document, history: &mut History, command: Command) {
    history.execute(doc, &CommandRequest::new(command)).unwrap();
}
fn project(kind: &str) -> (Document, History) {
    let mut doc = Document::new();
    let mut history = History::new(64);
    let surface = SurfaceId::new(1);
    execute(
        &mut doc,
        &mut history,
        Command::CreateSurface {
            id: surface,
            name: kind.into(),
        },
    );
    execute(
        &mut doc,
        &mut history,
        Command::SetSurfaceGeometry {
            surface,
            origin: [-100., -80.],
            dimensions: [64., 64.],
        },
    );
    execute(
        &mut doc,
        &mut history,
        Command::SetSurfaceBackground {
            surface,
            background: None,
        },
    );
    let shape = match kind {
        "poster" => ShapeKind::Text {
            content: "Petúnia\nDesign".into(),
            font_family: "DejaVu Sans".into(),
            font_size: 12.,
            line_height: 1.2,
            letter_spacing: 0.,
            on_path: None,
        },
        "painting" => {
            let mut layer =
                RasterLayer::new(16, 16, RasterLayerKind::Pixels, BitDepth::Sixteen).unwrap();
            for y in 0..16 {
                for x in 0..16 {
                    layer
                        .set_pixel(x, y, [x as f32 / 15., y as f32 / 15., 0.5, 1.])
                        .unwrap();
                }
            }
            layer.commit();
            ShapeKind::Raster {
                layer: Arc::new(layer),
            }
        }
        _ => ShapeKind::Star {
            points: 5,
            inner_ratio: 0.5,
        },
    };
    execute(
        &mut doc,
        &mut history,
        Command::CreateShapeObject {
            surface,
            id: ObjectId::new(2),
            name: "Artwork".into(),
            shape,
            bounds: Some([-96., -76., 56., 56.]),
            fill: Some("#3498db".into()),
            stroke: None,
            stroke_width: 0.,
        },
    );
    if kind == "painting" {
        execute(
            &mut doc,
            &mut history,
            Command::CreateShapeObject {
                surface,
                id: ObjectId::new(3),
                name: "Mask".into(),
                shape: ShapeKind::Ellipse,
                bounds: Some([-88., -68., 40., 40.]),
                fill: Some("#ffffff".into()),
                stroke: None,
                stroke_width: 0.,
            },
        );
        execute(
            &mut doc,
            &mut history,
            Command::CreateClipGroup {
                surface,
                group_id: ObjectId::new(4),
                mask_id: ObjectId::new(3),
                content_ids: vec![ObjectId::new(2)],
            },
        );
    }
    if kind == "artboards" {
        let second = SurfaceId::new(5);
        execute(
            &mut doc,
            &mut history,
            Command::CreateSurface {
                id: second,
                name: "Second".into(),
            },
        );
        execute(
            &mut doc,
            &mut history,
            Command::SetSurfaceGeometry {
                surface: second,
                origin: [200., 100.],
                dimensions: [32., 48.],
            },
        );
        execute(
            &mut doc,
            &mut history,
            Command::CreateShapeObject {
                surface: second,
                id: ObjectId::new(6),
                name: "Second artwork".into(),
                shape: ShapeKind::Ellipse,
                bounds: Some([204., 104., 20., 32.]),
                fill: Some("#ef4f78".into()),
                stroke: None,
                stroke_width: 0.,
            },
        );
    }
    (doc, history)
}
fn exercise(kind: &str) {
    let (mut doc, mut history) = project(kind);
    let baseline = doc.clone();
    let mut cancelled = Transaction::begin(&doc, "Cancelled change");
    cancelled
        .update(&CommandRequest::new(Command::RenameObject {
            id: ObjectId::new(2),
            name: "Never committed".into(),
        }))
        .unwrap();
    cancelled.cancel();
    assert_eq!(doc, baseline);
    execute(
        &mut doc,
        &mut history,
        Command::RenameObject {
            id: ObjectId::new(2),
            name: "Edited".into(),
        },
    );
    let edited = doc.clone();
    history.undo(&mut doc).unwrap();
    assert_eq!(doc, baseline);
    history.redo(&mut doc).unwrap();
    assert_eq!(doc, edited);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(format!("{kind}.PTND"));
    save_package(&doc, &path).unwrap();
    let restored = open_package(&path).unwrap().document;
    assert_eq!(restored, doc);
    for format in [ExportFormat::Svg, ExportFormat::Png, ExportFormat::Pdf] {
        let request = ExportRequest::new(format, dir.path().join(kind));
        let report = export_document(&restored, &request).unwrap();
        assert!(report.degradations.is_empty(), "{:?}", report.degradations);
        let bytes = std::fs::read(&request.path).unwrap();
        assert!(!bytes.is_empty());
        match format {
            ExportFormat::Svg => {
                assert_svg_artifact(&bytes);
            }
            ExportFormat::Png => {
                let image = petunia_design_io::import_raster(&bytes, 1024 * 1024).unwrap();
                assert_eq!((image.width, image.height), (64, 64));
            }
            ExportFormat::Pdf => assert!(bytes.starts_with(b"%PDF-")),
        }
    }
    assert_eq!(doc, edited);
    assert_eq!(restored, edited);
}
fn assert_svg_artifact(bytes: &[u8]) {
    let svg = std::str::from_utf8(bytes).unwrap();
    assert!(svg.contains("<svg") && svg.contains("</svg>"));
}
#[test]
fn logo_commands_history_native_and_exports() {
    exercise("logo");
}
#[test]
fn poster_commands_history_native_and_exports() {
    exercise("poster");
}
#[test]
fn masked_painting_commands_history_native_and_exports() {
    exercise("painting");
}
#[test]
fn multiple_artboards_commands_history_native_and_exports() {
    exercise("artboards");
}
