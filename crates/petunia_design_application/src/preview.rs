//! Latest-request preview ownership. Workers only see immutable surface copies;
//! source identity is scoped to this process, never persisted as document ID.
use crate::view_camera::ViewportCamera;
use petunia_design_document::Surface;
use petunia_design_jobs::{JobExecutor, JobFailure, JobHandle, JobManager};
use petunia_design_render::{
    CpuRenderer, PixelBufferRgba8, RenderError, RenderLimits, RenderRequest, RenderSurface,
};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, OnceLock,
};

/// Process-local identity prevents same-revision tabs from sharing results.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreviewSourceId(u64);
/// Captured once per surface revision; selection and camera do not change it.
#[derive(Clone)]
pub struct PreviewSource {
    id: PreviewSourceId,
    revision: u64,
    surface: Arc<Surface>,
    raster_edit: Option<RasterPreview>,
    text_edit: Option<(petunia_design_foundation::ObjectId, String)>,
    prepared: Arc<OnceLock<Arc<RenderSurface>>>,
}
#[derive(Clone)]
struct RasterPreview {
    target: Option<petunia_design_foundation::ObjectId>,
    layer: Arc<petunia_design_raster::RasterLayer>,
    pixels_to_world: petunia_design_geometry::GAffine,
}
fn next_source_id() -> PreviewSourceId {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    PreviewSourceId(
        NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .expect("preview source identity space exhausted"),
    )
}
impl std::fmt::Debug for PreviewSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreviewSource")
            .field("id", &self.id)
            .field("revision", &self.revision)
            .field("surface", &self.surface.id)
            .field("objects", &self.surface.objects().len())
            .finish_non_exhaustive()
    }
}
impl PartialEq for PreviewSource {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl PreviewSource {
    /// Copies canonical descriptors; immutable geometry/image bytes stay shared.
    pub fn capture(surface: &Surface, revision: u64) -> Arc<Self> {
        Arc::new(Self {
            id: next_source_id(),
            revision,
            surface: Arc::new(surface.clone()),
            raster_edit: None,
            text_edit: None,
            prepared: Arc::new(OnceLock::new()),
        })
    }
    /// Immutable draft snapshot. It neither changes canonical IDs nor adds a
    /// document revision/history entry. Successive drafts share the source.
    pub fn with_raster_edit(
        &self,
        target: Option<petunia_design_foundation::ObjectId>,
        layer: Arc<petunia_design_raster::RasterLayer>,
        pixels_to_world: petunia_design_geometry::GAffine,
    ) -> Arc<Self> {
        Arc::new(Self {
            id: next_source_id(),
            revision: self.revision,
            surface: self.surface.clone(),
            prepared: Arc::new(OnceLock::new()),
            text_edit: self.text_edit.clone(),
            raster_edit: Some(RasterPreview {
                target,
                layer,
                pixels_to_world,
            }),
        })
    }
    /// Worker-produced immutable glyph/scene data; no shaping on UI reads.
    pub fn prepared_scene(&self) -> Option<Arc<RenderSurface>> {
        self.prepared.get().cloned()
    }
    /// Derived text draft through the document mutation boundary. Neither the
    /// live session nor its history is touched; shaping stays on the worker.
    pub fn with_text_edit(
        &self,
        target: petunia_design_foundation::ObjectId,
        content: String,
    ) -> Result<Arc<Self>, petunia_design_foundation::PetuniaError> {
        use petunia_design_document::ShapeKind;
        if content.len() > 64 * 1024 {
            return Err(petunia_design_foundation::PetuniaError::invalid_input(
                "text draft exceeds 64 KiB",
            ));
        }
        if !self.surface.objects().iter().any(|object| {
            object.id == target && matches!(object.shape, Some(ShapeKind::Text { .. }))
        }) {
            return Err(petunia_design_foundation::PetuniaError::not_found(
                "text draft target",
            ));
        }
        Ok(Arc::new(Self {
            id: next_source_id(),
            revision: self.revision,
            surface: self.surface.clone(),
            raster_edit: self.raster_edit.clone(),
            text_edit: Some((target, content)),
            prepared: Arc::new(OnceLock::new()),
        }))
    }
    /// Cache identity, distinct from canonical `SurfaceId`.
    pub fn id(&self) -> PreviewSourceId {
        self.id
    }
    /// Revision at capture.
    pub fn revision(&self) -> u64 {
        self.revision
    }
    /// Stable canonical surface identity.
    pub fn surface_id(&self) -> petunia_design_foundation::SurfaceId {
        self.surface.id
    }
    pub fn surface_snapshot(&self) -> &Surface {
        &self.surface
    }
    /// Reuses worker-prepared glyph/geometry data across viewport and analyses.
    pub fn prepare_scene(
        &self,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<Arc<RenderSurface>, RenderError> {
        if cancelled() {
            return Err(RenderError::Cancelled);
        }
        if let Some(scene) = self.prepared_scene() {
            return Ok(scene);
        }
        let document = if let Some((target, content)) = &self.text_edit {
            let mut document = petunia_design_document::Document::new();
            let mut mutator = petunia_design_document::DocumentMutator::new(&mut document);
            mutator.attach_surface(self.surface.as_ref().clone());
            let mut shape = self
                .surface
                .objects()
                .iter()
                .find(|o| o.id == *target)
                .and_then(|o| o.shape.clone())
                .ok_or_else(|| RenderError::Invalid("text draft target missing".into()))?;
            let petunia_design_document::ShapeKind::Text { content: text, .. } = &mut shape else {
                return Err(RenderError::Invalid(
                    "text draft target changed kind".into(),
                ));
            };
            *text = content.clone();
            mutator
                .set_shape(*target, Some(shape))
                .map_err(|e| RenderError::Invalid(e.to_string()))?;
            document
                .validate()
                .map_err(|e| RenderError::Invalid(e.to_string()))?;
            Some(document)
        } else {
            None
        };
        let surface = document
            .as_ref()
            .map_or(self.surface.as_ref(), |doc| &doc.surfaces()[0]);
        let mut scene = RenderSurface::extract_cancellable(surface, cancelled)?;
        if let Some(edit) = self
            .raster_edit
            .as_ref()
            .filter(|edit| edit.target.is_some())
        {
            scene = scene
                .with_raster_preview(edit.target.expect("filtered target"), edit.layer.clone())?;
        }
        let scene = Arc::new(scene);
        if cancelled() {
            return Err(RenderError::Cancelled);
        }
        let _ = self.prepared.set(scene.clone());
        Ok(self.prepared_scene().unwrap_or(scene))
    }
    /// Disposable new-layer artwork is shared by canvas and pixel analyses.
    pub fn composite_draft(
        &self,
        pixels: &mut PixelBufferRgba8,
        render: RenderRequest,
        token: &petunia_design_jobs::CancellationToken,
    ) -> Result<(), RenderError> {
        if let Some(edit) = self.raster_edit.as_ref().filter(|e| e.target.is_none()) {
            petunia_design_render::composite_raster_preview(
                pixels,
                render,
                &edit.layer,
                edit.pixels_to_world,
                token,
            )?;
        }
        Ok(())
    }
}
/// A complete presentation key, including viewport and display mode.
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewRequest {
    pub source: Arc<PreviewSource>,
    pub render: RenderRequest,
    pub channel: usize,
    pub soft_proof: bool,
    pub proof_settings: Option<petunia_design_color::IccProofSettings>,
    pub transparent_artboard: bool,
}
impl PreviewRequest {
    /// Validates the camera before dimensions are converted or work is admitted.
    pub fn from_camera(
        source: Arc<PreviewSource>,
        camera: &ViewportCamera,
        channel: usize,
        soft_proof: bool,
    ) -> Result<Self, JobFailure> {
        let viewport = camera.visible_doc_rect();
        let w = camera.viewport_width.ceil();
        let h = camera.viewport_height.ceil();
        if !viewport.is_finite()
            || viewport.width() <= 0.0
            || viewport.height() <= 0.0
            || ![w, h, camera.zoom]
                .iter()
                .all(|v| v.is_finite() && *v > 0.0)
        {
            return Err(JobFailure::Failed("invalid preview camera".into()));
        }
        if w > f64::from(u32::MAX)
            || h > f64::from(u32::MAX)
            || w * h > RenderLimits::default().max_output_pixels as f64
        {
            return Err(JobFailure::Failed("preview output pixel limit".into()));
        }
        if channel > 4 {
            return Err(JobFailure::Failed("unsupported preview channel".into()));
        }
        Ok(Self {
            source,
            render: RenderRequest {
                viewport,
                width: w as u32,
                height: h as u32,
                background: [0; 4],
            },
            channel,
            soft_proof,
            proof_settings: None,
            transparent_artboard: false,
        })
    }
}
/// Only complete pixels may be presented; their world region travels with them.
#[derive(Debug)]
pub struct PreviewFrame {
    pub request: PreviewRequest,
    pub pixels: PixelBufferRgba8,
    pub warnings: Vec<String>,
}
struct Completed {
    frame: Arc<PreviewFrame>,
    scene: Arc<RenderSurface>,
}
/// One pending request and one visible result, with bounded worker admission.
pub struct PreviewController {
    executor: JobExecutor,
    desired: Option<PreviewRequest>,
    pending: Option<JobHandle<Completed>>,
    latest: Option<Arc<PreviewFrame>>,
    scene: Option<(PreviewSourceId, Arc<RenderSurface>)>,
    failure: Option<JobFailure>,
}
impl PreviewController {
    /// No panic if threads cannot be created. Drop cancels without joining UI.
    pub fn new() -> Result<Self, JobFailure> {
        Ok(Self {
            executor: JobExecutor::new(2, 2, JobManager::new())?,
            desired: None,
            pending: None,
            latest: None,
            scene: None,
            failure: None,
        })
    }
    /// Replaces obsolete work. A previous viewport can stay visible only for
    /// the exact same immutable source and display mode, at its original region.
    pub fn request(&mut self, desired: Option<PreviewRequest>) {
        if self.desired == desired {
            return;
        }
        self.pending = None;
        self.failure = None;
        if self.latest.as_ref().is_some_and(|frame| {
            desired.as_ref().is_none_or(|next| {
                next.source.id() != frame.request.source.id()
                    || next.channel != frame.request.channel
                    || next.soft_proof != frame.request.soft_proof
                    || next.proof_settings != frame.request.proof_settings
                    || next.transparent_artboard != frame.request.transparent_artboard
            })
        }) {
            self.latest = None;
        }
        if self
            .scene
            .as_ref()
            .is_some_and(|(id, _)| desired.as_ref().is_none_or(|next| next.source.id() != *id))
        {
            self.scene = None;
        }
        self.desired = desired;
        self.schedule();
    }
    fn schedule(&mut self) {
        if self.pending.is_some() || self.failure.is_some() {
            return;
        }
        let Some(request) = self.desired.clone() else {
            return;
        };
        if self
            .latest
            .as_ref()
            .is_some_and(|frame| frame.request == request)
        {
            return;
        }
        let scene = self
            .scene
            .as_ref()
            .filter(|(id, _)| *id == request.source.id())
            .map(|(_, scene)| scene.clone());
        let result = self.executor.submit(
            "canvas-preview",
            request.source.revision(),
            move |context| {
                context.check_cancelled()?;
                if request.soft_proof && request.proof_settings.is_none() {
                    return Err(JobFailure::Failed(
                        "ICC soft proof requires loaded CMYK press and RGB monitor profiles".into(),
                    ));
                }
                let render = request.render;
                if u64::from(render.width) * u64::from(render.height)
                    > RenderLimits::default().max_output_pixels
                    || render.width == 0
                    || render.height == 0
                    || request.channel > 4
                    || !render.viewport.is_finite()
                    || render.viewport.width() <= 0.0
                    || render.viewport.height() <= 0.0
                {
                    return Err(JobFailure::Failed(
                        "invalid preview dimensions/channel".into(),
                    ));
                }
                let scene = match scene {
                    Some(scene) => scene,
                    None => request
                        .source
                        .prepare_scene(&|| context.cancellation().is_cancelled())
                        .map_err(render_failure)?,
                };
                let count = (u64::from(render.width) * u64::from(render.height) * 4) as usize;
                let mut data = Vec::new();
                data.try_reserve_exact(count)
                    .map_err(|_| JobFailure::Failed("preview backdrop allocation".into()))?;
                data.resize(count, 0);
                let [x, y, w, h] = scene.bounds();
                let v = render.viewport;
                // Canonical artboard background. Root blend modes must see it
                // during CPU composition, not after a transparent render is uploaded.
                let background = if request.transparent_artboard {
                    [0; 4]
                } else {
                    request
                        .source
                        .surface
                        .background_rgba8()
                        .map_err(|e| JobFailure::Failed(e.to_string()))?
                };
                for row in 0..render.height {
                    context.check_cancelled()?;
                    let py = v.y0 + (f64::from(row) + 0.5) * v.height() / f64::from(render.height);
                    if py < y || py >= y + h {
                        continue;
                    }
                    for col in 0..render.width {
                        let px =
                            v.x0 + (f64::from(col) + 0.5) * v.width() / f64::from(render.width);
                        if px >= x && px < x + w {
                            let offset = (row as usize * render.width as usize + col as usize) * 4;
                            data[offset..offset + 4].copy_from_slice(&background);
                        }
                    }
                }
                let backdrop = PixelBufferRgba8 {
                    width: render.width,
                    height: render.height,
                    data,
                };
                let mut pixels = CpuRenderer::default()
                    .render_over_cancellable(&scene, render, &backdrop, context.cancellation())
                    .map_err(render_failure)?;
                request
                    .source
                    .composite_draft(&mut pixels, render, context.cancellation())
                    .map_err(render_failure)?;
                if request.soft_proof {
                    request
                        .proof_settings
                        .as_ref()
                        .expect("preflighted ICC proof")
                        .apply_rgba8(&mut pixels.data, &|| context.cancellation().is_cancelled())
                        .map_err(|e| JobFailure::Failed(e.to_string()))?;
                    context.check_cancelled()?;
                }
                for row in pixels.data.chunks_exact_mut(render.width as usize * 4) {
                    context.check_cancelled()?;
                    for pixel in row.as_chunks_mut::<4>().0.iter_mut() {
                        match request.channel {
                            1..=3 => {
                                let c = pixel[request.channel - 1];
                                pixel[..3].fill(c);
                            }
                            4 => {
                                let a = pixel[3];
                                pixel.copy_from_slice(&[a, a, a, 255]);
                            }
                            _ => {}
                        }
                    }
                }
                context.check_cancelled()?;
                Ok(Completed {
                    frame: {
                        let warnings = scene.text_warnings();
                        let _ = request.source.prepared.set(scene.clone());
                        Arc::new(PreviewFrame {
                            request,
                            pixels,
                            warnings,
                        })
                    },
                    scene,
                })
            },
        );
        match result {
            Ok(handle) => self.pending = Some(handle),
            Err(JobFailure::QueueFull) => {}
            Err(error) => self.failure = Some(error),
        }
    }
    /// Nonblocking publication rejects mismatched source, revision, camera/mode.
    pub fn poll(&mut self) {
        let result = self.pending.as_ref().map(|handle| {
            handle.try_result(
                self.desired
                    .as_ref()
                    .map_or(0, |request| request.source.revision()),
            )
        });
        match result {
            Some(Ok(Some(done))) => {
                self.pending = None;
                if self.desired.as_ref() == Some(&done.frame.request) {
                    self.scene = Some((done.frame.request.source.id(), done.scene));
                    self.latest = Some(done.frame);
                }
            }
            Some(Err(error)) => {
                self.pending = None;
                self.failure = Some(error);
            }
            _ => {}
        }
        self.schedule();
    }
    /// Current visible complete frame.
    pub fn frame(&self) -> Option<Arc<PreviewFrame>> {
        self.latest.clone()
    }
    /// Failure reason for the current request; replaced requests may retry.
    pub fn failure(&self) -> Option<&JobFailure> {
        self.failure.as_ref()
    }
    /// Whether the UI should poll again, including temporary queue pressure.
    pub fn is_pending(&self) -> bool {
        self.failure.is_none()
            && self.desired.as_ref().is_some_and(|request| {
                self.latest
                    .as_ref()
                    .is_none_or(|frame| frame.request != *request)
            })
    }
    /// Clears presentation and cancels captured work on document/owner teardown.
    pub fn clear(&mut self) {
        self.request(None);
    }
}
fn render_failure(error: RenderError) -> JobFailure {
    match error {
        RenderError::Cancelled => JobFailure::Cancelled,
        error => JobFailure::Failed(error.to_string()),
    }
}
