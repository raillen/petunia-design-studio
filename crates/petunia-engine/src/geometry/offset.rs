//! Miter polygon offsetting.
//!
//! Each edge shifts along its normal and vertices rejoin at miter
//! intersections. Positive distances expand counter-clockwise rings;
//! spikes fall back to a bevel join instead of exploding. Arcs and
//! round joins stay out of v0.1: the contract is exact miters with
//! documented limits.

use crate::error::{EngineError, Result};
use petunia_core::Point;

/// Offset a closed polygon ring by `distance`. Returns the shifted
/// ring, or an error for degenerate input (fewer than three points
/// or zero-length edges).
pub fn offset_ring(ring: &[Point], distance: f64) -> Result<Vec<Point>> {
    if ring.len() < 3 {
        return Err(EngineError::Execution(
            "offset needs a closed ring of at least three points".to_string(),
        ));
    }
    if !distance.is_finite() {
        return Err(EngineError::Execution(format!(
            "non-finite offset distance rejected: {distance}"
        )));
    }
    let count = ring.len();
    let mut edges = Vec::with_capacity(count);
    for index in 0..count {
        let a = ring[index];
        let b = ring[(index + 1) % count];
        let direction = normalize(Point::new(b.x - a.x, b.y - a.y)).ok_or_else(|| {
            EngineError::Execution("offset rejects zero-length edges".to_string())
        })?;
        edges.push(direction);
    }
    let mut out = Vec::with_capacity(count);
    for index in 0..count {
        let previous = edges[(index + count - 1) % count];
        let current = edges[index];
        // Left normals of both incident edges.
        let n0 = Point::new(-previous.y, previous.x);
        let n1 = Point::new(-current.y, current.x);
        let dot = previous.x * current.x + previous.y * current.y;
        let vertex = ring[index];
        if (1.0 + dot).abs() < 1e-9 {
            // Spike (180° reversal): bevel instead of an infinite miter.
            out.push(Point::new(
                vertex.x + (n0.x + n1.x) * 0.5 * distance,
                vertex.y + (n0.y + n1.y) * 0.5 * distance,
            ));
            continue;
        }
        let miter = Point::new(n0.x + n1.x, n0.y + n1.y);
        let scale = distance / (1.0 + dot);
        out.push(Point::new(
            vertex.x + miter.x * scale,
            vertex.y + miter.y * scale,
        ));
    }
    Ok(out)
}

fn normalize(edge: Point) -> Option<Point> {
    let length = edge.x.hypot(edge.y);
    if length == 0.0 || !length.is_finite() {
        return None;
    }
    Some(Point::new(edge.x / length, edge.y / length))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Vec<Point> {
        vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ]
    }

    fn approx(points: &[Point], expected: &[[f64; 2]]) -> bool {
        points.len() == expected.len()
            && points.iter().zip(expected.iter()).all(|(point, want)| {
                (point.x - want[0]).abs() < 1e-9 && (point.y - want[1]).abs() < 1e-9
            })
    }

    #[test]
    fn miter_offset_expands_square_symmetrically() {
        // CCW square: positive distance offsets outward (left of edges).
        let ring = vec![
            Point::new(0.0, 0.0),
            Point::new(0.0, 10.0),
            Point::new(10.0, 10.0),
            Point::new(10.0, 0.0),
        ];
        let expanded = offset_ring(&ring, 2.0).expect("valid");
        assert!(
            approx(
                &expanded,
                &[[-2.0, -2.0], [-2.0, 12.0], [12.0, 12.0], [12.0, -2.0],],
            ),
            "got {expanded:?}"
        );
        let shrunk = offset_ring(&ring, -2.0).expect("valid");
        assert!(
            approx(&shrunk, &[[2.0, 2.0], [2.0, 8.0], [8.0, 8.0], [8.0, 2.0]],),
            "got {shrunk:?}"
        );
    }

    #[test]
    fn degenerate_input_errors() {
        assert!(offset_ring(&square()[..2], 1.0).is_err());
        assert!(offset_ring(&square(), f64::NAN).is_err());
        let mut with_spike = square();
        with_spike[1] = Point::new(0.0, 0.0);
        assert!(offset_ring(&with_spike, 1.0).is_err());
    }
}
