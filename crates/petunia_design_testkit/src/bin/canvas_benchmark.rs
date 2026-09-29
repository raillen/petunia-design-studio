use petunia_design_testkit::{build_scene, FixtureOptions};
use std::time::Instant;

fn main() {
    println!("Running Canvas Benchmark...");
    for &object_count in &[500_usize, 2000, 10000] {
        let options = FixtureOptions {
            object_count,
            ..FixtureOptions::default()
        };
        let start = Instant::now();
        let fixture = build_scene(options).expect("build scene failed");
        let build_elapsed = start.elapsed();
        println!(
            "Canvas fixture built in {:?}: {} objects, digest: {}",
            build_elapsed,
            fixture.all_ids.len(),
            fixture.digest_hex()
        );

        // Time repeated `canvas_snapshot` calls: the first call is a cache
        // miss (full rebuild), the rest must be cache hits (Arc bumps +
        // viewport filter only). The reported `canvas-snapshot` line is the
        // steady-state per-frame cost.
        let shell = fixture.shell;
        let warmup = shell.canvas_snapshot();
        let visible = warmup.objects.len();
        let iterations = 50;
        let start = Instant::now();
        for _ in 0..iterations {
            let snapshot = shell.canvas_snapshot();
            std::hint::black_box(snapshot.objects.len());
        }
        let elapsed = start.elapsed();
        let mean_us = elapsed.as_micros() as f64 / iterations as f64;
        println!(
            "canvas-snapshot objects={} visible={} iterations={} total={:?} mean={:.1}us/snapshot",
            object_count, visible, iterations, elapsed, mean_us
        );
    }
}
