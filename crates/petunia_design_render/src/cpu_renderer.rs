//! Headless antialiased vector backend, premultiplied internally, straight RGBA
//! at the API boundary. Isolation follows Porter–Duff/W3C composition.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Arc;

use petunia_design_raster::{
    EncodedImage, ImageAssetError, ImageCache, ImageContentKey, PreparedImage,
};

use petunia_design_document::{
    ContainerRole, EffectKind, GradientStop, MaskMode, Paint, ShapeKind, StrokeAlignment,
    StrokeCap, StrokeJoin,
};
use petunia_design_foundation::ObjectId;
use petunia_design_geometry::{GAffine, GPath, GPoint, GRect, PathVerb};
use petunia_design_jobs::CancellationToken;
use tiny_skia as sk;

use crate::render_scene::expand;
use crate::{PixelBufferRgba8, RenderError, RenderNode, RenderSurface};

/// Caller-owned synchronous rendering quotas. Worker scheduling is independent.
#[derive(Clone, Copy, Debug)]
pub struct RenderLimits {
    /// Maximum output pixel count.
    pub max_output_pixels: u64,
    /// Aggregate live intermediate/scratch bytes (including the output pixmap).
    pub max_working_bytes: usize,
    /// Maximum Gaussian standard deviation in device pixels.
    pub max_blur_sigma: f64,
    /// Maximum recursive composition depth, including mask dependencies.
    pub max_depth: usize,
}

impl Default for RenderLimits {
    fn default() -> Self {
        Self {
            max_output_pixels: 16_777_216,
            max_working_bytes: 256 * 1024 * 1024,
            max_blur_sigma: 128.0,
            max_depth: 128,
        }
    }
}

/// Document region mapped directly to output dimensions; supports negative origins.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderRequest {
    /// Pasteboard region in points, independent of the window camera.
    pub viewport: GRect,
    /// Output width in pixels.
    pub width: u32,
    /// Output height in pixels.
    pub height: u32,
    /// Straight RGBA background.
    pub background: [u8; 4],
}

impl RenderRequest {
    /// Maps a surface's region to pixels at a requested DPI (72 points/inch).
    pub fn for_surface(surface: &RenderSurface, dpi: f64) -> Result<Self, RenderError> {
        let [x, y, w, h] = surface.bounds();
        if ![x, y, w, h, dpi].iter().all(|v| v.is_finite()) || w <= 0.0 || h <= 0.0 || dpi <= 0.0 {
            return Err(RenderError::Invalid(
                "surface dimensions and DPI must be finite and positive".into(),
            ));
        }
        let pw = (w * dpi / 72.0).ceil();
        let ph = (h * dpi / 72.0).ceil();
        if pw > f64::from(u32::MAX) || ph > f64::from(u32::MAX) {
            return Err(RenderError::Limit("surface output dimensions"));
        }
        Ok(Self {
            viewport: GRect::new(x, y, x + w, y + h),
            width: (pw as u32).max(1),
            height: (ph as u32).max(1),
            background: [0; 4],
        })
    }
}

/// CPU backend with shared immutable image derivatives. Content without a faithful adapter returns a reason.
#[derive(Debug)]
pub struct CpuRenderer {
    limits: RenderLimits,
    images: Arc<ImageCache>,
}

impl Default for CpuRenderer {
    fn default() -> Self {
        Self::new(RenderLimits::default())
    }
}
impl CpuRenderer {
    /// Creates a backend with caller-supplied limits.
    pub fn new(limits: RenderLimits) -> Self {
        Self {
            limits,
            images: ImageCache::shared(),
        }
    }

    /// Supplies an isolated or application-owned cache with explicit quotas.
    pub fn with_image_cache(limits: RenderLimits, images: Arc<ImageCache>) -> Self {
        Self { limits, images }
    }

    /// Renders one immutable snapshot, never accessing a GUI or live document.
    pub fn render(
        &self,
        surface: &RenderSurface,
        request: RenderRequest,
    ) -> Result<PixelBufferRgba8, RenderError> {
        self.render_over(surface, request, None)
    }

    /// Renders directly over a caller-supplied straight RGBA backdrop. This
    /// preserves blend modes when incrementally repainting a dirty rectangle.
    pub fn render_over(
        &self,
        surface: &RenderSurface,
        request: RenderRequest,
        backdrop: Option<&PixelBufferRgba8>,
    ) -> Result<PixelBufferRgba8, RenderError> {
        self.render_internal(surface, request, backdrop, None)
    }

    /// Checks cancellation between objects, mask rows and convolution rows.
    /// The caller receives a complete image or an error, never partial pixels.
    pub fn render_cancellable(
        &self,
        surface: &RenderSurface,
        request: RenderRequest,
        cancellation: &CancellationToken,
    ) -> Result<PixelBufferRgba8, RenderError> {
        self.render_internal(surface, request, None, Some(cancellation))
    }

    /// Composes against the preview backdrop with cooperative cancellation.
    pub fn render_over_cancellable(
        &self,
        surface: &RenderSurface,
        request: RenderRequest,
        backdrop: &PixelBufferRgba8,
        cancellation: &CancellationToken,
    ) -> Result<PixelBufferRgba8, RenderError> {
        self.render_internal(surface, request, Some(backdrop), Some(cancellation))
    }

