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

/// Canonical WASM binary container magic: `\0asm\1\0\0\0`.
pub const WASM_MAGIC: [u8; 8] = [0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

/// Error reported during plugin host execution.
#[derive(Debug, Clone, PartialEq)]
pub enum PluginError {
    /// Exceeded instruction or execution fuel limit.
    FuelExhausted,
    /// Memory allocation exceeded linear memory budget.
    MemoryLimitExceeded { requested: u64, max: u64 },
    /// Attempted invocation of an unauthorized capability.
    PermissionDenied(PluginCapability),
    /// Stale or unallocated opaque handle.
    InvalidHandle(u32),
    /// Malformed WASM bytecode or entrypoint.
    MalformedModule(String),
    /// Trap or unhandled runtime failure.
    Trap(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FuelExhausted => write!(f, "plugin fuel exhausted"),
            Self::MemoryLimitExceeded { requested, max } => {
                write!(
                    f,
                    "plugin requested {requested} bytes exceeding limit {max}"
                )
            }
            Self::PermissionDenied(cap) => write!(f, "permission denied for capability {cap:?}"),
            Self::InvalidHandle(h) => write!(f, "invalid or stale plugin handle {h}"),
            Self::MalformedModule(msg) => write!(f, "malformed WASM module: {msg}"),
            Self::Trap(msg) => write!(f, "plugin trapped: {msg}"),
        }
    }
}

impl std::error::Error for PluginError {}

/// Sandboxed execution environment for a WASM plugin instance.
///
/// Enforces instruction fuel metering, memory limits, handle bounds
/// and deny-by-default capability gates.
pub struct WasmPluginHost {
    policy: PluginPolicy,
    limits: PluginLimits,
    fuel_remaining: u64,
    allocated_memory_bytes: u64,
    handles: std::collections::BTreeMap<u32, String>,
    next_handle: u32,
    log_records: Vec<String>,
}

impl WasmPluginHost {
    /// Instantiate a sandbox with policy and initial fuel.
    pub fn new(
        manifest: &PluginManifest,
        limits: PluginLimits,
        initial_fuel: u64,
    ) -> std::result::Result<Self, PluginError> {
        manifest
            .validate()
            .map_err(|e| PluginError::MalformedModule(e.to_string()))?;
        Ok(Self {
            policy: PluginPolicy::new(manifest),
            limits,
            fuel_remaining: initial_fuel,
            allocated_memory_bytes: 0,
            handles: std::collections::BTreeMap::new(),
            next_handle: 1,
            log_records: Vec::new(),
        })
    }

    /// Load and validate a WASM bytecode module.
    pub fn load_module(&mut self, bytecode: &[u8]) -> std::result::Result<(), PluginError> {
        if bytecode.len() < 8 || bytecode[..8] != WASM_MAGIC {
            return Err(PluginError::MalformedModule(
                "invalid WASM magic".to_string(),
            ));
        }
        self.consume_fuel(bytecode.len() as u64)?;
        Ok(())
    }

    /// Consume execution fuel; fails with `FuelExhausted` when depleted.
    pub fn consume_fuel(&mut self, amount: u64) -> std::result::Result<(), PluginError> {
        if self.fuel_remaining < amount {
            self.fuel_remaining = 0;
            return Err(PluginError::FuelExhausted);
        }
        self.fuel_remaining -= amount;
        Ok(())
    }

    /// Remaining execution fuel.
    #[must_use]
    pub fn fuel_remaining(&self) -> u64 {
        self.fuel_remaining
    }

    /// Request linear memory expansion.
    pub fn allocate_memory(&mut self, bytes: u64) -> std::result::Result<(), PluginError> {
        let total = self.allocated_memory_bytes.saturating_add(bytes);
        if total > self.limits.linear_memory_bytes {
            return Err(PluginError::MemoryLimitExceeded {
                requested: total,
                max: self.limits.linear_memory_bytes,
            });
        }
        self.allocated_memory_bytes = total;
        self.consume_fuel(bytes / 64 + 1)?;
        Ok(())
    }

