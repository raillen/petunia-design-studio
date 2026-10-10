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
use crate::paint_eval::{sample_conical, sample_linear, sample_radial};
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
    frame_budget_bytes: usize,
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
            frame_budget_bytes: 256 << 20,
        }
    }

    /// Limit full-frame working allocations independently of cache retention.
    #[must_use]
    pub fn with_frame_budget(mut self, bytes: usize) -> Self {
        self.frame_budget_bytes = bytes;
        self
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
        let count = (target.width as usize)
            .checked_mul(target.height as usize)
            .ok_or_else(|| RenderError::Draw("render target size overflow".into()))?;
        if !frame.view.scale.is_finite()
            || frame.view.scale <= 0.0
            || !frame.view.rotation.is_finite()
            || !frame.view.offset_x.is_finite()
            || !frame.view.offset_y.is_finite()
        {
            return Err(RenderError::Draw("invalid view transform".into()));
        }
        let depth = validate_frame(frame)?;
        let required = count
            .checked_mul(std::mem::size_of::<Pixel>())
            .and_then(|bytes| bytes.checked_mul(depth + 4))
            .and_then(|bytes| bytes.checked_add(count.checked_mul(4)?))
            .ok_or_else(|| RenderError::Draw("working allocation size overflow".into()))?;
        if required > self.frame_budget_bytes {
            return Err(RenderError::Draw(
                "frame working allocation budget exceeded".into(),
            ));
        }
        let tile_edge = options.tile_size.clamp(16, 512);
        let background = srgb_to_linear_pixel(options.background);
        let items = prepare_items(frame);
        let mut pixels = vec![background; count];
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

fn validate_frame(frame: &RenderFrame) -> Result<usize> {
    let mut stack: Vec<_> = frame
        .snapshot
        .pages
        .iter()
        .flat_map(|page| page.primitives.iter().map(|primitive| (primitive, 1usize)))
        .collect();
    let mut maximum = 1;
    let mut count = 0usize;
    while let Some((primitive, depth)) = stack.pop() {
        count += 1;
        if depth > 64 || count > 100_000 {
            return Err(RenderError::Draw(
                "render graph depth/count limit exceeded".into(),
            ));
        }
        maximum = maximum.max(depth);
        if matches!(
            primitive,
            RenderPrimitive::Text(_) | RenderPrimitive::Raster(_)
        ) {
            return Err(RenderError::Draw(
                "text/raster primitives must be materialized into paths/images".into(),
            ));
        }
        if let RenderPrimitive::Group(group) = primitive {
            if matches!(
                group.mask,
                Some(RenderMask::Alpha(_) | RenderMask::Luminance(_))
            ) {
                return Err(RenderError::Draw(
                    "mask source must be resolved into snapshot primitives".into(),
                ));
            }
            for effect in &group.effects {
                if let RenderEffect::DropShadow(shadow) = effect {
                    if shadow.spread != 0.0
                        || !shadow.opacity.is_finite()
                        || !(0.0..=1.0).contains(&shadow.opacity)
                        || !shadow.offset.0.is_finite()
                        || !shadow.offset.1.is_finite()
                    {
                        return Err(RenderError::Draw(
                            "unsupported shadow spread or invalid shadow parameters".into(),
                        ));
                    }
                }
                let sigmas = match effect {
                    RenderEffect::Blur { sigma_x, sigma_y } => (*sigma_x, *sigma_y),
                    RenderEffect::DropShadow(shadow) => shadow.sigma,
                    _ => (0.0, 0.0),
                };
                if !sigmas.0.is_finite()
                    || !sigmas.1.is_finite()
                    || sigmas.0 < 0.0
                    || sigmas.1 < 0.0
                {
                    return Err(RenderError::Draw("invalid effect sigma".into()));
                }
                let radius = match effect {
                    RenderEffect::Blur {
                        sigma_x, sigma_y, ..
                    } => sigma_x.max(*sigma_y),
                    RenderEffect::DropShadow(shadow) => shadow.sigma.0.max(shadow.sigma.1),
                    _ => 0.0,
                };
                let radius = radius * frame.view.scale;
                if !radius.is_finite() || !(0.0..=1024.0).contains(&radius) {
                    return Err(RenderError::Draw("effect kernel limit exceeded".into()));
                }
            }
            stack.extend(
                group
                    .children
                    .iter()
                    .map(|primitive| (primitive, depth + 1)),
            );
            if let Some(RenderMask::Primitives { children, .. }) = &group.mask {
                stack.extend(children.iter().map(|primitive| (primitive, depth + 2)));
            }
        }
    }
    for image in frame.snapshot.resources.images.values() {
        if image.width == 0
            || image.height == 0
            || (image.width as usize).checked_mul(image.height as usize) != Some(image.pixels.len())
        {
            return Err(RenderError::Draw("invalid image extent/buffer".into()));
        }
    }
    Ok(maximum)
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
    Conical(RenderGradient),
    Pattern(
        petunia_render_model::RenderPattern,
        Option<std::sync::Arc<petunia_render_model::image::ResolvedImage>>,
    ),
}

impl PaintSampler {
    fn sample(&self, doc_x: f64, doc_y: f64) -> petunia_render_model::RenderColor {
        match self {
            Self::Pattern(pattern, image) => {
                sample_pattern(pattern, image.as_deref(), doc_x, doc_y)
            }
            Self::Solid(color) => *color,
            Self::Linear(gradient) => sample_linear(gradient, (doc_x, doc_y))
                .unwrap_or(petunia_render_model::RenderColor::TRANSPARENT),
            Self::Radial(gradient) => sample_radial(gradient, (doc_x, doc_y))
                .unwrap_or(petunia_render_model::RenderColor::TRANSPARENT),
            Self::Conical(gradient) => sample_conical(gradient, (doc_x, doc_y))
                .unwrap_or(petunia_render_model::RenderColor::TRANSPARENT),
        }
    }
}

fn sampler_for(
    paint: &RenderPaint,
    resources: &petunia_render_model::RenderResourceTable,
) -> PaintSampler {
    match paint {
        RenderPaint::Solid(color) => PaintSampler::Solid(*color),
        RenderPaint::LinearGradient(gradient) => PaintSampler::Linear(gradient.clone()),
        RenderPaint::RadialGradient(gradient) => PaintSampler::Radial(gradient.clone()),
        RenderPaint::ConicalGradient(gradient) => PaintSampler::Conical(gradient.clone()),
        RenderPaint::Pattern(pattern) => PaintSampler::Pattern(
            pattern.clone(),
            resources.images.get(&pattern.resource).cloned(),
        ),
    }
}

fn sample_image(
    image: &petunia_render_model::image::ResolvedImage,
    x: f64,
    y: f64,
    policy: petunia_core::ImageSamplingPolicy,
) -> petunia_render_model::RenderColor {
    let at = |x: i64, y: i64| {
        image.pixels[(y.clamp(0, image.height as i64 - 1) as usize) * image.width as usize
            + x.clamp(0, image.width as i64 - 1) as usize]
    };
    if policy == petunia_core::ImageSamplingPolicy::Nearest {
        return at(x.floor() as i64, y.floor() as i64);
    }
    let px = x - 0.5;
    let py = y - 0.5;
    let ix = px.floor() as i64;
    let iy = py.floor() as i64;
    if policy == petunia_core::ImageSamplingPolicy::Bicubic {
        let weight = |distance: f64| {
            let t = distance.abs();
            if t < 1.0 {
                1.5 * t.powi(3) - 2.5 * t.powi(2) + 1.0
            } else if t < 2.0 {
                -0.5 * t.powi(3) + 2.5 * t.powi(2) - 4.0 * t + 2.0
            } else {
                0.0
            }
        };
        let mut color = petunia_render_model::RenderColor::TRANSPARENT;
        for dy in -1..=2 {
            for dx in -1..=2 {
                let sample = at(ix + dx, iy + dy);
                let coefficient =
                    (weight(px - (ix + dx) as f64) * weight(py - (iy + dy) as f64)) as f32;
                let alpha = sample.a * coefficient;
                color.r += sample.r * alpha;
                color.g += sample.g * alpha;
                color.b += sample.b * alpha;
                color.a += alpha;
            }
        }
        if color.a > 1e-6 {
            color.r /= color.a;
            color.g /= color.a;
            color.b /= color.a;
            color.a = color.a.clamp(0.0, 1.0);
        } else {
            color = petunia_render_model::RenderColor::TRANSPARENT;
        }
        return color;
    }
    let fx = (px - ix as f64) as f32;
    let fy = (py - iy as f64) as f32;
    let weights = [
        (at(ix, iy), (1.0 - fx) * (1.0 - fy)),
        (at(ix + 1, iy), fx * (1.0 - fy)),
        (at(ix, iy + 1), (1.0 - fx) * fy),
        (at(ix + 1, iy + 1), fx * fy),
    ];
    let mut color = petunia_render_model::RenderColor::TRANSPARENT;
    for (sample, weight) in weights {
        let alpha = sample.a * weight;
        color.r += sample.r * alpha;
        color.g += sample.g * alpha;
        color.b += sample.b * alpha;
        color.a += alpha;
    }
    if color.a > 0.0 {
        color.r /= color.a;
        color.g /= color.a;
        color.b /= color.a;
    }
    color
}
fn sample_pattern(
    pattern: &petunia_render_model::RenderPattern,
    image: Option<&petunia_render_model::image::ResolvedImage>,
    x: f64,
    y: f64,
) -> petunia_render_model::RenderColor {
    let Some(image) = image else {
        return petunia_render_model::RenderColor::TRANSPARENT;
    };
    let Some(inverse) = pattern.transform.inverse() else {
        return petunia_render_model::RenderColor::TRANSPARENT;
    };
    let point = inverse.transform_point(petunia_core::Point::new(x, y));
    let repeat = |value: f64, extent: u32, policy: petunia_core::PatternRepeat| {
        let size = extent as f64;
        match policy {
            petunia_core::PatternRepeat::Repeat => value.rem_euclid(size),
            petunia_core::PatternRepeat::Mirror => {
                let value = value.rem_euclid(2.0 * size);
                if value >= size {
                    2.0 * size - value
                } else {
                    value
                }
            }
            petunia_core::PatternRepeat::Clamp => value.clamp(0.5, size - 0.5),
        }
    };
    sample_image(
        image,
        repeat(point.x, image.width, pattern.repeat_x),
        repeat(point.y, image.height, pattern.repeat_y),
        petunia_core::ImageSamplingPolicy::Bilinear,
    )
}
fn draw_image(
    pixels: &mut [Pixel],
    stride: u32,
    primitive: &petunia_render_model::ImagePrimitive,
    image: &petunia_render_model::image::ResolvedImage,
    opacity: f32,
    view: ViewTransform,
) -> Result<()> {
    if primitive.sampling == petunia_core::ImageSamplingPolicy::Bicubic {
        return Err(RenderError::Draw(
            "bicubic image sampling is not implemented".into(),
        ));
    }
    let inverse = primitive
        .transform
        .inverse()
        .ok_or_else(|| RenderError::Draw("image transform is singular".into()))?;
    if view.scale == 0.0 {
        return Err(RenderError::Draw("view scale is zero".into()));
    }
    let crop = primitive.source_rect;
    let (min_x, min_y, max_x, max_y) = crop.map_or(
        (0.0, 0.0, image.width as f64, image.height as f64),
        |crop| {
            (
                crop.min.x * image.width as f64,
                crop.min.y * image.height as f64,
                crop.max.x * image.width as f64,
                crop.max.y * image.height as f64,
            )
        },
    );
    for (index, pixel) in pixels.iter_mut().enumerate() {
        let x = index as u32 % stride;
        let y = index as u32 / stride;
        let (doc_x, doc_y) = view
            .inverse_apply(x as f64 + 0.5, y as f64 + 0.5)
            .ok_or_else(|| RenderError::Draw("invalid view transform".into()))?;
        let point = inverse.transform_point(petunia_core::Point::new(doc_x, doc_y));
        if point.x < min_x || point.y < min_y || point.x >= max_x || point.y >= max_y {
            continue;
        }
        let sample = sample_image(image, point.x, point.y, primitive.sampling);
        let alpha = sample.a * opacity * primitive.opacity;
        *pixel = composite(
            Pixel {
                r: sample.r * alpha,
                g: sample.g * alpha,
                b: sample.b * alpha,
                a: alpha,
            },
            *pixel,
            BlendMode::Normal,
        );
    }
    Ok(())
}

/// Device-space vector content with resolved paint.
struct PreparedVector {
    rings: Vec<(Vec<(f64, f64)>, bool)>,
    fill_rule: FillRule,
    fill: Option<PaintSampler>,
    stroke: Option<(PaintSampler, f64)>,
    opacity: f32,
    blend: BlendMode,
}

/// One frame entry: flat draws rasterize in place; isolated groups
/// composite through a temporary with effects, clip and group blend.
enum FrameItem {
    Draw(PreparedVector),
    Image {
        primitive: petunia_render_model::ImagePrimitive,
        image: Option<std::sync::Arc<petunia_render_model::image::ResolvedImage>>,
        opacity: f32,
    },
    Isolated {
        children: Vec<FrameItem>,
        opacity: f32,
        blend: BlendMode,
        effects: Vec<RenderEffect>,
        clip: Option<RenderClip>,
        mask: Option<(Vec<FrameItem>, bool)>,
    },
}

fn prepare_items(frame: &RenderFrame) -> Vec<FrameItem> {
    let mut out = Vec::new();
    for page in &frame.snapshot.pages {
        for primitive in &page.primitives {
            collect_primitive(
                primitive,
                frame.view,
                1.0,
                BlendMode::Normal,
                &frame.snapshot.resources,
                &mut out,
            );
        }
    }
    out
}

fn collect_primitive(
    primitive: &RenderPrimitive,
    view: ViewTransform,
    opacity: f32,
    blend: BlendMode,
    resources: &petunia_render_model::RenderResourceTable,
    out: &mut Vec<FrameItem>,
) {
    match primitive {
        RenderPrimitive::Vector(vector) => {
            let mut prepared = prepared_vector(
                &vector.geometry,
                &vector.appearance,
                &vector.transform,
                view,
                opacity,
                blend,
                resources,
            );
            prepared.fill_rule = vector.geometry.fill_rule;
            out.push(FrameItem::Draw(prepared));
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
                    collect_primitive(child, view, opacity * group.opacity, blend, resources, out);
                }
            } else {
                let mut children = Vec::new();
                for child in &group.children {
                    collect_primitive(
                        child,
                        view,
                        1.0,
                        BlendMode::Normal,
                        resources,
                        &mut children,
                    );
                }
                out.push(FrameItem::Isolated {
                    children,
                    opacity: opacity * group.opacity,
                    blend: group.blend_mode,
                    effects: group.effects.clone(),
                    clip: group.clip.clone(),
                    mask: group.mask.as_ref().map(|mask| {
                        let (primitives, luminance) = match mask {
                            RenderMask::Primitives {
                                children,
                                luminance,
                            } => (children.as_slice(), *luminance),
                            _ => (&[][..], false),
                        };
                        let mut items = Vec::new();
                        for primitive in primitives {
                            collect_primitive(
                                primitive,
                                view,
                                1.0,
                                BlendMode::Normal,
                                resources,
                                &mut items,
                            );
                        }
                        (items, luminance)
                    }),
                });
            }
        }
        // Text, images and raster surfaces need decoded resources or
        // shaped runs the compiler only emits with providers present;
        // the v0.1 compiler degrades them with warnings instead.
        RenderPrimitive::Image(image) => out.push(FrameItem::Image {
            primitive: image.clone(),
            image: resources.images.get(&image.resource).cloned(),
            opacity,
        }),
        RenderPrimitive::Text(_) | RenderPrimitive::Raster(_) => {}
    }
}

