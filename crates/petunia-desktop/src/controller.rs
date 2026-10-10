//! Qt-independent adapter for the native window. All authorial changes cross
//! StudioSession's single transaction writer; JSON is confined to this edge.
mod document;
mod files;
mod preferences;
mod state;
#[cfg(test)]
mod tests;

use petunia_core::{ObjectId, Point, Rect};
use petunia_engine::{CommandId, DocumentOp, HistoryDescription, TransactionRequest};
use petunia_ui::{PointerSample, StudioSession, ToolKind, ToolResponse, UiPreferences};
use serde_json::Value;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DesktopError {
    #[error("{0}")]
    Invalid(String),
    #[error("Confirme ou cancele a operação atual antes de continuar")]
    PendingInteraction,
    #[error("Salve as alterações ou confirme o descarte antes de continuar")]
    UnsavedDocument,
    #[error(transparent)]
    Ui(#[from] petunia_ui::UiError),
    #[error(transparent)]
    Engine(#[from] petunia_engine::EngineError),
    #[error(transparent)]
    Preferences(#[from] petunia_ui::PreferenceError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Image(#[from] image::ImageError),
}

pub struct DesktopController {
    pub session: StudioSession,
    pub preferences: UiPreferences,
    preference_path: Option<PathBuf>,
    status: String,
    error: String,
    overlays: Vec<Value>,
    shape_origin: Option<Point>,
}

impl Default for DesktopController {
    fn default() -> Self {
        Self::new()
    }
}

impl DesktopController {
    pub fn new() -> Self {
        let mut controller = Self {
            session: StudioSession::new("Sem título"),
            preferences: UiPreferences::default(),
            preference_path: None,
            status: "Pronto. Escolha uma ferramenta ou crie uma forma.".into(),
            error: String::new(),
            overlays: Vec::new(),
            shape_origin: None,
        };
        controller.apply_persona_layout();
        controller
    }

    pub fn invoke(&mut self, command: &str, payload: &Value) -> Result<(), DesktopError> {
        let result = self.execute(command, payload);
        match &result {
            Ok(()) => self.error.clear(),
            Err(error) => {
                self.error = error.to_string();
                self.status.clone_from(&self.error);
            }
        }
        result
    }

    fn execute(&mut self, command: &str, payload: &Value) -> Result<(), DesktopError> {
        match command {
            "new" | "open" | "save" | "saveAs" | "exportPng" => self.file_command(command, payload),
            "chooseTool" => {
                self.require_idle()?;
                let tool = parse_tool(string(payload, "tool")?)?;
                self.session.select_tool(tool);
                self.overlays.clear();
                self.status = format!("Ferramenta: {}", tool_name(tool));
                Ok(())
            }
            "pointer" => self.pointer(payload),
            "escape" => {
                self.shape_origin = None;
                self.session.on_escape();
                self.overlays.clear();
                self.status = "Operação cancelada".into();
                Ok(())
            }
            "enter" => {
                if self.shape_origin.is_some() {
                    return Err(DesktopError::PendingInteraction);
                }
                if self.session.active_tool() == ToolKind::Pen && self.session.interaction_active()
                {
                    let response = self.session.finish_pen()?;
                    self.response(response)?;
                } else {
                    self.require_idle()?;
                    self.session.on_enter();
                }
                self.overlays.clear();
                Ok(())
            }
            "undo" | "redo" => {
                self.require_idle()?;
                let action = if command == "undo" {
                    petunia_ui::UserAction::Undo
                } else {
                    petunia_ui::UserAction::Redo
                };
                let response = self.session.dispatch_action(action)?;
                self.response(response)?;
                let selected = self.session.selection().objects().to_vec();
                let retained: Vec<_> = selected
                    .iter()
                    .copied()
                    .filter(|id| self.session.is_editable(*id))
                    .collect();
                if retained != selected {
                    self.session.set_selection(retained)?;
                }
                self.status = if command == "undo" {
                    "Alteração desfeita"
                } else {
                    "Alteração refeita"
                }
                .into();
                Ok(())
            }
            "shape" | "transform" | "setFill" | "delete" | "selectAll" | "duplicate"
            | "selectNode" | "moveNode" | "selectLayer" | "visibility" | "locked"
            | "smartDelete" => {
                self.require_idle()?;
                self.document_command(command, payload)
            }
            "preferences" | "persona" | "panel" | "rebind" => {
                self.preference_command(command, payload)
            }
            "restoreShortcuts" => self.restore_shortcuts(),
            "nudge" => {
                self.require_idle()?;
                let dx = number(payload, "dx")?;
                let dy = number(payload, "dy")?;
                if !self.session.selection().sub.nodes().is_empty() {
                    return self.nudge_nodes(dx, dy);
                }
                let response = self.session.nudge(dx, dy)?;
                self.response(response)
            }
            "zoom" => self.zoom(payload),
            "pan" => {
                self.require_idle()?;
                let dx = number(payload, "dx")?;
                let dy = number(payload, "dy")?;
                self.session.view_mut().pan_x += dx;
                self.session.view_mut().pan_y += dy;
                Ok(())
            }
            "fit" => {
                self.require_idle()?;
                self.fit();
                Ok(())
            }
            "command" => self.action_command(string(payload, "id")?),
            _ => Err(DesktopError::Invalid(format!(
                "Comando desconhecido: {command}"
            ))),
        }
    }

    fn require_idle(&self) -> Result<(), DesktopError> {
        if self.shape_origin.is_some() || self.session.interaction_active() {
            Err(DesktopError::PendingInteraction)
        } else {
            Ok(())
        }
    }

    fn commit(
        &mut self,
        operations: Vec<DocumentOp>,
        description: HistoryDescription,
    ) -> Result<(), DesktopError> {
        self.session.commit_request(
            TransactionRequest {
                command_id: CommandId::new_v4(),
                operations,
                merge_key: None,
            },
            description,
        )?;
        Ok(())
    }

    fn response(&mut self, response: ToolResponse) -> Result<(), DesktopError> {
        match response {
            ToolResponse::Failed(message) => return Err(DesktopError::Invalid(message)),
            ToolResponse::Status(message) | ToolResponse::Preview(message) => self.status = message,
            ToolResponse::Overlay(primitives) => self.overlays = self.overlay_values(primitives),
            ToolResponse::Commit(_) => {
                self.overlays.clear();
                self.status = "Alteração confirmada".into();
            }
            _ => {}
        }
        Ok(())
    }

    fn pointer(&mut self, payload: &Value) -> Result<(), DesktopError> {
        let phase = string(payload, "phase")?;
        let position = Point::new(number(payload, "x")?, number(payload, "y")?);
        let pressure = optional_number(payload, "pressure", 1.0)?;
        if !(0.0..=1.0).contains(&pressure) {
            return invalid("Pressão precisa estar entre 0 e 1");
        }
        if !matches!(phase, "down" | "move" | "up" | "double") {
            return invalid("Fase de ponteiro inválida");
        }
        if phase == "double" {
            self.session.on_double_click(position);
            return Ok(());
        }
        if matches!(
            self.session.active_tool(),
            ToolKind::Rectangle | ToolKind::Ellipse
        ) {
            return self.shape_pointer(phase, position, boolean_or(payload, "shift", false)?);
        }
        if self.session.active_tool() == ToolKind::Zoom {
            if phase == "down" {
                return self.zoom(&serde_json::json!({"factor": if boolean_or(payload,"shift",false)? {0.8} else {1.25},"anchorX":position.x,"anchorY":position.y}));
            }
            return Ok(());
        }
        let sample = PointerSample {
            position,
            pressure: pressure as f32,
            shift: boolean_or(payload, "shift", false)?,
            ctrl: boolean_or(payload, "ctrl", false)?,
            alt: false,
            timestamp_ms: 0,
        };
        let pointer = match phase {
            "down" => petunia_ui::PointerEvent::Down {
                position,
                pressure: pressure as f32,
            },
            "move" => petunia_ui::PointerEvent::Move {
                position,
                pressure: pressure as f32,
            },
            _ => petunia_ui::PointerEvent::Up { position },
        };
        let response = self.session.dispatch_pointer_sample(pointer, sample)?;
        self.response(response)?;
        if phase == "up" && !self.session.interaction_active() {
            self.overlays.clear();
        }
        Ok(())
    }

    fn zoom(&mut self, payload: &Value) -> Result<(), DesktopError> {
        self.require_idle()?;
        let factor = number(payload, "factor")?;
        if factor <= 0.0 {
            return invalid("O zoom deve ser positivo");
        }
        let view = *self.session.view();
        let x = optional_number(payload, "anchorX", view.viewport.width / 2.0)?;
        let y = optional_number(payload, "anchorY", view.viewport.height / 2.0)?;
        let scale = (view.scale * factor).clamp(0.05, 32.0);
        let ratio = scale / view.scale;
        let target = self.session.view_mut();
        target.scale = scale;
        target.pan_x = x - (x - view.pan_x) * ratio;
        target.pan_y = y - (y - view.pan_y) * ratio;
        self.overlays.clear();
        Ok(())
    }

    fn fit(&mut self) {
        let size = self.page_size();
        let view = self.session.view_mut();
        view.scale = ((view.viewport.width - 48.0).max(1.0) / size.0)
            .min((view.viewport.height - 48.0).max(1.0) / size.1)
            .clamp(0.05, 32.0);
        view.pan_x = (view.viewport.width - size.0 * view.scale) / 2.0;
        view.pan_y = (view.viewport.height - size.1 * view.scale) / 2.0;
        self.overlays.clear();
    }

    pub fn render_rgba(
        &mut self,
        width: u32,
        height: u32,
        dpr: f64,
    ) -> Result<Vec<u8>, DesktopError> {
        if width == 0
            || height == 0
            || !dpr.is_finite()
            || !(0.5..=8.0).contains(&dpr)
            || f64::from(width) * f64::from(height) * dpr * dpr > (16 << 20) as f64
        {
            return invalid("Dimensões do canvas excedem o limite");
        }
        self.session.view_mut().viewport = Rect::new(0.0, 0.0, f64::from(width), f64::from(height));
        self.session.view_mut().dpr = dpr;
        let (pixels, _) = self
            .session
            .render_headless(&petunia_render::RenderOptions::default())?;
        Ok(pixels)
    }
}

fn string<'a>(payload: &'a Value, key: &str) -> Result<&'a str, DesktopError> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| DesktopError::Invalid(format!("Campo textual obrigatório: {key}")))
}
fn number(payload: &Value, key: &str) -> Result<f64, DesktopError> {
    let value = payload
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| DesktopError::Invalid(format!("Campo numérico obrigatório: {key}")))?;
    if !value.is_finite() || value.abs() > 1e9 {
        return invalid("Número inválido ou fora do limite");
    }
    Ok(value)
}
fn optional_number(payload: &Value, key: &str, default: f64) -> Result<f64, DesktopError> {
    if payload.get(key).is_some() {
        number(payload, key)
    } else {
        Ok(default)
    }
}
fn boolean_or(payload: &Value, key: &str, default: bool) -> Result<bool, DesktopError> {
    if let Some(value) = payload.get(key) {
        value
            .as_bool()
            .ok_or_else(|| DesktopError::Invalid(format!("Campo booleano inválido: {key}")))
    } else {
        Ok(default)
    }
}
fn invalid<T>(message: &str) -> Result<T, DesktopError> {
    Err(DesktopError::Invalid(message.into()))
}
fn object_id(payload: &Value) -> Result<ObjectId, DesktopError> {
    Ok(serde_json::from_value(
        payload
            .get("id")
            .cloned()
            .ok_or_else(|| DesktopError::Invalid("Objeto obrigatório".into()))?,
    )?)
}
fn parse_tool(tool: &str) -> Result<ToolKind, DesktopError> {
    match tool {
        "select" => Ok(ToolKind::Select),
        "node" => Ok(ToolKind::NodeEdit),
        "pen" => Ok(ToolKind::Pen),
        "rectangle" => Ok(ToolKind::Rectangle),
        "ellipse" => Ok(ToolKind::Ellipse),
        "zoom" => Ok(ToolKind::Zoom),
        _ => invalid("Ferramenta indisponível"),
    }
}
fn tool_name(tool: ToolKind) -> &'static str {
    match tool {
        ToolKind::Select => "Seleção",
        ToolKind::NodeEdit => "Nós",
        ToolKind::Pen => "Caneta",
        ToolKind::Rectangle => "Retângulo",
        ToolKind::Ellipse => "Elipse",
        ToolKind::Zoom => "Zoom",
    }
}
fn tool_id(tool: ToolKind) -> &'static str {
    match tool {
        ToolKind::Select => "select",
        ToolKind::NodeEdit => "node",
        ToolKind::Pen => "pen",
        ToolKind::Rectangle => "rectangle",
        ToolKind::Ellipse => "ellipse",
        ToolKind::Zoom => "zoom",
    }
}
