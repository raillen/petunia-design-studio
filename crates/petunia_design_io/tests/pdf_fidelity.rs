use petunia_design_document::{
    AppearanceStack, Document, DocumentMutator, DocumentObject, FillItem, GradientStop,
    LinearGradient, ShapeKind,
};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_geometry::GRect;
use petunia_design_io::{export_document_pdf, PdfExportOptions};
use petunia_design_render::{CpuRenderer, RenderRequest, RenderSurface};
use std::{path::Path, process::Command, sync::Arc};
fn object(id: u64, bounds: [f64; 4], shape: ShapeKind, color: &str) -> DocumentObject {
    let mut object = DocumentObject::new(ObjectId::new(id), "Art");
    object.bounds = Some(bounds);
    object.shape = Some(shape);
    object.fill = Some(color.into());
    object
}
fn document() -> Document {
    let mut doc = Document::new();
    let mut m = DocumentMutator::new(&mut doc);
    m.add_surface(SurfaceId::new(1), "Page").unwrap();
    m.set_surface_geometry(SurfaceId::new(1), [-300., 800.], [96., 64.])
        .unwrap();
    doc
}
fn poppler(program: &str, args: &[&str], path: &Path) -> String {
    let output = Command::new(program)
        .args(args)
        .arg(path)
        .output()
        .expect("poppler-utils required for PDF interoperability");
    assert!(
        output.status.success(),
        "{program}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}
fn compare_cpu_pdf(doc: &Document, options: PdfExportOptions) {
    let (bytes, _) = export_document_pdf(doc, &options).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("page.pdf");
    std::fs::write(&path, bytes).unwrap();
    let prefix = dir.path().join("page");
    let output = Command::new("pdftoppm")
        .args(["-r", "72", "-png", "-singlefile"])
        .arg(&path)
        .arg(&prefix)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual = image::open(prefix.with_extension("png"))
        .unwrap()
        .to_rgba8();
    let surface = &doc.surfaces()[0];
    let scene = RenderSurface::extract(surface).unwrap();
    let expected = CpuRenderer::default()
        .render(
            &scene,
            RenderRequest {
                width: 96,
                height: 64,
                viewport: GRect::new(-300., 800., -204., 864.),
                background: [255; 4],
            },
        )
        .unwrap();
    assert_eq!(actual.dimensions(), (96, 64));
    // Independent renderer artifacts are opt-in evidence, never inferred from
    // the exported bytes or the scene alone.
    if let Some(directory) = std::env::var_os("PETUNIA_PDF_EVIDENCE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        let thread = std::thread::current();
        let name = thread.name().unwrap_or("comparison");
        std::fs::copy(&path, directory.join(format!("{name}.pdf"))).unwrap();
        actual
            .save(directory.join(format!("{name}-poppler.png")))
            .unwrap();
        image::save_buffer(
            directory.join(format!("{name}-cpu.png")),
            &expected.data,
            96,
            64,
            image::ColorType::Rgba8,
        )
        .unwrap();
    }
    assert!(
        expected
            .data
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| pixel[..3] != [255; 3])
            .count()
            > 100,
        "comparison must contain visible artwork"
    );
    let error = actual
        .as_raw()
        .as_chunks::<4>()
        .0
        .iter()
        .zip(expected.data.as_chunks::<4>().0.iter())
        .map(|(a, b)| {
            a[..3]
                .iter()
                .zip(&b[..3])
                .map(|(x, y)| u64::from(x.abs_diff(*y)))
                .sum::<u64>()
        })
        .sum::<u64>() as f64
        / (96. * 64. * 3.);
    assert!(error < 4.0, "PDF/CPU mean channel error {error}");
}
#[test]
fn transformed_clipped_vector_gradient_and_multiple_fills_match_an_independent_renderer() {
    let mut doc = document();
    let mut m = DocumentMutator::new(&mut doc);
    let mask = object(2, [-292., 808., 55., 40.], ShapeKind::Ellipse, "#ffffff");
    let mut art = object(
        3,
        [-292., 808., 70., 40.],
        ShapeKind::Rectangle {
            corner_radii: [5.; 4],
        },
        "#ff0000",
    );
    let mut app = AppearanceStack::default();
    app.fills.push(FillItem::linear_gradient(
        1,
        LinearGradient::new(
            [0., 0.],
            [70., 0.],
            vec![
                GradientStop::new(0., "#ff0000"),
                GradientStop::new(1., "#0000ff"),
            ],
        ),
    ));
    let mut overlay = FillItem::solid(2, "#00ff0080");
    overlay.opacity = 0.25;
    app.fills.push(overlay);
    art.appearance = Some(app);
    m.add_object(SurfaceId::new(1), mask).unwrap();
    m.add_object(SurfaceId::new(1), art).unwrap();
    m.create_clip_group(
        SurfaceId::new(1),
        ObjectId::new(4),
        ObjectId::new(2),
        vec![ObjectId::new(3)],
    )
    .unwrap();
    m.set_bounds(
        ObjectId::new(4),
        Some([-287., 811., 55., 40.]),
        12_f64.to_radians(),
    )
    .unwrap();
    compare_cpu_pdf(&doc, Default::default());
}
#[test]
fn sixteen_bit_raster_pixels_and_alpha_are_embedded_without_a_rectangle_substitute() {
    let mut doc = document();
    let mut layer = petunia_design_raster::RasterLayer::new(
        4,
        4,
        petunia_design_raster::RasterLayerKind::Pixels,
        petunia_design_raster::BitDepth::Sixteen,
    )
    .unwrap();
    for y in 0..4 {
        for x in 0..4 {
            layer
                .set_pixel(x, y, [x as f32 / 3., y as f32 / 3., 0.25, 0.75])
                .unwrap();
        }
    }
    layer.commit();
    DocumentMutator::new(&mut doc)
        .add_object(
            SurfaceId::new(1),
            object(
                2,
                [-280., 818., 40., 32.],
                ShapeKind::Raster {
                    layer: Arc::new(layer),
                },
                "#ff0000",
            ),
        )
        .unwrap();
    compare_cpu_pdf(&doc, Default::default());
}
#[test]
fn actual_shaped_fonts_are_embedded_and_text_is_extractable() {
    let mut doc = document();
    let text = ShapeKind::Text {
        content: "Petunia fi".into(),
        font_family: "DejaVu Sans".into(),
        font_size: 12.,
        line_height: 1.2,
        letter_spacing: 0.,
        on_path: None,
    };
    DocumentMutator::new(&mut doc)
        .add_object(
            SurfaceId::new(1),
            object(2, [-295., 805., 90., 50.], text, "#123456"),
        )
        .unwrap();
    let (bytes, report) = export_document_pdf(&doc, &Default::default()).unwrap();
    assert!(report.degradations.is_empty(), "{:?}", report.degradations);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("text.pdf");
    std::fs::write(&path, bytes).unwrap();
    let fonts = poppler("pdffonts", &[], &path);
    assert!(fonts.contains("DejaVu"), "{fonts}");
    assert!(
        fonts.lines().skip(2).any(|row| row
            .split_whitespace()
            .filter(|word| *word == "yes")
            .count()
            >= 2),
        "{fonts}"
    );
    let output = Command::new("pdftotext")
        .arg(&path)
        .arg("-")
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("Petunia") && text.contains("fi"), "{text}");
}
#[test]
fn effects_require_explicit_degradation_and_fallback_reuses_canonical_pixels() {
    let mut doc = document();
    let mut art = object(2, [-278., 816., 30., 22.], ShapeKind::Ellipse, "#ff0000");
    let mut app = AppearanceStack::new().with_fill("#ff0000");
    app.effects.push(petunia_design_document::EffectItem {
        id: 1,
        visible: true,
        kind: petunia_design_document::EffectKind::GaussianBlur { radius: 2. },
    });
    art.appearance = Some(app);
    DocumentMutator::new(&mut doc)
        .add_object(SurfaceId::new(1), art)
        .unwrap();
    assert!(export_document_pdf(&doc, &Default::default()).is_err());
    let options = PdfExportOptions {
        allow_degradations: true,
        raster_fallback_dpi: 72.,
        ..Default::default()
    };
    let (_, report) = export_document_pdf(&doc, &options).unwrap();
    assert!(report
        .degradations
        .iter()
        .any(|d| d.code == "PAGE_RASTERIZED"));
    compare_cpu_pdf(&doc, options);
}
#[test]
fn unprofiled_process_inks_and_cancellation_fail_before_publication() {
    let mut doc = document();
    DocumentMutator::new(&mut doc)
        .add_object(
            SurfaceId::new(1),
            object(
                2,
                [-290., 810., 30., 20.],
                ShapeKind::Ellipse,
                "cmyk(1,0,0,0)",
            ),
        )
        .unwrap();
    assert!(export_document_pdf(&doc, &Default::default()).is_err());
    let token = petunia_design_jobs::CancellationToken::new();
    token.cancel();
    assert!(petunia_design_io::export_document_pdf_cancellable(
        &document(),
        &Default::default(),
        &token
    )
    .is_err());
}
