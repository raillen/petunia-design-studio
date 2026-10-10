//! Benchmarks for the render hot path: scan conversion, compositing
//! and a full frame, at interactive/authoring qualities.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use petunia_core::paint::{GradientInterpolation, GradientSpread, PaintSpace};
use petunia_core::FillRule;
use petunia_render::paint_eval::sample_linear;
use petunia_render::rasterize::{fill_path, stroke_path, Target};
use petunia_render::{
    frame_to_rgba8, output::linear_to_srgb_byte, paint_eval::sample_radial, Pixel,
};
use petunia_render_model::{RenderColor, RenderGradient, RenderGradientStop, RenderPath};

fn square_path(x0: f64, y0: f64, x1: f64, y1: f64) -> RenderPath {
    let mut path = RenderPath::new();
    path.push_contour(vec![(x0, y0), (x1, y0), (x1, y1), (x0, y1)], true);
    path
}

fn red(_x: f64, _y: f64) -> RenderColor {
    RenderColor {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    }
}

fn bench_fill(c: &mut Criterion) {
    let mut group = c.benchmark_group("render/scan-fill");
    for (name, side) in [("small", 32u32), ("normal", 128), ("stress", 512)] {
        let path = square_path(2.0, 2.0, (side - 2) as f64, (side - 2) as f64);
        group.bench_with_input(
            BenchmarkId::new("square", format!("{name}px={side}")),
            &path,
            |b, path| {
                b.iter(|| {
                    let mut target = Target::transparent(side, side).expect("target");
                    fill_path(
                        &mut target,
                        path,
                        FillRule::NonZero,
                        &red,
                        1.0,
                        petunia_core::appearance::BlendMode::Normal,
                    );
                    target.pixels.len()
                })
            },
        );
    }
    group.finish();
}

fn bench_stroke(c: &mut Criterion) {
    let mut group = c.benchmark_group("render/scan-stroke");
    let mut path = RenderPath::new();
    path.push_contour(vec![(4.0, 8.0), (124.0, 20.0), (20.0, 120.0)], false);
    group.bench_function("polyline-3pts-128px", |b| {
        b.iter(|| {
            let mut target = Target::transparent(128, 128).expect("target");
            stroke_path(
                &mut target,
                &path,
                &red,
                8.0,
                1.0,
                petunia_core::appearance::BlendMode::Normal,
            );
            target.pixels.len()
        })
    });
    group.finish();
}

fn bench_gradients(c: &mut Criterion) {
    let mut group = c.benchmark_group("render/paint-eval");
    let gradient = RenderGradient {
        stops: vec![
            RenderGradientStop {
                offset: 0.0,
                color: RenderColor::BLACK,
                midpoint: 0.5,
            },
            RenderGradientStop {
                offset: 0.5,
                color: RenderColor::WHITE,
                midpoint: 0.35,
            },
            RenderGradientStop {
                offset: 1.0,
                color: RenderColor::BLACK,
                midpoint: 0.5,
            },
        ],
        interpolation: GradientInterpolation::LinearRgb,
        spread: GradientSpread::Pad,
        space: PaintSpace::Object,
        start: (0.0, 0.0),
        end: (100.0, 0.0),
        radius: 50.0,
        start_angle: 0.0,
    };
    let oklab = RenderGradient {
        interpolation: GradientInterpolation::Oklab,
        ..gradient.clone()
    };
    group.bench_function("linear/srgb-stops", |b| {
        b.iter(|| sample_linear(&gradient, (50.0, 10.0)).expect("samples").r)
    });
    group.bench_function("linear/oklab-stops", |b| {
        b.iter(|| sample_linear(&oklab, (50.0, 10.0)).expect("samples").r)
    });
    group.bench_function("radial/sample", |b| {
        b.iter(|| sample_radial(&gradient, (25.0, 25.0)).expect("samples").r)
    });
    group.bench_function("conical/sample", |b| {
        b.iter(|| {
            petunia_render::paint_eval::sample_conical(&gradient, (25.0, 25.0))
                .expect("samples")
                .r
        })
    });
    group.finish();
}

fn bench_output(c: &mut Criterion) {
    let mut group = c.benchmark_group("render/output");
    for side in [64u32, 256] {
        let pixels: Vec<Pixel> = (0..side as usize * side as usize)
            .map(|index| Pixel {
                r: (index % 255) as f32 / 255.0,
                g: 0.5,
                b: 0.25,
                a: 1.0,
            })
            .collect();
        group.bench_with_input(
            BenchmarkId::new("frame-rgba8", format!("{side}px")),
            &pixels,
            |b, pixels| b.iter(|| frame_to_rgba8(pixels, side, false).len()),
        );
    }
    group.finish();
}

fn bench_blend(c: &mut Criterion) {
    let mut group = c.benchmark_group("render/blend");
    let source = Pixel {
        r: 0.7,
        g: 0.2,
        b: 0.1,
        a: 0.6,
    };
    let backdrop = Pixel {
        r: 0.1,
        g: 0.4,
        b: 0.8,
        a: 1.0,
    };
    for mode in [
        petunia_core::appearance::BlendMode::Normal,
        petunia_core::appearance::BlendMode::Multiply,
        petunia_core::appearance::BlendMode::Luminosity,
    ] {
        group.bench_with_input(
            BenchmarkId::new("composite", format!("{mode:?}")),
            &(source, backdrop, mode),
            |b, (source, backdrop, mode)| {
                b.iter(|| petunia_render::composite(*source, *backdrop, *mode))
            },
        );
    }
    let value = 0.5f32;
    group.bench_function("transfer/linear-to-srgb", |b| {
        b.iter(|| linear_to_srgb_byte(value, 0.0))
    });
    group.finish();
}

criterion_group!(
    render_benches,
    bench_fill,
    bench_stroke,
    bench_gradients,
    bench_output,
    bench_blend
);
criterion_main!(render_benches);
