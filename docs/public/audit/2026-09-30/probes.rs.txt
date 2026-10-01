use petunia_design_application::{Command, CommandRequest, DocumentSession, History, Transaction};
use petunia_design_color::{
    ColorManagementProvider, ColorValue, DefaultColorManagementProvider, ProofContext, Srgb,
};
use petunia_design_document::{Document, DocumentMutator, DocumentObject, ShapeKind};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_geometry::{
    boolean_op, simplify_rdp, BooleanInput, BooleanOp, GPath, GPoint, PathVerb,
};
use petunia_design_io::{export_document_pdf, PdfExportOptions};
use petunia_design_raster::{AlphaMode, BlendMode, BrushDab, PixelFormat, TileMap};
use petunia_design_shell::PetuniaShell;

fn sample(shape: ShapeKind) -> Document {
    let mut doc = Document::new();
    let mut object = DocumentObject::new(ObjectId::new(2), "probe");
    object.bounds = Some([10., 10., 20., 20.]);
    object.shape = Some(shape);
    object.fill = Some("#ff0000".into());
    let mut mutator = DocumentMutator::new(&mut doc);
    mutator.add_surface(SurfaceId::new(1), "probe").unwrap();
    mutator.add_object(SurfaceId::new(1), object).unwrap();
    doc
}

