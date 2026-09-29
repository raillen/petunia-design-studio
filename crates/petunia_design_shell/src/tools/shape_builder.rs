//! Shape Builder and Vector Flood Fill tools (08.24, 10.3, TOOLS_DECISIONS Batch 9).
//!
//! Regions are coverage faces over the selected evaluated outlines: the set
//! of selected objects covering a point defines its region (intersection of
//! covering minus union of the rest). Click creates (or Alt-subtracts) one
//! region; drag merges every crossed region into one object. SmartFill shares
//! the engine but fills with the default token. Flood of unenclosed negative
//! space stays future work (face detection). One gesture, one undo.

use petunia_design_application::Command;
use petunia_design_document::{ChangeSet, ShapeKind};
use petunia_design_foundation::{ObjectId, PetuniaError};
use petunia_design_geometry::{GPath, GPoint};

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{CanvasOverlays, CursorAffordance, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase,
};

/// Click-vs-drag threshold in screen pixels.
const CLICK_THRESHOLD_PX: f64 = 3.0;
/// Drag sampling step in document points for region collection (F4 LOD).
const DRAG_SAMPLE_STEP: f64 = 8.0;
/// Flatten tolerance for region booleans (F-21).
const REGION_TOLERANCE: f64 = 0.5;
/// Default SmartFill token (matches gradient-tool default precedent).
const SMART_FILL_TOKEN: &str = "ptnd.blue/500";

/// Operational mode for region construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuilderMode {
    /// Interactive shape builder combining or subtracting candidate regions.
    ShapeBuilder,
    /// Smart Fill clicking an enclosed region to create a new filled path.
    SmartFill,
}

/// Operation mode for Shape Builder: Add (unify region) or Subtract (carve region out).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuilderOp {
    /// Add candidate regions together.
    Add,
    /// Subtract candidate regions from covering shapes.
    Subtract,
}

/// Interactive tool for constructive geometry region synthesis.
#[derive(Clone, Debug)]
pub struct ShapeBuilderTool {
    mode: BuilderMode,
    op: BuilderOp,
    fill_token: String,
    start_doc: Option<GPoint>,
    current_doc: Option<GPoint>,
    hover_doc: Option<GPoint>,
    drag_path: Vec<GPoint>,
    subtract_mode: bool,
}

impl ShapeBuilderTool {
    /// Creates a shape builder or smart fill tool.
    #[must_use]
    pub fn new(mode: BuilderMode) -> Self {
        Self {
            mode,
            op: BuilderOp::Add,
            fill_token: SMART_FILL_TOKEN.to_string(),
            start_doc: None,
            current_doc: None,
            hover_doc: None,
            drag_path: Vec::new(),
            subtract_mode: false,
        }
    }

    /// Returns the builder mode.
    #[must_use]
    pub fn mode(&self) -> BuilderMode {
        self.mode
    }

    /// Returns the builder operation (Add vs Subtract).
    #[must_use]
    pub fn op(&self) -> BuilderOp {
        self.op
    }

    /// Sets the builder operation (Add vs Subtract).
    pub fn set_op(&mut self, op: BuilderOp) {
        self.op = op;
    }

    /// Returns the fill token used for SmartFill.
    #[must_use]
    pub fn fill_token(&self) -> &str {
        &self.fill_token
    }

    /// Sets the fill token used for SmartFill.
    pub fn set_fill_token(&mut self, token: impl Into<String>) {
        self.fill_token = token.into();
    }

    /// Resets active drag.
    pub fn cancel(&mut self) {
        self.start_doc = None;
        self.current_doc = None;
        self.hover_doc = None;
        self.drag_path.clear();
        self.subtract_mode = false;
    }

