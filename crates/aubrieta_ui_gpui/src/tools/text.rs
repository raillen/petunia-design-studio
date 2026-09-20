//! Typography text creation tools (10.6).

use aubrieta_application::{Command, CommandRequest};
use aubrieta_document::ChangeSet;
use aubrieta_foundation::AubrietaError;
use aubrieta_geometry::{GPoint, GRect};

use crate::bridge::AubrietaGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use super::input::{NormalizedPointerEvent, PointerButton, PointerPhase};

/// Typography tool mode (10.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextToolMode {
    /// Click to create auto-sized headline text.
    Artistic,
    /// Drag to create bounded paragraph container frame.
    Frame,
}

/// Interactive tool for creating artistic headlines and text frames (10.6).
#[derive(Clone, Debug)]
pub struct TextTool {
    mode: TextToolMode,
    start_doc: Option<GPoint>,
    current_doc: Option<GPoint>,
}

impl TextTool {
    /// Creates a text tool in a given mode.
    #[must_use]
    pub fn new(mode: TextToolMode) -> Self {
        Self {
            mode,
            start_doc: None,
            current_doc: None,
        }
    }

    /// Cancels active text creation gesture.
    pub fn cancel(&mut self) {
        self.start_doc = None;
        self.current_doc = None;
    }

    /// Handles normalized pointer events.
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut AubrietaGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, AubrietaError> {
        match event.phase {
            PointerPhase::Down => {
                if event.button != PointerButton::Primary {
                    return Ok(ChangeSet::empty());
                }
                let mut pt = event.doc_pos;
                if !event.modifiers.disable_snap {
                    pt = snap.snap_point(pt, camera, &[]).point;
                }
                self.start_doc = Some(pt);
                self.current_doc = Some(pt);
                Ok(ChangeSet::empty())
            }
            PointerPhase::Move => {
                if self.start_doc.is_some() {
                    let mut pt = event.doc_pos;
                    if !event.modifiers.disable_snap {
                        pt = snap.snap_point(pt, camera, &[]).point;
                    }
                    self.current_doc = Some(pt);
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Up => {
                let start = self.start_doc.take();
                let current = self.current_doc.take();

                if let (Some(p0), Some(p1)) = (start, current) {
                    let w = (p1.x - p0.x).abs();
                    let h = (p1.y - p0.y).abs();

                    let bounds = if self.mode == TextToolMode::Artistic || (w < 4.0 && h < 4.0) {
                        [p0.x, p0.y, 160.0, 32.0]
                    } else {
                        let x = p0.x.min(p1.x);
                        let y = p0.y.min(p1.y);
                        [x, y, w.max(20.0), h.max(20.0)]
                    };

                    return self.commit_text(bridge, bounds);
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Cancel => {
                self.cancel();
                Ok(ChangeSet::empty())
            }
        }
    }

    fn commit_text(
        &mut self,
        bridge: &mut AubrietaGuiBridge,
        bounds: [f64; 4],
    ) -> Result<ChangeSet, AubrietaError> {
        let active_surface = bridge
            .session()
            .and_then(|s| s.active_surface)
            .ok_or_else(|| AubrietaError::invalid_input("no active surface for text creation"))?;

        let obj_id = bridge.next_object_id()?;
        let name = match self.mode {
            TextToolMode::Artistic => "Artistic Text",
            TextToolMode::Frame => "Text Frame",
        };

        let mut combined = ChangeSet::empty();

        let c1 = bridge.submit_command(CommandRequest::new(Command::CreateObject {
            surface: active_surface,
            id: obj_id,
            name: name.to_string(),
        }))?;
        combined.extend(c1);

        let c2 = bridge.submit_command(CommandRequest::new(Command::SetBounds {
            id: obj_id,
            bounds: Some(bounds),
            rotation: 0.0,
        }))?;
        combined.extend(c2);

        let text_shape = aubrieta_document::ShapeKind::Text {
            content: "Aubrieta Typography".to_string(),
            font_family: "Inter".to_string(),
            font_size: if self.mode == TextToolMode::Artistic {
                24.0
            } else {
                14.0
            },
            line_height: 1.3,
            letter_spacing: 0.0,
        };
        let c3 = bridge.submit_command(CommandRequest::new(Command::SetShape {
            id: obj_id,
            shape: Some(text_shape),
        }))?;
        combined.extend(c3);

        let c4 = bridge.submit_command(CommandRequest::new(Command::SetFill {
            id: obj_id,
            fill: Some("aubrieta.gray/900".to_string()),
        }))?;
        combined.extend(c4);

        bridge.set_selection(vec![obj_id]);
        Ok(combined)
    }

    /// Resolves overlays when dragging out a frame text bounding box.
    #[must_use]
    pub fn overlays(&self, camera: &ViewportCamera) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        if let (Some(p0), Some(p1)) = (self.start_doc, self.current_doc) {
            let s0 = camera.doc_to_screen(p0);
            let s1 = camera.doc_to_screen(p1);
            overlays.marquee_screen = Some(GRect::new(s0.x, s0.y, s1.x, s1.y));
        }
        overlays
    }
}
