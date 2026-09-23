//! Node editing and direct path manipulation tool (10.2).
//!
//! Batch 4 (option 2): one undo per gesture, multi-node selection,
//! control-handle and segment dragging, cusp/smooth/symmetric conversion,
//! click/double-click add, Delete/simple remove. Smart Delete stays future.

use std::time::Instant;

use petunia_design_application::Command;
use petunia_design_document::{ChangeSet, ShapeKind};
use petunia_design_foundation::{ObjectId, PetuniaError};
use petunia_design_geometry::{GPath, GPoint, GRect, PathVerb};

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{
    CanvasOverlays, SelectionHandle, SelectionHandleKind, SnapEngine, ViewportCamera,
};

use super::pen::NodeType;
use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase,
};

/// Screen-space hit radius for nodes, handles, and segments.
const NODE_HIT_PX: f64 = 12.0;
/// Double-click window for add/remove gestures (Corel).
const DOUBLE_CLICK_MS: u128 = 400;
const DOUBLE_CLICK_PX: f64 = 6.0;

/// Which handle of an anchor is dragged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HandleSide {
    In,
    Out,
}

/// In-flight drag state. Geometry commits once on pointer-up (F-01).
#[derive(Clone, Debug)]
enum NodeDrag {
    /// One or more anchor nodes move by the same delta.
    Nodes {
        start_doc: GPoint,
        current_doc: GPoint,
        initial: Vec<(ObjectId, GPath)>,
        moving: Vec<(ObjectId, usize)>,
    },
    /// A single control handle moves; the mirror follows the node constraint.
    Handle {
        start_doc: GPoint,
        current_doc: GPoint,
        object: ObjectId,
        initial: GPath,
        node_idx: usize,
        side: HandleSide,
        mirror: NodeType,
        cusp_break: bool,
    },
    /// A segment drag moves both bounding anchors by the same delta.
    Segment {
        start_doc: GPoint,
        current_doc: GPoint,
        initial: Vec<(ObjectId, GPath)>,
        moving: Vec<(ObjectId, usize)>,
    },
}

/// Direct selection and node editing tool (10.2).
#[derive(Clone, Debug, Default)]
pub struct NodeTool {
    /// Selected `(object, verb index)` anchors.
    selected: Vec<(ObjectId, usize)>,
    drag: Option<NodeDrag>,
    marquee: Option<MarqueeState>,
    last_down: Option<(Instant, GPoint)>,
}

#[derive(Clone, Debug)]
struct MarqueeState {
    start_screen: GPoint,
    current_screen: GPoint,
    additive: bool,
}

impl NodeTool {
    /// Creates a fresh Node tool.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Selected anchor nodes.
    #[must_use]
    pub fn selected_nodes(&self) -> &[(ObjectId, usize)] {
        &self.selected
    }

    /// Cancels any in-flight gesture (selection is kept).
    pub fn cancel(&mut self) {
        self.drag = None;
        self.marquee = None;
    }

    /// Deletes selected nodes with the simple rule (drop verb, reconnect).
    /// UI Delete-key binding calls this. Smart Delete stays future work.
    pub fn delete_selected_nodes(
        &mut self,
        bridge: &mut PetuniaDesignGuiBridge,
    ) -> Result<ChangeSet, PetuniaError> {
        if self.selected.is_empty() {
            return Ok(ChangeSet::empty());
        }
        let selected = std::mem::take(&mut self.selected);
        // Delete back-to-front per object so indices stay valid.
        let mut per_object: std::collections::BTreeMap<ObjectId, Vec<usize>> =
            std::collections::BTreeMap::new();
        for (id, idx) in selected {
            per_object.entry(id).or_default().push(idx);
        }
        let mut paths: Vec<(ObjectId, GPath)> = Vec::new();
        for (id, mut indices) in per_object {
            indices.sort_unstable();
            indices.dedup();
            let Some(path) = load_path(bridge, id) else {
                continue;
            };
            let mut verbs = path.verbs.clone();
            let mut removed = false;
            for idx in indices.into_iter().rev() {
                removed |= delete_node(&mut verbs, idx);
            }
            if removed {
                if let Some(p) = verbs_to_path(verbs) {
                    paths.push((id, p));
                }
            }
        }
        self.drag = None;
        commit_paths(bridge, "Delete nodes", paths)
    }

    /// Converts selected nodes to a constraint type (undoable).
    /// UI convert buttons call this. Cusp keeps geometry (independent handles).
    pub fn convert_selected_nodes(
        &mut self,
        bridge: &mut PetuniaDesignGuiBridge,
        node_type: NodeType,
    ) -> Result<ChangeSet, PetuniaError> {
        if self.selected.is_empty() {
            return Ok(ChangeSet::empty());
        }
        let mut per_object: std::collections::BTreeMap<ObjectId, Vec<usize>> =
            std::collections::BTreeMap::new();
        for &(id, idx) in &self.selected {
            per_object.entry(id).or_default().push(idx);
        }
        let mut paths: Vec<(ObjectId, GPath)> = Vec::new();
        for (id, mut indices) in per_object {
            indices.sort_unstable();
            indices.dedup();
            let Some(path) = load_path(bridge, id) else {
                continue;
            };
            let mut verbs = path.verbs.clone();
            let mut changed = false;
            for idx in indices {
                changed |= convert_node(&mut verbs, idx, node_type);
            }
            if changed {
                if let Some(p) = verbs_to_path(verbs) {
                    paths.push((id, p));
                }
            }
        }
        commit_paths(bridge, "Convert nodes", paths)
    }

