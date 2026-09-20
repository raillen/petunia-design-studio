#![forbid(unsafe_code)]

//! Foundation contracts: typed stable IDs, diagnostics, schema version.
//!
//! A `Vec` index, pointer or handle is never identity. Every entity is
//! addressed by a typed stable ID that survives save/reopen.

mod diagnostics;
mod ids;

pub use diagnostics::{AubrietaError, Diagnostic, DiagnosticCode};
pub use ids::{EffectId, IdGenerator, ObjectId, ResourceId, StyleId, SurfaceId, TextStoryId};

/// Canonical native schema version emitted by this workspace build.
pub const NATIVE_SCHEMA_VERSION: u32 = 1;
