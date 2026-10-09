//! Benchmarks for geometry hot paths.
//!
//! Every benchmark records fixture, input size and metric per the
//! canonical methodology: small/normal/stress scales, deterministic
//! fixtures, and the operation named explicitly.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use petunia_core::{Point, Rect, Tolerance, Transform2D, VectorPath};
use petunia_engine::geometry::{
    boolean_paths, boolean_rings, offset_ring, simplify_open, BooleanOp, CubicBez,
};

fn square(x: f64, y: f64, size: f64) -> Vec<Point> {
    vec![
        Point::new(x, y),
        Point::new(x + size, y),
        Point::new(x + size, y + size),
        Point::new(x, y + size),
    ]
}

fn cubic_ring(segments: usize, tolerance: Tolerance) -> Vec<Point> {
    let mut ring = Vec::new();
    let curve = CubicBez {
        p0: Point::new(0.0, 0.0),
        p1: Point::new(40.0, 80.0),
        p2: Point::new(160.0, 80.0),
        p3: Point::new(200.0, 0.0),
    };
    let per_segment = (tolerance.0 * segments as f64).max(1.0) as usize;
    for segment in 0..segments {
        let start = segment as f64 / segments as f64;
        let end = (segment + 1) as f64 / segments as f64;
        for step in 0..per_segment {
            let t = start + (end - start) * step as f64 / per_segment as f64;
            ring.push(curve.evaluate(t));
        }
    }
    if ring.len() > 1 {
        ring.pop();
    }
    ring
}

fn bench_bezier_flatten(c: &mut Criterion) {
    let mut group = c.benchmark_group("bezier/flatten");
    let curve = CubicBez {
        p0: Point::new(0.0, 0.0),
        p1: Point::new(40.0, 80.0),
        p2: Point::new(160.0, 80.0),
        p3: Point::new(200.0, 0.0),
    };
    for tolerance in [0.5f64, 0.15, 0.05] {
        let band = Tolerance::new(tolerance).expect("valid");
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("tol={tolerance}")),
            &curve,
            |b, curve| b.iter(|| curve.flatten(band).len()),
        );
    }
    group.finish();
}

fn bench_offset(c: &mut Criterion) {
    let mut group = c.benchmark_group("geometry/offset");
    for segments in [4usize, 12, 32] {
        let ring = if segments == 4 {
            square(0.0, 0.0, 100.0)
        } else {
            cubic_ring(segments, Tolerance::new(0.5).expect("valid"))
        };
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("verts={}", ring.len())),
            &ring,
            |b, ring| b.iter(|| offset_ring(ring, 2.0).expect("offsets").len()),
        );
    }
    group.finish();
}

fn bench_simplify(c: &mut Criterion) {
    let mut group = c.benchmark_group("geometry/simplify");
    for points in [64usize, 512, 4_096] {
        let polyline: Vec<Point> = (0..points)
            .map(|index| Point::new(index as f64, if index % 2 == 0 { 0.0 } else { 0.01 }))
            .collect();
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("points={points}")),
            &polyline,
            |b, polyline| b.iter(|| simplify_open(polyline, 0.5).len()),
        );
    }
    group.finish();
}

fn bench_boolean(c: &mut Criterion) {
    let mut group = c.benchmark_group("geometry/boolean");
    let subject = vec![square(0.0, 0.0, 100.0)];
    let clip = vec![square(50.0, 0.0, 100.0)];
    group.bench_function("rings/union-2-squares", |b| {
        b.iter(|| {
            boolean_rings(
                &subject,
                &clip,
                BooleanOp::Union,
                petunia_core::FillRule::NonZero,
            )
            .len()
        })
    });

    let mut a = VectorPath::new();
    let mut contour = petunia_core::Contour::new(true);
    for point in square(0.0, 0.0, 100.0) {
        contour.push_node(petunia_core::PathNode::new(
            point,
            petunia_core::NodeKind::Cusp,
        ));
    }
    a.push_contour(contour);
    let mut b_path = VectorPath::new();
    let mut contour = petunia_core::Contour::new(true);
    for point in square(50.0, 50.0, 100.0) {
        contour.push_node(petunia_core::PathNode::new(
            point,
            petunia_core::NodeKind::Cusp,
        ));
    }
    b_path.push_contour(contour);
    group.bench_function("paths/union-2-squares", |b| {
        b.iter(|| {
            boolean_paths(
                &a,
                &b_path,
                BooleanOp::Union,
                petunia_core::FillRule::NonZero,
                Tolerance::new(0.15).expect("valid"),
            )
            .contours
            .len()
        })
    });
    group.finish();
}

fn bench_transform(c: &mut Criterion) {
    let mut group = c.benchmark_group("math/transform");
    let transform = Transform2D {
        a: 2.0,
        c: 1.0,
        b: 0.5,
        d: 1.5,
        tx: 10.0,
        ty: -4.0,
    };
    let inverse = transform.inverse().expect("invertible");
    let point = Point::new(37.0, -12.0);
    group.bench_function("point/apply", |b| {
        b.iter(|| transform.transform_point(point))
    });
    group.bench_function("matrix/inverse", |b| {
        b.iter(|| inverse.transform_point(transform.transform_point(point)))
    });
    let rect = Rect::new(0.0, 0.0, 1_000.0, 800.0);
    group.bench_function("rect/contains", |b| b.iter(|| rect.contains_point(point)));
    group.finish();
}

fn bench_refit(c: &mut Criterion) {
    let mut group = c.benchmark_group("geometry/refit");
    use petunia_core::{Contour, NodeKind, PathNode};
    for samples in [64usize, 256] {
        let points: Vec<Point> = (0..=samples)
            .map(|index| {
                let t = index as f64 / samples as f64;
                Point::new(t * 200.0, (t * std::f64::consts::FRAC_PI_2).sin() * 80.0)
            })
            .collect();
        let mut contour = Contour::new(false);
        for point in &points {
            contour.push_node(PathNode::new(*point, NodeKind::Cusp));
        }
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("samples={samples}")),
            &contour,
            |b, contour| {
                b.iter(|| {
                    petunia_engine::geometry::fit::refit_contour(contour, 0.25)
                        .map(|nodes| nodes.len())
                        .unwrap_or(0)
                })
            },
        );
    }
    group.finish();
}

criterion_group!(
    geometry_benches,
    bench_bezier_flatten,
    bench_offset,
    bench_simplify,
    bench_boolean,
    bench_refit,
    bench_transform
);
criterion_main!(geometry_benches);