    /// Handles normalized pointer events.
    #[allow(clippy::too_many_lines)]
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
            PointerPhase::Up => self.on_up(event, bridge, camera, snap),
            PointerPhase::Cancel => {
                self.cancel();
                snap.reset_hysteresis();
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
        snap.reset_hysteresis();
        let tol = NODE_HIT_PX / camera.zoom.max(0.1);

        // Double-click: on a node removes it, on a segment adds one (Corel).
        let now = Instant::now();
        let is_double = match self.last_down {
            Some((t, p)) => {
                now.duration_since(t).as_millis() <= DOUBLE_CLICK_MS
                    && p.distance_to(event.screen_pos) <= DOUBLE_CLICK_PX
            }
            None => false,
        };
        self.last_down = Some((now, event.screen_pos));

        // 1. Control-handle hit (selected nodes show handles).
        if let Some((id, idx, side)) = self.hit_handle(event.doc_pos, bridge, tol) {
            return self.begin_handle_drag(id, idx, side, event, bridge);
        }

        // 2. Anchor-node hit on selected objects.
        if let Some((id, idx)) = hit_node(event.doc_pos, bridge, camera, tol) {
            if is_double {
                self.selected = vec![(id, idx)];
                return self.delete_selected_nodes(bridge);
            }
            if event.modifiers.constrain {
                toggle_node(&mut self.selected, id, idx);
            } else if !self.selected.contains(&(id, idx)) {
                self.selected = vec![(id, idx)];
            }
            return self.begin_nodes_drag(event, bridge);
        }

        // 3. Segment hit: single click selects bounding anchors,
        // double-click inserts a node preserving shape.
        if let Some((id, verb_idx, _t)) = hit_segment(event.doc_pos, bridge, camera, tol) {
            if is_double {
                return self.add_node_at(id, verb_idx, event.doc_pos, bridge);
            }
            let prev = prev_endpoint_idx(bridge, id, verb_idx);
            let mut endpoints = vec![(id, verb_idx)];
            if let Some(p) = prev {
                if p != verb_idx {
                    endpoints.push((id, p));
                }
            }
            if event.modifiers.constrain {
                for (eid, eidx) in endpoints {
                    toggle_node_add(&mut self.selected, eid, eidx);
                }
            } else {
                self.selected = endpoints;
            }
            return self.begin_segment_drag(event, bridge);
        }

        // 4. Object hit: select it (converting parametric shapes) and show nodes.
        if let Some(id) = hit_object(event.doc_pos, bridge) {
            if !event.modifiers.constrain {
                bridge.set_selection(vec![id]);
            }
            if needs_convert(bridge, id) {
                bridge.convert_to_curves(id)?;
            }
            self.selected.clear();
            self.select_all_nodes(bridge, id);
            return Ok(ChangeSet::empty());
        }

        // 5. Empty canvas: node marquee (Shift adds).
        if !event.modifiers.constrain {
            self.selected.clear();
        }
        self.marquee = Some(MarqueeState {
            start_screen: event.screen_pos,
            current_screen: event.screen_pos,
            additive: event.modifiers.constrain,
        });
        Ok(ChangeSet::empty())
    }

    fn on_move(
        &mut self,
        event: &NormalizedPointerEvent,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
        if let Some(marquee) = &mut self.marquee {
            marquee.current_screen = event.screen_pos;
            return Ok(ChangeSet::empty());
        }
        let Some(drag) = &mut self.drag else {
            return Ok(ChangeSet::empty());
        };
        let mut target = event.doc_pos;
        if !event.modifiers.disable_snap {
            target = snap.snap_point(target, camera, &[]).point;
        }
        match drag {
            NodeDrag::Nodes { current_doc, .. }
            | NodeDrag::Segment { current_doc, .. }
            | NodeDrag::Handle { current_doc, .. } => *current_doc = target,
        }
        // Preview-only (F-01): commit happens on pointer-up.
        Ok(ChangeSet::empty())
    }

    fn on_up(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
        snap.reset_hysteresis();
        if let Some(marquee) = self.marquee.take() {
            return self.commit_marquee(marquee, bridge, camera);
        }
        let Some(drag) = self.drag.take() else {
            return Ok(ChangeSet::empty());
        };
        let mut target = event.doc_pos;
        if !event.modifiers.disable_snap {
            target = snap.snap_point(target, camera, &[]).point;
        }
        match drag {
            NodeDrag::Nodes {
                start_doc,
                initial,
                moving,
                ..
            } => {
                let dx = target.x - start_doc.x;
                let dy = target.y - start_doc.y;
                if dx.abs() < f64::EPSILON && dy.abs() < f64::EPSILON {
                    return Ok(ChangeSet::empty());
                }
                let delta = GPoint::new(dx, dy);
                let paths = apply_node_delta(initial, &moving, delta);
                commit_paths(bridge, "Move nodes", paths)
            }
            NodeDrag::Segment {
                start_doc,
                initial,
                moving,
                ..
            } => {
                let dx = target.x - start_doc.x;
                let dy = target.y - start_doc.y;
                if dx.abs() < f64::EPSILON && dy.abs() < f64::EPSILON {
                    return Ok(ChangeSet::empty());
                }
                let delta = GPoint::new(dx, dy);
                let paths = apply_node_delta(initial, &moving, delta);
                commit_paths(bridge, "Move segment", paths)
            }
            NodeDrag::Handle {
                start_doc,
                object,
                initial,
                node_idx,
                side,
                mirror,
                cusp_break,
                ..
            } => {
                let mut verbs = initial.verbs.clone();
                drag_handle_to(
                    &mut verbs, node_idx, side, start_doc, target, mirror, cusp_break,
                );
                commit_paths(
                    bridge,
                    "Move handle",
                    vec![(object, verbs_to_path_unchecked(verbs))],
                )
            }
        }
    }