    fn render_internal(
        &self,
        surface: &RenderSurface,
        request: RenderRequest,
        backdrop: Option<&PixelBufferRgba8>,
        cancellation: Option<&CancellationToken>,
    ) -> Result<PixelBufferRgba8, RenderError> {
        let v = request.viewport;
        if !v.is_finite()
            || v.width() <= 0.0
            || v.height() <= 0.0
            || request.width == 0
            || request.height == 0
        {
            return Err(RenderError::Invalid(
                "invalid render viewport/dimensions".into(),
            ));
        }
        if u64::from(request.width) * u64::from(request.height) > self.limits.max_output_pixels {
            return Err(RenderError::Limit("output pixel count"));
        }
        if !self.limits.max_blur_sigma.is_finite()
            || self.limits.max_blur_sigma < 0.0
            || self.limits.max_depth == 0
        {
            return Err(RenderError::Invalid("invalid renderer limits".into()));
        }
        let sx = f64::from(request.width) / v.width();
        let sy = f64::from(request.height) / v.height();
        let context = Context {
            surface,
            limits: self.limits,
            live_bytes: Cell::new(0),
            world_to_device: GAffine::scale(sx, sy).after(GAffine::translate(-v.x0, -v.y0)),
            scale: sx.max(sy),
            cancellation,
            images: &self.images,
            prepared: RefCell::new(HashMap::new()),
        };
        context.check_cancelled()?;
        // Capability preflight covers visible content, even if it is outside the
        // requested crop. Strict export never silently loses unsupported artwork.
        for id in surface.roots() {
            context.preflight(*id, 0)?;
        }
        let mut output = context.allocate(request.width, request.height, 0, 0)?;
        let [r, g, b, a] = request.background;
        output.pixmap.fill(sk::Color::from_rgba8(r, g, b, a));
        if let Some(backdrop) = backdrop {
            if backdrop.width != request.width
                || backdrop.height != request.height
                || backdrop.data.len() != output.pixmap.data().len()
            {
                return Err(RenderError::Invalid(
                    "backdrop dimensions/data do not match".into(),
                ));
            }
            for (source, target) in backdrop
                .data
                .chunks_exact(4)
                .zip(output.pixmap.data_mut().chunks_exact_mut(4))
            {
                for c in 0..3 {
                    target[c] = ((u16::from(source[c]) * u16::from(source[3]) + 127) / 255) as u8;
                }
                target[3] = source[3];
            }
        }
        let window = GRect::new(
            0.0,
            0.0,
            f64::from(request.width),
            f64::from(request.height),
        );
        for id in surface.roots() {
            if let Some(layer) = context.render_node(*id, window, 0, false)? {
                composite(
                    &mut output,
                    &layer,
                    map_blend(surface.nodes[id].source.effective_appearance().blend_mode),
                );
            }
        }
        // Keep the returned allocation inside the same peak-memory budget.
        let _straight_permit = context.reserve(output.pixmap.data().len())?;
        let mut data = Vec::new();
        data.try_reserve_exact(output.pixmap.data().len())
            .map_err(|_| RenderError::Limit("output allocation"))?;
        for (index, pixel) in output.pixmap.pixels().iter().enumerate() {
            if index % request.width as usize == 0 {
                context.check_cancelled()?;
            }
            let p = pixel.demultiply();
            data.extend_from_slice(&[p.red(), p.green(), p.blue(), p.alpha()]);
        }
        Ok(PixelBufferRgba8 {
            width: request.width,
            height: request.height,
            data,
        })
    }
}

struct Permit<'a> {
    live: &'a Cell<usize>,
    bytes: usize,
}
impl Drop for Permit<'_> {
    fn drop(&mut self) {
        self.live.set(self.live.get() - self.bytes);
    }
}
struct Layer<'a> {
    pixmap: sk::Pixmap,
    x: i32,
    y: i32,
    _permit: Permit<'a>,
}
struct Context<'a> {
    surface: &'a RenderSurface,
    limits: RenderLimits,
    live_bytes: Cell<usize>,
    world_to_device: GAffine,
    scale: f64,
    cancellation: Option<&'a CancellationToken>,
    images: &'a ImageCache,
    // Pin each admitted resource once through the complete render. Active
    // snapshots cannot evade cache residency limits by evicting their entries.
    prepared: RefCell<HashMap<ImageContentKey, Arc<PreparedImage>>>,
}

