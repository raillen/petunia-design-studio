//! Typography text creation tools (10.6, TOOLS_DECISIONS Batch 11).
//!
//! Artistic and Frame modes create straight text as before. Clicking (or
//! dragging along) a path object with either mode creates text-on-path:
//! the text flows along the target's evaluated outline between normalized
//! handles. Start/end handles drag with one undo entry; Alt-click detaches
//! back to straight text. The path object is never consumed.

use petunia_design_application::Command;
use petunia_design_document::{ChangeSet, ShapeKind, TextOnPathAttachment};
use petunia_design_foundation::{ObjectId, PetuniaError};
use petunia_design_geometry::{GPoint, GRect};

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase,
};

/// Handle hit radius in screen pixels for start/end grips.
const HANDLE_HIT_PX: f64 = 10.0;
/// Click-vs-drag threshold in screen pixels.
const CLICK_THRESHOLD_PX: f64 = 3.0;

/// Typography tool mode (10.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextToolMode {
    /// Click to create auto-sized headline text.
    Artistic,
    /// Drag to create bounded paragraph container frame.
    Frame,
}

/// Which span handle is dragged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpanHandle {
    Start,
    End,
}

/// Interactive tool for creating artistic headlines and text frames (10.6).
#[derive(Clone, Debug)]
pub struct TextTool {
    mode: TextToolMode,
    start_doc: Option<GPoint>,
    current_doc: Option<GPoint>,
    pending_path: Option<(ObjectId, f64)>,
    handle_drag: Option<(ObjectId, SpanHandle)>,
}

impl TextTool {
    /// Creates a text tool in a given mode.
    #[must_use]
    pub fn new(mode: TextToolMode) -> Self {
        Self {
            mode,
            start_doc: None,
            current_doc: None,
            pending_path: None,
            handle_drag: None,
        }
    }

    /// Cancels active text creation gesture.
    pub fn cancel(&mut self) {
        self.start_doc = None;
        self.current_doc = None;
        self.pending_path = None;
        self.handle_drag = None;
    }

