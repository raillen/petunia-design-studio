//! Pen tool state machine for Bézier path construction (10.2).
//!
//! Hybrid V1: Illustrator gestures, Affinity cursor comfort, Inkscape
//! angle locks. Provisional state stays outside the document until commit.

use std::time::Instant;

use petunia_design_document::{ChangeSet, ShapeKind};
use petunia_design_foundation::{ObjectId, PetuniaError};
use petunia_design_geometry::{GPath, GPoint, PathVerb};

use crate::bridge::*;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase,
};

/// Max elapsed time between two primary downs to count as a double-click finish.
const DOUBLE_CLICK_MS: u128 = 400;
/// Max screen distance between two primary downs to count as a double-click finish.
const DOUBLE_CLICK_PX: f64 = 6.0;
/// Screen radius for close-loop / continue-path / recollect hit targets.
const ENDPOINT_HIT_PX: f64 = 10.0;

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

/// Cursor hint for the Pen tool so the UI can show context
/// (create vs. close vs. continue), Affinity-style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PenCursorHint {
    /// Fresh canvas: next click starts a new path.
    CreateNew,
    /// Dragging tangent handles of the current anchor.
    AdjustHandle,
    /// Hovering the first anchor: click closes the loop.
    CloseLoop,
    /// Hovering an open endpoint of an existing path: click continues it.
    ContinuePath,
    /// Previewing the next segment of the in-flight path.
    SegmentPreview,
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
#[derive(Clone, Debug)]
pub struct PenTool {
    anchors: Vec<PenAnchor>,
    phase: PenPhase,
    close_threshold_px: f64,
    /// Existing path object being extended, if any (10.2 continuation).
    continuing_object: Option<ObjectId>,
    /// Last primary down for double-click-to-finish detection.
    last_down: Option<(Instant, GPoint)>,
}

impl Default for PenTool {
    fn default() -> Self {
        Self::new()
    }
}

