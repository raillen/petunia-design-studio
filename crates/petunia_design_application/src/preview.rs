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
    Arc,
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
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Arc::new(Self {
            id: PreviewSourceId(NEXT.fetch_add(1, Ordering::Relaxed)),
            revision,
            surface: Arc::new(surface.clone()),
        })
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
}
/// A complete presentation key, including viewport and display mode.
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewRequest {
    pub source: Arc<PreviewSource>,
    pub render: RenderRequest,
    pub channel: usize,
    pub soft_proof: bool,
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
        })
    }
}
/// Only complete pixels may be presented; their world region travels with them.
#[derive(Debug)]
pub struct PreviewFrame {
    pub request: PreviewRequest,
    pub pixels: PixelBufferRgba8,
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
                if request.soft_proof {
                    return Err(JobFailure::Failed(
                        "ICC soft proof requires an available color-management engine".into(),
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
                    None => Arc::new(
                        RenderSurface::extract_cancellable(&request.source.surface, &|| {
                            context.cancellation().is_cancelled()
                        })
                        .map_err(render_failure)?,
                    ),
                };
                let count = (u64::from(render.width) * u64::from(render.height) * 4) as usize;
                let mut data = Vec::new();
                data.try_reserve_exact(count)
                    .map_err(|_| JobFailure::Failed("preview backdrop allocation".into()))?;
                data.resize(count, 0);
                let [x, y, w, h] = scene.bounds();
                let v = render.viewport;
                // Artboard gray matches the canvas. Root blend modes must see it
                // during CPU composition, not after a transparent render is uploaded.
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
                            data[offset..offset + 4].copy_from_slice(&[0xe2, 0xe4, 0xe8, 255]);
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
                for row in pixels.data.chunks_exact_mut(render.width as usize * 4) {
                    context.check_cancelled()?;
                    for pixel in row.chunks_exact_mut(4) {
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
                    frame: Arc::new(PreviewFrame { request, pixels }),
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
