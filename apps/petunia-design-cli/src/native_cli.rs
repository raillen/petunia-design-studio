//! Authored process-ink operations. No arguments keep the conformance runner;
//! explicit subcommands never turn a whole page into an unlabelled layer.
use petunia_design_application::{Command, CommandRequest, History};
use petunia_design_color::{IccProfile, IccTransformOptions};
use petunia_design_document::{Document, ShapeKind};
use petunia_design_foundation::{IdGenerator, ObjectId};
use petunia_design_io::{import_cmyk_tiff, open_package, save_package, write_cmyk_tiff};
use petunia_design_raster::RasterLayer;
use std::{path::Path, sync::Arc};

const HELP: &str = "Petunia Design Studio — native file operations
  cmyk-import INPUT.tif OUTPUT.PTND [--dpi 300]
  cmyk-export-layer INPUT.PTND OUTPUT.tif [--object ObjectId:2] [--dpi 300]
  cmyk-inspect INPUT.PTND [--object ObjectId:2] [--tac-percent 300]
  cmyk-assign INPUT.PTND PROFILE.icc OUTPUT.PTND [--object ObjectId:2]
  cmyk-convert INPUT.PTND PROFILE.icc OUTPUT.PTND [--object ObjectId:2]
  export-pdf INPUT.PTND OUTPUT.pdf [--dpi 144]

CMYK TIFF export writes one authored layer, before masks, effects and page
composition. Assignment preserves ink samples; conversion is explicit and
uses ICC relative colorimetric intent with black point compensation. Inspection
reports layer ink coverage; the optional TAC limit is a caller's press policy.
Without arguments the headless conformance flow runs.";

