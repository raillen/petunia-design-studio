//! Deterministic scan conversion with subpixel coverage.
//!
//! Flattened rings convert to coverage through a fixed 4×4
//! supersample grid; fill rules match Core semantics exactly, and
//! strokes evaluate a distance field (round joins and caps by
//! construction). Paint arrives as a sampler so gradients evaluate
//! positionally without coupling this module to paint parameters.

use crate::compositor::{composite, Pixel};
use petunia_core::appearance::BlendMode;
use petunia_core::FillRule;
use petunia_render_model::{RenderColor, RenderPath};

/// Supersamples per pixel axis (4×4 = 16 coverage steps).
pub const SAMPLES_PER_AXIS: usize = 4;

/// Premultiplied linear working target.
#[derive(Debug, Clone)]
pub struct Target {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Pixel>,
}

impl Target {
    /// Transparent target of the given size.
    pub fn transparent(width: u32, height: u32) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        Some(Self {
            width,
            height,
            pixels: vec![Pixel::CLEAR; (width as usize) * (height as usize)],
        })
    }

    /// Fill with one premultiplied color.
    pub fn clear(&mut self, color: Pixel) {
        self.pixels.fill(color);
    }

    /// Blend one pixel through the given mode.
    pub fn blend_pixel(&mut self, x: u32, y: u32, src: Pixel, mode: BlendMode) {
        if x >= self.width || y >= self.height {
            return;
        }
        let index = (y * self.width + x) as usize;
        self.pixels[index] = composite(src, self.pixels[index], mode);
    }

    /// Read one pixel.
    #[must_use]
    pub fn pixel(&self, x: u32, y: u32) -> Option<Pixel> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(self.pixels[(y * self.width + x) as usize])
    }
}

/// Reference twin of the engine winding test, kept beside the
/// sampler it serves so the renderer never depends on engine code.
fn point_in_ring(x: f64, y: f64, ring: &[(f64, f64)], rule: FillRule) -> bool {
    if ring.len() < 3 {
        return false;
    }
    let mut winding = 0i32;
    for index in 0..ring.len() {
        let a = ring[index];
        let b = ring[(index + 1) % ring.len()];
        let cross = (b.0 - a.0) * (y - a.1) - (x - a.0) * (b.1 - a.1);
        if a.1 <= y {
            if b.1 > y && cross > 0.0 {
                winding += 1;
            }
        } else if b.1 <= y && cross < 0.0 {
            winding -= 1;
        }
    }
    match rule {
        FillRule::NonZero => winding != 0,
        FillRule::EvenOdd => winding % 2 != 0,
    }
}

fn point_segment_distance(x: f64, y: f64, a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let length_squared = dx * dx + dy * dy;
    if length_squared == 0.0 {
        return ((x - a.0).powi(2) + (y - a.1).powi(2)).sqrt();
    }
    let t = (((x - a.0) * dx + (y - a.1) * dy) / length_squared).clamp(0.0, 1.0);
    let nx = a.0 + t * dx;
    let ny = a.1 + t * dy;
    ((x - nx).powi(2) + (y - ny).powi(2)).sqrt()
}

/// Fill flattened contours with a positional sampler. Coverage comes
/// from the supersample grid; `opacity` scales source alpha.
pub fn fill_path(
    target: &mut Target,
    path: &RenderPath,
    rule: FillRule,
    sampler: &dyn Fn(f64, f64) -> RenderColor,
    opacity: f32,
    mode: BlendMode,
) {
    let step = 1.0 / SAMPLES_PER_AXIS as f64;
    for y in 0..target.height {
        for x in 0..target.width {
            let mut covered = 0usize;
            for sy in 0..SAMPLES_PER_AXIS {
                for sx in 0..SAMPLES_PER_AXIS {
                    let px = x as f64 + (sx as f64 + 0.5) * step;
                    let py = y as f64 + (sy as f64 + 0.5) * step;
                    if path
                        .contours
                        .iter()
                        .zip(path.closed.iter())
                        .any(|(contour, closed)| {
                            *closed && contour.len() >= 3 && point_in_ring(px, py, contour, rule)
                        })
                    {
                        covered += 1;
                    }
                }
            }
            if covered == 0 {
                continue;
            }
            let coverage = covered as f32 / (SAMPLES_PER_AXIS * SAMPLES_PER_AXIS) as f32;
            let sample = sampler(x as f64 + 0.5, y as f64 + 0.5);
            target.blend_pixel(
                x,
                y,
                straight_to_premultiplied(sample, opacity * coverage),
                mode,
            );
        }
    }
}

