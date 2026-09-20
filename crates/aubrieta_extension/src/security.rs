//! Security boundaries, capability enforcement and sandbox protection (09.14 & 09.28).
//!
//! Enforces:
//! - "A missing capability is a normal state with a disabled reason, never a panic".
//! - Explicit permission brokering before plugins can touch documents, clipboard or storage.
//! - Sandboxed script isolation preventing access to dangerous host OS/process APIs.

use crate::manifest::{PluginId, PluginPermission};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Errors arising from capability enforcement and sandbox boundaries.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum PluginSecurityError {
    /// Attempted operation without the required permission.
    #[error("plugin `{plugin_id}` denied `{permission}`: {reason}")]
    PermissionDenied {
        /// Target plugin identifier.
        plugin_id: PluginId,
        /// Missing required permission.
        permission: PluginPermission,
        /// Human-readable explanation.
        reason: String,
    },

    /// Dangerous sandbox escape or forbidden library access attempted.
    #[error("sandbox security violation: {0}")]
    SandboxViolation(String),

    /// Runtime script evaluation or syntax failure.
    #[error("plugin script execution error: {0}")]
    ScriptError(String),

    /// Plugin ID not recognized by host.
    #[error("plugin `{0}` not found")]
    PluginNotFound(PluginId),
}

/// Central capability broker determining whether operations are authorized.
#[derive(Debug, Clone, Default)]
pub struct CapabilityBroker {
    granted_permissions: HashMap<PluginId, HashSet<PluginPermission>>,
}

impl CapabilityBroker {
    /// Creates an empty broker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers the granted permissions for a plugin.
    pub fn register_plugin(&mut self, plugin_id: PluginId, permissions: HashSet<PluginPermission>) {
        self.granted_permissions.insert(plugin_id, permissions);
    }

    /// Unregisters a plugin upon unload.
    pub fn unregister_plugin(&mut self, plugin_id: &PluginId) {
        self.granted_permissions.remove(plugin_id);
    }

    /// Validates whether a plugin has the requested permission.
    pub fn check_permission(
        &self,
        plugin_id: &PluginId,
        permission: PluginPermission,
    ) -> Result<(), PluginSecurityError> {
        let perms = self
            .granted_permissions
            .get(plugin_id)
            .ok_or_else(|| PluginSecurityError::PluginNotFound(plugin_id.clone()))?;

        if perms.contains(&permission) {
            Ok(())
        } else {
            Err(PluginSecurityError::PermissionDenied {
                plugin_id: plugin_id.clone(),
                permission,
                reason: format!(
                    "plugin manifest did not declare or user did not grant `{permission}`"
                ),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_granted_passes() {
        let mut broker = CapabilityBroker::new();
        let plugin = PluginId::new("test.plugin");
        let mut perms = HashSet::new();
        perms.insert(PluginPermission::DocumentRead);

        broker.register_plugin(plugin.clone(), perms);
        assert!(broker
            .check_permission(&plugin, PluginPermission::DocumentRead)
            .is_ok());
    }

    #[test]
    fn missing_permission_returns_disabled_reason_never_panics() {
        let mut broker = CapabilityBroker::new();
        let plugin = PluginId::new("test.plugin");
        broker.register_plugin(plugin.clone(), HashSet::new());

        let result = broker.check_permission(&plugin, PluginPermission::Clipboard);
        assert!(result.is_err());
        match result.unwrap_err() {
            PluginSecurityError::PermissionDenied {
                permission, reason, ..
            } => {
                assert_eq!(permission, PluginPermission::Clipboard);
                assert!(reason.contains("did not grant `clipboard`"));
            }
            other => panic!("expected PermissionDenied, got {other:?}"),
        }
    }
}
