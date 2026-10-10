//! Snapping: intention in, correction out.
//!
//! Providers (guides, grids, bounds, nodes) emit candidates in one
//! vocabulary; [`rank_candidates`] picks deterministically and
//! [`solve`] combines per-axis corrections. Snapping never mutates
//! the document, and hysteresis state stays in the session.

use petunia_core::{AffineGridSpec, Guide, GuideAxis, ObjectId, PageId, Point, Rect, Tolerance};
use serde::{Deserialize, Serialize};

/// A 2D correction to the proposed transform.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct TransformDelta {
    pub dx: f64,
    pub dy: f64,
}

impl TransformDelta {
    /// Combine two corrections.
    #[must_use]
    pub fn combined(self, other: Self) -> Self {
        Self {
            dx: self.dx + other.dx,
            dy: self.dy + other.dy,
        }
    }
}

/// Which providers participate in one request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SnapProvider {
    Guide,
    Grid,
    ObjectBounds,
    PathNode,
}

/// Semantic priority, lowest rank first. Declaration order is the
/// ranking table: guides and exact nodes outrank grids, and spacing
/// suggestions come last. Configured per request, never scattered
/// as magic numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SnapPriority {
    Guide,
    PathNode,
    Intersection,
    EdgeCenter,
    Grid,
    Spacing,
}

/// What a candidate snaps onto.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SnapTarget {
    Guide(petunia_core::GuideId),
    GridPoint(Point),
    ObjectBounds(ObjectId),
    PathNode(ObjectId, usize),
}

/// Degrees of freedom left by an active constraint (Shift-style).
/// Constraints apply before candidate ranking so results never jump
/// off-axis after a free snap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SnapConstraint {
    #[default]
    Free,
    /// Movement along X: corrections keep `dy == 0`.
    Horizontal,
    /// Movement along Y: corrections keep `dx == 0`.
    Vertical,
}

impl SnapConstraint {
    /// True when the correction respects the constraint.
    #[must_use]
    pub fn allows(self, correction: TransformDelta) -> bool {
        match self {
            Self::Free => true,
            Self::Horizontal => correction.dy == 0.0,
            Self::Vertical => correction.dx == 0.0,
        }
    }
}

/// Stable identity of a candidate source for deterministic tiebreaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapSourceId {
    pub provider: SnapProvider,
    pub key: u64,
}

/// One possible correction, not a decision. `distance_px` is always
/// screen-space: providers multiply document distances by the view
/// scale at generation time.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SnapCandidate {
    pub target: SnapTarget,
    pub constraint: SnapConstraint,
    pub correction: TransformDelta,
    pub distance_px: f64,
    pub priority: SnapPriority,
    pub source: SnapSourceId,
}

/// Request knobs: enabled providers, screen tolerance and the
/// hysteresis band. Numbers travel with the request so tests and
/// sessions share one policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapSettings {
    pub enabled: bool,
    pub tolerance_px: f64,
    pub acquire_px: f64,
    pub release_px: f64,
    pub providers: Vec<SnapProvider>,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            tolerance_px: 8.0,
            acquire_px: 8.0,
            release_px: 11.0,
            providers: vec![
                SnapProvider::Guide,
                SnapProvider::Grid,
                SnapProvider::ObjectBounds,
                SnapProvider::PathNode,
            ],
        }
    }
}

/// Geometry being dragged: excluded objects (no self-snap) plus the
/// anchor points that seek targets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MovingGeometry {
    pub excluded: Vec<ObjectId>,
    pub anchors: Vec<Point>,
}

/// One snap evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapRequest {
    pub page: PageId,
    pub moving: MovingGeometry,
    pub proposed: TransformDelta,
    pub constraint: SnapConstraint,
    pub settings: SnapSettings,
    pub view_scale: f64,
    pub previous: Option<SnapLatch>,
}

/// Hysteresis latch: the held target survives until the pointer
/// leaves the release band. Session state, never PTND.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SnapLatch {
    pub target: SnapTarget,
    pub priority: SnapPriority,
}