/// Stroke flattened polylines as a distance field of half `width`.
/// Closed rings join up; open ones cap round by construction.
pub fn stroke_path(
    target: &mut Target,
    path: &RenderPath,
    sampler: &dyn Fn(f64, f64) -> RenderColor,
    width: f64,
    opacity: f32,
    mode: BlendMode,
) {
    if !(width.is_finite() && width > 0.0) {
        return;
    }
    let half = width / 2.0;
    let step = 1.0 / SAMPLES_PER_AXIS as f64;
    for y in 0..target.height {
        for x in 0..target.width {
            let mut covered = 0usize;
            for sy in 0..SAMPLES_PER_AXIS {
                for sx in 0..SAMPLES_PER_AXIS {
                    let px = x as f64 + (sx as f64 + 0.5) * step;
                    let py = y as f64 + (sy as f64 + 0.5) * step;
                    if within_stroke(path, px, py, half) {
                        covered += 1;
                    }
                }
            }
            if covered == 0 {
                continue;
            }
            let coverage = covered as f32 / (SAMPLES_PER_AXIS * SAMPLES_PER_AXIS) as f32;
            let sample = sampler(x as f64 + 0.5, y as f64 + 0.5);
            target.blend_pixel(
                x,
                y,
                straight_to_premultiplied(sample, opacity * coverage),
                mode,
            );
        }
    }
}

fn within_stroke(path: &RenderPath, x: f64, y: f64, half: f64) -> bool {
    path.contours
        .iter()
        .zip(path.closed.iter())
        .any(|(contour, closed)| {
            if contour.len() < 2 {
                return contour.len() == 1 && (x - contour[0].0).hypot(y - contour[0].1) <= half;
            }
            let mut segments: Vec<((f64, f64), (f64, f64))> =
                contour.windows(2).map(|pair| (pair[0], pair[1])).collect();
            if *closed && contour.len() > 2 {
                segments.push((contour[contour.len() - 1], contour[0]));
            }
            segments
                .iter()
                .any(|(a, b)| point_segment_distance(x, y, *a, *b) <= half)
        })
}

fn straight_to_premultiplied(color: RenderColor, alpha_scale: f32) -> Pixel {
    let a = (color.a * alpha_scale).clamp(0.0, 1.0);
    Pixel {
        r: color.r * a,
        g: color.g * a,
        b: color.b * a,
        a,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn filled_square_covers_interior_not_exterior() {
        let mut target = Target::transparent(20, 20).expect("target");
        // Half-pixel offsets so edges cut through pixels.
        fill_path(
            &mut target,
            &square_path(5.5, 5.5, 15.5, 15.5),
            FillRule::NonZero,
            &red,
            1.0,
            BlendMode::Normal,
        );
        let inside = target.pixel(10, 10).expect("pixel");
        assert!((inside.r - 1.0).abs() < 1e-5 && inside.a == 1.0);
        let outside = target.pixel(0, 0).expect("pixel");
        assert_eq!(outside, Pixel::CLEAR);
        // Edge pixels carry partial coverage, never full or empty here.
        let edge = target.pixel(5, 10).expect("pixel");
        assert!(edge.a > 0.0 && edge.a < 1.0, "{edge:?}");
    }

    #[test]
    fn stroke_follows_centerline_with_round_caps() {
        let mut target = Target::transparent(20, 20).expect("target");
        let mut path = RenderPath::new();
        path.push_contour(vec![(2.0, 10.0), (18.0, 10.0)], false);
        stroke_path(&mut target, &path, &red, 4.0, 1.0, BlendMode::Normal);
        let middle = target.pixel(10, 10).expect("pixel");
        assert!(middle.a > 0.9, "{middle:?}");
        let far = target.pixel(10, 2).expect("pixel");
        assert_eq!(far, Pixel::CLEAR);
        // Round cap reaches past the endpoint.
        let cap = target.pixel(0, 10).expect("pixel");
        assert!(cap.a > 0.0, "{cap:?}");
    }
}
