//! Freehand path sketching and curve fitting tool (10.2).

use petunia_design_document::ChangeSet;
use petunia_design_foundation::PetuniaError;
use petunia_design_geometry::GPoint;

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{NormalizedPointerEvent, PointerButton, PointerPhase};

/// Freehand pencil tool capturing raw pointer gestures and committing smoothed paths (10.2).
#[derive(Clone, Debug, Default)]
pub struct PencilTool {
    sampled_points: Vec<GPoint>,
}

impl PencilTool {
    /// Creates a fresh pencil tool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sampled_points: Vec::new(),
        }
    }

    /// Cancels active stroke gesture.
    pub fn cancel(&mut self) {
        self.sampled_points.clear();
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
                self.sampled_points.clear();
                self.sampled_points.push(pt);
                Ok(ChangeSet::empty())
            }
            PointerPhase::Move => {
                if !self.sampled_points.is_empty() {
                    let mut pt = event.doc_pos;
                    if !event.modifiers.disable_snap {
                        pt = snap.snap_point(pt, camera, &[]).point;
                    }
                    if let Some(last) = self.sampled_points.last() {
                        let dx = pt.x - last.x;
                        let dy = pt.y - last.y;
                        if (dx * dx + dy * dy).sqrt() >= 2.0 {
                            self.sampled_points.push(pt);
                        }
                    }
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Up => {
                if self.sampled_points.len() < 2 {
                    self.sampled_points.clear();
                    return Ok(ChangeSet::empty());
                }
                let pts = std::mem::take(&mut self.sampled_points);
                self.commit_path(bridge, &pts)
            }
            PointerPhase::Cancel => {
                self.cancel();
                Ok(ChangeSet::empty())
            }
        }
    }

    fn commit_path(
        &mut self,
        bridge: &mut PetuniaDesignGuiBridge,
        pts: &[GPoint],
    ) -> Result<ChangeSet, PetuniaError> {
        let active_surface = bridge
            .session()
            .and_then(|s| s.active_surface())
            .ok_or_else(|| PetuniaError::invalid_input("no active surface for path creation"))?;

        // Shared freehand pipeline (10.2, F-14): simplify, smooth, fit.
        // Deterministic; pressure/width stay POST_V1.
        let smoothed = petunia_design_geometry::smooth_samples(pts, 1.5, 1);
        let path = petunia_design_geometry::fit_midpoint_quads(&smoothed)
            .map_err(PetuniaError::invalid_input)?;

        let bounds = path
            .bounding_box()
            .map(|r| [r.x0, r.y0, r.width().max(1.0), r.height().max(1.0)])
            .unwrap_or([pts[0].x, pts[0].y, 10.0, 10.0]);

        let obj_id = bridge.next_object_id()?;

        // One gesture, one undo entry (F-01).
        let changes = bridge.submit_all(
            "Freehand path",
            petunia_design_application::create_shape_commands(
                active_surface,
                obj_id,
                "Freehand Path",
                petunia_design_document::ShapeKind::Path(path),
                Some(bounds),
                None,
                Some((
                    petunia_design_document::shape_factory::DEFAULT_PATH_STROKE.to_string(),
                    petunia_design_document::shape_factory::DEFAULT_PENCIL_STROKE_WIDTH,
                )),
            ),
        )?;

        bridge.set_selection(vec![obj_id]);
        Ok(changes)
    }

    /// Resolves live preview overlays for active freehand drawing.
    #[must_use]
    pub fn overlays(&self) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        if self.sampled_points.len() >= 2 {
            overlays.pen_preview = Some(self.sampled_points.clone());
        }
        overlays
    }
}
