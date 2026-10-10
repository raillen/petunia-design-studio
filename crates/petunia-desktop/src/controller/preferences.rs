//! Atomic customization: stage, validate, persist, then publish to the window.
use super::*;
use petunia_ui::{ActionId, KeyCombo};

impl DesktopController {
    pub fn with_preferences(path: PathBuf) -> Self {
        let recovery = UiPreferences::load_recovering(&path);
        let mut controller = Self::new();
        controller.preferences = recovery.preferences;
        controller.preference_path = Some(path);
        if let Some(message) = recovery.diagnostic {
            controller.error = message.clone();
            controller.status = message;
        }
        if let Ok(table) = controller.preferences.shortcut_table() {
            *controller.session.shortcuts_mut() = table;
        }
        controller.apply_persona_layout();
        controller
    }

    pub(super) fn apply_persona_layout(&mut self) {
        if let Ok(persona) = self.preferences.active_persona() {
            self.session
                .workspace_mut()
                .set_panel_open("properties", false);
            for id in [
                "inspector",
                "layers",
                "history",
                "navigator",
                "color",
                "stroke",
                "brushes",
                "assets",
            ] {
                self.session
                    .workspace_mut()
                    .set_panel_open(id, persona.panels.iter().any(|panel| panel == id));
            }
            if let Some(tool) = persona
                .tools
                .first()
                .and_then(|tool| parse_tool(tool.trim_start_matches("tool.")).ok())
            {
                self.session.select_tool(tool);
            }
        }
    }

    fn publish_preferences(&mut self, preferences: UiPreferences) -> Result<(), DesktopError> {
        preferences.validate()?;
        let table = preferences.shortcut_table()?;
        if let Some(path) = &self.preference_path {
            if let Some(parent) = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
            {
                std::fs::create_dir_all(parent)?;
            }
            preferences.save(path)?;
        }
        self.preferences = preferences;
        *self.session.shortcuts_mut() = table;
        Ok(())
    }

    pub(super) fn preference_command(
        &mut self,
        command: &str,
        payload: &Value,
    ) -> Result<(), DesktopError> {
        let mut staged = self.preferences.clone();
        match command {
            "preferences" => {
                if let Some(value) = payload.get("appearance") {
                    staged.appearance = serde_json::from_value(value.clone())?;
                }
                if let Some(value) = payload.get("iconFamily") {
                    staged.icon_family = serde_json::from_value(value.clone())?;
                }
                if let Some(value) = payload.get("iconStyle") {
                    staged.icon_style = serde_json::from_value(value.clone())?;
                }
                if let Some(value) = payload.get("density") {
                    staged.density = serde_json::from_value(value.clone())?;
                }
                if payload.get("uiScale").is_some() {
                    staged.ui_scale = number(payload, "uiScale")?;
                }
                staged.reduced_motion =
                    boolean_or(payload, "reducedMotion", staged.reduced_motion)?;
                self.publish_preferences(staged)?;
            }
            "persona" => {
                self.require_idle()?;
                let id = string(payload, "id")?;
                let operation = string(payload, "operation")?;
                match operation {
                    "activate" => staged.activate_persona(id)?,
                    "show" => {
                        staged.set_persona_visible(id, boolean_or(payload, "visible", true)?)?
                    }
                    "duplicate" => {
                        staged.duplicate_persona(id, string(payload, "name")?)?;
                    }
                    "rename" => staged.rename_persona(id, string(payload, "name")?)?,
                    "remove" => staged.remove_persona(id)?,
                    "restore" => staged.restore_persona(id)?,
                    "reorder" => {
                        let current = staged
                            .personas
                            .iter()
                            .position(|persona| persona.id == id)
                            .ok_or_else(|| DesktopError::Invalid("Persona inexistente".into()))?;
                        let direction = payload
                            .get("direction")
                            .and_then(Value::as_i64)
                            .unwrap_or_else(|| {
                                if payload.get("direction").and_then(Value::as_str) == Some("up") {
                                    -1
                                } else {
                                    1
                                }
                            });
                        if !matches!(direction, -1 | 1) {
                            return invalid("Direção precisa ser -1 ou 1");
                        }
                        let index = current
                            .saturating_add_signed(direction as isize)
                            .min(staged.personas.len() - 1);
                        staged.reorder_persona(id, index)?;
                    }
                    "edit" => {
                        staged.rename_persona(id, string(payload, "name")?)?;
                        let tools =
                            serde_json::from_value(payload.get("tools").cloned().ok_or_else(
                                || DesktopError::Invalid("Ferramentas obrigatórias".into()),
                            )?)?;
                        let panels =
                            serde_json::from_value(payload.get("panels").cloned().ok_or_else(
                                || DesktopError::Invalid("Painéis obrigatórios".into()),
                            )?)?;
                        staged.configure_persona(id, string(payload, "icon")?, tools, panels)?;
                    }
                    _ => return invalid("Operação de persona desconhecida"),
                }
                self.publish_preferences(staged)?;
                if matches!(operation, "activate" | "restore" | "edit")
                    && id == self.preferences.active_persona_id
                {
                    self.apply_persona_layout();
                }
            }
            "panel" => {
                let id = string(payload, "id")?;
                if !matches!(
                    id,
                    "inspector"
                        | "layers"
                        | "history"
                        | "navigator"
                        | "color"
                        | "stroke"
                        | "brushes"
                        | "assets"
                ) {
                    return invalid("Painel indisponível");
                }
                let open = boolean_or(payload, "open", true)?;
                let active = staged.active_persona()?.clone();
                let mut panels = active.panels;
                if open && !panels.iter().any(|panel| panel == id) {
                    panels.push(id.into());
                }
                if !open {
                    panels.retain(|panel| panel != id);
                }
                staged.configure_persona(&active.id, &active.icon_key, active.tools, panels)?;
                self.publish_preferences(staged)?;
                self.session.workspace_mut().set_panel_open(id, open);
            }
            "rebind" => {
                let action = parse_action(string(payload, "action")?)?;
                if boolean_or(payload, "restore", false)? {
                    staged.reset_shortcut_binding(action)?;
                } else {
                    staged.set_shortcut_binding(
                        action,
                        parse_shortcut(string(payload, "shortcut")?)?,
                    )?;
                }
                self.publish_preferences(staged)?;
            }
            _ => return invalid("Customização indisponível"),
        }
        self.status = "Preferências atualizadas".into();
        Ok(())
    }