impl Context<'_> {
    fn check_cancelled(&self) -> Result<(), RenderError> {
        if self
            .cancellation
            .is_some_and(CancellationToken::is_cancelled)
        {
            return Err(RenderError::Cancelled);
        }
        Ok(())
    }
    fn reserve(&self, bytes: usize) -> Result<Permit<'_>, RenderError> {
        let next = self
            .live_bytes
            .get()
            .checked_add(bytes)
            .ok_or(RenderError::Limit("working byte overflow"))?;
        if next > self.limits.max_working_bytes {
            return Err(RenderError::Limit("working byte budget"));
        }
        self.live_bytes.set(next);
        Ok(Permit {
            live: &self.live_bytes,
            bytes,
        })
    }

    fn allocate(&self, width: u32, height: u32, x: i32, y: i32) -> Result<Layer<'_>, RenderError> {
        let bytes = (width as usize)
            .checked_mul(height as usize)
            .and_then(|v| v.checked_mul(4))
            .ok_or(RenderError::Limit("intermediate dimensions"))?;
        let permit = self.reserve(bytes)?;
        let mut data = Vec::new();
        data.try_reserve_exact(bytes)
            .map_err(|_| RenderError::Limit("intermediate allocation"))?;
        data.resize(bytes, 0);
        let size = sk::IntSize::from_wh(width, height)
            .ok_or(RenderError::Limit("intermediate dimensions"))?;
        let pixmap = sk::Pixmap::from_vec(data, size)
            .ok_or(RenderError::Limit("intermediate allocation"))?;
        Ok(Layer {
            pixmap,
            x,
            y,
            _permit: permit,
        })
    }

    fn node(&self, id: ObjectId) -> Result<&RenderNode, RenderError> {
        self.surface
            .nodes
            .get(&id)
            .ok_or_else(|| RenderError::Invalid(format!("missing render dependency {id}")))
    }

    fn preflight(&self, id: ObjectId, depth: usize) -> Result<(), RenderError> {
        self.check_cancelled()?;
        if depth >= self.limits.max_depth {
            return Err(RenderError::Limit("composition depth"));
        }
        let node = self.node(id)?;
        if !node.source.visible {
            return Ok(());
        }
        let unsupported = |feature| RenderError::Unsupported {
            object: id,
            feature,
        };
        if node.source.is_container()
            && node.source.modifiers.iter().any(|m| {
                m.enabled
                    && !matches!(
                        m.kind,
                        petunia_design_document::ModifierKind::TransparentGradient { .. }
                    )
            })
        {
            return Err(unsupported("geometry modifiers on composed groups"));
        }
        match &node.source.shape {
            Some(ShapeKind::Raster { layer }) => {
                layer
                    .validate()
                    .map_err(|e| RenderError::Invalid(e.to_string()))?;
                if node.source.modifiers.iter().any(|m| {
                    m.enabled
                        && !matches!(
                            m.kind,
                            petunia_design_document::ModifierKind::CropRect { .. }
                                | petunia_design_document::ModifierKind::TransparentGradient { .. }
                        )
                }) {
                    return Err(unsupported("raster perspective/contour sampling"));
                }
            }
            Some(ShapeKind::Image { data, .. }) => {
                if node.source.modifiers.iter().any(|m| {
                    m.enabled
                        && !matches!(
                            m.kind,
                            petunia_design_document::ModifierKind::CropRect { .. }
                                | petunia_design_document::ModifierKind::TransparentGradient { .. }
                        )
                }) {
                    return Err(unsupported("image perspective/contour sampling"));
                }
                let source = data
                    .as_deref()
                    .ok_or_else(|| unsupported("embedded image source"))?;
                self.prepare_image(id, source)?;
            }
            _ => {}
        }
        let app = node.source.effective_appearance();
        if !app.opacity.is_finite() || !(0.0..=1.0).contains(&app.opacity) {
            return Err(RenderError::Invalid(format!("invalid opacity of {id}")));
        }
        for effect in app.effects.iter().filter(|e| e.visible) {
            match effect.kind {
                EffectKind::GaussianBlur { radius } => self.check_sigma(radius * self.scale)?,
                EffectKind::DropShadow {
                    blur,
                    offset,
                    opacity,
                    ..
                } => {
                    self.check_sigma(blur * self.scale)?;
                    if !offset.iter().all(|v| v.is_finite())
                        || !opacity.is_finite()
                        || !(0.0..=1.0).contains(&opacity)
                    {
                        return Err(RenderError::Invalid(format!("invalid shadow of {id}")));
                    }
                }
                EffectKind::InnerShadow { .. } => return Err(unsupported("inner shadow")),
                EffectKind::Sharpen { .. } => return Err(unsupported("sharpen kernel")),
                EffectKind::Noise { .. } => return Err(unsupported("deterministic noise kernel")),
            }
        }
        for stroke in app.strokes.iter().filter(|s| s.visible) {
            if !stroke.width.is_finite()
                || stroke.width < 0.0
                || !stroke.miter_limit.is_finite()
                || stroke.miter_limit < 1.0
            {
                return Err(RenderError::Invalid(format!("invalid stroke of {id}")));
            }
            if stroke.alignment != StrokeAlignment::Center && !closed_contours(&node.geometry) {
                return Err(unsupported(
                    "inside/outside stroke requires closed contours",
                ));
            }
        }
        if let Some(mask) = node.source.clip_mask_id {
            self.node(mask)?;
        }
        for child in &node.source.children {
            self.preflight(*child, depth + 1)?;
        }
        Ok(())
    }

    fn prepare_image(
        &self,
        id: ObjectId,
        source: &EncodedImage,
    ) -> Result<Arc<PreparedImage>, RenderError> {
        self.check_cancelled()?;
        if let Some(image) = self.prepared.borrow().get(&source.content_key()) {
            return Ok(image.clone());
        }
        let image = self
            .images
            .prepare(source, &|| {
                self.cancellation
                    .is_some_and(CancellationToken::is_cancelled)
            })
            .map_err(|error| match error {
                ImageAssetError::Cancelled => RenderError::Cancelled,
                ImageAssetError::Limit(reason) => RenderError::Limit(reason),
                ImageAssetError::Invalid(reason) => RenderError::Invalid(reason),
                ImageAssetError::Unsupported(feature) => RenderError::Unsupported {
                    object: id,
                    feature,
                },
            })?;
        self.prepared
            .borrow_mut()
            .insert(source.content_key(), image.clone());
        Ok(image)
    }

    fn check_sigma(&self, sigma: f64) -> Result<(), RenderError> {
        if !sigma.is_finite() || sigma < 0.0 {
            return Err(RenderError::Invalid("invalid Gaussian sigma".into()));
        }
        if sigma > self.limits.max_blur_sigma {
            return Err(RenderError::Limit("blur sigma"));
        }
        Ok(())
    }

    fn render_node(
        &self,
        id: ObjectId,
        window: GRect,
        depth: usize,
        as_mask: bool,
    ) -> Result<Option<Layer<'_>>, RenderError> {
        self.check_cancelled()?;
        if depth >= self.limits.max_depth {
            return Err(RenderError::Limit("composition/mask dependency depth"));
        }
        let node = self.node(id)?;
        if !node.source.visible || (node.source.is_clip_mask && !as_mask) {
            return Ok(None);
        }
        let Some(bounds) = node.visual_bounds else {
            return Ok(None);
        };
        let footprint = transform_rect(bounds, self.world_to_device);
        if footprint.intersection(window).is_none() {
            return Ok(None);
        }
        let app = node.source.effective_appearance();
        // Gather a halo before convolution. Cropping first would create seams
        // and discard offscreen geometry which casts blur/shadow into the region.
        let halo = app
            .effects
            .iter()
            .filter(|e| e.visible)
            .map(|e| match e.kind {
                EffectKind::GaussianBlur { radius } => radius * self.scale * 3.0,
                EffectKind::DropShadow { blur, offset, .. } => {
                    (blur * 3.0 + offset[0].abs().max(offset[1].abs())) * self.scale
                }
                _ => 0.0,
            })
            .sum::<f64>();
        let Some(region) = footprint.intersection(expand(window, halo + 1.0)) else {
            return Ok(None);
        };
        let x = region.x0.floor();
        let y = region.y0.floor();
        let right = region.x1.ceil();
        let bottom = region.y1.ceil();
        if x < f64::from(i32::MIN)
            || y < f64::from(i32::MIN)
            || right > f64::from(i32::MAX)
            || bottom > f64::from(i32::MAX)
        {
            return Err(RenderError::Limit("intermediate coordinate range"));
        }
        let mut layer = self.allocate(
            (right - x).max(1.0) as u32,
            (bottom - y).max(1.0) as u32,
            x as i32,
            y as i32,
        )?;
        let local_to_layer = GAffine::translate(-x, -y)
            .after(self.world_to_device)
            .after(node.world);
        let fill_rule = node_fill_rule(node);
        if let Some(path) = sk_path(&node.geometry)? {
            let transform = sk_transform(local_to_layer)?;
            if let Some(ShapeKind::Image {
                data: Some(source), ..
            }) = &node.source.shape
            {
                let image = self.prepare_image(id, source)?;
                let [_, _, w, h] = node
                    .source
                    .bounds
                    .ok_or_else(|| RenderError::Invalid("image has no local frame".into()))?;
                let pixels_to_local =
                    GAffine::scale(w / f64::from(image.width()), h / f64::from(image.height()));
                let pixels_to_device = self
                    .world_to_device
                    .after(node.world)
                    .after(pixels_to_local);
                let [a, b, c, d, _, _] = pixels_to_device.coeffs;
                let trace = a * a + b * b + c * c + d * d;
                let delta = (a * a + b * b - c * c - d * d).hypot(2.0 * (a * c + b * d));
                let scale = ((trace + delta) * 0.5).sqrt();
                if !scale.is_finite() || scale <= 0.0 {
                    return Err(RenderError::Limit("image sampling transform"));
                }
                let level = &image.levels()[image.level_for_scale(scale)];
                let pixmap = sk::PixmapRef::from_bytes(
                    level.premultiplied_rgba8(),
                    level.width(),
                    level.height(),
                )
                .ok_or_else(|| RenderError::Invalid("invalid prepared image".into()))?;
                let paint = sk::Paint {
                    shader: sk::Pattern::new(
                        pixmap,
                        sk::SpreadMode::Pad,
                        sk::FilterQuality::Bilinear,
                        1.0,
                        sk_transform(GAffine::scale(
                            w / f64::from(level.width()),
                            h / f64::from(level.height()),
                        ))?,
                    ),
                    anti_alias: true,
                    ..sk::Paint::default()
                };
                layer
                    .pixmap
                    .fill_path(&path, &paint, fill_rule, transform, None);
            }
            if let Some(ShapeKind::Raster { layer: source }) = &node.source.shape {
                let [_, _, w, h] = node
                    .source
                    .bounds
                    .ok_or_else(|| RenderError::Invalid("raster has no local frame".into()))?;
                let pixels_to_layer = local_to_layer.after(GAffine::scale(
                    w / f64::from(source.width()),
                    h / f64::from(source.height()),
                ));
                let inverse = pixels_to_layer
                    .inverse()
                    .ok_or_else(|| RenderError::Invalid("singular raster sampling frame".into()))?;
                let (_permit, coverage) =
                    self.path_mask(&layer, &path, local_to_layer, fill_rule)?;
                let width = layer.pixmap.width() as usize;
                for (index, output) in layer.pixmap.data_mut().chunks_exact_mut(4).enumerate() {
                    if index % width == 0 {
                        self.check_cancelled()?;
                    }
                    let mask = f32::from(coverage.data()[index]) / 255.0;
                    if mask == 0.0 {
                        continue;
                    }
                    let point = inverse.apply(GPoint::new(
                        (index % width) as f64 + 0.5,
                        (index / width) as f64 + 0.5,
                    ));
                    let pixel = sample_raster(source, point);
                    for channel in 0..4 {
                        output[channel] =
                            (pixel[channel] * mask * 255.0).round().clamp(0.0, 255.0) as u8;
                    }
                }
            }
            let [px, py, _, _] = node.source.bounds.unwrap_or([0.0; 4]);
            // Existing color paints are parent-frame descriptors. Convert to
            // local explicitly; do not confuse them with local modifier frames.
            let paint_to_local = GAffine::translate(-px, -py);
            for fill in app.fills.iter().filter(|f| f.visible) {
                if let Some(paint) =
                    sk_paint(&fill.paint, fill.opacity, fill.blend_mode, paint_to_local)?
                {
                    layer
                        .pixmap
                        .fill_path(&path, &paint, fill_rule, transform, None);
                }
            }
            for stroke in app.strokes.iter().filter(|s| s.visible && s.width > 0.0) {
                let Some(paint) = sk_paint(
                    &stroke.paint,
                    stroke.opacity,
                    stroke.blend_mode,
                    paint_to_local,
                )?
                else {
                    continue;
                };
                let dash = if stroke.dash_array.is_empty() {
                    None
                } else {
                    Some(
                        sk::StrokeDash::new(
                            stroke.dash_array.iter().map(|v| *v as f32).collect(),
                            stroke.dash_offset as f32,
                        )
                        .ok_or_else(|| {
                            RenderError::Invalid(format!("invalid dash pattern of {id}"))
                        })?,
                    )
                };
                let style = sk::Stroke {
                    width: (stroke.width
                        * if stroke.alignment == StrokeAlignment::Center {
                            1.0
                        } else {
                            2.0
                        }) as f32,
                    miter_limit: stroke.miter_limit as f32,
                    line_cap: match stroke.cap {
                        StrokeCap::Butt => sk::LineCap::Butt,
                        StrokeCap::Round => sk::LineCap::Round,
                        StrokeCap::Square => sk::LineCap::Square,
                    },
                    line_join: match stroke.join {
                        StrokeJoin::Miter => sk::LineJoin::Miter,
                        StrokeJoin::Round => sk::LineJoin::Round,
                        StrokeJoin::Bevel => sk::LineJoin::Bevel,
                    },
                    dash,
                };
                if stroke.alignment == StrokeAlignment::Center {
                    layer
                        .pixmap
                        .stroke_path(&path, &paint, &style, transform, None);
                } else {
                    let (_permit, mut mask) =
                        self.path_mask(&layer, &path, local_to_layer, fill_rule)?;
                    if stroke.alignment == StrokeAlignment::Outside {
                        mask.invert();
                    }
                    layer
                        .pixmap
                        .stroke_path(&path, &paint, &style, transform, Some(&mask));
                }
            }
        }
        // Clip frame ink before effects. Artistic text remains unwrapped and unbounded
        // by its placement frame; both modes retain editable source text.
        if matches!(node.source.shape, Some(ShapeKind::Text { .. }))
            && node.source.text_style.flow == petunia_design_document::TextFlow::Frame
        {
            let [_, _, w, h] = node
                .source
                .bounds
                .ok_or_else(|| RenderError::Invalid("text has no frame".into()))?;
            let frame = sk_path(&GPath::rect(GRect::new(0.0, 0.0, w, h), 0.0, 0.0))?
                .ok_or_else(|| RenderError::Invalid("empty text frame".into()))?;
            let (_permit, mask) =
                self.path_mask(&layer, &frame, local_to_layer, sk::FillRule::Winding)?;
            layer.pixmap.apply_mask(&mask);
        }
        let child_window = GRect::new(x, y, right, bottom);
        for child in &node.source.children {
            if let Some(child_layer) = self.render_node(*child, child_window, depth + 1, false)? {
                composite(
                    &mut layer,
                    &child_layer,
                    map_blend(self.node(*child)?.source.effective_appearance().blend_mode),
                );
            }
        }
        for effect in app.effects.iter().filter(|e| e.visible) {
            match &effect.kind {
                EffectKind::GaussianBlur { radius } => self.blur(
                    &mut layer.pixmap,
                    radius * self.world_to_device.coeffs[0],
                    radius * self.world_to_device.coeffs[3],
                )?,
                EffectKind::DropShadow {
                    offset,
                    blur,
                    color,
                    opacity,
                } => {
                    let mut shadow = self.allocate(
                        layer.pixmap.width(),
                        layer.pixmap.height(),
                        layer.x,
                        layer.y,
                    )?;
                    let dx = offset[0] * self.world_to_device.coeffs[0];
                    let dy = offset[1] * self.world_to_device.coeffs[3];
                    shadow.pixmap.draw_pixmap(
                        0,
                        0,
                        layer.pixmap.as_ref(),
                        &sk::PixmapPaint::default(),
                        sk::Transform::from_translate(dx as f32, dy as f32),
                        None,
                    );
                    let rgb = petunia_design_document::resolve_color_to_rgb(color);
                    for p in shadow.pixmap.data_mut().chunks_exact_mut(4) {
                        let alpha = f64::from(p[3]) * opacity;
                        p[0] = (f64::from(rgb[0]) * alpha).round() as u8;
                        p[1] = (f64::from(rgb[1]) * alpha).round() as u8;
                        p[2] = (f64::from(rgb[2]) * alpha).round() as u8;
                        p[3] = alpha.round() as u8;
                    }
                    self.blur(
                        &mut shadow.pixmap,
                        blur * self.world_to_device.coeffs[0],
                        blur * self.world_to_device.coeffs[3],
                    )?;
                    shadow.pixmap.draw_pixmap(
                        0,
                        0,
                        layer.pixmap.as_ref(),
                        &sk::PixmapPaint::default(),
                        sk::Transform::identity(),
                        None,
                    );
                    std::mem::swap(&mut layer.pixmap, &mut shadow.pixmap);
                }
                _ => {
                    return Err(RenderError::Unsupported {
                        object: id,
                        feature: "effect pixel kernel",
                    });
                }
            }
        }
        self.adjust(&mut layer.pixmap, &app.adjustments)?;
        if let Some(mask_id) = node.source.clip_mask_id {
            let mask = self.node(mask_id)?;
            let grouped = node
                .source
                .parent
                .and_then(|parent| self.surface.nodes.get(&parent))
                .is_some_and(|parent| {
                    parent.source.role == Some(ContainerRole::ClipGroup)
                        && parent.source.children.contains(&mask_id)
                });
            // ClipGroup applies its shared coverage after sibling composition.
            // Applying it to each child as well would square alpha/edge coverage.
            if !grouped {
                let mode = if mask.source.mask_mode != MaskMode::Vector {
                    mask.source.mask_mode
                } else {
                    node.source.mask_mode
                }; // legacy per-reference mode
                self.apply_clip(&mut layer, mask_id, mode, depth + 1)?;
            }
        }
        if node.source.role == Some(ContainerRole::ClipGroup) {
            let masks: Vec<_> = node
                .source
                .children
                .iter()
                .filter(|id| self.surface.nodes[*id].source.is_clip_mask)
                .collect();
            if masks.len() != 1 {
                return Err(RenderError::Invalid(format!(
                    "clip group {id} requires exactly one mask"
                )));
            }
            let mask_id = *masks[0];
            self.apply_clip(
                &mut layer,
                mask_id,
                self.node(mask_id)?.source.mask_mode,
                depth + 1,
            )?;
        }
        let inverse = local_to_layer
            .inverse()
            .ok_or_else(|| RenderError::Invalid(format!("singular opacity frame of {id}")))?;
        let [_, _, w, h] = node.source.bounds.unwrap_or([0.0; 4]);
        let spatial_mask = node.source.modifiers.iter().any(|m| {
            m.enabled
                && matches!(
                    m.kind,
                    petunia_design_document::ModifierKind::TransparentGradient { .. }
                )
        });
        if spatial_mask || app.opacity < 1.0 {
            for (index, pixel) in layer.pixmap.data_mut().chunks_exact_mut(4).enumerate() {
                if index % (right - x).max(1.0) as usize == 0 {
                    self.check_cancelled()?;
                }
                let mask = if spatial_mask {
                    let point = inverse.apply(GPoint::new(
                        (index % (right - x).max(1.0) as usize) as f64 + 0.5,
                        (index / (right - x).max(1.0) as usize) as f64 + 0.5,
                    ));
                    petunia_design_document::modifiers::evaluate_opacity_local(
                        &node.source.modifiers,
                        point,
                        [w, h],
                    )
                    .ok_or_else(|| {
                        RenderError::Invalid(format!("invalid opacity modifier of {id}"))
                    })?
                } else {
                    1.0
                };
                let alpha = (app.opacity * mask).clamp(0.0, 1.0);
                for channel in pixel {
                    *channel = (f64::from(*channel) * alpha).round() as u8;
                }
            }
        }
        Ok(Some(layer))
    }

    fn path_mask<'a>(
        &'a self,
        layer: &Layer<'_>,
        path: &sk::Path,
        transform: GAffine,
        fill_rule: sk::FillRule,
    ) -> Result<(Permit<'a>, sk::Mask), RenderError> {
        let size = sk::IntSize::from_wh(layer.pixmap.width(), layer.pixmap.height())
            .ok_or(RenderError::Limit("mask dimensions"))?;
        let bytes = size.width() as usize * size.height() as usize;
        let permit = self.reserve(bytes)?;
        let mut data = Vec::new();
        data.try_reserve_exact(bytes)
            .map_err(|_| RenderError::Limit("mask allocation"))?;
        data.resize(bytes, 0);
        let mut mask =
            sk::Mask::from_vec(data, size).ok_or(RenderError::Limit("mask allocation"))?;
        mask.fill_path(path, fill_rule, true, sk_transform(transform)?);
        Ok((permit, mask))
    }

    fn apply_clip(
        &self,
        layer: &mut Layer<'_>,
        mask_id: ObjectId,
        mode: MaskMode,
        depth: usize,
    ) -> Result<(), RenderError> {
        let node = self.node(mask_id)?;
        if !node.source.is_clip_mask {
            return Err(RenderError::Invalid(format!("{mask_id} is not a mask")));
        }
        if !node.source.visible {
            layer.pixmap.data_mut().fill(0);
            return Ok(());
        }
        if mode == MaskMode::Vector {
            if let Some(path) = sk_path(&node.geometry)? {
                let frame = GAffine::translate(-f64::from(layer.x), -f64::from(layer.y))
                    .after(self.world_to_device)
                    .after(node.world);
                let (_permit, mask) = self.path_mask(layer, &path, frame, node_fill_rule(node))?;
                layer.pixmap.apply_mask(&mask);
            } else {
                layer.pixmap.data_mut().fill(0);
            }
        } else {
            let window = GRect::new(
                f64::from(layer.x),
                f64::from(layer.y),
                f64::from(layer.x) + f64::from(layer.pixmap.width()),
                f64::from(layer.y) + f64::from(layer.pixmap.height()),
            );
            let mask = self.render_node(mask_id, window, depth, true)?;
            let width = layer.pixmap.width() as usize;
            for (index, pixel) in layer.pixmap.data_mut().chunks_exact_mut(4).enumerate() {
                if index % width == 0 {
                    self.check_cancelled()?;
                }
                let x = i64::from(layer.x) + (index % width) as i64;
                let y = i64::from(layer.y) + (index / width) as i64;
                let coverage = mask
                    .as_ref()
                    .and_then(|m| {
                        let mx = x - i64::from(m.x);
                        let my = y - i64::from(m.y);
                        if mx < 0
                            || my < 0
                            || mx >= i64::from(m.pixmap.width())
                            || my >= i64::from(m.pixmap.height())
                        {
                            return None;
                        }
                        let p = m.pixmap.pixels()
                            [(my as usize) * m.pixmap.width() as usize + mx as usize];
                        // Luminance is computed in linear sRGB, then multiplied by alpha.
                        Some(if mode == MaskMode::Alpha {
                            f64::from(p.alpha()) / 255.0
                        } else {
                            let c = p.demultiply();
                            let linear = |v: u8| {
                                let s = f64::from(v) / 255.0;
                                if s <= 0.04045 {
                                    s / 12.92
                                } else {
                                    ((s + 0.055) / 1.055).powf(2.4)
                                }
                            };
                            (0.2126 * linear(c.red())
                                + 0.7152 * linear(c.green())
                                + 0.0722 * linear(c.blue()))
                                * f64::from(p.alpha())
                                / 255.0
                        })
                    })
                    .unwrap_or(0.0);
                for c in pixel {
                    *c = (f64::from(*c) * coverage).round() as u8;
                }
            }
        }
        Ok(())
    }

    fn blur(&self, pixmap: &mut sk::Pixmap, sigma_x: f64, sigma_y: f64) -> Result<(), RenderError> {
        self.check_sigma(sigma_x)?;
        self.check_sigma(sigma_y)?;
        if sigma_x <= 0.01 && sigma_y <= 0.01 {
            return Ok(());
        }
        let (radius_x, kernel_x) = gaussian_kernel(sigma_x);
        let (radius_y, kernel_y) = gaussian_kernel(sigma_y);
        let bytes = pixmap
            .data()
            .len()
            .checked_mul(std::mem::size_of::<f32>())
            .ok_or(RenderError::Limit("blur scratch size"))?;
        let _permit = self.reserve(bytes)?;
        let mut scratch = Vec::<f32>::new();
        scratch
            .try_reserve_exact(pixmap.data().len())
            .map_err(|_| RenderError::Limit("blur scratch allocation"))?;
        scratch.resize(pixmap.data().len(), 0.0);
        let width = pixmap.width() as usize;
        let height = pixmap.height() as usize;
        // Separable Gaussian: O(W H radius), not a quadratic 2D kernel. RGBA
        // stays premultiplied to prevent fringes around transparent artwork.
        for y in 0..height {
            self.check_cancelled()?;
            for x in 0..width {
                for (k, weight) in kernel_x.iter().enumerate() {
                    let source_x = x as i64 + k as i64 - i64::from(radius_x);
                    if source_x >= 0 && source_x < width as i64 {
                        for c in 0..4 {
                            scratch[(y * width + x) * 4 + c] +=
                                f32::from(pixmap.data()[(y * width + source_x as usize) * 4 + c])
                                    * weight;
                        }
                    }
                }
            }
        }
        for y in 0..height {
            self.check_cancelled()?;
            for x in 0..width {
                let mut rgba = [0.0_f32; 4];
                for (k, weight) in kernel_y.iter().enumerate() {
                    let source_y = y as i64 + k as i64 - i64::from(radius_y);
                    if source_y >= 0 && source_y < height as i64 {
                        for c in 0..4 {
                            rgba[c] += scratch[(source_y as usize * width + x) * 4 + c] * weight;
                        }
                    }
                }
                for c in 0..4 {
                    pixmap.data_mut()[(y * width + x) * 4 + c] =
                        rgba[c].round().clamp(0.0, 255.0) as u8;
                }
            }
        }
        Ok(())
    }

    fn adjust(
        &self,
        pixmap: &mut sk::Pixmap,
        adjustments: &[petunia_design_document::adjustments::AdjustmentItem],
    ) -> Result<(), RenderError> {
        if !adjustments.iter().any(|a| a.visible) {
            return Ok(());
        }
        let compiled: Vec<_> = adjustments
            .iter()
            .filter(|a| a.visible)
            .map(PreparedAdjustment::new)
            .collect::<Result<_, _>>()?;
        let width = pixmap.width() as usize;
        for (index, p) in pixmap.data_mut().chunks_exact_mut(4).enumerate() {
            if index % width == 0 {
                self.check_cancelled()?;
            }
            if p[3] == 0 {
                continue;
            }
            let alpha = f32::from(p[3]);
            let mut rgb = [
                f32::from(p[0]) / alpha,
                f32::from(p[1]) / alpha,
                f32::from(p[2]) / alpha,
            ]
            .map(srgb_to_linear);
            for adjustment in &compiled {
                rgb = adjustment.apply(rgb);
            }
            for c in 0..3 {
                p[c] = (linear_to_srgb(rgb[c].clamp(0.0, 1.0)) * alpha).round() as u8;
            }
        }
        Ok(())
    }
}

