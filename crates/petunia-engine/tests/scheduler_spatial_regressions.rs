use petunia_core::{
    AppearanceItem, AppearanceItemId, AppearanceKind, BlendMode, ColorSource, ColorValue, Contour,
    Document, FillRule, NodeKind, ObjectId, ParentRef, PathNode, Point, ProcessColor,
    ProcessColorValue, Rgba, SceneItem, SceneNode, StrokeCap, StrokeJoin, StrokeStyle, Transform2D,
    VectorPath,
};
use petunia_engine::jobs::{JobClass, JobError, Scheduler, SchedulerConfig};
use petunia_engine::spatial::{
    entry_for, hit_test, hit_test_with_view, paint_order, HitTestMode, HitTestRequest, RStarIndex,
};
use petunia_engine::transaction::DocumentRevision;
use std::sync::mpsc;
use std::time::Duration;

fn index(document: &Document) -> RStarIndex {
    RStarIndex::bulk_load(
        paint_order(document)
            .into_iter()
            .filter_map(|id| entry_for(document, id))
            .collect(),
    )
}

fn request(document: &Document, point: Point, mode: HitTestMode) -> HitTestRequest {
    HitTestRequest {
        page: document.scene.default_page(),
        point_document: point,
        tolerance_px: 0.0,
        mode,
    }
}

fn square(document: &mut Document, parent: ParentRef) -> ObjectId {
    let node = SceneNode::new_path("square", VectorPath::rect(0.0, 0.0, 10.0, 10.0), parent);
    let id = node.id;
    match parent {
        ParentRef::Page(page) => document.scene.insert_root(page, node).unwrap(),
        ParentRef::Object(id) => document.scene.insert_child(id, node, None).unwrap(),
    }
    id
}

#[test]
fn page_filter_and_group_world_transform_share_broad_and_narrow_phase() {
    let mut document = Document::new("spatial");
    let page = document.scene.default_page();
    let mut group = SceneNode::new_path("group", VectorPath::new(), ParentRef::Page(page));
    group.item = SceneItem::Group(Vec::new());
    group.transform = Transform2D::translation(100.0, 40.0).concat(Transform2D::scale(2.0, 3.0));
    let parent = group.id;
    document.scene.insert_root(page, group).unwrap();
    let child = square(&mut document, ParentRef::Object(parent));
    let other_page = document.scene.create_page();
    let other = square(&mut document, ParentRef::Page(other_page));
    document.scene.get_node_mut(other).unwrap().transform = Transform2D::translation(100.0, 40.0);
    let tree = index(&document);
    let hits = hit_test(
        &document,
        &tree,
        request(&document, Point::new(105.0, 45.0), HitTestMode::Fill),
    );
    assert_eq!(
        hits.iter().map(|hit| hit.object).collect::<Vec<_>>(),
        vec![child]
    );
    assert!(hit_test(
        &document,
        &tree,
        request(&document, Point::new(5.0, 5.0), HitTestMode::Fill)
    )
    .is_empty());
    for hidden in [true, false] {
        let parent = document.scene.get_node_mut(parent).unwrap();
        parent.visible = !hidden;
        parent.locked = !hidden;
        assert!(hit_test(
            &document,
            &tree,
            request(&document, Point::new(105.0, 45.0), HitTestMode::Fill)
        )
        .is_empty());
    }
}

#[test]
fn multiple_contours_obey_evenodd_and_nonzero_orientation() {
    let mut document = Document::new("holes");
    let page = document.scene.default_page();
    let mut path = VectorPath::rect(0.0, 0.0, 20.0, 20.0);
    path.contours
        .extend(VectorPath::rect(5.0, 5.0, 10.0, 10.0).contours);
    path.fill_rule = FillRule::EvenOdd;
    let node = SceneNode::new_path("donut", path, ParentRef::Page(page));
    let id = node.id;
    document.scene.insert_root(page, node).unwrap();
    let tree = index(&document);
    let probe = request(&document, Point::new(10.0, 10.0), HitTestMode::Fill);
    assert!(hit_test(&document, &tree, probe).is_empty());
    let SceneItem::Path(object) = &mut document.scene.get_node_mut(id).unwrap().item else {
        panic!()
    };
    object.path.fill_rule = FillRule::NonZero;
    assert_eq!(hit_test(&document, &tree, probe).len(), 1);
    let SceneItem::Path(object) = &mut document.scene.get_node_mut(id).unwrap().item else {
        panic!()
    };
    object.path.contours[1].nodes.reverse();
    assert!(hit_test(&document, &tree, probe).is_empty());
}

