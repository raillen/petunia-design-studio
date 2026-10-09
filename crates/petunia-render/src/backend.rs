//! Vendor-agnostic rendering contract.
//!
//! Backends consume immutable [`RenderFrame`] snapshots: scene,
//! view, target and options travel together, and every call reports
//! statistics. Options stay backend-side so the snapshot contract
//! never depends on renderer code.

use crate::error::Result;
use petunia_core::ColorRgba;
use petunia_render_model::{RenderFrame, RenderQuality, RenderStats};

/// Backend render options: quality, DPR policy, antialiasing and the
/// clear color. Geometry never changes with these knobs.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub quality: RenderQuality,
    pub dpr: f64,
    pub antialias: bool,
    pub background: ColorRgba,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            quality: RenderQuality::Authoring,
            dpr: 1.0,
            antialias: true,
            background: ColorRgba::WHITE,
        }
    }
}

/// Abstract contract for any visual renderer (software, GPU,
/// headless). Implementations rasterize the frame and return RGBA8
/// bytes plus statistics.
pub trait RenderBackend: Send + Sync {
    /// Human-readable backend identifier.
    fn name(&self) -> &str;

    /// Render one frame into RGBA8 bytes plus statistics.
    fn render(
        &mut self,
        frame: &RenderFrame,
        options: &RenderOptions,
    ) -> Result<(Vec<u8>, RenderStats)>;
}