enum PreparedAdjustment<'a> {
    Direct(&'a petunia_design_document::adjustments::AdjustmentItem),
    Curves {
        master: petunia_design_document::adjustments::PreparedCurve,
        channels: [Option<petunia_design_document::adjustments::PreparedCurve>; 3],
        opacity: f32,
    },
}
impl<'a> PreparedAdjustment<'a> {
    fn new(
        item: &'a petunia_design_document::adjustments::AdjustmentItem,
    ) -> Result<Self, RenderError> {
        use petunia_design_document::adjustments::{AdjustmentKind, PreparedCurve};
        if let AdjustmentKind::Curves {
            master_points,
            red_points,
            green_points,
            blue_points,
        } = &item.kind
        {
            let prepare = |points: &Option<Vec<[f64; 2]>>| {
                points
                    .as_deref()
                    .map(PreparedCurve::new)
                    .transpose()
                    .map_err(|e| RenderError::Invalid(e.to_string()))
            };
            Ok(Self::Curves {
                master: PreparedCurve::new(master_points)
                    .map_err(|e| RenderError::Invalid(e.to_string()))?,
                channels: [
                    prepare(red_points)?,
                    prepare(green_points)?,
                    prepare(blue_points)?,
                ],
                opacity: item.opacity as f32,
            })
        } else {
            Ok(Self::Direct(item))
        }
    }
    fn apply(&self, rgb: [f32; 3]) -> [f32; 3] {
        match self {
            Self::Direct(item) => item.apply(rgb),
            Self::Curves {
                master,
                channels,
                opacity,
            } => std::array::from_fn(|i| {
                let channel = channels[i]
                    .as_ref()
                    .map_or(rgb[i], |curve| curve.sample(rgb[i]));
                rgb[i] + (master.sample(channel) - rgb[i]) * opacity.clamp(0.0, 1.0)
            }),
        }
    }
}
fn srgb_to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}
fn linear_to_srgb(value: f32) -> f32 {
    if value <= 0.0031308 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}

