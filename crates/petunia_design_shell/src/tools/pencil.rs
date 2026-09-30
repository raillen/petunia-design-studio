//! Freehand path sketching and curve fitting tool (10.2).
//!
//! Batch 3 (V1 + Sculpt): auto-close, Shift straight lines, three-level
//! fidelity, and reshape/extend of the selected open path by drawing over it.
//! Live stabilization stays future work (PENDING_PENCIL_STABILIZER).

use petunia_design_document::{ChangeSet, ShapeKind};
use petunia_design_foundation::{ObjectId, PetuniaError};
use petunia_design_geometry::{GPath, GPoint, PathVerb};

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{CanvasOverlays, CursorAffordance, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase,
};

/// Screen-space hit radius for sculpt targeting and auto-close.
const SCULPT_HIT_PX: f64 = 12.0;
/// Minimum samples for a committable stroke.
const MIN_SAMPLES: usize = 2;

/// Freehand stroke stabilization settings (StreamLine + velocity compensation, Dossier V1 §3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PencilStabilizer {
    /// Smoothing strength from 0.0 (raw input) to 1.0 (heavy stabilization).
    pub weight: f64,
    /// Whether velocity-weighted compensation is enabled.
    pub velocity_compensation: bool,
}

impl Default for PencilStabilizer {
    fn default() -> Self {
        Self {
            weight: 0.35,
            velocity_compensation: true,
        }
    }
}

impl PencilStabilizer {
    /// Disables live stabilization (raw pointer tracking).
    #[must_use]
    pub const fn off() -> Self {
        Self {
            weight: 0.0,
            velocity_compensation: false,
        }
    }

    /// Creates an active stabilizer with a given weight (`0.0..=1.0`).
    #[must_use]
    pub fn new(weight: f64, velocity_compensation: bool) -> Self {
        Self {
            weight: weight.clamp(0.0, 1.0),
            velocity_compensation,
        }
    }

    /// Filters an in-flight pointer move sample against the previous stabilized anchor.
    #[must_use]
    pub fn filter_point(&self, raw: GPoint, previous: GPoint) -> GPoint {
        if self.weight <= 0.0 {
            return raw;
        }

        let dx = raw.x - previous.x;
        let dy = raw.y - previous.y;
        let dist = (dx * dx + dy * dy).sqrt();

        // Baseline alpha from weight (0.0 weight -> alpha 1.0; 1.0 weight -> alpha 0.15)
        let base_alpha = 1.0 - (self.weight * 0.85);

        // Velocity compensation: fast strokes have higher alpha so they track crisply
        let alpha = if self.velocity_compensation {
            let speed_boost = (dist / 80.0).clamp(0.0, 0.4);
            (base_alpha + speed_boost).min(1.0)
        } else {
            base_alpha
        };

        GPoint::new(previous.x + dx * alpha, previous.y + dy * alpha)
    }
}

/// Freehand fidelity: how closely the fit follows the hand.
/// Illustrator-style slider compressed to three deterministic levels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PencilFidelity {
    /// Follows the hand: light simplification, no smoothing pass.
    Precise,
    /// Default balance (legacy `smooth_samples(1.5, 1)` behavior).
    #[default]
    Balanced,
    /// Heavy simplification plus two smoothing passes.
    Smooth,
}

impl PencilFidelity {
    /// `(simplify epsilon, chaikin iterations)` for the shared pipeline.
    fn pipeline(self) -> (f64, usize) {
        match self {
            Self::Precise => (0.5, 0),
            Self::Balanced => (1.5, 1),
            Self::Smooth => (3.0, 2),
        }
    }
}

/// Freehand pencil tool capturing raw pointer gestures and committing smoothed paths (10.2).
#[derive(Clone, Debug, Default)]
pub struct PencilTool {
    sampled_points: Vec<GPoint>,
    fidelity: PencilFidelity,
    stabilizer: PencilStabilizer,
    close_threshold_px: f64,
    straight: bool,
    sculpt_target: Option<ObjectId>,
    sculpt_baseline: Vec<GPoint>,
}

