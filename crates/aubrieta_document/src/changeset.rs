//! Explicit change records. Every mutation emits a [`ChangeSet`].

use aubrieta_foundation::{ObjectId, SurfaceId};
use serde::{Deserialize, Serialize};

use crate::document_object::DocumentObject;

/// Single auditable change.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Change {
    /// A surface was added.
    SurfaceAdded { id: SurfaceId, name: String },
    /// An object was added to a surface.
    ObjectAdded {
        surface: SurfaceId,
        object: DocumentObject,
    },
    /// An object was removed; the full object is kept for undo.
    ObjectRemoved {
        surface: SurfaceId,
        object: DocumentObject,
    },
    /// An object's fill token changed; previous value kept for undo.
    FillChanged {
        id: ObjectId,
        previous: Option<String>,
        next: Option<String>,
    },
    /// An object's visibility flag changed.
    VisibilityChanged {
        id: ObjectId,
        previous: bool,
        next: bool,
    },
    /// An object's locked flag changed.
    LockChanged {
        id: ObjectId,
        previous: bool,
        next: bool,
    },
    /// An object's opacity changed.
    OpacityChanged {
        id: ObjectId,
        previous: f64,
        next: f64,
    },
    /// An object's stroke changed.
    StrokeChanged {
        id: ObjectId,
        previous_stroke: Option<String>,
        next_stroke: Option<String>,
        previous_width: f64,
        next_width: f64,
    },
    /// An object's bounds or rotation changed.
    BoundsChanged {
        id: ObjectId,
        previous_bounds: Option<[f64; 4]>,
        next_bounds: Option<[f64; 4]>,
        previous_rotation: f64,
        next_rotation: f64,
    },
    /// An object was reordered within its surface.
    ObjectReordered {
        surface: SurfaceId,
        id: ObjectId,
        previous_index: usize,
        next_index: usize,
    },
    /// An object's appearance stack changed.
    AppearanceChanged {
        id: ObjectId,
        previous: Option<crate::appearance::AppearanceStack>,
        next: Option<crate::appearance::AppearanceStack>,
    },
}

/// Ordered list of changes produced by one mutation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChangeSet {
    /// Changes in application order.
    pub changes: Vec<Change>,
}

impl ChangeSet {
    /// Creates an empty change set.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    /// Records one change.
    pub fn push(&mut self, change: Change) {
        self.changes.push(change);
    }

    /// True when the mutation produced no observable change.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Number of recorded changes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.changes.len()
    }
}
