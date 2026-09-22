//! Pen tool state machine for Bézier path construction (10.2).

use petunia_design_document::ChangeSet;
use petunia_design_foundation::PetuniaError;
use petunia_design_geometry::GPoint;

use crate::bridge::*;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase,
};

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
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
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
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
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
    ) -> Result<ChangeSet, PetuniaError> {
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

    fn on_up(&mut self, _event: &NormalizedPointerEvent) -> Result<ChangeSet, PetuniaError> {
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
        bridge: &mut PetuniaDesignGuiBridge,
        _closed: bool,
    ) -> Result<ChangeSet, PetuniaError> {
        if self.anchors.is_empty() {
            return Ok(ChangeSet::empty());
        }

        let active_surface = bridge
            .session()
            .and_then(|s| s.active_surface())
            .ok_or_else(|| PetuniaError::invalid_input("no active surface for path creation"))?;

        let obj_id = bridge.next_object_id()?;
        let anchor_count = self.anchors.len();

        // Absolute handle positions + bounding box over anchors and handles.
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        let mut include = |p: petunia_design_geometry::GPoint| {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        };
        let abs_anchors: Vec<(
            petunia_design_geometry::GPoint,
            Option<petunia_design_geometry::GPoint>,
            Option<petunia_design_geometry::GPoint>,
        )> = self
            .anchors
            .iter()
            .map(|a| {
                let abs_in = a.handle_in.map(|off| {
                    petunia_design_geometry::GPoint::new(a.point.x + off.x, a.point.y + off.y)
                });
                let abs_out = a.handle_out.map(|off| {
                    petunia_design_geometry::GPoint::new(a.point.x + off.x, a.point.y + off.y)
                });
                include(a.point);
                if let Some(h) = abs_in {
                    include(h);
                }
                if let Some(h) = abs_out {
                    include(h);
                }
                (a.point, abs_in, abs_out)
            })
            .collect();

        let bounds = [
            min_x,
            min_y,
            (max_x - min_x).max(1.0),
            (max_y - min_y).max(1.0),
        ];

        // Bézier handles are preserved via the shared geometry builder (F-02):
        // segments with handles become CubicTo, otherwise LineTo.
        let path = petunia_design_geometry::anchors_to_path(&abs_anchors, _closed)
            .map_err(PetuniaError::invalid_input)?;

        // One gesture, one undo entry (F-01).
        let changes = bridge.submit_all(
            "Create path",
            petunia_design_application::create_shape_commands(
                active_surface,
                obj_id,
                format!("Path {anchor_count}"),
                petunia_design_document::ShapeKind::Path(path),
                Some(bounds),
                None,
                Some((
                    petunia_design_document::shape_factory::DEFAULT_PATH_STROKE.to_string(),
                    petunia_design_document::shape_factory::DEFAULT_PATH_STROKE_WIDTH,
                )),
            ),
        )?;

        // Select the newly created path
        bridge.set_selection(vec![obj_id]);

        // Reset pen state
        self.anchors.clear();
        self.phase = PenPhase::Idle;

        Ok(changes)
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