fn stroke(width: f64) -> AppearanceItem {
    AppearanceItem {
        id: AppearanceItemId::new_v4(),
        enabled: true,
        opacity: 1.0,
        blend_mode: BlendMode::Normal,
        kind: AppearanceKind::Stroke(
            StrokeStyle::new(
                width,
                StrokeCap::Butt,
                StrokeJoin::Miter,
                ColorSource::Value(ColorValue::Process(ProcessColor {
                    value: ProcessColorValue::Rgb(Rgba {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        alpha: 1.0,
                    }),
                    space: petunia_core::ColorSpaceRef::Builtin(
                        petunia_core::BuiltinColorSpace::Srgb,
                    ),
                })),
            )
            .unwrap(),
        ),
    }
}

#[test]
fn thick_stroke_and_screen_tolerance_survive_anisotropic_view() {
    let mut document = Document::new("stroke");
    let page = document.scene.default_page();
    let id = square(&mut document, ParentRef::Page(page));
    let SceneItem::Path(object) = &mut document.scene.get_node_mut(id).unwrap().item else {
        panic!()
    };
    object.appearance.items = vec![stroke(20.0)];
    let tree = index(&document);
    let mut probe = request(&document, Point::new(25.0, 5.0), HitTestMode::Stroke);
    probe.tolerance_px = 4.0;
    let view = Transform2D::scale(0.5, 5.0);
    assert_eq!(hit_test_with_view(&document, &tree, probe, view).len(), 1);
    probe.point_document = Point::new(29.0, 5.0);
    assert!(hit_test_with_view(&document, &tree, probe, view).is_empty());
    probe.point_document = Point::new(5.0, 21.0);
    assert!(hit_test_with_view(&document, &tree, probe, view).is_empty());
}

#[test]
fn curve_control_hull_is_indexed_and_flattened_for_fill() {
    let mut document = Document::new("curve");
    let page = document.scene.default_page();
    let mut contour = Contour::new(true);
    let mut left = PathNode::line(Point::new(0.0, 0.0), NodeKind::Cusp);
    left.handle_out = Some(Point::new(0.0, 30.0));
    left.outgoing = petunia_core::SegmentKind::Cubic;
    let mut right = PathNode::line(Point::new(10.0, 0.0), NodeKind::Cusp);
    right.handle_in = Some(Point::new(10.0, 30.0));
    contour.push_node(left);
    contour.push_node(right);
    let mut path = VectorPath::new();
    path.push_contour(contour);
    document
        .scene
        .insert_root(
            page,
            SceneNode::new_path("curve", path, ParentRef::Page(page)),
        )
        .unwrap();
    assert_eq!(
        hit_test(
            &document,
            &index(&document),
            request(&document, Point::new(5.0, 15.0), HitTestMode::Fill)
        )
        .len(),
        1
    );
}

#[test]
fn rtree_update_replaces_previous_envelope() {
    let mut document = Document::new("index");
    let page = document.scene.default_page();
    let id = square(&mut document, ParentRef::Page(page));
    let mut tree = index(&document);
    document.scene.get_node_mut(id).unwrap().transform = Transform2D::translation(100.0, 0.0);
    tree.insert(entry_for(&document, id).unwrap());
    assert_eq!(tree.len(), 1);
    assert!(hit_test(
        &document,
        &tree,
        request(&document, Point::new(5.0, 5.0), HitTestMode::Fill)
    )
    .is_empty());
}

#[test]
fn queue_budget_and_cancel_free_capacity_without_running_payload() {
    let mut scheduler = Scheduler::with_config(SchedulerConfig {
        worker_threads: 1,
        max_queue_depth: 1,
        interactive_budget_ms: 16,
    });
    let (started_tx, started_rx) = mpsc::channel();
    let blocker = scheduler.submit(
        JobClass::Background,
        DocumentRevision::GENESIS,
        (),
        mpsc::channel().0,
        move |(), token, _| {
            started_tx.send(()).unwrap();
            while !token.is_cancelled() {
                std::thread::yield_now();
            }
            Ok(())
        },
    );
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let queued = scheduler.submit(
        JobClass::Batch,
        DocumentRevision::GENESIS,
        (),
        mpsc::channel().0,
        |(), _, _| panic!("cancelled payload must not run"),
    );
    let full = scheduler.submit(
        JobClass::Interactive,
        DocumentRevision::GENESIS,
        (),
        mpsc::channel().0,
        |(), _, _| Ok(()),
    );
    assert_eq!(
        full.result(Duration::from_secs(2)).unwrap().result,
        Err(JobError::QueueFull { limit: 1 })
    );
    queued.cancel();
    assert_eq!(
        queued.result(Duration::from_secs(2)).unwrap().result,
        Err(JobError::Cancelled)
    );
    let replacement = scheduler.submit(
        JobClass::Interactive,
        DocumentRevision::GENESIS,
        (),
        mpsc::channel().0,
        |(), _, _| Ok(()),
    );
    scheduler.shutdown(Duration::from_secs(2)).unwrap();
    assert_eq!(
        blocker.result(Duration::from_secs(2)).unwrap().result,
        Err(JobError::Cancelled)
    );
    assert_eq!(
        replacement.result(Duration::from_secs(2)).unwrap().result,
        Err(JobError::Cancelled)
    );
    let closed = scheduler.submit(
        JobClass::Background,
        DocumentRevision::GENESIS,
        (),
        mpsc::channel().0,
        |(), _, _| Ok(()),
    );
    assert_eq!(
        closed.result(Duration::from_secs(2)).unwrap().result,
        Err(JobError::SchedulerShutdown)
    );
}

