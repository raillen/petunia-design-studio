use petunia_core::{Contour, NodeKind, PathNode, Point, SegmentKind, VectorPath};
use petunia_engine::{
    io::{import_raster_image, path_to_svg_data, IoLimits},
    ptnd::{save_atomic, CooperativeFileLock},
};

#[test]
fn svg_keeps_segment_kind_missing_handles_and_curved_closure() {
    let mut first = PathNode::new(Point::new(0.0, 0.0), NodeKind::Cusp);
    first.handle_out = Some(Point::new(0.0, 10.0));
    let mut last = PathNode::new(Point::new(10.0, 0.0), NodeKind::Cusp);
    last.handle_out = Some(Point::new(10.0, -10.0));
    let mut contour = Contour::new(true);
    contour.nodes = vec![first, last];
    let mut path = VectorPath::new();
    path.push_contour(contour);
    let data = path_to_svg_data(&path);
    assert!(data.contains("C 0 10 10 0 10 0"), "{data}");
    assert!(data.contains("C 10 -10 0 0 0 0 Z"), "{data}");
    path.contours[0].nodes[0].outgoing = SegmentKind::Line;
    assert!(path_to_svg_data(&path).contains("L 10 0"));
}

#[test]
fn decoded_image_bytes_are_bounded_before_rgba_allocation() {
    use image::{codecs::png::PngEncoder, ExtendedColorType, ImageEncoder};
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes)
        .write_image(&[255; 16], 2, 2, ExtendedColorType::Rgba8)
        .unwrap();
    let limits = IoLimits {
        max_embedded_bytes: 8,
        ..IoLimits::default()
    };
    assert!(matches!(
        import_raster_image(&bytes, &limits),
        Err(petunia_engine::EngineError::Limit(_))
    ));
    let limits = IoLimits {
        max_dimension_px: 1,
        ..IoLimits::default()
    };
    assert!(import_raster_image(&bytes, &limits).is_err());
}

#[test]
fn concurrent_lock_acquisition_has_exactly_one_owner() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Barrier,
    };
    let directory =
        std::env::temp_dir().join(format!("petunia-lock-regression-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&directory).unwrap();
    let target = Arc::new(directory.join("document.ptnd"));
    let start = Arc::new(Barrier::new(8));
    let held = Arc::new(Barrier::new(8));
    let successes = Arc::new(AtomicUsize::new(0));
    let mut workers = Vec::new();
    for _ in 0..8 {
        let (target, start, held, successes) = (
            target.clone(),
            start.clone(),
            held.clone(),
            successes.clone(),
        );
        workers.push(std::thread::spawn(move || {
            start.wait();
            let lock = CooperativeFileLock::acquire(&target).ok();
            if lock.is_some() {
                successes.fetch_add(1, Ordering::SeqCst);
            }
            held.wait();
            drop(lock);
        }));
    }
    for worker in workers {
        worker.join().unwrap();
    }
    assert_eq!(successes.load(Ordering::SeqCst), 1);
    let mut owner = CooperativeFileLock::acquire(&target).unwrap();
    owner.release();
    assert!(CooperativeFileLock::acquire(&target).is_ok());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn failed_atomic_replace_leaves_no_temporary_file() {
    let directory = std::env::temp_dir().join(format!(
        "petunia-atomic-regression-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(directory.join("target")).unwrap();
    assert!(save_atomic(&directory.join("target"), b"complete payload").is_err());
    assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
    std::fs::remove_dir_all(directory).unwrap();
}
