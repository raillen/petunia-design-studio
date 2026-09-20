//! Document and surface: one tree, stable IDs, JSON persistence.

use aubrieta_foundation::{AubrietaError, ObjectId, SurfaceId, NATIVE_SCHEMA_VERSION};
use serde::{Deserialize, Serialize};

use crate::document_object::DocumentObject;

/// Drawing surface (artboard/page/export region depending on context).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Surface {
    /// Stable identity.
    pub id: SurfaceId,
    /// Human label. Not identity.
    pub name: String,
    /// Objects owned by this surface, in z-order.
    pub objects: Vec<DocumentObject>,
}

impl Surface {
    /// Creates an empty surface with an explicit stable ID.
    #[must_use]
    pub fn new(id: SurfaceId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            objects: Vec::new(),
        }
    }
}

/// Canonical document: owns surfaces; objects live inside surfaces.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// Schema version of this payload.
    pub schema_version: u32,
    /// Surfaces in document order.
    pub surfaces: Vec<Surface>,
}

impl Document {
    /// Creates an empty document at the current native schema version.
    #[must_use]
    pub fn new() -> Self {
        Self {
            schema_version: NATIVE_SCHEMA_VERSION,
            surfaces: Vec::new(),
        }
    }

    /// Finds a surface by stable ID.
    pub fn surface(&self, id: SurfaceId) -> Result<&Surface, AubrietaError> {
        self.surfaces
            .iter()
            .find(|s| s.id == id)
            .ok_or_else(|| AubrietaError::not_found(format!("surface `{id}` does not exist")))
    }

    /// Finds a mutable surface by stable ID.
    pub fn surface_mut(&mut self, id: SurfaceId) -> Result<&mut Surface, AubrietaError> {
        self.surfaces
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or_else(|| AubrietaError::not_found(format!("surface `{id}` does not exist")))
    }

    /// Finds an object anywhere in the document by stable ID.
    #[must_use]
    pub fn find_object(&self, id: ObjectId) -> Option<&DocumentObject> {
        self.surfaces
            .iter()
            .flat_map(|s| s.objects.iter())
            .find(|o| o.id == id)
    }

    /// Serializes the document to canonical JSON.
    pub fn to_json(&self) -> Result<String, AubrietaError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| AubrietaError::io(format!("serialize document: {e}")))
    }

    /// Parses a document from canonical JSON, rejecting unknown schemas.
    pub fn from_json(text: &str) -> Result<Self, AubrietaError> {
        let doc: Self = serde_json::from_str(text)
            .map_err(|e| AubrietaError::io(format!("parse document: {e}")))?;
        if doc.schema_version != NATIVE_SCHEMA_VERSION {
            return Err(AubrietaError::invalid_input(format!(
                "unsupported schema {}, expected {}",
                doc.schema_version, NATIVE_SCHEMA_VERSION
            )));
        }
        Ok(doc)
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aubrieta_foundation::IdGenerator;

    #[test]
    fn save_reopen_roundtrip_preserves_ids() {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let surface_id = gen.next_surface();
        let mut surface = Surface::new(surface_id, "Page 1");
        surface
            .objects
            .push(DocumentObject::new(gen.next_object(), "Rect"));
        doc.surfaces.push(surface);

        let json = doc.to_json().expect("serialize");
        let reopened = Document::from_json(&json).expect("reopen");
        assert_eq!(doc, reopened);
    }

    #[test]
    fn unknown_schema_is_rejected() {
        let bad = r#"{"schema_version":999,"surfaces":[]}"#;
        assert!(Document::from_json(bad).is_err());
    }
}
