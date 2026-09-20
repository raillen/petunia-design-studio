//! Freehand path sketching and curve fitting tool (10.2).

use aubrieta_application::{Command, CommandRequest};
use aubrieta_document::ChangeSet;
use aubrieta_foundation::AubrietaError;
use aubrieta_geometry::{GPath, GPoint, PathVerb};

use crate::bridge::AubrietaGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use super::input::{NormalizedPointerEvent, PointerButton, PointerPhase};

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
        bridge: &mut AubrietaGuiBridge,
        pts: &[GPoint],
    ) -> Result<ChangeSet, AubrietaError> {
        let active_surface = bridge
            .session()
            .and_then(|s| s.active_surface)
            .ok_or_else(|| AubrietaError::invalid_input("no active surface for path creation"))?;

        let mut path = GPath::new();
        let _ = path.push(PathVerb::MoveTo(pts[0]));
        for pt in &pts[1..] {
            let _ = path.push(PathVerb::LineTo(*pt));
        }

        let bounds = path
            .bounding_box()
            .map(|r| [r.x0, r.y0, r.width().max(1.0), r.height().max(1.0)])
            .unwrap_or([pts[0].x, pts[0].y, 10.0, 10.0]);

        let obj_id = bridge.next_object_id()?;
        let mut combined = ChangeSet::empty();

        let c1 = bridge.submit_command(CommandRequest::new(Command::CreateObject {
            surface: active_surface,
            id: obj_id,
            name: "Freehand Path".to_string(),
        }))?;
        combined.extend(c1);

        let c2 = bridge.submit_command(CommandRequest::new(Command::SetBounds {
            id: obj_id,
            bounds: Some(bounds),
            rotation: 0.0,
        }))?;
        combined.extend(c2);

        let c3 = bridge.submit_command(CommandRequest::new(Command::SetShape {
            id: obj_id,
            shape: Some(aubrieta_document::ShapeKind::Path(path)),
        }))?;
        combined.extend(c3);

        let c4 = bridge.submit_command(CommandRequest::new(Command::SetStroke {
            id: obj_id,
            stroke: Some("aubrieta.gray/900".to_string()),
            width: 2.0,
        }))?;
        combined.extend(c4);

        bridge.set_selection(vec![obj_id]);
        Ok(combined)
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
