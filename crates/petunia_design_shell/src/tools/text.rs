//! Typography text creation tools (10.6).

use petunia_design_document::ChangeSet;
use petunia_design_foundation::PetuniaError;
use petunia_design_geometry::{GPoint, GRect};

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{NormalizedPointerEvent, PointerButton, PointerPhase};

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
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
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
        bridge: &mut PetuniaDesignGuiBridge,
        bounds: [f64; 4],
    ) -> Result<ChangeSet, PetuniaError> {
        let active_surface = bridge
            .session()
            .and_then(|s| s.active_surface())
            .ok_or_else(|| PetuniaError::invalid_input("no active surface for text creation"))?;

        let obj_id = bridge.next_object_id()?;
        let (name, text_shape) = match self.mode {
            TextToolMode::Artistic => petunia_design_document::shape_factory::artistic_text(),
            TextToolMode::Frame => petunia_design_document::shape_factory::frame_text(),
        };

        // One gesture, one undo entry (F-01).
        let changes = bridge.submit_all(
            "Create text",
            petunia_design_application::create_shape_commands(
                active_surface,
                obj_id,
                name,
                text_shape,
                Some(bounds),
                Some(petunia_design_document::shape_factory::DEFAULT_TEXT_FILL.to_string()),
                None,
            ),
        )?;

        bridge.set_selection(vec![obj_id]);
        Ok(changes)
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
