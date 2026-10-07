//! Deterministic software reference renderer for headless execution and unit tests.

use crate::backend::{RenderBackend, RenderOptions};
use crate::error::Result;
use petunia_core::{Rect, SceneGraph};

/// A lightweight reference renderer that counts draw operations without requiring GPU hardware.
#[derive(Debug, Default)]
pub struct SoftwareReferenceRenderer {
    pub draw_calls: usize,
    pub last_viewport: Option<Rect>,
}

impl SoftwareReferenceRenderer {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl RenderBackend for SoftwareReferenceRenderer {
    fn name(&self) -> &str {
        "SoftwareReference"
    }

    fn render_scene(
        &mut self,
        scene: &SceneGraph,
        viewport: Rect,
        _options: &RenderOptions,
    ) -> Result<()> {
        self.last_viewport = Some(viewport);
        self.draw_calls = scene.len();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_core::{SceneNode, VectorPath};

    #[test]
    fn test_software_reference_render_scene() {
        let mut renderer = SoftwareReferenceRenderer::new();
        let mut scene = SceneGraph::new();

        scene.insert_node(SceneNode::new_path("Rect", VectorPath::rect(0.0, 0.0, 50.0, 50.0)));

        let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);
        let options = RenderOptions::default();

        renderer
            .render_scene(&scene, viewport, &options)
            .expect("render succeeds");

        assert_eq!(renderer.draw_calls, 1);
        assert_eq!(renderer.last_viewport, Some(viewport));
    }
}