#[test]
fn reserved_interactive_lane_is_available_during_background_work() {
    let mut scheduler = Scheduler::new(2);
    let (started_tx, started_rx) = mpsc::channel();
    let blocker = scheduler.submit(
        JobClass::Background,
        DocumentRevision::GENESIS,
        (),
        mpsc::channel().0,
        move |(), token, _| {
            started_tx.send(()).unwrap();
            while !token.is_cancelled() {
                std::thread::yield_now();
            }
            Ok(())
        },
    );
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let interactive = scheduler.submit(
        JobClass::Interactive,
        DocumentRevision::GENESIS,
        42u32,
        mpsc::channel().0,
        |value, _, _| Ok(value),
    );
    assert_eq!(
        interactive.result(Duration::from_secs(2)).unwrap().result,
        Ok(42)
    );
    scheduler.shutdown(Duration::from_secs(2)).unwrap();
    assert_eq!(
        blocker.result(Duration::from_secs(2)).unwrap().result,
        Err(JobError::Cancelled)
    );
}

#[test]
fn panic_returns_algorithm_error_without_losing_worker() {
    let mut scheduler = Scheduler::new(1);
    let panic = scheduler.submit(
        JobClass::Background,
        DocumentRevision::GENESIS,
        (),
        mpsc::channel().0,
        |(), _, _| panic!("algorithm panic"),
    );
    assert!(matches!(
        panic.result(Duration::from_secs(2)).unwrap().result,
        Err(JobError::AlgorithmFailure(_))
    ));
    let next = scheduler.submit(
        JobClass::Background,
        DocumentRevision::GENESIS,
        3u32,
        mpsc::channel().0,
        |n, _, _| Ok(n),
    );
    assert_eq!(next.result(Duration::from_secs(2)).unwrap().result, Ok(3));
    scheduler.shutdown(Duration::from_secs(2)).unwrap();
}

#[test]
fn shutdown_deadline_does_not_hang_on_noncooperative_work() {
    let mut scheduler = Scheduler::new(1);
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let job = scheduler.submit(
        JobClass::Background,
        DocumentRevision::GENESIS,
        (),
        mpsc::channel().0,
        move |(), _, _| {
            started_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            Ok(())
        },
    );
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(
        scheduler.shutdown(Duration::from_millis(5)),
        Err(JobError::ShutdownTimeout)
    );
    release_tx.send(()).unwrap();
    scheduler.shutdown(Duration::from_secs(2)).unwrap();
    assert_eq!(
        job.result(Duration::from_secs(2)).unwrap().result,
        Err(JobError::Cancelled)
    );
}

fn snap_request() -> petunia_engine::spatial::SnapRequest {
    use petunia_engine::spatial::*;
    SnapRequest {
        page: petunia_core::PageId::new_v4(),
        moving: MovingGeometry {
            excluded: Vec::new(),
            anchors: vec![Point::new(0.0, 0.0)],
        },
        proposed: TransformDelta::default(),
        constraint: SnapConstraint::Free,
        settings: SnapSettings::default(),
        view_scale: 1.0,
        previous: None,
    }
}

#[test]
fn snap_latch_release_band_changes_the_returned_correction() {
    use petunia_engine::spatial::*;
    let mut request = snap_request();
    let target = SnapTarget::Guide(petunia_core::GuideId::new_v4());
    request.previous = Some(SnapLatch {
        target,
        priority: SnapPriority::Guide,
    });
    let candidate = SnapCandidate {
        target,
        constraint: SnapConstraint::Free,
        correction: TransformDelta { dx: 9.0, dy: 0.0 },
        distance_px: 9.0,
        priority: SnapPriority::Guide,
        source: SnapSourceId {
            provider: SnapProvider::Guide,
            key: 0,
        },
    };
    let result = solve(&request, &[candidate]);
    assert_eq!(result.corrected.dx, 9.0);
    assert_eq!(result.matches.len(), 1);
    request.settings.providers.clear();
    assert_eq!(solve(&request, &[candidate]).latch, None);
}

