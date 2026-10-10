//! Smart Delete: two explicit node-deletion actions (ADR-0012 D7).
//!
//! *Preserve Shape* removes a node and refits the curve within a
//! declared tolerance, reporting the maximum deviation it actually
//! achieved. *Hard Delete* joins the neighbours directly, with no
//! fitting. There is no silent simplification: when the fitted error
//! exceeds the tolerance the caller must choose Hard Delete or Cancel.

use petunia_core::{Contour, ContourId, PathNode, Tolerance};

/// Which of the two explicit actions the user picked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmartDeleteMode {
    /// Remove the node, refit by least squares, report the max error.
    PreserveShape,
    /// Join the neighbours directly, with no fitting.
    HardDelete,
}

/// Why a Smart Delete was refused instead of applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmartDeleteRefusal {
    /// The index is outside the contour.
    IndexOutOfRange,
    /// The contour would collapse below its minimum node count.
    TooFewNodes,
    /// A handle or anchor is not finite.
    NonFiniteGeometry,
}

/// Result of a Smart Delete attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum SmartDeleteOutcome {
    /// The deletion is applied; these are the new contour nodes, with
    /// the maximum deviation the fit achieved against the source
    /// geometry. The caller displays this number (ADR-0012 D7).
    Applied {
        /// Nodes of the contour after the deletion.
        nodes: Vec<PathNode>,
        /// Largest deviation between the source and the result.
        max_error: f64,
    },
    /// Fitting exceeded the tolerance; never simplify silently.
    NeedsConfirmation {
        /// Largest deviation between the fitted and original geometry.
        max_error: f64,
        /// Tolerance declared for the operation.
        tolerance: f64,
    },
    /// The request cannot be honoured.
    Refused(SmartDeleteRefusal),
}

impl SmartDeleteOutcome {
    /// Whether the caller may commit this outcome directly.
    #[must_use]
    pub fn is_applied(&self) -> bool {
        matches!(self, Self::Applied { .. })
    }

    /// Whether the user must choose Hard Delete or Cancel.
    #[must_use]
    pub fn needs_confirmation(&self) -> bool {
        matches!(self, Self::NeedsConfirmation { .. })
    }
}

/// Minimum nodes a contour keeps after a deletion: a closed contour
/// needs three corners to stay an area, an open one two to stay a
/// polyline.
const fn minimum_nodes(contour: &Contour) -> usize {
    if contour.closed {
        3
    } else {
        2
    }
}

/// Delete one node from `contour` using the chosen action.
///
/// `index` addresses `contour.nodes`. The surviving nodes keep their
/// order, so `ContourId` and the identity of the other anchors are
/// untouched.
#[must_use]
pub fn delete_node(
    contour: &Contour,
    index: usize,
    mode: SmartDeleteMode,
    tolerance: f64,
) -> SmartDeleteOutcome {
    let preview = match preview_delete(contour, index, tolerance) {
        Ok(preview) => preview,
        Err(refusal) => return SmartDeleteOutcome::Refused(refusal),
    };
    match (mode, preview.within_tolerance) {
        (SmartDeleteMode::HardDelete, _) => {
            let mut joined = contour.clone();
            joined.nodes.remove(index);
            join_directly(&mut joined);
            // The reported error always describes the returned nodes:
            // the plain join has its own deviation, not the fit's.
            let band = Tolerance::new(tolerance).unwrap_or(Tolerance(1e-6));
            let before = super::bezier::flatten_contour(contour, band);
            let max_error = max_deviation(&before, &joined);
            SmartDeleteOutcome::Applied {
                max_error,
                nodes: joined.nodes,
            }
        }
        (SmartDeleteMode::PreserveShape, true) => SmartDeleteOutcome::Applied {
            max_error: preview.max_error,
            nodes: preview.nodes,
        },
        (SmartDeleteMode::PreserveShape, false) => SmartDeleteOutcome::NeedsConfirmation {
            max_error: preview.max_error,
            tolerance,
        },
    }
}

