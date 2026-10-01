use petunia_design_testkit::{build_scene, FixtureOptions};
use serde::Serialize;
use std::time::Instant;

const USAGE: &str = "canvas-benchmark [--objects 500,2000,10000] [--iterations 50] [--warmup 1] [--json]\nMeasures fixture construction and canvas_snapshot only; excludes painting/presentation.";

#[derive(Debug, PartialEq)]
struct Options {
    objects: Vec<usize>,
    iterations: usize,
    warmup: usize,
    json: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            objects: vec![500, 2000, 10000],
            iterations: 50,
            warmup: 1,
            json: false,
        }
    }
}

fn parse_options(args: impl IntoIterator<Item = String>) -> Result<Option<Options>, String> {
    let mut options = Options::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--json" => options.json = true,
            "--objects" | "--iterations" | "--warmup" => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("{arg} requires a value"))?;
                let parse = |value: &str| {
                    value
                        .parse::<usize>()
                        .map_err(|_| format!("invalid value for {arg}: {value}"))
                };
                match arg.as_str() {
                    "--objects" => {
                        options.objects = value.split(',').map(parse).collect::<Result<_, _>>()?;
                        if options.objects.len() > 32
                            || options
                                .objects
                                .iter()
                                .any(|&n| !(1..=1_000_000).contains(&n))
                        {
                            return Err("--objects accepts 1–32 counts in 1..=1000000".into());
                        }
                    }
                    "--iterations" => options.iterations = parse(&value)?,
                    _ => options.warmup = parse(&value)?,
                }
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }
    if !(1..=100_000).contains(&options.iterations) || options.warmup > 100_000 {
        return Err("iterations must be in 1..=100000; warmup in 0..=100000".into());
    }
    Ok(Some(options))
}

#[derive(Serialize)]
struct SampleSummary {
    mean_us: f64,
    p50_us: f64,
    p95_us: f64,
    p99_us: f64,
}

fn summarize(mut samples: Vec<f64>) -> SampleSummary {
    let mean_us = samples.iter().sum::<f64>() / samples.len() as f64;
    samples.sort_by(f64::total_cmp);
    // Nearest-rank percentiles. Small runs are smoke checks, not tail estimates.
    let percentile =
        |percentage: usize| samples[(samples.len() * percentage).div_ceil(100).saturating_sub(1)];
    SampleSummary {
        mean_us,
        p50_us: percentile(50),
        p95_us: percentile(95),
        p99_us: percentile(99),
    }
}

#[derive(Serialize)]
struct SceneResult {
    objects: usize,
    visible: usize,
    digest: String,
    fixture_build_us: f64,
    cold_snapshot_us: f64,
    warmup: usize,
    iterations: usize,
    snapshot: SampleSummary,
}

#[derive(Serialize)]
struct Report {
    schema_version: u32,
    measurement: &'static str,
    excludes: &'static str,
    profile: &'static str,
    os: &'static str,
    arch: &'static str,
    scenes: Vec<SceneResult>,
}

fn run(options: Options) -> Result<(), String> {
    let mut scenes = Vec::new();
    for object_count in options.objects {
        let start = Instant::now();
        let fixture = build_scene(FixtureOptions {
            object_count,
            ..FixtureOptions::default()
        })
        .map_err(|error| format!("build scene failed: {error}"))?;
        let fixture_build_us = start.elapsed().as_secs_f64() * 1e6;
        let digest = fixture.digest_hex();
        let shell = fixture.shell;
        let start = Instant::now();
        let cold = shell.canvas_snapshot();
        let cold_snapshot_us = start.elapsed().as_secs_f64() * 1e6;
        let visible = cold.objects.len();
        std::hint::black_box(&cold);
        drop(cold);
        for _ in 0..options.warmup {
            std::hint::black_box(shell.canvas_snapshot());
        }
        let mut samples = Vec::with_capacity(options.iterations);
        for _ in 0..options.iterations {
            let start = Instant::now();
            // Include disposal: every iteration has the same snapshot lifetime.
            std::hint::black_box(shell.canvas_snapshot());
            samples.push(start.elapsed().as_secs_f64() * 1e6);
        }
        scenes.push(SceneResult {
            objects: object_count,
            visible,
            digest,
            fixture_build_us,
            cold_snapshot_us,
            warmup: options.warmup,
            iterations: options.iterations,
            snapshot: summarize(samples),
        });
    }
    let report = Report {
        schema_version: 1,
        measurement: "fixture construction and canvas_snapshot; microseconds",
        excludes: "painting, GPU work, presentation, input latency, allocation/RSS profiling",
        profile: if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        scenes,
    };
    if options.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
        );
    } else {
        println!("{USAGE}\nProfile: {}", report.profile);
        for s in report.scenes {
            println!("objects={} visible={} digest={} build={:.1}us cold={:.1}us warmup={} iterations={} snapshot mean={:.1}us p50={:.1}us p95={:.1}us p99={:.1}us", s.objects, s.visible, s.digest, s.fixture_build_us, s.cold_snapshot_us, s.warmup, s.iterations, s.snapshot.mean_us, s.snapshot.p50_us, s.snapshot.p95_us, s.snapshot.p99_us);
        }
    }
    Ok(())
}

fn main() {
    let result = parse_options(std::env::args().skip(1)).and_then(|options| match options {
        Some(options) => run(options),
        None => {
            println!("{USAGE}");
            Ok(())
        }
    });
    if let Err(error) = result {
        eprintln!("{error}\n{USAGE}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Option<Options>, String> {
        parse_options(args.iter().map(|s| (*s).to_owned()))
    }

    #[test]
    fn arguments_control_scene_counts_samples_and_output() {
        assert_eq!(
            parse(&[
                "--objects",
                "12,34",
                "--iterations",
                "7",
                "--warmup",
                "0",
                "--json"
            ])
            .unwrap(),
            Some(Options {
                objects: vec![12, 34],
                iterations: 7,
                warmup: 0,
                json: true
            })
        );
        assert_eq!(parse(&[]).unwrap(), Some(Options::default()));
        assert_eq!(parse(&["--help"]).unwrap(), None);
    }

    #[test]
    fn invalid_or_unbounded_arguments_fail_instead_of_being_ignored() {
        for args in [
            &["--objects", "0"][..],
            &["--objects", "2,"],
            &["--iterations", "0"],
            &["--warmup", "100001"],
            &["--objects", "1000001"],
            &["--iterations"],
            &["--unknown"],
        ] {
            assert!(parse(args).is_err(), "{args:?}");
        }
    }

    #[test]
    fn nearest_rank_percentiles_include_tail_and_single_sample() {
        let summary = summarize(vec![4.0, 1.0, 3.0, 2.0]);
        assert_eq!(summary.mean_us, 2.5);
        assert_eq!(summary.p50_us, 2.0);
        assert_eq!(summary.p95_us, 4.0);
        assert_eq!(summary.p99_us, 4.0);
        assert_eq!(summarize(vec![7.0]).p99_us, 7.0);
    }
}