fn composite(target: &mut Layer<'_>, source: &Layer<'_>, blend: sk::BlendMode) {
    let paint = sk::PixmapPaint {
        blend_mode: blend,
        ..sk::PixmapPaint::default()
    };
    // Coordinates are bounded on allocation; widened subtraction prevents wrap.
    let dx = i64::from(source.x) - i64::from(target.x);
    let dy = i64::from(source.y) - i64::from(target.y);
    if let (Ok(dx), Ok(dy)) = (i32::try_from(dx), i32::try_from(dy)) {
        target.pixmap.draw_pixmap(
            dx,
            dy,
            source.pixmap.as_ref(),
            &paint,
            sk::Transform::identity(),
            None,
        );
    }
}

fn gaussian_kernel(sigma: f64) -> (i32, Vec<f32>) {
    if sigma <= 0.01 {
        return (0, vec![1.0]);
    }
    let radius = (sigma * 3.0).ceil() as i32;
    let mut kernel: Vec<f32> = (-radius..=radius)
        .map(|x| (-f64::from(x).powi(2) / (2.0 * sigma * sigma)).exp() as f32)
        .collect();
    let sum: f32 = kernel.iter().sum();
    for weight in &mut kernel {
        *weight /= sum;
    }
    (radius, kernel)
}

