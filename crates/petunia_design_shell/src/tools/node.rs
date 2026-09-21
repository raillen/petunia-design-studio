//! Node editing and direct path manipulation tool (10.2).

use petunia_design_document::{ChangeSet, ShapeKind};
use petunia_design_foundation::{PetuniaError, ObjectId};
use petunia_design_geometry::{GPoint, GRect, PathVerb};

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{
    CanvasOverlays, SelectionHandle, SelectionHandleKind, SnapEngine, ViewportCamera,
};

use petunia_design_application::interaction::{NormalizedPointerEvent, PointerButton, PointerPhase};

/// Direct selection and node editing tool (10.2).
#[derive(Clone, Debug, Default)]
pub struct NodeTool {
    active_object: Option<ObjectId>,
    active_node_verb_idx: Option<usize>,
    is_dragging: bool,
    drag_start: GPoint,
}

impl NodeTool {
    /// Creates a fresh Node tool.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets active node selection.
    pub fn cancel(&mut self) {
        self.active_node_verb_idx = None;
        self.is_dragging = false;
    }

    /// Handles normalized pointer events.
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
        _snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
        match event.phase {
            PointerPhase::Down => {
                if event.button != PointerButton::Primary {
                    return Ok(ChangeSet::empty());
                }

                let tol = 12.0 / camera.zoom.max(0.1);
                let sel_ids = bridge.selection().selected_ids;

                let mut node_hit = None;
                let mut needs_convert = None;

                if let Some(session) = bridge.session() {
                    for &id in &sel_ids {
                        if let Some(obj) = session.find_object(id) {
                            let path = match &obj.shape {
                                Some(ShapeKind::Path(p)) => Some(p.clone()),
                                Some(_) => {
                                    needs_convert = Some(id);
                                    Some(obj.to_path())
                                }
                                None => None,
                            };

                            if let Some(p) = path {
                                for (idx, verb) in p.verbs.iter().enumerate() {
                                    let pt = match verb {
                                        PathVerb::MoveTo(pt) | PathVerb::LineTo(pt) => *pt,
                                        PathVerb::QuadTo(_, pt) | PathVerb::CubicTo(_, _, pt) => {
                                            *pt
                                        }
                                        PathVerb::Close => continue,
                                    };
                                    let dist = ((pt.x - event.doc_pos.x).powi(2)
                                        + (pt.y - event.doc_pos.y).powi(2))
                                    .sqrt();
                                    if dist <= tol {
                                        node_hit = Some((id, idx));
                                        break;
                                    }
                                }
                                if node_hit.is_some() {
                                    break;
                                }
                            }
                        }
                    }
                }

                if let Some((id, idx)) = node_hit {
                    if needs_convert == Some(id) {
                        let _ = bridge.convert_to_curves(id);
                    }
                    self.active_object = Some(id);
                    self.active_node_verb_idx = Some(idx);
                    self.is_dragging = true;
                    self.drag_start = event.doc_pos;
                    return Ok(ChangeSet::empty());
                }

                // 2. If no node on selected object was hit, hit-test any object to select and convert
                let mut hit_obj_id = None;
                if let Some(session) = bridge.session() {
                    if let Some(surface_id) = session.active_surface() {
                        if let Ok(surface) = session.surface(surface_id) {
                            for obj in surface.objects().iter().rev() {
                                if let Some([x, y, w, h]) = obj.bounds {
                                    if event.doc_pos.x >= x
                                        && event.doc_pos.x <= x + w
                                        && event.doc_pos.y >= y
                                        && event.doc_pos.y <= y + h
                                    {
                                        hit_obj_id = Some(obj.id);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(id) = hit_obj_id {
                    bridge.set_selection(vec![id]);
                    let _ = bridge.convert_to_curves(id);
                    self.active_object = Some(id);
                    return Ok(ChangeSet::empty());
                }

                self.active_object = None;
                self.active_node_verb_idx = None;
                self.is_dragging = false;
                Ok(ChangeSet::empty())
            }
            PointerPhase::Move => {
                if self.is_dragging {
                    if let (Some(obj_id), Some(idx)) =
                        (self.active_object, self.active_node_verb_idx)
                    {
                        let mut updated_path = None;
                        if let Some(session) = bridge.session() {
                            if let Some(obj) = session.find_object(obj_id) {
                                if let Some(ShapeKind::Path(mut path)) = obj.shape.clone() {
                                    // Shared primitive moves endpoint and control
                                    // handles together, preserving tangents (Table B).
                                    if petunia_design_geometry::move_verb_to(
                                        &mut path.verbs,
                                        idx,
                                        event.doc_pos,
                                    ) {
                                        updated_path = Some(path);
                                    }
                                }
                            }
                        }
                        if let Some(path) = updated_path {
                            return bridge.set_shape(obj_id, Some(ShapeKind::Path(path)));
                        }
                    }
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Up => {
                self.is_dragging = false;
                Ok(ChangeSet::empty())
            }
            PointerPhase::Cancel => {
                self.cancel();
                Ok(ChangeSet::empty())
            }
        }
    }

    /// Resolves overlays for the Node tool (node handle points).
    #[must_use]
    pub fn overlays(&self, camera: &ViewportCamera, bridge: &PetuniaDesignGuiBridge) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
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
