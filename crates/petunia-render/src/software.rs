//! Tiled software reference renderer: deterministic pixels.
//!
//! Frames execute in paint order. Flat draws rasterize through
//! parallel tiles sharing read-only device geometry; isolated groups
//! render into full-frame temporaries (effects stay seam-free),
//! then composite as units. Intermediate tiles merge disjointly, so
//! parallel and sequential runs agree.

use crate::adjustments::{apply_adjustment, StraightPixel};
use crate::compositor::{composite, Pixel};
use crate::error::{RenderError, Result};
use crate::graph::RenderGraph;
use crate::output::frame_to_rgba8;
use crate::paint_eval::{sample_linear, sample_radial};
use crate::rasterize::{fill_path, stroke_path, Target};
use petunia_core::appearance::BlendMode;
use petunia_core::FillRule;
use petunia_core::Rect;
use petunia_render_model::{
    IsolationMode, RenderAppearance, RenderClip, RenderEffect, RenderFrame, RenderGradient,
    RenderMask, RenderPaint, RenderPrimitive, RenderStats, RenderTarget, ViewTransform,
};
use rayon::prelude::*;

/// Software reference backend: headless, deterministic, CPU-only.
pub struct SoftwareRenderer {
    _cache_bytes: usize,
    _pool_bytes: usize,
}

impl SoftwareRenderer {
    /// Create a renderer with byte capacities for cache and pool.
    /// Capacities are reserved for the renderer-owned stores; the
    /// v0.1 pipeline allocates frame temporaries directly.
    #[must_use]
    pub fn new(cache_bytes: usize, pool_bytes: usize) -> Self {
        Self {
            _cache_bytes: cache_bytes,
            _pool_bytes: pool_bytes,
        }
    }

    /// Render one frame into RGBA8 bytes plus statistics.
    pub fn render(
        &mut self,
        frame: &RenderFrame,
        options: &crate::backend::RenderOptions,
    ) -> Result<(Vec<u8>, RenderStats)> {
        let target = frame.target;
        if target.width == 0 || target.height == 0 {
            return Err(RenderError::Draw("empty render target".to_string()));
        }
        let tile_edge = options.tile_size.clamp(16, 512);
        let background = srgb_to_linear_pixel(options.background);
        let items = prepare_items(frame);
        let mut pixels = vec![background; (target.width as usize) * (target.height as usize)];
        let mut drawn = 0usize;
        for item in &items {
            drawn += draw_item(&mut pixels, target.width, item, frame.view, tile_edge)?;
        }
        let stats = RenderStats {
            primitives_drawn: drawn,
            tiles_processed: tile_count(frame, tile_edge),
            pixels_written: pixels.iter().filter(|pixel| **pixel != background).count() as u64,
        };
        Ok((frame_to_rgba8(&pixels, target.width, false), stats))
    }
}

impl crate::backend::RenderBackend for SoftwareRenderer {
    fn name(&self) -> &str {
        "SoftwareReference"
    }

    fn render(
        &mut self,
        frame: &petunia_render_model::RenderFrame,
        options: &crate::backend::RenderOptions,
    ) -> crate::error::Result<(Vec<u8>, petunia_render_model::RenderStats)> {
        SoftwareRenderer::render(self, frame, options)
    }
}

fn full_rect(target: RenderTarget) -> Rect {
    Rect::new(0.0, 0.0, target.width as f64, target.height as f64)
}

fn tile_count(frame: &RenderFrame, tile_size: u32) -> usize {
    let graph = RenderGraph::compile_with_tile_size(
        &frame.snapshot,
        frame.view,
        full_rect(frame.target),
        tile_size,
    );
    graph.tiles.len()
}

fn srgb_to_linear_pixel(color: petunia_core::ColorRgba) -> Pixel {
    Pixel {
        r: encoded_to_linear(color.r),
        g: encoded_to_linear(color.g),
        b: encoded_to_linear(color.b),
        a: color.a.clamp(0.0, 1.0),
    }
}

