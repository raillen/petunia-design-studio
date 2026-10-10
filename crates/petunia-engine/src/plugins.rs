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
    pub max_recursion_depth: u32,
    pub value_stack_bytes: u32,
}

impl Default for PluginLimits {
    fn default() -> Self {
        Self {
            linear_memory_bytes: 64 << 20,
            message_bytes: 1 << 20,
            max_handles: 256,
            max_jobs: 8,
            output_bytes: 16 << 20,
            max_recursion_depth: 256,
            value_stack_bytes: 1 << 20,
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
    /// Requested permissions are not authority. Hosts grant permissions separately.
    #[must_use]
    pub fn new(manifest: &PluginManifest) -> Self {
        Self {
            capabilities: manifest.capabilities.clone(),
            granted: Vec::new(),
        }
    }

    /// Grant only permissions requested by this package and approved by the host.
    #[must_use]
    pub fn with_grants(manifest: &PluginManifest, grants: &[Permission]) -> Self {
        Self {
            capabilities: manifest.capabilities.clone(),
            granted: grants
                .iter()
                .copied()
                .filter(|p| manifest.permissions.contains(p))
                .collect(),
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

/// Version supported by the concrete JSON/handle ABI below.
pub const HOST_API_VERSION: HostApiVersion = HostApiVersion { major: 1, minor: 2 };
pub const WASM_MAGIC: [u8; 8] = [0, 97, 115, 109, 1, 0, 0, 0];

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PluginError {
    #[error("plugin fuel exhausted")]
    FuelExhausted,
    #[error("plugin requested {requested} bytes exceeding limit {max}")]
    MemoryLimitExceeded { requested: u64, max: u64 },
    #[error("permission denied for capability {0:?}")]
    PermissionDenied(PluginCapability),
    #[error("invalid or stale plugin handle {0}")]
    InvalidHandle(u32),
    #[error("malformed WASM module: {0}")]
    MalformedModule(String),
    #[error("plugin trapped: {0}")]
    Trap(String),
    #[error("host API version unsupported")]
    IncompatibleVersion,
    #[error("capability {0:?} has no service bound")]
    ServiceUnavailable(PluginCapability),
    #[error("plugin resource limit: {0}")]
    ResourceLimit(String),
}

/// Stable command DTO; revision validation precedes staging and the host
/// commits all staged commands as one undoable transaction only after success.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginCommand {
    pub revision: crate::transaction::DocumentRevision,
    pub operations: Vec<crate::transaction::DocumentOp>,
}

/// A bounded query returns IDs and names, not internal objects or document pointers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginQuery {
    #[serde(default)]
    pub offset: usize,
    pub limit: usize,
}

struct HostState {
    policy: PluginPolicy,
    limits: PluginLimits,
    runtime_limits: wasmi::StoreLimits,
    handles: std::collections::BTreeMap<u32, Vec<u8>>,
    next_handle: u32,
    output_size: u64,
    logs: Vec<String>,
    document: Option<petunia_core::Document>,
    revision: crate::transaction::DocumentRevision,
    staged: Vec<crate::transaction::DocumentOp>,
    error: Option<PluginError>,
}

impl wasmi::ResourceLimiter for HostState {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> std::result::Result<bool, wasmi_core::LimiterError> {
        if desired as u64 > self.limits.linear_memory_bytes {
            self.error = Some(PluginError::MemoryLimitExceeded {
                requested: desired as u64,
                max: self.limits.linear_memory_bytes,
            });
        }
        self.runtime_limits
            .memory_growing(current, desired, maximum)
    }
    fn table_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> std::result::Result<bool, wasmi_core::LimiterError> {
        self.runtime_limits.table_growing(current, desired, maximum)
    }
    fn instances(&self) -> usize {
        1
    }
    fn memories(&self) -> usize {
        1
    }
    fn tables(&self) -> usize {
        1
    }
}

impl HostState {
    fn response(&mut self, value: Vec<u8>) -> std::result::Result<u32, PluginError> {
        self.reserve_output(value.len())?;
        if self.handles.len() >= self.limits.max_handles as usize {
            return Err(PluginError::ResourceLimit("handle table full".into()));
        }
        // Never recycle an ID: a released handle cannot alias a newer response.
        let id = self.next_handle;
        self.next_handle = id
            .checked_add(1)
            .filter(|n| *n <= i32::MAX as u32)
            .ok_or_else(|| PluginError::ResourceLimit("handle identities exhausted".into()))?;
        self.handles.insert(id, value);
        Ok(id)
    }

    fn reserve_output(&mut self, count: usize) -> std::result::Result<(), PluginError> {
        let total = self
            .output_size
            .checked_add(count as u64)
            .ok_or_else(|| PluginError::ResourceLimit("output overflow".into()))?;
        if total > self.limits.output_bytes {
            return Err(PluginError::ResourceLimit("output bytes exceeded".into()));
        }
        self.output_size = total;
        Ok(())
    }

    fn call(
        &mut self,
        cap: PluginCapability,
        payload: &[u8],
    ) -> std::result::Result<Vec<u8>, PluginError> {
        use crate::transaction::{
            commit_transaction, prepare_transaction, CommandId, TransactionRequest,
        };
        if !self.policy.may_use(cap) {
            return Err(PluginError::PermissionDenied(cap));
        }
        if payload.len() as u64 > self.limits.message_bytes {
            return Err(PluginError::ResourceLimit("message bytes exceeded".into()));
        }
        match cap {
            PluginCapability::LogWrite => {
                let text =
                    std::str::from_utf8(payload).map_err(|e| PluginError::Trap(e.to_string()))?;
                self.reserve_output(text.len())?;
                self.logs.push(text.into());
                Ok(b"ok".to_vec())
            }
            PluginCapability::DocumentQuery => {
                let q: PluginQuery = serde_json::from_slice(payload)
                    .map_err(|e| PluginError::Trap(e.to_string()))?;
                if q.limit == 0 || q.limit > 256 {
                    return Err(PluginError::ResourceLimit(
                        "query page must contain 1..256 nodes".into(),
                    ));
                }
                let document = self
                    .document
                    .as_ref()
                    .ok_or(PluginError::ServiceUnavailable(cap))?;
                let nodes: Vec<_> = document
                    .scene
                    .root_lists()
                    .into_iter()
                    .flatten()
                    .flat_map(|id| std::iter::once(*id).chain(document.scene.descendants(*id)))
                    .skip(q.offset)
                    .take(q.limit)
                    .filter_map(|id| document.scene.get_node(id))
                    .map(|n| serde_json::json!({"id":n.id,"name":n.name}))
                    .collect();
                serde_json::to_vec(&serde_json::json!({"revision":self.revision,"total":document.scene.len(),"nodes":nodes}))
                    .map_err(|e| PluginError::Trap(e.to_string()))
            }
            PluginCapability::CommandSubmit => {
                let command: PluginCommand = serde_json::from_slice(payload)
                    .map_err(|e| PluginError::Trap(e.to_string()))?;
                if command.revision != self.revision || command.operations.is_empty() {
                    return Err(PluginError::Trap("stale revision or empty command".into()));
                }
                let encoded_size = serde_json::to_vec(&self.staged)
                    .map_err(|e| PluginError::Trap(e.to_string()))?
                    .len();
                if encoded_size.saturating_add(payload.len()) as u64 > self.limits.message_bytes {
                    return Err(PluginError::ResourceLimit(
                        "staged commands exceeded message budget".into(),
                    ));
                }
                let doc = self
                    .document
                    .as_mut()
                    .ok_or(PluginError::ServiceUnavailable(cap))?;
                let prepared = prepare_transaction(
                    doc,
                    TransactionRequest {
                        command_id: CommandId::new_v4(),
                        operations: command.operations.clone(),
                        merge_key: None,
                    },
                    self.revision,
                )
                .map_err(|e| PluginError::Trap(e.to_string()))?;
                commit_transaction(doc, prepared).map_err(|e| PluginError::Trap(e.to_string()))?;
                self.staged.extend(command.operations);
                Ok(b"{\"staged\":true}".to_vec())
            }
            // No implicit filesystem, jobs, network, process, effect or registration service.
            _ => Err(PluginError::ServiceUnavailable(cap)),
        }
    }
}

/// Real Wasmi interpreter. Each invocation has fresh memory and bounded fuel;
/// no WASI imports are linked. Public ABI imports live under `petunia`:
/// `call(capability, ptr, len) -> response_handle`, `response_len(handle)`,
/// `response_read(handle, ptr, capacity)`, and `release(handle)`.
/// Entrypoints have `() -> i32`; negative results abort staged commands.
/// Capability numbers: 0=query, 1=command, 2=log. Unknown numbers trap.
pub struct WasmPluginHost {
    manifest: PluginManifest,
    policy: PluginPolicy,
    limits: PluginLimits,
    engine: wasmi::Engine,
    module: Option<wasmi::Module>,
    fuel_remaining: u64,
    logs: Vec<String>,
}

impl WasmPluginHost {
    pub fn new(
        manifest: &PluginManifest,
        limits: PluginLimits,
        initial_fuel: u64,
    ) -> std::result::Result<Self, PluginError> {
        Self::with_grants(manifest, limits, initial_fuel, &[])
    }

    pub fn with_grants(
        manifest: &PluginManifest,
        limits: PluginLimits,
        initial_fuel: u64,
        grants: &[Permission],
    ) -> std::result::Result<Self, PluginError> {
        manifest
            .validate()
            .map_err(|e| PluginError::MalformedModule(e.to_string()))?;
        if !manifest.host_api_version.compatible_with(HOST_API_VERSION) {
            return Err(PluginError::IncompatibleVersion);
        }
        let mut config = wasmi::Config::default();
        config.consume_fuel(true);
        if limits.max_recursion_depth == 0
            || limits.value_stack_bytes < 1024
            || limits.message_bytes > i32::MAX as u64
            || limits.output_bytes > i32::MAX as u64
        {
            return Err(PluginError::ResourceLimit(
                "invalid stack or ABI message limits".into(),
            ));
        }
        config.set_max_recursion_depth(limits.max_recursion_depth as usize);
        config.set_min_stack_height(1024);
        config.set_max_stack_height(limits.value_stack_bytes as usize);
        Ok(Self {
            manifest: manifest.clone(),
            policy: PluginPolicy::with_grants(manifest, grants),
            limits,
            engine: wasmi::Engine::new(&config),
            module: None,
            fuel_remaining: initial_fuel,
            logs: Vec::new(),
        })
    }

    pub fn load_module(&mut self, bytecode: &[u8]) -> std::result::Result<(), PluginError> {
        self.module = None;
        if bytecode.len() as u64 > self.limits.message_bytes {
            return Err(PluginError::ResourceLimit(
                "module bytes exceeded message budget".into(),
            ));
        }
        self.module = Some(
            wasmi::Module::new(&self.engine, bytecode)
                .map_err(|e| PluginError::MalformedModule(e.to_string()))?,
        );
        Ok(())
    }

    #[must_use]
    pub fn fuel_remaining(&self) -> u64 {
        self.fuel_remaining
    }
    #[must_use]
    pub fn logs(&self) -> &[String] {
        &self.logs
    }

    /// Execute against a cloned snapshot. A runtime trap, denied call or negative
    /// result discards all staged mutations; successful work uses normal History.
    pub fn execute(
        &mut self,
        entrypoint: &str,
        document: &mut petunia_core::Document,
        history: &mut crate::history::History,
    ) -> std::result::Result<i32, PluginError> {
        if !self.manifest.entrypoints.iter().any(|e| e == entrypoint) {
            return Err(PluginError::MalformedModule("undeclared entrypoint".into()));
        }
        let module = self
            .module
            .as_ref()
            .ok_or_else(|| PluginError::MalformedModule("no module loaded".into()))?;
        let linear_limit = usize::try_from(self.limits.linear_memory_bytes)
            .map_err(|_| PluginError::ResourceLimit("memory size overflow".into()))?;
        let state = HostState {
            policy: self.policy.clone(),
            limits: self.limits,
            runtime_limits: wasmi::StoreLimitsBuilder::new()
                .memory_size(linear_limit)
                .memories(1)
                .tables(1)
                .table_elements(4096)
                .instances(1)
                .trap_on_grow_failure(true)
                .build(),
            handles: Default::default(),
            next_handle: 1,
            output_size: 0,
            logs: Vec::new(),
            document: Some(document.clone()),
            revision: history.current_revision(),
            staged: Vec::new(),
            error: None,
        };
        let mut store = wasmi::Store::new(&self.engine, state);
        store.limiter(|s| s);
        store.set_fuel(self.fuel_remaining).map_err(runtime_error)?;
        let linker = host_linker(&self.engine)?;
        let result = (|| {
            let instance = linker
                .instantiate_and_start(&mut store, module)
                .map_err(runtime_error)?;
            let func = instance
                .get_typed_func::<(), i32>(&store, entrypoint)
                .map_err(|e| PluginError::MalformedModule(e.to_string()))?;
            func.call(&mut store, ()).map_err(runtime_error)
        })();
        self.fuel_remaining = store.get_fuel().map_err(runtime_error)?;
        self.logs.clone_from(&store.data().logs);
        if let Some(error) = store.data_mut().error.take() {
            return Err(error);
        }
        let value = result?;
        if value < 0 {
            return Err(PluginError::Trap("entrypoint aborted".into()));
        }
        if !store.data().staged.is_empty() {
            let operations = std::mem::take(&mut store.data_mut().staged);
            let revision = history.current_revision();
            let request = crate::transaction::TransactionRequest {
                command_id: crate::transaction::CommandId::new_v4(),
                operations,
                merge_key: None,
            };
            let prepared = crate::transaction::prepare_transaction(document, request, revision)
                .map_err(|e| PluginError::Trap(e.to_string()))?;
            history
                .commit(
                    document,
                    prepared,
                    crate::history::HistoryDescription::EditObjects,
                )
                .map_err(|e| PluginError::Trap(e.to_string()))?;
        }
        Ok(value)
    }
}

fn runtime_error(error: wasmi::Error) -> PluginError {
    if error.as_trap_code() == Some(wasmi::TrapCode::OutOfFuel) {
        PluginError::FuelExhausted
    } else {
        PluginError::Trap(error.to_string())
    }
}

fn fail(caller: &mut wasmi::Caller<'_, HostState>, error: PluginError) -> wasmi::Error {
    let message = error.to_string();
    caller.data_mut().error = Some(error);
    wasmi::Error::new(message)
}

fn host_linker(
    engine: &wasmi::Engine,
) -> std::result::Result<wasmi::Linker<HostState>, PluginError> {
    let mut linker = wasmi::Linker::<HostState>::new(engine);
    linker
        .func_wrap(
            "petunia",
            "call",
            |mut caller: wasmi::Caller<'_, HostState>,
             cap: i32,
             ptr: i32,
             len: i32|
             -> std::result::Result<i32, wasmi::Error> {
                let cap = match cap {
                    0 => PluginCapability::DocumentQuery,
                    1 => PluginCapability::CommandSubmit,
                    2 => PluginCapability::LogWrite,
                    _ => {
                        return Err(fail(
                            &mut caller,
                            PluginError::Trap("unknown capability number".into()),
                        ))
                    }
                };
                let Ok(len) = usize::try_from(len) else {
                    return Err(fail(
                        &mut caller,
                        PluginError::ResourceLimit("negative message length".into()),
                    ));
                };
                let Ok(ptr) = usize::try_from(ptr) else {
                    return Err(fail(
                        &mut caller,
                        PluginError::Trap("negative memory pointer".into()),
                    ));
                };
                if len as u64 > caller.data().limits.message_bytes {
                    return Err(fail(
                        &mut caller,
                        PluginError::ResourceLimit("message bytes exceeded".into()),
                    ));
                }
                let memory = caller
                    .get_export("memory")
                    .and_then(wasmi::Extern::into_memory)
                    .ok_or_else(|| {
                        fail(
                            &mut caller,
                            PluginError::Trap("missing exported memory".into()),
                        )
                    })?;
                let mut payload = vec![0; len];
                memory
                    .read(&caller, ptr, &mut payload)
                    .map_err(|e| fail(&mut caller, PluginError::Trap(e.to_string())))?;
                let fuel = caller.get_fuel()?;
                let charge = 10u64.saturating_add(len as u64);
                if fuel < charge {
                    return Err(fail(&mut caller, PluginError::FuelExhausted));
                }
                caller.set_fuel(fuel - charge)?;
                let response = caller
                    .data_mut()
                    .call(cap, &payload)
                    .map_err(|e| fail(&mut caller, e))?;
                let handle = caller
                    .data_mut()
                    .response(response)
                    .map_err(|e| fail(&mut caller, e))?;
                Ok(handle as i32)
            },
        )
        .map_err(|e| PluginError::Trap(e.to_string()))?;
    linker
        .func_wrap(
            "petunia",
            "response_len",
            |mut caller: wasmi::Caller<'_, HostState>,
             handle: i32|
             -> std::result::Result<i32, wasmi::Error> {
                caller
                    .data()
                    .handles
                    .get(&(handle as u32))
                    .map(|s| s.len() as i32)
                    .ok_or_else(|| fail(&mut caller, PluginError::InvalidHandle(handle as u32)))
            },
        )
        .map_err(|e| PluginError::Trap(e.to_string()))?;
    linker
        .func_wrap(
            "petunia",
            "response_read",
            |mut caller: wasmi::Caller<'_, HostState>,
             handle: i32,
             ptr: i32,
             capacity: i32|
             -> std::result::Result<i32, wasmi::Error> {
                let bytes = caller
                    .data()
                    .handles
                    .get(&(handle as u32))
                    .cloned()
                    .ok_or_else(|| fail(&mut caller, PluginError::InvalidHandle(handle as u32)))?;
                if ptr < 0 || capacity < 0 || (capacity as usize) < bytes.len() {
                    return Err(fail(
                        &mut caller,
                        PluginError::ResourceLimit("response buffer too small".into()),
                    ));
                }
                let memory = caller
                    .get_export("memory")
                    .and_then(wasmi::Extern::into_memory)
                    .ok_or_else(|| {
                        fail(
                            &mut caller,
                            PluginError::Trap("missing exported memory".into()),
                        )
                    })?;
                memory
                    .write(&mut caller, ptr as usize, &bytes)
                    .map_err(|e| fail(&mut caller, PluginError::Trap(e.to_string())))?;
                Ok(bytes.len() as i32)
            },
        )
        .map_err(|e| PluginError::Trap(e.to_string()))?;
    linker
        .func_wrap(
            "petunia",
            "release",
            |mut caller: wasmi::Caller<'_, HostState>,
             handle: i32|
             -> std::result::Result<(), wasmi::Error> {
                if caller.data_mut().handles.remove(&(handle as u32)).is_none() {
                    return Err(fail(&mut caller, PluginError::InvalidHandle(handle as u32)));
                }
                Ok(())
            },
        )
        .map_err(|e| PluginError::Trap(e.to_string()))?;
    Ok(linker)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{history::History, transaction::DocumentOp};
    use petunia_core::{Document, ParentRef, SceneNode, VectorPath};
    fn manifest() -> PluginManifest {
        PluginManifest {
            plugin_id: "org.petunia.test".into(),
            version: PluginVersion {
                major: 1,
                minor: 0,
                patch: 0,
            },
            host_api_version: HOST_API_VERSION,
            entrypoints: vec!["run".into()],
            capabilities: vec![
                PluginCapability::DocumentQuery,
                PluginCapability::CommandSubmit,
                PluginCapability::LogWrite,
            ],
            permissions: vec![],
            metadata: PluginMetadata::default(),
        }
    }
    fn host(limits: PluginLimits, fuel: u64) -> WasmPluginHost {
        WasmPluginHost::new(&manifest(), limits, fuel).expect("host")
    }
    fn run(host: &mut WasmPluginHost, wat: &str) -> std::result::Result<i32, PluginError> {
        host.load_module(&wat::parse_str(wat).expect("WAT compiles"))
            .expect("loads");
        host.execute(
            "run",
            &mut Document::new("plugin"),
            &mut History::new(1 << 20, 2 << 20),
        )
    }
    fn data(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("\\{b:02x}")).collect()
    }
    fn call_module(cap: i32, payload: &[u8], after: &str) -> String {
        format!(
            r#"(module
          (import "petunia" "call" (func $call (param i32 i32 i32) (result i32)))
          (import "petunia" "response_len" (func $len (param i32) (result i32)))
          (import "petunia" "response_read" (func $read (param i32 i32 i32) (result i32)))
          (import "petunia" "release" (func $release (param i32)))
          (memory (export "memory") 1 2)
          (data (i32.const 0) "{}")
          (func (export "run") (result i32) (local $h i32)
            (local.set $h (call $call (i32.const {cap}) (i32.const 0) (i32.const {})))
            {after} (i32.const 7)))"#,
            data(payload),
            payload.len()
        )
    }
    #[test]
    fn permission_requests_never_self_grant_and_version_is_enforced() {
        let mut m = manifest();
        m.capabilities.push(PluginCapability::ResourceRead);
        m.permissions.push(Permission::FilesystemReadSelected);
        assert!(!PluginPolicy::new(&m).may_use(PluginCapability::ResourceRead));
        assert!(
            PluginPolicy::with_grants(&m, &[Permission::FilesystemReadSelected])
                .may_use(PluginCapability::ResourceRead)
        );
        assert!(!PluginPolicy::with_grants(&m, &[Permission::Network])
            .may_use(PluginCapability::ResourceRead));
        m.host_api_version.major += 1;
        assert!(matches!(
            WasmPluginHost::new(&m, PluginLimits::default(), 100),
            Err(PluginError::IncompatibleVersion)
        ));
    }
    #[test]
    fn interpreter_executes_arithmetic_and_fuel_stops_infinite_guest_loop() {
        let mut host = host(PluginLimits::default(), 10_000);
        assert_eq!(
            run(
                &mut host,
                r#"(module (func (export "run") (result i32) (i32.mul (i32.const 6) (i32.const 7))))"#
            ),
            Ok(42)
        );
        assert!(host.fuel_remaining() < 10_000);
        assert_eq!(
            run(
                &mut host,
                r#"(module (func (export "run") (result i32) (loop $again (br $again)) (i32.const 0)))"#
            ),
            Err(PluginError::FuelExhausted)
        );
        assert_eq!(host.fuel_remaining(), 0);
    }
    #[test]
    fn parser_rejects_valid_magic_followed_by_invalid_sections() {
        let mut host = host(PluginLimits::default(), 1000);
        let mut corrupt = WASM_MAGIC.to_vec();
        corrupt.extend([1, 2, 3, 4]);
        assert!(matches!(
            host.load_module(&corrupt),
            Err(PluginError::MalformedModule(_))
        ));
    }
    #[test]
    fn actual_guest_memory_growth_and_oversized_initial_memory_are_limited() {
        let limits = PluginLimits {
            linear_memory_bytes: 65536,
            ..PluginLimits::default()
        };
        let mut h = host(limits, 10000);
        assert!(run(&mut h,r#"(module (memory 1 3) (func (export "run") (result i32) (memory.grow (i32.const 1))))"#).is_err());
        assert!(run(
            &mut h,
            r#"(module (memory 2) (func (export "run") (result i32) (i32.const 0)))"#
        )
        .is_err());
    }
    #[test]
    fn module_imports_cannot_access_wasi_or_undeclared_capabilities() {
        let mut h = host(PluginLimits::default(), 10000);
        assert!(run(&mut h,r#"(module (import "wasi_snapshot_preview1" "fd_write" (func)) (func (export "run") (result i32) (i32.const 0)))"#).is_err());
        let mut m = manifest();
        m.capabilities.clear();
        let mut denied = WasmPluginHost::new(&m, PluginLimits::default(), 10000).expect("host");
        assert_eq!(
            run(&mut denied, &call_module(2, b"test", "")),
            Err(PluginError::PermissionDenied(PluginCapability::LogWrite))
        );
    }
    #[test]
    fn actual_query_reads_document_and_host_handles_copy_json_into_guest_memory() {
        let mut document = Document::new("real");
        let node = SceneNode::new_path(
            "Actual object",
            VectorPath::rect(0.0, 0.0, 1.0, 1.0),
            ParentRef::Page(document.scene.default_page()),
        );
        document.scene.insert_node(node);
        let mut h = host(PluginLimits::default(), 100000);
        let after = r#"(drop (call $call (i32.const 2) (i32.const 2048)
             (call $read (local.get $h) (i32.const 2048) (call $len (local.get $h)))))
             (call $release (local.get $h))"#;
        h.load_module(
            &wat::parse_str(call_module(0, br#"{"offset":0,"limit":10}"#, after)).expect("WAT"),
        )
        .expect("load");
        assert_eq!(
            h.execute("run", &mut document, &mut History::new(1 << 20, 2 << 20)),
            Ok(7)
        );
        let query: serde_json::Value = serde_json::from_str(&h.logs()[0]).expect("query JSON");
        assert_eq!(query["total"], 1);
        assert_eq!(query["nodes"][0]["name"], "Actual object");
    }
    #[test]
    fn submitted_command_commits_through_history_and_undo_restores() {
        let mut document = Document::new("real");
        let node = SceneNode::new_path(
            "Object",
            VectorPath::rect(0.0, 0.0, 1.0, 1.0),
            ParentRef::Page(document.scene.default_page()),
        );
        let id = node.id;
        document.scene.insert_node(node);
        let mut history = History::new(1 << 20, 2 << 20);
        let command = PluginCommand {
            revision: history.current_revision(),
            operations: vec![DocumentOp::SetVisibility {
                object: id,
                visible: false,
            }],
        };
        let mut h = host(PluginLimits::default(), 100000);
        h.load_module(
            &wat::parse_str(call_module(
                1,
                &serde_json::to_vec(&command).expect("JSON"),
                "",
            ))
            .expect("WAT"),
        )
        .expect("load");
        assert_eq!(h.execute("run", &mut document, &mut history), Ok(7));
        assert!(!document.scene.get_node(id).expect("node").visible);
        assert!(history.can_undo());
        history.undo(&mut document).expect("undo");
        assert!(document.scene.get_node(id).expect("node").visible);
    }
    #[test]
    fn guest_trap_discards_previously_staged_commands_and_keeps_history() {
        let mut document = Document::new("real");
        let node = SceneNode::new_path(
            "Object",
            VectorPath::rect(0.0, 0.0, 1.0, 1.0),
            ParentRef::Page(document.scene.default_page()),
        );
        let id = node.id;
        document.scene.insert_node(node);
        let original = document.clone();
        let mut history = History::new(1 << 20, 2 << 20);
        let command = PluginCommand {
            revision: history.current_revision(),
            operations: vec![DocumentOp::SetVisibility {
                object: id,
                visible: false,
            }],
        };
        let mut h = host(PluginLimits::default(), 100000);
        h.load_module(
            &wat::parse_str(call_module(
                1,
                &serde_json::to_vec(&command).expect("JSON"),
                "unreachable",
            ))
            .expect("WAT"),
        )
        .expect("load");
        assert!(h.execute("run", &mut document, &mut history).is_err());
        assert_eq!(document.scene, original.scene);
        assert!(history.is_empty());
    }
    #[test]
    fn released_handles_are_stale_and_handle_message_output_limits_trap() {
        let mut h = host(PluginLimits::default(), 100000);
        assert_eq!(
            run(
                &mut h,
                &call_module(
                    2,
                    b"hi",
                    "(call $release (local.get $h)) (drop (call $len (local.get $h)))"
                )
            ),
            Err(PluginError::InvalidHandle(1))
        );
        let mut h = host(
            PluginLimits {
                max_handles: 0,
                ..PluginLimits::default()
            },
            100000,
        );
        assert!(matches!(
            run(&mut h, &call_module(2, b"hi", "")),
            Err(PluginError::ResourceLimit(_))
        ));
        let mut h = host(
            PluginLimits {
                output_bytes: 1,
                ..PluginLimits::default()
            },
            100000,
        );
        assert!(matches!(
            run(&mut h, &call_module(2, b"hi", "")),
            Err(PluginError::ResourceLimit(_))
        ));
        let mut h = host(PluginLimits::default(), 100000);
        assert!(run(
            &mut h,
            &call_module(
                2,
                b"hi",
                "(drop (call $read (local.get $h) (i32.const 65535) (i32.const 100)))"
            )
        )
        .is_err());
    }
    #[test]
    fn guest_recursion_hits_explicit_stack_limit() {
        let mut h = host(
            PluginLimits {
                max_recursion_depth: 8,
                ..PluginLimits::default()
            },
            100000,
        );
        assert!(run(&mut h,r#"(module (func $recur (result i32) (i32.add (i32.const 1) (call $recur))) (export "run" (func $recur)))"#).is_err());
        assert!(h.fuel_remaining() > 0);
    }
}
