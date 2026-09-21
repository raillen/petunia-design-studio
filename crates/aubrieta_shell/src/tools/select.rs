//! Selection and transform tool state machine (10.1).

use aubrieta_application::{Command, CommandRequest};
use aubrieta_document::ChangeSet;
use aubrieta_foundation::{AubrietaError, ObjectId};
use aubrieta_geometry::{GPoint, GRect};

use crate::bridge::*;
use crate::canvas::{
    compute_selection_handles, hit_test_handle_or_border, CanvasOverlays, SelectionHandleKind,
    SnapEngine, ViewportCamera,
};

use aubrieta_application::interaction::{NormalizedPointerEvent, PointerButton, PointerPhase};

/// Internal state machine for the selection tool.
#[derive(Clone, Debug, PartialEq)]
pub enum SelectToolState {
    /// Idle, waiting for pointer input.
    Idle,
    /// Dragging a rectangular marquee over empty space.
    Marquee {
        start_screen: GPoint,
        current_screen: GPoint,
    },
    /// Translating one or more selected objects across document space.
    DraggingObjects {
        start_doc: GPoint,
        current_doc: GPoint,
        initial_positions: Vec<(ObjectId, [f64; 4], f64)>,
        is_duplicate: bool,
    },
    /// Interactively resizing via bounding box handle.
    TransformingHandle {
        handle: SelectionHandleKind,
        start_doc: GPoint,
        current_doc: GPoint,
        initial_bounds: [f64; 4],
    },
}

/// Primary tool for selecting, arranging, moving, and transforming objects (10.1).
#[derive(Clone, Debug)]
pub struct SelectTool {
    state: SelectToolState,
    handle_size_px: f64,
}

