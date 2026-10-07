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

/// Strongly-typed identifier for documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocumentId(Uuid);

impl DocumentId {
    #[must_use]
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for DocumentId {
    fn default() -> Self {
        Self::new_v4()
    }
}

impl fmt::Display for DocumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
}