    /// True while a gesture is in flight.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.start_doc.is_some() || self.handle_drag.is_some()
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
            PointerPhase::Move => self.on_move(event),
            PointerPhase::Up => self.on_up(event, bridge, camera),
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
        // 1. Span-handle drag on the single selected attached text.
        if let Some((id, which)) = hit_span_handle(event.doc_pos, bridge, camera) {
            // Alt-click a handle detaches instead of dragging.
            if event.modifiers.duplicate {
                return detach_text(bridge, id);
            }
            self.handle_drag = Some((id, which));
            self.start_doc = Some(event.doc_pos);
            self.current_doc = Some(event.doc_pos);
            return Ok(ChangeSet::empty());
        }
        // 2. Fresh click on a path arms text-on-path creation.
        if let Some((target, t)) = hit_path(event.doc_pos, bridge, camera) {
            let mut pt = event.doc_pos;
            if !event.modifiers.disable_snap {
                pt = snap.snap_point(pt, camera, &[]).point;
            }
            // Clicking the text's own target with Alt detaches selected texts.
            if event.modifiers.duplicate {
                return detach_selected_on_path(bridge, target);
            }
            self.pending_path = Some((target, t));
            self.start_doc = Some(pt);
            self.current_doc = Some(pt);
            return Ok(ChangeSet::empty());
        }
        // 3. Straight text creation drag as before.
        let mut pt = event.doc_pos;
        if !event.modifiers.disable_snap {
            pt = snap.snap_point(pt, camera, &[]).point;
        }
        self.pending_path = None;
        self.start_doc = Some(pt);
        self.current_doc = Some(pt);
        Ok(ChangeSet::empty())
    }

    fn on_move(&mut self, event: &NormalizedPointerEvent) -> Result<ChangeSet, PetuniaError> {
        if self.start_doc.is_some() {
            self.current_doc = Some(event.doc_pos);
        }
        Ok(ChangeSet::empty())
    }

    fn on_up(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
    ) -> Result<ChangeSet, PetuniaError> {
        if let Some((id, which)) = self.handle_drag.take() {
            self.start_doc = None;
            self.current_doc = None;
            self.pending_path = None;
            let exact_tol = petunia_design_geometry::zoom_flatten_tol(camera.zoom);
            return commit_handle_drag(bridge, id, which, event.doc_pos, exact_tol);
        }
        let start = self.start_doc.take();
        let current = self.current_doc.take();
        let pending = self.pending_path.take();
        let (Some(p0), Some(p1)) = (start, current) else {
            return Ok(ChangeSet::empty());
        };
        if let Some((target, t0)) = pending {
            // Drag along the path extends the span; a click runs to the end.
            let exact_tol = petunia_design_geometry::zoom_flatten_tol(camera.zoom);
            let clicked = p0.distance_to(p1) * camera.zoom.max(0.1) <= CLICK_THRESHOLD_PX;
            let t1 = if clicked {
                1.0
            } else {
                path_t_at(bridge, target, p1, exact_tol).unwrap_or(t0)
            };
            return commit_attached_text(bridge, target, t0, t1, self.mode, exact_tol);
        }
        let w = (p1.x - p0.x).abs();
        let h = (p1.y - p0.y).abs();
        let bounds = if self.mode == TextToolMode::Artistic || (w < 4.0 && h < 4.0) {
            [p0.x, p0.y, 160.0, 32.0]
        } else {
            let x = p0.x.min(p1.x);
            let y = p0.y.min(p1.y);
            [x, y, w.max(20.0), h.max(20.0)]
        };
        self.commit_text(bridge, bounds)
    }

    fn commit_text(
        &mut self,
        bridge: &mut PetuniaDesignGuiBridge,
        bounds: [f64; 4],
    ) -> Result<ChangeSet, PetuniaError> {
        let active_surface = bridge
            .session()
            .and_then(|s| s.active_surface())
            .ok_or_else(|| PetuniaError::invalid_input("no active surface for text creation"))?;

        let obj_id = bridge.next_object_id()?;
        let (name, text_shape) = match self.mode {
            TextToolMode::Artistic => petunia_design_document::shape_factory::artistic_text(),
            TextToolMode::Frame => petunia_design_document::shape_factory::frame_text(),
        };

        // One gesture, one undo entry (F-01).
        let changes = bridge.submit_all(
            "Create text",
            petunia_design_application::create_shape_commands(
                active_surface,
                obj_id,
                name,
                text_shape,
                Some(bounds),
                Some(petunia_design_document::shape_factory::DEFAULT_TEXT_FILL.to_string()),
                None,
            ),
        )?;

        bridge.set_selection(vec![obj_id]);
        Ok(changes)
    }

    /// Resolves overlays: frame marquee plus span handles of attached texts.
    #[must_use]
    pub fn overlays(
        &self,
        camera: &ViewportCamera,
        bridge: &PetuniaDesignGuiBridge,
    ) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        if let (Some(p0), Some(p1)) = (self.start_doc, self.current_doc) {
            // Creation drag preview (frame mode); attach drags preview the span.
            if self.pending_path.is_none() {
                let s0 = camera.doc_to_screen(p0);
                let s1 = camera.doc_to_screen(p1);
                overlays.marquee_screen = Some(GRect::new(s0.x, s0.y, s1.x, s1.y));
            } else if let Some((target, _)) = self.pending_path {
                let exact_tol = petunia_design_geometry::zoom_flatten_tol(camera.zoom);
                if let Some(span) = span_points(bridge, target, None, exact_tol) {
                    overlays.text_path_handles = Some(span);
                }
            }
        }
        // Committed span handles of the single selected attached text.
        let exact_tol = petunia_design_geometry::zoom_flatten_tol(camera.zoom);
        if let Some(handles) = selected_span_handles(bridge, exact_tol) {
            overlays.text_path_handles = Some(handles);
        }
        overlays
    }
}

