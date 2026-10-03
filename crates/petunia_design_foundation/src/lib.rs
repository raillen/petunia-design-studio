#![forbid(unsafe_code)]

//! Foundation contracts: typed stable IDs, diagnostics, schema version.
//!
//! A `Vec` index, pointer or handle is never identity. Every entity is
//! addressed by a typed stable ID that survives save/reopen.

mod diagnostics;
mod ids;
mod namespace;
pub mod numeric;

pub use diagnostics::{Diagnostic, DiagnosticCode, PetuniaError};
pub use ids::{EffectId, IdGenerator, ObjectId, ResourceId, StyleId, SurfaceId, TextStoryId};
pub use namespace::{
    is_canonical_action_id, is_current_namespace, normalize_action_id, normalize_legacy_namespace,
    normalized, LEGACY_NAMESPACE, NAMESPACE,
};
pub use numeric::{parse_numeric_input, NumericFieldKind, NumericParseError};

/// Canonical native schema version emitted by this workspace build.
pub const NATIVE_SCHEMA_VERSION: u32 = 5;