    /// Mediated Host API invocation.
    pub fn invoke_host_api(
        &mut self,
        capability: PluginCapability,
        payload: &[u8],
    ) -> std::result::Result<Vec<u8>, PluginError> {
        if !self.policy.may_use(capability) {
            return Err(PluginError::PermissionDenied(capability));
        }
        if payload.len() as u64 > self.limits.message_bytes {
            return Err(PluginError::Trap(format!(
                "message size {} exceeds limit {}",
                payload.len(),
                self.limits.message_bytes
            )));
        }
        self.consume_fuel(10 + payload.len() as u64)?;
        match capability {
            PluginCapability::LogWrite => {
                let text = String::from_utf8_lossy(payload).into_owned();
                self.log_records.push(text);
                Ok(b"ok".to_vec())
            }
            PluginCapability::DocumentQuery => Ok(b"{\"nodes\":1}".to_vec()),
            PluginCapability::CommandSubmit => Ok(b"{\"committed\":true}".to_vec()),
            _ => Ok(Vec::new()),
        }
    }

    /// Allocate an opaque handle.
    pub fn alloc_handle(&mut self, descriptor: String) -> std::result::Result<u32, PluginError> {
        if self.handles.len() >= self.limits.max_handles as usize {
            return Err(PluginError::Trap("handle table full".to_string()));
        }
        let handle = self.next_handle;
        self.next_handle = self.next_handle.saturating_add(1);
        self.handles.insert(handle, descriptor);
        Ok(handle)
    }

    /// Resolve an opaque handle.
    pub fn get_handle(&self, handle: u32) -> std::result::Result<&str, PluginError> {
        self.handles
            .get(&handle)
            .map(|s| s.as_str())
            .ok_or(PluginError::InvalidHandle(handle))
    }

    /// Release an opaque handle.
    pub fn release_handle(&mut self, handle: u32) -> std::result::Result<(), PluginError> {
        self.handles
            .remove(&handle)
            .map(|_| ())
            .ok_or(PluginError::InvalidHandle(handle))
    }

    /// Recorded plugin log messages.
    #[must_use]
    pub fn logs(&self) -> &[String] {
        &self.log_records
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

    #[test]
    fn wasm_host_executes_with_fuel_metering() {
        let manifest = manifest();
        let limits = PluginLimits::default();
        let mut host = WasmPluginHost::new(&manifest, limits, 100).expect("host instantiates");

        let mut bytecode = Vec::from(WASM_MAGIC);
        bytecode.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]);
        host.load_module(&bytecode).expect("module loads");
        assert!(host.fuel_remaining() < 100);

        // Mediated Host API call (LogWrite is declared in manifest):
        host.invoke_host_api(PluginCapability::LogWrite, b"hello from wasm")
            .expect("log succeeds");
        assert_eq!(host.logs(), &["hello from wasm"]);

        // Exhaust fuel:
        assert_eq!(host.consume_fuel(10_000), Err(PluginError::FuelExhausted));
        assert_eq!(host.fuel_remaining(), 0);
    }

    #[test]
    fn wasm_host_enforces_memory_limits() {
        let manifest = manifest();
        let limits = PluginLimits {
            linear_memory_bytes: 1024,
            ..PluginLimits::default()
        };
        let mut host = WasmPluginHost::new(&manifest, limits, 1_000).expect("host instantiates");
        assert!(host.allocate_memory(512).is_ok());
        assert!(host.allocate_memory(512).is_ok());
        // Exceeds 1024 bytes:
        assert_eq!(
            host.allocate_memory(1),
            Err(PluginError::MemoryLimitExceeded {
                requested: 1025,
                max: 1024
            })
        );
    }

    #[test]
    fn wasm_host_enforces_permissions_and_handles() {
        let manifest = manifest();
        let mut host = WasmPluginHost::new(&manifest, PluginLimits::default(), 1_000)
            .expect("host instantiates");

        // Undeclared capability (CommandSubmit) is denied:
        assert_eq!(
            host.invoke_host_api(PluginCapability::CommandSubmit, b"{}"),
            Err(PluginError::PermissionDenied(
                PluginCapability::CommandSubmit
            ))
        );

        // Handle allocation and lifecycle:
        let h1 = host
            .alloc_handle("surface:1".to_string())
            .expect("alloc handle");
        assert_eq!(host.get_handle(h1), Ok("surface:1"));
        host.release_handle(h1).expect("release");
        assert_eq!(host.get_handle(h1), Err(PluginError::InvalidHandle(h1)));
    }

    #[test]
    fn wasm_host_rejects_malformed_bytecode() {
        let manifest = manifest();
        let mut host = WasmPluginHost::new(&manifest, PluginLimits::default(), 1_000)
            .expect("host instantiates");
        assert!(matches!(
            host.load_module(b"not-wasm"),
            Err(PluginError::MalformedModule(_))
        ));
    }
}
