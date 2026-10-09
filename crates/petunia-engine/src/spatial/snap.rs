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
    for (guide_index, guide) in guides.iter().enumerate() {
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
                    key: ((guide_index as u64) << 32) | anchor_index as u64,
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
    for (object_index, (id, bounds)) in objects.iter().enumerate() {
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
                            key: ((object_index as u64) << 40)
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
                            key: ((object_index as u64) << 40)
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
    if !request.settings.enabled {
        return None;
    }
    if let Some(previous) = request.previous {
        if let Some(held) = candidates.iter().find(|candidate| {
            candidate.target == previous.target
                && candidate.distance_px <= request.settings.release_px
                && request.constraint.allows(candidate.correction)
        }) {
            return Some(held);
        }
    }
    candidates
        .iter()
        .filter(|candidate| {
            request
                .settings
                .providers
                .contains(&candidate.source.provider)
                && request.constraint.allows(candidate.correction)
                && candidate.distance_px <= request.settings.acquire_px
        })
        .min_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| {
                    a.distance_px
                        .partial_cmp(&b.distance_px)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| {
                    (a.source.provider as u8, a.source.key)
                        .cmp(&(b.source.provider as u8, b.source.key))
                })
        })
}

/// Solve one request: rank, then combine the best X and best Y
/// corrections independently when constraints stay orthogonal.
#[must_use]
pub fn solve(request: &SnapRequest, candidates: &[SnapCandidate]) -> SnapResult {
    let winner = rank_candidates(candidates, request);
    let Some(winner) = winner else {
        return SnapResult {
            corrected: TransformDelta::default(),
            matches: Vec::new(),
            visuals: Vec::new(),
            latch: None,
        };
    };
    // Independent axes: keep the best candidate affecting each axis.
    // Axis pools honor the same provider, constraint and acquire
    // gates as ranking.
    let axis_eligible = |candidate: &&SnapCandidate| {
        request
            .settings
            .providers
            .contains(&candidate.source.provider)
            && candidate.distance_px <= request.settings.acquire_px
    };
    let best_x = candidates
        .iter()
        .filter(|candidate| {
            axis_eligible(candidate)
                && candidate.correction.dx != 0.0
                && request.constraint.allows(TransformDelta {
                    dx: candidate.correction.dx,
                    dy: 0.0,
                })
        })
        .min_by(|a, b| rank_key(a).cmp(&rank_key(b)));
    let best_y = candidates
        .iter()
        .filter(|candidate| {
            axis_eligible(candidate)
                && candidate.correction.dy != 0.0
                && request.constraint.allows(TransformDelta {
                    dx: 0.0,
                    dy: candidate.correction.dy,
                })
        })
        .min_by(|a, b| rank_key(a).cmp(&rank_key(b)));
    let dx = best_x
        .map(|candidate| candidate.correction.dx)
        .unwrap_or(0.0);
    let dy = best_y
        .map(|candidate| candidate.correction.dy)
        .unwrap_or(0.0);
    let corrected = TransformDelta { dx, dy };
    let mut matches = Vec::new();
    if let Some(candidate) = best_x {
        if dx != 0.0 {
            matches.push(SnapMatch {
                target: candidate.target,
                correction: TransformDelta { dx, dy: 0.0 },
                distance_px: candidate.distance_px,
            });
        }
    }
    if let Some(candidate) = best_y {
        if dy != 0.0 {
            matches.push(SnapMatch {
                target: candidate.target,
                correction: TransformDelta { dx: 0.0, dy },
                distance_px: candidate.distance_px,
            });
        }
    }
    SnapResult {
        corrected,
        matches,
        visuals: Vec::new(),
        latch: Some(SnapLatch {
            target: winner.target,
            priority: winner.priority,
        }),
    }
}

fn rank_key(candidate: &SnapCandidate) -> (SnapPriority, u64, u64) {
    let distance = (candidate.distance_px * 1000.0) as u64;
    (candidate.priority, distance, candidate.source.key)
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
