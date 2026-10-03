use petunia_design_application::{
    histogram::{analyze, Histogram, HistogramController, HistogramRequest},
    preview::PreviewSource,
};
use petunia_design_document::{DocumentObject, ShapeKind, Surface};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_jobs::CancellationToken;
use petunia_design_render::PixelBufferRgba8;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
fn object(id: u64, color: &str) -> DocumentObject {
    let mut o = DocumentObject::new(ObjectId::new(id), "Rect");
    o.bounds = Some([0., 0., 8., 8.]);
    o.shape = Some(ShapeKind::Rectangle {
        corner_radii: [0.; 4],
    });
    o.fill = Some(color.into());
    o
}
fn source(color: &str) -> Arc<PreviewSource> {
    let mut s = Surface::with_objects(SurfaceId::new(1), "Page", vec![object(2, color)]);
    s.dimensions = [8., 8.];
    PreviewSource::capture(&s, 0)
}
fn finish(controller: &mut HistogramController) -> Arc<Histogram> {
    let end = Instant::now() + Duration::from_secs(10);
    loop {
        controller.poll();
        assert!(controller.failure().is_none(), "{:?}", controller.failure());
        if let Some(result) = controller.result() {
            return result;
        }
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(1));
    }
}
#[test]
fn empty_pixels_do_not_invent_statistics_and_alpha_weights_visible_channels() {
    assert_eq!(
        Histogram::from_pixels(&PixelBufferRgba8::with_fill(2, 2, [100, 100, 100, 0])).summary(0),
        ([0.; 32], 0, 0, 0, 0)
    );
    let mut pixels = PixelBufferRgba8::with_fill(2, 1, [0, 0, 0, 255]);
    pixels.data[4..].copy_from_slice(&[255, 255, 255, 85]);
    let (_, mean, shadows, midtones, highlights) = Histogram::from_pixels(&pixels).summary(1);
    assert_eq!((mean, shadows, midtones, highlights), (64, 75, 0, 25));
}
#[test]
fn selected_histogram_measures_composition_and_occlusion_in_object_bounds() {
    let mut s = Surface::with_objects(
        SurfaceId::new(1),
        "Page",
        vec![object(2, "#ff0000"), object(3, "#0000ff")],
    );
    s.dimensions = [8., 8.];
    let result = analyze(
        &HistogramRequest {
            source: PreviewSource::capture(&s, 0),
            object: Some(ObjectId::new(2)),
        },
        &CancellationToken::new(),
    )
    .unwrap();
    assert_eq!(result.summary(1).1, 0);
    assert_eq!(result.summary(3).1, 255);
    assert_eq!(result.sampled_pixels, 64);
}
#[test]
fn same_revision_tabs_and_rapid_requests_only_publish_the_latest_source() {
    let mut controller = HistogramController::new().unwrap();
    controller.request(Some(HistogramRequest {
        source: source("#ff0000"),
        object: None,
    }));
    assert_eq!(finish(&mut controller).summary(1).1, 255);
    for _ in 0..40 {
        controller.request(Some(HistogramRequest {
            source: source("#ff0000"),
            object: None,
        }));
    }
    controller.request(Some(HistogramRequest {
        source: source("#0000ff"),
        object: None,
    }));
    assert!(controller.result().is_none());
    let result = finish(&mut controller);
    assert_eq!(result.summary(1).1, 0);
    assert_eq!(result.summary(3).1, 255);
    controller.request(None);
    assert!(!controller.is_pending());
    assert!(controller.result().is_none());
}
#[test]
fn cancellation_does_not_publish_a_histogram() {
    let token = CancellationToken::new();
    token.cancel();
    assert!(analyze(
        &HistogramRequest {
            source: source("#ff0000"),
            object: None
        },
        &token
    )
    .is_err());
}