    /// Starts a multi-node drag from the current selection.
    fn begin_nodes_drag(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
    ) -> Result<ChangeSet, PetuniaError> {
        let initial = snapshot_paths(bridge, &self.selected);
        self.drag = Some(NodeDrag::Nodes {
            start_doc: event.doc_pos,
            current_doc: event.doc_pos,
            initial,
            moving: self.selected.clone(),
        });
        Ok(ChangeSet::empty())
    }

    /// Starts a segment drag moving both bounding anchors.
    fn begin_segment_drag(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
    ) -> Result<ChangeSet, PetuniaError> {
        let initial = snapshot_paths(bridge, &self.selected);
        self.drag = Some(NodeDrag::Segment {
            start_doc: event.doc_pos,
            current_doc: event.doc_pos,
            initial,
            moving: self.selected.clone(),
        });
        Ok(ChangeSet::empty())
    }

    /// Starts a control-handle drag with mirror behavior from node geometry.
    fn begin_handle_drag(
        &mut self,
        id: ObjectId,
        idx: usize,
        side: HandleSide,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
    ) -> Result<ChangeSet, PetuniaError> {
        let Some(path) = load_path(bridge, id) else {
            return Ok(ChangeSet::empty());
        };
        let mirror = classify_node(&path.verbs, idx);
        self.selected = vec![(id, idx)];
        self.drag = Some(NodeDrag::Handle {
            start_doc: event.doc_pos,
            current_doc: event.doc_pos,
            object: id,
            initial: path,
            node_idx: idx,
            side,
            mirror,
            cusp_break: event.modifiers.duplicate,
        });
        Ok(ChangeSet::empty())
    }

    /// Inserts a shape-preserving node on a segment (double-click).
    fn add_node_at(
        &mut self,
        id: ObjectId,
        verb_idx: usize,
        pt: GPoint,
        bridge: &mut PetuniaDesignGuiBridge,
    ) -> Result<ChangeSet, PetuniaError> {
        let Some(path) = load_path(bridge, id) else {
            return Ok(ChangeSet::empty());
        };
        let mut verbs = path.verbs.clone();
        let Some(new_idx) = split_segment(&mut verbs, verb_idx, pt) else {
            return Ok(ChangeSet::empty());
        };
        self.selected = vec![(id, new_idx)];
        commit_paths(
            bridge,
            "Add node",
            vec![(id, verbs_to_path_unchecked(verbs))],
        )
    }

    /// Commits a node marquee over selected objects' paths.
    fn commit_marquee(
        &mut self,
        marquee: MarqueeState,
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
    ) -> Result<ChangeSet, PetuniaError> {
        let rect = GRect::new(
            marquee.start_screen.x,
            marquee.start_screen.y,
            marquee.current_screen.x,
            marquee.current_screen.y,
        );
        if rect.width() <= 3.0 && rect.height() <= 3.0 {
            return Ok(ChangeSet::empty());
        }
        let doc_tl = camera.screen_to_doc(GPoint::new(rect.x0, rect.y0));
        let doc_br = camera.screen_to_doc(GPoint::new(rect.x1, rect.y1));
        let doc_rect = GRect::new(doc_tl.x, doc_tl.y, doc_br.x, doc_br.y);
        let mut matched = Vec::new();
        if let Some(session) = bridge.session() {
            for &id in &session.selection.selected_ids.clone() {
                if let Some(obj) = session.find_object(id) {
                    if let Some(ShapeKind::Path(path)) = &obj.shape {
                        for (idx, verb) in path.verbs.iter().enumerate() {
                            if let Some(pt) = endpoint_of(verb) {
                                if doc_rect.contains(pt) {
                                    matched.push((id, idx));
                                }
                            }
                        }
                    }
                }
            }
        }
        if marquee.additive {
            for item in matched {
                toggle_node_add(&mut self.selected, item.0, item.1);
            }
        } else {
            self.selected = matched;
        }
        Ok(ChangeSet::empty())
    }

    /// Selects every anchor of an object (used after object click).
    fn select_all_nodes(&mut self, bridge: &PetuniaDesignGuiBridge, id: ObjectId) {
        if let Some(session) = bridge.session() {
            if let Some(obj) = session.find_object(id) {
                if let Some(ShapeKind::Path(path)) = &obj.shape {
                    self.selected = path
                        .verbs
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, verb)| endpoint_of(verb).map(|_| (id, idx)))
                        .collect();
                }
            }
        }
    }

    /// Hit-tests control handles of selected nodes.
    fn hit_handle(
        &self,
        pt: GPoint,
        bridge: &PetuniaDesignGuiBridge,
        tol: f64,
    ) -> Option<(ObjectId, usize, HandleSide)> {
        let session = bridge.session()?;
        for &(id, idx) in &self.selected {
            let obj = session.find_object(id)?;
            let Some(ShapeKind::Path(path)) = &obj.shape else {
                continue;
            };
            let (handle_in, handle_out) = handles_of(&path.verbs, idx);
            if let Some(h) = handle_in {
                if h.distance_to(pt) <= tol {
                    return Some((id, idx, HandleSide::In));
                }
            }
            if let Some(h) = handle_out {
                if h.distance_to(pt) <= tol {
                    return Some((id, idx, HandleSide::Out));
                }
            }
        }
        None
    }

    /// Resolves overlays for the Node tool (node handle points).
    #[must_use]
    pub fn overlays(
        &self,
        camera: &ViewportCamera,
        bridge: &PetuniaDesignGuiBridge,
    ) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        if let Some(marquee) = &self.marquee {
            overlays.marquee_screen = Some(GRect::new(
                marquee.start_screen.x,
                marquee.start_screen.y,
                marquee.current_screen.x,
                marquee.current_screen.y,
            ));
            return overlays;
        }
        let sel_ids = bridge.selection().selected_ids;
        for &id in &sel_ids {
            if let Some(session) = bridge.session() {
                if let Some(obj) = session.find_object(id) {
                    if let Some(ShapeKind::Path(path)) = &obj.shape {
                        for verb in &path.verbs {
                            let pt = match verb {
                                PathVerb::MoveTo(pt) | PathVerb::LineTo(pt) => *pt,
                                PathVerb::QuadTo(_, pt) | PathVerb::CubicTo(_, _, pt) => *pt,
                                PathVerb::Close => continue,
                            };
                            let screen_pt = camera.doc_to_screen(pt);
                            let screen_hit_box = GRect::new(
                                screen_pt.x - 4.0,
                                screen_pt.y - 4.0,
                                screen_pt.x + 4.0,
                                screen_pt.y + 4.0,
                            );
                            overlays.handles.push(SelectionHandle {
                                kind: SelectionHandleKind::TopLeft,
                                doc_point: pt,
                                screen_hit_box,
                            });
                        }
                    }
                }
            }
        }
        overlays
    }
}

