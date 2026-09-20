//! Stable semantic action identifiers (`aubrieta.*` namespace).

use serde::{Deserialize, Serialize};

/// Namespaced action identifier, e.g. `aubrieta.surface.create`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActionId(pub String);

impl ActionId {
    /// Well-known actions for the P00 MVP slice.
    pub const SURFACE_CREATE: &'static str = "aubrieta.surface.create";
    pub const OBJECT_CREATE: &'static str = "aubrieta.object.create";
    pub const OBJECT_DELETE: &'static str = "aubrieta.object.delete";
    pub const FILL_SET: &'static str = "aubrieta.fill.set";

    /// Creates an action ID. Callers must use the `aubrieta.*` namespace.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// User intent before it becomes an undoable [`crate::Command`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionRequest {
    /// Which operation was requested.
    pub action: ActionId,
    /// Opaque JSON payload interpreted by the command layer.
    pub payload: serde_json::Value,
}

impl ActionRequest {
    /// Creates a request with an explicit payload.
    #[must_use]
    pub fn new(action: ActionId, payload: serde_json::Value) -> Self {
        Self { action, payload }
    }
}
