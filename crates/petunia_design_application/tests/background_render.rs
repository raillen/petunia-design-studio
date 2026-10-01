use petunia_design_application::background_render::schedule_surface_render;
use petunia_design_document::{DocumentObject, ShapeKind, Surface};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_jobs::{JobExecutor, JobFailure, JobManager};
use petunia_design_render::{RenderRequest, RenderSurface};
use std::time::{Duration, Instant};

#[test]
fn application_renders_owned_snapshot_on_worker_and_suppresses_stale_result() {
    let mut object = DocumentObject::new(ObjectId::new(2), "Art");
    object.bounds = Some([0.0, 0.0, 10.0, 10.0]);
    object.fill = Some("#ff0000".into());
    object.shape = Some(ShapeKind::Rectangle {
        corner_radii: [0.0; 4],
    });
    let mut surface = Surface::with_objects(SurfaceId::new(1), "Page", vec![object]);
    surface.dimensions = [10.0, 10.0];
    let scene = RenderSurface::extract(&surface).unwrap();
    let request = RenderRequest::for_surface(&scene, 72.0).unwrap();
    let executor = JobExecutor::new(1, 4, JobManager::new()).unwrap();
    let handle = schedule_surface_render(&executor, scene.clone(), 5, request).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let pixels = loop {
        if let Some(pixels) = handle.try_result(5).unwrap() {
            break pixels;
        }
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    };
    assert_eq!(pixels.get_pixel(5, 5).unwrap(), [255, 0, 0, 255]);
    let stale = schedule_surface_render(&executor, scene, 5, request).unwrap();
    loop {
        match stale.try_result(6) {
            Ok(None) => {
                assert!(Instant::now() < deadline);
                std::thread::yield_now();
            }
            Err(JobFailure::StaleRevision {
                source_revision: 5,
                current_revision: 6,
            }) => break,
            _ => panic!("obsolete snapshot was published"),
        }
    }
    executor.shutdown_and_join();
}

#[test]
fn export_payload_preserves_dpi_and_rejects_invalid_density() {
    use petunia_design_application::export_service::ExportRequest;
    let request = ExportRequest::from_payload(
        &serde_json::json!({"path": "/tmp/art.png", "dpi": 300.0}),
        None,
    )
    .unwrap();
    assert_eq!(request.dpi, 300.0);
    for value in [
        serde_json::json!(0),
        serde_json::json!(-72),
        serde_json::json!("300"),
    ] {
        assert!(ExportRequest::from_payload(
            &serde_json::json!({"path": "/tmp/art.png", "dpi": value}),
            None
        )
        .is_err());
    }
}
