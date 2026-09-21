//! Vector gradient and transparency tools (08.24, 10.4).

use aubrieta_application::{Command, CommandRequest};
use aubrieta_document::ChangeSet;
use aubrieta_foundation::AubrietaError;
use aubrieta_geometry::GPoint;

use crate::bridge::AubrietaGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use aubrieta_application::interaction::{NormalizedPointerEvent, PointerButton, PointerPhase};

/// Mode for the gradient tool (10.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GradientToolMode {
    /// Color gradient on fill/stroke.
    Fill,
    /// Mask transparency gradient.
    Transparency,
}

/// Interactive tool for plotting gradient vector lines on canvas (10.4).
#[derive(Clone, Debug)]
pub struct GradientTool {
    mode: GradientToolMode,
    start_doc: Option<GPoint>,
    current_doc: Option<GPoint>,
}

impl GradientTool {
    /// Creates a gradient tool in fill or transparency mode.
    #[must_use]
    pub fn new(mode: GradientToolMode) -> Self {
        Self {
            mode,
            start_doc: None,
            current_doc: None,
        }
    }

    /// Resets active gradient drag.
    pub fn cancel(&mut self) {
        self.start_doc = None;
        self.current_doc = None;
    }

    /// Handles pointer events.
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut AubrietaGuiBridge,
        _camera: &ViewportCamera,
        _snap: &mut SnapEngine,
    ) -> Result<ChangeSet, AubrietaError> {
        match event.phase {
            PointerPhase::Down => {
                if event.button != PointerButton::Primary {
                    return Ok(ChangeSet::empty());
                }
                self.start_doc = Some(event.doc_pos);
                self.current_doc = Some(event.doc_pos);
                Ok(ChangeSet::empty())
            }
            PointerPhase::Move => {
                if self.start_doc.is_some() {
                    self.current_doc = Some(event.doc_pos);
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Up => {
                let start = self.start_doc.take();
                let current = self.current_doc.take();

                if let (Some(_p0), Some(_p1)) = (start, current) {
                    let selected = bridge.selection().selected_ids;
                    let mut combined = ChangeSet::empty();

                    for id in selected {
                        match self.mode {
                            GradientToolMode::Fill => {
                                let c = bridge.submit_command(CommandRequest::new(
                                    Command::SetFill {
                                        id,
                                        fill: Some("aubrieta.gradient/linear".to_string()),
                                    },
                                ))?;
                                combined.extend(c);
                            }
                            GradientToolMode::Transparency => {
                                let c = bridge.submit_command(CommandRequest::new(
                                    Command::SetOpacity { id, opacity: 0.8 },
                                ))?;
                                combined.extend(c);
                            }
                        }
                    }
                    return Ok(combined);
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Cancel => {
                self.cancel();
                Ok(ChangeSet::empty())
            }
        }
    }

    /// Resolves overlays showing the gradient vector line.
    #[must_use]
    pub fn overlays(&self) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        if let (Some(p0), Some(p1)) = (self.start_doc, self.current_doc) {
            overlays.pen_preview = Some(vec![p0, p1]);
        }
        overlays
    }
}
