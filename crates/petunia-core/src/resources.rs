//! Document resources: identity, storage source and integrity.
//!
//! `ResourceId` is the logical identity; `ContentHash` describes known
//! bytes. Runtime resolution (`Resolved`, `Unresolved`, …) is derived
//! state and never replaces the persistent record.

use crate::error::{CoreError, Result};
use crate::id::ResourceId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What a resource holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    Image,
    Font,
    IccProfile,
    Pattern,
}

/// Where the bytes live. Embedded entries use a logical container
/// reference, never an arbitrary filesystem path; linked entries keep
/// their `ResourceId` across relinks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceSource {
    Embedded { entry: String },
    Linked { uri: String },
}

impl ResourceSource {
    /// Entry names and URIs must be non-empty; loaders validate
    /// permissions and size limits before opening content.
    pub fn new_embedded(entry: impl Into<String>) -> Result<Self> {
        let entry = entry.into();
        if entry.trim().is_empty() {
            return Err(CoreError::InvariantViolation(
                "embedded resource needs an entry name".to_string(),
            ));
        }
        Ok(Self::Embedded { entry })
    }

    /// Linked URIs are parsed/normalized by the loader, never
    /// concatenated into filesystem paths here.
    pub fn new_linked(uri: impl Into<String>) -> Result<Self> {
        let uri = uri.into();
        if uri.trim().is_empty() {
            return Err(CoreError::InvariantViolation(
                "linked resource needs a URI".to_string(),
            ));
        }
        Ok(Self::Linked { uri })
    }
}

/// SHA-256 over known bytes: detects external change, shares caches,
/// deduplicates blobs and validates integrity. Never identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentHash {
    pub sha256: [u8; 32],
}

impl ContentHash {
    /// Parse a 64-character lowercase hex digest.
    pub fn from_hex(digest: &str) -> Result<Self> {
        if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(CoreError::InvariantViolation(
                "content hash needs 64 hex characters".to_string(),
            ));
        }
        let mut sha256 = [0u8; 32];
        for (index, chunk) in digest.as_bytes().chunks(2).enumerate() {
            let text = std::str::from_utf8(chunk).expect("ascii hex");
            sha256[index] = u8::from_str_radix(text, 16).expect("hex pair");
        }
        Ok(Self { sha256 })
    }
}

/// Font embedding policy recorded in resource metadata so exports
/// never embed a prohibited face silently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontEmbedPolicy {
    Installable,
    Editable,
    PreviewPrint,
    Restricted,
}

/// Descriptive metadata; licensing flags travel with the record.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ResourceMetadata {
    pub byte_size: Option<u64>,
    pub font_policy: Option<FontEmbedPolicy>,
}

/// One persistent resource record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceRecord {
    pub id: ResourceId,
    pub kind: ResourceKind,
    pub source: ResourceSource,
    pub content_hash: Option<ContentHash>,
    pub metadata: ResourceMetadata,
}

/// Registry by identity. Deleting a scene node never purges here;
/// collection checks scene, style, symbol and history references.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ResourceRegistry {
    records: BTreeMap<ResourceId, ResourceRecord>,
}

impl ResourceRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            records: BTreeMap::new(),
        }
    }

    /// Insert or replace the record for its ID.
    pub fn insert(&mut self, record: ResourceRecord) {
        self.records.insert(record.id, record);
    }

    /// Look up a record by logical identity.
    #[must_use]
    pub fn get(&self, id: ResourceId) -> Option<&ResourceRecord> {
        self.records.get(&id)
    }

    /// Iterate records in deterministic order; traversal only.
    pub fn iter(&self) -> impl Iterator<Item = (ResourceId, &ResourceRecord)> {
        self.records.iter().map(|(id, record)| (*id, record))
    }

    /// Number of tracked records.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// True when no record is tracked.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sources_require_non_empty_locations() {
        assert!(ResourceSource::new_embedded("images/logo.png").is_ok());
        assert!(ResourceSource::new_embedded("  ").is_err());
        assert!(ResourceSource::new_linked("file:///assets/a.png").is_ok());
        assert!(ResourceSource::new_linked("").is_err());
    }

    #[test]
    fn content_hash_parses_strict_hex() {
        let digest = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert!(ContentHash::from_hex(digest).is_ok());
        assert!(ContentHash::from_hex("xyz").is_err());
        assert!(ContentHash::from_hex(&digest[..63]).is_err());
    }

    #[test]
    fn registry_tracks_records_by_identity() {
        let mut registry = ResourceRegistry::new();
        assert!(registry.is_empty());
        let record = ResourceRecord {
            id: ResourceId::new_v4(),
            kind: ResourceKind::Image,
            source: ResourceSource::new_linked("file:///a.png").expect("valid"),
            content_hash: None,
            metadata: ResourceMetadata::default(),
        };
        let id = record.id;
        registry.insert(record);
        assert_eq!(registry.len(), 1);
        assert!(registry.get(id).is_some());
    }
}
