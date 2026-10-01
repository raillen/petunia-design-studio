//! Polygon booleans over flattened contours. i_overlay stays in the adapter.
//!
//! Curve-exact booleans are `POST_V1`: callers flatten via
//! [`crate::GPath::to_polygons`] first. The tolerance used is part of the
//! caller's evidence, never silently chosen here.

use serde::{Deserialize, Serialize};

use crate::GPoint;

/// Boolean operation selector.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BooleanOp {
    /// Union of both inputs.
    Union,
    /// Overlap of both inputs.
    Intersection,
    /// Subject minus clip.
    Difference,
    /// Union minus intersection.
    Xor,
}

/// One polygon input: closed contours (outer + holes).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BooleanInput {
    /// Closed contours; closure points are not duplicated.
    pub contours: Vec<Vec<GPoint>>,
}

impl BooleanInput {
    /// Creates an input from contours.
    #[must_use]
    pub fn new(contours: Vec<Vec<GPoint>>) -> Self {
        Self { contours }
    }

    /// Single-contour input.
    #[must_use]
    pub fn single(contour: Vec<GPoint>) -> Self {
        Self {
            contours: vec![contour],
        }
    }
}

/// Flattening tolerance policy for polygon booleans (F-21, 09.5).
/// Curve-exact booleans are `POST_V1`: callers flatten via
/// [`crate::GPath::to_polygons`] first. The tolerance used is part of the
/// caller's evidence and must travel explicitly via this type — never a
/// hardcoded literal at the call site.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeometryTolerance {
    /// Maximum deviation in document points when flattening curves.
    pub flatten: f64,
}

impl GeometryTolerance {
    /// Default document-scale tolerance (0.5pt). Matches the historical
    /// call-site behavior; prefer threading an explicit value instead.
    #[must_use]
    pub const fn default_tolerance() -> Self {
        Self { flatten: 0.5 }
    }

    /// Validated tolerance, clamped to a sane finite range.
    #[must_use]
    pub fn clamped(self) -> Self {
        Self {
            flatten: self.flatten.clamp(0.001, 10.0),
        }
    }
}

impl Default for GeometryTolerance {
    fn default() -> Self {
        Self::default_tolerance()
    }
}

/// Applies `op` to `subject` and `clip`, returning result contours.
/// Empty inputs follow set-theory identity (union keeps the other side,
/// intersection with either empty side is empty; A minus empty keeps A,
/// empty minus A is empty; xor keeps the other).
pub fn boolean_op(subject: &BooleanInput, clip: &BooleanInput, op: BooleanOp) -> Vec<Vec<GPoint>> {
    boolean_op_with_fill(subject, clip, op, FillRule::NonZero)
}

/// Fill-rule selector for boolean operations (10.3).
/// `EvenOdd` treats holes by parity; `NonZero` by winding direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FillRule {
    /// Even-odd parity rule.
    EvenOdd,
    /// Non-zero winding rule (document default).
    NonZero,
}

/// Applies `op` with an explicit fill rule for hole handling (F-21, 10.3).
pub fn boolean_op_with_fill(
    subject: &BooleanInput,
    clip: &BooleanInput,
    op: BooleanOp,
    fill_rule: FillRule,
) -> Vec<Vec<GPoint>> {
    let subject_empty = subject.contours.iter().all(|contour| contour.len() < 3);
    let clip_empty = clip.contours.iter().all(|contour| contour.len() < 3);
    let valid_contours = |input: &BooleanInput| {
        input
            .contours
            .iter()
            .filter(|contour| contour.len() >= 3)
            .cloned()
            .collect()
    };
    match op {
        BooleanOp::Union | BooleanOp::Xor => {
            if subject_empty {
                return valid_contours(clip);
            }
            if clip_empty {
                return valid_contours(subject);
            }
        }
        BooleanOp::Intersection => {
            if subject_empty || clip_empty {
                return Vec::new();
            }
        }
        BooleanOp::Difference => {
            if subject_empty {
                return Vec::new();
            }
            if clip_empty {
                return valid_contours(subject);
            }
        }
    }
    overlay_adapter::apply(subject, clip, op, fill_rule)
}

