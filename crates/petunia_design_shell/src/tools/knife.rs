//! Knife and Scissors vector slicing tools (10.2, TOOLS_DECISIONS Batch 6).
//!
//! Market split: Knife drags a cut line across shapes (Corel/Affinity);
//! Scissors click-splits at points (Illustrator). One gesture, one undo.

use petunia_design_application::Command;
use petunia_design_document::{ChangeSet, ShapeKind};
use petunia_design_foundation::{ObjectId, PetuniaError, SurfaceId};
use petunia_design_geometry::{GPoint, GRect};

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{
    CanvasOverlays, CursorAffordance, SelectionHandle, SelectionHandleKind, SnapEngine,
    ViewportCamera,
};

use super::stroke_hit::contours_near_point;

use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase,
};

/// Click-vs-drag threshold in screen pixels (scissors click = split).
const CLICK_THRESHOLD_PX: f64 = 3.0;

/// Snaps a 2D line segment to the nearest 45-degree angle (0, 45, 90, 135, etc.).
fn snap_linear_45(p0: GPoint, p1: GPoint) -> GPoint {
    let dx = p1.x - p0.x;
    let dy = p1.y - p0.y;
    let dist = dx.hypot(dy);
    if dist < 1e-6 {
        return p1;
    }
    let angle = dy.atan2(dx);
    let step = std::f64::consts::FRAC_PI_4;
    let snapped = (angle / step).round() * step;
    GPoint::new(p0.x + dist * snapped.cos(), p0.y + dist * snapped.sin())
}

/// Slicing tool mode (10.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnifeMode {
    /// Knife dragging a cut line across paths.
    Knife,
    /// Scissors clicking directly at a path point to split.
    Scissors,
}

/// Knife and scissors vector splitting tool.
#[derive(Clone, Debug)]
pub struct KnifeTool {
    mode: KnifeMode,
    start_doc: Option<GPoint>,
    current_doc: Option<GPoint>,
    cut_point: Option<GPoint>,
    hover_doc: Option<GPoint>,
}

impl KnifeTool {
    /// Creates a knife or scissors tool.
    #[must_use]
    pub fn new(mode: KnifeMode) -> Self {
        Self {
            mode,
            start_doc: None,
            current_doc: None,
            cut_point: None,
            hover_doc: None,
        }
    }

    /// Current mode.
    #[must_use]
    pub fn mode(&self) -> KnifeMode {
        self.mode
    }

    /// Cancels active slicing gesture.
    pub fn cancel(&mut self) {
        self.start_doc = None;
        self.current_doc = None;
        self.cut_point = None;
        self.hover_doc = None;
    }

