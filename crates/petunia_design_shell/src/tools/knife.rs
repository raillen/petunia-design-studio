//! Knife and Scissors vector slicing tools (10.2).

use petunia_design_document::ChangeSet;
use petunia_design_foundation::PetuniaError;
use petunia_design_geometry::GPoint;

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase,
};

/// Slicing tool mode (10.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnifeMode {
    /// Knife dragging a cut line across paths.
    Knife,
    /// Scissors clicking directly at a path point to split.
    Scissors,
}

/// Knife and scissors vector splitting tool.
#[derive(Clone, Debug)]
pub struct KnifeTool {
    mode: KnifeMode,
    start_doc: Option<GPoint>,
    current_doc: Option<GPoint>,
}

impl KnifeTool {
    /// Creates a knife or scissors tool.
    #[must_use]
    pub fn new(mode: KnifeMode) -> Self {
        Self {
            mode,
            start_doc: None,
            current_doc: None,
        }
    }

    /// Cancels active slicing gesture.
    pub fn cancel(&mut self) {
        self.start_doc = None;
        self.current_doc = None;
    }

    /// Handles pointer events for vector slicing.
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
        _camera: &ViewportCamera,
        _snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
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

                if let Some(p0) = start {
                    let pt = current.unwrap_or(p0);
                    let session = match bridge.session() {
                        Some(s) => s,
                        None => return Ok(ChangeSet::empty()),
                    };
                    let surface_id = match session.active_surface() {
                        Some(s) => s,
                        None => return Ok(ChangeSet::empty()),
                    };
                    let surface = match session.surface(surface_id) {
                        Ok(s) => s,
                        Err(_) => return Ok(ChangeSet::empty()),
                    };

                    let target_ids: Vec<petunia_design_foundation::ObjectId> = surface
                        .objects()
                        .iter()
                        .filter(|obj| obj.hit_test(p0) || obj.hit_test(pt))
                        .map(|obj| obj.id)
                        .collect();

                    let mut combined = ChangeSet::empty();
                    for id in target_ids {
                        let c = bridge.slice_path(id, [pt.x, pt.y])?;
                        combined.extend(c);
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

    /// Resolves overlays displaying the active cutting line.
    #[must_use]
    pub fn overlays(&self) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        if self.mode == KnifeMode::Knife {
            if let (Some(p0), Some(p1)) = (self.start_doc, self.current_doc) {
                overlays.pen_preview = Some(vec![p0, p1]);
            }
        }
        overlays
    }
}