fn encoded_to_linear(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

/// One paint sampler over document space.
#[derive(Clone)]
enum PaintSampler {
    Solid(petunia_render_model::RenderColor),
    Linear(RenderGradient),
    Radial(RenderGradient),
}

impl PaintSampler {
    fn sample(&self, doc_x: f64, doc_y: f64) -> petunia_render_model::RenderColor {
        match self {
            Self::Solid(color) => *color,
            Self::Linear(gradient) => sample_linear(gradient, (doc_x, doc_y))
                .unwrap_or(petunia_render_model::RenderColor::TRANSPARENT),
            Self::Radial(gradient) => sample_radial(gradient, (doc_x, doc_y))
                .unwrap_or(petunia_render_model::RenderColor::TRANSPARENT),
        }
    }
}

fn sampler_for(paint: &RenderPaint) -> PaintSampler {
    match paint {
        RenderPaint::Solid(color) => PaintSampler::Solid(*color),
        RenderPaint::LinearGradient(gradient) => PaintSampler::Linear(gradient.clone()),
        RenderPaint::RadialGradient(gradient) => PaintSampler::Radial(gradient.clone()),
    }
}

/// Device-space vector content with resolved paint.
struct PreparedVector {
    rings: Vec<(Vec<(f64, f64)>, bool)>,
    fill: Option<PaintSampler>,
    stroke: Option<(PaintSampler, f64)>,
    opacity: f32,
    blend: BlendMode,
}

/// One frame entry: flat draws rasterize in place; isolated groups
/// composite through a temporary with effects, clip and group blend.
enum FrameItem {
    Draw(PreparedVector),
    Isolated {
        children: Vec<FrameItem>,
        opacity: f32,
        blend: BlendMode,
        effects: Vec<RenderEffect>,
        clip: Option<RenderClip>,
        mask: Option<RenderMask>,
    },
}

fn prepare_items(frame: &RenderFrame) -> Vec<FrameItem> {
    let mut out = Vec::new();
    for page in &frame.snapshot.pages {
        for primitive in &page.primitives {
            collect_primitive(primitive, frame.view, 1.0, BlendMode::Normal, &mut out);
        }
    }
    out
}

fn collect_primitive(
    primitive: &RenderPrimitive,
    view: ViewTransform,
    opacity: f32,
    blend: BlendMode,
    out: &mut Vec<FrameItem>,
) {
    match primitive {
        RenderPrimitive::Vector(vector) => {
            out.push(FrameItem::Draw(prepared_vector(
                &vector.geometry.contours,
                &vector.geometry.closed,
                &vector.appearance,
                &vector.transform,
                view,
                opacity,
                blend,
            )));
        }
        RenderPrimitive::Group(group) => {
            let simple = group.blend_mode == BlendMode::Normal
                && group.opacity >= 1.0
                && group.effects.is_empty()
                && group.mask.is_none()
                && group.clip.is_none()
                && matches!(group.isolation, IsolationMode::Flattened);
            if simple {
                for child in &group.children {
                    collect_primitive(child, view, opacity * group.opacity, blend, out);
                }
            } else {
                let mut children = Vec::new();
                for child in &group.children {
                    collect_primitive(child, view, 1.0, BlendMode::Normal, &mut children);
                }
                out.push(FrameItem::Isolated {
                    children,
                    opacity: opacity * group.opacity,
                    blend: group.blend_mode,
                    effects: group.effects.clone(),
                    clip: group.clip.clone(),
                    mask: group.mask.clone(),
                });
            }
        }
        // Text, images and raster surfaces need decoded resources or
        // shaped runs the compiler only emits with providers present;
        // the v0.1 compiler degrades them with warnings instead.
        RenderPrimitive::Text(_) | RenderPrimitive::Image(_) | RenderPrimitive::Raster(_) => {}
    }
}

fn prepared_vector(
    contours: &[Vec<(f64, f64)>],
    closed: &[bool],
    appearance: &RenderAppearance,
    transform: &petunia_core::Transform2D,
    view: ViewTransform,
    opacity: f32,
    blend: BlendMode,
) -> PreparedVector {
    let rings = contours
        .iter()
        .zip(closed.iter())
        .map(|(contour, closed)| {
            (
                contour
                    .iter()
                    .map(|(x, y)| {
                        let (lx, ly) = (
                            transform.a * x + transform.c * y + transform.tx,
                            transform.b * x + transform.d * y + transform.ty,
                        );
                        view.apply(lx, ly)
                    })
                    .collect(),
                *closed,
            )
        })
        .collect();
    PreparedVector {
        rings,
        fill: appearance.fill.as_ref().map(sampler_for),
        stroke: appearance.stroke.as_ref().map(|stroke| {
            (
                sampler_for(&stroke.paint),
                (stroke.width * view.scale).max(0.0),
            )
        }),
        opacity,
        blend,
    }
}

/// Draw one frame item onto device pixels; returns leaves drawn.
fn draw_item(
    pixels: &mut [Pixel],
    stride: u32,
    item: &FrameItem,
    view: ViewTransform,
    tile_edge: u32,
) -> Result<usize> {
    match item {
        FrameItem::Draw(vector) => {
            draw_vector_parallel(pixels, stride, vector, view, tile_edge);
            Ok(1)
        }
        FrameItem::Isolated {
            children,
            opacity,
            blend,
            effects,
            clip,
            mask,
        } => {
            if mask.is_some() {
                // Masks need decoded coverage resources; failing loud
                // beats silently wrong pixels.
                return Err(RenderError::Draw(
                    "masks need decoded resources in v0.1".to_string(),
                ));
            }
            // Full-frame temporary: effects stay seam-free.
            let height = pixels.len() as u32 / stride.max(1);
            let mut temp = vec![Pixel::CLEAR; pixels.len()];
            let mut drawn = 0usize;
            for child in children {
                drawn += draw_item(&mut temp, stride, child, view, tile_edge)?;
            }
            apply_effects(&mut temp, stride, effects)?;
            composite_temp(
                pixels,
                stride,
                &temp,
                *opacity,
                *blend,
                clip.as_ref(),
                height,
            )?;
            Ok(drawn)
        }
    }
}

/// One rasterized tile: its origin index plus exact extent. `height`
/// is retained for edge-tile indexing when frames are not aligned to
/// the nominal tile edge.
struct TilePixels {
    tx: u32,
    ty: u32,
    width: u32,
    _height: u32,
    pixels: Vec<Pixel>,
}

fn draw_vector_parallel(
    pixels: &mut [Pixel],
    stride: u32,
    vector: &PreparedVector,
    view: ViewTransform,
    tile_edge: u32,
) {
    let height = pixels.len() as u32 / stride.max(1);
    let bounds = vector_bounds(vector);
    let tiles = tiles_for_rect(bounds, stride, height, tile_edge);
    if tiles.is_empty() {
        return;
    }
    let buffers: Vec<TilePixels> = tiles
        .into_par_iter()
        .map(|(tx, ty, rect)| {
            let width = rect.2.min(stride.saturating_sub(rect.0)) as usize;
            let height = rect.3.min(height.saturating_sub(rect.1)) as usize;
            let mut local = Target {
                width: width as u32,
                height: height as u32,
                pixels: vec![Pixel::CLEAR; width * height],
            };
            rasterize_vector(&mut local, vector, view, (rect.0 as f64, rect.1 as f64));
            TilePixels {
                tx,
                ty,
                width: width as u32,
                _height: height as u32,
                pixels: local.pixels,
            }
        })
        .collect();
    for tile in &buffers {
        for (index, pixel) in tile.pixels.iter().enumerate() {
            let x = tile.tx * tile_edge + (index as u32 % tile.width);
            let y = tile.ty * tile_edge + (index as u32 / tile.width.max(1));
            if x < stride && (y as usize) < pixels.len() / stride as usize {
                let slot = (y * stride + x) as usize;
                if let Some(slot) = pixels.get_mut(slot) {
                    *slot = composite(*pixel, *slot, vector.blend);
                }
            }
        }
    }
}

fn vector_bounds(vector: &PreparedVector) -> Rect {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for (ring, _) in &vector.rings {
        for (x, y) in ring {
            min_x = min_x.min(*x);
            min_y = min_y.min(*y);
            max_x = max_x.max(*x);
            max_y = max_y.max(*y);
        }
    }
    if !min_x.is_finite() {
        return Rect::new(0.0, 0.0, 0.0, 0.0);
    }
    Rect::new(
        min_x,
        min_y,
        (max_x - min_x).max(0.0),
        (max_y - min_y).max(0.0),
    )
}

/// One tile range in device space: origin index plus exact extent.
type TileRect = (u32, u32, u32, u32);

fn tiles_for_rect(
    bounds: Rect,
    stride: u32,
    height: u32,
    tile_edge: u32,
) -> Vec<(u32, u32, TileRect)> {
    let edge = tile_edge;
    if stride == 0 || height == 0 {
        return Vec::new();
    }
    let x0 = (bounds.x.max(0.0) / edge as f64).floor().max(0.0) as u32;
    let y0 = (bounds.y.max(0.0) / edge as f64).floor().max(0.0) as u32;
    let x1 = ((bounds.x + bounds.width).min(stride as f64) / edge as f64).ceil() as u32;
    let y1 = ((bounds.y + bounds.height).min(height as f64) / edge as f64).ceil() as u32;
    let mut out = Vec::new();
    for ty in y0..y1.max(y0 + 1) {
        for tx in x0..x1.max(x0 + 1) {
            if tx * edge >= stride || ty * edge >= height {
                continue;
            }
            out.push((tx, ty, (tx * edge, ty * edge, edge, edge)));
        }
    }
    // Clamp degenerate ranges to the origin tile when bounds are empty.
    if out.is_empty() {
        out.push((0, 0, (0, 0, edge, edge)));
    }
    out
}

fn rasterize_vector(
    target: &mut Target,
    vector: &PreparedVector,
    view: ViewTransform,
    origin: (f64, f64),
) {
    let to_local = |x: f64, y: f64| (x - origin.0, y - origin.1);
    let to_doc = |x: f64, y: f64| {
        (
            (x + origin.0 - view.offset_x) / view.scale,
            (y + origin.1 - view.offset_y) / view.scale,
        )
    };
    if view.scale == 0.0 {
        return;
    }
    if let Some(fill) = &vector.fill {
        let sampler = |x: f64, y: f64| {
            let (doc_x, doc_y) = to_doc(x, y);
            fill.sample(doc_x, doc_y)
        };
        let path = local_path(&vector.rings, to_local);
        fill_path(
            target,
            &path,
            FillRule::NonZero,
            &sampler,
            vector.opacity,
            BlendMode::Normal,
        );
    }
    if let Some((stroke, width)) = &vector.stroke {
        let sampler = |x: f64, y: f64| {
            let (doc_x, doc_y) = to_doc(x, y);
            stroke.sample(doc_x, doc_y)
        };
        let path = local_path(&vector.rings, to_local);
        stroke_path(
            target,
            &path,
            &sampler,
            *width,
            vector.opacity,
            BlendMode::Normal,
        );
    }
}

fn local_path(
    rings: &[(Vec<(f64, f64)>, bool)],
    to_local: impl Fn(f64, f64) -> (f64, f64) + Copy,
) -> petunia_render_model::RenderPath {
    let mut path = petunia_render_model::RenderPath::new();
    for (ring, closed) in rings {
        path.push_contour(
            ring.iter().map(|(x, y)| to_local(*x, *y)).collect(),
            *closed,
        );
    }
    path
}

/// Apply group effects to a temporary, then composite it over the
/// destination with group opacity and blend.
fn composite_temp(
    pixels: &mut [Pixel],
    stride: u32,
    temp: &[Pixel],
    opacity: f32,
    blend: BlendMode,
    clip: Option<&RenderClip>,
    _height: u32,
) -> Result<()> {
    for (index, pixel) in temp.iter().enumerate() {
        let x = index as u32 % stride;
        let y = index as u32 / stride;
        if pixel.a <= 0.0 {
            continue;
        }
        if let Some(clip) = clip {
            if !clip_contains(clip, x as f64 + 0.5, y as f64 + 0.5) {
                continue;
            }
        }
        let scaled = Pixel {
            r: pixel.r * opacity,
            g: pixel.g * opacity,
            b: pixel.b * opacity,
            a: pixel.a * opacity,
        };
        if let Some(slot) = pixels.get_mut(index) {
            *slot = composite(scaled, *slot, blend);
        }
    }
    Ok(())
}

fn clip_contains(clip: &RenderClip, x: f64, y: f64) -> bool {
    match clip {
        RenderClip::Rect(rect) => {
            x >= rect.x && x <= rect.x + rect.width && y >= rect.y && y <= rect.y + rect.height
        }
        RenderClip::Polygon(points) => point_in_poly(x, y, points),
    }
}

fn point_in_poly(x: f64, y: f64, ring: &[(f64, f64)]) -> bool {
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
    winding != 0
}

/// Blur, shadow and adjustment passes over a temporary surface.
fn apply_effects(pixels: &mut [Pixel], stride: u32, effects: &[RenderEffect]) -> Result<()> {
    for effect in effects {
        match effect {
            RenderEffect::Blur { sigma_x, sigma_y } => {
                separable_blur(pixels, stride, *sigma_x, *sigma_y);
            }
            RenderEffect::DropShadow(shadow) => {
                apply_drop_shadow(pixels, stride, shadow);
            }
            RenderEffect::Adjustment(adjustment) => {
                for pixel in pixels.iter_mut() {
                    let straight = StraightPixel {
                        r: unpremult(pixel.r, pixel.a),
                        g: unpremult(pixel.g, pixel.a),
                        b: unpremult(pixel.b, pixel.a),
                        a: pixel.a,
                    };
                    match apply_adjustment(adjustment, straight) {
                        Ok(out) => {
                            *pixel = Pixel {
                                r: out.r * out.a,
                                g: out.g * out.a,
                                b: out.b * out.a,
                                a: out.a,
                            };
                        }
                        Err(error) => return Err(RenderError::Draw(error.to_string())),
                    }
                }
            }
        }
    }
    Ok(())
}

fn unpremult(channel: f32, alpha: f32) -> f32 {
    if alpha <= 0.0 {
        0.0
    } else {
        channel / alpha
    }
}

fn blur_kernel_1d(sigma: f64) -> Vec<f32> {
    let radius = (sigma * 3.0).ceil().max(0.0) as usize;
    if radius == 0 {
        return vec![1.0];
    }
    let mut weights: Vec<f32> = (0..=radius)
        .map(|offset| (-0.5 * (offset as f64 / sigma).powi(2)).exp() as f32)
        .collect();
    let total = weights[0] + 2.0 * weights[1..].iter().sum::<f32>();
    for weight in weights.iter_mut() {
        *weight /= total;
    }
    weights
}

fn separable_blur(pixels: &mut [Pixel], stride: u32, sigma_x: f64, sigma_y: f64) {
    if stride == 0 || pixels.is_empty() {
        return;
    }
    if !(sigma_x.is_finite() && sigma_y.is_finite() && sigma_x >= 0.0 && sigma_y >= 0.0) {
        return;
    }
    let height = pixels.len() as u32 / stride;
    let kernel_x = blur_kernel_1d(sigma_x);
    let kernel_y = blur_kernel_1d(sigma_y);
    let radius_x = kernel_x.len() - 1;
    let radius_y = kernel_y.len() - 1;
    if radius_x == 0 && radius_y == 0 {
        return;
    }
    let source = pixels.to_vec();
    let sample = |x: i64, y: i64| -> Pixel {
        if x < 0 || y < 0 || x >= stride as i64 || y >= height as i64 {
            // Documental blur reads transparent black outside.
            Pixel::CLEAR
        } else {
            source[(y as u32 * stride + x as u32) as usize]
        }
    };
    let mut middle = source.clone();
    for y in 0..height {
        for x in 0..stride {
            let mut acc = [0.0f32; 4];
            for (offset, weight) in kernel_x.iter().enumerate() {
                let o = offset as i64;
                let left = sample(x as i64 - o, y as i64);
                let right = sample(x as i64 + o, y as i64);
                let lanes = [
                    [left.r, right.r],
                    [left.g, right.g],
                    [left.b, right.b],
                    [left.a, right.a],
                ];
                for lane in 0..4 {
                    acc[lane] += if o == 0 {
                        lanes[lane][0] * weight
                    } else {
                        (lanes[lane][0] + lanes[lane][1]) * weight
                    };
                }
            }
            middle[(y * stride + x) as usize] = Pixel {
                r: acc[0],
                g: acc[1],
                b: acc[2],
                a: acc[3],
            };
        }
    }
    for y in 0..height {
        for x in 0..stride {
            let mut acc = [0.0f32; 4];
            for (offset, weight) in kernel_y.iter().enumerate() {
                let o = offset as i64;
                let up = middle
                    [((y as i64 - o).max(0).min(height as i64 - 1) as u32 * stride + x) as usize];
                let down = middle
                    [((y as i64 + o).max(0).min(height as i64 - 1) as u32 * stride + x) as usize];
                // Transparent black outside the logical surface.
                let up = if y as i64 - o < 0 { Pixel::CLEAR } else { up };
                let down = if y as i64 + o >= height as i64 {
                    Pixel::CLEAR
                } else {
                    down
                };
                let lanes = [
                    [up.r, down.r],
                    [up.g, down.g],
                    [up.b, down.b],
                    [up.a, down.a],
                ];
                for lane in 0..4 {
                    acc[lane] += if o == 0 {
                        lanes[lane][0] * weight
                    } else {
                        (lanes[lane][0] + lanes[lane][1]) * weight
                    };
                }
            }
            pixels[(y * stride + x) as usize] = Pixel {
                r: acc[0],
                g: acc[1],
                b: acc[2],
                a: acc[3],
            };
        }
    }
}

fn apply_drop_shadow(
    pixels: &mut [Pixel],
    stride: u32,
    shadow: &petunia_render_model::ShadowEffect,
) {
    if stride == 0 || pixels.is_empty() {
        return;
    }
    // Coverage → offset → blur → colorize, composited behind content.
    let mut layer: Vec<Pixel> = pixels
        .iter()
        .map(|pixel| Pixel {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: pixel.a,
        })
        .collect();
    let ox = shadow.offset.0.round() as i64;
    let oy = shadow.offset.1.round() as i64;
    let height = pixels.len() as u32 / stride;
    let mut shifted = vec![Pixel::CLEAR; pixels.len()];
    for y in 0..height {
        for x in 0..stride {
            let (sx, sy) = (x as i64 - ox, y as i64 - oy);
            if sx >= 0 && sy >= 0 && sx < stride as i64 && sy < height as i64 {
                shifted[(y * stride + x) as usize] =
                    layer[(sy as u32 * stride + sx as u32) as usize];
            }
        }
    }
    layer = shifted;
    separable_blur(&mut layer, stride, shadow.sigma.0, shadow.sigma.1);
    // Colorize in compositing space, then over behind the content.
    for pixel in layer.iter_mut() {
        let color = shadow.color;
        *pixel = Pixel {
            r: color.r * pixel.a,
            g: color.g * pixel.a,
            b: color.b * pixel.a,
            a: pixel.a,
        };
    }
    for (dst, shadow_pixel) in pixels.iter_mut().zip(layer.iter()) {
        *dst = composite(*shadow_pixel, *dst, BlendMode::Normal);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_core::{ObjectId, PageId, Size2};
    use petunia_render_model::{
        IsolationMode, RenderAppearance, RenderColor, RenderPage, RenderPaint, RenderPath,
        RenderSnapshot, SnapshotRevision, VectorPrimitive,
    };

    fn frame_with_square() -> RenderFrame {
        let mut path = RenderPath::new();
        path.push_contour(
            vec![(10.0, 10.0), (30.0, 10.0), (30.0, 30.0), (10.0, 30.0)],
            true,
        );
        let snapshot = RenderSnapshot {
            revision: SnapshotRevision(1),
            pages: vec![RenderPage {
                page: PageId::new_v4(),
                size: Size2::new(64.0, 64.0).expect("valid"),
                primitives: vec![RenderPrimitive::Vector(VectorPrimitive {
                    source: ObjectId::new_v4(),
                    geometry: path,
                    appearance: RenderAppearance {
                        fill: Some(RenderPaint::Solid(RenderColor {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        })),
                        stroke: None,
                        opacity: 1.0,
                    },
                    transform: petunia_core::Transform2D::IDENTITY,
                    bounds: petunia_core::Rect::new(10.0, 10.0, 20.0, 20.0),
                })],
            }],
            resources: petunia_render_model::RenderResourceTable::new(),
        };
        RenderFrame {
            snapshot,
            view: ViewTransform {
                scale: 1.0,
                offset_x: 0.0,
                offset_y: 0.0,
            },
            target: petunia_render_model::RenderTarget {
                width: 64,
                height: 64,
            },
        }
    }

    fn options() -> crate::backend::RenderOptions {
        crate::backend::RenderOptions::default()
    }

    #[test]
    fn golden_red_square_on_background() {
        let mut renderer = SoftwareRenderer::new(1 << 20, 1 << 20);
        let (bytes, stats) = renderer
            .render(&frame_with_square(), &options())
            .expect("renders");
        assert_eq!(stats.primitives_drawn, 1);
        assert_eq!(stats.tiles_processed, 1);
        assert!(stats.pixels_written > 300);
        // Center pixel is opaque red; corner keeps the background.
        let center = (20 * 64 + 20) * 4;
        assert_eq!(&bytes[center..center + 4], &[255, 0, 0, 255]);
        assert_eq!(&bytes[0..4], &[255, 255, 255, 255]);
    }

    #[test]
    fn empty_frame_renders_background_only() {
        let mut frame = frame_with_square();
        frame.snapshot.pages[0].primitives.clear();
        let mut renderer = SoftwareRenderer::new(1 << 20, 1 << 20);
        let (bytes, stats) = renderer.render(&frame, &options()).expect("renders");
        assert_eq!(stats.primitives_drawn, 0);
        assert_eq!(stats.pixels_written, 0);
        assert!(bytes
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| *pixel == [255, 255, 255, 255]));
    }

    #[test]
    fn group_opacity_scales_coverage() {
        use petunia_render_model::RenderGroup;
        let mut frame = frame_with_square();
        let only = frame.snapshot.pages[0].primitives.pop().expect("primitive");
        frame.snapshot.pages[0]
            .primitives
            .push(RenderPrimitive::Group(RenderGroup {
                source: ObjectId::new_v4(),
                children: vec![only],
                opacity: 0.5,
                blend_mode: BlendMode::Normal,
                mask: None,
                clip: None,
                effects: Vec::new(),
                isolation: IsolationMode::Flattened,
                bounds: petunia_core::Rect::new(10.0, 10.0, 20.0, 20.0),
            }));
        let mut renderer = SoftwareRenderer::new(1 << 20, 1 << 20);
        let (bytes, _) = renderer.render(&frame, &options()).expect("renders");
        let center = (20 * 64 + 20) * 4;
        // Half red over white: pinkish midpoint.
        assert!(bytes[center] > 200, "{:?}", &bytes[center..center + 4]);
        assert!(bytes[center + 1] > 100 && bytes[center + 1] < 200);
        assert_eq!(bytes[center + 3], 255);
    }

    #[test]
    fn blur_effect_spreads_group_content() {
        use petunia_render_model::{RenderEffect, RenderGroup};
        let mut frame = frame_with_square();
        let only = frame.snapshot.pages[0].primitives.pop().expect("primitive");
        frame.snapshot.pages[0]
            .primitives
            .push(RenderPrimitive::Group(RenderGroup {
                source: ObjectId::new_v4(),
                children: vec![only],
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
                mask: None,
                clip: None,
                effects: vec![RenderEffect::Blur {
                    sigma_x: 3.0,
                    sigma_y: 3.0,
                }],
                isolation: IsolationMode::Flattened,
                bounds: petunia_core::Rect::new(10.0, 10.0, 20.0, 20.0),
            }));
        let mut renderer = SoftwareRenderer::new(1 << 20, 1 << 20);
        let (bytes, _) = renderer.render(&frame, &options()).expect("renders");
        // The edge softens: composited over the opaque background the
        // pixel stays opaque but turns pinkish instead of pure red.
        let edge = (10 * 64 + 20) * 4;
        assert_eq!(bytes[edge + 3], 255);
        assert!(bytes[edge] > 200, "{:?}", &bytes[edge..edge + 4]);
        assert!(
            bytes[edge + 1] > 0 && bytes[edge + 1] < 255,
            "{:?}",
            &bytes[edge..edge + 4]
        );
    }

    #[test]
    fn golden_linear_gradient_renders_color_ramp() {
        use petunia_core::paint::{GradientInterpolation, GradientSpread, PaintSpace};
        use petunia_render_model::{RenderGradient, RenderGradientStop};

        let mut frame = frame_with_square();
        let gradient = RenderGradient {
            stops: vec![
                RenderGradientStop {
                    offset: 0.0,
                    color: RenderColor {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                    midpoint: 0.5,
                },
                RenderGradientStop {
                    offset: 1.0,
                    color: RenderColor {
                        r: 0.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    },
                    midpoint: 0.5,
                },
            ],
            interpolation: GradientInterpolation::LinearRgb,
            spread: GradientSpread::Pad,
            space: PaintSpace::Document,
            start: (10.0, 20.0),
            end: (30.0, 20.0),
            radius: 0.0,
        };
        let RenderPrimitive::Vector(vector) = &mut frame.snapshot.pages[0].primitives[0] else {
            panic!("expected vector");
        };
        vector.appearance.fill = Some(RenderPaint::LinearGradient(gradient));

        let mut renderer = SoftwareRenderer::new(1 << 20, 1 << 20);
        let (bytes, stats) = renderer.render(&frame, &options()).expect("renders");
        assert_eq!(stats.primitives_drawn, 1);

        let left_idx = (20 * 64 + 11) * 4;
        let mid_idx = (20 * 64 + 20) * 4;
        let right_idx = (20 * 64 + 29) * 4;

        // Near the start (x=11, y=20): predominantly red.
        assert!(bytes[left_idx] > 200, "red: {}", bytes[left_idx]);
        assert!(bytes[left_idx + 2] < 90, "blue: {}", bytes[left_idx + 2]);

        // Near the end (x=29, y=20): predominantly blue.
        assert!(bytes[right_idx + 2] > 200, "blue: {}", bytes[right_idx + 2]);
        assert!(bytes[right_idx] < 90, "red: {}", bytes[right_idx]);

        // Middle (x=20, y=20): smooth mixture of red and blue.
        assert!(
            bytes[mid_idx] > 80 && bytes[mid_idx] < 220,
            "mid red: {}",
            bytes[mid_idx]
        );
        assert!(
            bytes[mid_idx + 2] > 80 && bytes[mid_idx + 2] < 220,
            "mid blue: {}",
            bytes[mid_idx + 2]
        );
        assert_eq!(bytes[mid_idx + 3], 255);
    }

    #[test]
    fn golden_radial_gradient_renders_radial_ramp() {
        use petunia_core::paint::{GradientInterpolation, GradientSpread, PaintSpace};
        use petunia_render_model::{RenderGradient, RenderGradientStop};

        let mut frame = frame_with_square();
        let gradient = RenderGradient {
            stops: vec![
                RenderGradientStop {
                    offset: 0.0,
                    color: RenderColor {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                    midpoint: 0.5,
                },
                RenderGradientStop {
                    offset: 1.0,
                    color: RenderColor {
                        r: 0.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    },
                    midpoint: 0.5,
                },
            ],
            interpolation: GradientInterpolation::LinearRgb,
            spread: GradientSpread::Pad,
            space: PaintSpace::Document,
            start: (20.0, 20.0),
            end: (20.0, 20.0),
            radius: 10.0,
        };
        let RenderPrimitive::Vector(vector) = &mut frame.snapshot.pages[0].primitives[0] else {
            panic!("expected vector");
        };
        vector.appearance.fill = Some(RenderPaint::RadialGradient(gradient));

        let mut renderer = SoftwareRenderer::new(1 << 20, 1 << 20);
        let (bytes, stats) = renderer.render(&frame, &options()).expect("renders");
        assert_eq!(stats.primitives_drawn, 1);

        // Center (x=20, y=20): predominantly red.
        let center_idx = (20 * 64 + 20) * 4;
        let edge_idx = (20 * 64 + 29) * 4;

        assert!(bytes[center_idx] > 200, "center red: {}", bytes[center_idx]);
        assert!(bytes[center_idx + 2] < 90);

        // Near perimeter (x=29, y=20, r=9): predominantly blue.
        assert!(
            bytes[edge_idx + 2] > 180,
            "edge blue: {}",
            bytes[edge_idx + 2]
        );
        assert!(bytes[edge_idx] < 90, "edge red: {}", bytes[edge_idx]);
    }

    #[test]
    fn measure_tile_size_variations_render_identically() {
        let frame = frame_with_square();
        let mut renderer = SoftwareRenderer::new(1 << 20, 1 << 20);

        // Render at 32px, 64px, and 128px tile edges.
        let mut results = Vec::new();
        for &tile_size in &[32, 64, 128] {
            let mut opts = options();
            opts.tile_size = tile_size;
            let (bytes, stats) = renderer.render(&frame, &opts).expect("renders");
            results.push((tile_size, bytes, stats));
        }

        // All variations must produce identical device pixels.
        let (_, baseline_bytes, _) = &results[1]; // 64px baseline
        for (tile_size, bytes, stats) in &results {
            assert_eq!(
                bytes, baseline_bytes,
                "pixel mismatch at tile_size {tile_size}"
            );
            assert_eq!(stats.primitives_drawn, 1);
        }

        // Smaller tiles produce more tile bins for fine-grained culling.
        let (_, _, stats_32) = &results[0];
        let (_, _, stats_64) = &results[1];
        let (_, _, stats_128) = &results[2];
        assert!(stats_32.tiles_processed >= stats_64.tiles_processed);
        assert!(stats_64.tiles_processed >= stats_128.tiles_processed);
    }
}
