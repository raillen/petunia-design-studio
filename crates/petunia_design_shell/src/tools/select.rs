//! Selection and transform tool state machine (10.1).
//!
//! Batch 1: click feedback (hover/pressed), rectangle + lasso gestures,
//! and a settings-ready marquee rule (overlap vs. fully contained).

use petunia_design_application::Command;
use petunia_design_document::ChangeSet;
use petunia_design_foundation::{ObjectId, PetuniaError};
use petunia_design_geometry::{GPoint, GRect};

use crate::bridge::*;
use crate::canvas::{
    compute_selection_handles, compute_selection_handles_oriented, hit_test_handle_or_border,
    hit_test_handle_or_border_oriented, CanvasOverlays, CursorAffordance, SelectionHandleKind,
    SnapEngine, TransformPreview, TransformPreviewObject, ViewportCamera,
};

use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase,
};

/// Click threshold in screen pixels below which a press is a click, not a drag.
const CLICK_THRESHOLD_PX: f64 = 3.0;
/// Minimum screen distance between consecutive lasso samples.
const LASSO_SAMPLE_PX: f64 = 3.0;
/// Minimum lasso path length (screen px) to count as a lasso, not a click.
const LASSO_MIN_LENGTH_PX: f64 = 10.0;

/// Which pointer gesture the Select tool uses on empty canvas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectGestureMode {
    /// Drag a rectangular marquee (default).
    #[default]
    Rectangle,
    /// Draw a freehand lasso polygon.
    Lasso,
}

/// Settings-ready marquee rule for the future settings menu.
///
/// `Intersect` selects on overlap ("ao sobrepor").
/// `Contained` selects only fully enclosed objects ("todo o objeto").
/// `Directional` keeps the legacy policy (left-to-right = overlap,
/// right-to-left = contained).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarqueeSelectRule {
    /// Overlap selects (Photoshop default, CorelDRAW default).
    Intersect,
    /// Only fully enclosed objects select.
    Contained,
    /// Direction decides (legacy 10.1 policy).
    #[default]
    Directional,
}