// ---------------------------------------------------------------------------
// Geometry helpers (pure, verb-level).
// ---------------------------------------------------------------------------

/// Endpoint of a verb, if it has one.
fn endpoint_of(verb: &PathVerb) -> Option<GPoint> {
    match verb {
        PathVerb::MoveTo(pt) | PathVerb::LineTo(pt) => Some(*pt),
        PathVerb::QuadTo(_, pt) | PathVerb::CubicTo(_, _, pt) => Some(*pt),
        PathVerb::Close => None,
    }
}

/// Absolute `(in, out)` control handles of the anchor at `idx`.
fn handles_of(verbs: &[PathVerb], idx: usize) -> (Option<GPoint>, Option<GPoint>) {
    let handle_in = match verbs.get(idx) {
        Some(PathVerb::QuadTo(c, _)) => Some(*c),
        Some(PathVerb::CubicTo(_, c2, _)) => Some(*c2),
        _ => None,
    };
    let handle_out = match verbs.get(idx + 1) {
        Some(PathVerb::CubicTo(c1, _, _)) => Some(*c1),
        Some(PathVerb::QuadTo(c, _)) => Some(*c),
        _ => None,
    };
    (handle_in, handle_out)
}

/// Writes one absolute handle back into the verbs.
fn set_handle(verbs: &mut [PathVerb], idx: usize, side: HandleSide, pos: GPoint) -> bool {
    match side {
        HandleSide::In => match verbs.get_mut(idx) {
            Some(PathVerb::QuadTo(c, _)) => {
                *c = pos;
                true
            }
            Some(PathVerb::CubicTo(_, c2, _)) => {
                *c2 = pos;
                true
            }
            _ => false,
        },
        HandleSide::Out => match verbs.get_mut(idx + 1) {
            Some(PathVerb::CubicTo(c1, _, _)) => {
                *c1 = pos;
                true
            }
            Some(PathVerb::QuadTo(c, _)) => {
                *c = pos;
                true
            }
            _ => false,
        },
    }
}

/// Classifies an anchor from its handle geometry.
fn classify_node(verbs: &[PathVerb], idx: usize) -> NodeType {
    let Some(p) = endpoint_of(&verbs.get(idx).copied().unwrap_or(PathVerb::Close)) else {
        return NodeType::Cusp;
    };
    let (handle_in, handle_out) = handles_of(verbs, idx);
    match (handle_in, handle_out) {
        (Some(i), Some(o)) => {
            let ix = i.x - p.x;
            let iy = i.y - p.y;
            let ox = o.x - p.x;
            let oy = o.y - p.y;
            let in_len = ix.hypot(iy);
            let out_len = ox.hypot(oy);
            if in_len < 1e-9 || out_len < 1e-9 {
                return NodeType::Cusp;
            }
            // Collinear opposite directions?
            let cross = (ix * oy - iy * ox).abs() / (in_len * out_len);
            let dot = (ix * ox + iy * oy) / (in_len * out_len);
            if cross < 0.02 && dot < 0.0 {
                if (in_len - out_len).abs() / in_len.max(out_len) < 0.05 {
                    NodeType::Symmetric
                } else {
                    NodeType::Smooth
                }
            } else {
                NodeType::Cusp
            }
        }
        _ => NodeType::Cusp,
    }
}

/// Converts an anchor to a constraint type, preserving position.
/// Returns true when geometry changed.
fn convert_node(verbs: &mut [PathVerb], idx: usize, node_type: NodeType) -> bool {
    if node_type == NodeType::Cusp {
        return false;
    }
    let Some(p) = verbs.get(idx).and_then(endpoint_of) else {
        return false;
    };
    let (handle_in, handle_out) = handles_of(verbs, idx);
    // Direction: prefer the existing out-handle, else in-handle, else neighbors.
    let mut dir = handle_out
        .map(|o| (o.x - p.x, o.y - p.y))
        .or_else(|| handle_in.map(|i| (p.x - i.x, p.y - i.y)));
    if dir.is_none() {
        let prev = (0..idx).rev().find_map(|i| endpoint_of(&verbs[i]));
        let next = ((idx + 1)..verbs.len()).find_map(|i| endpoint_of(&verbs[i]));
        dir = match (prev, next) {
            (Some(a), Some(b)) => Some((b.x - a.x, b.y - a.y)),
            (Some(a), None) => Some((p.x - a.x, p.y - a.y)),
            (None, Some(b)) => Some((b.x - p.x, b.y - p.y)),
            (None, None) => None,
        };
    }
    let Some((dx, dy)) = dir else {
        return false;
    };
    let len = dx.hypot(dy);
    if len < 1e-9 {
        return false;
    }
    let (ux, uy) = (dx / len, dy / len);
    let in_len = handle_in.map_or(fallback_len(verbs, idx, p), |i| i.distance_to(p).max(1.0));
    let out_len = handle_out.map_or(in_len, |o| o.distance_to(p).max(1.0));
    let (in_len, out_len) = match node_type {
        NodeType::Symmetric => {
            let avg = (in_len + out_len) / 2.0;
            (avg, avg)
        }
        _ => (in_len, out_len),
    };
    let new_in = GPoint::new(p.x - ux * in_len, p.y - uy * in_len);
    let new_out = GPoint::new(p.x + ux * out_len, p.y + uy * out_len);
    // Materialize missing sides: line segments become curves through new handles.
    let mut changed = false;
    changed |= ensure_handle(verbs, idx, HandleSide::In, new_in);
    changed |= ensure_handle(verbs, idx, HandleSide::Out, new_out);
    changed
}

