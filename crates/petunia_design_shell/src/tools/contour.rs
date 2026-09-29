//! Corner radius and Contour offset tools (08.24, 09.31, 10.2, 10.3).
//!
//! Both modes are non-destructive: Corner edits parametric rectangle radii
//! (per corner), Contour upserts the live `ContourOffset` modifier. Geometry
//! commits once on pointer-up (F-01). Baking stays an explicit user action.

use petunia_design_application::Command;
use petunia_design_document::{ChangeSet, ModifierItem, ModifierKind};
use petunia_design_foundation::{ObjectId, PetuniaError};
use petunia_design_geometry::{GPath, GPoint, GRect, OffsetCap, OffsetJoin};

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{
    CanvasOverlays, CursorAffordance, SelectionHandle, SelectionHandleKind, SnapEngine,
    ViewportCamera,
};

use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase,
};

/// Screen-space hit radius for corner widgets.
const CORNER_HIT_PX: f64 = 14.0;

/// Operational mode for corner and contour manipulation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContourMode {
    /// Corner tool adjusting corner radii on parametric shapes or paths.
    Corner,
    /// Contour tool performing live path offsets.
    Contour,
}

/// Pending corner drag across selected rectangles.
#[derive(Clone, Debug)]
struct CornerDrag {
    start_doc: GPoint,
    current_doc: GPoint,
    /// `(object, corner index or None for all four, start radii)`.
    targets: Vec<(ObjectId, Option<usize>, [f64; 4])>,
    bounds: Vec<(ObjectId, [f64; 4])>,
}

/// Pending contour drag.
#[derive(Clone, Debug)]
struct ContourDrag {
    start_doc: GPoint,
    current_doc: GPoint,
}

/// Interactive Corner and Contour tool.
#[derive(Clone, Debug)]
pub struct ContourTool {
    mode: ContourMode,
    corner_drag: Option<CornerDrag>,
    contour_drag: Option<ContourDrag>,
    join: OffsetJoin,
    cap: OffsetCap,
    current_hover: Option<GPoint>,
}

impl ContourTool {
    /// Creates a contour or corner tool.
    #[must_use]
    pub fn new(mode: ContourMode) -> Self {
        Self {
            mode,
            corner_drag: None,
            contour_drag: None,
            join: OffsetJoin::Round,
            cap: OffsetCap::None,
            current_hover: None,
        }
    }

    /// Current contour join style (future toolbar binding).
    #[must_use]
    pub fn join(&self) -> OffsetJoin {
        self.join
    }

    /// Sets the contour join style.
    pub fn set_join(&mut self, join: OffsetJoin) {
        self.join = join;
    }

    /// Resets active drag.
    pub fn cancel(&mut self) {
        self.corner_drag = None;
        self.contour_drag = None;
        self.current_hover = None;
    }