impl PencilTool {
    /// Creates a fresh pencil tool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sampled_points: Vec::new(),
            fidelity: PencilFidelity::Balanced,
            stabilizer: PencilStabilizer::default(),
            close_threshold_px: SCULPT_HIT_PX,
            straight: false,
            sculpt_target: None,
            sculpt_baseline: Vec::new(),
        }
    }

    /// Cancels active stroke gesture.
    pub fn cancel(&mut self) {
        self.sampled_points.clear();
        self.sculpt_target = None;
        self.sculpt_baseline.clear();
        self.straight = false;
    }

    /// Current fidelity level.
    #[must_use]
    pub fn fidelity(&self) -> PencilFidelity {
        self.fidelity
    }

    /// Sets the fidelity level, canceling any in-flight stroke.
    pub fn set_fidelity(&mut self, fidelity: PencilFidelity) {
        if self.fidelity != fidelity {
            self.cancel();
            self.fidelity = fidelity;
        }
    }

    /// Current stroke stabilizer settings.
    #[must_use]
    pub fn stabilizer(&self) -> PencilStabilizer {
        self.stabilizer
    }

    /// Updates stroke stabilizer settings.
    pub fn set_stabilizer(&mut self, stabilizer: PencilStabilizer) {
        self.stabilizer = stabilizer;
    }

    /// Raw in-flight samples (read-only, for tests and HUD).
    #[must_use]
    pub fn samples(&self) -> &[GPoint] {
        &self.sampled_points
    }

    /// Selected path being sculpted, if the gesture started on one.
    #[must_use]
    pub fn sculpt_target(&self) -> Option<ObjectId> {
        self.sculpt_target
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
            PointerPhase::Down => {
                if event.button != PointerButton::Primary {
                    return Ok(ChangeSet::empty());
                }
                let mut pt = event.doc_pos;
                if !event.modifiers.disable_snap {
                    pt = snap.snap_point(pt, camera, &[]).point;
                }
                self.sampled_points.clear();
                self.sampled_points.push(pt);
                // Shift draws a straight segment (first to last sample).
                self.straight = event.modifiers.constrain;
                // Sculpt arms when starting on a selected open path (10.2):
                // endpoints extend it, interior redraws reshape it.
                let (target, baseline) =
                    find_sculpt_target(pt, bridge, camera, self.close_threshold_px);
                self.sculpt_target = target;
                self.sculpt_baseline = baseline;
                Ok(ChangeSet::empty())
            }
            PointerPhase::Move => {
                if !self.sampled_points.is_empty() {
                    let mut pt = event.doc_pos;
                    if !event.modifiers.disable_snap {
                        pt = snap.snap_point(pt, camera, &[]).point;
                    }
                    if let Some(last) = self.sampled_points.last() {
                        let stabilized = self.stabilizer.filter_point(pt, *last);
                        let dx = stabilized.x - last.x;
                        let dy = stabilized.y - last.y;
                        if (dx * dx + dy * dy).sqrt() >= 2.0 {
                            self.sampled_points.push(stabilized);
                        }
                    }
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Up => {
                if self.sampled_points.len() < MIN_SAMPLES {
                    self.cancel();
                    return Ok(ChangeSet::empty());
                }
                if let Some(last) = self.sampled_points.last() {
                    let mut final_pt = event.doc_pos;
                    if !event.modifiers.disable_snap {
                        final_pt = snap.snap_point(final_pt, camera, &[]).point;
                    }
                    let dx = final_pt.x - last.x;
                    let dy = final_pt.y - last.y;
                    if (dx * dx + dy * dy).sqrt() >= 2.0 {
                        self.sampled_points.push(final_pt);
                    }
                }
                let pts = std::mem::take(&mut self.sampled_points);
                let straight = self.straight;
                let sculpt = self.sculpt_target.take();
                let baseline = std::mem::take(&mut self.sculpt_baseline);
                self.straight = false;
                self.commit_stroke(bridge, camera, &pts, straight, sculpt, &baseline)
            }
            PointerPhase::Cancel => {
                self.cancel();
                Ok(ChangeSet::empty())
            }
        }
    }

    /// Commits a finished stroke: sculpt replace, straight line, or freehand fit.
    fn commit_stroke(
        &mut self,
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
        pts: &[GPoint],
        straight: bool,
        sculpt: Option<ObjectId>,
        baseline: &[GPoint],
    ) -> Result<ChangeSet, PetuniaError> {
        if let Some(target) = sculpt {
            return self.commit_sculpt(bridge, camera, target, baseline, pts);
        }
        let active_surface = bridge
            .session()
            .and_then(|s| s.active_surface())
            .ok_or_else(|| PetuniaError::invalid_input("no active surface for path creation"))?;

        let path = if straight {
            straight_path(pts)
        } else {
            fit_samples(pts, self.fidelity)?
        };
        let close = !straight && auto_close(pts, self.close_threshold_px / camera.zoom.max(1e-6));
        let path = close_path(path, close);

        let bounds = path
            .bounding_box()
            .map(|r| [r.x0, r.y0, r.width().max(1.0), r.height().max(1.0)])
            .unwrap_or([pts[0].x, pts[0].y, 10.0, 10.0]);

        let obj_id = bridge.next_object_id()?;

        // One gesture, one undo entry (F-01).
        let changes = bridge.submit_all(
            "Freehand path",
            petunia_design_application::create_shape_commands(
                active_surface,
                obj_id,
                "Freehand Path",
                petunia_design_document::ShapeKind::Path(path),
                Some(bounds),
                None,
                Some((
                    petunia_design_document::shape_factory::DEFAULT_PATH_STROKE.to_string(),
                    petunia_design_document::shape_factory::DEFAULT_PENCIL_STROKE_WIDTH,
                )),
            ),
        )?;

        bridge.set_selection(vec![obj_id]);
        Ok(changes)
    }

    /// Replaces a sculpted path: splice gesture samples into its baseline,
    /// refit, and commit as one undo entry on the same object.
    fn commit_sculpt(
        &mut self,
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
        target: ObjectId,
        baseline: &[GPoint],
        pts: &[GPoint],
    ) -> Result<ChangeSet, PetuniaError> {
        if baseline.is_empty() {
            return Ok(ChangeSet::empty());
        }
        let merged = splice_baseline(baseline, pts);
        if merged.len() < MIN_SAMPLES {
            return Ok(ChangeSet::empty());
        }
        let path = fit_samples(&merged, self.fidelity)?;
        let close = auto_close(&merged, self.close_threshold_px / camera.zoom.max(1e-6));
        let path = close_path(path, close);
        let bounds = path
            .bounding_box()
            .map(|r| [r.x0, r.y0, r.width().max(1.0), r.height().max(1.0)])
            .unwrap_or([merged[0].x, merged[0].y, 10.0, 10.0]);
        let changes = bridge.submit_all(
            "Sculpt path",
            vec![
                petunia_design_application::Command::SetShape {
                    id: target,
                    shape: Some(ShapeKind::Path(path)),
                },
                petunia_design_application::Command::SetBounds {
                    id: target,
                    bounds: Some(bounds),
                    rotation: 0.0,
                },
            ],
        )?;
        bridge.set_selection(vec![target]);
        Ok(changes)
    }

    /// Resolves live preview overlays for active freehand drawing.
    #[must_use]
    pub fn overlays(&self) -> CanvasOverlays {
        let mut overlays = CanvasOverlays {
            cursor: CursorAffordance::Crosshair,
            ..Default::default()
        };
        if self.sampled_points.len() >= MIN_SAMPLES {
            if self.straight {
                let first = self.sampled_points[0];
                let last = *self.sampled_points.last().unwrap();
                overlays.pen_preview = Some(vec![first, last]);
            } else if self.sampled_points.len() >= 3 {
                if let Ok(fitted) = fit_samples(&self.sampled_points, self.fidelity) {
                    let is_closed = auto_close(&self.sampled_points, self.close_threshold_px);
                    overlays.path_preview = Some(close_path(fitted, is_closed));
                } else {
                    overlays.pen_preview = Some(self.sampled_points.clone());
                }
            } else {
                overlays.pen_preview = Some(self.sampled_points.clone());
            }
        }
        overlays
    }
}