/// Candidate result of deleting one node, computed without mutating
/// anything. The caller decides whether the error is acceptable:
/// `within_tolerance` mirrors the `delete_node` decision boundary
/// exactly, so preview and commit can never disagree.
#[must_use]
pub fn preview_delete(
    contour: &Contour,
    index: usize,
    tolerance: f64,
) -> Result<SmartDeletePreview, SmartDeleteRefusal> {
    if index >= contour.nodes.len() {
        return Err(SmartDeleteRefusal::IndexOutOfRange);
    }
    if contour
        .nodes
        .iter()
        .any(|node| !node.point.x.is_finite() || !node.point.y.is_finite())
    {
        return Err(SmartDeleteRefusal::NonFiniteGeometry);
    }
    if contour.nodes.len() <= minimum_nodes(contour) {
        return Err(SmartDeleteRefusal::TooFewNodes);
    }

    // Original geometry, for measuring what the deletion costs.
    let before = super::bezier::flatten_contour(
        contour,
        Tolerance::new(tolerance).unwrap_or(Tolerance(1e-6)),
    );
    if before.is_empty() {
        return Err(SmartDeleteRefusal::NonFiniteGeometry);
    }

    let mut trimmed = contour.clone();
    trimmed.nodes.remove(index);
    if trimmed.nodes.len() < minimum_nodes(&trimmed) {
        return Err(SmartDeleteRefusal::TooFewNodes);
    }

    // A closed contour must keep its minimum node count, so a fit
    // that collapses it is not usable: fall back to the plain join
    // and report its honest deviation.
    let fitted = super::fit::refit_contour(&trimmed, tolerance.max(1e-9))
        .filter(|nodes| nodes.len() >= minimum_nodes(&trimmed))
        .map(|nodes| Contour {
            id: ContourId::new_v4(),
            nodes,
            closed: contour.closed,
        });
    let candidate = match fitted {
        Some(fitted) => fitted,
        None => {
            let mut joined = trimmed.clone();
            join_directly(&mut joined);
            joined
        }
    };
    let max_error = max_deviation(&before, &candidate);
    Ok(SmartDeletePreview {
        nodes: candidate.nodes,
        max_error,
        within_tolerance: max_error <= tolerance,
    })
}

/// Candidate deletion result for preview rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct SmartDeletePreview {
    /// Contour nodes the deletion would produce.
    pub nodes: Vec<PathNode>,
    /// Largest deviation between source and candidate geometry.
    pub max_error: f64,
    /// Whether the error fits the declared tolerance.
    pub within_tolerance: bool,
}

/// Hard Delete: the neighbours are joined with no curve at all.
fn join_directly(contour: &mut Contour) {
    for node in &mut contour.nodes {
        node.handle_in = None;
        node.handle_out = None;
        node.kind = petunia_core::NodeKind::Cusp;
    }
}

/// Largest distance from any point of the original geometry to the
/// polyline of the new one.
fn max_deviation(original: &[petunia_core::Point], fitted: &Contour) -> f64 {
    let band = Tolerance::new(1e-6).unwrap_or(Tolerance(1e-6));
    let after = super::bezier::flatten_contour(fitted, band);
    if after.is_empty() {
        return f64::INFINITY;
    }
    let mut worst = 0.0;
    for point in original {
        let mut best = f64::INFINITY;
        for segment in after.windows(2) {
            let distance = point_to_segment(*point, segment[0], segment[1]);
            if distance < best {
                best = distance;
            }
        }
        if best > worst {
            worst = best;
        }
    }
    worst
}

fn point_to_segment(
    point: petunia_core::Point,
    start: petunia_core::Point,
    end: petunia_core::Point,
) -> f64 {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length_squared = dx * dx + dy * dy;
    if length_squared <= 0.0 {
        return hypot(point.x - start.x, point.y - start.y);
    }
    let t = ((point.x - start.x) * dx + (point.y - start.y) * dy) / length_squared;
    let clamped = t.clamp(0.0, 1.0);
    hypot(
        point.x - (start.x + clamped * dx),
        point.y - (start.y + clamped * dy),
    )
}