/// Creates one text object attached to `target` between `t0` and `t1`.
fn commit_attached_text(
    bridge: &mut PetuniaDesignGuiBridge,
    target: ObjectId,
    t0: f64,
    t1: f64,
    mode: TextToolMode,
    tol: f64,
) -> Result<ChangeSet, PetuniaError> {
    let active_surface = bridge
        .session()
        .and_then(|s| s.active_surface())
        .ok_or_else(|| PetuniaError::invalid_input("no active surface for text creation"))?;
    let attachment = TextOnPathAttachment::new(target, t0, t1);
    let Some(span_bounds) = span_bounds(bridge, target, &attachment, tol) else {
        return Ok(ChangeSet::empty());
    };
    let obj_id = bridge.next_object_id()?;
    let (name, mut text_shape) = match mode {
        TextToolMode::Artistic => petunia_design_document::shape_factory::artistic_text(),
        TextToolMode::Frame => petunia_design_document::shape_factory::frame_text(),
    };
    if let ShapeKind::Text { on_path, .. } = &mut text_shape {
        *on_path = Some(attachment);
    }
    let changes = bridge.submit_all(
        "Create text on path",
        petunia_design_application::create_shape_commands(
            active_surface,
            obj_id,
            name,
            text_shape,
            Some(span_bounds),
            Some(petunia_design_document::shape_factory::DEFAULT_TEXT_FILL.to_string()),
            None,
        ),
    )?;
    bridge.set_selection(vec![obj_id]);
    Ok(changes)
}

/// Commits a span-handle drag: rewrites offsets and span bounds in one entry.
fn commit_handle_drag(
    bridge: &mut PetuniaDesignGuiBridge,
    id: ObjectId,
    which: SpanHandle,
    pt: GPoint,
    tol: f64,
) -> Result<ChangeSet, PetuniaError> {
    let (target, mut attachment, text_shape) = {
        let session = bridge
            .session()
            .ok_or_else(|| PetuniaError::invalid_input("no active document session"))?;
        let obj = session
            .find_object(id)
            .ok_or_else(|| PetuniaError::invalid_input(format!("object `{id}` does not exist")))?;
        let ShapeKind::Text {
            content,
            font_family,
            font_size,
            line_height,
            letter_spacing,
            on_path: Some(attachment),
        } = obj
            .shape
            .clone()
            .ok_or_else(|| PetuniaError::invalid_input(format!("object `{id}` has no shape")))?
        else {
            return Err(PetuniaError::invalid_input(format!(
                "object `{id}` is not attached text"
            )));
        };
        (
            attachment.target,
            attachment,
            ShapeKind::Text {
                content,
                font_family,
                font_size,
                line_height,
                letter_spacing,
                on_path: None,
            },
        )
    };
    let _ = text_shape;
    let Some(t) = path_t_at(bridge, target, pt, tol) else {
        return Ok(ChangeSet::empty());
    };
    match which {
        SpanHandle::Start => attachment.start = t,
        SpanHandle::End => attachment.end = t,
    }
    let attachment = TextOnPathAttachment::new(target, attachment.start, attachment.end);
    let Some(bounds) = span_bounds(bridge, target, &attachment, tol) else {
        return Ok(ChangeSet::empty());
    };
    // Rebuild the full text shape with the new attachment.
    let session_shape = bridge
        .session()
        .and_then(|s| s.find_object(id))
        .and_then(|o| o.shape.clone());
    let Some(ShapeKind::Text {
        content,
        font_family,
        font_size,
        line_height,
        letter_spacing,
        ..
    }) = session_shape
    else {
        return Ok(ChangeSet::empty());
    };
    bridge.submit_all(
        "Move text on path",
        vec![
            Command::SetShape {
                id,
                shape: Some(ShapeKind::Text {
                    content,
                    font_family,
                    font_size,
                    line_height,
                    letter_spacing,
                    on_path: Some(attachment),
                }),
            },
            Command::SetBounds {
                id,
                bounds: Some(bounds),
                rotation: 0.0,
            },
        ],
    )
}

