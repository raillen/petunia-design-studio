//! MCP adapter: external automation over Query/Command APIs.
//!
//! The adapter exposes a fixed tool catalog with JSON schemas and
//! turns validated calls into engine requests. The server stays
//! disabled by default, local-only, and blind to Qt internals:
//! mutations always travel as commands, destructive ones stay
//! explicit, and sensitive actions can require application policy.

use crate::error::{EngineError, Result};
use crate::history::HistoryDescription;
use crate::transaction::{CommandId, DocumentOp, TransactionRequest};
use serde::{Deserialize, Serialize};

/// Transport policy. Local-only unless the application explicitly
/// opts into more.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum McpTransport {
    #[default]
    LocalOnly,
    LoopbackTcp {
        port: u16,
    },
}

/// Server configuration: disabled until the application or user
/// starts it on purpose.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub enabled: bool,
    pub transport: McpTransport,
    pub require_confirmation_for_destructive: bool,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            transport: McpTransport::LocalOnly,
            require_confirmation_for_destructive: true,
        }
    }
}

/// One catalog tool with its JSON schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

fn tool(name: &str, description: &str) -> McpTool {
    McpTool {
        name: name.to_string(),
        description: description.to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "additionalProperties": true,
        }),
    }
}

/// The fixed adapter catalog: reads, inspection, render requests and
/// command submission. No filesystem, network or Qt surface.
#[must_use]
pub fn tool_catalog() -> Vec<McpTool> {
    vec![
        tool(
            "query_document",
            "Read document structure and revisions through validated snapshots.",
        ),
        tool(
            "inspect_resource",
            "Inspect one resource record and its resolution state.",
        ),
        tool(
            "request_render",
            "Ask the render service for an output of a snapshot region.",
        ),
        tool(
            "submit_command",
            "Submit operations as one atomic command; destructive ones stay explicit.",
        ),
    ]
}

/// A validated call into the catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpCall {
    pub tool: String,
    pub params: serde_json::Value,
}

impl McpCall {
    /// Reject unknown tools and non-object params before anything runs.
    pub fn validate(&self) -> Result<()> {
        if !tool_catalog().iter().any(|tool| tool.name == self.tool) {
            return Err(EngineError::Execution(format!(
                "unknown MCP tool: {}",
                self.tool
            )));
        }
        if !self.params.is_object() {
            return Err(EngineError::Execution(
                "MCP params must be a JSON object".to_string(),
            ));
        }
        Ok(())
    }
}

/// A command submission through the adapter: operations plus their
/// stable description, ready to become one transaction request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpCommandSubmit {
    pub description: HistoryDescription,
    pub operations: Vec<DocumentOp>,
    pub destructive: bool,
}

impl McpCommandSubmit {
    /// Build the transaction request. Destructive submissions need
    /// explicit confirmation under a strict server policy.
    pub fn into_request(
        self,
        config: &McpServerConfig,
        confirmed: bool,
    ) -> Result<TransactionRequest> {
        if !config.enabled {
            return Err(EngineError::Execution("MCP server is disabled".to_string()));
        }
        if self.operations.is_empty() {
            return Err(EngineError::Execution(
                "MCP submissions need at least one operation".to_string(),
            ));
        }
        if self.destructive && config.require_confirmation_for_destructive && !confirmed {
            return Err(EngineError::Execution(
                "destructive MCP command needs explicit confirmation".to_string(),
            ));
        }
        Ok(TransactionRequest {
            command_id: CommandId::new_v4(),
            operations: self.operations,
            merge_key: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_core::{SceneNode, VectorPath};

    #[test]
    fn server_is_disabled_by_default() {
        let config = McpServerConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.transport, McpTransport::LocalOnly);
        assert!(config.require_confirmation_for_destructive);
    }

    #[test]
    fn catalog_has_four_tools_and_validates_calls() {
        assert_eq!(tool_catalog().len(), 4);
        let good = McpCall {
            tool: "query_document".to_string(),
            params: serde_json::json!({}),
        };
        assert!(good.validate().is_ok());
        let unknown = McpCall {
            tool: "format_disk".to_string(),
            params: serde_json::json!({}),
        };
        assert!(unknown.validate().is_err());
        let sloppy = McpCall {
            tool: "query_document".to_string(),
            params: serde_json::json!([]),
        };
        assert!(sloppy.validate().is_err());
    }

    #[test]
    fn destructive_submissions_need_confirmation() {
        let config = McpServerConfig {
            enabled: true,
            ..McpServerConfig::default()
        };
        let node = SceneNode::new_path(
            "box",
            VectorPath::rect(0.0, 0.0, 5.0, 5.0),
            petunia_core::ParentRef::Page(petunia_core::PageId::new_v4()),
        );
        let submit = McpCommandSubmit {
            description: HistoryDescription::DeleteObjects,
            operations: vec![DocumentOp::RemoveSubtree { root: node.id }],
            destructive: true,
        };
        assert!(submit.clone().into_request(&config, false).is_err());
        let request = submit.into_request(&config, true).expect("confirmed");
        assert_eq!(request.operations.len(), 1);
        let disabled = McpServerConfig::default();
        let submit = McpCommandSubmit {
            description: HistoryDescription::InsertObjects,
            operations: vec![DocumentOp::RemoveSubtree { root: node.id }],
            destructive: false,
        };
        assert!(submit.into_request(&disabled, true).is_err());
    }
}