/// Fits samples through the shared freehand pipeline at a fidelity level.
fn fit_samples(pts: &[GPoint], fidelity: PencilFidelity) -> Result<GPath, PetuniaError> {
    let (epsilon, iterations) = fidelity.pipeline();
    let smoothed = petunia_design_geometry::smooth_samples(pts, epsilon, iterations);
    petunia_design_geometry::fit_midpoint_quads(&smoothed).map_err(PetuniaError::invalid_input)
}

/// Builds a straight `MoveTo`/`LineTo` path from first to last sample.
fn straight_path(pts: &[GPoint]) -> GPath {
    let mut path = GPath::new();
    if let (Some(first), Some(last)) = (pts.first(), pts.last()) {
        let _ = path.push(PathVerb::MoveTo(*first));
        if last.distance_to(*first) > 1e-9 {
            let _ = path.push(PathVerb::LineTo(*last));
        }
    }
    path
}

/// True when the stroke ends near its start: auto-close (Illustrator/Affinity).
fn auto_close(pts: &[GPoint], threshold_doc: f64) -> bool {
    pts.len() >= 3
        && pts
            .last()
            .is_some_and(|last| last.distance_to(pts[0]) <= threshold_doc)
}

/// Appends `Close` to an open fitted path when requested.
fn close_path(mut path: GPath, close: bool) -> GPath {
    if close && !path.verbs.contains(&PathVerb::Close) && !path.is_empty() {
        let _ = path.push(PathVerb::Close);
    }
    path
}

