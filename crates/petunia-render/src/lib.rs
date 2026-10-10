//! # petunia-render
//!
//! Software tiled renderer, compositor, paint evaluation and output
//! for the Petunia Design Studio editor.
//!
//! Strict invariants:
//! - `#![forbid(unsafe_code)]`
//! - Consumes `petunia-render-model` snapshots, never the authorial scene.
//! - No dependency on engine, UI or GPU code.
//! - Deterministic reference behavior for tests and export.

#![forbid(unsafe_code)]

pub mod adjustments;
pub mod backend;
pub mod cache;
pub mod compositor;
pub mod error;
pub mod gpu;
pub mod graph;
pub mod output;
pub mod overlays;
pub mod paint_eval;
pub mod rasterize;
pub mod software;

pub use adjustments::{apply_adjustment, StraightPixel};
pub use backend::{RenderBackend, RenderOptions};
pub use cache::{ClockCache, RebuildCost, SurfacePool};
pub use compositor::{apply_blend, composite, Pixel};
pub use error::{RenderError, Result};
pub use gpu::{
    GpuCapabilities, GpuMesh, GpuPipelineDescriptor, GpuRenderContract, GpuTessellator, GpuVertex,
    MockGpuBackend,
};
pub use graph::{RenderGraph, TILE_EDGE};
pub use output::{
    encode_png_rgba8, frame_to_rgba8, frame_to_rgba8_mapped, linear_to_srgb_byte, ToneMapping,
};
pub use overlays::{OverlayPrimitive, OverlayTarget};
pub use paint_eval::{
    sample_conical, sample_gradient, sample_linear, sample_pattern, sample_radial,
};
pub use rasterize::{fill_path, stroke_path, Target, SAMPLES_PER_AXIS};
pub use software::SoftwareRenderer;
