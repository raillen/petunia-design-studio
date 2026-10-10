//! Top-level Petunia document representation and PTND serializable payload.

use crate::error::Result;
use crate::id::DocumentId;
use crate::scene::SceneGraph;
use serde::{Deserialize, Serialize};

/// Document canvas setup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSetup {
    pub width: f64,
    pub height: f64,
    pub dpi: f64,
}

impl Default for DocumentSetup {
    fn default() -> Self {
        Self {
            width: 1920.0,
            height: 1080.0,
            dpi: 72.0,
        }
    }
}

/// In-memory document model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub title: String,
    pub setup: DocumentSetup,
    pub scene: SceneGraph,
}

impl Document {
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: DocumentId::new_v4(),
            title: title.into(),
            setup: DocumentSetup::default(),
            scene: SceneGraph::new(),
        }
    }

    /// Serializes document into canonical JSON string.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Deserializes document from canonical JSON string.
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::VectorPath;
    use crate::scene::SceneNode;

    #[test]
    fn test_document_roundtrip_json() {
        let mut doc = Document::new("My Poster");
        let page = doc.scene.default_page();
        let node = SceneNode::new_path(
            "Rectangle",
            VectorPath::rect(10.0, 10.0, 200.0, 100.0),
            crate::scene::ParentRef::Page(page),
        );
        doc.scene.insert_node(node);

        let json = doc.to_json().expect("to_json succeeds");
        let restored = Document::from_json(&json).expect("from_json succeeds");

        assert_eq!(doc.id, restored.id);
        assert_eq!(doc.title, restored.title);
        assert_eq!(doc.scene.len(), restored.scene.len());
    }
}
