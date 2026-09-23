#![forbid(unsafe_code)]

//! Rebuildable render scene: `petunia_design_scene` MVP cut lives here until the
//! split criteria (heavy GPU deps) justify a separate crate.
//!
//! Export adapters read the document or this scene, never viewport pixels.
//! GPU backends (Vello/wgpu) are `POST_V1` behind [`RenderBackend`].

pub mod blend;
pub mod composition;
pub mod pixel_compositor;
mod scene;
pub mod surface_planner;

pub use blend::BlendMode;
pub use composition::{EffectiveContext, IsolationGroup};
pub use pixel_compositor::{PixelBufferRgba16, PixelBufferRgba8, SoftwarePixelCompositor};
pub use scene::{HeadlessSummaryBackend, RenderBackend, Scene, SceneFragment};
pub use surface_planner::{
    IntermediateSurfacePlanner, PlannedSurface, SurfaceAllocationError, SurfaceFormat,
};