/// One committed match.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SnapMatch {
    pub target: SnapTarget,
    pub correction: TransformDelta,
    pub distance_px: f64,
}

/// Guide line visual for the renderer; style stays with Render/UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuideVisual {
    pub from: Point,
    pub to: Point,
    pub label: String,
}

/// The solved correction plus matches, visuals and the next latch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapResult {
    pub corrected: TransformDelta,
    pub matches: Vec<SnapMatch>,
    pub visuals: Vec<GuideVisual>,
    pub latch: Option<SnapLatch>,
}

/// Screen-space scale guard shared by providers: without a valid
/// scale, pixel distances are meaningless and no candidate is made.
fn check_scale(view_scale: f64) -> bool {
    view_scale.is_finite() && view_scale > 0.0
}

fn stable_uuid_key(id: u128) -> u64 {
    (id as u64) ^ ((id >> 64) as u64)
}

/// Guide candidates: anchors within tolerance of an axis line snap
/// along that axis only.
#[must_use]
pub fn guide_candidates(
    guides: &[Guide],
    anchors: &[Point],
    tolerance: Tolerance,
    view_scale: f64,
) -> Vec<SnapCandidate> {
    if !check_scale(view_scale) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for guide in guides {
        for (anchor_index, anchor) in anchors.iter().enumerate() {
            let (delta, distance) = match guide.axis {
                GuideAxis::Vertical => {
                    (guide.position - anchor.x, (guide.position - anchor.x).abs())
                }
                GuideAxis::Horizontal => {
                    (guide.position - anchor.y, (guide.position - anchor.y).abs())
                }
            };
            if distance > tolerance.0 {
                continue;
            }
            let correction = match guide.axis {
                GuideAxis::Vertical => TransformDelta { dx: delta, dy: 0.0 },
                GuideAxis::Horizontal => TransformDelta { dx: 0.0, dy: delta },
            };
            out.push(SnapCandidate {
                target: SnapTarget::Guide(guide.id),
                constraint: SnapConstraint::Free,
                correction,
                distance_px: distance * view_scale,
                priority: SnapPriority::Guide,
                source: SnapSourceId {
                    provider: SnapProvider::Guide,
                    key: stable_uuid_key(guide.id.as_uuid().as_u128()) ^ anchor_index as u64,
                },
            });
        }
    }
    out
}

/// Affine lattice candidates: convert the anchor into basis
/// coordinates, round `i`/`j`, convert back. Shared by Cartesian,
/// isometric and axonometric presets through their basis vectors.
#[must_use]
pub fn affine_grid_candidates(
    spec: &AffineGridSpec,
    origin: Point,
    anchors: &[Point],
    tolerance: Tolerance,
    view_scale: f64,
) -> Vec<SnapCandidate> {
    if !check_scale(view_scale) {
        return Vec::new();
    }
    let det = spec.basis_u.dx * spec.basis_v.dy - spec.basis_u.dy * spec.basis_v.dx;
    if det == 0.0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (anchor_index, anchor) in anchors.iter().enumerate() {
        let relative = Point::new(anchor.x - origin.x, anchor.y - origin.y);
        let i = (relative.x * spec.basis_v.dy - relative.y * spec.basis_v.dx) / det;
        let j = (spec.basis_u.dx * relative.y - spec.basis_u.dy * relative.x) / det;
        let snapped = Point::new(
            origin.x + i.round() * spec.basis_u.dx + j.round() * spec.basis_v.dx,
            origin.y + i.round() * spec.basis_u.dy + j.round() * spec.basis_v.dy,
        );
        let correction = TransformDelta {
            dx: snapped.x - anchor.x,
            dy: snapped.y - anchor.y,
        };
        let distance = correction.dx.hypot(correction.dy);
        if distance > tolerance.0 {
            continue;
        }
        out.push(SnapCandidate {
            target: SnapTarget::GridPoint(snapped),
            constraint: SnapConstraint::Free,
            correction,
            distance_px: distance * view_scale,
            priority: SnapPriority::Grid,
            source: SnapSourceId {
                provider: SnapProvider::Grid,
                key: anchor_index as u64,
            },
        });
    }
    out
}

