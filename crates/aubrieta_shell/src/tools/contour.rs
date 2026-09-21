//! Corner radius and Contour offset tools (08.24, 10.2, 10.3).

use aubrieta_document::ChangeSet;
use aubrieta_foundation::AubrietaError;
use aubrieta_geometry::GPoint;

use crate::bridge::AubrietaGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use aubrieta_application::interaction::{NormalizedPointerEvent, PointerButton, PointerPhase};

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

                if let (Some(p0), Some(p1)) = (start, current) {
                    let delta_x = p1.x - p0.x;
                    let delta_y = p1.y - p0.y;
                    let dist = (delta_x * delta_x + delta_y * delta_y).sqrt();
                    let sign = if delta_x + delta_y >= 0.0 { 1.0 } else { -1.0 };
                    let delta = dist * sign;

                    let selected = bridge.selection().selected_ids;
                    let mut combined = ChangeSet::empty();

                    match self.mode {
                        ContourMode::Contour => {
                            for id in selected {
                                let c = bridge.offset_path(id, delta)?;
                                combined.extend(c);
                            }
                        }
                        ContourMode::Corner => {
                            for id in selected {
                                if let Some(session) = bridge.session() {
                                    if let Some(obj) = session.find_object(id) {
                                        if let Some(aubrieta_document::ShapeKind::Rectangle {
                                            corner_radii,
                                        }) = obj.shape
                                        {
                                            // Physical clamp from bounds (Table B).
                                            let new_r =
                                                aubrieta_geometry::step_corner_radius(
                                                    corner_radii[0],
                                                    delta,
                                                    obj.bounds,
                                                );
                                            let c = bridge.set_shape(
                                                id,
                                                Some(aubrieta_document::ShapeKind::Rectangle {
                                                    corner_radii: [new_r; 4],
                                                }),
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