/// Splices gesture samples into a sculpt baseline between the nearest
/// vertices to the gesture start/end. Endpoint touches extend the path.
fn splice_baseline(baseline: &[GPoint], pts: &[GPoint]) -> Vec<GPoint> {
    let Some(first) = pts.first() else {
        return baseline.to_vec();
    };
    let Some(last) = pts.last() else {
        return baseline.to_vec();
    };
    let (mut i0, mut i1) = (
        nearest_vertex(baseline, *first),
        nearest_vertex(baseline, *last),
    );
    if i1 < i0 {
        std::mem::swap(&mut i0, &mut i1);
    }
    let mut merged = Vec::with_capacity(baseline.len() + pts.len());
    merged.extend_from_slice(&baseline[..=i0.min(baseline.len().saturating_sub(1))]);
    merged.extend_from_slice(pts);
    if i1 + 1 < baseline.len() {
        merged.extend_from_slice(&baseline[i1 + 1..]);
    }
    merged
}

/// Index of the baseline vertex nearest `pt`.
fn nearest_vertex(baseline: &[GPoint], pt: GPoint) -> usize {
    baseline
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            a.distance_to(pt)
                .partial_cmp(&b.distance_to(pt))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map_or(0, |(i, _)| i)
}

/// Shortest distance from `pt` to a polyline (segment-aware, so long
/// straight runs match along their whole length, not just at vertices).
fn dist_to_polyline(pt: GPoint, baseline: &[GPoint]) -> f64 {
    if baseline.len() < 2 {
        return baseline
            .first()
            .map_or(f64::INFINITY, |p| p.distance_to(pt));
    }
    baseline
        .windows(2)
        .map(|w| dist_to_segment(pt, w[0], w[1]))
        .fold(f64::INFINITY, f64::min)
}

/// Shortest distance from `pt` to segment `a->b`.
fn dist_to_segment(pt: GPoint, a: GPoint, b: GPoint) -> f64 {
    let abx = b.x - a.x;
    let aby = b.y - a.y;
    let len2 = abx * abx + aby * aby;
    if len2 < 1e-12 {
        return pt.distance_to(a);
    }
    let t = ((pt.x - a.x) * abx + (pt.y - a.y) * aby) / len2;
    let t = t.clamp(0.0, 1.0);
    pt.distance_to(GPoint::new(a.x + abx * t, a.y + aby * t))
}

/// Finds a selected open path near `pt` for sculpting.
/// Returns `(object, flattened baseline)`. Closed paths stay future work.
fn find_sculpt_target(
    pt: GPoint,
    bridge: &PetuniaDesignGuiBridge,
    camera: &ViewportCamera,
    threshold_px: f64,
) -> (Option<ObjectId>, Vec<GPoint>) {
    let session = match bridge.session() {
        Some(session) => session,
        None => return (None, Vec::new()),
    };
    let threshold_doc = threshold_px / camera.zoom.max(1e-6);
    // Topmost selected first.
    let mut selected: Vec<ObjectId> = session.selection.selected_ids.clone();
    selected.reverse();
    for id in selected {
        let Some(obj) = session.find_object(id) else {
            continue;
        };
        if !obj.visible || obj.locked {
            continue;
        }
        let Some(ShapeKind::Path(path)) = obj.shape.as_ref() else {
            continue;
        };
        if path.verbs.contains(&PathVerb::Close) {
            continue;
        }
        let baseline: Vec<GPoint> = path.to_polygons(0.5).into_iter().flatten().collect();
        if baseline.len() < MIN_SAMPLES {
            continue;
        }
        if dist_to_polyline(pt, &baseline) <= threshold_doc {
            return (Some(id), baseline);
        }
    }
    (None, Vec::new())
}

