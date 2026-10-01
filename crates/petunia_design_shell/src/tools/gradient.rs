//! Vector gradient and transparency tools (08.24, 10.4, TOOLS_DECISIONS Batch 7).
//!
//! Fill mode draws gradient geometry (linear line, radial radius) and edits
//! stops directly on canvas: double-click the line adds a sampled stop,
//! double-click a stop removes it (minimum two), dragging a stop moves its
//! offset. One gesture, one undo. Transparency mode stays the documented
//! whole-stack proxy until it becomes a live modifier (roadmap #10).

use std::time::Instant;

use petunia_design_application::Command;
use petunia_design_document::{AppearanceStack, ChangeSet, Paint};
use petunia_design_foundation::{ObjectId, PetuniaError};
use petunia_design_geometry::GPoint;

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{
    CanvasOverlays, CursorAffordance, GradientOverlay, GradientOverlayKind, SnapEngine,
    ViewportCamera,
};

use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase,
};

/// Click-vs-drag threshold in screen pixels.
const CLICK_THRESHOLD_PX: f64 = 3.0;
/// Double-click window for stop add/remove.
const DOUBLE_CLICK_MS: u128 = 400;
const DOUBLE_CLICK_PX: f64 = 6.0;
/// Stop handle hit radius in screen pixels.
const STOP_HIT_PX: f64 = 8.0;

/// Mode for the gradient tool (10.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GradientToolMode {
    /// Color gradient on fill/stroke.
    Fill,
    /// Mask transparency gradient.
    Transparency,
}

/// Which gradient geometry the Fill mode draws.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GradientKind {
    /// Linear gradient along the drag vector.
    #[default]
    Linear,
    /// Radial gradient: drag sets center then edge radius.
    Radial,
}

/// Interactive tool for plotting gradient vector lines on canvas (10.4).
#[derive(Clone, Debug)]
pub struct GradientTool {
    mode: GradientToolMode,
    kind: GradientKind,
    start_doc: Option<GPoint>,
    current_doc: Option<GPoint>,
    hover_doc: Option<GPoint>,
    drag_stop: Option<usize>,
    constrain: bool,
    last_down: Option<(Instant, GPoint)>,
}

impl GradientTool {
    /// Creates a gradient tool in fill or transparency mode.
    #[must_use]
    pub fn new(mode: GradientToolMode) -> Self {
        Self {
            mode,
            kind: GradientKind::Linear,
            start_doc: None,
            current_doc: None,
            hover_doc: None,
            drag_stop: None,
            constrain: false,
            last_down: None,
        }
    }

    /// Which gradient geometry Fill mode draws (future toolbar binding).
    #[must_use]
    pub fn kind(&self) -> GradientKind {
        self.kind
    }

    /// Switches between linear and radial geometry.
    pub fn set_kind(&mut self, kind: GradientKind) {
        if self.kind != kind {
            self.cancel();
            self.kind = kind;
        }
    }

    /// Resets active gradient drag.
    pub fn cancel(&mut self) {
        self.start_doc = None;
        self.current_doc = None;
        self.hover_doc = None;
        self.drag_stop = None;
        self.constrain = false;
    }

