//! RenderBackend trait and rendering context abstraction.

use crate::error::Result;
use petunia_core::{ColorRgba, Rect, SceneGraph};

/// Render options and quality settings.
#[derive(Debug, Clone, Copy)]
pub struct RenderOptions {
    pub scale_factor: f64,
    pub antialias: bool,
    pub background_color: ColorRgba,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            scale_factor: 1.0,
            antialias: true,
            background_color: ColorRgba::WHITE,
        }
    }
}

/// Abstract contract for any visual renderer (GPU, CPU, headless).
pub trait RenderBackend: Send + Sync {
    /// Returns the human-readable identifier of the backend (e.g. "Software-Reference", "Vello-GPU").
    fn name(&self) -> &str;

    /// Renders an entire scene graph within the viewport bounds.
    fn render_scene(
        &mut self,
        scene: &SceneGraph,
        viewport: Rect,
        options: &RenderOptions,
    ) -> Result<()>;
}