/// Edge and center candidates of object bounds, excluding the moving
/// selection itself.
#[must_use]
pub fn bounds_candidates(
    objects: &[(ObjectId, Rect)],
    excluded: &[ObjectId],
    anchors: &[Point],
    tolerance: Tolerance,
    view_scale: f64,
) -> Vec<SnapCandidate> {
    if !check_scale(view_scale) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (id, bounds) in objects {
        if excluded.contains(id) {
            continue;
        }
        let edges_x = [
            bounds.x,
            bounds.x + bounds.width / 2.0,
            bounds.x + bounds.width,
        ];
        let edges_y = [
            bounds.y,
            bounds.y + bounds.height / 2.0,
            bounds.y + bounds.height,
        ];
        for (anchor_index, anchor) in anchors.iter().enumerate() {
            for (edge_index, edge) in edges_x.iter().enumerate() {
                let delta = edge - anchor.x;
                if delta.abs() <= tolerance.0 {
                    out.push(SnapCandidate {
                        target: SnapTarget::ObjectBounds(*id),
                        constraint: SnapConstraint::Free,
                        correction: TransformDelta { dx: delta, dy: 0.0 },
                        distance_px: delta.abs() * view_scale,
                        priority: SnapPriority::EdgeCenter,
                        source: SnapSourceId {
                            provider: SnapProvider::ObjectBounds,
                            key: stable_uuid_key(id.as_uuid().as_u128())
                                | ((anchor_index as u64) << 20)
                                | edge_index as u64,
                        },
                    });
                }
            }
            for (edge_index, edge) in edges_y.iter().enumerate() {
                let delta = edge - anchor.y;
                if delta.abs() <= tolerance.0 {
                    out.push(SnapCandidate {
                        target: SnapTarget::ObjectBounds(*id),
                        constraint: SnapConstraint::Free,
                        correction: TransformDelta { dx: 0.0, dy: delta },
                        distance_px: delta.abs() * view_scale,
                        priority: SnapPriority::EdgeCenter,
                        source: SnapSourceId {
                            provider: SnapProvider::ObjectBounds,
                            key: stable_uuid_key(id.as_uuid().as_u128())
                                | ((anchor_index as u64) << 20)
                                | (edge_index as u64 + 8),
                        },
                    });
                }
            }
        }
    }
    out
}

/// Deterministic lexicographic ranking over one request:
///
/// 1. enabled providers and request constraint;
/// 2. acquire band for fresh targets (the latch path below bypasses
///    it inside the release band);
/// 3. semantic priority;
/// 4. smaller screen distance;
/// 5. stable source key.
#[must_use]
pub fn rank_candidates<'a>(
    candidates: &'a [SnapCandidate],
    request: &SnapRequest,
) -> Option<&'a SnapCandidate> {
    candidates
        .iter()
        .filter(|candidate| eligible(candidate, request))
        .min_by(|a, b| compare(a, b, request))
}

fn held(candidate: &SnapCandidate, request: &SnapRequest) -> bool {
    request
        .previous
        .is_some_and(|latch| latch.target == candidate.target)
}

fn eligible(candidate: &SnapCandidate, request: &SnapRequest) -> bool {
    let settings = &request.settings;
    if !settings.enabled
        || !check_scale(request.view_scale)
        || !settings.acquire_px.is_finite()
        || settings.acquire_px < 0.0
        || !settings.release_px.is_finite()
        || settings.release_px < settings.acquire_px
        || !settings.tolerance_px.is_finite()
        || settings.tolerance_px < 0.0
        || !candidate.distance_px.is_finite()
        || candidate.distance_px < 0.0
        || !candidate.correction.dx.is_finite()
        || !candidate.correction.dy.is_finite()
        || !settings.providers.contains(&candidate.source.provider)
        || !request.constraint.allows(candidate.correction)
        || !candidate.constraint.allows(candidate.correction)
    {
        return false;
    }
    if let SnapTarget::ObjectBounds(id) | SnapTarget::PathNode(id, _) = candidate.target {
        if request.moving.excluded.contains(&id) {
            return false;
        }
    }
    let threshold = if held(candidate, request) {
        settings.release_px
    } else {
        settings.acquire_px.min(settings.tolerance_px)
    };
    candidate.distance_px <= threshold
}