    /// True while a drag gesture is in flight.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.start_doc.is_some()
    }

    /// True while subtraction mode is active (Alt / Option held or Subtract op).
    #[must_use]
    pub fn is_subtract_mode(&self) -> bool {
        self.subtract_mode || self.op == BuilderOp::Subtract
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
            PointerPhase::Down => {
                if event.button != PointerButton::Primary {
                    return Ok(ChangeSet::empty());
                }
                snap.reset_hysteresis();
                self.start_doc = Some(event.doc_pos);
                self.current_doc = Some(event.doc_pos);
                self.drag_path = vec![event.doc_pos];
                self.hover_doc = None;
                self.subtract_mode = event.modifiers.duplicate || self.op == BuilderOp::Subtract;
                Ok(ChangeSet::empty())
            }
            PointerPhase::Move => {
                self.subtract_mode = event.modifiers.duplicate || self.op == BuilderOp::Subtract;
                if self.start_doc.is_some() {
                    self.current_doc = Some(event.doc_pos);
                    if let Some(last) = self.drag_path.last() {
                        if last.distance_to(event.doc_pos) >= 2.0 {
                            self.drag_path.push(event.doc_pos);
                        }
                    } else {
                        self.drag_path.push(event.doc_pos);
                    }
                } else {
                    self.hover_doc = Some(event.doc_pos);
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Up => {
                let start = self.start_doc.take();
                let current = self.current_doc.take();
                let drag_path = std::mem::take(&mut self.drag_path);
                self.hover_doc = None;
                let (Some(p0), Some(p1)) = (start, current) else {
                    self.subtract_mode = false;
                    return Ok(ChangeSet::empty());
                };
                let clicked = p0.distance_to(p1) * camera.zoom.max(0.1) <= CLICK_THRESHOLD_PX;
                let subtract = event.modifiers.duplicate
                    || self.subtract_mode
                    || self.op == BuilderOp::Subtract;
                self.subtract_mode = false;
                if clicked {
                    self.click_region(p0, subtract, bridge)
                } else {
                    self.drag_regions_path(&drag_path, p0, p1, subtract, bridge)
                }
            }
            PointerPhase::Cancel => {
                self.cancel();
                snap.reset_hysteresis();
                Ok(ChangeSet::empty())
            }
        }
    }

    /// Clicks one region: creates it (merge) or carves it out (Alt-subtract).
    /// SmartFill on empty canvas floods the bounded negative-space face;
    /// unbounded faces (touching the frame) are a NoOp.
    fn click_region(
        &mut self,
        pt: GPoint,
        subtract: bool,
        bridge: &mut PetuniaDesignGuiBridge,
    ) -> Result<ChangeSet, PetuniaError> {
        let covering = covering_set(bridge, pt);
        if covering.is_empty() {
            if self.mode == BuilderMode::SmartFill && !subtract {
                return flood_empty_face(bridge, pt, &self.fill_token);
            }
            return Ok(ChangeSet::empty());
        }
        let region = region_polygons(bridge, &covering);
        if region.is_empty() {
            return Ok(ChangeSet::empty());
        }
        if subtract {
            subtract_region(bridge, &covering, &region)
        } else {
            create_region(bridge, &covering, &region, self.mode, &self.fill_token)
        }
    }

    /// Drags across regions along the freehand path: merges every crossed region into one object,
    /// or subtracts their union (Alt) from the covering objects.
    fn drag_regions_path(
        &mut self,
        path: &[GPoint],
        p0: GPoint,
        p1: GPoint,
        subtract: bool,
        bridge: &mut PetuniaDesignGuiBridge,
    ) -> Result<ChangeSet, PetuniaError> {
        let segments: Vec<(GPoint, GPoint)> = if path.len() >= 2 {
            path.windows(2).map(|w| (w[0], w[1])).collect()
        } else {
            vec![(p0, p1)]
        };
        let mut signatures: Vec<Vec<ObjectId>> = Vec::new();
        for (seg_start, seg_end) in segments {
            let length = seg_start.distance_to(seg_end);
            let steps = ((length / DRAG_SAMPLE_STEP).ceil() as usize).clamp(1, 64);
            for i in 0..=steps {
                let t = (i as f64) / (steps as f64);
                let pt = GPoint::new(
                    seg_start.x + (seg_end.x - seg_start.x) * t,
                    seg_start.y + (seg_end.y - seg_start.y) * t,
                );
                let mut covering = covering_set(bridge, pt);
                covering.sort();
                if !covering.is_empty() && !signatures.contains(&covering) {
                    signatures.push(covering);
                }
            }
        }
        if signatures.is_empty() {
            // SmartFill drags flood at the release point (click semantics).
            if self.mode == BuilderMode::SmartFill && !subtract {
                return flood_empty_face(bridge, p1, &self.fill_token);
            }
            return Ok(ChangeSet::empty());
        }
        let mut regions: Vec<Vec<GPoint>> = Vec::new();
        for signature in &signatures {
            regions.extend(region_polygons(bridge, signature));
        }
        let merged = union_all(&regions);
        let covering: Vec<ObjectId> = {
            let mut all = signatures.concat();
            all.sort();
            all.dedup();
            all
        };
        if subtract {
            subtract_region(bridge, &covering, &merged)
        } else {
            create_region(bridge, &covering, &merged, self.mode, &self.fill_token)
        }
    }

    /// Resolves overlays: hovered or in-flight merged region outline.
    #[must_use]
    pub fn overlays(&self, bridge: &PetuniaDesignGuiBridge) -> CanvasOverlays {
        let mut overlays = CanvasOverlays {
            region_subtractive: self.subtract_mode || self.op == BuilderOp::Subtract,
            ..Default::default()
        };

        let preview = if let (Some(p0), Some(p1)) = (self.start_doc, self.current_doc) {
            // In-flight drag: outline the merged crossed regions and render cutting path.
            if self.drag_path.len() >= 2 {
                overlays.pen_preview = Some(self.drag_path.clone());
            }
            drag_path_preview(bridge, &self.drag_path, p0, p1)
        } else if let Some(hover) = self.hover_doc {
            let covering = covering_set(bridge, hover);
            if covering.is_empty() {
                // SmartFill previews the flood face on empty canvas.
                if self.mode == BuilderMode::SmartFill {
                    flood_face_polygons(bridge, hover).map(|face| face.concat())
                } else {
                    None
                }
            } else {
                Some(region_polygons(bridge, &covering).concat())
            }
        } else {
            None
        };
        if let Some(points) = preview {
            if points.len() >= 2 {
                overlays.region_preview = Some(points);
            }
        }

        overlays.cursor = if self.is_active() {
            CursorAffordance::Grabbing
        } else if let Some(hover) = self.hover_doc {
            let covering = covering_set(bridge, hover);
            if !covering.is_empty()
                || (self.mode == BuilderMode::SmartFill
                    && flood_face_polygons(bridge, hover).is_some())
            {
                CursorAffordance::Pointer
            } else {
                CursorAffordance::Crosshair
            }
        } else {
            CursorAffordance::Crosshair
        };

        overlays
    }
}