fn transform_rect(rect: GRect, transform: GAffine) -> GRect {
    let points = [
        GPoint::new(rect.x0, rect.y0),
        GPoint::new(rect.x1, rect.y0),
        GPoint::new(rect.x0, rect.y1),
        GPoint::new(rect.x1, rect.y1),
    ]
    .map(|p| transform.apply(p));
    GRect::new(
        points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min),
        points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min),
        points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max),
        points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max),
    )
}

fn sk_transform(value: GAffine) -> Result<sk::Transform, RenderError> {
    let [a, b, c, d, e, f] = value.coeffs;
    let result =
        sk::Transform::from_row(a as f32, b as f32, c as f32, d as f32, e as f32, f as f32);
    if !result.is_finite() || result.invert().is_none() {
        return Err(RenderError::Limit("backend transform range"));
    }
    Ok(result)
}

fn sk_path(path: &GPath) -> Result<Option<sk::Path>, RenderError> {
    if path.is_empty() {
        return Ok(None);
    }
    if !path.is_finite() || path.verbs.len() > 1_000_000 {
        return Err(RenderError::Limit("path coordinates/verb count"));
    }
    let mut builder = sk::PathBuilder::new();
    for verb in &path.verbs {
        match *verb {
            PathVerb::MoveTo(p) => builder.move_to(p.x as f32, p.y as f32),
            PathVerb::LineTo(p) => builder.line_to(p.x as f32, p.y as f32),
            PathVerb::QuadTo(c, p) => {
                builder.quad_to(c.x as f32, c.y as f32, p.x as f32, p.y as f32)
            }
            PathVerb::CubicTo(a, b, p) => builder.cubic_to(
                a.x as f32, a.y as f32, b.x as f32, b.y as f32, p.x as f32, p.y as f32,
            ),
            PathVerb::Close => builder.close(),
        }
    }
    builder
        .finish()
        .map(Some)
        .ok_or_else(|| RenderError::Invalid("path cannot be rasterized".into()))
}

