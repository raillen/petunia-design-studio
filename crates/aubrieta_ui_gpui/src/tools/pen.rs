//! Pen tool state machine for Bézier path construction (10.2).

use aubrieta_application::{Command, CommandRequest};
use aubrieta_document::ChangeSet;
use aubrieta_foundation::AubrietaError;
use aubrieta_geometry::GPoint;

use crate::bridge::*;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use super::input::{NormalizedPointerEvent, PointerButton, PointerPhase};

/// Node constraint type (cusp, smooth, symmetric).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeType {
    Cusp,
    Smooth,
    Symmetric,
}

/// Anchor point on an in-progress Bézier path.
#[derive(Clone, Debug, PartialEq)]
pub struct PenAnchor {
    /// Document anchor coordinates.
    pub point: GPoint,
    /// Ingoing control handle offset.
    pub handle_in: Option<GPoint>,
    /// Outgoing control handle offset.
    pub handle_out: Option<GPoint>,
    /// Constraint type.
    pub node_type: NodeType,
}

/// Canonical phases for the Pen tool (10.2).
#[derive(Clone, Debug, Default, PartialEq)]
pub enum PenPhase {
    /// Waiting for first anchor.
    #[default]
    Idle,
    /// First anchor established.
    Started,
    /// Hovering next prospective anchor position.
    SegmentPreview { cursor_doc: GPoint },
    /// Dragging tangent handles for the current anchor.
    HandleAdjust {
        anchor_idx: usize,
        handle_pos: GPoint,
    },
    /// Hovering near start node, ready to close loop.
    ClosePreview { cursor_doc: GPoint },
}

/// Interactive vector Pen tool (10.2).
#[derive(Clone, Debug, Default)]
pub struct PenTool {
    anchors: Vec<PenAnchor>,
    phase: PenPhase,
    close_threshold_px: f64,
}