/// Objects in the selection whose evaluated outline covers `pt`.
fn covering_set(bridge: &PetuniaDesignGuiBridge, pt: GPoint) -> Vec<ObjectId> {
    let Some(session) = bridge.session() else {
        return Vec::new();
    };
    session
        .selection
        .selected_ids
        .iter()
        .filter_map(|id| session.find_object(*id))
        .filter(|obj| obj.visible && !obj.locked)
        // Fixed tolerance here (overlays lack camera context); the shared
        // flatten cache still dedups repeated queries at this tolerance.
        .filter(|obj| bridge.cached_hit(obj.id, pt, 0.5))
        .map(|obj| obj.id)
        .collect()
}

/// Region polygons for a coverage signature: intersection of covering
/// outlines minus the union of the other selected outlines.
fn region_polygons(bridge: &PetuniaDesignGuiBridge, covering: &[ObjectId]) -> Vec<Vec<GPoint>> {
    let Some(session) = bridge.session() else {
        return Vec::new();
    };
    let outlines = |id: ObjectId| -> Vec<Vec<GPoint>> {
        bridge
            .cached_polygons(id, REGION_TOLERANCE)
            .unwrap_or_default()
    };
    // Intersect all covering outlines.
    let mut acc: Option<Vec<Vec<GPoint>>> = None;
    for id in covering {
        let polys = outlines(*id);
        if polys.is_empty() {
            return Vec::new();
        }
        acc = Some(match acc {
            None => polys,
            Some(current) => {
                // Bounding box early out (F5): disjoint bounding boxes cannot intersect
                if !polys_intersect_bbox(&current, &polys) {
                    return Vec::new();
                }
                intersect_many(&current, &polys)
            }
        });
        if acc.as_ref().is_some_and(|a| a.is_empty()) {
            return Vec::new();
        }
    }
    let Some(mut acc) = acc else {
        return Vec::new();
    };
    // Subtract every other selected outline.
    for id in session.selection.selected_ids.clone() {
        if covering.contains(&id) {
            continue;
        }
        let other = outlines(id);
        if other.is_empty() {
            continue;
        }
        // Bounding box early out (F5): disjoint other cannot carve anything
        if !polys_intersect_bbox(&acc, &other) {
            continue;
        }
        acc = difference_many(&acc, &other);
        if acc.is_empty() {
            break;
        }
    }
    acc.into_iter()
        .filter(|poly| poly.len() >= 3 && poly_area(poly).abs() >= 1e-6)
        .collect()
}

