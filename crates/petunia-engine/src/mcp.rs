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
    let (properties, required) = match name {
        "query_document" => (
            serde_json::json!({"offset":{"type":"integer","minimum":0},"limit":{"type":"integer","minimum":1,"maximum":256}}),
            vec!["limit"],
        ),
        "inspect_resource" => (
            serde_json::json!({"id":{"type":"string","format":"uuid"}}),
            vec!["id"],
        ),
        "request_render" => (
            serde_json::json!({"width":{"type":"integer","minimum":1,"maximum":8192},"height":{"type":"integer","minimum":1,"maximum":8192}}),
            vec!["width", "height"],
        ),
        _ => (
            serde_json::json!({"revision":{"type":"integer","minimum":0},"command":{"type":"object"}}),
            vec!["revision", "command"],
        ),
    };
    McpTool {
        name: name.to_string(),
        description: description.to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "properties": properties,
            "required": required,
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
        match self.tool.as_str() {
            "query_document" => {
                let query: crate::plugins::PluginQuery = decode(self.params.clone())?;
                if query.limit == 0 || query.limit > 256 {
                    return Err(EngineError::Execution("query limit outside 1..256".into()));
                }
            }
            "inspect_resource" => {
                let _: ResourceQuery = decode(self.params.clone())?;
            }
            "request_render" => {
                let _: RenderQuery = decode(self.params.clone())?;
            }
            "submit_command" => {
                let _: Submission = decode(self.params.clone())?;
            }
            _ => return Err(EngineError::Execution("unknown MCP tool".into())),
        }
        Ok(())
    }
}