    /// True while a gesture is in flight.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.start_doc.is_some()
    }

    /// Handles pointer events.
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
        self.constrain = event.modifiers.constrain;
        self.hover_doc = None;
        if self.mode == GradientToolMode::Fill {
            // Double-click on a stop removes it; on the line adds one.
            let now = Instant::now();
            let is_double = match self.last_down {
                Some((t, p)) => {
                    now.duration_since(t).as_millis() <= DOUBLE_CLICK_MS
                        && p.distance_to(event.screen_pos) <= DOUBLE_CLICK_PX
                }
                None => false,
            };
            self.last_down = Some((now, event.screen_pos));
            if is_double {
                // A double-click never starts a line drag.
                self.start_doc = None;
                self.current_doc = None;
                self.drag_stop = None;
                return self.double_click(event.doc_pos, bridge, camera);
            }
            // Down on a stop handle starts a stop drag.
            if let Some(index) = hit_stop(event.doc_pos, bridge, camera) {
                self.drag_stop = Some(index);
                self.start_doc = Some(event.doc_pos);
                self.current_doc = Some(event.doc_pos);
                return Ok(ChangeSet::empty());
            }
        }
        self.start_doc = Some(event.doc_pos);
        self.current_doc = Some(event.doc_pos);
        Ok(ChangeSet::empty())
    }

    fn on_move(&mut self, event: &NormalizedPointerEvent) -> Result<ChangeSet, PetuniaError> {
        self.constrain = event.modifiers.constrain;
        if self.start_doc.is_some() {
            self.current_doc = Some(event.doc_pos);
        } else {
            self.hover_doc = Some(event.doc_pos);
        }
        Ok(ChangeSet::empty())
    }

    fn on_up(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
    ) -> Result<ChangeSet, PetuniaError> {
        self.constrain = false;
        self.hover_doc = Some(event.doc_pos);
        if let Some(index) = self.drag_stop.take() {
            self.start_doc = None;
            self.current_doc = None;
            return self.commit_stop_move(
                index,
                event.doc_pos,
                event.modifiers.constrain,
                bridge,
                camera,
            );
        }
        let start = self.start_doc.take();
        let current = self.current_doc.take();
        if let (Some(p0), Some(p1)) = (start, current) {
            // Clicks never create or destroy gradients.
            if p0.distance_to(p1) * camera.zoom.max(0.1) <= CLICK_THRESHOLD_PX {
                return Ok(ChangeSet::empty());
            }
            let selected = bridge.selection().selected_ids;
            match self.mode {
                GradientToolMode::Fill => {
                    let end = if event.modifiers.constrain {
                        snap_linear_45(p0, p1)
                    } else {
                        p1
                    };
                    let mut cmds = Vec::new();
                    for id in selected {
                        if let Some(stack) = apply_fill_vector(bridge, id, p0, end, self.kind) {
                            cmds.push(Command::SetAppearance {
                                id,
                                appearance: Some(stack),
                            });
                        }
                    }
                    return bridge.submit_all("Edit gradient", cmds);
                }
                GradientToolMode::Transparency => {
                    // Live transparency vector (09.31): the drag defines the
                    // mask gradient; nothing flattens until explicit Bake.
                    let end = if event.modifiers.constrain {
                        snap_linear_45(p0, p1)
                    } else {
                        p1
                    };
                    return commit_transparency_vector(bridge, &selected, p0, end);
                }
            }
        }
        Ok(ChangeSet::empty())
    }

    /// Double-click: on a stop removes it, on the line adds a sampled stop.
    fn double_click(
        &mut self,
        pt: GPoint,
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
    ) -> Result<ChangeSet, PetuniaError> {
        if let Some(index) = hit_stop(pt, bridge, camera) {
            return commit_stop_remove(bridge, index);
        }
        commit_stop_add(bridge, pt, camera)
    }

    /// Commits a stop-offset drag (Shift snaps to 0.05 steps).
    fn commit_stop_move(
        &mut self,
        index: usize,
        pt: GPoint,
        snap_steps: bool,
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut cmds = Vec::new();
        for id in bridge.selection().selected_ids.clone() {
            let Some(stack) = gradient_stack(bridge, id) else {
                continue;
            };
            let Some((start, end)) = gradient_line(&stack) else {
                continue;
            };
            let mut t = project_t(pt, start, end, camera);
            if snap_steps {
                t = (t / 0.05).round() * 0.05;
            }
            let t = t.clamp(0.0, 1.0);
            let mut stack = stack;
            if set_stop_offset(&mut stack, index, t) {
                cmds.push(Command::SetAppearance {
                    id,
                    appearance: Some(stack),
                });
            }
        }
        if cmds.is_empty() {
            return Ok(ChangeSet::empty());
        }
        bridge.submit_all("Move gradient stop", cmds)
    }

    /// Resolves overlays showing the gradient line plus stop handles.
    #[must_use]
    pub fn overlays(
        &self,
        bridge: &PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
    ) -> CanvasOverlays {
        // 1. Resolve cursor affordance
        let mut overlays = CanvasOverlays {
            cursor: if self.drag_stop.is_some() {
                CursorAffordance::Grabbing
            } else if self.is_active() {
                CursorAffordance::Crosshair
            } else if let Some(hover) = self.hover_doc {
                if hit_stop(hover, bridge, camera).is_some() {
                    CursorAffordance::Pointer
                } else {
                    CursorAffordance::Crosshair
                }
            } else {
                CursorAffordance::Crosshair
            },
            ..Default::default()
        };

        // 2. Resolve gradient overlay
        let selected_id = bridge.selection().selected_ids.first().copied();
        let selected_stack = selected_id.and_then(|id| gradient_stack(bridge, id));

        if let (Some(p0), Some(p1)) = (self.start_doc, self.current_doc) {
            if let Some(stop_idx) = self.drag_stop {
                // Moving an existing stop handle in-flight
                if let Some(stack) = &selected_stack {
                    if let Some((start, end)) = gradient_line(stack) {
                        let mut t = project_t(p1, start, end, camera);
                        if self.constrain {
                            t = (t / 0.05).round() * 0.05;
                        }
                        let t = t.clamp(0.0, 1.0);
                        let mut stops: Vec<(f64, GPoint)> = gradient_stops(stack)
                            .iter()
                            .enumerate()
                            .map(|(i, (offset, _))| {
                                let off = if i == stop_idx { t } else { *offset };
                                (off, camera.doc_to_screen(lerp_point(start, end, off)))
                            })
                            .collect();
                        stops.sort_by(|a, b| {
                            a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        let stop_colors = gradient_stops(stack)
                            .iter()
                            .map(|(_, c)| petunia_design_document::resolve_color_to_rgb(c))
                            .collect();
                        overlays.gradient = Some(GradientOverlay {
                            start: camera.doc_to_screen(start),
                            end: camera.doc_to_screen(end),
                            stops,
                            stop_colors,
                            kind: match gradient_paint(stack) {
                                Some(Paint::RadialGradient(_)) => GradientOverlayKind::Radial,
                                _ => GradientOverlayKind::Linear,
                            },
                        });
                    }
                }
            } else {
                // In-flight vector creation or reposition
                let end = if self.constrain {
                    snap_linear_45(p0, p1)
                } else {
                    p1
                };
                overlays.pen_preview = Some(vec![p0, end]);
                let stops = if let Some(stack) = &selected_stack {
                    gradient_stops(stack)
                        .iter()
                        .map(|(offset, _)| {
                            (*offset, camera.doc_to_screen(lerp_point(p0, end, *offset)))
                        })
                        .collect()
                } else {
                    vec![
                        (0.0, camera.doc_to_screen(p0)),
                        (1.0, camera.doc_to_screen(end)),
                    ]
                };
                let stop_colors = if let Some(stack) = &selected_stack {
                    gradient_stops(stack)
                        .iter()
                        .map(|(_, c)| petunia_design_document::resolve_color_to_rgb(c))
                        .collect()
                } else {
                    vec![[1.0, 1.0, 1.0], [0.0, 0.0, 0.0]]
                };
                overlays.gradient = Some(GradientOverlay {
                    start: camera.doc_to_screen(p0),
                    end: camera.doc_to_screen(end),
                    stops,
                    stop_colors,
                    kind: match self.kind {
                        GradientKind::Radial => GradientOverlayKind::Radial,
                        GradientKind::Linear => GradientOverlayKind::Linear,
                    },
                });
            }
        } else if let Some(stack) = &selected_stack {
            // Committed gradient overlay
            if let Some((start, end)) = gradient_line(stack) {
                let stops = gradient_stops(stack)
                    .iter()
                    .map(|(offset, _)| {
                        (
                            *offset,
                            camera.doc_to_screen(lerp_point(start, end, *offset)),
                        )
                    })
                    .collect();
                let stop_colors = gradient_stops(stack)
                    .iter()
                    .map(|(_, c)| petunia_design_document::resolve_color_to_rgb(c))
                    .collect();
                overlays.gradient = Some(GradientOverlay {
                    start: camera.doc_to_screen(start),
                    end: camera.doc_to_screen(end),
                    stops,
                    stop_colors,
                    kind: match gradient_paint(stack) {
                        Some(Paint::RadialGradient(_)) => GradientOverlayKind::Radial,
                        _ => GradientOverlayKind::Linear,
                    },
                });
            }
        }

        overlays
    }
}

/// Snaps a linear drag vector to 45° steps (Shift).
fn snap_linear_45(from: GPoint, to: GPoint) -> GPoint {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let dist = dx.hypot(dy);
    if dist < 1e-9 {
        return from;
    }
    let step = std::f64::consts::FRAC_PI_4;
    let angle = (dy.atan2(dx) / step).round() * step;
    GPoint::new(from.x + dist * angle.cos(), from.y + dist * angle.sin())
}

fn lerp_point(a: GPoint, b: GPoint, t: f64) -> GPoint {
    GPoint::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

/// Primary gradient stack of one object, if its primary fill is a gradient.
fn gradient_stack(bridge: &PetuniaDesignGuiBridge, id: ObjectId) -> Option<AppearanceStack> {
    let stack = bridge.session()?.find_object(id)?.effective_appearance();
    if gradient_paint(&stack).is_some() {
        Some(stack)
    } else {
        None
    }
}

/// Primary gradient paint of a stack, if any.
fn gradient_paint(stack: &AppearanceStack) -> Option<&Paint> {
    let fill = stack.primary_fill()?;
    match &fill.paint {
        Paint::LinearGradient(_) | Paint::RadialGradient(_) => Some(&fill.paint),
        _ => None,
    }
}

/// Gradient geometry endpoints in document space.
/// Linear uses its vector; radial runs center to edge.
fn gradient_line(stack: &AppearanceStack) -> Option<(GPoint, GPoint)> {
    match gradient_paint(stack)? {
        Paint::LinearGradient(g) => Some((
            GPoint::new(g.start[0], g.start[1]),
            GPoint::new(g.end[0], g.end[1]),
        )),
        Paint::RadialGradient(g) => Some((
            GPoint::new(g.center[0], g.center[1]),
            GPoint::new(g.center[0] + g.radius, g.center[1]),
        )),
        _ => None,
    }
}

/// `(offset, color)` stops of the primary gradient.
fn gradient_stops(stack: &AppearanceStack) -> Vec<(f64, String)> {
    match gradient_paint(stack) {
        Some(Paint::LinearGradient(g)) => g
            .stops
            .iter()
            .map(|s| (s.offset, s.color.clone()))
            .collect(),
        Some(Paint::RadialGradient(g)) => g
            .stops
            .iter()
            .map(|s| (s.offset, s.color.clone()))
            .collect(),
        _ => Vec::new(),
    }
}

/// Projects `pt` onto the gradient line as normalized `t`, in screen space
/// so handle hit-testing matches what the user sees.
fn project_t(pt: GPoint, start: GPoint, end: GPoint, camera: &ViewportCamera) -> f64 {
    let a = camera.doc_to_screen(start);
    let b = camera.doc_to_screen(end);
    let p = camera.doc_to_screen(pt);
    let abx = b.x - a.x;
    let aby = b.y - a.y;
    let len2 = (abx * abx + aby * aby).max(1e-12);
    (((p.x - a.x) * abx + (p.y - a.y) * aby) / len2).clamp(0.0, 1.0)
}

/// Hit-tests stop handles of selected gradients (screen-space radius).
fn hit_stop(pt: GPoint, bridge: &PetuniaDesignGuiBridge, camera: &ViewportCamera) -> Option<usize> {
    let tol = STOP_HIT_PX / camera.zoom.max(0.1);
    // Doc-space equivalent: reuse project geometry per object below.
    let _ = tol;
    for id in bridge.selection().selected_ids.clone() {
        let Some(stack) = gradient_stack(bridge, id) else {
            continue;
        };
        let Some((start, end)) = gradient_line(&stack) else {
            continue;
        };
        let screen = camera.doc_to_screen(pt);
        for (index, (offset, _)) in gradient_stops(&stack).iter().enumerate() {
            let handle = camera.doc_to_screen(lerp_point(start, end, *offset));
            if handle.distance_to(screen) <= STOP_HIT_PX {
                return Some(index);
            }
        }
    }
    None
}

/// Sets one stop offset (by sorted position index) across the stack.
fn set_stop_offset(stack: &mut AppearanceStack, index: usize, offset: f64) -> bool {
    let Some(paint) = stack.fills.first_mut().map(|f| &mut f.paint) else {
        return false;
    };
    let stops = match paint {
        Paint::LinearGradient(g) => &mut g.stops,
        Paint::RadialGradient(g) => &mut g.stops,
        _ => return false,
    };
    let mut order: Vec<usize> = (0..stops.len()).collect();
    order.sort_by(|a, b| {
        stops[*a]
            .offset
            .partial_cmp(&stops[*b].offset)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let Some(&slot) = order.get(index) else {
        return false;
    };
    if (stops[slot].offset - offset).abs() < 1e-9 {
        return false;
    }
    stops[slot].offset = offset;
    match paint {
        Paint::LinearGradient(g) => g.sort_and_reindex(),
        // RadialGradient shares stop semantics; reindex through the same rule.
        Paint::RadialGradient(g) => sort_stops(&mut g.stops),
        _ => {}
    }
    true
}

/// Sorts stops by offset and reassigns stable sequential ids.
fn sort_stops(stops: &mut [petunia_design_document::GradientStop]) {
    stops.sort_by(|a, b| {
        a.offset
            .partial_cmp(&b.offset)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (i, s) in stops.iter_mut().enumerate() {
        s.id = i as u32 + 1;
    }
}

/// Adds a sampled stop at the click position on every selected gradient.
fn commit_stop_add(
    bridge: &mut PetuniaDesignGuiBridge,
    pt: GPoint,
    camera: &ViewportCamera,
) -> Result<ChangeSet, PetuniaError> {
    use petunia_design_document::GradientStop;
    let mut cmds = Vec::new();
    for id in bridge.selection().selected_ids.clone() {
        let Some(mut stack) = gradient_stack(bridge, id) else {
            continue;
        };
        let Some((start, end)) = gradient_line(&stack) else {
            continue;
        };
        let t = project_t(pt, start, end, camera);
        let color = match gradient_paint(&stack) {
            Some(Paint::LinearGradient(g)) => g.sample_rgba(t).map(|(rgb, _)| {
                format!(
                    "rgb({},{},{})",
                    (rgb[0] * 255.0).round() as u8,
                    (rgb[1] * 255.0).round() as u8,
                    (rgb[2] * 255.0).round() as u8
                )
            }),
            Some(Paint::RadialGradient(g)) => g.sample_rgba(t).map(|(rgb, _)| {
                format!(
                    "rgb({},{},{})",
                    (rgb[0] * 255.0).round() as u8,
                    (rgb[1] * 255.0).round() as u8,
                    (rgb[2] * 255.0).round() as u8
                )
            }),
            _ => None,
        };
        let Some(color) = color else {
            continue;
        };
        let paint = stack.fills.first_mut().map(|f| &mut f.paint);
        match paint {
            Some(Paint::LinearGradient(g)) => {
                g.stops.push(GradientStop::new(t, color));
                g.sort_and_reindex();
            }
            Some(Paint::RadialGradient(g)) => {
                g.stops.push(GradientStop::new(t, color));
                sort_stops(&mut g.stops);
            }
            _ => continue,
        }
        cmds.push(Command::SetAppearance {
            id,
            appearance: Some(stack),
        });
    }
    if cmds.is_empty() {
        return Ok(ChangeSet::empty());
    }
    bridge.submit_all("Add gradient stop", cmds)
}

/// Removes the stop at sorted position `index` (minimum two survive).
fn commit_stop_remove(
    bridge: &mut PetuniaDesignGuiBridge,
    index: usize,
) -> Result<ChangeSet, PetuniaError> {
    let mut cmds = Vec::new();
    for id in bridge.selection().selected_ids.clone() {
        let Some(mut stack) = gradient_stack(bridge, id) else {
            continue;
        };
        let paint = stack.fills.first_mut().map(|f| &mut f.paint);
        let removed = match paint {
            Some(Paint::LinearGradient(g)) if g.stops.len() > 2 => {
                remove_sorted_stop(&mut g.stops, index);
                g.sort_and_reindex();
                true
            }
            Some(Paint::RadialGradient(g)) if g.stops.len() > 2 => {
                remove_sorted_stop(&mut g.stops, index);
                sort_stops(&mut g.stops);
                true
            }
            _ => false,
        };
        if removed {
            cmds.push(Command::SetAppearance {
                id,
                appearance: Some(stack),
            });
        }
    }
    if cmds.is_empty() {
        return Ok(ChangeSet::empty());
    }
    bridge.submit_all("Remove gradient stop", cmds)
}

/// Removes the stop at sorted position `index`.
#[allow(clippy::ptr_arg)]
fn remove_sorted_stop(stops: &mut Vec<petunia_design_document::GradientStop>, index: usize) {
    let mut order: Vec<usize> = (0..stops.len()).collect();
    order.sort_by(|a, b| {
        stops[*a]
            .offset
            .partial_cmp(&stops[*b].offset)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    if let Some(&slot) = order.get(index) {
        stops.remove(slot);
    }
}

/// Repositions (or creates) the primary gradient of one object.
/// Linear edits the vector; radial sets center and edge radius.
/// Returns the new stack, or `None` when the object is missing.
fn apply_fill_vector(
    bridge: &PetuniaDesignGuiBridge,
    id: ObjectId,
    start: GPoint,
    end: GPoint,
    kind: GradientKind,
) -> Option<AppearanceStack> {
    use petunia_design_document::{GradientStop, LinearGradient, Paint, RadialGradient};
    let stack = bridge.session()?.find_object(id)?.effective_appearance();
    let paint = match (kind, stack.primary_fill().map(|f| f.paint.clone())) {
        (GradientKind::Linear, Some(Paint::LinearGradient(mut g))) => {
            g.start = [start.x, start.y];
            g.end = [end.x, end.y];
            g.sort_and_reindex();
            Paint::LinearGradient(g)
        }
        (GradientKind::Radial, Some(Paint::RadialGradient(mut g))) => {
            g.center = [start.x, start.y];
            g.radius = start.distance_to(end).max(1.0);
            Paint::RadialGradient(g)
        }
        // Switching geometry keeps stops, remaps anchors (no color invented).
        (GradientKind::Linear, Some(Paint::RadialGradient(g))) => {
            Paint::LinearGradient(LinearGradient {
                start: [start.x, start.y],
                end: [end.x, end.y],
                stops: remap_stops(g.stops),
            })
        }
        (GradientKind::Radial, Some(Paint::LinearGradient(g))) => {
            Paint::RadialGradient(RadialGradient {
                center: [start.x, start.y],
                radius: start.distance_to(end).max(1.0),
                stops: remap_stops(g.stops),
            })
        }
        (_, Some(Paint::Solid(token))) => match kind {
            GradientKind::Linear => Paint::LinearGradient(LinearGradient::new(
                [start.x, start.y],
                [end.x, end.y],
                vec![
                    GradientStop::new(0.0, token.clone()),
                    GradientStop::new(1.0, token),
                ],
            )),
            GradientKind::Radial => Paint::RadialGradient(RadialGradient::new(
                [start.x, start.y],
                start.distance_to(end).max(1.0),
                vec![
                    GradientStop::new(0.0, token.clone()),
                    GradientStop::new(1.0, token),
                ],
            )),
        },
        _ => match kind {
            GradientKind::Linear => Paint::LinearGradient(LinearGradient::new(
                [start.x, start.y],
                [end.x, end.y],
                vec![
                    GradientStop::new(0.0, "ptnd.blue/500"),
                    GradientStop::new(1.0, "ptnd.blue/500"),
                ],
            )),
            GradientKind::Radial => Paint::RadialGradient(RadialGradient::new(
                [start.x, start.y],
                start.distance_to(end).max(1.0),
                vec![
                    GradientStop::new(0.0, "ptnd.blue/500"),
                    GradientStop::new(1.0, "ptnd.blue/500"),
                ],
            )),
        },
    };
    Some(petunia_design_application::appearance_service::with_primary_gradient(stack, paint))
}

/// Re-sorts stops and reassigns stable ids after a geometry switch.
fn remap_stops(
    mut stops: Vec<petunia_design_document::GradientStop>,
) -> Vec<petunia_design_document::GradientStop> {
    sort_stops(&mut stops);
    stops
}

/// Commits one live transparency vector per selected object in one undo
/// entry (09.31). Clicks clear nothing and create nothing.
fn commit_transparency_vector(
    bridge: &mut PetuniaDesignGuiBridge,
    selected: &[ObjectId],
    p0: GPoint,
    p1: GPoint,
) -> Result<ChangeSet, PetuniaError> {
    if p0.distance_to(p1) < 1e-9 {
        return Ok(ChangeSet::empty());
    }
    let mut cmds = Vec::new();
    for id in selected {
        let next = transparency_chain_for(bridge, *id, p0, p1)?;
        let current = bridge.modifiers(*id);
        if next != current {
            cmds.push(Command::SetModifiers {
                id: *id,
                modifiers: next,
            });
        }
    }
    if cmds.is_empty() {
        return Ok(ChangeSet::empty());
    }
    bridge.submit_all("Transparency vector", cmds)
}

/// Chain for one object with the transparency vector applied (upsert).
/// Distances accumulate like contour: each drag adds to the live entry by
/// replacing its vector (vectors don't sum, latest drag wins per entry).
fn transparency_chain_for(
    bridge: &PetuniaDesignGuiBridge,
    id: ObjectId,
    p0: GPoint,
    p1: GPoint,
) -> Result<Vec<petunia_design_document::ModifierItem>, PetuniaError> {
    use petunia_design_document::{ModifierItem, ModifierKind, OpacityStop};
    let current = bridge.modifiers(id);
    if bridge.session().and_then(|s| s.find_object(id)).is_none() {
        return Err(PetuniaError::invalid_input(format!(
            "object `{id}` does not exist"
        )));
    }
    let mut next: Vec<ModifierItem> = current
        .into_iter()
        .filter(|m| !matches!(m.kind, ModifierKind::TransparentGradient { .. }))
        .collect();
    let nid = next
        .iter()
        .map(|m| m.id)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| PetuniaError::invalid_input("modifier IDs exhausted"))?;
    let item = ModifierItem::enabled(
        nid,
        ModifierKind::TransparentGradient {
            start: [p0.x, p0.y],
            end: [p1.x, p1.y],
            stops: vec![OpacityStop::new(0.0, 1.0), OpacityStop::new(1.0, 0.0)],
        },
    );
    let document = bridge
        .session()
        .ok_or_else(|| PetuniaError::invalid_input("no session"))?
        .document();
    next.push(document.modifier_from_world(id, item)?);
    Ok(next)
}
