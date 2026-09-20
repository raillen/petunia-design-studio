//! JSON-RPC 2.0 and MCP Protocol envelopes (09.15 & 09.29).

use serde::{Deserialize, Serialize};

/// Standard JSON-RPC error codes.
pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const INTERNAL_ERROR: i32 = -32603;

/// Aubrieta specific MCP error codes.
pub const STALE_REVISION: i32 = -32001;
pub const MUTATION_FAILED: i32 = -32002;

/// Structured JSON-RPC error representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpError {
    /// Numeric error code.
    pub code: i32,
    /// Human-readable explanation.
    pub message: String,
    /// Optional structured debugging data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl McpError {
    /// Creates a new error.
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Appends structured error data.
    #[must_use]
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

/// Incoming MCP JSON-RPC request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpRequest {
    /// Protocol version ("2.0").
    #[serde(default = "default_jsonrpc")]
    pub jsonrpc: String,
    /// Client request identifier.
    #[serde(default)]
    pub id: serde_json::Value,
    /// Method name (e.g. `document.summary`).
    pub method: String,
    /// Method parameters.
    #[serde(default)]
    pub params: serde_json::Value,
}

fn default_jsonrpc() -> String {
    "2.0".to_string()
}

impl McpRequest {
    /// Creates a new request.
    pub fn new(
        id: impl Into<serde_json::Value>,
        method: impl Into<String>,
        params: serde_json::Value,
    ) -> Self {
        Self {
            jsonrpc: default_jsonrpc(),
            id: id.into(),
            method: method.into(),
            params,
        }
    }
}

/// Outgoing MCP JSON-RPC response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpResponse {
    /// Protocol version ("2.0").
    pub jsonrpc: String,
    /// Correlated request identifier.
    pub id: serde_json::Value,
    /// Result payload if successful.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error payload if failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

impl McpResponse {
    /// Creates a successful response.
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: default_jsonrpc(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Creates an error response.
    pub fn error(id: serde_json::Value, error: McpError) -> Self {
        Self {
            jsonrpc: default_jsonrpc(),
            id,
            result: None,
            error: Some(error),
        }
    }
}