struct Arguments<'a> {
    paths: Vec<&'a str>,
    object: Option<ObjectId>,
    dpi: Option<f64>,
    tac: Option<f32>,
}
fn parse<'a>(command: &str, args: &'a [String]) -> Result<Arguments<'a>, String> {
    let mut out = Arguments {
        paths: Vec::new(),
        object: None,
        dpi: None,
        tac: None,
    };
    let mut iter = args.iter();
    while let Some(value) = iter.next() {
        match value.as_str() {
            "--object"
                if command != "cmyk-import" && command != "export-pdf" && out.object.is_none() =>
            {
                out.object = Some(
                    iter.next()
                        .ok_or("missing --object value")?
                        .parse()
                        .map_err(|e: petunia_design_foundation::PetuniaError| e.to_string())?,
                );
            }
            "--dpi"
                if matches!(command, "cmyk-import" | "cmyk-export-layer" | "export-pdf")
                    && out.dpi.is_none() =>
            {
                let dpi: f64 = iter
                    .next()
                    .ok_or("missing --dpi value")?
                    .parse()
                    .map_err(|_| "invalid DPI")?;
                if !dpi.is_finite() || !(1.0..=10000.).contains(&dpi) {
                    return Err("DPI must be between 1 and 10000".into());
                }
                out.dpi = Some(dpi);
            }
            "--tac-percent" if command == "cmyk-inspect" && out.tac.is_none() => {
                let tac: f32 = iter
                    .next()
                    .ok_or("missing --tac-percent value")?
                    .parse()
                    .map_err(|_| "invalid TAC")?;
                if !tac.is_finite() || !(0.0..=400.).contains(&tac) {
                    return Err("TAC limit must be between 0 and 400 percent".into());
                }
                out.tac = Some(tac);
            }
            option if option.starts_with('-') => {
                return Err(format!("unsupported or repeated option: {option}"))
            }
            path => out.paths.push(path),
        }
    }
    let count = match command {
        "cmyk-inspect" => 1,
        "cmyk-assign" | "cmyk-convert" => 3,
        _ => 2,
    };
    if out.paths.len() != count {
        return Err(format!(
            "{command} requires {count} path arguments; use --help"
        ));
    }
    Ok(out)
}
fn select_layer(
    document: &Document,
    requested: Option<ObjectId>,
) -> Result<(ObjectId, Arc<RasterLayer>), String> {
    let layers: Vec<_> = document
        .surfaces()
        .iter()
        .flat_map(|s| s.objects())
        .filter_map(|o| match &o.shape {
            Some(ShapeKind::Raster { layer })
                if layer.is_cmyk() && requested.is_none_or(|id| id == o.id) =>
            {
                Some((o.id, layer.clone()))
            }
            _ => None,
        })
        .collect();
    if layers.len() != 1 {
        return Err("select exactly one native CMYK raster with --object ObjectId:N; the document may contain zero or multiple native layers".into());
    }
    Ok(layers[0].clone())
}
pub fn run(args: &[String]) -> Result<(), String> {
    let command = args[0].as_str();
    if matches!(command, "--help" | "-h" | "help") {
        println!("{HELP}");
        return Ok(());
    }
    if !matches!(
        command,
        "cmyk-import"
            | "cmyk-export-layer"
            | "cmyk-inspect"
            | "cmyk-assign"
            | "cmyk-convert"
            | "export-pdf"
    ) {
        return Err(format!("unknown command: {command}; use --help"));
    }
    let args = parse(command, &args[1..])?;
    let input = Path::new(args.paths[0]);
    if command == "cmyk-import" {
        let source = petunia_design_io::read_encoded_image(input).map_err(|e| e.to_string())?;
        let layer = import_cmyk_tiff(
            source.as_slice(),
            petunia_design_raster::EncodedImage::MAX_BYTES,
        )
        .map_err(|e| e.to_string())?;
        let dpi = args.dpi.unwrap_or(300.);
        let size = [
            f64::from(layer.width()) * 72. / dpi,
            f64::from(layer.height()) * 72. / dpi,
        ];
        let profile = layer
            .cmyk_profile()
            .ok_or("CMYK layer has no ICC profile")?
            .clone();
        let mut ids = IdGenerator::new();
        let surface = ids.next_surface();
        let id = ids.next_object();
        let mut document = Document::new();
        let mut history = History::new(8);
        for command in [
            Command::CreateSurface {
                id: surface,
                name: "CMYK image".into(),
            },
            Command::SetSurfaceGeometry {
                surface,
                origin: [0.; 2],
                dimensions: size,
            },
            Command::SetSurfaceCmykProfile {
                surface,
                profile: Some(profile),
            },
            Command::CreateShapeObject {
                surface,
                id,
                name: "Native CMYK layer".into(),
                shape: ShapeKind::Raster {
                    layer: Arc::new(layer),
                },
                bounds: Some([0., 0., size[0], size[1]]),
                fill: None,
                stroke: None,
                stroke_width: 0.,
            },
        ] {
            history
                .execute(&mut document, &CommandRequest::new(command))
                .map_err(|e| e.to_string())?;
        }
        save_package(&document, Path::new(args.paths[1])).map_err(|e| e.to_string())?;
        println!(
            "{}",
            serde_json::json!({"object": id.to_string(), "dpi": dpi, "output": args.paths[1]})
        );
        return Ok(());
    }
    let mut document = open_package(input).map_err(|e| e.to_string())?.document;
    if command == "export-pdf" {
        let options = petunia_design_io::PdfExportOptions {
            raster_fallback_dpi: args.dpi.unwrap_or(144.),
            ..Default::default()
        };
        let (bytes, report) = petunia_design_io::export_document_pdf(&document, &options)
            .map_err(|e| e.to_string())?;
        petunia_design_io::atomic_output::write_atomic(Path::new(args.paths[1]), &bytes, &|| false)
            .map_err(|e| e.to_string())?;
        println!(
            "{}",
            serde_json::json!({"output": args.paths[1], "preflight": report})
        );
        return Ok(());
    }
    let (id, layer) = select_layer(&document, args.object)?;
    match command {
        "cmyk-export-layer" => {
            write_cmyk_tiff(
                &layer,
                Path::new(args.paths[1]),
                args.dpi.unwrap_or(300.),
                &|| false,
            )
            .map_err(|e| e.to_string())?;
            println!(
                "{}",
                serde_json::json!({"output": args.paths[1], "scope": "authored-layer", "object": id.to_string()})
            );
        }
        "cmyk-inspect" => {
            let report = layer
                .ink_coverage(args.tac, &|| false)
                .map_err(|e| e.to_string())?;
            println!(
                "{}",
                serde_json::json!({"scope": "authored-layer", "object": id.to_string(), "width": layer.width(), "height": layer.height(), "format": format!("{:?}", layer.tiles().format), "profile": layer.cmyk_profile().map(IccProfile::name), "maximum_percent": report.maximum_percent, "channel_maxima": report.channel_maxima, "visible_pixels": report.visible_pixels, "pixels_over_limit": report.pixels_over_limit, "limit_percent": report.limit_percent})
            );
        }
        "cmyk-assign" | "cmyk-convert" => {
            let profile = IccProfile::read(Path::new(args.paths[1])).map_err(|e| e.to_string())?;
            let command = if command == "cmyk-assign" {
                Command::AssignRasterCmykProfile { id, profile }
            } else {
                Command::SetShape {
                    id,
                    shape: Some(ShapeKind::Raster {
                        layer: Arc::new(
                            layer
                                .convert_cmyk_profile(
                                    profile,
                                    IccTransformOptions::default(),
                                    &|| false,
                                )
                                .map_err(|e| e.to_string())?,
                        ),
                    }),
                }
            };
            History::new(2)
                .execute(&mut document, &CommandRequest::new(command))
                .map_err(|e| e.to_string())?;
            save_package(&document, Path::new(args.paths[2])).map_err(|e| e.to_string())?;
            println!(
                "{}",
                serde_json::json!({"object": id.to_string(), "output": args.paths[2]})
            );
        }
        _ => unreachable!("validated subcommand"),
    }
    Ok(())
}
