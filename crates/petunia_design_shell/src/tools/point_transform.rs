//! Point Transform tool rotating and scaling around custom pivot origins (08.24, 10.1).

use petunia_design_document::ChangeSet;
use petunia_design_foundation::PetuniaError;
use petunia_design_geometry::GPoint;

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{NormalizedPointerEvent, PointerButton, PointerPhase};

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

    /// Resets active transform, releasing the sticky pivot (Table B).
    pub fn cancel(&mut self) {
        self.pivot = None;
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
                    // Shared primitives reject degenerate vectors instead of
                    // applying atan2(0,0) phantom rotations or 0/0 scales.
                    let delta_angle =
                        petunia_design_geometry::pivot_angle_delta(p0, p1, pivot).unwrap_or(0.0);
                    let scale = petunia_design_geometry::scale_factor_around(p0, p1, pivot);

                    let selected = bridge.selection().selected_ids;
                    let mut cmds = Vec::new();

                    for id in selected {
                        let obj = bridge
                            .session()
                            .and_then(|s| s.find_object(id))
                            .cloned();

                        if let Some(o) = obj {
                            let next_bounds = match (o.bounds, scale) {
                                (Some(b), Some(k))
                                    if (k - 1.0).abs() > f64::EPSILON =>
                                {
                                    petunia_design_geometry::scale_bounds_about(b, pivot, k)
                                }
                                _ => o.bounds,
                            };
                            cmds.push(petunia_design_application::Command::SetBounds {
                                id,
                                bounds: next_bounds,
                                rotation: o.rotation + delta_angle,
                            });
                        }
                    }
                    // One gesture, one undo entry (F-01).
                    return bridge.submit_all("Transform around pivot", cmds);
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
