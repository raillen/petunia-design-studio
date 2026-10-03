//! Composition-derived, alpha-weighted display histogram. No fabricated bins,
//! encoded-resource I/O or cold image decoding occurs on the UI thread.
use crate::preview::PreviewSource;
use petunia_design_foundation::ObjectId;
use petunia_design_geometry::GRect;
use petunia_design_jobs::{CancellationToken, JobExecutor, JobFailure, JobHandle, JobManager};
use petunia_design_render::{CpuRenderer, PixelBufferRgba8, RenderRequest};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct HistogramRequest {
    pub source: Arc<PreviewSource>,
    /// Visible composite within this object's evaluated bounds; other layers
    /// still participate in occlusion, masks and ancestor composition.
    pub object: Option<ObjectId>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Histogram {
    bins: [[u64; 256]; 5],
    pub sampled_pixels: usize,
}
impl Default for Histogram {
    fn default() -> Self {
        Self {
            bins: [[0; 256]; 5],
            sampled_pixels: 0,
        }
    }
}
impl Histogram {
    pub fn from_pixels(pixels: &PixelBufferRgba8) -> Self {
        let mut histogram = Self::default();
        for pixel in pixels.data.as_chunks::<4>().0.iter() {
            let weight = u64::from(pixel[3]);
            if weight == 0 {
                continue;
            }
            histogram.sampled_pixels += 1;
            for (index, &value) in pixel[..3].iter().enumerate() {
                histogram.bins[0][usize::from(value)] += weight;
                histogram.bins[index + 1][usize::from(value)] += weight;
            }
            let luma = ((2126 * u32::from(pixel[0])
                + 7152 * u32::from(pixel[1])
                + 722 * u32::from(pixel[2])
                + 5000)
                / 10000) as usize;
            histogram.bins[4][luma] += weight;
        }
        histogram
    }
    pub fn summary(&self, channel: u8) -> ([f32; 32], u32, u32, u32, u32) {
        let bins = &self.bins[usize::from(channel.min(4))];
        let total: u64 = bins.iter().sum();
        if total == 0 {
            return ([0.; 32], 0, 0, 0, 0);
        }
        let mean = (bins
            .iter()
            .enumerate()
            .map(|(i, &count)| i as u64 * count)
            .sum::<u64>() as f64
            / total as f64)
            .round() as u32;
        let mut display = [0f32; 32];
        for (index, group) in bins.as_chunks::<8>().0.iter().enumerate() {
            display[index] = group.iter().sum::<u64>() as f32;
        }
        let max = display.iter().copied().fold(0f32, f32::max);
        for bin in &mut display {
            *bin /= max;
        }
        let shadows = (bins[..64].iter().sum::<u64>() as f64 * 100. / total as f64).round() as u32;
        let through_midtones =
            (bins[..192].iter().sum::<u64>() as f64 * 100. / total as f64).round() as u32;
        (
            display,
            mean,
            shadows,
            through_midtones - shadows,
            100 - through_midtones,
        )
    }
}
pub fn analyze(
    request: &HistogramRequest,
    cancelled: &CancellationToken,
) -> Result<Histogram, JobFailure> {
    let scene = request
        .source
        .prepare_scene(&|| cancelled.is_cancelled())
        .map_err(|e| JobFailure::Failed(e.to_string()))?;
    let [x, y, w, h] = scene.bounds();
    let viewport = if let Some(id) = request.object {
        let node = scene
            .node(id)
            .ok_or_else(|| JobFailure::Failed("histogram object missing".into()))?;
        let Some(bounds) = node.visual_bounds() else {
            return Ok(Histogram::default());
        };
        bounds
    } else {
        GRect::new(x, y, x + w, y + h)
    };
    if viewport.width() <= 0. || viewport.height() <= 0. {
        return Ok(Histogram::default());
    }
    let scale = (512. / viewport.width().max(viewport.height())).min(1.);
    let render = RenderRequest {
        viewport,
        width: (viewport.width() * scale).ceil().max(1.) as u32,
        height: (viewport.height() * scale).ceil().max(1.) as u32,
        background: [0; 4],
    };
    let mut pixels = CpuRenderer::default()
        .render_cancellable(&scene, render, cancelled)
        .map_err(|e| {
            if cancelled.is_cancelled() {
                JobFailure::Cancelled
            } else {
                JobFailure::Failed(e.to_string())
            }
        })?;
    request
        .source
        .composite_draft(&mut pixels, render, cancelled)
        .map_err(|e| JobFailure::Failed(e.to_string()))?;
    if cancelled.is_cancelled() {
        return Err(JobFailure::Cancelled);
    }
    Ok(Histogram::from_pixels(&pixels))
}
pub struct HistogramController {
    executor: JobExecutor,
    desired: Option<HistogramRequest>,
    pending: Option<JobHandle<Histogram>>,
    latest: Option<Arc<Histogram>>,
    failure: Option<JobFailure>,
}
impl HistogramController {
    pub fn new() -> Result<Self, JobFailure> {
        Ok(Self {
            executor: JobExecutor::new(1, 1, JobManager::new())?,
            desired: None,
            pending: None,
            latest: None,
            failure: None,
        })
    }
    pub fn request(&mut self, desired: Option<HistogramRequest>) {
        if self.desired == desired {
            return;
        }
        self.pending = None;
        self.latest = None;
        self.failure = None;
        self.desired = desired;
        self.schedule();
    }
    fn schedule(&mut self) {
        if self.pending.is_some() || self.latest.is_some() || self.failure.is_some() {
            return;
        }
        let Some(request) = self.desired.clone() else {
            return;
        };
        match self
            .executor
            .submit("histogram", request.source.revision(), move |context| {
                analyze(&request, context.cancellation())
            }) {
            Ok(handle) => self.pending = Some(handle),
            Err(JobFailure::QueueFull) => {}
            Err(error) => self.failure = Some(error),
        }
    }
    pub fn poll(&mut self) {
        let result = self
            .pending
            .as_ref()
            .map(|job| job.try_result(self.desired.as_ref().map_or(0, |r| r.source.revision())));
        match result {
            Some(Ok(Some(histogram))) => {
                self.pending = None;
                self.latest = Some(Arc::new(histogram));
            }
            Some(Err(error)) => {
                self.pending = None;
                self.failure = Some(error);
            }
            _ => {}
        }
        self.schedule();
    }
    pub fn result(&self) -> Option<Arc<Histogram>> {
        self.latest.clone()
    }
    pub fn failure(&self) -> Option<&JobFailure> {
        self.failure.as_ref()
    }
    pub fn is_pending(&self) -> bool {
        self.pending.is_some()
            || (self.desired.is_some() && self.latest.is_none() && self.failure.is_none())
    }
}
