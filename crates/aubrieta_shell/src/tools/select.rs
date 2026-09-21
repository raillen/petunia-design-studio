//! Selection and transform tool state machine (10.1).

use aubrieta_application::Command;
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
        initial_objects: Vec<(ObjectId, [f64; 4], f64)>,
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
                let mut initial_objects = Vec::new();
                if let Some(session) = bridge.session() {
                    for &sel_id in &session.selection.selected_ids {
                        if let Some(obj) = session.find_object(sel_id) {
                            if !obj.locked {
                                let b = obj.bounds.unwrap_or([0.0, 0.0, 100.0, 100.0]);
                                initial_objects.push((sel_id, b, obj.rotation));
                            }
                        }
                    }
                }
                self.state = SelectToolState::TransformingHandle {
                    handle: handle_kind,
                    start_doc: event.doc_pos,
                    current_doc: event.doc_pos,
                    initial_bounds: [bx, by, bw, bh],
                    initial_objects,
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
        _bridge: &mut AubrietaGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, AubrietaError> {
        match &mut self.state {
            SelectToolState::Marquee { current_screen, .. } => {
                *current_screen = event.screen_pos;
            }
            SelectToolState::DraggingObjects { current_doc, .. } => {
                let mut target_pt = event.doc_pos;
                if !event.modifiers.disable_snap {
                    let snap_res = snap.snap_point(target_pt, camera, &[]);
                    target_pt = snap_res.point;
                }
                *current_doc = target_pt;
                // Preview-only: the commit happens once on pointer-up,
                // so dragging never floods undo (F-01).
            }
            SelectToolState::TransformingHandle { current_doc, .. } => {
                let mut target_pt = event.doc_pos;
                if !event.modifiers.disable_snap {
                    let snap_res = snap.snap_point(target_pt, camera, &[]);
                    target_pt = snap_res.point;
                }
                *current_doc = target_pt;
                // Preview-only (F-01): see on_up commit.
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
                    // Directional policy (10.1): left-to-right selects
                    // intersecting objects, right-to-left only fully
                    // contained ones. GRect normalizes flips, so compare
                    // the raw drag direction instead.
                    let require_contained = current_screen.x < start_screen.x;

                    let mut matched = Vec::new();
                    if let Some(session) = bridge.session() {
                        if let Some(surface_id) = session.active_surface() {
                            if let Ok(surface) = session.surface(surface_id) {
                                for obj in surface.objects() {
                                    if obj.visible && !obj.locked {
                                        if let Some([ox, oy, ow, oh]) = obj.bounds {
                                            let hit = if require_contained {
                                                ox >= doc_marquee.x0
                                                    && oy >= doc_marquee.y0
                                                    && ox + ow <= doc_marquee.x1
                                                    && oy + oh <= doc_marquee.y1
                                            } else {
                                                doc_marquee
                                                    .intersection(GRect::new(
                                                        ox, oy, ox + ow, oy + oh,
                                                    ))
                                                    .is_some()
                                            };
                                            if hit {
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

                // One commit for the whole gesture (F-01).
                if is_duplicate {
                    // Duplicate selected objects at the new offset, cloning
                    // full appearance (shape, fill, stroke, opacity, stack).
                    let active_surface = bridge.session().and_then(|s| s.active_surface());
                    if let Some(surface_id) = active_surface {
                        let mut cmds = Vec::new();
                        let mut new_ids = Vec::new();
                        for (orig_id, [x, y, w, h], rot) in initial_positions {
                            let new_id = bridge.next_object_id()?;
                            let source = bridge
                                .session()
                                .and_then(|s| s.find_object(orig_id))
                                .cloned();
                            let (name, shape, fill, stroke, stroke_width, opacity, appearance) =
                                match source {
                                    Some(o) => (
                                        format!("{} Copy", o.name),
                                        o.shape.clone(),
                                        o.fill.clone(),
                                        o.stroke.clone(),
                                        o.stroke_width,
                                        o.opacity,
                                        o.appearance.clone(),
                                    ),
                                    None => (
                                        "Object Copy".to_string(),
                                        None,
                                        None,
                                        None,
                                        1.0,
                                        1.0,
                                        None,
                                    ),
                                };
                            cmds.push(Command::CreateShapeObject {
                                surface: surface_id,
                                id: new_id,
                                name,
                                shape: shape
                                    .unwrap_or(aubrieta_document::ShapeKind::Rectangle {
                                        corner_radii: [0.0; 4],
                                    }),
                                bounds: Some([x + dx, y + dy, w, h]),
                                fill,
                                stroke,
                                stroke_width,
                            });
                            if appearance.is_some() {
                                cmds.push(Command::SetAppearance {
                                    id: new_id,
                                    appearance,
                                });
                            }
                            if (opacity - 1.0).abs() > f64::EPSILON {
                                cmds.push(Command::SetOpacity {
                                    id: new_id,
                                    opacity,
                                });
                            }
                            // Preserve the source rotation omitted from the
                            // shape-creation command.
                            if rot.abs() > f64::EPSILON {
                                cmds.push(Command::SetBounds {
                                    id: new_id,
                                    bounds: Some([x + dx, y + dy, w, h]),
                                    rotation: rot,
                                });
                            }
                            new_ids.push(new_id);
                        }
                        let changes = bridge.submit_all("Duplicate objects", cmds)?;
                        bridge.set_selection(new_ids);
                        return Ok(changes);
                    }
                    return Ok(ChangeSet::empty());
                }
                // Normal translation: update bounds for each moved object.
                let cmds = initial_positions
                    .into_iter()
                    .map(|(id, [x, y, w, h], rot)| Command::SetBounds {
                        id,
                        bounds: Some([x + dx, y + dy, w, h]),
                        rotation: rot,
                    })
                    .collect();
                bridge.submit_all("Move objects", cmds)
            }
            SelectToolState::TransformingHandle {
                handle,
                start_doc,
                current_doc,
                initial_bounds,
                initial_objects,
            } => {
                if handle == SelectionHandleKind::Rotation {
                    // Rotation drag: angle delta around the combined center,
                    // added to each object's own rotation (never zeroed).
                    let [bx, by, bw, bh] = initial_bounds;
                    let center =
                        aubrieta_geometry::GPoint::new(bx + bw / 2.0, by + bh / 2.0);
                    let delta = aubrieta_geometry::pivot_angle_delta(
                        start_doc,
                        current_doc,
                        center,
                    )
                    .unwrap_or(0.0);
                    let cmds = initial_objects
                        .into_iter()
                        .filter_map(|(id, bounds, rot)| {
                            let session = bridge.session()?;
                            let current = session.find_object(id)?;
                            if current.locked {
                                return None;
                            }
                            Some(Command::SetBounds {
                                id,
                                bounds: Some(bounds),
                                rotation: rot + delta,
                            })
                        })
                        .collect();
                    return bridge.submit_all("Rotate objects", cmds);
                }
                // Resize drag: map the combined-bounds transform onto each
                // object proportionally, preserving sizes and rotations.
                let dx = current_doc.x - start_doc.x;
                let dy = current_doc.y - start_doc.y;
                let (nx, ny, nw, nh) =
                    calculate_resized_bounds(handle, initial_bounds, dx, dy);
                let [ibx, iby, ibw, ibh] = initial_bounds;
                let sx = if ibw.abs() > f64::EPSILON {
                    nw / ibw
                } else {
                    1.0
                };
                let sy = if ibh.abs() > f64::EPSILON {
                    nh / ibh
                } else {
                    1.0
                };
                let cmds = initial_objects
                    .into_iter()
                    .filter_map(|(id, [x, y, w, h], rot)| {
                        let session = bridge.session()?;
                        let current = session.find_object(id)?;
                        if current.locked {
                            return None;
                        }
                        Some(Command::SetBounds {
                            id,
                            bounds: Some([
                                nx + (x - ibx) * sx,
                                ny + (y - iby) * sy,
                                (w * sx).max(1.0),
                                (h * sy).max(1.0),
                            ]),
                            rotation: rot,
                        })
                    })
                    .collect();
                bridge.submit_all("Transform objects", cmds)
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

        // Search in reverse z-order (topmost first): bbox pre-check
        // with tolerance, then exact shape hit-test (10.1).
        for obj in surface.objects().iter().rev() {
            if obj.visible && !obj.locked {
                if let Some([x, y, w, h]) = obj.bounds {
                    let rect = GRect::new(
                        x - tolerance,
                        y - tolerance,
                        x + w + tolerance,
                        y + h + tolerance,
                    );
                    if rect.contains(doc_pos) && obj.hit_test(doc_pos) {
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
/// Thin mapping over the shared geometry primitive (Table B); rotation and
/// node affordances are not resizable and pass bounds through unchanged.
#[must_use]
pub fn calculate_resized_bounds(
    handle: SelectionHandleKind,
    initial: [f64; 4],
    dx: f64,
    dy: f64,
) -> (f64, f64, f64, f64) {
    let mapped = match handle {
        SelectionHandleKind::TopLeft => Some(aubrieta_geometry::ResizeHandle::TopLeft),
        SelectionHandleKind::Top => Some(aubrieta_geometry::ResizeHandle::Top),
        SelectionHandleKind::TopRight => Some(aubrieta_geometry::ResizeHandle::TopRight),
        SelectionHandleKind::Right => Some(aubrieta_geometry::ResizeHandle::Right),
        SelectionHandleKind::BottomRight => {
            Some(aubrieta_geometry::ResizeHandle::BottomRight)
        }
        SelectionHandleKind::Bottom => Some(aubrieta_geometry::ResizeHandle::Bottom),
        SelectionHandleKind::BottomLeft => {
            Some(aubrieta_geometry::ResizeHandle::BottomLeft)
        }
        SelectionHandleKind::Left => Some(aubrieta_geometry::ResizeHandle::Left),
        SelectionHandleKind::Rotation => None,
    };
    match mapped {
        Some(h) => aubrieta_geometry::resize_rect_from_handle(h, initial, dx, dy),
        None => (initial[0], initial[1], initial[2], initial[3]),
    }
}
