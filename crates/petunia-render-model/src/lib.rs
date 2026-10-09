//! Immutable contract between evaluation and rendering.
//!
//! The engine produces snapshots; backends consume them. This crate
//! holds evaluated data only: no scene graph, no commands, no
//! caches, no GPU handles, no geometry algorithms. It may depend on
//! stable Core types, never on UI, engine or backend code.
//!
//! Strict invariants:
//! - `#![forbid(unsafe_code)]`
//! - Engine produces; render consumes.
//! - Paint order is explicit and deterministic.
//! - Bounds are conservative: extra area allowed, missing pixels are bugs.

#![forbid(unsafe_code)]

pub mod composite;
pub mod effect;
pub mod error;
pub mod image;
pub mod paint;
pub mod primitive;
pub mod resource;
pub mod snapshot;
pub mod text;

pub use composite::{
    IsolationMode, RenderFrame, RenderGroup, RenderStats, RenderTarget, ViewTransform,
};
pub use effect::{
    CurvePoint, LevelsChannel, LevelsChannels, RenderAdjustment, RenderClip, RenderEffect,
    RenderMask, ShadowEffect,
};
pub use error::{CompileError, CompileWarning, Result};
pub use image::{ImagePrimitive, RasterPrimitive};
pub use paint::{
    RenderAppearance, RenderColor, RenderGradient, RenderGradientStop, RenderPaint, RenderStroke,
};
pub use primitive::{RenderPath, RenderPrimitive, VectorPrimitive};
pub use resource::{RenderResourceTable, ResourceEntry};
pub use snapshot::{RenderPage, RenderQuality, RenderSnapshot, SnapshotRevision};
pub use text::{PositionedGlyph, TextPrimitive, TextRun};