impl Default for SelectTool {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectTool {
    /// Creates a fresh select tool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: SelectToolState::Idle,
            handle_size_px: 14.0,
        }
    }

    /// Cancels any active gesture and resets to idle.
    pub fn cancel(&mut self) {
        self.state = SelectToolState::Idle;
    }

    /// Handles a normalized pointer event.
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut AubrietaGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, AubrietaError> {
        match event.phase {
            PointerPhase::Down => self.on_down(event, bridge, camera, snap),
            PointerPhase::Move => self.on_move(event, bridge, camera, snap),
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
        bridge: &mut AubrietaGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, AubrietaError> {
        if event.button != PointerButton::Primary {
            return Ok(ChangeSet::empty());
        }

        snap.reset_hysteresis();

        let sel_vm = bridge.selection();

        // 1. Check if clicking on any transform handle or bounding box border of active selection
        if let Some([bx, by, bw, bh]) = sel_vm.combined_bounds {
            let doc_box = GRect::new(bx, by, bx + bw, by + bh);
            if let Some(handle_kind) = hit_test_handle_or_border(
                doc_box,
                event.screen_pos,
                event.doc_pos,
                camera,
                self.handle_size_px,
                8.0,
            ) {
                self.state = SelectToolState::TransformingHandle {
                    handle: handle_kind,
                    start_doc: event.doc_pos,
                    current_doc: event.doc_pos,
                    initial_bounds: [bx, by, bw, bh],
                };
                return Ok(ChangeSet::empty());
            }
        }

        // 2. Hit-test document objects under cursor
        let hit_object_id = self.hit_test_objects(event.doc_pos, bridge, camera);

        if let Some(id) = hit_object_id {
            if event.modifiers.constrain {
                // Shift-click toggles selection presence
                bridge.toggle_selection(id);
            } else if !sel_vm.contains(id) {
                // Regular click on unselected object selects it exclusively
                bridge.set_selection(vec![id]);
            }

            // Capture initial positions for drag
            let mut initial_positions = Vec::new();
            if let Some(session) = bridge.session() {
                for &sel_id in &session.selection.selected_ids {
                    if let Some(obj) = session.find_object(sel_id) {
                        if !obj.locked {
                            let b = obj.bounds.unwrap_or([0.0, 0.0, 100.0, 100.0]);
                            initial_positions.push((sel_id, b, obj.rotation));
                        }
                    }
                }
            }

            self.state = SelectToolState::DraggingObjects {
                start_doc: event.doc_pos,
                current_doc: event.doc_pos,
                initial_positions,
                is_duplicate: event.modifiers.duplicate,
            };
        } else {
            // Clicked on empty canvas
            if !event.modifiers.constrain {
                bridge.clear_selection();
            }
            self.state = SelectToolState::Marquee {
                start_screen: event.screen_pos,
                current_screen: event.screen_pos,
            };
        }

        Ok(ChangeSet::empty())
    }

    fn on_move(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut AubrietaGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, AubrietaError> {
        match &mut self.state {
            SelectToolState::Marquee { current_screen, .. } => {
                *current_screen = event.screen_pos;
            }
            SelectToolState::DraggingObjects {
                start_doc,
                current_doc,
                initial_positions,
                ..
            } => {
                let mut target_pt = event.doc_pos;
                if !event.modifiers.disable_snap {
                    let snap_res = snap.snap_point(target_pt, camera, &[]);
                    target_pt = snap_res.point;
                }
                *current_doc = target_pt;

                let dx = target_pt.x - start_doc.x;
                let dy = target_pt.y - start_doc.y;
                for (id, [x, y, w, h], rot) in initial_positions {
                    let _ = bridge.set_bounds(*id, Some([*x + dx, *y + dy, *w, *h]), *rot);
                }
            }
            SelectToolState::TransformingHandle {
                handle,
                start_doc,
                current_doc,
                initial_bounds,
            } => {
                let mut target_pt = event.doc_pos;
                if !event.modifiers.disable_snap {
                    let snap_res = snap.snap_point(target_pt, camera, &[]);
                    target_pt = snap_res.point;
                }
                *current_doc = target_pt;

                let dx = target_pt.x - start_doc.x;
                let dy = target_pt.y - start_doc.y;
                let (nx, ny, nw, nh) = calculate_resized_bounds(*handle, *initial_bounds, dx, dy);
                let sel_ids = bridge.selection().selected_ids;
                for id in sel_ids {
                    let _ = bridge.set_bounds(id, Some([nx, ny, nw, nh]), 0.0);
                }
            }
            SelectToolState::Idle => {}
        }
        Ok(ChangeSet::empty())
    }

    fn on_up(
        &mut self,
        _event: &NormalizedPointerEvent,
        bridge: &mut AubrietaGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, AubrietaError> {
        snap.reset_hysteresis();

        let prev_state = std::mem::replace(&mut self.state, SelectToolState::Idle);

        match prev_state {
            SelectToolState::Marquee {
                start_screen,
                current_screen,
            } => {
                let marquee_rect = GRect::new(
                    start_screen.x,
                    start_screen.y,
                    current_screen.x,
                    current_screen.y,
                );
                // Only evaluate if dragged beyond minimal click threshold
                if marquee_rect.width() > 3.0 || marquee_rect.height() > 3.0 {
                    let doc_tl =
                        camera.screen_to_doc(GPoint::new(marquee_rect.x0, marquee_rect.y0));
                    let doc_br =
                        camera.screen_to_doc(GPoint::new(marquee_rect.x1, marquee_rect.y1));
                    let doc_marquee = GRect::new(doc_tl.x, doc_tl.y, doc_br.x, doc_br.y);

                    let mut matched = Vec::new();
                    if let Some(session) = bridge.session() {
                        if let Some(surface_id) = session.active_surface() {
                            if let Ok(surface) = session.surface(surface_id) {
                                for obj in &surface.objects {
                                    if obj.visible && !obj.locked {
                                        if let Some([ox, oy, ow, oh]) = obj.bounds {
                                            let obj_rect = GRect::new(ox, oy, ox + ow, oy + oh);
                                            if doc_marquee.intersection(obj_rect).is_some() {
                                                matched.push(obj.id);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !matched.is_empty() {
                        bridge.set_selection(matched);
                    }
                }
                Ok(ChangeSet::empty())
            }
            SelectToolState::DraggingObjects {
                start_doc,
                current_doc,
                initial_positions,
                is_duplicate,
            } => {
                let dx = current_doc.x - start_doc.x;
                let dy = current_doc.y - start_doc.y;

                if dx.abs() < f64::EPSILON && dy.abs() < f64::EPSILON {
                    return Ok(ChangeSet::empty());
                }

                let mut combined = ChangeSet::empty();

                if is_duplicate {
                    // Duplicate selected objects at the new offset (10.1 duplicate-drag)
                    let mut new_ids = Vec::new();
                    let active_surface = bridge.session().and_then(|s| s.active_surface());

                    if let Some(surface_id) = active_surface {
                        for (orig_id, [x, y, w, h], rot) in initial_positions {
                            let new_id = bridge.next_object_id()?;
                            let name = bridge
                                .session()
                                .and_then(|s| s.find_object(orig_id))
                                .map(|o| format!("{} Copy", o.name))
                                .unwrap_or_else(|| "Object Copy".to_string());

                            let create_cmd = CommandRequest::new(Command::CreateObject {
                                surface: surface_id,
                                id: new_id,
                                name,
                            });
                            let c1 = bridge.submit_command(create_cmd)?;
                            for c in c1.changes {
                                combined.push(c);
                            }

                            let bounds_cmd = CommandRequest::new(Command::SetBounds {
                                id: new_id,
                                bounds: Some([x + dx, y + dy, w, h]),
                                rotation: rot,
                            });
                            let c2 = bridge.submit_command(bounds_cmd)?;
                            for c in c2.changes {
                                combined.push(c);
                            }

                            new_ids.push(new_id);
                        }
                        bridge.set_selection(new_ids);
                    }
                } else {
                    // Normal translation: update bounds for each moved object
                    for (id, [x, y, w, h], rot) in initial_positions {
                        let cmd = CommandRequest::new(Command::SetBounds {
                            id,
                            bounds: Some([x + dx, y + dy, w, h]),
                            rotation: rot,
                        });
                        let c = bridge.submit_command(cmd)?;
                        for change in c.changes {
                            combined.push(change);
                        }
                    }
                }

                Ok(combined)
            }
            SelectToolState::TransformingHandle {
                handle,
                start_doc,
                current_doc,
                initial_bounds,
            } => {
                let dx = current_doc.x - start_doc.x;
                let dy = current_doc.y - start_doc.y;
                let (nx, ny, nw, nh) = calculate_resized_bounds(handle, initial_bounds, dx, dy);

                let mut combined = ChangeSet::empty();
                let sel_ids = bridge.selection().selected_ids;
                for id in sel_ids {
                    let cmd = CommandRequest::new(Command::SetBounds {
                        id,
                        bounds: Some([nx, ny, nw, nh]),
                        rotation: 0.0,
                    });
                    let c = bridge.submit_command(cmd)?;
                    for change in c.changes {
                        combined.push(change);
                    }
                }
                Ok(combined)
            }
            SelectToolState::Idle => Ok(ChangeSet::empty()),
        }
    }

    /// Spatial hit-testing for selecting objects.
    fn hit_test_objects(
        &self,
        doc_pos: GPoint,
        bridge: &AubrietaGuiBridge,
        camera: &ViewportCamera,
    ) -> Option<ObjectId> {
        let session = bridge.session()?;
        let surface_id = session.active_surface()?;
        let surface = session.surface(surface_id).ok()?;

        let tolerance = 4.0 / camera.zoom;

        // Search in reverse z-order (topmost first)
        for obj in surface.objects.iter().rev() {
            if obj.visible && !obj.locked {
                if let Some([x, y, w, h]) = obj.bounds {
                    let rect = GRect::new(
                        x - tolerance,
                        y - tolerance,
                        x + w + tolerance,
                        y + h + tolerance,
                    );
                    if rect.contains(doc_pos) {
                        return Some(obj.id);
                    }
                }
            }
        }
        None
    }

    /// Resolves visual overlay descriptors for the selection tool.
    #[must_use]
    pub fn overlays(&self, camera: &ViewportCamera, bridge: &AubrietaGuiBridge) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();

        match &self.state {
            SelectToolState::Marquee {
                start_screen,
                current_screen,
            } => {
                overlays.marquee_screen = Some(GRect::new(
                    start_screen.x,
                    start_screen.y,
                    current_screen.x,
                    current_screen.y,
                ));
            }
            _ => {
                let sel_vm = bridge.selection();
                if let Some([bx, by, bw, bh]) = sel_vm.combined_bounds {
                    overlays.handles = compute_selection_handles(
                        GRect::new(bx, by, bx + bw, by + bh),
                        camera,
                        self.handle_size_px,
                    );
                }
            }
        }

        overlays
    }
}

/// Calculates new bounding box coordinates when dragging a specific transform handle.
#[must_use]
pub fn calculate_resized_bounds(
    handle: SelectionHandleKind,
    initial: [f64; 4],
    dx: f64,
    dy: f64,
) -> (f64, f64, f64, f64) {
    let [ix, iy, iw, ih] = initial;
    let mut nx = ix;
    let mut ny = iy;
    let mut nw = iw;
    let mut nh = ih;

    match handle {
        SelectionHandleKind::TopLeft => {
            nx += dx;
            ny += dy;
            nw -= dx;
            nh -= dy;
        }
        SelectionHandleKind::Top => {
            ny += dy;
            nh -= dy;
        }
        SelectionHandleKind::TopRight => {
            ny += dy;
            nw += dx;
            nh -= dy;
        }
        SelectionHandleKind::Right => {
            nw += dx;
        }
        SelectionHandleKind::BottomRight => {
            nw += dx;
            nh += dy;
        }
        SelectionHandleKind::Bottom => {
            nh += dy;
        }
        SelectionHandleKind::BottomLeft => {
            nx += dx;
            nw -= dx;
            nh += dy;
        }
        SelectionHandleKind::Left => {
            nx += dx;
            nw -= dx;
        }
        SelectionHandleKind::Rotation => {}
    }

    if nw < 5.0 {
        nw = 5.0;
    }
    if nh < 5.0 {
        nh = 5.0;
    }

    (nx, ny, nw, nh)
}