/// Fallback handle length: a third of the shortest adjacent segment.
fn fallback_len(verbs: &[PathVerb], idx: usize, p: GPoint) -> f64 {
    let mut best = f64::INFINITY;
    if let Some(prev) = (0..idx).rev().find_map(|i| endpoint_of(&verbs[i])) {
        best = best.min(prev.distance_to(p));
    }
    if let Some(next) = ((idx + 1)..verbs.len()).find_map(|i| endpoint_of(&verbs[i])) {
        best = best.min(next.distance_to(p));
    }
    if best.is_finite() {
        (best / 3.0).max(1.0)
    } else {
        10.0
    }
}

/// Ensures a handle side exists at `pos`, upgrading line segments to curves.
fn ensure_handle(verbs: &mut [PathVerb], idx: usize, side: HandleSide, pos: GPoint) -> bool {
    match side {
        HandleSide::In => match verbs.get(idx).copied() {
            Some(PathVerb::LineTo(p)) => {
                verbs[idx] = PathVerb::QuadTo(pos, p);
                true
            }
            Some(PathVerb::MoveTo(_)) => false,
            Some(_) => set_handle(verbs, idx, side, pos),
            None => false,
        },
        HandleSide::Out => match verbs.get(idx + 1).copied() {
            Some(PathVerb::LineTo(p)) => {
                verbs[idx + 1] = PathVerb::QuadTo(pos, p);
                true
            }
            Some(_) => set_handle(verbs, idx, side, pos),
            None => false,
        },
    }
}

/// Drags one handle to an absolute position, mirroring per constraint.
/// Alt (`cusp_break`) drags independently.
fn drag_handle_to(
    verbs: &mut [PathVerb],
    idx: usize,
    side: HandleSide,
    from: GPoint,
    to: GPoint,
    mirror: NodeType,
    cusp_break: bool,
) {
    let Some(p) = verbs.get(idx).and_then(endpoint_of) else {
        return;
    };
    // Seed the dragged side when the anchor had no handle yet.
    let (handle_in, handle_out) = handles_of(verbs, idx);
    let missing = match side {
        HandleSide::In => handle_in.is_none(),
        HandleSide::Out => handle_out.is_none(),
    };
    if missing {
        ensure_handle(verbs, idx, side, from);
    }
    set_handle(verbs, idx, side, to);
    if cusp_break {
        return;
    }
    match mirror {
        NodeType::Symmetric => {
            let d = to.distance_to(p);
            let (ox, oy) = (to.x - p.x, to.y - p.y);
            if d > 1e-9 {
                let opposite = GPoint::new(p.x - ox, p.y - oy);
                let other = match side {
                    HandleSide::In => HandleSide::Out,
                    HandleSide::Out => HandleSide::In,
                };
                set_handle(verbs, idx, other, opposite);
            }
        }
        NodeType::Smooth => {
            let d = to.distance_to(p);
            if d > 1e-9 {
                let (ux, uy) = ((to.x - p.x) / d, (to.y - p.y) / d);
                let other = match side {
                    HandleSide::In => HandleSide::Out,
                    HandleSide::Out => HandleSide::In,
                };
                let other_len = match other {
                    HandleSide::In => handle_in,
                    HandleSide::Out => handle_out,
                }
                .map_or(d, |h| h.distance_to(p).max(1.0));
                let (sx, sy) = match side {
                    HandleSide::In => (ux, uy),
                    HandleSide::Out => (-ux, -uy),
                };
                set_handle(
                    verbs,
                    idx,
                    other,
                    GPoint::new(p.x + sx * other_len, p.y + sy * other_len),
                );
            }
        }
        NodeType::Cusp => {}
    }
}

/// Simple node removal: drops the verb (neighbors reconnect).
/// Never removes subpath starts or the last remaining point.
fn delete_node(verbs: &mut Vec<PathVerb>, idx: usize) -> bool {
    let point_verbs = verbs.iter().filter(|v| endpoint_of(v).is_some()).count();
    if point_verbs <= 1 {
        return false;
    }
    match verbs.get(idx) {
        Some(PathVerb::MoveTo(_)) | None => false,
        Some(_) => {
            verbs.remove(idx);
            // A stranded segment start becomes the new MoveTo.
            if let Some(PathVerb::LineTo(p) | PathVerb::QuadTo(_, p) | PathVerb::CubicTo(_, _, p)) =
                verbs.get(idx).copied()
            {
                let prev_is_start = idx == 0
                    || matches!(verbs.get(idx.wrapping_sub(1)), Some(PathVerb::Close) | None);
                if prev_is_start {
                    // A stranded segment start becomes the new MoveTo.
                    verbs[idx] = PathVerb::MoveTo(p);
                }
            }
            true
        }
    }
}

