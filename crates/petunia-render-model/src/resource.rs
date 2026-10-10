//! Immutable resource handles for one snapshot lifetime.
//!
//! Decoded bytes, atlas slots and GPU textures stay runtime state;
//! the table maps identities to kinds and revisions so caches key
//! correctly. Pointers and file descriptors never serialize here.

use petunia_core::{ResourceId, ResourceKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One table entry: kind plus the content revision caches key on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceEntry {
    pub kind: ResourceKind,
    pub revision: u64,
}

/// Identity-to-entry map for a snapshot.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderResourceTable {
    pub entries: HashMap<ResourceId, ResourceEntry>,
    /// Runtime immutable resource handles; never persisted into document files.
    #[serde(skip)]
    pub images: HashMap<ResourceId, std::sync::Arc<crate::image::ResolvedImage>>,
}

impl RenderResourceTable {
    /// Empty table.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            images: HashMap::new(),
        }
    }

    /// Register one handle.
    pub fn insert(&mut self, id: ResourceId, entry: ResourceEntry) {
        self.entries.insert(id, entry);
    }
}