impl PenTool {
    /// Creates a fresh Pen tool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            anchors: Vec::new(),
            phase: PenPhase::Idle,
            close_threshold_px: 10.0,
        }
    }

    /// True if the tool has in-flight uncommitted anchors.
    #[must_use]
    pub fn is_active(&self) -> bool {
        !self.anchors.is_empty()
    }

    /// Cancels current provisional segment or resets tool.
    pub fn cancel(&mut self) {
        if matches!(self.phase, PenPhase::HandleAdjust { .. }) {
            self.phase = PenPhase::SegmentPreview {
                cursor_doc: self.anchors.last().map_or(GPoint::ORIGIN, |a| a.point),
            };
        } else {
            self.anchors.clear();
            self.phase = PenPhase::Idle;
        }
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
            PointerPhase::Down => self.on_down(event, bridge, camera, snap),
            PointerPhase::Move => self.on_move(event, camera, snap),
            PointerPhase::Up => self.on_up(event),
            PointerPhase::Cancel => {
                self.cancel();
                Ok(ChangeSet::empty())
            }
        }
    }

    fn on_down(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut AubrietaGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, AubrietaError> {
        if event.button != PointerButton::Primary {
            return Ok(ChangeSet::empty());
        }

        let mut pt = event.doc_pos;
        if !event.modifiers.disable_snap {
            pt = snap.snap_point(pt, camera, &[]).point;
        }

        // Check if clicking near first anchor to close path loop
        if self.anchors.len() >= 2 {
            let first_pt = self.anchors[0].point;
            let screen_first = camera.doc_to_screen(first_pt);
            let screen_cur = camera.doc_to_screen(pt);
            let dist_px = ((screen_first.x - screen_cur.x).powi(2)
                + (screen_first.y - screen_cur.y).powi(2))
            .sqrt();

            if dist_px <= self.close_threshold_px {
                // Close loop and commit path
                return self.commit_path(bridge, true);
            }
        }

        // Add new anchor point
        let new_anchor = PenAnchor {
            point: pt,
            handle_in: None,
            handle_out: None,
            node_type: NodeType::Cusp,
        };
        self.anchors.push(new_anchor);
        let idx = self.anchors.len() - 1;

        // Enter handle adjust phase (if pointer continues dragging)
        self.phase = PenPhase::HandleAdjust {
            anchor_idx: idx,
            handle_pos: pt,
        };

        Ok(ChangeSet::empty())
    }

    fn on_move(
        &mut self,
        event: &NormalizedPointerEvent,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, AubrietaError> {
        let mut pt = event.doc_pos;
        if !event.modifiers.disable_snap {
            pt = snap.snap_point(pt, camera, &[]).point;
        }

        match &mut self.phase {
            PenPhase::HandleAdjust {
                anchor_idx,
                handle_pos,
            } => {
                *handle_pos = pt;
                if let Some(anchor) = self.anchors.get_mut(*anchor_idx) {
                    let dx = pt.x - anchor.point.x;
                    let dy = pt.y - anchor.point.y;
                    anchor.handle_out = Some(GPoint::new(dx, dy));
                    // Symmetric handle for in
                    anchor.handle_in = Some(GPoint::new(-dx, -dy));
                    anchor.node_type = NodeType::Smooth;
                }
            }
            _ => {
                if self.anchors.len() >= 2 {
                    let first_pt = self.anchors[0].point;
                    let screen_first = camera.doc_to_screen(first_pt);
                    let screen_cur = camera.doc_to_screen(pt);
                    let dist_px = ((screen_first.x - screen_cur.x).powi(2)
                        + (screen_first.y - screen_cur.y).powi(2))
                    .sqrt();

                    if dist_px <= self.close_threshold_px {
                        self.phase = PenPhase::ClosePreview { cursor_doc: pt };
                        return Ok(ChangeSet::empty());
                    }
                }

                if !self.anchors.is_empty() {
                    self.phase = PenPhase::SegmentPreview { cursor_doc: pt };
                }
            }
        }

        Ok(ChangeSet::empty())
    }

    fn on_up(&mut self, _event: &NormalizedPointerEvent) -> Result<ChangeSet, AubrietaError> {
        if let PenPhase::HandleAdjust { .. } = self.phase {
            let last_pt = self.anchors.last().map_or(GPoint::ORIGIN, |a| a.point);
            self.phase = PenPhase::SegmentPreview {
                cursor_doc: last_pt,
            };
        }
        Ok(ChangeSet::empty())
    }

    /// Commits the in-flight path into the active document surface.
    pub fn commit_path(
        &mut self,
        bridge: &mut AubrietaGuiBridge,
        _closed: bool,
    ) -> Result<ChangeSet, AubrietaError> {
        if self.anchors.is_empty() {
            return Ok(ChangeSet::empty());
        }

        let active_surface = bridge
            .session()
            .and_then(|s| s.active_surface)
            .ok_or_else(|| AubrietaError::invalid_input("no active surface for path creation"))?;

        let obj_id = bridge.next_object_id()?;

        // Calculate bounding box across anchors
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for a in &self.anchors {
            min_x = min_x.min(a.point.x);
            min_y = min_y.min(a.point.y);
            max_x = max_x.max(a.point.x);
            max_y = max_y.max(a.point.y);
        }

        let bounds = [
            min_x,
            min_y,
            (max_x - min_x).max(1.0),
            (max_y - min_y).max(1.0),
        ];

        let mut combined = ChangeSet::empty();

        let create_cmd = CommandRequest::new(Command::CreateObject {
            surface: active_surface,
            id: obj_id,
            name: format!("Path {}", self.anchors.len()),
        });
        let c1 = bridge.submit_command(create_cmd)?;
        for c in c1.changes {
            combined.push(c);
        }

        let bounds_cmd = CommandRequest::new(Command::SetBounds {
            id: obj_id,
            bounds: Some(bounds),
            rotation: 0.0,
        });
        let c2 = bridge.submit_command(bounds_cmd)?;
        for c in c2.changes {
            combined.push(c);
        }

        // Construct path from anchors
        let mut path = aubrieta_geometry::GPath::new();
        if let Some(first) = self.anchors.first() {
            let _ = path.push(aubrieta_geometry::PathVerb::MoveTo(first.point));
            for a in &self.anchors[1..] {
                let _ = path.push(aubrieta_geometry::PathVerb::LineTo(a.point));
            }
            if _closed {
                let _ = path.push(aubrieta_geometry::PathVerb::Close);
            }
        }
        let shape_cmd = CommandRequest::new(Command::SetShape {
            id: obj_id,
            shape: Some(aubrieta_document::ShapeKind::Path(path)),
        });
        let c_shape = bridge.submit_command(shape_cmd)?;
        for c in c_shape.changes {
            combined.push(c);
        }

        // Set default stroke
        let stroke_cmd = CommandRequest::new(Command::SetStroke {
            id: obj_id,
            stroke: Some("aubrieta.gray/900".to_string()),
            width: 1.5,
        });
        let c3 = bridge.submit_command(stroke_cmd)?;
        for c in c3.changes {
            combined.push(c);
        }

        // Select the newly created path
        bridge.set_selection(vec![obj_id]);

        // Reset pen state
        self.anchors.clear();
        self.phase = PenPhase::Idle;

        Ok(combined)
    }

    /// Resolves preview overlays for the Pen tool.
    #[must_use]
    pub fn overlays(&self) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        if self.anchors.is_empty() {
            return overlays;
        }

        let mut pts: Vec<GPoint> = self.anchors.iter().map(|a| a.point).collect();

        match &self.phase {
            PenPhase::SegmentPreview { cursor_doc } | PenPhase::ClosePreview { cursor_doc } => {
                pts.push(*cursor_doc);
            }
            _ => {}
        }

        overlays.pen_preview = Some(pts);
        overlays
    }
}