#[cfg(test)]
mod pencil_tool_tests {
    use super::*;

    fn jitter(n: usize) -> Vec<GPoint> {
        (0..n)
            .map(|i| GPoint::new(i as f64 * 5.0, if i % 2 == 0 { 1.5 } else { -1.5 }))
            .collect()
    }

    #[test]
    fn fidelity_smooth_removes_hand_jitter() {
        // Small high-frequency jitter: Precise preserves it, Smooth collapses
        // it. (Chaikin upsampling means Balanced sits between by construction,
        // so only the endpoints order strictly.)
        let pts = jitter(41);
        let verbs = |fidelity| {
            fit_samples(&pts, fidelity)
                .map(|path| path.verbs.len())
                .unwrap_or(0)
        };
        assert!(verbs(PencilFidelity::Smooth) < verbs(PencilFidelity::Precise));
    }

    #[test]
    fn fidelity_fit_is_deterministic() {
        let pts = jitter(41);
        for fidelity in [
            PencilFidelity::Precise,
            PencilFidelity::Balanced,
            PencilFidelity::Smooth,
        ] {
            let a = fit_samples(&pts, fidelity).unwrap();
            let b = fit_samples(&pts, fidelity).unwrap();
            assert_eq!(a.verbs, b.verbs);
        }
    }

    #[test]
    fn straight_path_is_two_verbs() {
        let pts = jitter(11);
        let path = straight_path(&pts);
        assert_eq!(path.verbs.len(), 2);
        assert!(matches!(path.verbs[0], PathVerb::MoveTo(_)));
        assert!(matches!(path.verbs[1], PathVerb::LineTo(_)));
    }

    #[test]
    fn splice_extends_at_endpoints() {
        let baseline: Vec<GPoint> = (0..5).map(|i| GPoint::new(i as f64 * 10.0, 0.0)).collect();
        let extension = vec![GPoint::new(40.0, 0.0), GPoint::new(50.0, 5.0)];
        let merged = splice_baseline(&baseline, &extension);
        assert_eq!(merged.first(), Some(&GPoint::new(0.0, 0.0)));
        assert_eq!(merged.last(), Some(&GPoint::new(50.0, 5.0)));
        assert!(merged.len() > baseline.len());
    }

    #[test]
    fn stabilizer_filters_jitter_via_moving_average() {
        let raw_off = PencilStabilizer::off();
        let prev = GPoint::new(0.0, 0.0);
        let raw_pt = GPoint::new(10.0, 10.0);
        assert_eq!(raw_off.filter_point(raw_pt, prev), raw_pt);

        let active = PencilStabilizer::new(0.5, false);
        let smoothed = active.filter_point(raw_pt, prev);
        // Alpha is 1.0 - 0.5 * 0.85 = 0.575
        // Smoothed x and y should be 5.75, which is between prev (0.0) and raw (10.0)
        assert!((smoothed.x - 5.75).abs() < 1e-5);
        assert!((smoothed.y - 5.75).abs() < 1e-5);
    }

    #[test]
    fn stabilizer_velocity_compensation_increases_responsiveness() {
        let prev = GPoint::new(0.0, 0.0);
        let slow_pt = GPoint::new(5.0, 0.0);
        let fast_pt = GPoint::new(80.0, 0.0);

        let stab_with_comp = PencilStabilizer::new(0.6, true);
        let slow_filtered = stab_with_comp.filter_point(slow_pt, prev);
        let fast_filtered = stab_with_comp.filter_point(fast_pt, prev);

        let slow_ratio = slow_filtered.x / slow_pt.x;
        let fast_ratio = fast_filtered.x / fast_pt.x;

        // Fast gesture has higher tracking alpha/ratio than slow gesture
        assert!(fast_ratio > slow_ratio);
    }
}