fn prepared_vector(
    geometry: &petunia_render_model::RenderPath,
    appearance: &RenderAppearance,
    transform: &petunia_core::Transform2D,
    view: ViewTransform,
    opacity: f32,
    blend: BlendMode,
    resources: &petunia_render_model::RenderResourceTable,
) -> PreparedVector {
    let rings = geometry
        .contours
        .iter()
        .zip(geometry.closed.iter())
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
        fill_rule: FillRule::NonZero,
        fill: appearance
            .fill
            .as_ref()
            .map(|paint| sampler_for(paint, resources)),
        stroke: appearance.stroke.as_ref().map(|stroke| {
            (
                sampler_for(&stroke.paint, resources),
                (stroke.width * view.scale).max(0.0),
            )
        }),
        opacity: opacity * appearance.opacity,
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
        FrameItem::Image {
            primitive,
            image,
            opacity,
        } => {
            let image = image
                .as_deref()
                .ok_or_else(|| RenderError::Draw("missing decoded image resource".into()))?;
            draw_image(pixels, stride, primitive, image, *opacity, view)?;
            Ok(1)
        }
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
            // Full-frame temporary: effects stay seam-free.
            let height = pixels.len() as u32 / stride.max(1);
            let mut temp = vec![Pixel::CLEAR; pixels.len()];
            let mut drawn = 0usize;
            for child in children {
                drawn += draw_item(&mut temp, stride, child, view, tile_edge)?;
            }
            apply_effects(&mut temp, stride, effects, view)?;
            if let Some((items, luminance)) = mask {
                let mut coverage = vec![Pixel::CLEAR; pixels.len()];
                for item in items {
                    draw_item(&mut coverage, stride, item, view, tile_edge)?;
                }
                for (pixel, mask) in temp.iter_mut().zip(coverage) {
                    let alpha = if *luminance {
                        0.2126 * mask.r + 0.7152 * mask.g + 0.0722 * mask.b
                    } else {
                        mask.a
                    };
                    pixel.r *= alpha;
                    pixel.g *= alpha;
                    pixel.b *= alpha;
                    pixel.a *= alpha;
                }
            }
            let clip = clip.as_ref().map(|clip| device_clip(clip, view));
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
        view.inverse_apply(x + origin.0, y + origin.1)
            .unwrap_or((0.0, 0.0))
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
            vector.fill_rule,
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

fn device_clip(clip: &RenderClip, view: ViewTransform) -> RenderClip {
    match clip {
        RenderClip::Rect(rect) => RenderClip::Polygon(
            [
                (rect.x, rect.y),
                (rect.x + rect.width, rect.y),
                (rect.x + rect.width, rect.y + rect.height),
                (rect.x, rect.y + rect.height),
            ]
            .into_iter()
            .map(|(x, y)| view.apply(x, y))
            .collect(),
        ),
        RenderClip::Polygon(points) => {
            RenderClip::Polygon(points.iter().map(|(x, y)| view.apply(*x, *y)).collect())
        }
        RenderClip::Path(path) => {
            let mut path = path.clone();
            for contour in &mut path.contours {
                for point in contour {
                    *point = view.apply(point.0, point.1);
                }
            }
            RenderClip::Path(path)
        }
    }
}

fn clip_contains(clip: &RenderClip, x: f64, y: f64) -> bool {
    match clip {
        RenderClip::Rect(rect) => {
            x >= rect.x && x <= rect.x + rect.width && y >= rect.y && y <= rect.y + rect.height
        }
        RenderClip::Polygon(points) => point_in_poly(x, y, points),
        RenderClip::Path(path) => {
            let winding: i32 = path
                .contours
                .iter()
                .zip(&path.closed)
                .filter(|(_, closed)| **closed)
                .map(|(ring, _)| crate::rasterize::winding_number(x, y, ring))
                .sum();
            match path.fill_rule {
                FillRule::NonZero => winding != 0,
                FillRule::EvenOdd => winding % 2 != 0,
            }
        }
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
fn apply_effects(
    pixels: &mut [Pixel],
    stride: u32,
    effects: &[RenderEffect],
    view: ViewTransform,
) -> Result<()> {
    for effect in effects {
        match effect {
            RenderEffect::Blur { sigma_x, sigma_y } => {
                separable_blur(
                    pixels,
                    stride,
                    *sigma_x * view.scale,
                    *sigma_y * view.scale,
                    view.rotation,
                );
            }
            RenderEffect::DropShadow(shadow) => {
                apply_drop_shadow(pixels, stride, shadow, view);
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

fn sample_pixel(source: &[Pixel], stride: u32, x: f64, y: f64) -> Pixel {
    let height = source.len() / stride as usize;
    if !x.is_finite()
        || !y.is_finite()
        || x < -1.0
        || y < -1.0
        || x >= stride as f64
        || y >= height as f64
    {
        return Pixel::CLEAR;
    }
    let at = |x: i64, y: i64| {
        if x < 0 || y < 0 || x >= stride as i64 || y >= height as i64 {
            Pixel::CLEAR
        } else {
            source[y as usize * stride as usize + x as usize]
        }
    };
    if (x - x.round()).abs() < 1e-12 && (y - y.round()).abs() < 1e-12 {
        return at(x.round() as i64, y.round() as i64);
    }
    let ix = x.floor() as i64;
    let iy = y.floor() as i64;
    let fx = (x - ix as f64) as f32;
    let fy = (y - iy as f64) as f32;
    let mut result = Pixel::CLEAR;
    for (pixel, weight) in [
        (at(ix, iy), (1. - fx) * (1. - fy)),
        (at(ix + 1, iy), fx * (1. - fy)),
        (at(ix, iy + 1), (1. - fx) * fy),
        (at(ix + 1, iy + 1), fx * fy),
    ] {
        result.r += pixel.r * weight;
        result.g += pixel.g * weight;
        result.b += pixel.b * weight;
        result.a += pixel.a * weight;
    }
    result
}

fn separable_blur(pixels: &mut [Pixel], stride: u32, sigma_x: f64, sigma_y: f64, rotation: f64) {
    if stride == 0 || pixels.is_empty() || (sigma_x == 0.0 && sigma_y == 0.0) {
        return;
    }
    let height = pixels.len() as u32 / stride;
    let (sin, cos) = rotation.sin_cos();
    let source = pixels.to_vec();
    let mut middle = source.clone();
    blur_axis(&source, &mut middle, stride, height, sigma_x, (cos, sin));
    blur_axis(&middle, pixels, stride, height, sigma_y, (-sin, cos));
}

fn blur_axis(
    source: &[Pixel],
    output: &mut [Pixel],
    stride: u32,
    height: u32,
    sigma: f64,
    axis: (f64, f64),
) {
    let kernel = blur_kernel_1d(sigma);
    for y in 0..height {
        for x in 0..stride {
            let mut value = Pixel::CLEAR;
            for (offset, weight) in kernel.iter().enumerate() {
                let dx = offset as f64 * axis.0;
                let dy = offset as f64 * axis.1;
                let left = sample_pixel(source, stride, x as f64 - dx, y as f64 - dy);
                let right = if offset == 0 {
                    Pixel::CLEAR
                } else {
                    sample_pixel(source, stride, x as f64 + dx, y as f64 + dy)
                };
                value.r += (left.r + right.r) * weight;
                value.g += (left.g + right.g) * weight;
                value.b += (left.b + right.b) * weight;
                value.a += (left.a + right.a) * weight;
            }
            output[(y * stride + x) as usize] = value;
        }
    }
}

fn apply_drop_shadow(
    pixels: &mut [Pixel],
    stride: u32,
    shadow: &petunia_render_model::ShadowEffect,
    view: ViewTransform,
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
    let (sin, cos) = view.rotation.sin_cos();
    let ox = (shadow.offset.0 * cos - shadow.offset.1 * sin) * view.scale;
    let oy = (shadow.offset.0 * sin + shadow.offset.1 * cos) * view.scale;
    let height = pixels.len() as u32 / stride;
    let mut shifted = vec![Pixel::CLEAR; pixels.len()];
    for y in 0..height {
        for x in 0..stride {
            shifted[(y * stride + x) as usize] =
                sample_pixel(&layer, stride, x as f64 - ox, y as f64 - oy);
        }
    }
    layer = shifted;
    separable_blur(
        &mut layer,
        stride,
        shadow.sigma.0 * view.scale,
        shadow.sigma.1 * view.scale,
        view.rotation,
    );
    // Colorize in compositing space, then over behind the content.
    for pixel in layer.iter_mut() {
        let color = shadow.color;
        let alpha = pixel.a * color.a * shadow.opacity;
        *pixel = Pixel {
            r: color.r * alpha,
            g: color.g * alpha,
            b: color.b * alpha,
            a: alpha,
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
                primitives: vec![RenderPrimitive::Vector(Box::new(VectorPrimitive {
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
                }))],
            }],
            resources: petunia_render_model::RenderResourceTable::new(),
        };
        RenderFrame {
            snapshot,
            view: ViewTransform {
                rotation: 0.0,
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
            transform: petunia_core::Transform2D::IDENTITY,
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
            start_angle: 0.0,
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
            transform: petunia_core::Transform2D::IDENTITY,
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
            start_angle: 0.0,
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