    /// True while a drag gesture is in flight.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.corner_drag.is_some() || self.contour_drag.is_some()
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
        match self.mode {
            ContourMode::Corner => {
                // Direct corner hit across visible rectangles (topmost first).
                if let Some((id, corner)) = hit_corner(event.doc_pos, bridge, camera) {
                    if !bridge.selection().selected_ids.contains(&id) {
                        bridge.set_selection(vec![id]);
                    }
                    let all = event.modifiers.constrain;
                    let mut targets = Vec::new();
                    let mut bounds = Vec::new();
                    if let Some(session) = bridge.session() {
                        for &sel in &session.selection.selected_ids.clone() {
                            if let Some(obj) = session.find_object(sel) {
                                if let (
                                    Some(petunia_design_document::ShapeKind::Rectangle {
                                        corner_radii,
                                    }),
                                    Some(b),
                                ) = (&obj.shape, obj.bounds)
                                {
                                    if obj.visible && !obj.locked {
                                        targets.push((
                                            sel,
                                            if all { None } else { Some(corner) },
                                            *corner_radii,
                                        ));
                                        bounds.push((sel, b));
                                    }
                                }
                            }
                        }
                    }
                    if targets.is_empty() {
                        return Ok(ChangeSet::empty());
                    }
                    self.corner_drag = Some(CornerDrag {
                        start_doc: event.doc_pos,
                        current_doc: event.doc_pos,
                        targets,
                        bounds,
                    });
                }
                Ok(ChangeSet::empty())
            }
            ContourMode::Contour => {
                self.contour_drag = Some(ContourDrag {
                    start_doc: event.doc_pos,
                    current_doc: event.doc_pos,
                });
                Ok(ChangeSet::empty())
            }
        }
    }

    fn on_move(&mut self, event: &NormalizedPointerEvent) -> Result<ChangeSet, PetuniaError> {
        self.current_hover = Some(event.doc_pos);
        match (&mut self.corner_drag, &mut self.contour_drag) {
            (Some(drag), _) => {
                drag.current_doc = event.doc_pos;
                Ok(ChangeSet::empty())
            }
            (_, Some(drag)) => {
                drag.current_doc = event.doc_pos;
                Ok(ChangeSet::empty())
            }
            _ => Ok(ChangeSet::empty()),
        }
    }

    fn on_up(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
        _camera: &ViewportCamera,
    ) -> Result<ChangeSet, PetuniaError> {
        if let Some(drag) = self.corner_drag.as_mut() {
            drag.current_doc = event.doc_pos;
        }
        if let Some(drag) = self.contour_drag.as_mut() {
            drag.current_doc = event.doc_pos;
        }
        if let Some(drag) = self.corner_drag.take() {
            return self.commit_corner(&drag, bridge);
        }
        if let Some(drag) = self.contour_drag.take() {
            return self.commit_contour(&drag, bridge);
        }
        Ok(ChangeSet::empty())
    }

    /// Commits per-corner radii for every targeted rectangle in one undo entry.
    fn commit_corner(
        &mut self,
        drag: &CornerDrag,
        bridge: &mut PetuniaDesignGuiBridge,
    ) -> Result<ChangeSet, PetuniaError> {
        let mut cmds = Vec::new();
        for ((id, corner, start_radii), (_, bounds)) in drag.targets.iter().zip(drag.bounds.iter())
        {
            let [bx, by, bw, bh] = *bounds;
            let center = GPoint::new(bx + bw / 2.0, by + bh / 2.0);
            // Radial convention: moving away from the object center grows.
            let radial = drag.current_doc.distance_to(center) - drag.start_doc.distance_to(center);
            let mut radii = *start_radii;
            let indices: Vec<usize> = match corner {
                Some(i) => vec![*i],
                None => vec![0, 1, 2, 3],
            };
            for i in indices {
                radii[i] = petunia_design_geometry::step_corner_radius(
                    start_radii[i],
                    radial,
                    Some(*bounds),
                );
            }
            if radii != *start_radii {
                cmds.push(Command::SetShape {
                    id: *id,
                    shape: Some(petunia_design_document::ShapeKind::Rectangle {
                        corner_radii: radii,
                    }),
                });
            }
        }
        if cmds.is_empty() {
            return Ok(ChangeSet::empty());
        }
        bridge.submit_all("Edit corners", cmds)
    }

    /// Commits one live contour offset per selected object in one undo entry.
    /// Distances accumulate: each drag adds its radial delta to the live value.
    fn commit_contour(
        &mut self,
        drag: &ContourDrag,
        bridge: &mut PetuniaDesignGuiBridge,
    ) -> Result<ChangeSet, PetuniaError> {
        let delta = radial_delta(bridge, drag.start_doc, drag.current_doc);
        if delta.abs() < 1e-9 {
            return Ok(ChangeSet::empty());
        }
        let mut cmds = Vec::new();
        for id in bridge.selection().selected_ids.clone() {
            let next = contour_chain_for(bridge, id, delta, self.join, self.cap)?;
            let current = bridge.modifiers(id);
            if next != current {
                cmds.push(Command::SetModifiers {
                    id,
                    modifiers: next,
                });
            }
        }
        if cmds.is_empty() {
            return Ok(ChangeSet::empty());
        }
        bridge.submit_all("Contour offset", cmds)
    }

    /// Resolves overlays: pending outline preview while dragging (09.31).
    #[must_use]
    pub fn overlays(
        &self,
        bridge: &PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
    ) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        if let Some(preview) = self.pending_outline(bridge) {
            let tol = petunia_design_geometry::zoom_flatten_tol(camera.zoom);
            let drag_tol = if self.contour_drag.is_some() {
                tol * 2.0
            } else {
                tol
            };
            let screen: Vec<GPoint> = preview
                .to_polygons(drag_tol)
                .into_iter()
                .flatten()
                .map(|p| camera.doc_to_screen(p))
                .collect();
            if screen.len() >= 2 {
                overlays.marquee_screen = Some(screen_marquee(&screen));
            }
            overlays.path_preview = Some(preview);
        }

        match self.mode {
            ContourMode::Corner => {
                let hovered_corner =
                    self.current_hover.and_then(|h| hit_corner(h, bridge, camera));
                if self.corner_drag.is_some() {
                    overlays.cursor = CursorAffordance::ResizeNwse;
                } else if hovered_corner.is_some() {
                    overlays.cursor = CursorAffordance::Pointer;
                } else {
                    overlays.cursor = CursorAffordance::Crosshair;
                }

                // Render corner handle widgets on rectangles
                if let Some(session) = bridge.session() {
                    let handle_sz = 9.0;
                    let half_sz = handle_sz / 2.0;
                    let sel_ids = &session.selection.selected_ids;
                    let target_ids: Vec<ObjectId> = if !sel_ids.is_empty() {
                        sel_ids.clone()
                    } else if let Some(surface_id) = session.active_surface() {
                        if let Ok(surface) = session.surface(surface_id) {
                            surface
                                .objects()
                                .iter()
                                .filter(|o| o.visible && !o.locked)
                                .map(|o| o.id)
                                .collect()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    };

                    for id in target_ids {
                        if let Some(obj) = session.find_object(id) {
                            if let (
                                Some(petunia_design_document::ShapeKind::Rectangle { .. }),
                                Some([x, y, w, h]),
                            ) = (&obj.shape, obj.bounds)
                            {
                                let corners = [
                                    GPoint::new(x, y),
                                    GPoint::new(x + w, y),
                                    GPoint::new(x + w, y + h),
                                    GPoint::new(x, y + h),
                                ];
                                for (i, corner_pt) in corners.iter().enumerate() {
                                    let screen_pt = camera.doc_to_screen(*corner_pt);
                                    let is_hit = hovered_corner == Some((id, i));
                                    let kind = if is_hit {
                                        SelectionHandleKind::NodeSmoothSelected
                                    } else {
                                        SelectionHandleKind::NodeSmooth
                                    };
                                    overlays.handles.push(SelectionHandle {
                                        kind,
                                        doc_point: *corner_pt,
                                        screen_hit_box: GRect::new(
                                            screen_pt.x - half_sz,
                                            screen_pt.y - half_sz,
                                            screen_pt.x + half_sz,
                                            screen_pt.y + half_sz,
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            ContourMode::Contour => {
                if let Some(drag) = &self.contour_drag {
                    overlays.cursor = CursorAffordance::Grabbing;
                    overlays.pen_preview = Some(vec![drag.start_doc, drag.current_doc]);
                    let delta = radial_delta(bridge, drag.start_doc, drag.current_doc);
                    overlays.measure_badge = Some((drag.current_doc, format!("{delta:+.1} pt")));
                    let s = camera.doc_to_screen(drag.current_doc);
                    let half_sz = 4.5;
                    overlays.handles.push(SelectionHandle {
                        kind: SelectionHandleKind::NodeSmoothSelected,
                        doc_point: drag.current_doc,
                        screen_hit_box: GRect::new(
                            s.x - half_sz,
                            s.y - half_sz,
                            s.x + half_sz,
                            s.y + half_sz,
                        ),
                    });
                } else {
                    overlays.cursor = CursorAffordance::Crosshair;
                }
            }
        }

        overlays
    }

    /// Computes the pending outline without touching the document.
    fn pending_outline(&self, bridge: &PetuniaDesignGuiBridge) -> Option<GPath> {
        if let Some(drag) = &self.corner_drag {
            let ((_, corner, start_radii), (_, bounds)) =
                drag.targets.first().zip(drag.bounds.first())?;
            let [bx, by, bw, bh] = *bounds;
            let center = GPoint::new(bx + bw / 2.0, by + bh / 2.0);
            let radial = drag.current_doc.distance_to(center) - drag.start_doc.distance_to(center);
            let mut radii = *start_radii;
            match corner {
                Some(i) => {
                    radii[*i] = petunia_design_geometry::step_corner_radius(
                        start_radii[*i],
                        radial,
                        Some(*bounds),
                    );
                }
                None => {
                    for i in 0..4 {
                        radii[i] = petunia_design_geometry::step_corner_radius(
                            start_radii[i],
                            radial,
                            Some(*bounds),
                        );
                    }
                }
            }
            let rect = GRect::new(bx, by, bx + bw, by + bh);
            return Some(GPath::rect_corners(rect, radii));
        }
        if let Some(drag) = &self.contour_drag {
            let id = bridge.selection().selected_ids.first().copied()?;
            let delta = radial_delta(bridge, drag.start_doc, drag.current_doc);
            if delta.abs() < 1e-9 {
                return None;
            }
            let session = bridge.session()?;
            let obj = session.find_object(id)?;
            let base = obj.to_path();
            if base.is_empty() {
                return None;
            }
            // LOD optimization (F4): during active drag over dense paths (>100 verbs),
            // downsample for interactive preview (Krita Instant Preview / Inkscape LOD).
            // The full exact offset is computed and committed on pointer release.
            let path_for_preview = if base.verbs.len() > 100 {
                let pts: Vec<GPoint> = base.to_polygons(1.0).into_iter().flatten().collect();
                let simplified = petunia_design_geometry::simplify_rdp(&pts, 1.5);
                if simplified.len() >= 3 {
                    petunia_design_geometry::GPath::from_polygons(&[simplified])
                } else {
                    base
                }
            } else {
                base
            };
            return petunia_design_geometry::offset_path(
                &path_for_preview,
                delta,
                self.join,
                self.cap,
            );
        }
        None
    }
}

/// Screen-space marquee covering preview points (selection-style feedback).
fn screen_marquee(points: &[GPoint]) -> GRect {
    let mut x0 = f64::MAX;
    let mut y0 = f64::MAX;
    let mut x1 = f64::MIN;
    let mut y1 = f64::MIN;
    for p in points {
        x0 = x0.min(p.x);
        y0 = y0.min(p.y);
        x1 = x1.max(p.x);
        y1 = y1.max(p.y);
    }
    GRect::new(x0, y0, x1, y1)
}

/// Signed radial drag distance, averaged over selected objects.
/// Outward-from-center is positive (expand), inward negative (inset).
/// Falls back to the legacy right/down-positive drag length.
fn radial_delta(bridge: &PetuniaDesignGuiBridge, p0: GPoint, p1: GPoint) -> f64 {
    let mut total = 0.0;
    let mut count = 0;
    if let Some(session) = bridge.session() {
        for id in session.selection.selected_ids.clone() {
            if let Some(obj) = session.find_object(id) {
                if let Some([bx, by, bw, bh]) = obj.bounds {
                    let center = GPoint::new(bx + bw / 2.0, by + bh / 2.0);
                    total += p1.distance_to(center) - p0.distance_to(center);
                    count += 1;
                }
            }
        }
    }
    if count == 0 {
        let dx = p1.x - p0.x;
        let dy = p1.y - p0.y;
        let len = (dx * dx + dy * dy).sqrt();
        return len * if dx + dy >= 0.0 { 1.0 } else { -1.0 };
    }
    total / f64::from(count)
}

/// Chain for one object with the contour distance applied (upsert).
fn contour_chain_for(
    bridge: &PetuniaDesignGuiBridge,
    id: ObjectId,
    delta: f64,
    join: OffsetJoin,
    cap: OffsetCap,
) -> Result<Vec<ModifierItem>, PetuniaError> {
    let current = bridge.modifiers(id);
    // Absolute set (drag from gesture start): read the object's live distance
    // and add the gesture delta so repeated drags accumulate predictably.
    let base_distance: f64 = current
        .iter()
        .map(|m| match &m.kind {
            ModifierKind::ContourOffset { distance, .. } => *distance,
            // Contour drags ignore other domains (transparency, warp, crop).
            _ => 0.0,
        })
        .sum();
    let absolute = base_distance + delta;
    let mut next: Vec<ModifierItem> = current
        .into_iter()
        .filter(|m| !matches!(m.kind, ModifierKind::ContourOffset { .. }))
        .collect();
    if absolute.abs() >= 1e-9 {
        let nid = next.iter().map(|m| m.id).max().unwrap_or(0) + 1;
        next.push(ModifierItem::enabled(
            nid,
            ModifierKind::ContourOffset {
                distance: absolute,
                join,
                cap,
            },
        ));
    }
    Ok(next)
}

/// Hit-tests rectangle corners topmost-first.
/// Returns `(object, corner index in rect_corners order)`.
fn hit_corner(
    pt: GPoint,
    bridge: &PetuniaDesignGuiBridge,
    camera: &ViewportCamera,
) -> Option<(ObjectId, usize)> {
    let session = bridge.session()?;
    let surface_id = session.active_surface()?;
    let surface = session.surface(surface_id).ok()?;
    let tol = CORNER_HIT_PX / camera.zoom.max(0.1);
    for obj in surface.objects().iter().rev() {
        if !obj.visible || obj.locked {
            continue;
        }
        if let (Some(petunia_design_document::ShapeKind::Rectangle { .. }), Some([x, y, w, h])) =
            (&obj.shape, obj.bounds)
        {
            let corners = [
                GPoint::new(x, y),
                GPoint::new(x + w, y),
                GPoint::new(x + w, y + h),
                GPoint::new(x, y + h),
            ];
            for (i, corner) in corners.iter().enumerate() {
                if corner.distance_to(pt) <= tol {
                    return Some((obj.id, i));
                }
            }
        }
    }
    None
}
