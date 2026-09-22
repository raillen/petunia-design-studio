//! Corner radius and Contour offset tools (08.24, 10.2, 10.3).

use petunia_design_document::ChangeSet;
use petunia_design_foundation::PetuniaError;
use petunia_design_geometry::GPoint;

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase,
};

/// Operational mode for corner and contour manipulation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContourMode {
    /// Corner tool adjusting corner radii on parametric shapes or paths.
    Corner,
    /// Contour tool performing live path offsets.
    Contour,
}

/// Interactive Corner and Contour tool.
#[derive(Clone, Debug)]
pub struct ContourTool {
    mode: ContourMode,
    start_doc: Option<GPoint>,
    current_doc: Option<GPoint>,
}

impl ContourTool {
    /// Creates a contour or corner tool.
    #[must_use]
    pub fn new(mode: ContourMode) -> Self {
        Self {
            mode,
            start_doc: None,
            current_doc: None,
        }
    }

    /// Resets active drag.
    pub fn cancel(&mut self) {
        self.start_doc = None;
        self.current_doc = None;
    }

    /// Handles pointer events.
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

                if let (Some(p0), Some(p1)) = (start, current) {
                    let selected = bridge.selection().selected_ids;
                    let mut combined = ChangeSet::empty();

                    // Corner drags use raw distance with right/down-positive
                    // sign (no radial convention applies to radii).
                    let (ddx, ddy) = (p1.x - p0.x, p1.y - p0.y);
                    let drag_dist =
                        (ddx * ddx + ddy * ddy).sqrt() * if ddx + ddy >= 0.0 { 1.0 } else { -1.0 };
                    match self.mode {
                        ContourMode::Contour => {
                            // Outward-from-center convention per object:
                            // dragging away from the bounds center expands
                            // (positive delta), dragging toward it insets.
                            for id in selected {
                                let delta = contour_delta(bridge, id, p0, p1);
                                let c = bridge.offset_path(id, delta)?;
                                combined.extend(c);
                            }
                        }
                        ContourMode::Corner => {
                            for id in selected {
                                if let Some(session) = bridge.session() {
                                    if let Some(obj) = session.find_object(id) {
                                        if let Some(
                                            petunia_design_document::ShapeKind::Rectangle {
                                                corner_radii,
                                            },
                                        ) = obj.shape
                                        {
                                            // Physical clamp from bounds (Table B).
                                            let new_r = petunia_design_geometry::step_corner_radius(
                                                corner_radii[0],
                                                drag_dist,
                                                obj.bounds,
                                            );
                                            let c = bridge.set_shape(
                                                id,
                                                Some(
                                                    petunia_design_document::ShapeKind::Rectangle {
                                                        corner_radii: [new_r; 4],
                                                    },
                                                ),
                                            )?;
                                            combined.extend(c);
                                            continue;
                                        }
                                    }
                                }
                                let c = bridge.bake_corners(id)?;
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

    /// Resolves overlays (none or preview).
    #[must_use]
    pub fn overlays(&self) -> CanvasOverlays {
        CanvasOverlays::default()
    }
}

/// Signed offset distance for one object: radial distance from the bounds
/// center to the release point minus the distance to the grab point.
/// Positive means outward (expand), negative means inward (inset).
/// Objects without bounds fall back to drag length with the legacy
/// right/down-positive sign.
fn contour_delta(
    bridge: &PetuniaDesignGuiBridge,
    id: petunia_design_foundation::ObjectId,
    p0: petunia_design_geometry::GPoint,
    p1: petunia_design_geometry::GPoint,
) -> f64 {
    let center = bridge
        .session()
        .and_then(|s| s.find_object(id))
        .and_then(|o| o.bounds)
        .map(|b| (b[0] + b[2] / 2.0, b[1] + b[3] / 2.0));
    match center {
        Some((cx, cy)) => {
            let d0 = ((p0.x - cx).powi(2) + (p0.y - cy).powi(2)).sqrt();
            let d1 = ((p1.x - cx).powi(2) + (p1.y - cy).powi(2)).sqrt();
            d1 - d0
        }
        None => {
            let dx = p1.x - p0.x;
            let dy = p1.y - p0.y;
            (dx * dx + dy * dy).sqrt() * if dx + dy >= 0.0 { 1.0 } else { -1.0 }
        }
    }
}