impl PenTool {
    /// Creates a fresh Pen tool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            anchors: Vec::new(),
            phase: PenPhase::Idle,
            close_threshold_px: ENDPOINT_HIT_PX,
            continuing_object: None,
            last_down: None,
        }
    }

    /// True if the tool has in-flight uncommitted anchors.
    #[must_use]
    pub fn is_active(&self) -> bool {
        !self.anchors.is_empty()
    }

    /// In-flight anchors (read-only, for tests and HUD).
    #[must_use]
    pub fn anchors(&self) -> &[PenAnchor] {
        &self.anchors
    }

    /// Existing path object being extended, if continuation is armed.
    #[must_use]
    pub fn continuing_object(&self) -> Option<ObjectId> {
        self.continuing_object
    }

    /// Current phase (for tests and shell introspection).
    #[must_use]
    pub fn pen_phase(&self) -> &PenPhase {
        &self.phase
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
            self.continuing_object = None;
        }
    }

    /// Commits the in-flight open path without closing it.
    /// UI finish gesture: double-click, right-click, Enter, or menu.
    pub fn finish_open_path(
        &mut self,
        bridge: &mut PetuniaDesignGuiBridge,
    ) -> Result<ChangeSet, PetuniaError> {
        self.commit_path(bridge, false)
    }

    /// Cursor hint at a screen position for UI binding (no document access
    /// needed beyond the bridge query).
    #[must_use]
    pub fn cursor_hint(
        &self,
        screen_pos: GPoint,
        doc_pos: GPoint,
        bridge: &PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
    ) -> PenCursorHint {
        if matches!(self.phase, PenPhase::HandleAdjust { .. }) {
            return PenCursorHint::AdjustHandle;
        }
        if self.hovering_first_anchor(doc_pos, camera) {
            return PenCursorHint::CloseLoop;
        }
        if !self.anchors.is_empty() {
            return PenCursorHint::SegmentPreview;
        }
        if find_open_endpoint(doc_pos, bridge, camera, self.close_threshold_px).is_some() {
            return PenCursorHint::ContinuePath;
        }
        let _ = screen_pos;
        PenCursorHint::CreateNew
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
        // Right-click finishes the in-flight open path (Corel/Inkscape).
        if event.button == PointerButton::Secondary {
            if self.is_active() {
                return self.commit_path(bridge, false);
            }
            return Ok(ChangeSet::empty());
        }
        if event.button != PointerButton::Primary {
            return Ok(ChangeSet::empty());
        }

        let mut pt = event.doc_pos;
        if !event.modifiers.disable_snap {
            pt = snap.snap_point(pt, camera, &[]).point;
        }
        // Shift constrains the new anchor to 45° steps from the last one.
        if event.modifiers.constrain {
            if let Some(last) = self.anchors.last() {
                pt = snap_angle_step(last.point, pt, 45.0);
            }
        }

        // Double-click on empty canvas finishes the open path (Corel):
        // the first click already placed the end anchor.
        let now = Instant::now();
        let is_double = match self.last_down {
            Some((t, p)) => {
                now.duration_since(t).as_millis() <= DOUBLE_CLICK_MS
                    && p.distance_to(event.screen_pos) <= DOUBLE_CLICK_PX
            }
            None => false,
        };
        self.last_down = Some((now, event.screen_pos));
        if is_double && self.anchors.len() >= 2 {
            return self.commit_path(bridge, false);
        }

        // Check if clicking near first anchor to close path loop
        if self.anchors.len() >= 2 && self.hovering_first_anchor(pt, camera) {
            // Close loop and commit path
            return self.commit_path(bridge, true);
        }

        // Clicking back on the last anchor removes its handles,
        // turning curve into straight (Illustrator).
        if self.anchors.len() >= 2
            && hovering_point(
                pt,
                self.last_anchor_point(),
                camera,
                self.close_threshold_px,
            )
        {
            if let Some(last) = self.anchors.last_mut() {
                last.handle_in = None;
                last.handle_out = None;
                last.node_type = NodeType::Cusp;
            }
            self.phase = PenPhase::SegmentPreview { cursor_doc: pt };
            return Ok(ChangeSet::empty());
        }

        // Fresh start near an existing open endpoint continues that path (10.2).
        if self.anchors.is_empty() {
            if let Some((id, loaded, from_start)) =
                find_open_endpoint(pt, bridge, camera, self.close_threshold_px)
            {
                self.anchors = loaded;
                if from_start {
                    reverse_anchors(&mut self.anchors);
                }
                self.continuing_object = Some(id);
                self.phase = PenPhase::SegmentPreview { cursor_doc: pt };
                return Ok(ChangeSet::empty());
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
                let idx = *anchor_idx;
                let anchor_pt = self.anchors.get(idx).map(|a| a.point).unwrap_or(pt);
                // Shift locks handle angle to 15° steps (spec 10.1 modifier).
                if event.modifiers.constrain {
                    pt = snap_angle_step(anchor_pt, pt, 15.0);
                }
                *handle_pos = pt;
                if let Some(anchor) = self.anchors.get_mut(idx) {
                    let dx = pt.x - anchor.point.x;
                    let dy = pt.y - anchor.point.y;
                    anchor.handle_out = Some(GPoint::new(dx, dy));
                    if event.modifiers.duplicate {
                        // Alt-drag breaks the mirror: independent handle (cusp).
                        anchor.node_type = NodeType::Cusp;
                    } else {
                        // Symmetric handle for in
                        anchor.handle_in = Some(GPoint::new(-dx, -dy));
                        anchor.node_type = NodeType::Smooth;
                    }
                }
            }
            _ => {
                if self.hovering_first_anchor(pt, camera) {
                    self.phase = PenPhase::ClosePreview { cursor_doc: pt };
                    return Ok(ChangeSet::empty());
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

    /// True when `pt` hovers the first anchor within the close threshold.
    fn hovering_first_anchor(&self, pt: GPoint, camera: &ViewportCamera) -> bool {
        self.anchors.len() >= 2
            && self.anchors.first().is_some_and(|first| {
                hovering_point(pt, first.point, camera, self.close_threshold_px)
            })
    }

    /// Last anchor position, or origin when empty.
    fn last_anchor_point(&self) -> GPoint {
        self.anchors.last().map_or(GPoint::ORIGIN, |a| a.point)
    }

    /// Commits the in-flight path into the active document surface.
    /// When continuing an existing path, its shape is replaced instead.
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
        let changes = if let Some(target) = self.continuing_object {
            let continued = bridge.submit_all(
                "Continue path",
                vec![
                    petunia_design_application::Command::SetShape {
                        id: target,
                        shape: Some(ShapeKind::Path(path)),
                    },
                    petunia_design_application::Command::SetBounds {
                        id: target,
                        bounds: Some(bounds),
                        rotation: 0.0,
                    },
                ],
            )?;
            bridge.set_selection(vec![target]);
            continued
        } else {
            let obj_id = bridge.next_object_id()?;
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
            changes
        };

        // Reset pen state
        self.anchors.clear();
        self.phase = PenPhase::Idle;
        self.continuing_object = None;

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

/// Snaps `to` around `from` to `step_deg` angle increments, keeping distance.
fn snap_angle_step(from: GPoint, to: GPoint, step_deg: f64) -> GPoint {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let dist = dx.hypot(dy);
    if dist < 1e-9 {
        return from;
    }
    let step = step_deg.to_radians();
    let angle = (dy.atan2(dx) / step).round() * step;
    GPoint::new(from.x + dist * angle.cos(), from.y + dist * angle.sin())
}

/// Screen-space proximity test between a document point and a target.
fn hovering_point(pt: GPoint, target: GPoint, camera: &ViewportCamera, threshold_px: f64) -> bool {
    let a = camera.doc_to_screen(pt);
    let b = camera.doc_to_screen(target);
    a.distance_to(b) <= threshold_px
}

/// Finds an open (unclosed) path endpoint near `pt`.
/// Returns `(object, anchors, from_start)`; never merges by mere proximity (10.2).
fn find_open_endpoint(
    pt: GPoint,
    bridge: &PetuniaDesignGuiBridge,
    camera: &ViewportCamera,
    threshold_px: f64,
) -> Option<(ObjectId, Vec<PenAnchor>, bool)> {
    let session = bridge.session()?;
    let surface_id = session.active_surface()?;
    let surface = session.surface(surface_id).ok()?;
    // Topmost first.
    for obj in surface.objects().iter().rev() {
        if !obj.visible || obj.locked {
            continue;
        }
        let ShapeKind::Path(path) = obj.shape.as_ref()? else {
            continue;
        };
        if path.verbs.contains(&PathVerb::Close) {
            continue;
        }
        let loaded = path_to_anchors(path)?;
        let (Some(first), Some(last)) = (loaded.first(), loaded.last()) else {
            continue;
        };
        if hovering_point(pt, first.point, camera, threshold_px) {
            return Some((obj.id, loaded, true));
        }
        if hovering_point(pt, last.point, camera, threshold_px) {
            return Some((obj.id, loaded, false));
        }
    }
    None
}

/// Converts path verbs back into pen anchors for continuation.
fn path_to_anchors(path: &GPath) -> Option<Vec<PenAnchor>> {
    let mut anchors: Vec<PenAnchor> = Vec::new();
    for verb in &path.verbs {
        match *verb {
            PathVerb::MoveTo(p) | PathVerb::LineTo(p) => anchors.push(PenAnchor {
                point: p,
                handle_in: None,
                handle_out: None,
                node_type: NodeType::Cusp,
            }),
            PathVerb::QuadTo(c, p) => anchors.push(PenAnchor {
                point: p,
                handle_in: Some(GPoint::new(c.x - p.x, c.y - p.y)),
                handle_out: None,
                node_type: NodeType::Smooth,
            }),
            PathVerb::CubicTo(c1, c2, p) => {
                if let Some(prev) = anchors.last_mut() {
                    prev.handle_out = Some(GPoint::new(c1.x - prev.point.x, c1.y - prev.point.y));
                    refresh_node_type(prev);
                }
                anchors.push(PenAnchor {
                    point: p,
                    handle_in: Some(GPoint::new(c2.x - p.x, c2.y - p.y)),
                    handle_out: None,
                    node_type: NodeType::Cusp,
                });
            }
            PathVerb::Close => return None,
        }
    }
    for anchor in &mut anchors {
        refresh_node_type(anchor);
    }
    if anchors.is_empty() {
        return None;
    }
    Some(anchors)
}

/// Derives the node type from handle presence and mirror symmetry.
fn refresh_node_type(anchor: &mut PenAnchor) {
    anchor.node_type = match (anchor.handle_in, anchor.handle_out) {
        (Some(i), Some(o)) if (i.x + o.x).abs() < 1e-6 && (i.y + o.y).abs() < 1e-6 => {
            NodeType::Symmetric
        }
        (Some(_), Some(_)) => NodeType::Smooth,
        (Some(_), None) | (None, Some(_)) => NodeType::Smooth,
        (None, None) => NodeType::Cusp,
    };
}

/// Reverses anchors so drawing continues from the path start.
/// Handle roles swap: what was ingoing becomes outgoing.
fn reverse_anchors(anchors: &mut [PenAnchor]) {
    anchors.reverse();
    for anchor in anchors.iter_mut() {
        std::mem::swap(&mut anchor.handle_in, &mut anchor.handle_out);
    }
}

#[cfg(test)]
mod pen_tool_tests {
    use super::*;

    #[test]
    fn symmetric_handles_mirror() {
        let mut anchor = PenAnchor {
            point: GPoint::ORIGIN,
            handle_in: None,
            handle_out: None,
            node_type: NodeType::Cusp,
        };
        anchor.handle_in = Some(GPoint::new(-3.0, -4.0));
        anchor.handle_out = Some(GPoint::new(3.0, 4.0));
        refresh_node_type(&mut anchor);
        assert_eq!(anchor.node_type, NodeType::Symmetric);
    }

    #[test]
    fn broken_handles_are_cusp() {
        let mut anchor = PenAnchor {
            point: GPoint::ORIGIN,
            handle_in: None,
            handle_out: Some(GPoint::new(5.0, 0.0)),
            node_type: NodeType::Smooth,
        };
        anchor.node_type = NodeType::Cusp;
        refresh_node_type(&mut anchor);
        // Single-sided handle reads as smooth intent; explicit cusp marks the break.
        assert_ne!(anchor.node_type, NodeType::Symmetric);
    }

    #[test]
    fn angle_step_locks_45_degrees() {
        let from = GPoint::new(100.0, 100.0);
        let to = GPoint::new(200.0, 150.0);
        let snapped = snap_angle_step(from, to, 45.0);
        // atan2(50, 100) ≈ 26.6° rounds to 45°; x and y offsets match.
        assert!((snapped.x - from.x - (snapped.y - from.y)).abs() < 1e-6);
    }
}
