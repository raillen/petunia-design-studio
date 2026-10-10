//! Deterministic and UUID-based identifiers for Petunia entities.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Strongly-typed identifier for objects in the scene graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ObjectId(Uuid);

impl ObjectId {
    /// Generates a new random (v4) unique identifier.
    #[must_use]
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates an identifier from an existing UUID.
    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the underlying UUID.
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for ObjectId {
    fn default() -> Self {
        Self::new_v4()
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Defines a UUID-backed entity identifier with the standard
/// Petunia API (`new_v4`, `from_uuid`, `as_uuid`, `Display`).
macro_rules! entity_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Generates a new random (v4) unique identifier.
            #[must_use]
            pub fn new_v4() -> Self {
                Self(Uuid::new_v4())
            }

            /// Creates an identifier from an existing UUID.
            #[must_use]
            pub const fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            /// Returns the underlying UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new_v4()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

entity_id!(
    /// Strongly-typed identifier for documents.
    DocumentId
);
entity_id!(
    /// Page identity inside a document.
    PageId
);
entity_id!(
    /// Spread identity for editorial page arrangements.
    SpreadId
);
entity_id!(
    /// Persistent guide identity.
    GuideId
);
entity_id!(
    /// Persistent grid definition identity.
    GridId
);
entity_id!(
    /// Export slice identity.
    SliceId
);
entity_id!(
    /// Shared style identity (appearance, character or paragraph).
    StyleId
);
entity_id!(
    /// Symbol definition identity.
    SymbolId
);
entity_id!(
    /// Logical resource identity (bytes live in resource storage).
    ResourceId
);
entity_id!(
    /// Reusable swatch identity.
    SwatchId
);
entity_id!(
    /// Spot ink identity, independent of its alternate preview.
    SpotColorId
);
entity_id!(
    /// Persistent appearance item identity for reorder and overrides.
    AppearanceItemId
);
entity_id!(
    /// Stable effect instance identity for reorder, history and cache.
    EffectId
);
entity_id!(
    /// Stable contour identity inside a `VectorPath`.
    ContourId
);
entity_id!(
    /// Stable geometric node identity inside a contour.
    ///
    /// This UUID identifies the same anchor across moves, handle edits,
    /// splits and undo. It is distinct from UI-local index coordinates.
    NodeId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_id_uniqueness_and_serialization() {
        let id1 = ObjectId::new_v4();
        let id2 = ObjectId::new_v4();
        assert_ne!(id1, id2);

        let json = serde_json::to_string(&id1).expect("serialization succeeds");
        let deserialized: ObjectId = serde_json::from_str(&json).expect("deserialization succeeds");
        assert_eq!(id1, deserialized);
    }

    #[test]
    fn test_generated_ids_share_uuid_api() {
        let style = StyleId::new_v4();
        assert_eq!(StyleId::from_uuid(style.as_uuid()), style);
        assert!(!style.to_string().is_empty());

        let page = PageId::new_v4();
        let json = serde_json::to_string(&page).expect("serialization succeeds");
        let back: PageId = serde_json::from_str(&json).expect("deserialization succeeds");
        assert_eq!(page, back);
    }
}