    pub(super) fn action_command(&mut self, id: &str) -> Result<(), DesktopError> {
        let (command, payload) = match id {
            "document.new" => ("new", serde_json::json!({})),
            "document.save" => ("save", serde_json::json!({})),
            "edit.undo" => ("undo", serde_json::json!({})),
            "edit.redo" => ("redo", serde_json::json!({})),
            "edit.select_all" => ("selectAll", serde_json::json!({})),
            "edit.delete" => ("delete", serde_json::json!({})),
            "edit.duplicate" => ("duplicate", serde_json::json!({})),
            "edit.confirm" => ("enter", serde_json::json!({})),
            "edit.cancel" => ("escape", serde_json::json!({})),
            "edit.hard_delete_nodes" => ("smartDelete", serde_json::json!({"mode":"hard"})),
            "view.fit" => ("fit", serde_json::json!({})),
            "view.zoom_in" => ("zoom", serde_json::json!({"factor":1.25})),
            "view.zoom_out" => ("zoom", serde_json::json!({"factor":0.8})),
            _ if id.starts_with("tool.") => (
                "chooseTool",
                serde_json::json!({"tool":id.trim_start_matches("tool.")}),
            ),
            "nudge.left" | "nudge.right" | "nudge.up" | "nudge.down" => {
                self.require_idle()?;
                let (dx, dy) = match id {
                    "nudge.left" => (-1.0, 0.0),
                    "nudge.right" => (1.0, 0.0),
                    "nudge.up" => (0.0, -1.0),
                    _ => (0.0, 1.0),
                };
                if !self.session.selection().sub.nodes().is_empty() {
                    return self.nudge_nodes(dx, dy);
                }
                let response = self.session.nudge(dx, dy)?;
                return self.response(response);
            }
            _ => return invalid("Ação indisponível nesta versão"),
        };
        self.execute(command, &payload)
    }

    pub(super) fn restore_shortcuts(&mut self) -> Result<(), DesktopError> {
        let mut staged = self.preferences.clone();
        staged.reset_shortcut_bindings()?;
        self.publish_preferences(staged)
    }
}

fn parse_action(id: &str) -> Result<ActionId, DesktopError> {
    petunia_ui::shortcuts::default_bindings()
        .into_iter()
        .map(|(action, _)| action)
        .find(|action| action.as_str() == id)
        .ok_or_else(|| DesktopError::Invalid("Ação desconhecida".into()))
}

fn parse_shortcut(text: &str) -> Result<KeyCombo, DesktopError> {
    if text.is_empty() || text.len() > 128 {
        return invalid("Atalho vazio ou muito longo");
    }
    let mut combo = KeyCombo::key("");
    for part in text.split('+') {
        match part.trim().to_lowercase().as_str() {
            "ctrl" | "cmd" => {
                if combo.ctrl {
                    return invalid("Modificador repetido");
                }
                combo.ctrl = true;
            }
            "shift" => {
                if combo.shift {
                    return invalid("Modificador repetido");
                }
                combo.shift = true;
            }
            "alt" => {
                if combo.alt {
                    return invalid("Modificador repetido");
                }
                combo.alt = true;
            }
            "" => return invalid("Atalho incompleto"),
            key => {
                if !combo.key.is_empty() {
                    return invalid("Use uma tecla principal por atalho");
                }
                combo.key = normalize_key(key);
            }
        }
    }
    if combo.key.is_empty() {
        return invalid("Escolha uma tecla principal");
    }
    if matches!(combo.key.as_str(), "alt" | "ctrl" | "shift" | "meta") {
        return invalid("Modificador sozinho não é um atalho");
    }
    Ok(combo)
}

fn normalize_key(key: &str) -> String {
    match key {
        "return" => "enter".into(),
        "left" | "arrowleft" => "ArrowLeft".into(),
        "right" | "arrowright" => "ArrowRight".into(),
        "up" | "arrowup" => "ArrowUp".into(),
        "down" | "arrowdown" => "ArrowDown".into(),
        _ => key.into(),
    }
}