/// Preview outline for an in-flight drag along the gesture path: merged crossed regions flattened.
fn drag_path_preview(
    bridge: &PetuniaDesignGuiBridge,
    path: &[GPoint],
    p0: GPoint,
    p1: GPoint,
) -> Option<Vec<GPoint>> {
    let segments: Vec<(GPoint, GPoint)> = if path.len() >= 2 {
        path.windows(2).map(|w| (w[0], w[1])).collect()
    } else {
        vec![(p0, p1)]
    };
    let mut signatures: Vec<Vec<ObjectId>> = Vec::new();
    for (seg_start, seg_end) in segments {
        let length = seg_start.distance_to(seg_end);
        let steps = ((length / DRAG_SAMPLE_STEP).ceil() as usize).clamp(1, 32);
        for i in 0..=steps {
            let t = (i as f64) / (steps as f64);
            let pt = GPoint::new(
                seg_start.x + (seg_end.x - seg_start.x) * t,
                seg_start.y + (seg_end.y - seg_start.y) * t,
            );
            let mut covering = covering_set(bridge, pt);
            covering.sort();
            if !covering.is_empty() && !signatures.contains(&covering) {
                signatures.push(covering);
            }
        }
    }
    let mut regions: Vec<Vec<GPoint>> = Vec::new();
    for signature in &signatures {
        regions.extend(region_polygons(bridge, signature));
    }
    let merged = union_all(&regions);
    let flat: Vec<GPoint> = merged.into_iter().flatten().collect();
    if flat.len() >= 2 {
        Some(flat)
    } else {
        None
    }
}

/// Creates one object from region polygons (merge path).
/// Builder clones the first covering style; SmartFill uses the configured fill token.
fn create_region(
    bridge: &mut PetuniaDesignGuiBridge,
    covering: &[ObjectId],
    region: &[Vec<GPoint>],
    mode: BuilderMode,
    fill_token: &str,
) -> Result<ChangeSet, PetuniaError> {
    let path = GPath::from_polygons(region);
    if path.is_empty() {
        return Ok(ChangeSet::empty());
    }
    let Some(rect) = path.bounding_box() else {
        return Ok(ChangeSet::empty());
    };
    let bounds = [
        rect.x0,
        rect.y0,
        rect.width().max(1.0),
        rect.height().max(1.0),
    ];
    let surface_id = bridge
        .session()
        .and_then(|s| s.active_surface())
        .ok_or_else(|| PetuniaError::invalid_input("no active surface for region synthesis"))?;
    let source = covering
        .first()
        .and_then(|id| bridge.session()?.find_object(*id).cloned());
    let new_id = bridge.next_object_id()?;
    let (name, fill, stroke, stroke_width, opacity, appearance) = match (&source, mode) {
        (Some(o), BuilderMode::ShapeBuilder) => (
            format!("{} Region", o.name),
            o.fill.clone(),
            o.stroke.clone(),
            o.stroke_width,
            o.opacity,
            o.appearance.clone(),
        ),
        _ => (
            "Smart Fill".to_string(),
            Some(fill_token.to_string()),
            None,
            1.0,
            1.0,
            None,
        ),
    };
    let mut cmds = petunia_design_application::create_shape_commands(
        surface_id,
        new_id,
        name,
        ShapeKind::Path(path),
        Some(bounds),
        fill,
        stroke.map(|s| (s, stroke_width)),
    );
    if let Some(appearance) = appearance {
        cmds.push(Command::SetAppearance {
            id: new_id,
            appearance: Some(appearance),
        });
    }
    if (opacity - 1.0).abs() > f64::EPSILON {
        cmds.push(Command::SetOpacity {
            id: new_id,
            opacity,
        });
    }
    let changes = bridge.submit_all("Shape builder region", cmds)?;
    bridge.set_selection(vec![new_id]);
    Ok(changes)
}

