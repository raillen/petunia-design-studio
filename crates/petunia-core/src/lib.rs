//! # petunia-core
//!
//! Core domain entities, vector geometry, scene graph, and serialization format
//! for the Petunia Design Studio editor.
//!
//! Strict invariants:
//! - `#![forbid(unsafe_code)]`
//! - Zero external GUI/GPU dependencies.
//! - Deterministic mathematical logic and serializable schemas.

#![forbid(unsafe_code)]

pub mod color;
pub mod document;
pub mod error;
pub mod id;
pub mod math;
pub mod path;
pub mod scene;

pub use color::{ColorRgba, ColorSpace};
pub use document::{Document, DocumentSetup};
pub use error::{CoreError, Result};
pub use id::{DocumentId, ObjectId};
pub use math::{Point, Rect, Transform2D, Vec2};
pub use path::{Contour, FillRule, NodeKind, PathNode, VectorPath};
pub use scene::{Fill, SceneGraph, SceneItem, SceneNode, Stroke};
