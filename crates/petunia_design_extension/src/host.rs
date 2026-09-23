//! Scripting host executing sandboxed Lua plugins with host capability mediation (09.28 & 09.30).

use crate::manifest::{PluginId, PluginManifest, PluginPermission};
use crate::security::{CapabilityBroker, PluginSecurityError};
use petunia_design_application::{ActionId, ActionRequest};
use petunia_design_document::Document;
use mlua::{Lua, LuaOptions, StdLib};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Action execution callback or recorded requests.
#[derive(Debug, Clone, Default)]
pub struct ScriptExecutionRecord {
    /// Recorded action requests triggered by scripts.
    pub actions_requested: Vec<ActionRequest>,
    /// Log messages emitted by scripts.
    pub logs: Vec<String>,
}

/// Scripting host managing isolated, secure Lua runtimes for extensions.
pub struct PluginHost {
    broker: CapabilityBroker,
    plugins: HashMap<PluginId, PluginManifest>,
    lua_states: HashMap<PluginId, Lua>,
    shared_record: Arc<Mutex<ScriptExecutionRecord>>,
    in_memory_clipboard: Arc<Mutex<Option<String>>>,
    document_state: Arc<Mutex<Document>>,
}

impl Default for PluginHost {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginHost {
    /// Creates a new PluginHost.
    #[must_use]
    pub fn new() -> Self {
        Self {
            broker: CapabilityBroker::new(),
            plugins: HashMap::new(),
            lua_states: HashMap::new(),
            shared_record: Arc::new(Mutex::new(ScriptExecutionRecord::default())),
            in_memory_clipboard: Arc::new(Mutex::new(None)),
            document_state: Arc::new(Mutex::new(Document::new())),
        }
    }

    /// Sets the active document snapshot for query mediation.
    pub fn set_document(&self, doc: Document) {
        if let Ok(mut lock) = self.document_state.lock() {
            *lock = doc;
        }
    }

    /// Sets the clipboard content for clipboard capability mediation.
    pub fn set_clipboard_text(&self, text: Option<String>) {
        if let Ok(mut lock) = self.in_memory_clipboard.lock() {
            *lock = text;
        }
    }

    /// Returns recorded action requests and logs.
    pub fn take_record(&self) -> ScriptExecutionRecord {
        let mut lock = self.shared_record.lock().unwrap();
        let current = lock.clone();
        lock.actions_requested.clear();
        lock.logs.clear();
        current
    }

    /// Loads and initializes a plugin in an isolated sandbox.
    pub fn load_plugin(
        &mut self,
        manifest: PluginManifest,
        script_source: &str,
    ) -> Result<(), PluginSecurityError> {
        let plugin_id = manifest.id.clone();
        self.broker
            .register_plugin(plugin_id.clone(), manifest.permissions.clone());

        // Create hardened sandbox: Only safe standard libraries (TABLE, STRING, MATH, UTF8).
        // OS, IO, PACKAGE, DEBUG are strictly omitted to prevent arbitrary host execution.
        let lua = Lua::new_with(
            StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8,
            LuaOptions::default(),
        )
        .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;

        // Bind the Petunia brokered API table into the Lua global environment
        // as `ptnd`, with a read-only `aubrieta` alias for legacy plugins (15.A).
        self.bind_petunia_design_api(&lua, &plugin_id)?;

        // Execute entrypoint script
        lua.load(script_source)
            .set_name(manifest.entrypoint.as_str())
            .exec()
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;

        self.plugins.insert(plugin_id.clone(), manifest);
        self.lua_states.insert(plugin_id, lua);
        Ok(())
    }

    /// Executes an exported plugin action function by name.
    pub fn execute_action(
        &self,
        plugin_id: &PluginId,
        action_name: &str,
        param: Option<&str>,
    ) -> Result<String, PluginSecurityError> {
        let lua = self
            .lua_states
            .get(plugin_id)
            .ok_or_else(|| PluginSecurityError::PluginNotFound(plugin_id.clone()))?;

        let globals = lua.globals();
        let func: mlua::Function = globals.get(action_name).map_err(|e| {
            PluginSecurityError::ScriptError(format!(
                "function `{action_name}` not found in plugin `{plugin_id}`: {e}"
            ))
        })?;

        let result: String = func
            .call(param.unwrap_or(""))
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;

        Ok(result)
    }

