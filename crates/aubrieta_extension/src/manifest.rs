//! Plugin manifest, permissions and identity contracts (09.14 & 09.28).

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

/// Stable namespaced identifier for a plugin (e.g. `org.aubrieta.shape_builder`).
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PluginId(pub String);

impl PluginId {
    /// Creates a new `PluginId`.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the ID as string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for PluginId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PluginId(\"{}\")", self.0)
    }
}

impl fmt::Display for PluginId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Fine-grained capabilities that a plugin may request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginPermission {
    /// Read document structure, layers, and objects.
    DocumentRead,
    /// Mutate document through Action/Command requests.
    DocumentWrite,
    /// Read and write to host clipboard.
    Clipboard,
    /// Access plugin-private persistent key-value storage.
    ScopedStorage,
    /// Brokered outbound network requests (restricted by default).
    Network,
}

impl fmt::Display for PluginPermission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DocumentRead => write!(f, "document:read"),
            Self::DocumentWrite => write!(f, "document:write"),
            Self::Clipboard => write!(f, "clipboard"),
            Self::ScopedStorage => write!(f, "storage:scoped"),
            Self::Network => write!(f, "network:outbound"),
        }
    }
}

/// Authoritative manifest declaring plugin identity, requirements and capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Unique plugin identifier.
    pub id: PluginId,
    /// User-visible name.
    pub name: String,
    /// Semantic version of the plugin.
    pub version: String,
    /// Aubrieta Plugin API version target.
    pub api_version: u32,
    /// Author or publisher name.
    pub author: String,
    /// Short summary of plugin capabilities.
    pub description: String,
    /// Entrypoint script filename (e.g. `main.lua`).
    pub entrypoint: String,
    /// Declared required permissions.
    pub permissions: HashSet<PluginPermission>,
}

impl PluginManifest {
    /// Helper to construct a simple plugin manifest.
    pub fn new(
        id: PluginId,
        name: impl Into<String>,
        version: impl Into<String>,
        entrypoint: impl Into<String>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            version: version.into(),
            api_version: 1,
            author: "Unknown".to_string(),
            description: String::new(),
            entrypoint: entrypoint.into(),
            permissions: HashSet::new(),
        }
    }

    /// Grants a permission in the manifest.
    #[must_use]
    pub fn with_permission(mut self, perm: PluginPermission) -> Self {
        self.permissions.insert(perm);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_serialization_roundtrip() {
        let mut manifest = PluginManifest::new(
            PluginId::new("aubrieta.test.generator"),
            "Generator Plugin",
            "1.0.0",
            "init.lua",
        )
        .with_permission(PluginPermission::DocumentRead)
        .with_permission(PluginPermission::Clipboard);

        manifest.author = "Core Team".to_string();

        let json = serde_json::to_string(&manifest).expect("serialize");
        let deserialized: PluginManifest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(manifest, deserialized);
        assert!(deserialized
            .permissions
            .contains(&PluginPermission::DocumentRead));
        assert!(deserialized
            .permissions
            .contains(&PluginPermission::Clipboard));
        assert!(!deserialized
            .permissions
            .contains(&PluginPermission::DocumentWrite));
    }
}