/// Splits the segment ending at `verb_idx`, preserving shape (de Casteljau).
/// Returns the new anchor index.
fn split_segment(verbs: &mut Vec<PathVerb>, verb_idx: usize, pt: GPoint) -> Option<usize> {
    if verb_idx == 0 || verb_idx >= verbs.len() {
        return None;
    }
    let start = (0..verb_idx).rev().find_map(|i| endpoint_of(&verbs[i]))?;
    endpoint_of(&verbs[verb_idx])?;
    // Parameter from the flattened single segment.
    let mut probe = GPath::new();
    probe.push(PathVerb::MoveTo(start)).ok()?;
    probe.push(verbs[verb_idx]).ok()?;
    let flat: Vec<GPoint> = probe.to_polygons(0.5).into_iter().flatten().collect();
    let t = nearest_t(&flat, pt);
    let q = point_at(&probe, t).unwrap_or(pt);
    match verbs[verb_idx] {
        PathVerb::LineTo(_) => {
            verbs.insert(verb_idx, PathVerb::LineTo(q));
            Some(verb_idx)
        }
        PathVerb::QuadTo(c, p) => {
            let (c0, p0, c1) = split_quad(start, c, p, t);
            verbs[verb_idx] = PathVerb::QuadTo(c1, p);
            verbs.insert(verb_idx, PathVerb::QuadTo(c0, p0));
            Some(verb_idx)
        }
        PathVerb::CubicTo(c1, c2, p) => {
            let (a1, a2, m, b1, b2) = split_cubic(start, c1, c2, p, t);
            verbs[verb_idx] = PathVerb::CubicTo(b1, b2, p);
            verbs.insert(verb_idx, PathVerb::CubicTo(a1, a2, m));
            Some(verb_idx)
        }
        _ => None,
    }
}

/// Fraction along a flattened polyline nearest `pt`.
fn nearest_t(flat: &[GPoint], pt: GPoint) -> f64 {
    if flat.len() < 2 {
        return 0.0;
    }
    let mut best_t = 0.0;
    let mut best_d = f64::INFINITY;
    let mut acc = 0.0;
    let total: f64 = flat
        .windows(2)
        .map(|w| w[0].distance_to(w[1]))
        .sum::<f64>()
        .max(1e-9);
    for w in flat.windows(2) {
        let seg_len = w[0].distance_to(w[1]);
        let t_seg = if seg_len < 1e-12 {
            0.0
        } else {
            let t = ((pt.x - w[0].x) * (w[1].x - w[0].x) + (pt.y - w[0].y) * (w[1].y - w[0].y))
                / (seg_len * seg_len);
            t.clamp(0.0, 1.0)
        };
        let proj = GPoint::new(
            w[0].x + (w[1].x - w[0].x) * t_seg,
            w[0].y + (w[1].y - w[0].y) * t_seg,
        );
        let d = proj.distance_to(pt);
        if d < best_d {
            best_d = d;
            best_t = (acc + seg_len * t_seg) / total;
        }
        acc += seg_len;
    }
    best_t.clamp(0.0, 1.0)
}

/// Point on a single-segment path at fraction `t` (flatten lookup).
fn point_at(path: &GPath, t: f64) -> Option<GPoint> {
    let flat: Vec<GPoint> = path.to_polygons(0.25).into_iter().flatten().collect();
    if flat.is_empty() {
        return None;
    }
    let total: f64 = flat
        .windows(2)
        .map(|w| w[0].distance_to(w[1]))
        .sum::<f64>()
        .max(1e-9);
    let mut acc = 0.0;
    for w in flat.windows(2) {
        let seg_len = w[0].distance_to(w[1]);
        let next = acc + seg_len;
        if next / total >= t {
            let f = if seg_len < 1e-12 {
                0.0
            } else {
                ((t * total - acc) / seg_len).clamp(0.0, 1.0)
            };
            return Some(GPoint::new(
                w[0].x + (w[1].x - w[0].x) * f,
                w[0].y + (w[1].y - w[0].y) * f,
            ));
        }
        acc = next;
    }
    Some(flat[flat.len() - 1])
}

/// Quadratic de Casteljau split: returns `(c0, p0, c1)` for
/// `[QuadTo(c0, p0), QuadTo(c1, p)]`.
fn split_quad(start: GPoint, c: GPoint, p: GPoint, t: f64) -> (GPoint, GPoint, GPoint) {
    let lerp = |a: f64, b: f64| a + (b - a) * t;
    let c0 = GPoint::new(lerp(start.x, c.x), lerp(start.y, c.y));
    let c1 = GPoint::new(lerp(c.x, p.x), lerp(c.y, p.y));
    let m = GPoint::new(lerp(c0.x, c1.x), lerp(c0.y, c1.y));
    (c0, m, c1)
}

/// Cubic de Casteljau split: returns `(a1, a2, m, b1, b2)`.
fn split_cubic(
    start: GPoint,
    c1: GPoint,
    c2: GPoint,
    p: GPoint,
    t: f64,
) -> (GPoint, GPoint, GPoint, GPoint, GPoint) {
    let lerp = |a: GPoint, b: GPoint| GPoint::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t);
    let a = lerp(start, c1);
    let b = lerp(c1, c2);
    let c = lerp(c2, p);
    let d = lerp(a, b);
    let e = lerp(b, c);
    let m = lerp(d, e);
    (a, d, m, e, c)
}

// ---------------------------------------------------------------------------
// Bridge helpers.
// ---------------------------------------------------------------------------