/// Carves region polygons out of every covering object (Alt path).
/// Fully consumed objects are deleted; the rest keep one path.
fn subtract_region(
    bridge: &mut PetuniaDesignGuiBridge,
    covering: &[ObjectId],
    region: &[Vec<GPoint>],
) -> Result<ChangeSet, PetuniaError> {
    if region.is_empty() {
        return Ok(ChangeSet::empty());
    }
    let mut cmds = Vec::new();
    for id in covering {
        let Some(source) = bridge.session().and_then(|s| s.find_object(*id)).cloned() else {
            continue;
        };
        let base = source.evaluated_path().to_polygons(REGION_TOLERANCE);
        let remaining = difference_many(&base, region);
        let remaining: Vec<Vec<GPoint>> = remaining
            .into_iter()
            .filter(|poly| poly.len() >= 3 && poly_area(poly).abs() >= 1e-6)
            .collect();
        if remaining.is_empty() {
            cmds.push(Command::DeleteObject { id: *id });
            continue;
        }
        // Single-path objects stay single; multi-contour results merge.
        let path = GPath::from_polygons(&remaining);
        let Some(rect) = path.bounding_box() else {
            continue;
        };
        cmds.push(Command::SetShape {
            id: *id,
            shape: Some(ShapeKind::Path(path)),
        });
        cmds.push(Command::SetBounds {
            id: *id,
            bounds: Some([
                rect.x0,
                rect.y0,
                rect.width().max(1.0),
                rect.height().max(1.0),
            ]),
            rotation: source.rotation,
        });
    }
    if cmds.is_empty() {
        return Ok(ChangeSet::empty());
    }
    bridge.submit_all("Shape builder subtract", cmds)
}

/// Intersects two polygon sets in ONE overlay call (F5).
fn intersect_many(a: &[Vec<GPoint>], b: &[Vec<GPoint>]) -> Vec<Vec<GPoint>> {
    use petunia_design_geometry::{boolean_op, BooleanInput};
    if a.is_empty() || b.is_empty() {
        return Vec::new();
    }
    boolean_op(
        &BooleanInput::new(a.to_vec()),
        &BooleanInput::new(b.to_vec()),
        petunia_design_geometry::BooleanOp::Intersection,
    )
    .into_iter()
    .filter(|contour| contour.len() >= 3 && poly_area(contour).abs() >= 1e-6)
    .collect()
}

/// Unions two polygon sets in ONE overlay call (F5).
fn union_polygons(a: &[Vec<GPoint>], b: &[Vec<GPoint>]) -> Vec<Vec<GPoint>> {
    use petunia_design_geometry::{boolean_op, BooleanInput};
    if a.is_empty() {
        return b.to_vec();
    }
    if b.is_empty() {
        return a.to_vec();
    }
    boolean_op(
        &BooleanInput::new(a.to_vec()),
        &BooleanInput::new(b.to_vec()),
        petunia_design_geometry::BooleanOp::Union,
    )
    .into_iter()
    .filter(|contour| contour.len() >= 3 && poly_area(contour).abs() >= 1e-6)
    .collect()
}

