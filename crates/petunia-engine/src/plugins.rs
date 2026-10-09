//! Plugin model: manifests, capabilities and deny-by-default permissions.
//!
//! Public engine extensions are sandboxed by default: WASM plus a
//! versioned Host API is the initial format, and the concrete WASM
//! runtime is chosen at its milestone with size/startup benchmarks.
//! This module pins down everything runtime-independent: identity,
//! manifests, capability vocabulary, permission checks and the ABI
//! boundary rules.

use crate::error::{EngineError, Result};
use serde::{Deserialize, Serialize};

/// Capability-oriented Host API surface. Plugins receive opaque
/// handles and stable DTOs through these, never `&mut Document`,
/// scene internals, Qt objects or raw Rust trait objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginCapability {
    DocumentQuery,
    CommandSubmit,
    ResourceRead,
    ImportRegister,
    ExportRegister,
    EffectEvaluate,
    LogWrite,
    JobSpawn,
}

/// Permission classes. Deny by default: a plugin without the grant
/// cannot perform the operation, and network/process stay off unless
/// explicitly allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    FilesystemReadSelected,
    FilesystemWriteSelected,
    Network,
    Clipboard,
    Process,
    UiExtension,
}

impl PluginCapability {
    /// The permission gate for one capability. Pure mediated calls
    /// (query, log, command submit through the host) need no grant;
    /// resource and interchange capabilities do.
    #[must_use]
    pub fn required_permission(self) -> Option<Permission> {
        match self {
            Self::ResourceRead | Self::ImportRegister => Some(Permission::FilesystemReadSelected),
            Self::ExportRegister => Some(Permission::FilesystemWriteSelected),
            Self::DocumentQuery
            | Self::CommandSubmit
            | Self::EffectEvaluate
            | Self::LogWrite
            | Self::JobSpawn => None,
        }
    }
}

/// Versioned Host API contract, independent from package and data
/// schema versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostApiVersion {
    pub major: u32,
    pub minor: u32,
}

impl HostApiVersion {
    /// Compatible when the major matches and the host minor covers
    /// the plugin minor.
    #[must_use]
    pub fn compatible_with(self, host: Self) -> bool {
        self.major == host.major && self.minor <= host.minor
    }
}

/// Package version of one plugin release.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

/// Versioned plugin package manifest. The display name is never the
/// identity; `plugin_id` uses a stable namespace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin_id: String,
    pub version: PluginVersion,
    pub host_api_version: HostApiVersion,
    pub entrypoints: Vec<String>,
    pub capabilities: Vec<PluginCapability>,
    pub permissions: Vec<Permission>,
    pub metadata: PluginMetadata,
}

/// Descriptive metadata, not authority.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub display_name: String,
    pub author: String,
    pub description: String,
}

/// Resource limits enforced by the runtime around every invocation.
/// Breaches end the call without corrupting the document.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PluginLimits {
    pub linear_memory_bytes: u64,
    pub message_bytes: u64,
    pub max_handles: u32,
    pub max_jobs: u32,
    pub output_bytes: u64,
}

impl Default for PluginLimits {
    fn default() -> Self {
        Self {
            linear_memory_bytes: 64 << 20,
            message_bytes: 1 << 20,
            max_handles: 256,
            max_jobs: 8,
            output_bytes: 16 << 20,
        }
    }
}

impl PluginManifest {
    /// Validate identity, versions and entrypoints. Capability and
    /// permission semantics check separately through [`PluginPolicy`].
    pub fn validate(&self) -> Result<()> {
        if self.plugin_id.trim().is_empty() || !self.plugin_id.contains('.') {
            return Err(EngineError::Execution(format!(
                "plugin id needs a stable namespace, got {:?}",
                self.plugin_id
            )));
        }
        if self.entrypoints.is_empty() || self.entrypoints.iter().any(|name| name.trim().is_empty())
        {
            return Err(EngineError::Execution(
                "plugin needs at least one named entrypoint".to_string(),
            ));
        }
        Ok(())
    }
}

/// Evaluated policy for one manifest: capability grants plus the
/// deny-by-default permission set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginPolicy {
    pub capabilities: Vec<PluginCapability>,
    pub granted: Vec<Permission>,
}

impl PluginPolicy {
    /// Build from a validated manifest.
    #[must_use]
    pub fn new(manifest: &PluginManifest) -> Self {
        Self {
            capabilities: manifest.capabilities.clone(),
            granted: manifest.permissions.clone(),
        }
    }

    /// True when the capability is declared and its permission gate
    /// (if any) is granted. Undeclared capabilities never pass.
    #[must_use]
    pub fn may_use(&self, capability: PluginCapability) -> bool {
        if !self.capabilities.contains(&capability) {
            return false;
        }
        match capability.required_permission() {
            Some(permission) => self.granted.contains(&permission),
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> PluginManifest {
        PluginManifest {
            plugin_id: "design.petunia.tint".to_string(),
            version: PluginVersion {
                major: 1,
                minor: 0,
                patch: 0,
            },
            host_api_version: HostApiVersion { major: 1, minor: 2 },
            entrypoints: vec!["evaluate".to_string()],
            capabilities: vec![PluginCapability::EffectEvaluate, PluginCapability::LogWrite],
            permissions: vec![],
            metadata: PluginMetadata {
                display_name: "Tint".to_string(),
                author: "Petunia".to_string(),
                description: "Test effect".to_string(),
            },
        }
    }

    #[test]
    fn manifest_validation_rejects_anonymous_packages() {
        assert!(manifest().validate().is_ok());
        let mut bad = manifest();
        bad.plugin_id = "tint".to_string();
        assert!(bad.validate().is_err());
        bad.plugin_id = "design.petunia.tint".to_string();
        bad.entrypoints.clear();
        assert!(bad.validate().is_err());
    }

    #[test]
    fn deny_by_default_blocks_undeclared_capabilities() {
        let policy = PluginPolicy::new(&manifest());
        assert!(policy.may_use(PluginCapability::EffectEvaluate));
        assert!(policy.may_use(PluginCapability::LogWrite));
        assert!(!policy.may_use(PluginCapability::CommandSubmit));
        assert!(!policy.may_use(PluginCapability::ResourceRead));
    }

    #[test]
    fn gated_capabilities_need_explicit_grants() {
        let mut granted = manifest();
        granted.capabilities.push(PluginCapability::ImportRegister);
        let denied = PluginPolicy::new(&granted);
        assert!(!denied.may_use(PluginCapability::ImportRegister));
        granted.permissions.push(Permission::FilesystemReadSelected);
        let allowed = PluginPolicy::new(&granted);
        assert!(allowed.may_use(PluginCapability::ImportRegister));
        // The wrong grant does not open the gate.
        granted.permissions.clear();
        granted.permissions.push(Permission::Network);
        let wrong = PluginPolicy::new(&granted);
        assert!(!wrong.may_use(PluginCapability::ImportRegister));
    }

    #[test]
    fn host_api_compatibility_ignores_package_version() {
        let plugin = HostApiVersion { major: 1, minor: 2 };
        assert!(plugin.compatible_with(HostApiVersion { major: 1, minor: 5 }));
        assert!(!plugin.compatible_with(HostApiVersion { major: 1, minor: 1 }));
        assert!(!plugin.compatible_with(HostApiVersion { major: 2, minor: 2 }));
    }
}
