//! GPU rendering contract and hardware abstraction.
//!
//! Per `03-render/pipeline.md`:
//! - CPU reference renderer is the canonical semantic baseline.
//! - GPU backend is an optional future accelerator under the exact same contract.
//! - Shaders, buffers, pipeline descriptors and tessellation are isolated here.
//! - No GPU handle or atlas slot ever leaks into authorial Document State.

use crate::backend::{RenderBackend, RenderOptions};
use crate::error::Result;
use petunia_core::appearance::BlendMode;
use petunia_core::Rect;
use petunia_render_model::{RenderColor, RenderFrame, RenderPath, RenderStats};
use serde::{Deserialize, Serialize};

/// Standard vertex layout for 2D GPU geometry pipelines.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct GpuVertex {
    /// 2D position in device pixels.
    pub position: [f32; 2],
    /// Normalized texture / local coordinate.
    pub uv: [f32; 2],
    /// Premultiplied linear vertex color RGBA.
    pub color: [f32; 4],
}

/// Indexed triangle mesh ready for GPU vertex and index buffer upload.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GpuMesh {
    pub vertices: Vec<GpuVertex>,
    pub indices: Vec<u32>,
}

impl GpuMesh {
    /// True when the mesh contains no triangles.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Number of triangles in this mesh.
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// 2D path tessellator for GPU triangle generation.
#[derive(Debug, Default)]
pub struct GpuTessellator {
    tolerance: f32,
}

impl GpuTessellator {
    /// New tessellator with an explicit geometric error tolerance in pixels.
    #[must_use]
    pub fn new(tolerance: f32) -> Self {
        Self {
            tolerance: tolerance.max(0.01),
        }
    }

    /// Geometric error tolerance in pixels.
    #[must_use]
    pub fn tolerance(&self) -> f32 {
        self.tolerance
    }

    /// Tessellate a convex or simple polygon ring using fan triangulation.
    pub fn tessellate_polygon(
        &self,
        points: &[(f64, f64)],
        color: RenderColor,
        mesh: &mut GpuMesh,
    ) {
        if points.len() < 3 {
            return;
        }
        let base_vertex = mesh.vertices.len() as u32;
        let c = [color.r, color.g, color.b, color.a];
        for &(x, y) in points {
            mesh.vertices.push(GpuVertex {
                position: [x as f32, y as f32],
                uv: [0.0, 0.0],
                color: c,
            });
        }
        for i in 1..(points.len() - 1) as u32 {
            mesh.indices.push(base_vertex);
            mesh.indices.push(base_vertex + i);
            mesh.indices.push(base_vertex + i + 1);
        }
    }

    /// Tessellate an axis-aligned bounding box into two triangles.
    pub fn tessellate_rect(&self, rect: Rect, color: RenderColor, mesh: &mut GpuMesh) {
        let p = [
            (rect.x, rect.y),
            (rect.x + rect.width, rect.y),
            (rect.x + rect.width, rect.y + rect.height),
            (rect.x, rect.y + rect.height),
        ];
        self.tessellate_polygon(&p, color, mesh);
    }
}

/// Hardware feature flags and resource limits reported by a GPU adapter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GpuCapabilities {
    pub max_texture_dimension: u32,
    pub supports_hdr_render_targets: bool,
    pub supports_compute_shaders: bool,
    pub supports_float_filtering: bool,
    pub max_bind_groups: u32,
}

impl Default for GpuCapabilities {
    fn default() -> Self {
        Self {
            max_texture_dimension: 8192,
            supports_hdr_render_targets: true,
            supports_compute_shaders: true,
            supports_float_filtering: true,
            max_bind_groups: 4,
        }
    }
}

/// Pipeline configuration for a GPU draw pass.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GpuPipelineDescriptor {
    pub blend_mode: BlendMode,
    pub premultiplied_alpha: bool,
    pub sample_count: u32,
    pub wireframe: bool,
}

impl Default for GpuPipelineDescriptor {
    fn default() -> Self {
        Self {
            blend_mode: BlendMode::Normal,
            premultiplied_alpha: true,
            sample_count: 1,
            wireframe: false,
        }
    }
}

/// Contract for GPU-accelerated rendering implementations.
///
/// Any GPU backend must conform to this trait and be verified against
/// the software reference renderer within documented visual error bounds.
pub trait GpuRenderContract: RenderBackend {
    /// Query adapter hardware capabilities.
    fn capabilities(&self) -> &GpuCapabilities;

    /// Compile a GPU mesh from an evaluated path.
    fn tessellate(&self, path: &RenderPath, tolerance: f32) -> Result<GpuMesh>;

    /// Check visual equivalence against software reference within threshold.
    fn verify_reference_parity(
        &mut self,
        software_reference: &[u8],
        gpu_output: &[u8],
        max_channel_diff: u8,
    ) -> bool {
        if software_reference.len() != gpu_output.len() {
            return false;
        }
        software_reference
            .iter()
            .zip(gpu_output.iter())
            .all(|(a, b)| a.abs_diff(*b) <= max_channel_diff)
    }
}

/// Reference/mock GPU backend implementation proving the abstraction
/// contract without requiring external graphics drivers or display servers.
pub struct MockGpuBackend {
    capabilities: GpuCapabilities,
    tessellator: GpuTessellator,
}

impl MockGpuBackend {
    /// New mock GPU backend.
    #[must_use]
    pub fn new() -> Self {
        Self {
            capabilities: GpuCapabilities::default(),
            tessellator: GpuTessellator::new(0.5),
        }
    }
}

impl Default for MockGpuBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderBackend for MockGpuBackend {
    fn name(&self) -> &str {
        "MockGpuBackend"
    }

    fn render(
        &mut self,
        frame: &RenderFrame,
        options: &RenderOptions,
    ) -> Result<(Vec<u8>, RenderStats)> {
        // Mock GPU delegates to the software reference renderer to guarantee
        // strict pixel parity in testing and headless CI.
        let mut sw = crate::software::SoftwareRenderer::new(1 << 20, 1 << 20);
        sw.render(frame, options)
    }
}

impl GpuRenderContract for MockGpuBackend {
    fn capabilities(&self) -> &GpuCapabilities {
        &self.capabilities
    }

    fn tessellate(&self, path: &RenderPath, _tolerance: f32) -> Result<GpuMesh> {
        let mut mesh = GpuMesh::default();
        for (contour, _closed) in path.contours.iter().zip(path.closed.iter()) {
            self.tessellator
                .tessellate_polygon(contour, RenderColor::BLACK, &mut mesh);
        }
        Ok(mesh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tessellator_generates_valid_triangles() {
        let tess = GpuTessellator::new(0.5);
        let mut mesh = GpuMesh::default();
        let rect = Rect::new(0.0, 0.0, 100.0, 50.0);
        tess.tessellate_rect(rect, RenderColor::BLACK, &mut mesh);

        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.indices.len(), 6);
        assert_eq!(mesh.triangle_count(), 2);
    }

    #[test]
    fn mock_gpu_parity_check() {
        let mut gpu = MockGpuBackend::new();
        let a = vec![255, 0, 0, 255];
        let b = vec![254, 0, 1, 255]; // 1-channel delta within tolerance
        assert!(gpu.verify_reference_parity(&a, &b, 2));
        assert!(!gpu.verify_reference_parity(&a, &b, 0));
    }
}