/// Unions multiple polygon sets using divide-and-conquer tree reduction (F5).
fn union_all(polys: &[Vec<GPoint>]) -> Vec<Vec<GPoint>> {
    if polys.is_empty() {
        return Vec::new();
    }
    if polys.len() == 1 {
        return polys.to_vec();
    }
    let half = polys.len() / 2;
    let (left, right) = polys.split_at(half);
    union_polygons(&union_all(left), &union_all(right))
}

/// True when bounding boxes of two polygon sets overlap (F5 early-out).
fn polys_intersect_bbox(a: &[Vec<GPoint>], b: &[Vec<GPoint>]) -> bool {
    let (Some(ba), Some(bb)) = (polys_bbox(a), polys_bbox(b)) else {
        return false;
    };
    ba[0] < bb[2] && ba[2] > bb[0] && ba[1] < bb[3] && ba[3] > bb[1]
}

/// Computes envelope `[x0, y0, x1, y1]` over polygon set (F5 early-out).
fn polys_bbox(polys: &[Vec<GPoint>]) -> Option<[f64; 4]> {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut has_points = false;
    for poly in polys {
        for p in poly {
            has_points = true;
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }
    }
    if has_points {
        Some([min_x, min_y, max_x, max_y])
    } else {
        None
    }
}

/// Subtracts polygon set `b` from `a` in ONE overlay call.
/// Per-contour pairwise subtraction would break hole semantics (holes are
/// sibling contours; each step must see the whole shape at once).
fn difference_many(a: &[Vec<GPoint>], b: &[Vec<GPoint>]) -> Vec<Vec<GPoint>> {
    use petunia_design_geometry::{boolean_op, BooleanInput};
    if a.is_empty() {
        return Vec::new();
    }
    if b.is_empty() {
        return a.to_vec();
    }
    boolean_op(
        &BooleanInput::new(a.to_vec()),
        &BooleanInput::new(b.to_vec()),
        petunia_design_geometry::BooleanOp::Difference,
    )
    .into_iter()
    .filter(|contour| contour.len() >= 3 && poly_area(contour).abs() >= 1e-6)
    .collect()
}

/// Flood scope: the selection when non-empty, else every visible
/// unlocked object on the active surface (Corel Smart Fill scope).
fn flood_scope_ids(bridge: &PetuniaDesignGuiBridge) -> Vec<ObjectId> {
    let Some(session) = bridge.session() else {
        return Vec::new();
    };
    if !session.selection.selected_ids.is_empty() {
        return session.selection.selected_ids.clone();
    }
    let Some(surface_id) = session.active_surface() else {
        return Vec::new();
    };
    let Ok(surface) = session.surface(surface_id) else {
        return Vec::new();
    };
    surface
        .objects()
        .iter()
        .filter(|obj| obj.visible && !obj.locked)
        .map(|obj| obj.id)
        .collect()
}

/// Obstacle polygons for flood: evaluated outlines; open paths buffer by
/// half stroke width into closed bands so strokes bound faces too.
fn obstacle_polygons(bridge: &PetuniaDesignGuiBridge, ids: &[ObjectId]) -> Vec<Vec<GPoint>> {
    use petunia_design_geometry::{offset_path, OffsetCap, OffsetJoin};
    let Some(session) = bridge.session() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for id in ids {
        let Some(obj) = session.find_object(*id) else {
            continue;
        };
        if !obj.visible || obj.locked {
            continue;
        }
        let path = match obj.shape.as_ref() {
            Some(ShapeKind::Path(path)) => path.clone(),
            Some(_) => obj.to_path(),
            None => continue,
        };
        if path.is_empty() {
            continue;
        }
        if path
            .verbs
            .contains(&petunia_design_geometry::PathVerb::Close)
        {
            out.extend(path.to_polygons(REGION_TOLERANCE));
        } else {
            // Open stroke: band it so it bounds the flood face.
            let half = (obj.stroke_width.max(1.0)) / 2.0;
            match offset_path(&path, half, OffsetJoin::Round, OffsetCap::Round) {
                Some(band) => out.extend(band.to_polygons(REGION_TOLERANCE)),
                None => out.extend(path.to_polygons(REGION_TOLERANCE)),
            }
        }
    }
    out.into_iter().filter(|poly| poly.len() >= 3).collect()
}