fn compare(a: &SnapCandidate, b: &SnapCandidate, request: &SnapRequest) -> std::cmp::Ordering {
    a.priority
        .cmp(&b.priority)
        .then_with(|| held(b, request).cmp(&held(a, request)))
        .then_with(|| a.distance_px.total_cmp(&b.distance_px))
        .then_with(|| {
            (a.source.provider as u8, a.source.key).cmp(&(b.source.provider as u8, b.source.key))
        })
        .then_with(|| target_key(a.target).cmp(&target_key(b.target)))
        .then_with(|| a.correction.dx.total_cmp(&b.correction.dx))
        .then_with(|| a.correction.dy.total_cmp(&b.correction.dy))
}

fn target_key(target: SnapTarget) -> (u8, u128, u64) {
    match target {
        SnapTarget::Guide(id) => (0, id.as_uuid().as_u128(), 0),
        SnapTarget::GridPoint(point) => (1, point.x.to_bits() as u128, point.y.to_bits()),
        SnapTarget::ObjectBounds(id) => (2, id.as_uuid().as_u128(), 0),
        SnapTarget::PathNode(id, node) => (3, id.as_uuid().as_u128(), node as u64),
    }
}

/// Keep a two-axis target atomic; combine only orthogonal single-axis
/// constraints. The same eligibility and hysteresis rules govern
/// ranking and the correction actually returned to the caller.
#[must_use]
pub fn solve(request: &SnapRequest, candidates: &[SnapCandidate]) -> SnapResult {
    let Some(winner) = rank_candidates(candidates, request) else {
        return SnapResult {
            corrected: TransformDelta::default(),
            matches: Vec::new(),
            visuals: Vec::new(),
            latch: None,
        };
    };
    let mut selected = vec![winner];
    let correction = winner.correction;
    if correction.dx == 0.0 || correction.dy == 0.0 {
        let other = candidates
            .iter()
            .filter(|candidate| {
                eligible(candidate, request)
                    && if correction.dx != 0.0 {
                        candidate.correction.dx == 0.0 && candidate.correction.dy != 0.0
                    } else {
                        candidate.correction.dy == 0.0 && candidate.correction.dx != 0.0
                    }
            })
            .min_by(|a, b| compare(a, b, request));
        if let Some(other) = other {
            selected.push(other);
        }
    }
    SnapResult {
        corrected: selected
            .iter()
            .fold(TransformDelta::default(), |delta, candidate| {
                delta.combined(candidate.correction)
            }),
        matches: selected
            .into_iter()
            .map(|candidate| SnapMatch {
                target: candidate.target,
                correction: candidate.correction,
                distance_px: candidate.distance_px,
            })
            .collect(),
        visuals: Vec::new(),
        latch: Some(SnapLatch {
            target: winner.target,
            priority: winner.priority,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anchor(x: f64, y: f64) -> Vec<Point> {
        vec![Point::new(x, y)]
    }

    fn tolerance() -> Tolerance {
        Tolerance::new(4.0).expect("valid")
    }

    fn guide_request(previous: Option<SnapLatch>) -> SnapRequest {
        SnapRequest {
            page: PageId::new_v4(),
            moving: MovingGeometry {
                excluded: Vec::new(),
                anchors: anchor(102.0, 50.0),
            },
            proposed: TransformDelta::default(),
            constraint: SnapConstraint::Free,
            settings: SnapSettings::default(),
            view_scale: 1.0,
            previous,
        }
    }

    #[test]
    fn guide_outranks_grid_at_equal_distance() {
        let guide = Guide::new(
            GuideAxis::Vertical,
            100.0,
            false,
            petunia_core::GuideScope::Document,
        )
        .expect("valid");
        let guides = guide_candidates(&[guide], &anchor(102.0, 50.0), tolerance(), 1.0);
        assert_eq!(guides.len(), 1);
        // Collinearity uses its own tight tolerance, not the
        // document-space snap tolerance.
        let tight = Tolerance::new(1e-9).expect("valid");
        let spec = AffineGridSpec::cartesian(10.0, 10.0, 1, tight).expect("valid");
        let mut grids = affine_grid_candidates(
            &spec,
            Point::new(0.0, 0.0),
            &anchor(102.0, 50.0),
            tolerance(),
            1.0,
        );
        assert_eq!(grids.len(), 1);
        // Force the tie the ranking must break deterministically.
        grids[0].distance_px = guides[0].distance_px;
        grids[0].priority = SnapPriority::Grid;
        let both = [guides[0], grids[0]];
        let request = guide_request(None);
        let winner = rank_candidates(&both, &request).expect("winner");
        assert_eq!(winner.priority, SnapPriority::Guide);
    }

    #[test]
    fn hysteresis_holds_inside_release_band() {
        let guide = Guide::new(
            GuideAxis::Vertical,
            100.0,
            false,
            petunia_core::GuideScope::Document,
        )
        .expect("valid");
        // Anchor sits 9px away: outside acquire (8px) but inside
        // release (11px), so only the latch keeps it.
        let candidates = guide_candidates(
            &[guide],
            &anchor(109.0, 50.0),
            Tolerance::new(12.0).expect("valid"),
            1.0,
        );
        assert_eq!(candidates.len(), 1);
        let latch = SnapLatch {
            target: candidates[0].target,
            priority: candidates[0].priority,
        };
        let mut request = guide_request(Some(latch));
        request.moving.anchors = anchor(109.0, 50.0);
        request.settings.acquire_px = 8.0;
        let winner = rank_candidates(&candidates, &request).expect("held");
        assert_eq!(winner.target, latch.target);
    }

    #[test]
    fn solve_combines_independent_axes() {
        let guide_x = Guide::new(
            GuideAxis::Vertical,
            100.0,
            false,
            petunia_core::GuideScope::Document,
        )
        .expect("valid");
        let guide_y = Guide::new(
            GuideAxis::Horizontal,
            200.0,
            false,
            petunia_core::GuideScope::Document,
        )
        .expect("valid");
        let candidates = [
            guide_candidates(&[guide_x], &anchor(102.0, 203.0), tolerance(), 1.0),
            guide_candidates(&[guide_y], &anchor(102.0, 203.0), tolerance(), 1.0),
        ]
        .concat();
        assert_eq!(candidates.len(), 2);
        let request = SnapRequest {
            moving: MovingGeometry {
                excluded: Vec::new(),
                anchors: anchor(102.0, 203.0),
            },
            ..guide_request(None)
        };
        let solved = solve(&request, &candidates);
        assert_eq!(solved.corrected, TransformDelta { dx: -2.0, dy: -3.0 });
        assert_eq!(solved.matches.len(), 2);
    }

    #[test]
    fn self_snap_exclusion_and_disabled_request() {
        let id = ObjectId::new_v4();
        let objects = vec![(id, Rect::new(90.0, 90.0, 20.0, 20.0))];
        let near = bounds_candidates(&objects, &[id], &anchor(92.0, 50.0), tolerance(), 1.0);
        assert!(near.is_empty());
        let far = bounds_candidates(&objects, &[], &anchor(92.0, 50.0), tolerance(), 1.0);
        assert!(!far.is_empty());
        let mut request = guide_request(None);
        request.settings.enabled = false;
        assert!(rank_candidates(&far, &request).is_none());
    }
}
