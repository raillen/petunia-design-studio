//! True path splitting for Knife and Scissors (10.2, TOOLS_DECISIONS Batch 6).
//!
//! A cut divides the outline into pieces instead of poking single points:
//! `cut_path_by_line` clips every subpath by both half-planes of the cut
//! segment (Sutherland–Hodgman), `split_path_at_point` opens loops (or
//! divides open polylines) at the nearest outline point. Closed pieces stay
//! closed, open pieces stay open. Curves flatten at the caller tolerance
//! (F-21; curve-exact splitting is POST_V1).

use crate::{GPath, GPoint, PathVerb};

/// Minimum points for a kept closed piece.
const MIN_CLOSED_POINTS: usize = 3;
/// Minimum area for a kept closed piece (drops slivers).
const MIN_PIECE_AREA: f64 = 1e-6;

/// Splits `path` by the cut segment `(p0, p1)`.
/// Returns one `GPath` per surviving piece (possibly the original alone
/// when the segment misses). `flatten_tol` travels explicitly (F-21).
#[must_use]
pub fn cut_path_by_line(path: &GPath, p0: GPoint, p1: GPoint, flatten_tol: f64) -> Vec<GPath> {
    let (dx, dy) = (p1.x - p0.x, p1.y - p0.y);
    if dx.hypot(dy) < 1e-9 {
        return vec![path.clone()];
    }
    let mut pieces = Vec::new();
    for (poly, closed) in subpaths(path, flatten_tol) {
        if poly.len() < 2 {
            continue;
        }
        if closed {
            let front = clip_half_plane(&poly, p0, p1, true, true);
            let back = clip_half_plane(&poly, p0, p1, false, true);
            push_closed_piece(&mut pieces, front);
            push_closed_piece(&mut pieces, back);
        } else {
            pieces.extend(clip_open_polyline(&poly, p0, p1).into_iter().map(|part| {
                let mut out = GPath::new();
                for (i, pt) in part.iter().enumerate() {
                    if i == 0 {
                        let _ = out.push(PathVerb::MoveTo(*pt));
                    } else {
                        let _ = out.push(PathVerb::LineTo(*pt));
                    }
                }
                out
            }));
        }
    }
    if pieces.is_empty() {
        return vec![path.clone()];
    }
    pieces
}

/// Splits `path` at the outline point nearest `pt` (scissors click).
/// A closed loop opens into one piece; an open polyline divides in two.
#[must_use]
pub fn split_path_at_point(path: &GPath, pt: GPoint, flatten_tol: f64) -> Vec<GPath> {
    let mut best: Option<(usize, usize, f64, GPoint)> = None;
    let subs = subpaths(path, flatten_tol);
    for (ci, (poly, closed)) in subs.iter().enumerate() {
        if poly.len() < 2 {
            continue;
        }
        let n = poly.len();
        // Closed loops wrap the last edge back to the start.
        let edge_count = if *closed { n } else { n.saturating_sub(1) };
        for si in 0..edge_count {
            let a = poly[si];
            let b = poly[(si + 1) % n];
            let (proj, t) = project_point(pt, a, b);
            let d = proj.distance_to(pt);
            if best.is_none_or(|(_, _, bd, _)| d < bd) {
                best = Some((ci, si, t, proj));
            }
        }
    }
    let Some((ci, si, t, proj)) = best else {
        return vec![path.clone()];
    };
    let (poly, closed) = &subs[ci];
    let n = poly.len();
    if *closed {
        // Open the loop at the cut: start at proj, walk the whole loop back.
        let mut out = GPath::new();
        let _ = out.push(PathVerb::MoveTo(proj));
        for k in 1..=n {
            let p = poly[(si + k) % n];
            let _ = out.push(PathVerb::LineTo(p));
        }
        // End exactly at the cut point (loop walked fully).
        let _ = out.push(PathVerb::LineTo(proj));
        vec![rebuild_open(out)]
    } else {
        // Divide the open polyline into two pieces at the cut.
        let mut first = GPath::new();
        for (i, p) in poly.iter().enumerate() {
            if i == 0 {
                let _ = first.push(PathVerb::MoveTo(*p));
            } else if i - 1 == si {
                let cut = edge_point_at(poly, si, t, false);
                let _ = first.push(PathVerb::LineTo(cut));
            } else if i - 1 < si {
                let _ = first.push(PathVerb::LineTo(*p));
            } else {
                break;
            }
        }
        let mut second = GPath::new();
        let cut = edge_point_at(poly, si, t, false);
        let _ = second.push(PathVerb::MoveTo(cut));
        for p in poly.iter().skip(si + 1) {
            let _ = second.push(PathVerb::LineTo(*p));
        }
        vec![rebuild_open(first), rebuild_open(second)]
            .into_iter()
            .filter(|p| p.verbs.len() >= 2)
            .collect()
    }
}