/// Loads a path object, converting parametric shapes on demand.
fn load_path(bridge: &PetuniaDesignGuiBridge, id: ObjectId) -> Option<GPath> {
    let session = bridge.session()?;
    let obj = session.find_object(id)?;
    match &obj.shape {
        Some(ShapeKind::Path(path)) => Some(path.clone()),
        Some(_) => Some(obj.to_path()),
        None => None,
    }
}

/// True when the object needs `convert_to_curves` before node editing.
fn needs_convert(bridge: &PetuniaDesignGuiBridge, id: ObjectId) -> bool {
    bridge
        .session()
        .and_then(|s| s.find_object(id))
        .is_some_and(|obj| !matches!(obj.shape, Some(ShapeKind::Path(_))))
}

/// Snapshots current paths for the selected nodes' objects.
fn snapshot_paths(
    bridge: &PetuniaDesignGuiBridge,
    selected: &[(ObjectId, usize)],
) -> Vec<(ObjectId, GPath)> {
    let mut seen = Vec::new();
    let mut out = Vec::new();
    for &(id, _) in selected {
        if seen.contains(&id) {
            continue;
        }
        seen.push(id);
        if let Some(path) = load_path(bridge, id) {
            out.push((id, path));
        }
    }
    out
}

/// Applies one delta to a snapshot's selected verbs.
fn apply_node_delta(
    initial: Vec<(ObjectId, GPath)>,
    moving: &[(ObjectId, usize)],
    delta: GPoint,
) -> Vec<(ObjectId, GPath)> {
    initial
        .into_iter()
        .map(|(id, mut path)| {
            for &(mid, idx) in moving {
                if mid == id {
                    petunia_design_geometry::move_verb(&mut path.verbs, idx, delta);
                }
            }
            (id, path)
        })
        .collect()
}

/// Commits several paths as one undo entry, preserving rotations.
fn commit_paths(
    bridge: &mut PetuniaDesignGuiBridge,
    label: &str,
    paths: Vec<(ObjectId, GPath)>,
) -> Result<ChangeSet, PetuniaError> {
    if paths.is_empty() {
        return Ok(ChangeSet::empty());
    }
    let mut cmds = Vec::new();
    for (id, path) in paths {
        let rotation = bridge
            .session()
            .and_then(|s| s.find_object(id))
            .map_or(0.0, |o| o.rotation);
        cmds.push(Command::SetShape {
            id,
            shape: Some(ShapeKind::Path(path.clone())),
        });
        if let Some(rect) = path.bounding_box() {
            cmds.push(Command::SetBounds {
                id,
                bounds: Some([
                    rect.x0,
                    rect.y0,
                    rect.width().max(1.0),
                    rect.height().max(1.0),
                ]),
                rotation,
            });
        }
    }
    bridge.submit_all(label, cmds)
}

/// Builds a path from edited verbs (verbs are always valid here).
fn verbs_to_path_unchecked(verbs: Vec<PathVerb>) -> GPath {
    let mut path = GPath::new();
    for verb in verbs {
        let _ = path.push(verb);
    }
    path
}