fn closed_contours(path: &GPath) -> bool {
    let mut open = false;
    let mut closed = false;
    for verb in &path.verbs {
        match verb {
            PathVerb::MoveTo(_) => {
                if open {
                    return false;
                }
                open = true;
            }
            PathVerb::Close => {
                open = false;
                closed = true;
            }
            _ => {
                if !open {
                    return false;
                }
            }
        }
    }
    closed && !open
}

fn stops(values: &[GradientStop], opacity: f64) -> Result<Vec<sk::GradientStop>, RenderError> {
    if values.is_empty() {
        return Err(RenderError::Invalid("empty gradient".into()));
    }
    let mut sorted = values.to_vec();
    if sorted
        .iter()
        .any(|s| !s.offset.is_finite() || !s.opacity.is_finite())
    {
        return Err(RenderError::Invalid("non-finite gradient stop".into()));
    }
    sorted.sort_by(|a, b| a.offset.total_cmp(&b.offset));
    Ok(sorted
        .iter()
        .map(|s| {
            let rgb = s.resolved_rgb();
            sk::GradientStop::new(
                s.offset as f32,
                sk::Color::from_rgba(
                    rgb[0],
                    rgb[1],
                    rgb[2],
                    (s.opacity * opacity).clamp(0.0, 1.0) as f32,
                )
                .unwrap_or(sk::Color::TRANSPARENT),
            )
        })
        .collect())
}

