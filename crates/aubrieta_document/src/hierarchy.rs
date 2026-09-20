//! Canonical Object Hierarchy, One-Tree containers, and clipping masks (10.5).
//!
//! Under the One-Tree invariant, UI "Layers" and groups share the single canonical
//! object tree. A Layer is a semantic container role (`ContainerRole::Layer`),
//! clipping is established via `ContainerRole::ClipGroup`, and no separate parallel
//! storage tree exists.

use aubrieta_foundation::ObjectId;
use serde::{Deserialize, Serialize};

/// Structural role of a container object in the One-Tree hierarchy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContainerRole {
    /// Standard structural container.
    Group,
    /// Named organizational layer container.
    Layer,
    /// Clipping container where content is clipped to a designated mask shape.
    ClipGroup,
}

impl ContainerRole {
    /// Returns true if this container role operates as a clipping group.
    #[must_use]
    pub const fn is_clip_group(&self) -> bool {
        matches!(self, Self::ClipGroup)
    }

    /// Returns true if this container role is an organizational layer.
    #[must_use]
    pub const fn is_layer(&self) -> bool {
        matches!(self, Self::Layer)
    }
}

/// Mask compositing mode for vector or raster masks (10.5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MaskMode {
    /// Standard vector path clipping boundary.
    #[default]
    Vector,
    /// Alpha channel luminance / transparency mask.
    Alpha,
    /// Luminance based clipping mask.
    Luminance,
}

/// Validation result for hierarchy reparenting and cyclic sanity checks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HierarchyValidation {
    /// Operation is structurally valid.
    Valid,
    /// Cycle detected (cannot parent an object under itself or its descendants).
    CycleDetected {
        /// Source object ID.
        source: ObjectId,
        /// Target parent ID that causes cycle.
        target_parent: ObjectId,
    },
    /// Target parent container does not exist.
    TargetNotFound(ObjectId),
    /// Target object is locked.
    TargetLocked(ObjectId),
}
