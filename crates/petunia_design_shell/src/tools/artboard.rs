//! Artboard / Surface creation tool (08.24, 10.7).

use petunia_design_document::ChangeSet;
use petunia_design_foundation::PetuniaError;
use petunia_design_geometry::{GPoint, GRect};

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{NormalizedPointerEvent, PointerButton, PointerPhase};

/// Interactive tool for creating new Surfaces / Artboards directly on the canvas (10.7).
#[derive(Clone, Debug, Default)]
pub struct ArtboardTool {
    start_doc: Option<GPoint>,
    current_doc: Option<GPoint>,
}

impl ArtboardTool {
    /// Creates a fresh artboard creation tool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            start_doc: None,
            current_doc: None,
        }
    }

    /// Cancels active artboard drag.
    pub fn cancel(&mut self) {
        self.start_doc = None;
        self.current_doc = None;
    }

    /// Handles pointer events.
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
                    let mut w = (p1.x - p0.x).abs();
                    let mut h = (p1.y - p0.y).abs();

                    if w < 10.0 && h < 10.0 {
                        w = 1280.0;
                        h = 720.0;
                    }

                    let x = p0.x.min(p1.x);
                    let y = p0.y.min(p1.y);

                    return self.commit_artboard(bridge, [x, y], [w, h]);
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Cancel => {
                self.cancel();
                Ok(ChangeSet::empty())
            }
        }
    }

    fn commit_artboard(
        &mut self,
        bridge: &mut PetuniaDesignGuiBridge,
        origin: [f64; 2],
        dimensions: [f64; 2],
    ) -> Result<ChangeSet, PetuniaError> {
        let surface_id = bridge.next_surface_id()?;

        // One gesture, one undo entry (F-01).
        let changes = bridge.submit_all(
            "Create artboard",
            petunia_design_application::create_artboard_commands(surface_id, origin, dimensions),
        )?;

        let _ = bridge.set_active_surface(surface_id);
        Ok(changes)
    }

    /// Resolves overlays displaying the dragging artboard boundaries.
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
