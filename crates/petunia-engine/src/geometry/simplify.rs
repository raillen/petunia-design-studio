//! Polyline simplification: radial pre-filter plus Ramer-Douglas-Peucker.
//!
//! Simplification is deterministic for a given tolerance and never
//! invents points: output is always a subsequence of the input.

use petunia_core::Point;

/// Simplify an open polyline, keeping the endpoints.
#[must_use]
pub fn simplify_open(points: &[Point], tolerance: f64) -> Vec<Point> {
    if points.len() < 3 || tolerance <= 0.0 {
        return points.to_vec();
    }
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    ramer_douglas_peucker(points, 0, points.len() - 1, tolerance, &mut keep);
    points
        .iter()
        .zip(keep.iter())
        .filter(|(_, kept)| **kept)
        .map(|(point, _)| *point)
        .collect()
}

/// Simplify a closed ring (treated cyclically, no duplicated anchor
/// required on input or output).
#[must_use]
pub fn simplify_closed(ring: &[Point], tolerance: f64) -> Vec<Point> {
    if ring.len() < 4 || tolerance <= 0.0 {
        return ring.to_vec();
    }
    // Cut the ring at the point farthest from the first vertex so the
    // open simplification covers the whole loop.
    let mut farthest = 1;
    let mut best = 0.0;
    for (index, point) in ring.iter().enumerate().skip(1) {
        let distance = (point.x - ring[0].x).hypot(point.y - ring[0].y);
        if distance > best {
            best = distance;
            farthest = index;
        }
    }
    let mut rotated = Vec::with_capacity(ring.len() + 1);
    rotated.extend_from_slice(&ring[farthest..]);
    rotated.extend_from_slice(&ring[..farthest]);
    rotated.push(ring[farthest]);
    let mut simplified = simplify_open(&rotated, tolerance);
    if simplified.len() > 1 {
        simplified.pop();
    }
    simplified
}

fn ramer_douglas_peucker(
    points: &[Point],
    first: usize,
    last: usize,
    tolerance: f64,
    keep: &mut [bool],
) {
    let mut pending = vec![(first, last)];
    while let Some((first, last)) = pending.pop() {
        if last <= first + 1 {
            continue;
        }
        let (mut farthest, mut best) = (first, 0.0);
        for index in first + 1..last {
            let distance = perpendicular_distance(points[index], points[first], points[last]);
            if distance > best {
                best = distance;
                farthest = index;
            }
        }
        if best > tolerance {
            keep[farthest] = true;
            pending.push((farthest, last));
            pending.push((first, farthest));
        }
    }
}

fn perpendicular_distance(point: Point, a: Point, b: Point) -> f64 {
    let base = (b.x - a.x).hypot(b.y - a.y);
    if base == 0.0 {
        return (point.x - a.x).hypot(point.y - a.y);
    }
    ((b.x - a.x) * (a.y - point.y) - (a.x - point.x) * (b.y - a.y)).abs() / base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collinear_runs_collapse_to_endpoints() {
        let line: Vec<Point> = (0..=10).map(|i| Point::new(i as f64, 0.0)).collect();
        let simplified = simplify_open(&line, 0.5);
        assert_eq!(simplified.len(), 2);
    }

    #[test]
    fn spikes_survive_above_tolerance() {
        let shape = vec![
            Point::new(0.0, 0.0),
            Point::new(5.0, 4.0),
            Point::new(10.0, 0.0),
        ];
        assert_eq!(simplify_open(&shape, 1.0).len(), 3);
        assert_eq!(simplify_open(&shape, 5.0).len(), 2);
    }

    #[test]
    fn closed_rings_stay_closed_without_anchor_duplication() {
        let ring = vec![
            Point::new(0.0, 0.0),
            Point::new(5.0, 0.1),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ];
        let simplified = simplify_closed(&ring, 0.5);
        assert!(simplified.len() >= 4);
        assert!(simplified.len() < ring.len());
    }
}