fn main() {
    let mut reproduced = 0;
    let mut map = TileMap::new(PixelFormat::Rgba8, AlphaMode::Straight);
    map.set_pixel(0, 0, [1., 0., 0., 1.]);
    BrushDab::eraser_dab(0., 0.).stamp_onto(&mut map);
    assert_eq!(map.get_pixel(0, 0), [1., 0., 0., 1.]);
    println!("PROBE-01 reproduced: eraser leaves opaque red pixel unchanged");
    reproduced += 1;

    let blend = BlendMode::Multiply.blend([1., 0., 0., 1.], [0., 0., 0., 0.]);
    assert_eq!(blend, [0., 0., 0., 1.]);
    let reference = petunia_design_render::BlendMode::Multiply
        .composite_pixel_straight([0., 0., 0., 0.], [1., 0., 0., 1.]);
    assert_eq!(reference, [1., 0., 0., 1.]);
    println!("PROBE-02 reproduced: raster Multiply on transparent black becomes black; render reference remains red");
    reproduced += 1;

    let triangle = vec![
        GPoint::new(0., 0.),
        GPoint::new(20., 0.),
        GPoint::new(0., 20.),
    ];
    let difference = boolean_op(
        &BooleanInput::single(triangle.clone()),
        &BooleanInput::single(Vec::new()),
        BooleanOp::Difference,
    );
    // Empty contours represented literally as no contours rather than one empty contour.
    let difference_empty = boolean_op(
        &BooleanInput {
            contours: vec![triangle.clone()],
        },
        &BooleanInput { contours: vec![] },
        BooleanOp::Difference,
    );
    assert!(difference_empty.is_empty());
    println!("PROBE-03 reproduced: A minus empty returns empty instead of A (also empty contour result length={})",difference.len());
    reproduced += 1;

    let mut session = DocumentSession::with_document(
        "probe",
        sample(ShapeKind::Rectangle {
            corner_radii: [0.; 4],
        }),
    );
    let old = session.cached_polygons(ObjectId::new(2), 0.1).unwrap();
    session
        .execute_command(CommandRequest::new(Command::SetBounds {
            id: ObjectId::new(2),
            bounds: Some([100., 10., 20., 20.]),
            rotation: 0.,
        }))
        .unwrap();
    let new_bounds = session.cached_bounds(ObjectId::new(2)).unwrap();
    let stale = session.cached_polygons(ObjectId::new(2), 0.1).unwrap();
    assert_eq!(stale, old);
    assert_eq!(new_bounds[0], 100.);
    println!("PROBE-04 reproduced: cached bounds move to x=100 but cached flattened polygons retain x={}",stale[0][0].x);
    reproduced += 1;

    let mut doc = sample(ShapeKind::Ellipse);
    let mut tx = Transaction::begin(&doc, "rename");
    tx.update(&CommandRequest::new(Command::RenameObject {
        id: ObjectId::new(2),
        name: "staged".into(),
    }))
    .unwrap();
    DocumentMutator::new(&mut doc)
        .add_object(
            SurfaceId::new(1),
            DocumentObject::new(ObjectId::new(3), "concurrent"),
        )
        .unwrap();
    tx.commit(&mut doc, &mut History::new(100));
    assert!(doc.find_object(ObjectId::new(3)).is_none());
    println!(
        "PROBE-05 reproduced: Transaction commit replaces document and drops intervening mutation"
    );
    reproduced += 1;

    let points = vec![
        GPoint::new(0., 0.),
        GPoint::new(20., 0.),
        GPoint::new(0., 0.),
    ];
    let simplified = simplify_rdp(&points, 0.1);
    assert_eq!(simplified.len(), 2);
    println!("PROBE-06 reproduced: RDP loses excursion when first and last points coincide");
    reproduced += 1;

    let provider = DefaultColorManagementProvider;
    let color = ColorValue::Rgb(Srgb::try_new(0.9, 0.4, 0.1).unwrap());
    let swop = provider.soft_proof(
        &color,
        &ProofContext::for_profile("US Web Coated (SWOP) v2"),
    );
    let fogra = provider.soft_proof(&color, &ProofContext::for_profile("FOGRA39"));
    assert_eq!(swop, fogra);
    println!("PROBE-07 reproduced: SWOP and FOGRA39 proof produce identical heuristic outputs");
    reproduced += 1;

    let mut path = GPath::new();
    for verb in [
        PathVerb::MoveTo(triangle[0]),
        PathVerb::LineTo(triangle[1]),
        PathVerb::LineTo(triangle[2]),
        PathVerb::Close,
    ] {
        path.push(verb).unwrap();
    }
    let path_doc = sample(ShapeKind::Path(path));
    let mut shell = PetuniaShell::new(800., 600.);
    shell.bridge.open_document("path", path_doc).unwrap();
    let snapshot = shell.canvas_snapshot();
    assert_eq!(snapshot.objects.len(), 1);
    assert!(snapshot.objects[0].outline.is_none());
    println!(
        "PROBE-08 reproduced: explicit Path reaches canvas projection with no drawable outline"
    );
    reproduced += 1;

    let mut cyclic = sample(ShapeKind::Ellipse);
    cyclic.find_object_mut(ObjectId::new(2)).unwrap().parent = Some(ObjectId::new(2));
    let decoded = Document::from_json(&cyclic.to_json().unwrap()).unwrap();
    assert!(decoded.world_transform_checked(ObjectId::new(2)).is_err());
    println!("PROBE-09 reproduced: document decoder accepts self-parent cycle (checked world transform rejects later)");
    reproduced += 1;

    let doc = sample(ShapeKind::Ellipse);
    let (pdf, report) = export_document_pdf(&doc, &PdfExportOptions::default()).unwrap();
    std::fs::write(
        std::env::var("PETUNIA_AUDIT_PDF").unwrap_or_else(|_| "audit-probe.pdf".into()),
        pdf,
    )
    .unwrap();
    assert!(report.passed);
    println!("PROBE-10 exported 800x600 surface using default PDF settings; inspect audit-probe.pdf page dimensions");

    let old_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let unicode =
        std::panic::catch_unwind(|| petunia_design_document::resolve_color_to_rgb("#€abc"));
    std::panic::set_hook(old_hook);
    assert!(unicode.is_err());
    println!("PROBE-11 reproduced: malformed Unicode color token panics rather than returning a diagnostic");
    reproduced += 1;

    let fresh = DocumentSession::with_document("fresh", sample(ShapeKind::Ellipse));
    assert!(fresh
        .spatial_candidates_point(GPoint::new(20., 20.), 1.)
        .is_empty());
    println!(
        "PROBE-12 reproduced: restored revision-zero document returns no R-tree hit candidates"
    );
    reproduced += 1;

    let shape_doc = sample(ShapeKind::Path(GPath::from_polygons(&[triangle])));
    let buffer = petunia_design_render::SoftwarePixelCompositor::render_surface_rgba8(
        &shape_doc.surfaces()[0],
        40,
        40,
        [0, 0, 0, 0],
    );
    assert_eq!(buffer.get_pixel(29, 29).unwrap(), [255, 0, 0, 255]);
    println!("PROBE-13 reproduced: software export paints bounding rectangle outside triangle");
    reproduced += 1;

    let a = petunia_design_render::IsolationGroup::default()
        .with_clip(petunia_design_geometry::GRect::new(0., 0., 10., 10.));
    let b = petunia_design_render::IsolationGroup::default()
        .with_clip(petunia_design_geometry::GRect::new(20., 20., 30., 30.));
    let context = petunia_design_render::EffectiveContext::default()
        .child_of(&a)
        .child_of(&b);
    assert!(context.effective_clip.is_none());
    println!("PROBE-14 reproduced: disjoint clipping intersection becomes None (also used for no clipping)");
    reproduced += 1;

    let mut excluded = sample(ShapeKind::Ellipse);
    excluded
        .surface_mut(SurfaceId::new(1))
        .unwrap()
        .export_enabled = false;
    let (_, report) = export_document_pdf(&excluded, &PdfExportOptions::default()).unwrap();
    assert_eq!(report.surfaces, 1);
    println!("PROBE-15 reproduced: PDF exports surface marked export_enabled=false");
    reproduced += 1;

    let mut half = sample(ShapeKind::Rectangle {
        corner_radii: [0.; 4],
    });
    half.find_object_mut(ObjectId::new(2)).unwrap().opacity = 0.5;
    let buffer = petunia_design_render::SoftwarePixelCompositor::render_surface_rgba8(
        &half.surfaces()[0],
        40,
        40,
        [0, 0, 0, 0],
    );
    let alpha = buffer.get_pixel(15, 15).unwrap()[3];
    assert!((i32::from(alpha) - 64).abs() <= 1);
    println!(
        "PROBE-16 reproduced: 50% object opacity produces alpha={} (about 25%)",
        alpha
    );
    reproduced += 1;

    let mut compound = GPath::rect(
        petunia_design_geometry::GRect::new(0., 0., 20., 20.),
        0.,
        0.,
    );
    compound.verbs.extend(
        GPath::rect(
            petunia_design_geometry::GRect::new(100., 0., 120., 20.),
            0.,
            0.,
        )
        .verbs,
    );
    let expanded = petunia_design_geometry::offset_path(
        &compound,
        2.,
        petunia_design_geometry::OffsetJoin::Round,
        petunia_design_geometry::OffsetCap::Round,
    )
    .unwrap();
    assert_eq!(expanded.subpath_count(), 1);
    println!("PROBE-17 reproduced: positive offset drops one of two disconnected components");
    reproduced += 1;

    let original = GPath::rect(
        petunia_design_geometry::GRect::new(0., 0., 20., 20.),
        0.,
        0.,
    );
    let pieces = petunia_design_geometry::cut_path_by_line(
        &original,
        GPoint::new(10., 100.),
        GPoint::new(10., 110.),
        0.1,
    );
    assert_eq!(pieces.len(), 1);
    println!("CHECK-18 passed: distant finite knife segment does not cut the rectangle");

    assert!(petunia_design_io::parse_path_d("M0,0L10,20Z").is_err());
    println!("PROBE-19 reproduced: valid compact SVG path rejected by whitespace-only parser");
    reproduced += 1;

    println!(
        "Reproduced {} behavioral defects/contract gaps; PDF dimensions recorded separately.",
        reproduced
    );

    for count in [500usize, 2000] {
        let mut doc = Document::new();
        let mut m = DocumentMutator::new(&mut doc);
        m.add_surface(SurfaceId::new(1), "benchmark").unwrap();
        for i in 0..count {
            let mut object = DocumentObject::new(ObjectId::new(i as u64 + 2), "rectangle");
            object.bounds = Some([10., 10., 20., 20.]);
            object.shape = Some(ShapeKind::Rectangle {
                corner_radii: [0.; 4],
            });
            m.add_object(SurfaceId::new(1), object).unwrap();
        }
        let mut shell = PetuniaShell::new(800., 600.);
        shell.bridge.open_document("benchmark", doc).unwrap();
        let t = std::time::Instant::now();
        let first = shell.canvas_snapshot();
        let miss = t.elapsed();
        let t = std::time::Instant::now();
        for _ in 0..50 {
            std::hint::black_box(shell.canvas_snapshot());
        }
        println!(
            "BENCH build={} rectangles={} visible={} snapshot_miss_ms={:.3} warm_mean_us={:.3}",
            if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
            count,
            first.objects.len(),
            miss.as_secs_f64() * 1000.,
            t.elapsed().as_secs_f64() * 1e6 / 50.
        );
    }
}