/// Detaches one attached text back to straight text (bounds kept).
fn detach_text(
    bridge: &mut PetuniaDesignGuiBridge,
    id: ObjectId,
) -> Result<ChangeSet, PetuniaError> {
    let session_shape = bridge
        .session()
        .and_then(|s| s.find_object(id))
        .and_then(|o| o.shape.clone());
    let Some(ShapeKind::Text {
        content,
        font_family,
        font_size,
        line_height,
        letter_spacing,
        on_path: Some(_),
    }) = session_shape
    else {
        return Ok(ChangeSet::empty());
    };
    bridge.submit_all(
        "Detach text from path",
        vec![Command::SetShape {
            id,
            shape: Some(ShapeKind::Text {
                content,
                font_family,
                font_size,
                line_height,
                letter_spacing,
                on_path: None,
            }),
        }],
    )
}

/// Detaches every selected text attached to `target` (Alt-click the path).
fn detach_selected_on_path(
    bridge: &mut PetuniaDesignGuiBridge,
    target: ObjectId,
) -> Result<ChangeSet, PetuniaError> {
    let ids: Vec<ObjectId> = bridge
        .session()
        .map(|s| {
            s.selection
                .selected_ids
                .iter()
                .filter(|id| {
                    s.find_object(**id).is_some_and(|o| {
                        matches!(
                            &o.shape,
                            Some(ShapeKind::Text {
                                on_path: Some(a),
                                ..
                            }) if a.target == target
                        )
                    })
                })
                .copied()
                .collect()
        })
        .unwrap_or_default();
    if ids.is_empty() {
        return Ok(ChangeSet::empty());
    }
    let mut cmds = Vec::new();
    for id in ids {
        let Some(ShapeKind::Text {
            content,
            font_family,
            font_size,
            line_height,
            letter_spacing,
            on_path: Some(_),
        }) = bridge
            .session()
            .and_then(|s| s.find_object(id))
            .and_then(|o| o.shape.clone())
        else {
            continue;
        };
        cmds.push(Command::SetShape {
            id,
            shape: Some(ShapeKind::Text {
                content,
                font_family,
                font_size,
                line_height,
                letter_spacing,
                on_path: None,
            }),
        });
    }
    if cmds.is_empty() {
        return Ok(ChangeSet::empty());
    }
    bridge.submit_all("Detach text from path", cmds)
}

/// Topmost path object under `pt` with its outline fraction, if any.
/// Text objects themselves never qualify as targets.
fn hit_path(
    pt: GPoint,
    bridge: &PetuniaDesignGuiBridge,
    camera: &ViewportCamera,
) -> Option<(ObjectId, f64)> {
    let session = bridge.session()?;
    let tol = 8.0 / camera.zoom.max(0.1);
    let exact_tol = petunia_design_geometry::zoom_flatten_tol(camera.zoom);
    for id in session.spatial_candidates_point(pt, tol) {
        let Some(obj) = session.find_object(id) else {
            continue;
        };
        if !obj.visible || obj.locked {
            continue;
        }
        if !matches!(obj.shape, Some(ShapeKind::Path(_))) {
            continue;
        }
        // Fill hit or outline proximity (open strokes have no interior).
        // Outline queries run on the memoized evaluated path (F1 + F2).
        let near = obj.hit_test(pt)
            || bridge
                .cached_polygons(obj.id, exact_tol)
                .is_some_and(|polys| {
                    polys
                        .iter()
                        .flat_map(|c| c.windows(2))
                        .any(|w| dist_to_segment(pt, w[0], w[1]) <= tol)
                });
        if near {
            return bridge
                .cached_nearest_t(obj.id, pt, exact_tol)
                .map(|t| (obj.id, t));
        }
    }
    None
}

/// Normalized outline fraction of `pt` on `target`'s evaluated outline.
fn path_t_at(
    bridge: &PetuniaDesignGuiBridge,
    target: ObjectId,
    pt: GPoint,
    tol: f64,
) -> Option<f64> {
    bridge.cached_nearest_t(target, pt, tol)
}

