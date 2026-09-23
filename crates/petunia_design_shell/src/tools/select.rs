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
    compute_selection_handles, hit_test_handle_or_border, CanvasOverlays, SelectionHandleKind,
    SnapEngine, ViewportCamera,
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
        initial_positions: Vec<(ObjectId, [f64; 4], f64)>,
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
    pressed_object: Option<ObjectId>,
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
            pressed_object: None,
        }
    }

    /// Cancels any active gesture and resets to idle.
    pub fn cancel(&mut self) {
        self.state = SelectToolState::Idle;
        self.pressed_object = None;
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
                self.pressed_object = None;
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
                initial_positions,
                is_duplicate: event.modifiers.duplicate,
                anchor_id: Some(id),
                was_already_selected,
            };
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
                self.hovered_object = self.hit_test_objects(event.doc_pos, bridge, camera);
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

        let prev_state = std::mem::replace(&mut self.state, SelectToolState::Idle);
        // Click feedback always releases on pointer-up.
        self.pressed_object = None;

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
                initial_positions,
                is_duplicate,
                anchor_id,
                was_already_selected,
            } => {
                let dx = current_doc.x - start_doc.x;
                let dy = current_doc.y - start_doc.y;

                if dx.abs() < f64::EPSILON && dy.abs() < f64::EPSILON {
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
                    let center = petunia_design_geometry::GPoint::new(bx + bw / 2.0, by + bh / 2.0);
                    let delta =
                        petunia_design_geometry::pivot_angle_delta(start_doc, current_doc, center)
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
            if let Some(surface_id) = session.active_surface() {
                if let Ok(surface) = session.surface(surface_id) {
                    for obj in surface.objects() {
                        if obj.visible && !obj.locked {
                            if let Some([ox, oy, ow, oh]) = obj.bounds {
                                if lasso_hits_rect(polygon_doc, [ox, oy, ow, oh], require_contained)
                                {
                                    matched.push(obj.id);
                                }
                            }
                        }
                    }
                }
            }
        }
        matched
    }

    /// Spatial hit-testing for selecting objects.
    fn hit_test_objects(
        &self,
        doc_pos: GPoint,
        bridge: &PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
    ) -> Option<ObjectId> {
        let session = bridge.session()?;
        let surface_id = session.active_surface()?;
        let surface = session.surface(surface_id).ok()?;

        let tolerance = 4.0 / camera.zoom;

        // Search in reverse z-order (topmost first): bbox pre-check
        // with tolerance, then exact shape hit-test (10.1).
        // The exact test runs on the memoized evaluated outline (F1).
        for obj in surface.objects().iter().rev() {
            if obj.visible && !obj.locked {
                if let Some([x, y, w, h]) = obj.bounds {
                    let rect = GRect::new(
                        x - tolerance,
                        y - tolerance,
                        x + w + tolerance,
                        y + h + tolerance,
                    );
                    if rect.contains(doc_pos) && bridge.cached_hit(obj.id, doc_pos) {
                        return Some(obj.id);
                    }
                }
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
        let mut overlays = CanvasOverlays {
            hovered_object: self.hovered_object,
            pressed_object: self.pressed_object,
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