    /// Handles pointer events for vector slicing.
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
        match event.phase {
            PointerPhase::Down => {
                if event.button != PointerButton::Primary {
                    return Ok(ChangeSet::empty());
                }
                snap.reset_hysteresis();
                self.start_doc = Some(event.doc_pos);
                self.current_doc = Some(event.doc_pos);
                self.cut_point = None;
                self.hover_doc = Some(event.doc_pos);
                Ok(ChangeSet::empty())
            }
            PointerPhase::Move => {
                self.hover_doc = Some(event.doc_pos);
                if let Some(p0) = self.start_doc {
                    let mut pt = event.doc_pos;
                    if event.modifiers.constrain {
                        pt = snap_linear_45(p0, pt);
                    }
                    self.current_doc = Some(pt);
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Up => {
                let start = self.start_doc.take();
                let current = self.current_doc.take();
                self.hover_doc = Some(event.doc_pos);
                let (Some(p0), Some(mut p1)) = (start, current) else {
                    return Ok(ChangeSet::empty());
                };
                if event.modifiers.constrain {
                    p1 = snap_linear_45(p0, p1);
                }
                // Scissors ignores drags; knife ignores clicks.
                let zoom = camera.zoom.max(0.1);
                let dragged = p0.distance_to(p1) * zoom > CLICK_THRESHOLD_PX;
                let tol = 8.0 / zoom;
                match self.mode {
                    KnifeMode::Knife if dragged => self.commit_knife(p0, p1, tol, bridge),
                    KnifeMode::Scissors if !dragged => self.commit_scissors(p0, tol, bridge),
                    _ => Ok(ChangeSet::empty()),
                }
            }
            PointerPhase::Cancel => {
                self.cancel();
                snap.reset_hysteresis();
                Ok(ChangeSet::empty())
            }
        }
    }

    /// Cuts along the drag segment: every crossed object splits into pieces,
    /// all in one undo entry. Parametric shapes convert in-batch (the gesture
    /// asks for a cut; a cut needs curves).
    fn commit_knife(
        &mut self,
        p0: GPoint,
        p1: GPoint,
        tol: f64,
        bridge: &mut PetuniaDesignGuiBridge,
    ) -> Result<ChangeSet, PetuniaError> {
        if p0.distance_to(p1) < 1e-9 {
            return Ok(ChangeSet::empty());
        }
        let targets = hit_targets_along(p0, p1, tol, bridge);
        if targets.is_empty() {
            return Ok(ChangeSet::empty());
        }
        let active_surface = bridge.session().and_then(|s| s.active_surface());
        let Some(surface_id) = active_surface else {
            return Ok(ChangeSet::empty());
        };
        let mut cmds = Vec::new();
        for id in targets {
            let Some(source) = bridge.session().and_then(|s| s.find_object(id)).cloned() else {
                continue;
            };
            let base = base_path(bridge, id);
            let Some(base) = base else {
                continue;
            };
            let pieces = petunia_design_geometry::cut_path_by_line(&base, p0, p1, 0.5);
            if pieces.len() <= 1 && pieces.first().is_some_and(|p| p.verbs == base.verbs) {
                continue;
            }
            push_cut_commands(bridge, &mut cmds, surface_id, &source, &pieces)?;
        }
        if cmds.is_empty() {
            return Ok(ChangeSet::empty());
        }
        bridge.submit_all("Knife cut", cmds)
    }

    /// Splits the topmost hit object once at the click point.
    fn commit_scissors(
        &mut self,
        pt: GPoint,
        tol: f64,
        bridge: &mut PetuniaDesignGuiBridge,
    ) -> Result<ChangeSet, PetuniaError> {
        self.cut_point = Some(pt);
        let Some(id) = hit_object_top(pt, tol, bridge) else {
            return Ok(ChangeSet::empty());
        };
        let active_surface = bridge.session().and_then(|s| s.active_surface());
        let Some(surface_id) = active_surface else {
            return Ok(ChangeSet::empty());
        };
        let Some(source) = bridge.session().and_then(|s| s.find_object(id)).cloned() else {
            return Ok(ChangeSet::empty());
        };
        let Some(base) = base_path(bridge, id) else {
            return Ok(ChangeSet::empty());
        };
        let pieces = petunia_design_geometry::split_path_at_point(&base, pt, 0.5);
        if pieces.len() <= 1 && pieces.first().is_some_and(|p| p.verbs == base.verbs) {
            return Ok(ChangeSet::empty());
        }
        let mut cmds = Vec::new();
        push_cut_commands(bridge, &mut cmds, surface_id, &source, &pieces)?;
        if cmds.is_empty() {
            return Ok(ChangeSet::empty());
        }
        bridge.submit_all("Scissors split", cmds)
    }

    /// Resolves overlays: knife shows the cut line, scissors the cut point.
    #[must_use]
    pub fn overlays(
        &self,
        camera: &ViewportCamera,
        bridge: &PetuniaDesignGuiBridge,
    ) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        match self.mode {
            KnifeMode::Knife => {
                overlays.cursor = CursorAffordance::Crosshair;
                if let (Some(p0), Some(p1)) = (self.start_doc, self.current_doc) {
                    overlays.pen_preview = Some(vec![p0, p1]);
                    overlays.region_subtractive = true;
                }
            }
            KnifeMode::Scissors => {
                let tol = 8.0 / camera.zoom.max(0.1);
                let hovered_sliceable = self
                    .hover_doc
                    .and_then(|pt| hit_object_top(pt, tol, bridge))
                    .is_some();
                if hovered_sliceable {
                    overlays.cursor = CursorAffordance::Pointer;
                    if let Some(hover) = self.hover_doc {
                        let s = camera.doc_to_screen(hover);
                        let half_sz = 4.0;
                        overlays.handles.push(SelectionHandle {
                            kind: SelectionHandleKind::NodeCuspSelected,
                            doc_point: hover,
                            screen_hit_box: GRect::new(
                                s.x - half_sz,
                                s.y - half_sz,
                                s.x + half_sz,
                                s.y + half_sz,
                            ),
                        });
                    }
                } else {
                    overlays.cursor = CursorAffordance::Crosshair;
                }
                if let Some(pt) = self.cut_point {
                    overlays.pen_preview = Some(vec![pt]);
                    overlays.region_subtractive = true;
                }
            }
        }
        overlays
    }
}

/// Objects crossed by segment `p0->p1`: fill hits plus outline proximity
/// (open strokes have no interior to hit), topmost first, restricted to
/// sliceable shapes (text and containers excluded).
fn hit_targets_along(
    p0: GPoint,
    p1: GPoint,
    tol: f64,
    bridge: &PetuniaDesignGuiBridge,
) -> Vec<ObjectId> {
    let Some(session) = bridge.session() else {
        return Vec::new();
    };
    // Candidates come from the active-surface spatial index (F3).
    let length = p0.distance_to(p1);
    let steps = ((length / 2.0).ceil() as usize).clamp(1, 64);
    // Spatial prefilter (F3): segment bbox over evaluated bounds, then the
    // exact sampled test only on candidates.
    let seg = [
        p0.x.min(p1.x) - tol,
        p0.y.min(p1.y) - tol,
        p0.x.max(p1.x) + tol,
        p0.y.max(p1.y) + tol,
    ];
    let mut targets = Vec::new();
    for id in session.spatial_candidates_rect(seg) {
        let Some(obj) = session.find_object(id) else {
            continue;
        };
        if !obj.visible || obj.locked || !is_sliceable(&obj.shape) {
            continue;
        }
        let crossed = (0..=steps).any(|i| {
            let t = (i as f64) / (steps as f64);
            near_object(
                obj,
                GPoint::new(p0.x + (p1.x - p0.x) * t, p0.y + (p1.y - p0.y) * t),
                tol,
            )
        });
        if crossed && !targets.contains(&obj.id) {
            targets.push(obj.id);
        }
    }
    targets
}

