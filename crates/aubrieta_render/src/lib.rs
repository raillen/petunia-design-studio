#![forbid(unsafe_code)]

//! Rebuildable render scene: `aubrieta_scene` MVP cut lives here until the
//! split criteria (heavy GPU deps) justify a separate crate.
//!
//! Export adapters read the document or this scene, never viewport pixels.
//! GPU backends (Vello/wgpu) are `POST_V1` behind [`RenderBackend`].

mod scene;

pub use scene::{HeadlessSummaryBackend, RenderBackend, Scene, SceneFragment};
