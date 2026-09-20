//! Point Transform tool rotating and scaling around custom pivot origins (08.24, 10.1).

use aubrieta_application::{Command, CommandRequest};
use aubrieta_document::ChangeSet;
use aubrieta_foundation::AubrietaError;
use aubrieta_geometry::GPoint;

use crate::bridge::AubrietaGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use super::input::{NormalizedPointerEvent, PointerButton, PointerPhase};

/// Interactive tool for arbitrary transformations around a user-defined anchor point (10.1).
#[derive(Clone, Debug, Default)]
pub struct PointTransformTool {
    pivot: Option<GPoint>,
    start_doc: Option<GPoint>,
    current_doc: Option<GPoint>,
}

impl PointTransformTool {
    /// Creates a point transform tool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pivot: None,
            start_doc: None,
            current_doc: None,
        }
    }

    /// Resets active transform.
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
                if self.pivot.is_none() {
                    self.pivot = Some(event.doc_pos);
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
                    let pivot = self.pivot.unwrap_or(p0);
                    let a0 = (p0.y - pivot.y).atan2(p0.x - pivot.x);
                    let a1 = (p1.y - pivot.y).atan2(p1.x - pivot.x);
                    let delta_angle = a1 - a0;

                    let selected = bridge.selection().selected_ids;
                    let mut combined = ChangeSet::empty();

                    for id in selected {
                        let obj = bridge
                            .session()
                            .and_then(|s| s.document.find_object(id))
                            .cloned();

                        if let Some(o) = obj {
                            let new_rot = o.rotation + delta_angle;
                            let cmd = CommandRequest::new(Command::SetBounds {
                                id,
                                bounds: o.bounds,
                                rotation: new_rot,
                            });
                            let c = bridge.submit_command(cmd)?;
                            combined.extend(c);
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

    /// Resolves overlays displaying the custom transform pivot.
    #[must_use]
    pub fn overlays(&self) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        if let (Some(p0), Some(p1)) = (self.start_doc, self.current_doc) {
            overlays.pen_preview = Some(vec![p0, p1]);
        }
        overlays
    }
}