    fn bind_petunia_design_api(
        &self,
        lua: &Lua,
        plugin_id: &PluginId,
    ) -> Result<(), PluginSecurityError> {
        let globals = lua.globals();
        let ptnd = lua
            .create_table()
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;

        // ptnd.app
        let app = lua
            .create_table()
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;
        app.set("name", "Petunia Design Studio")
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;
        app.set("version", "0.1.0")
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;
        ptnd
            .set("app", app)
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;

        // ptnd.log
        let record = Arc::clone(&self.shared_record);
        let log_fn = lua
            .create_function(move |_, msg: String| {
                if let Ok(mut r) = record.lock() {
                    r.logs.push(msg);
                }
                Ok(())
            })
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;
        ptnd
            .set("log", log_fn)
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;

        // ptnd.document
        let broker_doc = self.broker.clone();
        let pid_doc = plugin_id.clone();
        let doc_state = Arc::clone(&self.document_state);
        let doc_table = lua
            .create_table()
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;

        let surface_count_fn = lua
            .create_function(move |_, ()| {
                broker_doc
                    .check_permission(&pid_doc, PluginPermission::DocumentRead)
                    .map_err(|e| mlua::Error::runtime(e.to_string()))?;
                let doc = doc_state.lock().unwrap();
                Ok(doc.surfaces().len())
            })
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;
        doc_table
            .set("surface_count", surface_count_fn)
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;

        ptnd
            .set("document", doc_table)
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;

        // ptnd.clipboard
        let broker_clip = self.broker.clone();
        let pid_clip = plugin_id.clone();
        let clip_state = Arc::clone(&self.in_memory_clipboard);
        let clip_table = lua
            .create_table()
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;

        let clip_state_get = Arc::clone(&clip_state);
        let broker_clip_get = broker_clip.clone();
        let pid_clip_get = pid_clip.clone();
        let get_text_fn = lua
            .create_function(move |_, ()| {
                broker_clip_get
                    .check_permission(&pid_clip_get, PluginPermission::Clipboard)
                    .map_err(|e| mlua::Error::runtime(e.to_string()))?;
                let clip = clip_state_get.lock().unwrap();
                Ok(clip.clone())
            })
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;
        clip_table
            .set("get_text", get_text_fn)
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;

        let set_text_fn = lua
            .create_function(move |_, text: String| {
                broker_clip
                    .check_permission(&pid_clip, PluginPermission::Clipboard)
                    .map_err(|e| mlua::Error::runtime(e.to_string()))?;
                let mut clip = clip_state.lock().unwrap();
                *clip = Some(text);
                Ok(())
            })
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;
        clip_table
            .set("set_text", set_text_fn)
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;

        ptnd
            .set("clipboard", clip_table)
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;

        // ptnd.actions
        let broker_act = self.broker.clone();
        let pid_act = plugin_id.clone();
        let record_act = Arc::clone(&self.shared_record);
        let act_table = lua
            .create_table()
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;

        let request_fn = lua
            .create_function(move |_, (id_str, payload): (String, Option<String>)| {
                broker_act
                    .check_permission(&pid_act, PluginPermission::DocumentWrite)
                    .map_err(|e| mlua::Error::runtime(e.to_string()))?;

                let action_id = ActionId::new(&id_str);
                let val = payload.map_or_else(
                    || serde_json::json!({}),
                    |p| serde_json::json!({ "payload": p }),
                );
                let req = ActionRequest::new(action_id, val);
                if let Ok(mut r) = record_act.lock() {
                    r.actions_requested.push(req);
                }
                Ok(true)
            })
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;
        act_table
            .set("request", request_fn)
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;

        ptnd
            .set("actions", act_table)
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;

        globals
            .set("ptnd", ptnd.clone())
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;
        // Legacy alias: old plugins keep working, new ones use `ptnd`.
        globals
            .set("aubrieta", ptnd)
            .map_err(|e| PluginSecurityError::ScriptError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_design_foundation::SurfaceId;

    #[test]
    fn plugin_loads_and_executes_action() {
        let mut host = PluginHost::new();
        let manifest = PluginManifest::new(
            PluginId::new("test.greeter"),
            "Greeter Plugin",
            "1.0.0",
            "main.lua",
        );

        let script = r#"
            function greet(name)
                ptnd.log("greeting " .. name)
                return "Hello, " .. name .. " from " .. ptnd.app.name
            end
        "#;

        host.load_plugin(manifest, script).expect("load plugin");
        let result = host
            .execute_action(&PluginId::new("test.greeter"), "greet", Some("World"))
            .expect("execute action");

        assert_eq!(result, "Hello, World from Petunia Design Studio");
        let rec = host.take_record();
        assert_eq!(rec.logs, vec!["greeting World".to_string()]);
    }

    #[test]
    fn permission_denied_triggers_error_inside_lua() {
        let mut host = PluginHost::new();
        // Do NOT grant DocumentRead permission
        let manifest = PluginManifest::new(
            PluginId::new("test.restricted"),
            "Restricted Plugin",
            "1.0.0",
            "main.lua",
        );

        let script = r#"
            function inspect()
                return ptnd.document.surface_count()
            end
        "#;

        host.load_plugin(manifest, script).expect("load plugin");
        let err = host
            .execute_action(&PluginId::new("test.restricted"), "inspect", None)
            .unwrap_err();

        match err {
            PluginSecurityError::ScriptError(msg) => {
                assert!(msg.contains("denied `document:read`"));
            }
            other => panic!("expected ScriptError containing permission denial, got {other:?}"),
        }
    }

    #[test]
    fn permission_granted_allows_query_and_mutation() {
        let mut host = PluginHost::new();
        let manifest = PluginManifest::new(
            PluginId::new("test.authorized"),
            "Authorized Plugin",
            "1.0.0",
            "main.lua",
        )
        .with_permission(PluginPermission::DocumentRead)
        .with_permission(PluginPermission::DocumentWrite);

        // Pre-seed document with a surface through the mutator lane (A6).
        let mut doc = Document::new();
        petunia_design_document::DocumentMutator::new(&mut doc)
            .add_surface(SurfaceId::new(10), "Page A")
            .expect("seed surface");
        host.set_document(doc);

        let script = r#"
            function run_tool()
                local count = ptnd.document.surface_count()
                ptnd.actions.request("ptnd.action.add_rectangle", "100x100")
                return "surfaces=" .. count
            end
        "#;

        host.load_plugin(manifest, script).expect("load plugin");
        let result = host
            .execute_action(&PluginId::new("test.authorized"), "run_tool", None)
            .expect("execute action");

        assert_eq!(result, "surfaces=1");
        let rec = host.take_record();
        assert_eq!(rec.actions_requested.len(), 1);
        assert_eq!(
            rec.actions_requested[0].action.0.as_str(),
            "ptnd.action.add_rectangle"
        );
    }

    #[test]
    fn sandbox_strips_os_and_io_libraries() {
        let mut host = PluginHost::new();
        let manifest = PluginManifest::new(
            PluginId::new("test.jailbreak"),
            "Jailbreak Attempt",
            "1.0.0",
            "main.lua",
        );

        let script = r#"
            function test_jailbreak()
                if os ~= nil then return "os_accessible" end
                if io ~= nil then return "io_accessible" end
                return "secure"
            end
        "#;

        host.load_plugin(manifest, script).expect("load plugin");
        let result = host
            .execute_action(&PluginId::new("test.jailbreak"), "test_jailbreak", None)
            .expect("execute");
        assert_eq!(result, "secure");
    }
}