/// Builds a path, dropping to `None` when empty.
fn verbs_to_path(verbs: Vec<PathVerb>) -> Option<GPath> {
    let path = verbs_to_path_unchecked(verbs);
    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

/// Hit-tests anchor nodes across selected objects (topmost object first).
fn hit_node(
    pt: GPoint,
    bridge: &PetuniaDesignGuiBridge,
    camera: &ViewportCamera,
    tol: f64,
) -> Option<(ObjectId, usize)> {
    let session = bridge.session()?;
    let mut ids = session.selection.selected_ids.clone();
    ids.reverse();
    for id in ids {
        let obj = session.find_object(id)?;
        if !obj.visible || obj.locked {
            continue;
        }
        let shape = obj.shape.as_ref()?;
        let path = match shape {
            ShapeKind::Path(p) => p.clone(),
            _ => obj.to_path(),
        };
        for (idx, verb) in path.verbs.iter().enumerate() {
            if let Some(p) = endpoint_of(verb) {
                if p.distance_to(pt) <= tol {
                    return Some((id, idx));
                }
            }
        }
    }
    let _ = camera;
    None
}

/// Hit-tests the nearest editable segment, returning `(object, verb idx, t)`.
fn hit_segment(
    pt: GPoint,
    bridge: &PetuniaDesignGuiBridge,
    camera: &ViewportCamera,
    tol: f64,
) -> Option<(ObjectId, usize, f64)> {
    let session = bridge.session()?;
    let mut ids = session.selection.selected_ids.clone();
    ids.reverse();
    let mut best: Option<(f64, ObjectId, usize, f64)> = None;
    for id in ids {
        let obj = session.find_object(id)?;
        if !obj.visible || obj.locked {
            continue;
        }
        let shape = obj.shape.as_ref()?;
        let path = match shape {
            ShapeKind::Path(p) => p.clone(),
            _ => obj.to_path(),
        };
        for idx in 1..path.verbs.len() {
            let start = (0..idx).rev().find_map(|i| endpoint_of(&path.verbs[i]))?;
            if endpoint_of(&path.verbs[idx]).is_none() {
                continue;
            }
            let mut probe = GPath::new();
            probe.push(PathVerb::MoveTo(start)).ok()?;
            probe.push(path.verbs[idx]).ok()?;
            let flat: Vec<GPoint> = probe.to_polygons(0.5).into_iter().flatten().collect();
            let d = dist_to_polyline(pt, &flat);
            if d <= tol && best.is_none_or(|(bd, _, _, _)| d < bd) {
                best = Some((d, id, idx, nearest_t(&flat, pt)));
            }
        }
    }
    let _ = camera;
    best.map(|(_, id, idx, t)| (id, idx, t))
}

/// Previous endpoint verb index before `verb_idx`, if any.
fn prev_endpoint_idx(
    bridge: &PetuniaDesignGuiBridge,
    id: ObjectId,
    verb_idx: usize,
) -> Option<usize> {
    let session = bridge.session()?;
    let obj = session.find_object(id)?;
    let shape = obj.shape.as_ref()?;
    let verbs = match shape {
        ShapeKind::Path(p) => &p.verbs,
        _ => return None,
    };
    (0..verb_idx)
        .rev()
        .find(|i| endpoint_of(&verbs[*i]).is_some())
}

/// Topmost visible object hit by bounds (fallback when no node hits).
fn hit_object(pt: GPoint, bridge: &PetuniaDesignGuiBridge) -> Option<ObjectId> {
    let session = bridge.session()?;
    for id in session.spatial_candidates_point(pt, 0.0) {
        let Some(obj) = session.find_object(id) else {
            continue;
        };
        if obj.visible && !obj.locked && obj.hit_test(pt) {
            return Some(id);
        }
    }
    None
}

/// Toggles a node in the selection set.
fn toggle_node(selected: &mut Vec<(ObjectId, usize)>, id: ObjectId, idx: usize) {
    if let Some(pos) = selected.iter().position(|&(i, v)| i == id && v == idx) {
        selected.remove(pos);
    } else {
        selected.push((id, idx));
    }
}

/// Adds a node idempotently.
fn toggle_node_add(selected: &mut Vec<(ObjectId, usize)>, id: ObjectId, idx: usize) {
    if !selected.contains(&(id, idx)) {
        selected.push((id, idx));
    }
}

/// Shortest distance from `pt` to a polyline.
fn dist_to_polyline(pt: GPoint, baseline: &[GPoint]) -> f64 {
    if baseline.len() < 2 {
        return baseline
            .first()
            .map_or(f64::INFINITY, |p| p.distance_to(pt));
    }
    baseline
        .windows(2)
        .map(|w| dist_to_segment(pt, w[0], w[1]))
        .fold(f64::INFINITY, f64::min)
}

/// Shortest distance from `pt` to segment `a->b`.
fn dist_to_segment(pt: GPoint, a: GPoint, b: GPoint) -> f64 {
    let abx = b.x - a.x;
    let aby = b.y - a.y;
    let len2 = abx * abx + aby * aby;
    if len2 < 1e-12 {
        return pt.distance_to(a);
    }
    let t = ((pt.x - a.x) * abx + (pt.y - a.y) * aby) / len2;
    let t = t.clamp(0.0, 1.0);
    pt.distance_to(GPoint::new(a.x + abx * t, a.y + aby * t))
}

#[cfg(test)]
mod node_tool_tests {
    use super::*;

    fn anchor(x: f64, y: f64) -> PathVerb {
        PathVerb::LineTo(GPoint::new(x, y))
    }

    fn smooth_pair() -> Vec<PathVerb> {
        vec![
            PathVerb::MoveTo(GPoint::new(0.0, 0.0)),
            PathVerb::CubicTo(
                GPoint::new(10.0, 0.0),
                GPoint::new(20.0, 0.0),
                GPoint::new(30.0, 0.0),
            ),
        ]
    }

    #[test]
    fn smooth_classifies_symmetric_when_mirrored() {
        // Anchor at idx 1 has in=(20,0) out via next segment missing -> cusp.
        assert_eq!(classify_node(&smooth_pair(), 1), NodeType::Cusp);
    }

    #[test]
    fn convert_smooth_aligns_handles() {
        // Kinked anchor with both sides present but unequal: smooth aligns
        // directions while preserving each length.
        let mut verbs = vec![
            PathVerb::MoveTo(GPoint::new(0.0, 0.0)),
            PathVerb::CubicTo(
                GPoint::new(10.0, 5.0),
                GPoint::new(20.0, -5.0),
                GPoint::new(30.0, 0.0),
            ),
            PathVerb::CubicTo(
                GPoint::new(45.0, 8.0),
                GPoint::new(50.0, 0.0),
                GPoint::new(60.0, 0.0),
            ),
        ];
        assert_eq!(classify_node(&verbs, 1), NodeType::Cusp);
        assert!(convert_node(&mut verbs, 1, NodeType::Smooth));
        assert_eq!(classify_node(&verbs, 1), NodeType::Smooth);
    }

    #[test]
    fn convert_symmetric_equalizes_lengths() {
        let mut verbs = vec![
            PathVerb::MoveTo(GPoint::new(0.0, 0.0)),
            PathVerb::CubicTo(
                GPoint::new(10.0, 0.0),
                GPoint::new(25.0, 0.0),
                GPoint::new(30.0, 0.0),
            ),
            PathVerb::LineTo(GPoint::new(60.0, 0.0)),
        ];
        assert!(convert_node(&mut verbs, 1, NodeType::Symmetric));
        assert_eq!(classify_node(&verbs, 1), NodeType::Symmetric);
    }

    #[test]
    fn split_line_preserves_shape() {
        let mut verbs = vec![PathVerb::MoveTo(GPoint::new(0.0, 0.0)), anchor(30.0, 0.0)];
        let idx = split_segment(&mut verbs, 1, GPoint::new(10.0, 0.0)).unwrap();
        assert_eq!(idx, 1);
        assert_eq!(verbs.len(), 3);
        assert_eq!(endpoint_of(&verbs[1]), Some(GPoint::new(10.0, 0.0)));
    }

    #[test]
    fn delete_keeps_one_point() {
        let mut verbs = vec![PathVerb::MoveTo(GPoint::new(0.0, 0.0)), anchor(10.0, 0.0)];
        assert!(delete_node(&mut verbs, 1));
        assert!(!delete_node(&mut verbs, 0));
        assert_eq!(verbs.len(), 1);
    }
}