/// Start/end handle positions of one attachment in document space.
fn attachment_handles(
    bridge: &PetuniaDesignGuiBridge,
    attachment: &TextOnPathAttachment,
    tol: f64,
) -> Option<[GPoint; 2]> {
    let (p0, _) = bridge.cached_sample_at(attachment.target, attachment.start, tol)?;
    let (p1, _) = bridge.cached_sample_at(attachment.target, attachment.end, tol)?;
    Some([p0, p1])
}

/// Span handles of the single selected attached text, if exactly one.
fn selected_span_handles(bridge: &PetuniaDesignGuiBridge, tol: f64) -> Option<Vec<GPoint>> {
    let session = bridge.session()?;
    if session.selection.selected_ids.len() != 1 {
        return None;
    }
    let obj = session.find_object(session.selection.selected_ids[0])?;
    let Some(ShapeKind::Text {
        on_path: Some(attachment),
        ..
    }) = &obj.shape
    else {
        return None;
    };
    attachment_handles(bridge, attachment, tol).map(|[a, b]| vec![a, b])
}

/// Hit-tests span handles of the single selected attached text.
fn hit_span_handle(
    pt: GPoint,
    bridge: &PetuniaDesignGuiBridge,
    camera: &ViewportCamera,
) -> Option<(ObjectId, SpanHandle)> {
    let session = bridge.session()?;
    if session.selection.selected_ids.len() != 1 {
        return None;
    }
    let id = session.selection.selected_ids[0];
    let obj = session.find_object(id)?;
    let Some(ShapeKind::Text {
        on_path: Some(attachment),
        ..
    }) = &obj.shape
    else {
        return None;
    };
    let exact_tol = petunia_design_geometry::zoom_flatten_tol(camera.zoom);
    let [p0, p1] = attachment_handles(bridge, attachment, exact_tol)?;
    let tol = HANDLE_HIT_PX / camera.zoom.max(0.1);
    if p0.distance_to(pt) <= tol {
        return Some((id, SpanHandle::Start));
    }
    if p1.distance_to(pt) <= tol {
        return Some((id, SpanHandle::End));
    }
    None
}

/// Sampled span polyline of a target between optional fractions.
fn span_points(
    bridge: &PetuniaDesignGuiBridge,
    target: ObjectId,
    attachment: Option<&TextOnPathAttachment>,
    tol: f64,
) -> Option<Vec<GPoint>> {
    let (a, b) = match attachment {
        Some(att) => (att.start, att.end),
        None => (0.0, 1.0),
    };
    // One shared flatten serves all 25 samples (F2).
    let steps = 24;
    let mut pts = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = a + (b - a) * ((i as f64) / (steps as f64));
        pts.push(bridge.cached_sample_at(target, t, tol)?.0);
    }
    Some(pts)
}

/// Bounding box of an attachment span (text object bounds).
fn span_bounds(
    bridge: &PetuniaDesignGuiBridge,
    target: ObjectId,
    attachment: &TextOnPathAttachment,
    tol: f64,
) -> Option<[f64; 4]> {
    let pts = span_points(bridge, target, Some(attachment), tol)?;
    let mut x0 = f64::MAX;
    let mut y0 = f64::MAX;
    let mut x1 = f64::MIN;
    let mut y1 = f64::MIN;
    for p in pts {
        x0 = x0.min(p.x);
        y0 = y0.min(p.y);
        x1 = x1.max(p.x);
        y1 = y1.max(p.y);
    }
    Some([x0, y0, (x1 - x0).max(20.0), (y1 - y0).max(20.0)])
}

/// Shortest distance from `pt` to segment `a->b`.
fn dist_to_segment(pt: GPoint, a: GPoint, b: GPoint) -> f64 {
    let abx = b.x - a.x;
    let aby = b.y - a.y;
    let len2 = (abx * abx + aby * aby).max(1e-12);
    let t = (((pt.x - a.x) * abx + (pt.y - a.y) * aby) / len2).clamp(0.0, 1.0);
    pt.distance_to(GPoint::new(a.x + abx * t, a.y + aby * t))
}
