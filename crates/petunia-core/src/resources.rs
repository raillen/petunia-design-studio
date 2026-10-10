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

/// BLAKE3-256 over known bytes: detects external change, shares caches,
/// deduplicates blobs and validates integrity. Never identity.
///
/// The persistent text form is algorithm-tagged (`blake3:<hex>`) so a
/// future algorithm change cannot silently reinterpret old hashes.
/// Parsing accepts either hex case but always canonicalizes; only the
/// lowercase tagged form ever serializes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentHash {
    blake3: [u8; 32],
}

/// Tag prefixing every persisted content hash.
pub const CONTENT_HASH_TAG: &str = "blake3:";

impl ContentHash {
    /// Hash known bytes.
    #[must_use]
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            blake3: *blake3::hash(bytes).as_bytes(),
        }
    }

    /// Raw digest bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.blake3
    }

    /// Canonical tagged text: `blake3:<lowercase hex>`.
    #[must_use]
    pub fn to_tagged(&self) -> String {
        format!(
            "{CONTENT_HASH_TAG}{}",
            blake3::Hash::from_bytes(self.blake3).to_hex()
        )
    }

    /// Parse a tagged digest, refusing bare hex, wrong tags and
    /// malformed digits with typed errors.
    pub fn from_tagged(text: &str) -> Result<Self> {
        let hex = text.strip_prefix(CONTENT_HASH_TAG).ok_or_else(|| {
            CoreError::InvariantViolation(format!(
                "content hash needs the `{CONTENT_HASH_TAG}` tag: {text:?}"
            ))
        })?;
        if hex.len() != 64 {
            return Err(CoreError::InvariantViolation(format!(
                "content hash needs 64 hex characters, got {}",
                hex.len()
            )));
        }
        let digest = blake3::Hash::from_hex(hex)
            .map_err(|_| CoreError::InvariantViolation("content hash is not hex".to_string()))?;
        Ok(Self {
            blake3: *digest.as_bytes(),
        })
    }
}

impl serde::Serialize for ContentHash {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_tagged())
    }
}

impl<'de> serde::Deserialize<'de> for ContentHash {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        Self::from_tagged(&text).map_err(serde::de::Error::custom)
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
    #[serde(deserialize_with = "crate::serialization::deserialize_unique_btree_map")]
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
    fn content_hash_is_tagged_blake3() {
        let hash = ContentHash::new(b"petunia");
        let tagged = hash.to_tagged();
        assert!(tagged.starts_with("blake3:"), "{tagged}");
        assert_eq!(tagged, tagged.to_lowercase(), "canonical form is lowercase");
        assert_eq!(ContentHash::from_tagged(&tagged).expect("parses"), hash);
        // Bare digests never slip in untagged.
        assert!(ContentHash::from_tagged(tagged.trim_start_matches("blake3:")).is_err());
        assert!(ContentHash::from_tagged("sha256:abc").is_err());
        assert!(ContentHash::from_tagged("xyz").is_err());
        // Serialization round-trips through the tagged text.
        let json = serde_json::to_string(&hash).expect("serializes");
        assert_eq!(json, format!("\"{tagged}\""));
        let back: ContentHash = serde_json::from_str(&json).expect("parses");
        assert_eq!(back, hash);
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