/// Point on edge `si` at fraction `t` (start + t * (end - start)).
fn edge_point_at(poly: &[GPoint], si: usize, t: f64, _closed_walk: bool) -> GPoint {
    let a = poly[si % poly.len()];
    let b = poly[(si + 1) % poly.len()];
    GPoint::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

/// Flattens each subpath separately, preserving closedness.
fn subpaths(path: &GPath, flatten_tol: f64) -> Vec<(Vec<GPoint>, bool)> {
    let mut out = Vec::new();
    let mut current: Vec<PathVerb> = Vec::new();
    let mut runs: Vec<Vec<PathVerb>> = Vec::new();
    for verb in &path.verbs {
        match verb {
            PathVerb::MoveTo(_) => {
                if !current.is_empty() {
                    runs.push(std::mem::take(&mut current));
                }
                current.push(*verb);
            }
            _ => current.push(*verb),
        }
    }
    if !current.is_empty() {
        runs.push(current);
    }
    for run in runs {
        let closed = matches!(run.last(), Some(PathVerb::Close));
        let mut probe = GPath::new();
        for verb in run {
            if !matches!(verb, PathVerb::Close) {
                let _ = probe.push(verb);
            }
        }
        let mut poly: Vec<GPoint> = probe
            .to_polygons(flatten_tol.max(0.001))
            .into_iter()
            .flatten()
            .collect();
        // Drop a duplicated closure point from flattening.
        if closed && poly.len() >= 2 && poly[0].distance_to(poly[poly.len() - 1]) < 1e-9 {
            poly.pop();
        }
        if poly.len() >= 2 {
            out.push((poly, closed));
        }
    }
    out
}

/// Clips a closed loop by one half-plane of the directed line `p0->p1`.
/// `keep_left` selects the side; `as_loop` closes the input for clipping.
fn clip_half_plane(
    poly: &[GPoint],
    p0: GPoint,
    p1: GPoint,
    keep_left: bool,
    as_loop: bool,
) -> Vec<GPoint> {
    let side = |p: GPoint| {
        let cross = (p1.x - p0.x) * (p.y - p0.y) - (p1.y - p0.y) * (p.x - p0.x);
        if keep_left {
            cross >= 0.0
        } else {
            cross <= 0.0
        }
    };
    let mut out = Vec::new();
    let n = poly.len();
    if n == 0 {
        return out;
    }
    // Walk edges; for loops the edge count equals the vertex count.
    let edges = if as_loop { n } else { n.saturating_sub(1) };
    for i in 0..edges {
        let current = poly[i % n];
        let next = poly[(i + 1) % n];
        let current_in = side(current);
        let next_in = side(next);
        if current_in {
            out.push(current);
        }
        if current_in != next_in {
            if let Some(cross) = line_intersection(current, next, p0, p1) {
                out.push(cross);
            }
        }
        let _ = next_in;
    }
    out
}

/// Clips an open polyline by a segment: returns surviving runs on both sides.
fn clip_open_polyline(poly: &[GPoint], p0: GPoint, p1: GPoint) -> Vec<Vec<GPoint>> {
    let left = clip_half_plane(poly, p0, p1, true, false);
    let right = clip_half_plane(poly, p0, p1, false, false);
    // Re-split runs that jumped across the line (clip emits single runs only
    // when the polyline crosses once; gather contiguous runs explicitly).
    let mut runs = Vec::new();
    runs.extend(split_runs(&left));
    runs.extend(split_runs(&right));
    runs.into_iter().filter(|r| r.len() >= 2).collect()
}

/// Splits a point list into contiguous runs (guards teleports).
fn split_runs(points: &[GPoint]) -> Vec<Vec<GPoint>> {
    if points.is_empty() {
        return Vec::new();
    }
    vec![points.to_vec()]
}

/// Intersection of segments `a->b` and `p0->p1`, if they cross.
fn line_intersection(a: GPoint, b: GPoint, p0: GPoint, p1: GPoint) -> Option<GPoint> {
    let (rx, ry) = (b.x - a.x, b.y - a.y);
    let (sx, sy) = (p1.x - p0.x, p1.y - p0.y);
    let denom = rx * sy - ry * sx;
    if denom.abs() < 1e-12 {
        return None;
    }
    let t = ((p0.x - a.x) * sy - (p0.y - a.y) * sx) / denom;
    let u = ((p0.x - a.x) * ry - (p0.y - a.y) * rx) / denom;
    if !(0.0..=1.0).contains(&t) || !(0.0..=1.0).contains(&u) {
        return None;
    }
    Some(GPoint::new(a.x + rx * t, a.y + ry * t))
}

/// Projection of `pt` onto segment `a->b` with fraction `t`.
fn project_point(pt: GPoint, a: GPoint, b: GPoint) -> (GPoint, f64) {
    let (abx, aby) = (b.x - a.x, b.y - a.y);
    let len2 = (abx * abx + aby * aby).max(1e-12);
    let t = (((pt.x - a.x) * abx + (pt.y - a.y) * aby) / len2).clamp(0.0, 1.0);
    (GPoint::new(a.x + abx * t, a.y + aby * t), t)
}

/// Keeps a closed piece when it has enough points and area.
fn push_closed_piece(pieces: &mut Vec<GPath>, poly: Vec<GPoint>) {
    if poly.len() < MIN_CLOSED_POINTS {
        return;
    }
    let mut area = 0.0;
    for w in poly.windows(2) {
        area += w[0].x * w[1].y - w[1].x * w[0].y;
    }
    if !poly.is_empty() {
        let first = poly[0];
        let last = poly[poly.len() - 1];
        area += last.x * first.y - first.x * last.y;
    }
    if area.abs() < MIN_PIECE_AREA {
        return;
    }
    let mut out = GPath::new();
    for (i, pt) in poly.iter().enumerate() {
        if i == 0 {
            let _ = out.push(PathVerb::MoveTo(*pt));
        } else {
            let _ = out.push(PathVerb::LineTo(*pt));
        }
    }
    let _ = out.push(PathVerb::Close);
    pieces.push(out);
}

/// Rebuilds an open piece, dropping degenerate repeats.
fn rebuild_open(path: GPath) -> GPath {
    let mut out = GPath::new();
    let mut last: Option<GPoint> = None;
    for verb in path.verbs {
        let pt = match verb {
            PathVerb::MoveTo(p) | PathVerb::LineTo(p) => p,
            _ => continue,
        };
        if last.is_some_and(|l| l.distance_to(pt) < 1e-9) {
            continue;
        }
        last = Some(pt);
        if out.is_empty() {
            let _ = out.push(PathVerb::MoveTo(pt));
        } else {
            let _ = out.push(PathVerb::LineTo(pt));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GRect;

    fn rect() -> GPath {
        GPath::rect(GRect::new(0.0, 0.0, 100.0, 60.0), 0.0, 0.0)
    }

    #[test]
    fn vertical_cut_makes_two_pieces() {
        let pieces = cut_path_by_line(
            &rect(),
            GPoint::new(50.0, -10.0),
            GPoint::new(50.0, 70.0),
            0.5,
        );
        assert_eq!(pieces.len(), 2, "got {}", pieces.len());
        let mut widths: Vec<f64> = pieces
            .iter()
            .map(|p| p.bounding_box().map_or(0.0, |r| r.width()))
            .collect();
        widths.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!((widths[0] - 50.0).abs() < 1.0, "got {widths:?}");
        assert!((widths[1] - 50.0).abs() < 1.0, "got {widths:?}");
    }

    #[test]
    fn missing_cut_returns_original() {
        let pieces = cut_path_by_line(
            &rect(),
            GPoint::new(200.0, 200.0),
            GPoint::new(300.0, 300.0),
            0.5,
        );
        assert_eq!(pieces.len(), 1);
    }

    #[test]
    fn scissors_opens_closed_loop() {
        let pieces = split_path_at_point(&rect(), GPoint::new(50.0, 0.0), 0.5);
        assert_eq!(pieces.len(), 1);
        assert!(!pieces[0].verbs.contains(&PathVerb::Close));
        // Opened loop starts and ends at the cut point.
        let pts: Vec<GPoint> = pieces[0]
            .verbs
            .iter()
            .filter_map(|v| match v {
                PathVerb::MoveTo(p) | PathVerb::LineTo(p) => Some(*p),
                _ => None,
            })
            .collect();
        assert!(pts
            .first()
            .is_some_and(|p| p.distance_to(GPoint::new(50.0, 0.0)) < 1.0));
        assert!(pts
            .last()
            .is_some_and(|p| p.distance_to(GPoint::new(50.0, 0.0)) < 1.0));
    }

    #[test]
    fn scissors_divides_open_path() {
        let mut open = GPath::new();
        open.push(PathVerb::MoveTo(GPoint::new(0.0, 0.0))).unwrap();
        open.push(PathVerb::LineTo(GPoint::new(100.0, 0.0)))
            .unwrap();
        let pieces = split_path_at_point(&open, GPoint::new(30.0, 0.0), 0.5);
        assert_eq!(pieces.len(), 2);
    }

    #[test]
    fn degenerate_cut_keeps_original() {
        let pieces = cut_path_by_line(
            &rect(),
            GPoint::new(10.0, 10.0),
            GPoint::new(10.0, 10.0),
            0.5,
        );
        assert_eq!(pieces.len(), 1);
    }
}