/// Floods the bounded negative-space face containing `pt`.
/// Unbounded faces (touching the frame) are a NoOp: filling infinity
/// would create garbage, and Corel asks for a closed area too.
fn flood_empty_face(
    bridge: &mut PetuniaDesignGuiBridge,
    pt: GPoint,
    fill_token: &str,
) -> Result<ChangeSet, PetuniaError> {
    let Some(face) = flood_face_polygons(bridge, pt) else {
        return Ok(ChangeSet::empty());
    };
    create_region(bridge, &[], &face, BuilderMode::SmartFill, fill_token)
}

/// Computes the bounded face containing `pt`, if it is enclosed.
/// Returns the face components, or `None` for unbounded/missing faces.
fn flood_face_polygons(bridge: &PetuniaDesignGuiBridge, pt: GPoint) -> Option<Vec<Vec<GPoint>>> {
    let ids = flood_scope_ids(bridge);
    if ids.is_empty() {
        return None;
    }
    let obstacles = obstacle_polygons(bridge, &ids);
    if obstacles.is_empty() {
        return None;
    }
    // Frame: obstacle bounds expanded by margin, grown to contain the click.
    let margin = 50.0;
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for poly in &obstacles {
        for p in poly {
            x0 = x0.min(p.x);
            y0 = y0.min(p.y);
            x1 = x1.max(p.x);
            y1 = y1.max(p.y);
        }
    }
    x0 = (x0 - margin).min(pt.x - margin);
    y0 = (y0 - margin).min(pt.y - margin);
    x1 = (x1 + margin).max(pt.x + margin);
    y1 = (y1 + margin).max(pt.y + margin);
    let frame = vec![
        GPoint::new(x0, y0),
        GPoint::new(x1, y0),
        GPoint::new(x1, y1),
        GPoint::new(x0, y1),
    ];
    let remaining = difference_many(&[frame], &obstacles);
    if remaining.is_empty() {
        return None;
    }
    // The face containing the click. Difference output is a flat contour
    // list (holes are siblings, not nested): several contours may contain
    // the point, so the smallest wins — the outer boundary always contains
    // its holes geometrically.
    let face = remaining
        .into_iter()
        .filter(|poly| point_in_poly(pt, poly))
        .min_by(|a, b| {
            poly_area(a)
                .abs()
                .partial_cmp(&poly_area(b).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;
    if face.len() < 3 || poly_area(&face).abs() < 1e-6 {
        return None;
    }
    // Unbounded faces touch the frame border: refuse to fill infinity.
    let touches = face.iter().any(|p| {
        (p.x - x0).abs() < 0.5
            || (p.x - x1).abs() < 0.5
            || (p.y - y0).abs() < 0.5
            || (p.y - y1).abs() < 0.5
    });
    if touches {
        return None;
    }
    Some(vec![face])
}

/// Even-odd point-in-polygon.
fn point_in_poly(pt: GPoint, poly: &[GPoint]) -> bool {
    let mut inside = false;
    let n = poly.len();
    if n < 3 {
        return false;
    }
    let mut j = n - 1;
    for i in 0..n {
        let pi = poly[i];
        let pj = poly[j];
        if (pi.y > pt.y) != (pj.y > pt.y)
            && pt.x < (pj.x - pi.x) * (pt.y - pi.y) / (pj.y - pi.y) + pi.x
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}
/// Shoelace area of one contour.
fn poly_area(contour: &[GPoint]) -> f64 {
    if contour.len() < 3 {
        return 0.0;
    }
    let mut sum: f64 = contour
        .windows(2)
        .map(|w| w[0].x * w[1].y - w[1].x * w[0].y)
        .sum();
    let first = contour[0];
    let last = contour[contour.len() - 1];
    sum += last.x * first.y - first.x * last.y;
    sum / 2.0
}
