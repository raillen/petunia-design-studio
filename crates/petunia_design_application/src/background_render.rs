//! Application-owned scheduling of immutable render snapshots. UI owners poll
//! the handle with their current document revision before presenting pixels.
use petunia_design_jobs::{JobExecutor, JobFailure, JobHandle};
use petunia_design_render::{
    CpuRenderer, PixelBufferRgba8, RenderError, RenderRequest, RenderSurface,
};

/// Schedules real CPU rendering without granting workers mutable document access.
/// Revision checks and cancellation suppress obsolete presentation results.
pub fn schedule_surface_render(
    executor: &JobExecutor,
    snapshot: RenderSurface,
    source_revision: u64,
    request: RenderRequest,
) -> Result<JobHandle<PixelBufferRgba8>, JobFailure> {
    executor.submit("surface-render", source_revision, move |context| {
        context.check_cancelled()?;
        context.report_progress(5);
        let pixels = CpuRenderer::default()
            .render_cancellable(&snapshot, request, context.cancellation())
            .map_err(|error| match error {
                RenderError::Cancelled => JobFailure::Cancelled,
                error => JobFailure::Failed(error.to_string()),
            })?;
        context.check_cancelled()?;
        context.report_progress(100);
        Ok(pixels)
    })
}