fn sk_paint(
    paint: &Paint,
    opacity: f64,
    blend: petunia_design_document::BlendMode,
    frame: GAffine,
) -> Result<Option<sk::Paint<'static>>, RenderError> {
    if !opacity.is_finite() || !(0.0..=1.0).contains(&opacity) {
        return Err(RenderError::Invalid("invalid paint opacity".into()));
    }
    let shader = match paint {
        Paint::None => return Ok(None),
        Paint::Solid(color) => {
            let rgb = petunia_design_document::resolve_color_to_rgb(color);
            sk::Shader::SolidColor(
                sk::Color::from_rgba(rgb[0], rgb[1], rgb[2], opacity as f32)
                    .ok_or_else(|| RenderError::Invalid("invalid paint color".into()))?,
            )
        }
        Paint::LinearGradient(g) => sk::LinearGradient::new(
            local_gradient_point(frame, g.start)?,
            local_gradient_point(frame, g.end)?,
            stops(&g.stops, opacity)?,
            sk::SpreadMode::Pad,
            sk::Transform::identity(),
        )
        .ok_or_else(|| RenderError::Invalid("invalid linear gradient".into()))?,
        Paint::RadialGradient(g) => {
            let center = local_gradient_point(frame, g.center)?;
            sk::RadialGradient::new(
                center,
                center,
                g.radius as f32,
                stops(&g.stops, opacity)?,
                sk::SpreadMode::Pad,
                sk::Transform::identity(),
            )
            .ok_or_else(|| RenderError::Invalid("invalid radial gradient".into()))?
        }
    };
    Ok(Some(sk::Paint {
        shader,
        blend_mode: map_blend(blend),
        anti_alias: true,
        ..sk::Paint::default()
    }))
}

fn local_gradient_point(frame: GAffine, coordinates: [f64; 2]) -> Result<sk::Point, RenderError> {
    let point = frame.apply(GPoint::new(coordinates[0], coordinates[1]));
    let result = sk::Point::from_xy(point.x as f32, point.y as f32);
    if !result.is_finite() {
        return Err(RenderError::Limit("gradient coordinate range"));
    }
    Ok(result)
}

fn map_blend(mode: petunia_design_document::BlendMode) -> sk::BlendMode {
    use petunia_design_document::BlendMode as D;
    match mode {
        D::Normal => sk::BlendMode::SourceOver,
        D::Multiply => sk::BlendMode::Multiply,
        D::Screen => sk::BlendMode::Screen,
        D::Overlay => sk::BlendMode::Overlay,
        D::Darken => sk::BlendMode::Darken,
        D::Lighten => sk::BlendMode::Lighten,
        D::ColorDodge => sk::BlendMode::ColorDodge,
        D::ColorBurn => sk::BlendMode::ColorBurn,
        D::HardLight => sk::BlendMode::HardLight,
        D::SoftLight => sk::BlendMode::SoftLight,
        D::Difference => sk::BlendMode::Difference,
        D::Exclusion => sk::BlendMode::Exclusion,
        D::Hue => sk::BlendMode::Hue,
        D::Saturation => sk::BlendMode::Saturation,
        D::Color => sk::BlendMode::Color,
        D::Luminosity => sk::BlendMode::Luminosity,
    }
}

/// Composes an uncommitted new pixel layer above a completed preview. No
/// canonical object identity is fabricated for the disposable draft.
pub fn composite_raster_preview(
    output: &mut PixelBufferRgba8,
    request: RenderRequest,
    source: &petunia_design_raster::RasterLayer,
    pixels_to_world: GAffine,
    cancellation: &CancellationToken,
) -> Result<(), RenderError> {
    if output.width != request.width
        || output.height != request.height
        || output.data.len() != output.width as usize * output.height as usize * 4
    {
        return Err(RenderError::Invalid("raster preview dimensions".into()));
    }
    let inverse = pixels_to_world
        .inverse()
        .ok_or_else(|| RenderError::Invalid("singular raster preview".into()))?;
    let width = output.width as usize;
    for (index, pixel) in output.data.chunks_exact_mut(4).enumerate() {
        if index % width == 0 && cancellation.is_cancelled() {
            return Err(RenderError::Cancelled);
        }
        let world = GPoint::new(
            request.viewport.x0
                + (index % width) as f64 * request.viewport.width() / f64::from(request.width)
                + 0.5 * request.viewport.width() / f64::from(request.width),
            request.viewport.y0
                + ((index / width) as f64 + 0.5) * request.viewport.height()
                    / f64::from(request.height),
        );
        let src = sample_raster(source, inverse.apply(world));
        let dst_alpha = f32::from(pixel[3]) / 255.0;
        let alpha = src[3] + dst_alpha * (1.0 - src[3]);
        if alpha > 0.0 {
            for c in 0..3 {
                pixel[c] = ((src[c] + f32::from(pixel[c]) / 255.0 * dst_alpha * (1.0 - src[3]))
                    / alpha
                    * 255.0)
                    .round()
                    .clamp(0.0, 255.0) as u8;
            }
            pixel[3] = (alpha * 255.0).round() as u8;
        }
    }
    Ok(())
}

/// Bilinear premultiplied sampling with transparent exterior, including masks.
fn sample_raster(source: &petunia_design_raster::RasterLayer, point: GPoint) -> [f32; 4] {
    let x = point.x - 0.5;
    let y = point.y - 0.5;
    let x0 = x.floor() as i64;
    let y0 = y.floor() as i64;
    let fx = (x - x.floor()) as f32;
    let fy = (y - y.floor()) as f32;
    let mut result = [0.0; 4];
    for (dx, dy, weight) in [
        (0, 0, (1.0 - fx) * (1.0 - fy)),
        (1, 0, fx * (1.0 - fy)),
        (0, 1, (1.0 - fx) * fy),
        (1, 1, fx * fy),
    ] {
        let p = source.pixel(x0.saturating_add(dx), y0.saturating_add(dy));
        for c in 0..3 {
            result[c] += p[c] * p[3] * weight;
        }
        result[3] += p[3] * weight;
    }
    result
}

fn node_fill_rule(node: &RenderNode) -> sk::FillRule {
    if matches!(node.source.shape, Some(ShapeKind::Text { .. })) || node.source.fill_rule == petunia_design_geometry::FillRule::NonZero {
        sk::FillRule::Winding
    } else {
        sk::FillRule::EvenOdd
    }
}
