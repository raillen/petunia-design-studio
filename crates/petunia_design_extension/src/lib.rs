//! Petunia Extension System and Plugin SDK (09.14, 09.28, 09.30).
//!
//! Provides the extension architecture for Petunia Design Studio:
//! - Runtime-neutral Plugin SDK contracts (`PluginManifest`, `PluginPermission`, `PluginId`).
//! - Hardened capability broker enforcing security permissions without panic.
//! - Sandboxed Lua 5.4/5.5 scripting runtime mediated by host capabilities.
//! - Mutations strictly mediated through Action/Command requests.

pub mod host;
pub mod manifest;
pub mod security;

pub use host::{PluginHost, ScriptExecutionRecord};
pub use manifest::{PluginId, PluginManifest, PluginPermission};
pub use security::{CapabilityBroker, PluginSecurityError};