/// True for paths (slice directly) and convertible parametric shapes.
/// Text rejects conversion explicitly, containers have no outline.
fn is_sliceable(shape: &Option<ShapeKind>) -> bool {
    match shape {
        Some(ShapeKind::Path(_)) => true,
        Some(ShapeKind::Text { .. }) | Some(ShapeKind::Image { .. }) | None => false,
        Some(_) => true,
    }
}

/// Base outline for cutting: stored paths directly, parametric shapes via
/// their canonical outline (converted in-batch by the caller).
fn base_path(
    bridge: &PetuniaDesignGuiBridge,
    id: ObjectId,
) -> Option<petunia_design_geometry::GPath> {
    let session = bridge.session()?;
    let obj = session.find_object(id)?;
    match &obj.shape {
        Some(ShapeKind::Path(path)) => Some(path.clone()),
        Some(_) => Some(obj.to_path()),
        None => None,
    }
}

/// True when the object needs conversion before its pieces land.
fn needs_convert(bridge: &PetuniaDesignGuiBridge, id: ObjectId) -> bool {
    bridge
        .session()
        .and_then(|s| s.find_object(id))
        .is_some_and(|obj| !matches!(obj.shape, Some(ShapeKind::Path(_))))
}

/// Emits one undo-batch worth of cut commands: the source object becomes
/// piece zero (converting parametric shapes first), extra pieces are created
/// with copied style. Degenerate pieces are dropped; fully-degenerate cuts
/// yield no commands.
fn push_cut_commands(
    bridge: &mut PetuniaDesignGuiBridge,
    cmds: &mut Vec<Command>,
    surface_id: SurfaceId,
    source: &petunia_design_document::DocumentObject,
    pieces: &[petunia_design_geometry::GPath],
) -> Result<(), PetuniaError> {
    let mut kept: Vec<(petunia_design_geometry::GPath, [f64; 4])> = Vec::new();
    for piece in pieces {
        // Length-based degenerate check: straight cuts have zero-height boxes.
        if piece.approx_length(0.5) < 1.0 {
            continue;
        }
        let Some(rect) = piece.bounding_box() else {
            continue;
        };
        kept.push((
            piece.clone(),
            [
                rect.x0,
                rect.y0,
                rect.width().max(1.0),
                rect.height().max(1.0),
            ],
        ));
    }
    if kept.is_empty() {
        return Ok(());
    }
    if needs_convert(bridge, source.id) {
        cmds.push(Command::ConvertToCurves { id: source.id });
    }
    let stroke = source.stroke.clone().map(|s| (s, source.stroke_width));
    let mut first = true;
    for (piece, bounds) in kept {
        if first {
            first = false;
            cmds.push(Command::SetShape {
                id: source.id,
                shape: Some(ShapeKind::Path(piece)),
            });
            cmds.push(Command::SetBounds {
                id: source.id,
                bounds: Some(bounds),
                rotation: source.rotation,
            });
        } else {
            let new_id = bridge.next_object_id()?;
            cmds.extend(petunia_design_application::create_shape_commands(
                surface_id,
                new_id,
                format!("{} Cut", source.name),
                ShapeKind::Path(piece),
                Some(bounds),
                source.fill.clone(),
                stroke.clone(),
            ));
            if source.opacity != 1.0 {
                cmds.push(Command::SetOpacity {
                    id: new_id,
                    opacity: source.opacity,
                });
            }
        }
    }
    Ok(())
}

/// Topmost sliceable object under `pt`, if any.
fn hit_object_top(pt: GPoint, tol: f64, bridge: &PetuniaDesignGuiBridge) -> Option<ObjectId> {
    let session = bridge.session()?;
    for id in session.spatial_candidates_point(pt, tol) {
        let Some(obj) = session.find_object(id) else {
            continue;
        };
        if obj.visible && !obj.locked && is_sliceable(&obj.shape) && near_object(obj, pt, tol) {
            return Some(id);
        }
    }
    None
}

/// True on fill hit or within `tol` of the evaluated outline.
/// Open strokes have no interior, so outline proximity is the only way
/// to target them. Segment math is shared with the Select tool
/// (`super::stroke_hit`); only the polygon source stays local (direct
/// `evaluated_path` for cut targeting vs. memoized `GeoCache` in Select).
fn near_object(obj: &petunia_design_document::DocumentObject, pt: GPoint, tol: f64) -> bool {
    if obj.hit_test(pt) {
        return true;
    }
    contours_near_point(&obj.evaluated_path().to_polygons(0.5), pt, tol)
}
