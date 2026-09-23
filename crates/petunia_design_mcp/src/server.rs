//! MCP Server implementing discovery, document inspection, mutation and export (09.15 & 09.29).

use crate::protocol::{
    McpError, McpRequest, McpResponse, INTERNAL_ERROR, INVALID_PARAMS, METHOD_NOT_FOUND,
    MUTATION_FAILED, PARSE_ERROR, STALE_REVISION,
};
use petunia_design_application::{Command, CommandRequest, History};
use petunia_design_document::Document;
use petunia_design_evaluation::Evaluator;
use petunia_design_foundation::{IdGenerator, ObjectId, SurfaceId, NATIVE_SCHEMA_VERSION};
use serde_json::json;

/// Autonomous MCP server allowing AI agents to discover, query, and edit documents.
pub struct McpServer {
    document: Document,
    history: History,
    evaluator: Evaluator,
    ids: IdGenerator,
    revision: u64,
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl McpServer {
    /// Creates a new MCP server with an empty document and fresh history.
    #[must_use]
    pub fn new() -> Self {
        Self {
            document: Document::new(),
            history: History::new(100),
            evaluator: Evaluator::new(),
            ids: IdGenerator::new(),
            revision: 1,
        }
    }

    /// Sets an initial document snapshot and synchronizes ID generation.
    pub fn with_document(mut self, doc: Document) -> Self {
        let max_id = doc
            .surfaces()
            .iter()
            .map(|s| s.id.raw())
            .chain(
                doc.surfaces()
                    .iter()
                    .flat_map(|s| s.objects().iter().map(|o| o.id.raw())),
            )
            .max()
            .unwrap_or(0);
        self.ids = IdGenerator::with_start(max_id + 1);
        self.document = doc;
        self
    }

    /// Sets an explicit ID generator.
    pub fn with_ids(mut self, ids: IdGenerator) -> Self {
        self.ids = ids;
        self
    }

    /// Returns a reference to the current document state.
    #[must_use]
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Returns the current document revision number.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Processes a raw JSON-RPC string and returns the serialized response string.
    #[must_use]
    pub fn dispatch_json(&mut self, json_str: &str) -> String {
        let req: Result<McpRequest, _> = serde_json::from_str(json_str);
        let resp = match req {
            Ok(r) => self.dispatch(r),
            Err(e) => McpResponse::error(
                serde_json::Value::Null,
                McpError::new(PARSE_ERROR, format!("JSON parse error: {e}")),
            ),
        };
        serde_json::to_string(&resp).unwrap_or_else(|_| "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":-32603,\"message\":\"serialization failed\"}}".to_string())
    }

    /// Dispatches a typed MCP request to the appropriate semantic handler.
    pub fn dispatch(&mut self, req: McpRequest) -> McpResponse {
        let id = req.id.clone();
        match req.method.as_str() {
            "ptnd.discover" => self.handle_discover(id),
            "document.summary" => self.handle_summary(id),
            "document.create_surface" => self.handle_create_surface(id, req.params),
            "document.create_object" => self.handle_create_object(id, req.params),
            "document.set_fill" => self.handle_set_fill(id, req.params),
            "document.set_bounds" => self.handle_set_bounds(id, req.params),
            "document.set_opacity" => self.handle_set_opacity(id, req.params),
            "document.delete_object" => self.handle_delete_object(id, req.params),
            "document.align_objects" => self.handle_align_objects(id, req.params),
            "document.distribute_objects" => self.handle_distribute_objects(id, req.params),
            "document.arrange_object" => self.handle_arrange_object(id, req.params),
            "document.undo" => self.handle_undo(id),
            "document.redo" => self.handle_redo(id),
            "export.svg" => self.handle_export_svg(id),
            unknown => McpResponse::error(
                id,
                McpError::new(
                    METHOD_NOT_FOUND,
                    format!("method `{unknown}` is not supported"),
                ),
            ),
        }
    }

    /// Runs one command through the shared `History` (F-19, F-22).
    /// NoOp change sets neither bump the revision nor clear redo; only real
    /// commits advance the revision and evaluator generation.
    fn run_command(&mut self, cmd: Command) -> Result<petunia_design_document::ChangeSet, McpError> {
        match self
            .history
            .execute(&mut self.document, &CommandRequest::new(cmd))
        {
            Ok(changes) => {
                if changes.is_empty() {
                    Ok(changes)
                } else {
                    self.evaluator.note_changes(&changes);
                    self.revision += 1;
                    Ok(changes)
                }
            }
            Err(e) => Err(McpError::new(MUTATION_FAILED, e.to_string())),
        }
    }

    fn check_revision(&self, params: &serde_json::Value) -> Result<(), McpError> {
        if let Some(expected) = params
            .get("expected_revision")
            .and_then(serde_json::Value::as_u64)
        {
            if expected != self.revision {
                return Err(McpError::new(
                    STALE_REVISION,
                    format!(
                        "stale revision: expected revision {expected}, but server is at revision {}",
                        self.revision
                    ),
                ));
            }
        }
        Ok(())
    }

    fn handle_discover(&self, id: serde_json::Value) -> McpResponse {
        let capabilities = vec![
            "document.summary",
            "document.create_surface",
            "document.create_object",
            "document.set_fill",
            "document.set_bounds",
            "document.set_opacity",
            "document.delete_object",
            "document.align_objects",
            "document.distribute_objects",
            "document.arrange_object",
            "document.undo",
            "document.redo",
            "export.svg",
        ];

        McpResponse::success(
            id,
            json!({
                "app": "Petunia Design Studio",
                "version": "0.1.0",
                "schema_version": NATIVE_SCHEMA_VERSION,
                "capabilities": capabilities,
                "current_revision": self.revision,
            }),
        )
    }

    fn handle_summary(&mut self, id: serde_json::Value) -> McpResponse {
        let summary = self.evaluator.evaluate(&self.document);
        McpResponse::success(
            id,
            json!({
                "surfaces": summary.surfaces,
                "objects": summary.objects,
                "generation": summary.generation,
                "revision": self.revision,
            }),
        )
    }

    fn handle_create_surface(
        &mut self,
        id: serde_json::Value,
        params: serde_json::Value,
    ) -> McpResponse {
        if let Err(e) = self.check_revision(&params) {
            return McpResponse::error(id, e);
        }

        let name = params
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("New Surface")
            .to_string();

        let surface_id = self.ids.next_surface();
        let cmd = Command::CreateSurface {
            id: surface_id,
            name: name.clone(),
        };

        match self.run_command(cmd) {
            Ok(_) => McpResponse::success(
                id,
                json!({
                    "surface_id": surface_id.raw(),
                    "name": name,
                    "revision": self.revision,
                }),
            ),
            Err(e) => McpResponse::error(id, e),
        }
    }

    fn handle_create_object(
        &mut self,
        id: serde_json::Value,
        params: serde_json::Value,
    ) -> McpResponse {
        if let Err(e) = self.check_revision(&params) {
            return McpResponse::error(id, e);
        }

        let surface_raw = match params.get("surface_id").and_then(serde_json::Value::as_u64) {
            Some(s) => s,
            None => {
                return McpResponse::error(
                    id,
                    McpError::new(INVALID_PARAMS, "missing required parameter `surface_id`"),
                )
            }
        };

        let name = params
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Rectangle")
            .to_string();

        let object_id = self.ids.next_object();
        let cmd = Command::CreateObject {
            surface: SurfaceId::new(surface_raw),
            id: object_id,
            name: name.clone(),
        };

        match self.run_command(cmd) {
            Ok(_) => McpResponse::success(
                id,
                json!({
                    "object_id": object_id.raw(),
                    "surface_id": surface_raw,
                    "name": name,
                    "revision": self.revision,
                }),
            ),
            Err(e) => McpResponse::error(id, e),
        }
    }

    fn handle_set_fill(&mut self, id: serde_json::Value, params: serde_json::Value) -> McpResponse {
        if let Err(e) = self.check_revision(&params) {
            return McpResponse::error(id, e);
        }

        let object_raw = match params.get("object_id").and_then(serde_json::Value::as_u64) {
            Some(o) => o,
            None => {
                return McpResponse::error(
                    id,
                    McpError::new(INVALID_PARAMS, "missing required parameter `object_id`"),
                )
            }
        };

        let fill = params
            .get("fill")
            .and_then(serde_json::Value::as_str)
            .map(std::string::ToString::to_string);

        let cmd = Command::SetFill {
            id: ObjectId::new(object_raw),
            fill: fill.clone(),
        };

        match self.run_command(cmd) {
            Ok(_) => McpResponse::success(
                id,
                json!({
                    "object_id": object_raw,
                    "fill": fill,
                    "revision": self.revision,
                }),
            ),
            Err(e) => McpResponse::error(id, e),
        }
    }

    fn handle_set_bounds(&mut self, id: serde_json::Value, params: serde_json::Value) -> McpResponse {
        if let Err(e) = self.check_revision(&params) {
            return McpResponse::error(id, e);
        }
        let object_raw = match params.get("object_id").and_then(serde_json::Value::as_u64) {
            Some(o) => o,
            None => {
                return McpResponse::error(
                    id,
                    McpError::new(INVALID_PARAMS, "missing required parameter `object_id`"),
                )
            }
        };
        let bounds = params.get("bounds").and_then(|b| {
            let arr = b.as_array()?;
            if arr.len() != 4 {
                return None;
            }
            let mut out = [0.0f64; 4];
            for (i, v) in arr.iter().enumerate() {
                out[i] = v.as_f64()?;
            }
            Some(out)
        });
        let rotation = params
            .get("rotation")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let cmd = Command::SetBounds {
            id: ObjectId::new(object_raw),
            bounds,
            rotation,
        };
        match self.run_command(cmd) {
            Ok(_) => McpResponse::success(
                id,
                json!({ "object_id": object_raw, "revision": self.revision }),
            ),
            Err(e) => McpResponse::error(id, e),
        }
    }

    fn handle_set_opacity(
        &mut self,
        id: serde_json::Value,
        params: serde_json::Value,
    ) -> McpResponse {
        if let Err(e) = self.check_revision(&params) {
            return McpResponse::error(id, e);
        }
        let object_raw = match params.get("object_id").and_then(serde_json::Value::as_u64) {
            Some(o) => o,
            None => {
                return McpResponse::error(
                    id,
                    McpError::new(INVALID_PARAMS, "missing required parameter `object_id`"),
                )
            }
        };
        let opacity = match params.get("opacity").and_then(serde_json::Value::as_f64) {
            Some(o) => o,
            None => {
                return McpResponse::error(
                    id,
                    McpError::new(INVALID_PARAMS, "missing required parameter `opacity`"),
                )
            }
        };
        let cmd = Command::SetOpacity {
            id: ObjectId::new(object_raw),
            opacity,
        };
        match self.run_command(cmd) {
            Ok(_) => McpResponse::success(
                id,
                json!({ "object_id": object_raw, "revision": self.revision }),
            ),
            Err(e) => McpResponse::error(id, e),
        }
    }

    fn handle_delete_object(
        &mut self,
        id: serde_json::Value,
        params: serde_json::Value,
    ) -> McpResponse {
        if let Err(e) = self.check_revision(&params) {
            return McpResponse::error(id, e);
        }
        let object_raw = match params.get("object_id").and_then(serde_json::Value::as_u64) {
            Some(o) => o,
            None => {
                return McpResponse::error(
                    id,
                    McpError::new(INVALID_PARAMS, "missing required parameter `object_id`"),
                )
            }
        };
        let cmd = Command::DeleteObject {
            id: ObjectId::new(object_raw),
        };
        match self.run_command(cmd) {
            Ok(_) => McpResponse::success(
                id,
                json!({ "object_id": object_raw, "deleted": true, "revision": self.revision }),
            ),
            Err(e) => McpResponse::error(id, e),
        }
    }

    fn parse_object_ids(params: &serde_json::Value) -> Option<Vec<ObjectId>> {
        params
            .get("object_ids")?
            .as_array()?
            .iter()
            .map(|v| v.as_u64().map(ObjectId::new))
            .collect()
    }

    fn handle_align_objects(
        &mut self,
        id: serde_json::Value,
        params: serde_json::Value,
    ) -> McpResponse {
        if let Err(e) = self.check_revision(&params) {
            return McpResponse::error(id, e);
        }
        let surface_raw = match params.get("surface_id").and_then(serde_json::Value::as_u64) {
            Some(s) => s,
            None => {
                return McpResponse::error(
                    id,
                    McpError::new(INVALID_PARAMS, "missing required parameter `surface_id`"),
                )
            }
        };
        let ids = match Self::parse_object_ids(&params) {
            Some(v) if v.len() >= 2 => v,
            _ => {
                return McpResponse::error(
                    id,
                    McpError::new(
                        INVALID_PARAMS,
                        "parameter `object_ids` requires at least 2 ids",
                    ),
                )
            }
        };
        let mode = match params.get("mode").and_then(serde_json::Value::as_str) {
            Some("left") => petunia_design_document::AlignmentMode::Left,
            Some("center") => petunia_design_document::AlignmentMode::Center,
            Some("right") => petunia_design_document::AlignmentMode::Right,
            Some("top") => petunia_design_document::AlignmentMode::Top,
            Some("middle") => petunia_design_document::AlignmentMode::Middle,
            Some("bottom") => petunia_design_document::AlignmentMode::Bottom,
            _ => {
                return McpResponse::error(
                    id,
                    McpError::new(
                        INVALID_PARAMS,
                        "parameter `mode` must be left|center|right|top|middle|bottom",
                    ),
                )
            }
        };
        let cmd = Command::AlignObjects {
            surface: SurfaceId::new(surface_raw),
            ids,
            mode,
        };
        match self.run_command(cmd) {
            Ok(changes) => McpResponse::success(
                id,
                json!({ "committed": !changes.is_empty(), "revision": self.revision }),
            ),
            Err(e) => McpResponse::error(id, e),
        }
    }

    fn handle_distribute_objects(
        &mut self,
        id: serde_json::Value,
        params: serde_json::Value,
    ) -> McpResponse {
        if let Err(e) = self.check_revision(&params) {
            return McpResponse::error(id, e);
        }
        let surface_raw = match params.get("surface_id").and_then(serde_json::Value::as_u64) {
            Some(s) => s,
            None => {
                return McpResponse::error(
                    id,
                    McpError::new(INVALID_PARAMS, "missing required parameter `surface_id`"),
                )
            }
        };
        let ids = match Self::parse_object_ids(&params) {
            Some(v) if v.len() >= 3 => v,
            _ => {
                return McpResponse::error(
                    id,
                    McpError::new(
                        INVALID_PARAMS,
                        "parameter `object_ids` requires at least 3 ids",
                    ),
                )
            }
        };
        let axis = match params.get("axis").and_then(serde_json::Value::as_str) {
            Some("horizontal") => petunia_design_document::DistributionAxis::Horizontal,
            Some("vertical") => petunia_design_document::DistributionAxis::Vertical,
            _ => {
                return McpResponse::error(
                    id,
                    McpError::new(INVALID_PARAMS, "parameter `axis` must be horizontal|vertical"),
                )
            }
        };
        let cmd = Command::DistributeObjects {
            surface: SurfaceId::new(surface_raw),
            ids,
            axis,
        };
        match self.run_command(cmd) {
            Ok(changes) => McpResponse::success(
                id,
                json!({ "committed": !changes.is_empty(), "revision": self.revision }),
            ),
            Err(e) => McpResponse::error(id, e),
        }
    }

    fn handle_arrange_object(
        &mut self,
        id: serde_json::Value,
        params: serde_json::Value,
    ) -> McpResponse {
        if let Err(e) = self.check_revision(&params) {
            return McpResponse::error(id, e);
        }
        let surface_raw = match params.get("surface_id").and_then(serde_json::Value::as_u64) {
            Some(s) => s,
            None => {
                return McpResponse::error(
                    id,
                    McpError::new(INVALID_PARAMS, "missing required parameter `surface_id`"),
                )
            }
        };
        let object_raw = match params.get("object_id").and_then(serde_json::Value::as_u64) {
            Some(o) => o,
            None => {
                return McpResponse::error(
                    id,
                    McpError::new(INVALID_PARAMS, "missing required parameter `object_id`"),
                )
            }
        };
        let position = match params.get("position").and_then(serde_json::Value::as_str) {
            Some("front") => petunia_design_document::ArrangePosition::Front,
            Some("back") => petunia_design_document::ArrangePosition::Back,
            Some("forward") => petunia_design_document::ArrangePosition::Forward,
            Some("backward") => petunia_design_document::ArrangePosition::Backward,
            _ => {
                return McpResponse::error(
                    id,
                    McpError::new(
                        INVALID_PARAMS,
                        "parameter `position` must be front|back|forward|backward",
                    ),
                )
            }
        };
        let cmd = Command::ArrangeObject {
            surface: SurfaceId::new(surface_raw),
            id: ObjectId::new(object_raw),
            position,
        };
        match self.run_command(cmd) {
            Ok(changes) => McpResponse::success(
                id,
                json!({ "committed": !changes.is_empty(), "revision": self.revision }),
            ),
            Err(e) => McpResponse::error(id, e),
        }
    }

    fn handle_undo(&mut self, id: serde_json::Value) -> McpResponse {
        match self.history.undo(&mut self.document) {
            Ok(undone) => {
                if undone {
                    self.revision += 1;
                }
                McpResponse::success(
                    id,
                    json!({
                        "undone": undone,
                        "revision": self.revision,
                    }),
                )
            }
            Err(e) => McpResponse::error(id, McpError::new(INTERNAL_ERROR, e.to_string())),
        }
    }

    fn handle_redo(&mut self, id: serde_json::Value) -> McpResponse {
        match self.history.redo(&mut self.document) {
            Ok(redone) => {
                if redone {
                    self.revision += 1;
                }
                McpResponse::success(
                    id,
                    json!({
                        "redone": redone,
                        "revision": self.revision,
                    }),
                )
            }
            Err(e) => McpResponse::error(id, McpError::new(INTERNAL_ERROR, e.to_string())),
        }
    }

    fn handle_export_svg(&self, id: serde_json::Value) -> McpResponse {
        let svg = petunia_design_io::export_document_svg(&self.document);
        let len = svg.len();
        McpResponse::success(
            id,
            json!({
                "svg": svg,
                "length": len,
                "revision": self.revision,
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_returns_valid_capabilities() {
        let mut server = McpServer::new();
        let req = McpRequest::new(1, "ptnd.discover", json!({}));
        let resp = server.dispatch(req);

        assert!(resp.error.is_none());
        let res = resp.result.unwrap();
        assert_eq!(res["app"], "Petunia Design Studio");
        assert!(res["capabilities"].as_array().unwrap().len() >= 5);
    }

    #[test]
    fn full_crud_mutation_and_stale_revision_detection() {
        let mut server = McpServer::new();

        // 1. Create surface
        let req1 = McpRequest::new(
            1,
            "document.create_surface",
            json!({ "name": "Artboard 1" }),
        );
        let resp1 = server.dispatch(req1);
        let r1 = resp1.result.unwrap();
        let surface_id = r1["surface_id"].as_u64().unwrap();
        let rev = r1["revision"].as_u64().unwrap();

        // 2. Create object
        let req2 = McpRequest::new(
            2,
            "document.create_object",
            json!({ "surface_id": surface_id, "name": "Logo", "expected_revision": rev }),
        );
        let resp2 = server.dispatch(req2);
        let r2 = resp2.result.unwrap();
        let object_id = r2["object_id"].as_u64().unwrap();
        let new_rev = r2["revision"].as_u64().unwrap();

        // 3. Stale revision rejection
        let stale_req = McpRequest::new(
            3,
            "document.set_fill",
            json!({ "object_id": object_id, "fill": "blue", "expected_revision": rev }), // Old revision!
        );
        let stale_resp = server.dispatch(stale_req);
        assert!(stale_resp.error.is_some());
        assert_eq!(stale_resp.error.unwrap().code, STALE_REVISION);

        // 4. Valid revision acceptance
        let valid_req = McpRequest::new(
            4,
            "document.set_fill",
            json!({ "object_id": object_id, "fill": "ptnd.blue/500", "expected_revision": new_rev }),
        );
        let valid_resp = server.dispatch(valid_req);
        assert!(valid_resp.error.is_none());

        // 5. Export SVG
        let export_req = McpRequest::new(5, "export.svg", json!({}));
        let export_resp = server.dispatch(export_req);
        assert!(export_resp.result.unwrap()["svg"]
            .as_str()
            .unwrap()
            .contains("<svg"));
    }
}
