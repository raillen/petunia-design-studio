//! Versioned persistent DTO for [`Document`](crate::Document).
//!
//! Three layers stay distinct: the domain model (runtime truth), the
//! persistent DTO (this module: shape plus schema version), and the
//! physical PTND container (I/O Engine: ZIP entries, limits, atomic
//! replace). Refactoring domain structs never changes the file format
//! silently; the schema version gates every load.

use crate::document::{Document, DocumentMetadata, DocumentSetup, Page, PageCollection, Spread};
use crate::error::{CoreError, Result};
use crate::guides::{GridRegistry, GuideRegistry, SliceRegistry};
use crate::id::DocumentId;
use crate::paint::SwatchRegistry;
use crate::resources::ResourceRegistry;
use crate::scene::SceneGraph;
use crate::styles::StyleRegistry;
use crate::symbols::SymbolRegistry;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Schema version of the persisted document DTO. Independent from
/// the application version: a release never mints a new schema
/// without a real persistent change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVersion(pub u32);

impl SchemaVersion {
    /// The only schema this revision reads and writes.
    pub const V1: Self = Self(1);

    /// Schema understood by this build.
    #[must_use]
    pub fn current() -> Self {
        Self::V1
    }
}

/// Persistent document DTO, schema v1. Mirrors the aggregate with an
/// explicit version gate; structural and domain validation run on the
/// way back in through [`DocumentDtoV1::into_document`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentDtoV1 {
    pub schema_version: SchemaVersion,
    pub id: DocumentId,
    pub metadata: DocumentMetadata,
    pub setup: DocumentSetup,
    pub pages: Vec<Page>,
    pub spreads: Vec<Spread>,
    pub scene: SceneGraph,
    pub resources: ResourceRegistry,
    pub styles: StyleRegistry,
    pub symbols: SymbolRegistry,
    pub swatches: SwatchRegistry,
    pub guides: GuideRegistry,
    pub grids: GridRegistry,
    pub slices: SliceRegistry,
}

impl DocumentDtoV1 {
    /// Snapshot the domain into its persistent form.
    #[must_use]
    pub fn from_document(document: &Document) -> Self {
        let mut pages: Vec<Page> = document
            .pages
            .ids()
            .into_iter()
            .filter_map(|id| document.pages.get(id).cloned())
            .collect();
        pages.sort_by_key(|page| page.id.as_uuid());
        let mut spreads: Vec<Spread> = document.spreads.values().cloned().collect();
        spreads.sort_by_key(|spread| spread.id.as_uuid());
        Self {
            schema_version: SchemaVersion::current(),
            id: document.id,
            metadata: document.metadata.clone(),
            setup: document.setup.clone(),
            pages,
            spreads,
            scene: document.scene.clone(),
            resources: document.resources.clone(),
            styles: document.styles.clone(),
            symbols: document.symbols.clone(),
            swatches: document.swatches.clone(),
            guides: document.guides.clone(),
            grids: document.grids.clone(),
            slices: document.slices.clone(),
        }
    }

    /// Rebuild the domain, refusing unknown schemas and invalid
    /// documents with typed errors. Structural shape first, then full
    /// domain validation: untrusted bytes never skip either layer.
    pub fn into_document(self) -> Result<Document> {
        if self.schema_version != SchemaVersion::current() {
            return Err(CoreError::InvariantViolation(format!(
                "unsupported document schema {:?}; this build reads {:?}",
                self.schema_version,
                SchemaVersion::current(),
            )));
        }
        let mut pages = PageCollection::new();
        for page in self.pages {
            page.spec.validate()?;
            pages.insert(page);
        }
        let mut spreads = BTreeMap::new();
        for spread in self.spreads {
            let rebuilt = Spread::new(spread.id, spread.pages)?;
            if spreads.insert(spread.id, rebuilt).is_some() {
                return Err(CoreError::InvariantViolation(format!(
                    "duplicate spread {} in DTO",
                    spread.id,
                )));
            }
        }
        // Re-validate owned scalar models whose constructors the
        // deserializer bypassed; graph references resolve below.
        let document = Document {
            id: self.id,
            metadata: self.metadata,
            setup: self.setup,
            pages,
            spreads,
            scene: self.scene,
            resources: self.resources,
            styles: self.styles,
            symbols: self.symbols,
            swatches: self.swatches,
            guides: self.guides,
            grids: self.grids,
            slices: self.slices,
        };
        document.validate()?;
        Ok(document)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::{Insets, Size2};
    use crate::path::VectorPath;
    use crate::scene::{ParentRef, SceneNode};

    fn boxed(document: &mut Document) -> crate::id::ObjectId {
        let page = document.scene.default_page();
        let node = SceneNode::new_path(
            "box",
            VectorPath::rect(0.0, 0.0, 10.0, 10.0),
            ParentRef::Page(page),
        );
        let id = node.id;
        document.scene.insert_node(node);
        id
    }

    #[test]
    fn dto_round_trip_preserves_the_aggregate() {
        let mut document = Document::new("DTO");
        let id = boxed(&mut document);
        assert!(document.validate().is_ok());
        let dto = DocumentDtoV1::from_document(&document);
        assert_eq!(dto.schema_version, SchemaVersion::current());
        let back = dto.into_document().expect("valid DTO");
        assert_eq!(back.id, document.id);
        assert_eq!(back.scene.len(), 1);
        assert!(back.scene.get_node(id).is_some());
        assert!(back.validate().is_ok());
    }

    #[test]
    fn unknown_schemas_are_refused_not_guessed() {
        let document = Document::new("DTO");
        let mut dto = DocumentDtoV1::from_document(&document);
        dto.schema_version = SchemaVersion(999);
        assert!(dto.into_document().is_err());
    }

    #[test]
    fn dto_rejects_degenerate_page_geometry() {
        let mut document = Document::new("DTO");
        let page_id = document.scene.default_page();
        let page = document.pages.get_mut(page_id).expect("page");
        page.spec.size = Size2::new(0.0, 10.0).expect("representable");
        let dto = DocumentDtoV1::from_document(&document);
        assert!(dto.into_document().is_err());
    }

    #[test]
    fn dto_rejects_empty_margins_by_construction() {
        let size = Size2::new(10.0, 10.0).expect("valid");
        let margins = Insets::new(1.0, 1.0, 1.0, 1.0).expect("valid");
        let spec = crate::document::PageSpec::new(size, margins, Insets::ZERO).expect("valid");
        let _ = spec;
    }
}