/// Internal state machine for the selection tool.
#[derive(Clone, Debug, PartialEq)]
pub enum SelectToolState {
    /// Idle, waiting for pointer input.
    Idle,
    /// Dragging a rectangular marquee over empty space.
    Marquee {
        start_screen: GPoint,
        current_screen: GPoint,
        additive: bool,
        subtractive: bool,
    },
    /// Drawing a freehand lasso polygon over empty space.
    Lasso {
        points_screen: Vec<GPoint>,
        points_doc: Vec<GPoint>,
        additive: bool,
        subtractive: bool,
    },
    /// Translating one or more selected objects across document space.
    DraggingObjects {
        start_doc: GPoint,
        current_doc: GPoint,
        initial_objects: Vec<(ObjectId, [f64; 4], f64)>,
        is_duplicate: bool,
        anchor_id: Option<ObjectId>,
        was_already_selected: bool,
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
    gesture_mode: SelectGestureMode,
    marquee_rule: MarqueeSelectRule,
    hovered_object: Option<ObjectId>,
    hovered_handle: Option<SelectionHandleKind>,
    pressed_object: Option<ObjectId>,
    preview: Option<TransformPreview>,
    gesture_revision: Option<u64>,
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
            gesture_mode: SelectGestureMode::Rectangle,
            marquee_rule: MarqueeSelectRule::Directional,
            hovered_object: None,
            hovered_handle: None,
            pressed_object: None,
            preview: None,
            gesture_revision: None,
        }
    }

    /// Cancels any active gesture and resets to idle.
    pub fn cancel(&mut self) {
        self.state = SelectToolState::Idle;
        self.hovered_handle = None;
        self.pressed_object = None;
        self.preview = None;
        self.gesture_revision = None;
    }

    /// Which empty-canvas gesture is active (rectangle marquee or lasso).
    #[must_use]
    pub fn gesture_mode(&self) -> SelectGestureMode {
        self.gesture_mode
    }

    /// Switches between rectangle marquee and lasso, canceling any gesture.
    pub fn set_gesture_mode(&mut self, mode: SelectGestureMode) {
        if self.gesture_mode != mode {
            self.cancel();
            self.gesture_mode = mode;
        }
    }

    /// Current marquee rule (bind this to the settings menu).
    #[must_use]
    pub fn marquee_rule(&self) -> MarqueeSelectRule {
        self.marquee_rule
    }

    /// Sets the marquee rule: overlap, fully contained, or directional.
    pub fn set_marquee_rule(&mut self, rule: MarqueeSelectRule) {
        self.marquee_rule = rule;
    }

    /// Object currently hovered (visual click feedback).
    #[must_use]
    pub fn hovered_object(&self) -> Option<ObjectId> {
        self.hovered_object
    }

    /// Object pressed on pointer-down (visual click feedback).
    #[must_use]
    pub fn pressed_object(&self) -> Option<ObjectId> {
        self.pressed_object
    }

    /// Current tool state (for tests and shell introspection).
    #[must_use]
    pub fn tool_state(&self) -> &SelectToolState {
        &self.state
    }

    /// Current uncommitted transform preview.
    #[must_use]
    pub fn transform_preview(&self) -> Option<&TransformPreview> {
        self.preview.as_ref()
    }

    fn refresh_transform_preview(&mut self, bridge: &PetuniaDesignGuiBridge) {
        let Some(session) = bridge.session() else {
            self.preview = None;
            return;
        };
        let Some(base_revision) = self.gesture_revision else {
            self.preview = None;
            return;
        };
        if session.current_revision() != base_revision {
            self.preview = None;
            return;
        }
        let preview = match &self.state {
            SelectToolState::DraggingObjects {
                start_doc,
                current_doc,
                initial_objects,
                is_duplicate,
                ..
            } => {
                let dx = current_doc.x - start_doc.x;
                let dy = current_doc.y - start_doc.y;
                if *is_duplicate || dx.abs() <= f64::EPSILON && dy.abs() <= f64::EPSILON {
                    None
                } else {
                    let objects = initial_objects
                        .iter()
                        .map(|(id, [x, y, w, h], rotation)| TransformPreviewObject {
                            id: *id,
                            bounds: [x + dx, y + dy, *w, *h],
                            rotation: *rotation,
                        })
                        .collect::<Vec<_>>();
                    let frame = frame_from_preview_objects(&objects);
                    Some((objects, frame))
                }
            }
            SelectToolState::TransformingHandle {
                handle,
                start_doc,
                current_doc,
                initial_bounds,
                initial_objects,
            } => {
                let objects = if *handle == SelectionHandleKind::Rotation {
                    let [bx, by, bw, bh] = *initial_bounds;
                    let center = if initial_objects.len() == 1
                        && initial_objects[0].2.abs() > f64::EPSILON
                    {
                        let rot_trans = petunia_design_geometry::GAffine::translate(bx, by).after(
                            petunia_design_geometry::GAffine::rotate(initial_objects[0].2),
                        );
                        rot_trans.apply(GPoint::new(bw / 2.0, bh / 2.0))
                    } else {
                        GPoint::new(bx + bw / 2.0, by + bh / 2.0)
                    };
                    let delta = petunia_design_geometry::pivot_angle_delta(
                        *start_doc,
                        *current_doc,
                        center,
                    )
                    .unwrap_or(0.0);
                    rotate_preview_objects(initial_objects, center, delta)
                } else if initial_objects.len() == 1 && initial_objects[0].2.abs() > f64::EPSILON {
                    let theta = initial_objects[0].2;
                    let (id, [bx, by, bw, bh], _) = initial_objects[0];
                    let dx_doc = current_doc.x - start_doc.x;
                    let dy_doc = current_doc.y - start_doc.y;
                    let cos_t = (-theta).cos();
                    let sin_t = (-theta).sin();
                    let dx_local = dx_doc * cos_t - dy_doc * sin_t;
                    let dy_local = dx_doc * sin_t + dy_doc * cos_t;
                    let (nx_local, ny_local, nw, nh) =
                        calculate_resized_bounds(*handle, [0.0, 0.0, bw, bh], dx_local, dy_local);
                    let rot_trans = petunia_design_geometry::GAffine::translate(bx, by)
                        .after(petunia_design_geometry::GAffine::rotate(theta));
                    let new_origin = rot_trans.apply(GPoint::new(nx_local, ny_local));
                    vec![TransformPreviewObject {
                        id,
                        bounds: [new_origin.x, new_origin.y, nw, nh],
                        rotation: theta,
                    }]
                } else {
                    let dx = current_doc.x - start_doc.x;
                    let dy = current_doc.y - start_doc.y;
                    let (nx, ny, nw, nh) =
                        calculate_resized_bounds(*handle, *initial_bounds, dx, dy);
                    let [ibx, iby, ibw, ibh] = *initial_bounds;
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
                    initial_objects
                        .iter()
                        .map(|(id, [x, y, w, h], rotation)| TransformPreviewObject {
                            id: *id,
                            bounds: [
                                nx + (x - ibx) * sx,
                                ny + (y - iby) * sy,
                                (w * sx).max(1.0),
                                (h * sy).max(1.0),
                            ],
                            rotation: *rotation,
                        })
                        .collect::<Vec<_>>()
                };
                let frame = frame_from_preview_objects(&objects);
                Some((objects, frame))
            }
            SelectToolState::Idle
            | SelectToolState::Marquee { .. }
            | SelectToolState::Lasso { .. } => None,
        };
        self.preview = preview.map(|(objects, frame)| TransformPreview {
            base_revision,
            frame,
            objects,
        });
    }

    /// Handles a normalized pointer event.
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
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
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
        if event.button != PointerButton::Primary {
            return Ok(ChangeSet::empty());
        }

        snap.reset_hysteresis();
        self.gesture_revision = bridge.session().map(|session| session.current_revision());
        self.preview = None;

        let sel_vm = bridge.selection();

        // 1. Check if clicking on any transform handle or bounding box border of active selection
        let handle_hit = if let (1, Some(bounds), Some(transform)) = (
            sel_vm.count,
            sel_vm.primary_bounds,
            sel_vm.primary_transform,
        ) {
            hit_test_handle_or_border_oriented(
                bounds,
                transform,
                event.screen_pos,
                event.doc_pos,
                camera,
                self.handle_size_px,
                8.0,
            )
            .map(|h| (h, bounds))
        } else if let Some([bx, by, bw, bh]) = sel_vm.combined_bounds {
            let doc_box = GRect::new(bx, by, bx + bw, by + bh);
            hit_test_handle_or_border(
                doc_box,
                event.screen_pos,
                event.doc_pos,
                camera,
                self.handle_size_px,
                8.0,
            )
            .map(|h| (h, [bx, by, bw, bh]))
        } else {
            None
        };

        if let Some((handle_kind, initial_bounds)) = handle_hit {
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
            self.pressed_object = None;
            self.state = SelectToolState::TransformingHandle {
                handle: handle_kind,
                start_doc: event.doc_pos,
                current_doc: event.doc_pos,
                initial_bounds,
                initial_objects,
            };
            self.refresh_transform_preview(bridge);
            return Ok(ChangeSet::empty());
        }

        // 2. Hit-test document objects under cursor
        let hit_object_id = self.hit_test_objects(event.doc_pos, bridge, camera);

        if let Some(id) = hit_object_id {
            // Click feedback: hover + pressed track the same object.
            self.hovered_object = Some(id);
            self.pressed_object = Some(id);
            let was_already_selected = sel_vm.contains(id);
            if event.modifiers.constrain {
                // Shift-click toggles selection presence
                bridge.toggle_selection(id);
            } else if !was_already_selected {
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
                initial_objects: initial_positions,
                is_duplicate: event.modifiers.duplicate,
                anchor_id: Some(id),
                was_already_selected,
            };
            self.refresh_transform_preview(bridge);
        } else {
            // Clicked on empty canvas: start a selection gesture.
            self.pressed_object = None;
            self.hovered_object = None;
            let additive = event.modifiers.constrain;
            let subtractive = event.modifiers.duplicate && !event.modifiers.constrain;
            if !additive && !subtractive {
                bridge.clear_selection();
            }
            match self.gesture_mode {
                SelectGestureMode::Rectangle => {
                    self.state = SelectToolState::Marquee {
                        start_screen: event.screen_pos,
                        current_screen: event.screen_pos,
                        additive,
                        subtractive,
                    };
                }
                SelectGestureMode::Lasso => {
                    self.state = SelectToolState::Lasso {
                        points_screen: vec![event.screen_pos],
                        points_doc: vec![event.doc_pos],
                        additive,
                        subtractive,
                    };
                }
            }
        }

        Ok(ChangeSet::empty())
    }

    fn on_move(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
        match &mut self.state {
            SelectToolState::Idle => {
                // Hover feedback without touching the document.
                let sel_vm = bridge.selection();
                let handle_hit = if let (1, Some(bounds), Some(transform)) = (
                    sel_vm.count,
                    sel_vm.primary_bounds,
                    sel_vm.primary_transform,
                ) {
                    hit_test_handle_or_border_oriented(
                        bounds,
                        transform,
                        event.screen_pos,
                        event.doc_pos,
                        camera,
                        self.handle_size_px,
                        8.0,
                    )
                } else if let Some([bx, by, bw, bh]) = sel_vm.combined_bounds {
                    hit_test_handle_or_border(
                        GRect::new(bx, by, bx + bw, by + bh),
                        event.screen_pos,
                        event.doc_pos,
                        camera,
                        self.handle_size_px,
                        8.0,
                    )
                } else {
                    None
                };
                self.hovered_handle = handle_hit;
                if handle_hit.is_none() {
                    self.hovered_object = self.hit_test_objects(event.doc_pos, bridge, camera);
                } else {
                    self.hovered_object = None;
                }
            }
            SelectToolState::Marquee { current_screen, .. } => {
                *current_screen = event.screen_pos;
            }
            SelectToolState::Lasso {
                points_screen,
                points_doc,
                ..
            } => {
                if let Some(last) = points_screen.last() {
                    if last.distance_to(event.screen_pos) >= LASSO_SAMPLE_PX {
                        points_screen.push(event.screen_pos);
                        points_doc.push(event.doc_pos);
                    }
                }
            }
            SelectToolState::DraggingObjects { current_doc, .. } => {
                let mut target_pt = event.doc_pos;
                if !event.modifiers.disable_snap {
                    let snap_res = snap.snap_point(target_pt, camera, &[]);
                    target_pt = snap_res.point;
                }
                *current_doc = target_pt;
                self.refresh_transform_preview(bridge);
            }
            SelectToolState::TransformingHandle { current_doc, .. } => {
                let mut target_pt = event.doc_pos;
                if !event.modifiers.disable_snap {
                    let snap_res = snap.snap_point(target_pt, camera, &[]);
                    target_pt = snap_res.point;
                }
                *current_doc = target_pt;
                self.refresh_transform_preview(bridge);
            }
        }
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

        let base_revision = self.gesture_revision.take();
        self.preview = None;
        let revision_changed = base_revision.is_some_and(|base| {
            bridge
                .session()
                .is_none_or(|session| session.current_revision() != base)
        });
        let prev_state = std::mem::replace(&mut self.state, SelectToolState::Idle);
        // Click feedback always releases on pointer-up.
        self.pressed_object = None;

        if revision_changed {
            self.hovered_object = None;
            return Ok(ChangeSet::empty());
        }

        match prev_state {
            SelectToolState::Idle => {
                self.hovered_object = self.hit_test_objects(event.doc_pos, bridge, camera);
                Ok(ChangeSet::empty())
            }
            SelectToolState::Marquee {
                start_screen,
                current_screen,
                additive,
                subtractive,
            } => {
                let marquee_rect = GRect::new(
                    start_screen.x,
                    start_screen.y,
                    current_screen.x,
                    current_screen.y,
                );
                // Only evaluate if dragged beyond minimal click threshold
                if marquee_rect.width() <= CLICK_THRESHOLD_PX
                    && marquee_rect.height() <= CLICK_THRESHOLD_PX
                {
                    return Ok(ChangeSet::empty());
                }
                let doc_tl = camera.screen_to_doc(GPoint::new(marquee_rect.x0, marquee_rect.y0));
                let doc_br = camera.screen_to_doc(GPoint::new(marquee_rect.x1, marquee_rect.y1));
                let doc_marquee = GRect::new(doc_tl.x, doc_tl.y, doc_br.x, doc_br.y);
                let require_contained =
                    self.resolve_require_contained(start_screen, current_screen);
                let matched = self.match_rect(doc_marquee, require_contained, bridge);
                self.apply_matched(matched, additive, subtractive, bridge);
                Ok(ChangeSet::empty())
            }
            SelectToolState::Lasso {
                points_screen,
                points_doc,
                additive,
                subtractive,
            } => {
                if points_doc.len() < 3 || lasso_path_length(&points_screen) < LASSO_MIN_LENGTH_PX {
                    return Ok(ChangeSet::empty());
                }
                let require_contained = self.resolve_require_contained(
                    points_screen[0],
                    points_screen[points_screen.len() - 1],
                );
                let matched = self.match_lasso(&points_doc, require_contained, bridge);
                self.apply_matched(matched, additive, subtractive, bridge);
                Ok(ChangeSet::empty())
            }
            SelectToolState::DraggingObjects {
                start_doc,
                current_doc,
                initial_objects,
                is_duplicate,
                anchor_id,
                was_already_selected,
            } => {
                let dx = current_doc.x - start_doc.x;
                let dy = current_doc.y - start_doc.y;
                let drag_screen_dist = camera.zoom * (dx * dx + dy * dy).sqrt();

                if drag_screen_dist <= CLICK_THRESHOLD_PX {
                    // Click without drag on an already-selected object
                    // collapses a multi-selection onto the clicked object
                    // (Photoshop / Affinity / CorelDRAW behavior).
                    if was_already_selected
                        && !event.modifiers.constrain
                        && !event.modifiers.duplicate
                    {
                        if let Some(anchor) = anchor_id {
                            bridge.set_selection(vec![anchor]);
                            self.hovered_object = Some(anchor);
                        }
                    }
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
                        for (orig_id, [x, y, w, h], rot) in initial_objects {
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
                                shape: shape.unwrap_or(
                                    petunia_design_document::ShapeKind::Rectangle {
                                        corner_radii: [0.0; 4],
                                    },
                                ),
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
                        bridge.set_selection(new_ids.clone());
                        self.hovered_object = new_ids.last().copied();
                        return Ok(changes);
                    }
                    return Ok(ChangeSet::empty());
                }
                // Normal translation: update bounds for each moved object.
                let cmds = initial_objects
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
                    let center = if initial_objects.len() == 1
                        && initial_objects[0].2.abs() > f64::EPSILON
                    {
                        let rot_trans = petunia_design_geometry::GAffine::translate(bx, by).after(
                            petunia_design_geometry::GAffine::rotate(initial_objects[0].2),
                        );
                        rot_trans.apply(petunia_design_geometry::GPoint::new(bw / 2.0, bh / 2.0))
                    } else {
                        petunia_design_geometry::GPoint::new(bx + bw / 2.0, by + bh / 2.0)
                    };
                    let delta =
                        petunia_design_geometry::pivot_angle_delta(start_doc, current_doc, center)
                            .unwrap_or(0.0);
                    let preview_objects = rotate_preview_objects(&initial_objects, center, delta);
                    let cmds = preview_objects
                        .into_iter()
                        .filter_map(|preview| {
                            let session = bridge.session()?;
                            let current = session.find_object(preview.id)?;
                            if current.locked {
                                return None;
                            }
                            Some(Command::SetBounds {
                                id: preview.id,
                                bounds: Some(preview.bounds),
                                rotation: preview.rotation,
                            })
                        })
                        .collect();
                    return bridge.submit_all("Rotate objects", cmds);
                }
                if initial_objects.len() == 1 && initial_objects[0].2.abs() > f64::EPSILON {
                    // Single rotated object resize along its local axes
                    let theta = initial_objects[0].2;
                    let (id, [bx, by, bw, bh], _) = initial_objects[0];
                    let dx_doc = current_doc.x - start_doc.x;
                    let dy_doc = current_doc.y - start_doc.y;
                    let cos_t = (-theta).cos();
                    let sin_t = (-theta).sin();
                    let dx_local = dx_doc * cos_t - dy_doc * sin_t;
                    let dy_local = dx_doc * sin_t + dy_doc * cos_t;
                    let (nx_local, ny_local, nw, nh) =
                        calculate_resized_bounds(handle, [0.0, 0.0, bw, bh], dx_local, dy_local);
                    let rot_trans = petunia_design_geometry::GAffine::translate(bx, by)
                        .after(petunia_design_geometry::GAffine::rotate(theta));
                    let new_origin =
                        rot_trans.apply(petunia_design_geometry::GPoint::new(nx_local, ny_local));
                    let cmd = Command::SetBounds {
                        id,
                        bounds: Some([new_origin.x, new_origin.y, nw, nh]),
                        rotation: theta,
                    };
                    return bridge.submit_all("Resize object", vec![cmd]);
                }
                // Resize drag: map the combined-bounds transform onto each
                // object proportionally, preserving sizes and rotations.
                let dx = current_doc.x - start_doc.x;
                let dy = current_doc.y - start_doc.y;
                let (nx, ny, nw, nh) = calculate_resized_bounds(handle, initial_bounds, dx, dy);
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
        }
    }

    /// Resolves whether a marquee/lasso requires full containment.
    fn resolve_require_contained(&self, start: GPoint, current: GPoint) -> bool {
        match self.marquee_rule {
            MarqueeSelectRule::Intersect => false,
            MarqueeSelectRule::Contained => true,
            // Directional policy (10.1): left-to-right selects
            // intersecting objects, right-to-left only fully
            // contained ones.
            MarqueeSelectRule::Directional => current.x < start.x,
        }
    }

    /// Applies matched IDs honoring additive (Shift) / subtractive (Alt).
    fn apply_matched(
        &self,
        matched: Vec<ObjectId>,
        additive: bool,
        subtractive: bool,
        bridge: &mut PetuniaDesignGuiBridge,
    ) {
        if additive {
            let mut merged = bridge.selection().selected_ids;
            for id in matched {
                if !merged.contains(&id) {
                    merged.push(id);
                }
            }
            bridge.set_selection(merged);
        } else if subtractive {
            let remaining: Vec<ObjectId> = bridge
                .selection()
                .selected_ids
                .into_iter()
                .filter(|id| !matched.contains(id))
                .collect();
            bridge.set_selection(remaining);
        } else {
            bridge.set_selection(matched);
        }
    }

    /// Matches objects against a document-space marquee rect.
    fn match_rect(
        &self,
        doc_marquee: GRect,
        require_contained: bool,
        bridge: &PetuniaDesignGuiBridge,
    ) -> Vec<ObjectId> {
        let mut matched = Vec::new();
        if let Some(session) = bridge.session() {
            let rect = [
                doc_marquee.x0.min(doc_marquee.x1),
                doc_marquee.y0.min(doc_marquee.y1),
                doc_marquee.x0.max(doc_marquee.x1),
                doc_marquee.y0.max(doc_marquee.y1),
            ];
            for id in session.spatial_candidates_rect(rect) {
                let Some(obj) = session.find_object(id) else {
                    continue;
                };
                if obj.visible && !obj.locked {
                    if let Some([ox, oy, ow, oh]) = bridge
                        .cached_world_frame_bounds(id)
                        .or_else(|| bridge.cached_world_bounds(id))
                        .or(obj.bounds)
                    {
                        let hit = if require_contained {
                            ox >= doc_marquee.x0
                                && oy >= doc_marquee.y0
                                && ox + ow <= doc_marquee.x1
                                && oy + oh <= doc_marquee.y1
                        } else {
                            doc_marquee
                                .intersection(GRect::new(ox, oy, ox + ow, oy + oh))
                                .is_some()
                        };
                        if hit {
                            matched.push(obj.id);
                        }
                    }
                }
            }
        }
        matched
    }

    /// Matches objects against a freehand lasso polygon (document space).
    fn match_lasso(
        &self,
        polygon_doc: &[GPoint],
        require_contained: bool,
        bridge: &PetuniaDesignGuiBridge,
    ) -> Vec<ObjectId> {
        let mut matched = Vec::new();
        if let Some(session) = bridge.session() {
            if polygon_doc.is_empty() {
                return matched;
            }
            let mut min_x = polygon_doc[0].x;
            let mut max_x = polygon_doc[0].x;
            let mut min_y = polygon_doc[0].y;
            let mut max_y = polygon_doc[0].y;
            for pt in &polygon_doc[1..] {
                min_x = min_x.min(pt.x);
                max_x = max_x.max(pt.x);
                min_y = min_y.min(pt.y);
                max_y = max_y.max(pt.y);
            }
            for id in session.spatial_candidates_rect([min_x, min_y, max_x, max_y]) {
                let Some(obj) = session.find_object(id) else {
                    continue;
                };
                if obj.visible && !obj.locked {
                    if let Some([ox, oy, ow, oh]) = bridge
                        .cached_world_frame_bounds(id)
                        .or_else(|| bridge.cached_world_bounds(id))
                        .or(obj.bounds)
                    {
                        if lasso_hits_rect(polygon_doc, [ox, oy, ow, oh], require_contained) {
                            matched.push(obj.id);
                        }
                    }
                }
            }
        }
        matched
    }

    /// Spatial hit-testing for selecting objects (stroke-aware, P1).
    ///
    /// Fill/interior uses the existing world-aware memoized test
    /// (unchanged). Open paths have no interior, so a second pass checks
    /// outline proximity within `8/zoom` (knife parity) over the memoized
    /// `GeoCache` outline (F1+F2): world polygons when an explicit frame
    /// exists, legacy otherwise. Segment math is shared with the knife
    /// (`super::stroke_hit`). Objects without any vector contour
    /// (Text/Image) keep the frame-bbox fallback; every other miss is a
    /// miss even inside the bbox.
    fn hit_test_objects(
        &self,
        doc_pos: GPoint,
        bridge: &PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
    ) -> Option<ObjectId> {
        let session = bridge.session()?;
        let prox_tol = 8.0 / camera.zoom.max(0.1);
        let exact_tol = petunia_design_geometry::zoom_flatten_tol(camera.zoom);

        // Spatial prefilter (F3) over evaluated bounds, topmost-first, then
        // the exact test on the memoized outline (F1 + F2). Unlike the old
        // base-bounds pre-check, warped/inset outlines hit where drawn (09.31).
        let candidates = bridge.spatial_candidates_point(doc_pos, prox_tol);
        let ids: Vec<ObjectId> = if !candidates.is_empty() {
            candidates
        } else {
            session
                .document()
                .surfaces()
                .iter()
                .flat_map(|s| s.objects().iter().rev().map(|o| o.id))
                .collect()
        };

        for id in ids {
            let Some(obj) = session.find_object(id) else {
                continue;
            };
            if !obj.visible || obj.locked {
                continue;
            }

            // Fill / interior (existing world-aware behavior, unchanged).
            if bridge.cached_world_hit(id, doc_pos, exact_tol) {
                return Some(id);
            }
            if bridge.cached_world_bounds(id).is_none() && bridge.cached_hit(id, doc_pos, exact_tol)
            {
                return Some(id);
            }

            // Stroke proximity over the memoized outline. World-aware when
            // an explicit frame exists, legacy otherwise.
            let world_polys = session.cached_world_polygons(id, exact_tol);
            let legacy_polys = if world_polys.is_none() {
                session.cached_polygons(id, exact_tol)
            } else {
                None
            };
            let stroke_near = if let Some(ref polys) = world_polys {
                super::stroke_hit::contours_near_point(polys, doc_pos, prox_tol)
            } else if let Some(ref polys) = legacy_polys {
                super::stroke_hit::contours_near_point(polys, doc_pos, prox_tol)
            } else {
                false
            };
            if stroke_near {
                return Some(id);
            }

            // Empty-outline fallback (Text/Image without vector contour):
            // keep bbox selection where there is no contour to be near.
            let has_outline = world_polys
                .as_ref()
                .is_some_and(|polys| polys.iter().any(|contour| contour.len() >= 2))
                || legacy_polys
                    .as_ref()
                    .is_some_and(|polys| polys.iter().any(|contour| contour.len() >= 2));
            if !has_outline
                && session
                    .cached_world_frame_bounds(id)
                    .or_else(|| session.cached_bounds(id))
                    .or(obj.bounds)
                    .is_some_and(|[bx, by, bw, bh]| {
                        if obj.rotation.abs() <= 1e-4 {
                            let min_x = bx.min(bx + bw);
                            let max_x = bx.max(bx + bw);
                            let min_y = by.min(by + bh);
                            let max_y = by.max(by + bh);
                            doc_pos.x >= min_x - prox_tol
                                && doc_pos.x <= max_x + prox_tol
                                && doc_pos.y >= min_y - prox_tol
                                && doc_pos.y <= max_y + prox_tol
                        } else if let Ok(trans) = session.document().world_transform_checked(id) {
                            if let Some(inv) = trans.inverse() {
                                let local = inv.apply(doc_pos);
                                let [_, _, ow, oh] = obj.bounds.unwrap_or([0.0, 0.0, bw, bh]);
                                let min_x = 0.0f64.min(ow);
                                let max_x = 0.0f64.max(ow);
                                let min_y = 0.0f64.min(oh);
                                let max_y = 0.0f64.max(oh);
                                local.x >= min_x - prox_tol
                                    && local.x <= max_x + prox_tol
                                    && local.y >= min_y - prox_tol
                                    && local.y <= max_y + prox_tol
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    })
            {
                return Some(id);
            }
        }
        None
    }

    /// Resolves visual overlay descriptors for the selection tool.
    #[must_use]
    pub fn overlays(
        &self,
        camera: &ViewportCamera,
        bridge: &PetuniaDesignGuiBridge,
    ) -> CanvasOverlays {
        let transform_preview = self.preview.clone().filter(|preview| {
            bridge
                .session()
                .is_some_and(|session| session.current_revision() == preview.base_revision)
        });
        let mut overlays = CanvasOverlays {
            hovered_object: self.hovered_object,
            pressed_object: self.pressed_object,
            transform_preview,
            ..CanvasOverlays::default()
        };

        match &self.state {
            SelectToolState::Marquee {
                start_screen,
                current_screen,
                additive,
                subtractive,
            } => {
                overlays.marquee_screen = Some(GRect::new(
                    start_screen.x,
                    start_screen.y,
                    current_screen.x,
                    current_screen.y,
                ));
                overlays.marquee_additive = *additive;
                overlays.marquee_subtractive = *subtractive;
            }
            SelectToolState::Lasso {
                points_screen,
                additive,
                subtractive,
                ..
            } => {
                if points_screen.len() >= 2 {
                    overlays.lasso_screen = Some(points_screen.clone());
                }
                overlays.marquee_additive = *additive;
                overlays.marquee_subtractive = *subtractive;
            }
            _ => {
                let sel_vm = bridge.selection();
                if let (1, Some(bounds), Some(transform)) = (
                    sel_vm.count,
                    sel_vm.primary_bounds,
                    sel_vm.primary_transform,
                ) {
                    overlays.handles = compute_selection_handles_oriented(
                        bounds,
                        transform,
                        camera,
                        self.handle_size_px,
                    );
                } else if let Some([bx, by, bw, bh]) = sel_vm.combined_bounds {
                    overlays.handles = compute_selection_handles(
                        GRect::new(bx, by, bx + bw, by + bh),
                        camera,
                        self.handle_size_px,
                    );
                }
            }
        }

        overlays.cursor = match &self.state {
            SelectToolState::TransformingHandle { handle, .. } => map_handle_to_cursor(*handle),
            SelectToolState::DraggingObjects { .. } => CursorAffordance::Move,
            SelectToolState::Marquee { .. } | SelectToolState::Lasso { .. } => {
                CursorAffordance::Crosshair
            }
            SelectToolState::Idle => {
                if let Some(handle) = self.hovered_handle {
                    map_handle_to_cursor(handle)
                } else if let Some(hovered) = self.hovered_object {
                    let sel_vm = bridge.selection();
                    if sel_vm.contains(hovered) {
                        CursorAffordance::Move
                    } else {
                        CursorAffordance::Pointer
                    }
                } else {
                    CursorAffordance::Default
                }
            }
        };

        overlays
    }
}

fn map_handle_to_cursor(handle: SelectionHandleKind) -> CursorAffordance {
    match handle {
        SelectionHandleKind::TopLeft | SelectionHandleKind::BottomRight => {
            CursorAffordance::ResizeNwse
        }
        SelectionHandleKind::TopRight | SelectionHandleKind::BottomLeft => {
            CursorAffordance::ResizeNesw
        }
        SelectionHandleKind::Left | SelectionHandleKind::Right => CursorAffordance::ResizeCol,
        SelectionHandleKind::Top | SelectionHandleKind::Bottom => CursorAffordance::ResizeRow,
        SelectionHandleKind::Rotation => CursorAffordance::Rotate,
        SelectionHandleKind::NodeControl => CursorAffordance::Crosshair,
        SelectionHandleKind::NodeCusp
        | SelectionHandleKind::NodeCuspSelected
        | SelectionHandleKind::NodeSmooth
        | SelectionHandleKind::NodeSmoothSelected
        | SelectionHandleKind::NodeSymmetric
        | SelectionHandleKind::NodeSymmetricSelected => CursorAffordance::Pointer,
    }
}

fn rotate_preview_objects(
    objects: &[(ObjectId, [f64; 4], f64)],
    pivot: GPoint,
    delta: f64,
) -> Vec<TransformPreviewObject> {
    objects
        .iter()
        .map(|(id, [x, y, w, h], rotation)| {
            let origin =
                petunia_design_geometry::rotate_point_around(GPoint::new(*x, *y), pivot, delta);
            TransformPreviewObject {
                id: *id,
                bounds: [origin.x, origin.y, *w, *h],
                rotation: rotation + delta,
            }
        })
        .collect()
}

/// Computes the axis-aligned document frame for a set of previewed objects.
fn frame_from_preview_objects(objects: &[TransformPreviewObject]) -> [f64; 4] {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for object in objects {
        let [x, y, w, h] = object.bounds;
        let rotation = object.rotation;
        let (sin, cos) = rotation.sin_cos();
        let corners = [
            GPoint::new(x, y),
            GPoint::new(x + w, y),
            GPoint::new(x + w, y + h),
            GPoint::new(x, y + h),
        ];
        for corner in corners {
            let rotated = GPoint::new(
                x + (corner.x - x) * cos - (corner.y - y) * sin,
                y + (corner.x - x) * sin + (corner.y - y) * cos,
            );
            min_x = min_x.min(rotated.x);
            min_y = min_y.min(rotated.y);
            max_x = max_x.max(rotated.x);
            max_y = max_y.max(rotated.y);
        }
    }
    if objects.is_empty() {
        return [0.0, 0.0, 0.0, 0.0];
    }
    [
        min_x,
        min_y,
        (max_x - min_x).max(1.0),
        (max_y - min_y).max(1.0),
    ]
}

/// Total screen-space length of a lasso path.
fn lasso_path_length(points: &[GPoint]) -> f64 {
    points.windows(2).map(|w| w[0].distance_to(w[1])).sum()
}

/// Ray-casting point-in-polygon (implicitly closed).
fn point_in_polygon(point: GPoint, polygon: &[GPoint]) -> bool {
    let mut inside = false;
    let n = polygon.len();
    if n < 3 {
        return false;
    }
    let mut j = n - 1;
    for i in 0..n {
        let pi = polygon[i];
        let pj = polygon[j];
        if (pi.y > point.y) != (pj.y > point.y)
            && point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// True when two segments intersect (including touching).
fn segments_intersect(a: GPoint, b: GPoint, c: GPoint, d: GPoint) -> bool {
    let orient = |p: GPoint, q: GPoint, r: GPoint| {
        let v = (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y);
        if v.abs() < 1e-9 {
            0
        } else if v > 0.0 {
            1
        } else {
            2
        }
    };
    let on_segment = |p: GPoint, q: GPoint, r: GPoint| {
        q.x <= p.x.max(r.x) + 1e-9
            && q.x >= p.x.min(r.x) - 1e-9
            && q.y <= p.y.max(r.y) + 1e-9
            && q.y >= p.y.min(r.y) - 1e-9
    };
    let o1 = orient(a, b, c);
    let o2 = orient(a, b, d);
    let o3 = orient(c, d, a);
    let o4 = orient(c, d, b);
    if o1 != o2 && o3 != o4 {
        return true;
    }
    (o1 == 0 && on_segment(a, c, b))
        || (o2 == 0 && on_segment(a, d, b))
        || (o3 == 0 && on_segment(c, a, d))
        || (o4 == 0 && on_segment(c, b, d))
}

/// Tests whether a lasso polygon hits an object rect.
fn lasso_hits_rect(polygon: &[GPoint], bounds: [f64; 4], require_contained: bool) -> bool {
    let [ox, oy, ow, oh] = bounds;
    let corners = [
        GPoint::new(ox, oy),
        GPoint::new(ox + ow, oy),
        GPoint::new(ox + ow, oy + oh),
        GPoint::new(ox, oy + oh),
    ];
    let center = GPoint::new(ox + ow / 2.0, oy + oh / 2.0);
    if require_contained {
        return corners.iter().all(|c| point_in_polygon(*c, polygon));
    }
    // Intersect mode: any corner/center inside, any lasso vertex inside
    // the rect, or any edge crossing.
    if corners.iter().any(|c| point_in_polygon(*c, polygon)) || point_in_polygon(center, polygon) {
        return true;
    }
    let rect = GRect::new(ox, oy, ox + ow, oy + oh);
    if polygon.iter().any(|p| rect.contains(*p)) {
        return true;
    }
    let edges = [
        (corners[0], corners[1]),
        (corners[1], corners[2]),
        (corners[2], corners[3]),
        (corners[3], corners[0]),
    ];
    for w in polygon.windows(2) {
        for (e0, e1) in edges {
            if segments_intersect(w[0], w[1], e0, e1) {
                return true;
            }
        }
    }
    // Implicit closing edge.
    if let (Some(first), Some(last)) = (polygon.first(), polygon.last()) {
        for (e0, e1) in edges {
            if segments_intersect(*last, *first, e0, e1) {
                return true;
            }
        }
    }
    false
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
        SelectionHandleKind::TopLeft => Some(petunia_design_geometry::ResizeHandle::TopLeft),
        SelectionHandleKind::Top => Some(petunia_design_geometry::ResizeHandle::Top),
        SelectionHandleKind::TopRight => Some(petunia_design_geometry::ResizeHandle::TopRight),
        SelectionHandleKind::Right => Some(petunia_design_geometry::ResizeHandle::Right),
        SelectionHandleKind::BottomRight => {
            Some(petunia_design_geometry::ResizeHandle::BottomRight)
        }
        SelectionHandleKind::Bottom => Some(petunia_design_geometry::ResizeHandle::Bottom),
        SelectionHandleKind::BottomLeft => Some(petunia_design_geometry::ResizeHandle::BottomLeft),
        SelectionHandleKind::Left => Some(petunia_design_geometry::ResizeHandle::Left),
        SelectionHandleKind::Rotation => None,
        _ => None,
    };
    match mapped {
        Some(h) => petunia_design_geometry::resize_rect_from_handle(h, initial, dx, dy),
        None => (initial[0], initial[1], initial[2], initial[3]),
    }
}

#[cfg(test)]
mod select_tool_tests {
    use super::*;

    fn screen_doc(x: f64, y: f64) -> (GPoint, GPoint) {
        (GPoint::new(x, y), GPoint::new(x, y))
    }

    #[test]
    fn lasso_triangle_contains_center() {
        let tri = vec![
            GPoint::new(0.0, 0.0),
            GPoint::new(10.0, 0.0),
            GPoint::new(5.0, 10.0),
        ];
        assert!(point_in_polygon(GPoint::new(5.0, 4.0), &tri));
        assert!(!point_in_polygon(GPoint::new(0.0, 9.0), &tri));
    }

    #[test]
    fn lasso_rect_modes_differ_on_partial_overlap() {
        // Lasso covering only the left half of a 10x10 box at origin.
        let lasso = vec![
            GPoint::new(-5.0, -5.0),
            GPoint::new(5.0, -5.0),
            GPoint::new(5.0, 15.0),
            GPoint::new(-5.0, 15.0),
        ];
        assert!(lasso_hits_rect(&lasso, [0.0, 0.0, 10.0, 10.0], false));
        assert!(!lasso_hits_rect(&lasso, [0.0, 0.0, 10.0, 10.0], true));
    }

    #[test]
    fn rect_click_threshold_is_stable() {
        let (s0, _) = screen_doc(50.0, 50.0);
        let r = GRect::new(s0.x, s0.y, s0.x + 2.0, s0.y + 2.0);
        assert!(r.width() <= CLICK_THRESHOLD_PX && r.height() <= CLICK_THRESHOLD_PX);
    }
}
