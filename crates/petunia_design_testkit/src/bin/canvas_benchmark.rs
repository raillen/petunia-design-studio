use petunia_design_testkit::{build_scene, FixtureOptions};
use std::time::Instant;

fn main() {
    println!("Running Canvas Benchmark...");
    let options = FixtureOptions::default();
    let start = Instant::now();
    let fixture = build_scene(options).expect("build scene failed");
    let elapsed = start.elapsed();
    println!(
        "Canvas fixture built in {:?}: {} objects, digest: {}",
        elapsed,
        fixture.all_ids.len(),
        fixture.digest_hex()
    );
}
