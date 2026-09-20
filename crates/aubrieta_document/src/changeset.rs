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
    /// An object's vector shape changed.
    ShapeChanged {
        id: ObjectId,
        previous: Option<crate::ShapeKind>,
        next: Option<crate::ShapeKind>,
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
    /// An object's parent in the canonical tree changed.
    Reparented {
        id: ObjectId,
        previous_parent: Option<ObjectId>,
        next_parent: Option<ObjectId>,
        previous_index: usize,
        next_index: usize,
    },
    /// A container object's children list changed.
    ChildrenChanged {
        id: ObjectId,
        previous_children: Vec<ObjectId>,
        next_children: Vec<ObjectId>,
    },
    /// A container object's structural role changed.
    ContainerRoleChanged {
        id: ObjectId,
        previous: Option<crate::hierarchy::ContainerRole>,
        next: Option<crate::hierarchy::ContainerRole>,
    },
    /// An object's clip mask relationship changed.
    ClipMaskChanged {
        id: ObjectId,
        previous_mask: Option<ObjectId>,
        next_mask: Option<ObjectId>,
        previous_is_mask: bool,
        next_is_mask: bool,
    },
    /// A surface's origin or dimensions changed (10.7).
    SurfaceGeometryChanged {
        id: SurfaceId,
        previous_origin: [f64; 2],
        next_origin: [f64; 2],
        previous_dimensions: [f64; 2],
        next_dimensions: [f64; 2],
    },
    /// A surface's bleed configuration changed.
    SurfaceBleedChanged {
        id: SurfaceId,
        previous: crate::surface_metadata::Bleed,
        next: crate::surface_metadata::Bleed,
    },
    /// A surface's margins changed.
    SurfaceMarginsChanged {
        id: SurfaceId,
        previous: crate::surface_metadata::Margins,
        next: crate::surface_metadata::Margins,
    },
    /// A surface's background color changed.
    SurfaceBackgroundChanged {
        id: SurfaceId,
        previous: Option<String>,
        next: Option<String>,
    },
    /// A layout guide was added to a surface.
    SurfaceGuideAdded {
        surface: SurfaceId,
        guide: crate::surface_metadata::Guide,
    },
    /// A layout guide was removed from a surface.
    SurfaceGuideRemoved {
        surface: SurfaceId,
        guide: crate::surface_metadata::Guide,
    },
    /// A variable data source was added (10.11).
    DataSourceAdded {
        source: crate::variable_data::DataSourceDefinition,
    },
    /// A variable data source was removed (10.11).
    DataSourceRemoved {
        source: crate::variable_data::DataSourceDefinition,
    },
    /// A data binding was added (10.11).
    DataBindingAdded {
        binding: crate::variable_data::DataBinding,
    },
    /// A data binding was removed (10.11).
    DataBindingRemoved {
        binding: crate::variable_data::DataBinding,
    },
    /// Batch surfaces were materialized from data merge (10.11).
    BatchSurfacesAdded {
        surfaces: Vec<crate::document::Surface>,
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

    /// Appends all changes from another change set.
    pub fn extend(&mut self, other: ChangeSet) {
        self.changes.extend(other.changes);
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