/// i_overlay adapter. Alongside `offset.rs`, the only modules allowed to name
/// `i_overlay` types.
mod overlay_adapter {
    use i_overlay::core::overlay_rule::OverlayRule;
    use i_overlay::float::single::SingleFloatOverlay as _;

    use super::{BooleanInput, BooleanOp, FillRule};
    use crate::GPoint;

    type Contour = Vec<[f64; 2]>;

    fn to_contours(input: &BooleanInput) -> Vec<Contour> {
        input
            .contours
            .iter()
            .filter(|c| c.len() >= 3)
            .map(|c| c.iter().map(|p| [p.x, p.y]).collect())
            .collect()
    }

    pub(super) fn apply(
        subject: &BooleanInput,
        clip: &BooleanInput,
        op: BooleanOp,
        fill_rule: FillRule,
    ) -> Vec<Vec<GPoint>> {
        let subj = to_contours(subject);
        let clip = to_contours(clip);
        if subj.is_empty() || clip.is_empty() {
            return Vec::new();
        }
        let rule = match op {
            BooleanOp::Union => OverlayRule::Union,
            BooleanOp::Intersection => OverlayRule::Intersect,
            BooleanOp::Difference => OverlayRule::Difference,
            BooleanOp::Xor => OverlayRule::Xor,
        };
        let io_fill = match fill_rule {
            FillRule::EvenOdd => i_overlay::core::fill_rule::FillRule::EvenOdd,
            FillRule::NonZero => i_overlay::core::fill_rule::FillRule::NonZero,
        };
        // Default i32 engine: deterministic for document-scale coordinates.
        let shapes = subj.overlay(&clip, rule, io_fill);
        shapes
            .iter()
            .flat_map(|shape| shape.iter())
            .map(|contour| contour.iter().map(|p| GPoint::new(p[0], p[1])).collect())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<GPoint> {
        vec![
            GPoint::new(x0, y0),
            GPoint::new(x1, y0),
            GPoint::new(x1, y1),
            GPoint::new(x0, y1),
        ]
    }

    fn contour_area(contour: &[GPoint]) -> f64 {
        contour
            .iter()
            .zip(contour.iter().cycle().skip(1))
            .map(|(a, b)| a.x * b.y - b.x * a.y)
            .sum::<f64>()
            .abs()
            / 2.0
    }

    #[test]
    fn union_area_of_overlapping_squares() {
        let subject = BooleanInput::single(square(0.0, 0.0, 2.0, 2.0));
        let clip = BooleanInput::single(square(1.0, 1.0, 3.0, 3.0));
        let result = boolean_op(&subject, &clip, BooleanOp::Union);
        let area: f64 = result.iter().map(|c| contour_area(c)).sum();
        // 4 + 4 - 1 overlap = 7.
        assert!((area - 7.0).abs() < 1e-6, "area was {area}");
    }

    #[test]
    fn intersection_area_of_overlapping_squares() {
        let subject = BooleanInput::single(square(0.0, 0.0, 2.0, 2.0));
        let clip = BooleanInput::single(square(1.0, 1.0, 3.0, 3.0));
        let result = boolean_op(&subject, &clip, BooleanOp::Intersection);
        let area: f64 = result.iter().map(|c| contour_area(c)).sum();
        assert!((area - 1.0).abs() < 1e-6, "area was {area}");
    }

    #[test]
    fn empty_side_follows_set_identity() {
        let empty = BooleanInput::new(Vec::new());
        let square_input = BooleanInput::single(square(0.0, 0.0, 1.0, 1.0));
        assert_eq!(
            boolean_op(&empty, &square_input, BooleanOp::Union),
            square_input.contours
        );
        assert!(boolean_op(&empty, &square_input, BooleanOp::Intersection).is_empty());
    }
}
