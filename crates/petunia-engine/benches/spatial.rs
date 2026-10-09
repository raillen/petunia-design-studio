//! Benchmarks for the spatial hot path: R-tree queries against the
//! linear reference, at small/normal/stress scales.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use petunia_core::{ObjectId, Point, Rect};
use petunia_engine::spatial::{LinearIndex, RStarIndex, SpatialEntry, SpatialIndex};

/// Deterministic pseudo-random points without extra dependencies.
fn points(count: usize) -> Vec<(f64, f64)> {
    let mut state = 0x1234_5678_9abc_def0u64;
    (0..count)
        .map(|_| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let x = ((state >> 33) as f64 / u32::MAX as f64) * 1000.0;
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let y = ((state >> 33) as f64 / u32::MAX as f64) * 1000.0;
            (x, y)
        })
        .collect()
}

fn entry(x: f64, y: f64) -> SpatialEntry {
    SpatialEntry {
        object: ObjectId::new_v4(),
        bounds: Rect::new(x, y, 10.0, 10.0),
    }
}

fn build(count: usize) -> (RStarIndex, LinearIndex) {
    let mut tree = RStarIndex::new();
    let mut linear = LinearIndex::new();
    for (x, y) in points(count) {
        let item = entry(x, y);
        tree.insert(item.clone());
        linear.insert(item);
    }
    (tree, linear)
}

fn bench_query_aabb(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial/query-aabb");
    for count in [100usize, 1_000, 10_000] {
        let (tree, linear) = build(count);
        let probe = points(64);
        group.bench_with_input(
            BenchmarkId::new("rstar", format!("n={count}")),
            &(tree, probe.clone()),
            |b, (tree, probe)| {
                b.iter(|| {
                    let mut total = 0usize;
                    for (x, y) in probe {
                        let mut out = Vec::new();
                        tree.query_aabb(Rect::new(x - 15.0, y - 15.0, 30.0, 30.0), &mut out);
                        total += out.len();
                    }
                    total
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("linear", format!("n={count}")),
            &(linear, probe),
            |b, (linear, probe)| {
                b.iter(|| {
                    let mut total = 0usize;
                    for (x, y) in probe {
                        let mut out = Vec::new();
                        linear.query_aabb(Rect::new(x - 15.0, y - 15.0, 30.0, 30.0), &mut out);
                        total += out.len();
                    }
                    total
                })
            },
        );
    }
    group.finish();
}

fn bench_nearest(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial/nearest");
    for count in [1_000usize, 10_000] {
        let (tree, _linear) = build(count);
        let probe = points(32);
        group.bench_with_input(
            BenchmarkId::new("rstar", format!("n={count}")),
            &(tree, probe),
            |b, (tree, probe)| {
                b.iter(|| {
                    let mut total = 0usize;
                    for (x, y) in probe {
                        let mut out = Vec::new();
                        tree.nearest(Point::new(*x, *y), 60.0, &mut out);
                        total += out.len();
                    }
                    total
                })
            },
        );
    }
    group.finish();
}

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial/insert");
    for count in [100usize, 1_000] {
        group.bench_with_input(
            BenchmarkId::new("rstar", format!("n={count}")),
            &points(count),
            |b, source| {
                b.iter(|| {
                    let mut tree = RStarIndex::new();
                    for (x, y) in source {
                        tree.insert(entry(*x, *y));
                    }
                    tree.len()
                })
            },
        );
    }
    group.finish();
}

criterion_group!(
    spatial_benches,
    bench_query_aabb,
    bench_nearest,
    bench_insert
);
criterion_main!(spatial_benches);