#[test]
fn snap_two_axis_target_remains_atomic_and_self_targets_are_excluded() {
    use petunia_engine::spatial::*;
    let mut request = snap_request();
    let grid = SnapCandidate {
        target: SnapTarget::GridPoint(Point::new(3.0, 4.0)),
        constraint: SnapConstraint::Free,
        correction: TransformDelta { dx: 3.0, dy: 4.0 },
        distance_px: 5.0,
        priority: SnapPriority::Grid,
        source: SnapSourceId {
            provider: SnapProvider::Grid,
            key: 0,
        },
    };
    request.constraint = SnapConstraint::Horizontal;
    assert!(solve(&request, &[grid]).matches.is_empty());
    request.constraint = SnapConstraint::Free;
    assert_eq!(solve(&request, &[grid]).corrected, grid.correction);
    let id = ObjectId::new_v4();
    request.moving.excluded.push(id);
    let own = SnapCandidate {
        target: SnapTarget::ObjectBounds(id),
        source: SnapSourceId {
            provider: SnapProvider::ObjectBounds,
            key: 0,
        },
        ..grid
    };
    assert!(solve(&request, &[own]).matches.is_empty());
}

#[test]
fn legacy_guide_snapping_is_nearest_and_order_independent() {
    use petunia_engine::snapping::*;
    let left = SnapGuide {
        position: 2.0,
        orientation: SnapOrientation::Vertical,
    };
    let right = SnapGuide {
        position: 4.0,
        orientation: SnapOrientation::Vertical,
    };
    let config = SnapConfig {
        threshold: 5.0,
        grid_spacing: Some(0.0),
    };
    let point = Point::new(0.0, 1.0);
    assert_eq!(
        snap_point(point, &[left, right], &config),
        snap_point(point, &[right, left], &config)
    );
    assert_eq!(
        snap_point(point, &[left, right], &config).snapped_point.x,
        2.0
    );
}

#[test]
fn snapshot_image_hit_uses_transformed_cropped_source_rectangle() {
    use petunia_engine::compile::{compile_document_with_providers, CompileProviders};
    use petunia_engine::spatial::{entries_for_snapshot, hit_test_with_snapshot};
    use petunia_render_model::{RenderColor, RenderQuality};
    let mut document = Document::new("image");
    let page = document.scene.default_page();
    let resource = petunia_core::ResourceId::new_v4();
    let mut node = SceneNode::new_path("image", VectorPath::new(), ParentRef::Page(page));
    node.item = SceneItem::Image(petunia_core::ImageObject {
        resource,
        source_rect: Some(
            petunia_core::ImageSourceRect::new(
                petunia_core::NormalizedPoint::new(0.25, 0.25).unwrap(),
                petunia_core::NormalizedPoint::new(0.75, 0.75).unwrap(),
            )
            .unwrap(),
        ),
        sampling: petunia_core::ImageSamplingPolicy::Nearest,
    });
    node.transform = Transform2D {
        a: 1.0,
        b: 0.0,
        c: 1.0,
        d: 1.0,
        tx: 100.0,
        ty: 0.0,
    };
    let id = node.id;
    document.scene.insert_root(page, node).unwrap();
    let mut providers = CompileProviders::default();
    providers.images.insert(
        resource,
        std::sync::Arc::new(
            petunia_render_model::image::ResolvedImage::new(
                20,
                20,
                vec![
                    RenderColor {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0
                    };
                    400
                ],
            )
            .unwrap(),
        ),
    );
    let (snapshot, warnings) = compile_document_with_providers(
        &document,
        DocumentRevision::GENESIS,
        RenderQuality::Authoring,
        &providers,
    );
    assert!(warnings.is_empty());
    let tree = RStarIndex::bulk_load(entries_for_snapshot(&snapshot));
    let hit = hit_test_with_snapshot(
        &document,
        &snapshot,
        &tree,
        request(&document, Point::new(120.0, 10.0), HitTestMode::Any),
        Transform2D::IDENTITY,
    );
    assert_eq!(
        hit.iter().map(|hit| hit.object).collect::<Vec<_>>(),
        vec![id]
    );
    assert!(hit_test_with_snapshot(
        &document,
        &snapshot,
        &tree,
        request(&document, Point::new(102.0, 1.0), HitTestMode::Any),
        Transform2D::IDENTITY
    )
    .is_empty());
    assert!(hit_test_with_snapshot(
        &document,
        &snapshot,
        &tree,
        request(&document, Point::new(102.0, 18.0), HitTestMode::Any),
        Transform2D::IDENTITY
    )
    .is_empty());
}
