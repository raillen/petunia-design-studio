//! Application session lifecycle, coordinating document, engine, and rendering.

use crate::error::Result;
use crate::input::{PointerEvent, ToolKind, UserAction};
use petunia_core::{Document, Rect, SceneNode, VectorPath};
use petunia_engine::{AddNodeCommand, CommandHistory};
use petunia_render::{RenderBackend, RenderOptions};

/// Application session coordinating core document, command history, and active tool.
pub struct StudioSession {
    pub document: Document,
    pub history: CommandHistory,
    pub active_tool: ToolKind,
    pub viewport: Rect,
}

impl StudioSession {
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            document: Document::new(title),
            history: CommandHistory::new(),
            active_tool: ToolKind::Select,
            viewport: Rect::new(0.0, 0.0, 1920.0, 1080.0),
        }
    }

    /// Dispatches a high-level user action to the session.
    pub fn dispatch_action(&mut self, action: UserAction) -> Result<()> {
        match action {
            UserAction::SelectTool(tool) => {
                self.active_tool = tool;
            }
            UserAction::Undo => {
                let _ = self.history.undo(&mut self.document);
            }
            UserAction::Redo => {
                let _ = self.history.redo(&mut self.document);
            }
            UserAction::Pointer(PointerEvent::Down { position, .. }) => {
                if self.active_tool == ToolKind::Rectangle {
                    let path = VectorPath::rect(position.x, position.y, 100.0, 100.0);
                    let node = SceneNode::new_path("Rectangle", path);
                    self.history
                        .execute(Box::new(AddNodeCommand::new(node)), &mut self.document)?;
                }
            }
            UserAction::Pointer(_) => {}
        }
        Ok(())
    }

    /// Draws the current session through any specified backend.
    pub fn render_with<B: RenderBackend>(&self, backend: &mut B, options: &RenderOptions) -> Result<()> {
        backend.render_scene(&self.document.scene, self.viewport, options)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_core::Point;
    use petunia_render::SoftwareReferenceRenderer;

    #[test]
    fn test_studio_session_draw_and_render_pipeline() {
        let mut session = StudioSession::new("Canvas Session");
        session.dispatch_action(UserAction::SelectTool(ToolKind::Rectangle)).expect("action succeeds");

        session
            .dispatch_action(UserAction::Pointer(PointerEvent::Down {
                position: Point::new(20.0, 30.0),
                pressure: 1.0,
            }))
            .expect("action succeeds");

        assert_eq!(session.document.scene.len(), 1);

        let mut renderer = SoftwareReferenceRenderer::new();
        let options = RenderOptions::default();

        session.render_with(&mut renderer, &options).expect("render succeeds");
        assert_eq!(renderer.draw_calls, 1);

        // Undo action
        session.dispatch_action(UserAction::Undo).expect("undo succeeds");
        assert_eq!(session.document.scene.len(), 0);

        session.render_with(&mut renderer, &options).expect("render succeeds");
        assert_eq!(renderer.draw_calls, 0);
    }
}