/// A command submission through the adapter: operations plus their
/// stable description, ready to become one transaction request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
        if (self.destructive || self.operations.iter().any(operation_is_destructive))
            && config.require_confirmation_for_destructive
            && !confirmed
        {
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

/// The application injects its renderer; the adapter only sees immutable
/// snapshot DTOs. No fake render result is returned when a service is absent.
pub trait McpRenderService {
    fn render(
        &mut self,
        snapshot: &petunia_render_model::RenderSnapshot,
        width: u32,
        height: u32,
    ) -> Result<serde_json::Value>;
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceQuery {
    id: petunia_core::ResourceId,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RenderQuery {
    width: u32,
    height: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Submission {
    revision: crate::DocumentRevision,
    command: McpCommandSubmit,
}

fn decode<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> Result<T> {
    serde_json::from_value(value).map_err(|e| EngineError::Execution(e.to_string()))
}

/// Execute a local adapter request. Confirmation is supplied by the trusted
/// application separately from client-controlled JSON. Mutations always prepare
/// against a full revision and commit through the shared History lane.
pub fn execute_call(
    config: &McpServerConfig,
    call: McpCall,
    document: &mut petunia_core::Document,
    history: &mut crate::History,
    confirmed: bool,
    renderer: Option<&mut dyn McpRenderService>,
) -> Result<serde_json::Value> {
    if !config.enabled {
        return Err(EngineError::Execution("MCP server is disabled".into()));
    }
    call.validate()?;
    let bytes =
        serde_json::to_vec(&call.params).map_err(|e| EngineError::Execution(e.to_string()))?;
    if bytes.len() > 1 << 20 {
        return Err(EngineError::BudgetExceeded(
            "MCP message exceeds 1 MiB".into(),
        ));
    }
    match call.tool.as_str() {
        "query_document" => {
            let query: crate::plugins::PluginQuery = decode(call.params)?;
            if query.limit == 0 || query.limit > 256 {
                return Err(EngineError::Execution(
                    "query page must contain 1..256 nodes".into(),
                ));
            }
            let nodes: Vec<_> = document
                .scene
                .root_lists()
                .into_iter()
                .flatten()
                .flat_map(|id| std::iter::once(*id).chain(document.scene.descendants(*id)))
                .skip(query.offset)
                .take(query.limit)
                .filter_map(|id| document.scene.get_node(id))
                .map(|n| serde_json::json!({"id":n.id,"name":n.name,"visible":n.visible}))
                .collect();
            Ok(
                serde_json::json!({"revision":history.current_revision(),"total":document.scene.len(),"nodes":nodes}),
            )
        }
        "inspect_resource" => {
            let query: ResourceQuery = decode(call.params)?;
            let record = document
                .resources
                .get(query.id)
                .ok_or_else(|| EngineError::Execution("resource not found".into()))?;
            serde_json::to_value(record).map_err(|e| EngineError::Execution(e.to_string()))
        }
        "request_render" => {
            let query: RenderQuery = decode(call.params)?;
            if query.width == 0
                || query.height == 0
                || query.width > 8192
                || query.height > 8192
                || u64::from(query.width) * u64::from(query.height) > 16 << 20
            {
                return Err(EngineError::BudgetExceeded(
                    "invalid or excessive render dimensions".into(),
                ));
            }
            let renderer = renderer
                .ok_or_else(|| EngineError::Execution("render service unavailable".into()))?;
            let (snapshot, warnings) = crate::compile_document(
                document,
                history.current_revision(),
                petunia_render_model::RenderQuality::Export,
            );
            let output = renderer.render(&snapshot, query.width, query.height)?;
            Ok(serde_json::json!({"output":output,"warnings":warnings}))
        }
        "submit_command" => {
            let submit: Submission = decode(call.params)?;
            if submit.revision != history.current_revision() {
                return Err(EngineError::Execution("stale MCP document revision".into()));
            }
            let description = submit.command.description;
            let request = submit.command.into_request(config, confirmed)?;
            let prepared = crate::prepare_transaction(document, request, submit.revision)
                .map_err(|e| EngineError::Execution(e.to_string()))?;
            let revision = history.commit(document, prepared, description)?;
            Ok(serde_json::json!({"committed":true,"revision":revision}))
        }
        _ => Err(EngineError::Execution("unknown MCP tool".into())),
    }
}

fn operation_is_destructive(op: &DocumentOp) -> bool {
    use crate::transaction::RegistryOp;
    matches!(
        op,
        DocumentOp::RemoveSubtree { .. }
            | DocumentOp::RemoveObjects { .. }
            | DocumentOp::Registry(
                RegistryOp::Resource { value: None, .. }
                    | RegistryOp::Style { value: None, .. }
                    | RegistryOp::Symbol { value: None, .. }
                    | RegistryOp::Swatch { value: None, .. }
                    | RegistryOp::Spot { value: None, .. }
            )
    )
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
            params: serde_json::json!({"limit":10}),
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

#[cfg(test)]
mod execution_tests {
    use super::*;
    use crate::{DocumentRevision, History};
    use petunia_core::{Document, ParentRef, SceneNode, VectorPath};
    fn config() -> McpServerConfig {
        McpServerConfig {
            enabled: true,
            ..McpServerConfig::default()
        }
    }
    fn setup() -> (Document, History, petunia_core::ObjectId) {
        let mut document = Document::new("MCP");
        let node = SceneNode::new_path(
            "Real object",
            VectorPath::rect(0.0, 0.0, 5.0, 5.0),
            ParentRef::Page(document.scene.default_page()),
        );
        let id = node.id;
        document.scene.insert_node(node);
        (document, History::new(1 << 20, 2 << 20), id)
    }
    #[test]
    fn actual_query_is_paginated_and_unknown_fields_rejected() {
        let (mut document, mut history, _) = setup();
        let response = execute_call(
            &config(),
            McpCall {
                tool: "query_document".into(),
                params: serde_json::json!({"limit":1}),
            },
            &mut document,
            &mut history,
            false,
            None,
        )
        .expect("query");
        assert_eq!(response["total"], 1);
        assert_eq!(response["nodes"][0]["name"], "Real object");
        assert!(McpCall {
            tool: "query_document".into(),
            params: serde_json::json!({"limit":1,"path":"/secret"})
        }
        .validate()
        .is_err());
    }
    #[test]
    fn client_cannot_hide_destructive_command_and_host_confirmation_commits_with_undo() {
        let (mut document, mut history, id) = setup();
        let submit = McpCommandSubmit {
            description: HistoryDescription::DeleteObjects,
            operations: vec![DocumentOp::RemoveSubtree { root: id }],
            destructive: false,
        };
        let call = McpCall {
            tool: "submit_command".into(),
            params: serde_json::json!({"revision":0,"command":submit}),
        };
        assert!(execute_call(
            &config(),
            call.clone(),
            &mut document,
            &mut history,
            false,
            None
        )
        .is_err());
        assert_eq!(document.scene.len(), 1);
        assert!(history.is_empty());
        let response = execute_call(&config(), call, &mut document, &mut history, true, None)
            .expect("confirmed commit");
        assert_eq!(response["committed"], true);
        assert!(document.scene.is_empty());
        history.undo(&mut document).expect("undo");
        assert_eq!(document.scene.len(), 1);
    }
    #[test]
    fn stale_command_unknown_resource_and_missing_render_service_fail() {
        let (mut document, mut history, id) = setup();
        let submit = McpCommandSubmit {
            description: HistoryDescription::SetVisibility,
            operations: vec![DocumentOp::SetVisibility {
                object: id,
                visible: false,
            }],
            destructive: false,
        };
        let call = McpCall {
            tool: "submit_command".into(),
            params: serde_json::json!({"revision":DocumentRevision(42),"command":submit}),
        };
        assert!(execute_call(&config(), call, &mut document, &mut history, false, None).is_err());
        let call = McpCall {
            tool: "inspect_resource".into(),
            params: serde_json::json!({"id":petunia_core::ResourceId::new_v4()}),
        };
        assert!(execute_call(&config(), call, &mut document, &mut history, false, None).is_err());
        let call = McpCall {
            tool: "request_render".into(),
            params: serde_json::json!({"width":100,"height":100}),
        };
        assert!(execute_call(&config(), call, &mut document, &mut history, false, None).is_err());
        assert!(document.scene.get_node(id).expect("node").visible);
        assert!(history.is_empty());
    }
}
