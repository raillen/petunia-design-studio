//! Document and surface: one tree, stable IDs, JSON persistence.

use aubrieta_foundation::{AubrietaError, ObjectId, SurfaceId, NATIVE_SCHEMA_VERSION};
use serde::{Deserialize, Serialize};

use crate::document_object::DocumentObject;

fn default_dimensions() -> [f64; 2] {
    [800.0, 600.0]
}

fn default_surface_bg() -> Option<String> {
    Some("aubrieta.white".to_string())
}

fn default_true() -> bool {
    true
}

/// Drawing surface (artboard/page/export region depending on context) (10.7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Surface {
    /// Stable identity.
    pub id: SurfaceId,
    /// Human label. Not identity.
    pub name: String,
    /// Objects owned by this surface, in z-order.
    pub objects: Vec<DocumentObject>,
    /// Global origin coordinate [x, y] in document pasteboard points.
    #[serde(default)]
    pub origin: [f64; 2],
    /// Width and height [w, h] in document points.
    #[serde(default = "default_dimensions")]
    pub dimensions: [f64; 2],
    /// Background color token reference (e.g. `aubrieta.white`) or hex string. None = transparent.
    #[serde(default = "default_surface_bg")]
    pub background: Option<String>,
    /// Per-side bleed configuration.
    #[serde(default)]
    pub bleed: crate::surface_metadata::Bleed,
    /// Per-side inner layout margins.
    #[serde(default)]
    pub margins: crate::surface_metadata::Margins,
    /// Ordered layout guides attached to this surface.
    #[serde(default)]
    pub guides: Vec<crate::surface_metadata::Guide>,
    /// Whether this surface is included in batch export operations.
    #[serde(default = "default_true")]
    pub export_enabled: bool,
}

impl Surface {
    /// Creates an empty surface with an explicit stable ID and default 800x600 dimensions.
    #[must_use]
    pub fn new(id: SurfaceId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            objects: Vec::new(),
            origin: [0.0, 0.0],
            dimensions: [800.0, 600.0],
            background: Some("aubrieta.white".to_string()),
            bleed: crate::surface_metadata::Bleed::ZERO,
            margins: crate::surface_metadata::Margins::ZERO,
            guides: Vec::new(),
            export_enabled: true,
        }
    }

    /// Axis-aligned bounds `[x, y, w, h]` of the surface in global document points.
    #[must_use]
    pub fn bounds(&self) -> [f64; 4] {
        [
            self.origin[0],
            self.origin[1],
            self.dimensions[0],
            self.dimensions[1],
        ]
    }

    /// Extended bounds `[x, y, w, h]` including bleed area.
    #[must_use]
    pub fn bleed_bounds(&self) -> [f64; 4] {
        [
            self.origin[0] - self.bleed.left,
            self.origin[1] - self.bleed.top,
            self.dimensions[0] + self.bleed.left + self.bleed.right,
            self.dimensions[1] + self.bleed.top + self.bleed.bottom,
        ]
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

    /// Finds a mutable object anywhere in the document by stable ID.
    pub fn find_object_mut(&mut self, id: ObjectId) -> Option<&mut DocumentObject> {
        self.surfaces
            .iter_mut()
            .flat_map(|s| s.objects.iter_mut())
            .find(|o| o.id == id)
    }

    /// Finds the SurfaceId containing an object.
    #[must_use]
    pub fn find_object_surface(&self, id: ObjectId) -> Option<SurfaceId> {
        for s in &self.surfaces {
            if s.objects.iter().any(|o| o.id == id) {
                return Some(s.id);
            }
        }
        None
    }

    /// Checks if `candidate` is a descendant of `ancestor` (or is the ancestor itself).
    #[must_use]
    pub fn is_descendant(&self, candidate: ObjectId, ancestor: ObjectId) -> bool {
        if candidate == ancestor {
            return true;
        }
        let mut curr = Some(candidate);
        while let Some(c) = curr {
            if let Some(obj) = self.find_object(c) {
                if let Some(p) = obj.parent {
                    if p == ancestor {
                        return true;
                    }
                    curr = Some(p);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        false
    }

    /// Computes the accumulated world affine transform from root to this object.
    pub fn world_transform(
        &self,
        id: ObjectId,
    ) -> Result<aubrieta_geometry::GAffine, AubrietaError> {
        let mut chain = Vec::new();
        let mut curr = Some(id);
        while let Some(c) = curr {
            let obj = self
                .find_object(c)
                .ok_or_else(|| AubrietaError::not_found(format!("object `{c}` not found")))?;
            chain.push(obj.local_transform());
            curr = obj.parent;
        }

        let mut acc = aubrieta_geometry::GAffine::IDENTITY;
        for local in chain.into_iter().rev() {
            acc = acc.after(local);
        }
        Ok(acc)
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