fn hypot(a: f64, b: f64) -> f64 {
    (a * a + b * b).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_core::{FillRule, NodeKind, Point, VectorPath};

    /// Smooth pill: a closed contour with four smooth anchors. The top
    /// apex is 26 above the chord its neighbours would form, so
    /// deleting it is a real deformation, not a simplification.
    fn pill() -> Contour {
        let mut contour = Contour::new(true);
        contour.nodes = vec![
            PathNode::with_handles(
                Point::new(-50.0, 0.0),
                Some(Point::new(-20.0, -26.0)),
                Some(Point::new(-20.0, 26.0)),
                NodeKind::Smooth,
            ),
            PathNode::with_handles(
                Point::new(0.0, 26.0),
                Some(Point::new(-20.0, 26.0)),
                Some(Point::new(20.0, 26.0)),
                NodeKind::Smooth,
            ),
            PathNode::with_handles(
                Point::new(50.0, 0.0),
                Some(Point::new(20.0, 26.0)),
                Some(Point::new(20.0, -26.0)),
                NodeKind::Smooth,
            ),
            PathNode::with_handles(
                Point::new(0.0, -26.0),
                Some(Point::new(20.0, -26.0)),
                Some(Point::new(-20.0, -26.0)),
                NodeKind::Smooth,
            ),
        ];
        contour
    }

    /// Three collinear anchors on an open contour.
    fn collinear_polyline() -> Contour {
        let mut contour = Contour::new(false);
        contour.nodes = vec![
            PathNode::new(Point::new(0.0, 0.0), NodeKind::Cusp),
            PathNode::new(Point::new(40.0, 0.0), NodeKind::Cusp),
            PathNode::new(Point::new(80.0, 0.0), NodeKind::Cusp),
        ];
        contour
    }

    #[test]
    fn hard_delete_joins_neighbours_without_curves() {
        let outcome = delete_node(&pill(), 1, SmartDeleteMode::HardDelete, 30.0);
        let SmartDeleteOutcome::Applied { nodes, max_error } = outcome else {
            panic!("hard delete must apply: {outcome:?}");
        };
        assert_eq!(nodes.len(), 3);
        // The honest deformation the plain join costs, displayed to the
        // user instead of hidden.
        assert!(max_error > 1.0, "{max_error}");
        assert!(nodes.iter().all(|node| node.handle_in.is_none()));
        assert!(nodes.iter().all(|node| node.handle_out.is_none()));
        // The removed apex is gone; the others keep their order.
        assert_eq!(nodes[0].point, Point::new(-50.0, 0.0));
        assert_eq!(nodes[1].point, Point::new(50.0, 0.0));
        assert_eq!(nodes[2].point, Point::new(0.0, -26.0));
    }

    #[test]
    fn preserve_shape_applies_when_the_deformation_is_within_tolerance() {
        // The apex sits 26 above the chord, so a tolerance above that
        // covers the refit and the outcome is a straight Apply.
        let outcome = delete_node(&pill(), 1, SmartDeleteMode::PreserveShape, 40.0);
        assert!(outcome.is_applied(), "{outcome:?}");
        // The reported error is inside the declared tolerance, and the
        // contour stays drawable.
        let SmartDeleteOutcome::Applied { nodes, max_error } = &outcome else {
            unreachable!("applied above")
        };
        assert!(max_error <= &40.0, "{max_error}");
        assert!(nodes.len() >= minimum_nodes(&pill()), "{}", nodes.len());
    }

    #[test]
    fn preserve_shape_asks_instead_of_simplifying_silently() {
        // The same request under a tight tolerance must surface the
        // error instead of committing a worse curve.
        let outcome = delete_node(&pill(), 1, SmartDeleteMode::PreserveShape, 1.0);
        assert!(outcome.needs_confirmation(), "{outcome:?}");
        let SmartDeleteOutcome::NeedsConfirmation {
            max_error,
            tolerance,
        } = outcome
        else {
            panic!("expected a confirmation request");
        };
        assert!(max_error > tolerance, "{max_error} vs {tolerance}");
    }

    #[test]
    fn preview_matches_the_commit_boundary() {
        let contour = pill();
        let preview = preview_delete(&contour, 1, 40.0).expect("previews");
        assert!(preview.within_tolerance);
        assert_eq!(
            delete_node(&contour, 1, SmartDeleteMode::PreserveShape, 40.0),
            SmartDeleteOutcome::Applied {
                max_error: preview.max_error,
                nodes: preview.nodes.clone(),
            }
        );
        let tight = preview_delete(&contour, 1, 1.0).expect("previews");
        assert!(!tight.within_tolerance);
        assert!(matches!(
            delete_node(&contour, 1, SmartDeleteMode::PreserveShape, 1.0),
            SmartDeleteOutcome::NeedsConfirmation { .. }
        ));
        // Preview never mutates its input.
        assert_eq!(contour.nodes.len(), 4);
    }

    #[test]
    fn hard_delete_applies_even_when_the_shape_gets_worse() {
        // Hard Delete is explicit and has no error gate: the user
        // already chose the cheaper action.
        let outcome = delete_node(&pill(), 1, SmartDeleteMode::HardDelete, 1.0);
        assert!(outcome.is_applied(), "{outcome:?}");
    }

    #[test]
    fn deleting_collapses_and_invalid_indices_are_refused() {
        // A closed contour cannot drop below three corners.
        let mut triangle = Contour::new(true);
        triangle.nodes = vec![
            PathNode::new(Point::new(0.0, 0.0), NodeKind::Cusp),
            PathNode::new(Point::new(10.0, 0.0), NodeKind::Cusp),
            PathNode::new(Point::new(5.0, 10.0), NodeKind::Cusp),
        ];
        assert_eq!(
            delete_node(&triangle, 0, SmartDeleteMode::HardDelete, 1.0),
            SmartDeleteOutcome::Refused(SmartDeleteRefusal::TooFewNodes)
        );
        assert_eq!(
            delete_node(&triangle, 9, SmartDeleteMode::HardDelete, 1.0),
            SmartDeleteOutcome::Refused(SmartDeleteRefusal::IndexOutOfRange)
        );
        // An open polyline with two nodes is already minimal.
        let mut pair = Contour::new(false);
        pair.nodes = vec![
            PathNode::new(Point::new(0.0, 0.0), NodeKind::Cusp),
            PathNode::new(Point::new(1.0, 1.0), NodeKind::Cusp),
        ];
        assert_eq!(
            delete_node(&pair, 0, SmartDeleteMode::HardDelete, 1.0),
            SmartDeleteOutcome::Refused(SmartDeleteRefusal::TooFewNodes)
        );
    }

    #[test]
    fn collinear_middle_node_deletes_with_zero_error() {
        let outcome = delete_node(
            &collinear_polyline(),
            1,
            SmartDeleteMode::PreserveShape,
            0.5,
        );
        let SmartDeleteOutcome::Applied { nodes, max_error } = outcome else {
            panic!("collinear deletion must apply: {outcome:?}");
        };
        assert_eq!(max_error, 0.0, "nothing moves, nothing deviates");
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].point, Point::new(0.0, 0.0));
        assert_eq!(nodes[1].point, Point::new(80.0, 0.0));
    }

    #[test]
    fn non_finite_geometry_is_refused() {
        let mut broken = collinear_polyline();
        broken.nodes[1].point = Point::new(f64::NAN, 0.0);
        assert_eq!(
            delete_node(&broken, 0, SmartDeleteMode::HardDelete, 1.0),
            SmartDeleteOutcome::Refused(SmartDeleteRefusal::NonFiniteGeometry)
        );
    }

    #[test]
    fn surviving_nodes_keep_identity_of_the_contour() {
        let mut path = VectorPath::new();
        path.fill_rule = FillRule::EvenOdd;
        path.push_contour(pill());
        let before: Vec<Point> = path.contours[0]
            .nodes
            .iter()
            .map(|node| node.point)
            .collect();
        let SmartDeleteOutcome::Applied { nodes, .. } =
            delete_node(&path.contours[0], 2, SmartDeleteMode::HardDelete, 1.0)
        else {
            panic!("hard delete must apply");
        };
        let after: Vec<Point> = nodes.iter().map(|node| node.point).collect();
        // Same contour, same fill rule, one fewer node, same order.
        assert_eq!(path.contours.len(), 1);
        assert_eq!(path.fill_rule, FillRule::EvenOdd);
        assert_eq!(after.len(), before.len() - 1);
        assert_eq!(after[0], before[0]);
        assert_eq!(after[1], before[1]);
        assert_eq!(after[2], before[3]);
    }
}
