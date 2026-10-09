//! Application session lifecycle, coordinating document, engine, and rendering.

use crate::error::Result;
use crate::input::{PointerEvent, ToolKind, UserAction};
use petunia_core::{Document, Rect, SceneNode, VectorPath};
use petunia_engine::{
    compile, prepare_transaction, CommandId, DocumentOp, DocumentRevision, History,
    HistoryDescription, TransactionRequest,
};
use petunia_render::{RenderBackend, RenderOptions};
use petunia_render_model::RenderStats;

/// Application session: single writer over the document plus the
/// render entry point. Mutations travel as transactions, so the
/// session's revision always matches the compiled snapshot's.
pub struct StudioSession {
    pub document: Document,
    pub history: History,
    pub active_tool: ToolKind,
    pub viewport: Rect,
}

impl StudioSession {
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            document: Document::new(title),
            history: History::new(64 << 20, 256 << 20),
            active_tool: ToolKind::Select,
            viewport: Rect::new(0.0, 0.0, 1920.0, 1080.0),
        }
    }

    /// Current authorial revision, used to tag render snapshots.
    #[must_use]
    pub fn revision(&self) -> DocumentRevision {
        self.history.current_revision()
    }

    /// Commit one transaction, keeping document and history in step.
    pub fn commit(
        &mut self,
        operations: Vec<DocumentOp>,
        description: HistoryDescription,
    ) -> Result<()> {
        let request = TransactionRequest {
            command_id: CommandId::new_v4(),
            operations,
            merge_key: None,
        };
        let prepared = prepare_transaction(&self.document, request, self.revision())
            .map_err(|error| petunia_engine::EngineError::Execution(error.to_string()))?;
        self.history
            .commit(&mut self.document, prepared, description)?;
        Ok(())
    }

    /// Dispatches a high-level user action to the session.
    pub fn dispatch_action(&mut self, action: UserAction) -> Result<()> {
        match action {
            UserAction::SelectTool(tool) => {
                self.active_tool = tool;
            }
            UserAction::Undo => {
                self.history
                    .undo(&mut self.document)
                    .map_err(|error| petunia_engine::EngineError::Execution(error.to_string()))?;
            }
            UserAction::Redo => {
                self.history
                    .redo(&mut self.document)
                    .map_err(|error| petunia_engine::EngineError::Execution(error.to_string()))?;
            }
            UserAction::Pointer(PointerEvent::Down { position, .. }) => {
                if self.active_tool == ToolKind::Rectangle {
                    let path = VectorPath::rect(position.x, position.y, 100.0, 100.0);
                    self.commit(
                        vec![DocumentOp::InsertRoot {
                            index: self.document.scene.len(),
                            node: Box::new(SceneNode::new_path("Rectangle", path)),
                        }],
                        HistoryDescription::InsertObjects,
                    )?;
                }
            }
            UserAction::Pointer(_) => {}
        }
        Ok(())
    }

    /// Compiles the document and renders one frame through any
    /// backend. Warnings from degraded primitives ride along so the
    /// caller can surface them.
    pub fn render_with<B: RenderBackend>(
        &mut self,
        backend: &mut B,
        options: &RenderOptions,
    ) -> Result<(Vec<u8>, RenderStats)> {
        let (snapshot, warnings) =
            compile::compile_document(&self.document, self.revision(), options.quality);
        let frame = compile::headless_frame(
            snapshot,
            self.viewport.width.max(1.0) as u32,
            self.viewport.height.max(1.0) as u32,
            options.dpr,
        );
        let rendered = backend.render(&frame, options);
        if !warnings.is_empty() {
            // Degraded primitives are reported, never silently drawn.
            for warning in &warnings {
                eprintln!(
                    "[render] degraded primitive {}: {}",
                    warning.source, warning.message
                );
            }
        }
        rendered.map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_core::Point;
    use petunia_render::SoftwareRenderer;

    #[test]
    fn test_studio_session_draw_and_render_pipeline() {
        let mut session = StudioSession::new("Canvas Session");
        session
            .dispatch_action(UserAction::SelectTool(ToolKind::Rectangle))
            .expect("action succeeds");

        session
            .dispatch_action(UserAction::Pointer(PointerEvent::Down {
                position: Point::new(20.0, 30.0),
                pressure: 1.0,
            }))
            .expect("action succeeds");

        assert_eq!(session.document.scene.len(), 1);
        assert_eq!(session.revision(), DocumentRevision(1));

        let mut renderer = SoftwareRenderer::new(1 << 20, 1 << 20);
        let options = RenderOptions::default();

        let (bytes, stats) = session
            .render_with(&mut renderer, &options)
            .expect("render succeeds");
        assert_eq!(stats.primitives_drawn, 1);
        // The rectangle spans (20,30)-(120,130); its center carries the
        // authorial default fill (opaque black).
        let center = ((80 * 1920) + 70) * 4;
        assert_eq!(&bytes[center..center + 4], &[0, 0, 0, 255]);
        // Far outside the rectangle stays background.
        let corner = 0;
        assert_eq!(&bytes[corner..corner + 4], &[255, 255, 255, 255]);

        // Undo action
        session
            .dispatch_action(UserAction::Undo)
            .expect("undo succeeds");
        assert_eq!(session.document.scene.len(), 0);
        assert_eq!(session.revision(), DocumentRevision(0));

        let (_, stats) = session
            .render_with(&mut renderer, &options)
            .expect("render succeeds");
        assert_eq!(stats.primitives_drawn, 0);

        // Redo restores the rectangle at the same revision.
        session
            .dispatch_action(UserAction::Redo)
            .expect("redo succeeds");
        assert_eq!(session.document.scene.len(), 1);
        assert_eq!(session.revision(), DocumentRevision(1));
    }
}
