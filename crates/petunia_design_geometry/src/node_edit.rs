//! Direct node/vertex editing primitives (10.2, Table B).
//!
//! Moving a node translates its endpoint **and** its control handles by the
//! same delta, preserving tangents. The legacy tool behavior moved only the
//! endpoint and collapsed curves.

use crate::{GPoint, PathVerb};

/// Moves the verb at `idx` by `delta`, translating endpoint and control
/// handles together. Returns `false` for `Close` verbs and out-of-range
/// indices (no mutation).
pub fn move_verb(verbs: &mut [PathVerb], idx: usize, delta: GPoint) -> bool {
    let Some(verb) = verbs.get_mut(idx) else {
        return false;
    };
    let shift = |p: &mut GPoint| {
        p.x += delta.x;
        p.y += delta.y;
    };
    match verb {
        PathVerb::MoveTo(pt) | PathVerb::LineTo(pt) => {
            shift(pt);
            true
        }
        PathVerb::QuadTo(ctrl, pt) => {
            shift(ctrl);
            shift(pt);
            true
        }
        PathVerb::CubicTo(c1, c2, pt) => {
            shift(c1);
            shift(c2);
            shift(pt);
            true
        }
        PathVerb::Close => false,
    }
}

/// Moves the verb at `idx` to an absolute `target`, deriving the delta from
/// its current endpoint. Returns `false` when the verb has no endpoint.
pub fn move_verb_to(verbs: &mut [PathVerb], idx: usize, target: GPoint) -> bool {
    let current = match verbs.get(idx) {
        Some(PathVerb::MoveTo(p) | PathVerb::LineTo(p)) => *p,
        Some(PathVerb::QuadTo(_, p) | PathVerb::CubicTo(_, _, p)) => *p,
        _ => return false,
    };
    move_verb(
        verbs,
        idx,
        GPoint::new(target.x - current.x, target.y - current.y),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_path() -> Vec<PathVerb> {
        vec![
            PathVerb::MoveTo(GPoint::new(0.0, 0.0)),
            PathVerb::QuadTo(GPoint::new(1.0, 2.0), GPoint::new(2.0, 0.0)),
            PathVerb::Close,
        ]
    }

    #[test]
    fn move_preserves_tangents() {
        let mut verbs = quad_path();
        assert!(move_verb(&mut verbs, 1, GPoint::new(10.0, 0.0)));
        assert_eq!(
            verbs[1],
            PathVerb::QuadTo(GPoint::new(11.0, 2.0), GPoint::new(12.0, 0.0))
        );
    }

    #[test]
    fn close_and_oob_are_noops() {
        let mut verbs = quad_path();
        assert!(!move_verb(&mut verbs, 2, GPoint::new(1.0, 1.0)));
        assert!(!move_verb(&mut verbs, 9, GPoint::new(1.0, 1.0)));
        assert_eq!(verbs, quad_path());
    }
}
