//! Freehand pipeline primitives: simplify, smooth, Bézier fit (10.2, F-14).
//!
//! Pure functions over [`crate::GPoint`] slices so future tools (GUI,
//! plugins, MCP) share one deterministic implementation. No I/O, no global
//! state. Tolerances travel explicitly in document points.

use crate::{GPath, GPoint, PathVerb};

/// Midpoint of two document points.
#[must_use]
pub fn midpoint(a: GPoint, b: GPoint) -> GPoint {
    GPoint::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
}

/// Ramer-Douglas-Peucker simplification with perpendicular-distance
/// `epsilon` in document points. Deterministic; keeps endpoints.
#[must_use]
pub fn simplify_rdp(points: &[GPoint], epsilon: f64) -> Vec<GPoint> {
    if points.len() <= 2 {
        return points.to_vec();
    }
    let epsilon = epsilon.max(0.0);
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    let mut stack = vec![(0usize, points.len() - 1)];
    while let Some((first, last)) = stack.pop() {
        if last <= first + 1 {
            continue;
        }
        let a = points[first];
        let b = points[last];
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let length = dx.hypot(dy);
        let mut max_dist = 0.0;
        let mut max_idx = first;
        for (i, p) in points.iter().enumerate().take(last).skip(first + 1) {
            // Distance to the finite chord, not its supporting infinite line.
            // A return stroke can extend beyond either endpoint, including
            // the zero-length chord of a closed gesture.
            let dist = if length == 0.0 {
                p.distance_to(a)
            } else {
                let ux = dx / length;
                let uy = dy / length;
                let projection = ((p.x - a.x) * ux + (p.y - a.y) * uy).clamp(0.0, length);
                p.distance_to(GPoint::new(a.x + projection * ux, a.y + projection * uy))
            };
            if dist > max_dist {
                max_dist = dist;
                max_idx = i;
            }
        }
        if max_dist > epsilon {
            keep[max_idx] = true;
            stack.push((first, max_idx));
            stack.push((max_idx, last));
        }
    }
    points
        .iter()
        .enumerate()
        .filter(|(i, _)| keep[*i])
        .map(|(_, p)| *p)
        .collect()
}

/// Chaikin corner-cutting smoother. Each iteration replaces every segment
/// with Q(25%) and R(75%) points, preserving endpoints.
#[must_use]
pub fn chaikin_smooth(points: &[GPoint], iterations: usize) -> Vec<GPoint> {
    if points.len() <= 2 || iterations == 0 {
        return points.to_vec();
    }
    let mut current = points.to_vec();
    for _ in 0..iterations {
        if current.len() <= 2 {
            break;
        }
        let mut next = Vec::with_capacity(current.len() * 2);
        next.push(current[0]);
        for w in current.windows(2) {
            let (a, b) = (w[0], w[1]);
            next.push(GPoint::new(
                0.75 * a.x + 0.25 * b.x,
                0.75 * a.y + 0.25 * b.y,
            ));
            next.push(GPoint::new(
                0.25 * a.x + 0.75 * b.x,
                0.25 * a.y + 0.75 * b.y,
            ));
        }
        next.push(*current.last().unwrap());
        current = next;
    }
    current
}

/// Full freehand pipeline: `simplify_rdp(epsilon)` then
/// `chaikin_smooth(iterations)`. Returns the smoothed samples; call
/// [`fit_midpoint_quads`] to build the path.
#[must_use]
pub fn smooth_samples(points: &[GPoint], epsilon: f64, iterations: usize) -> Vec<GPoint> {
    chaikin_smooth(&simplify_rdp(points, epsilon), iterations)
}

/// Builds a path from smoothed samples with a midpoint quadratic chain:
/// control = sample, endpoint = midpoint to the next sample. Endpoints are
/// preserved exactly. Returns an error on non-finite input.
pub fn fit_midpoint_quads(samples: &[GPoint]) -> Result<GPath, String> {
    let mut path = GPath::new();
    if samples.is_empty() {
        return Ok(path);
    }
    path.push(PathVerb::MoveTo(samples[0]))?;
    if samples.len() == 2 {
        path.push(PathVerb::LineTo(samples[1]))?;
    } else if samples.len() > 2 {
        let mut i = 1;
        while i + 1 < samples.len() {
            let end = midpoint(samples[i], samples[i + 1]);
            path.push(PathVerb::QuadTo(samples[i], end))?;
            i += 1;
        }
        path.push(PathVerb::LineTo(*samples.last().unwrap()))?;
    }
    Ok(path)
}

/// Builds a path from Bézier anchors with explicit absolute handles.
/// Each entry is `(point, handle_in_absolute, handle_out_absolute)`.
/// Segments with handles on either side become `CubicTo` with the missing
/// side mirrored about the segment midpoint; otherwise `LineTo`.
/// `closed` appends `Close`.
pub fn anchors_to_path(
    anchors: &[(GPoint, Option<GPoint>, Option<GPoint>)],
    closed: bool,
) -> Result<GPath, String> {
    let mut path = GPath::new();
    let Some(first) = anchors.first() else {
        return Ok(path);
    };
    path.push(PathVerb::MoveTo(first.0))?;
    let mut prev = first;
    for current in &anchors[1..] {
        match (prev.2, current.1) {
            (Some(h1), Some(h2)) => {
                path.push(PathVerb::CubicTo(h1, h2, current.0))?;
            }
            (Some(h1), None) => {
                let mid = midpoint(prev.0, current.0);
                let h2 = GPoint::new(2.0 * mid.x - h1.x, 2.0 * mid.y - h1.y);
                path.push(PathVerb::CubicTo(h1, h2, current.0))?;
            }
            (None, Some(h2)) => {
                let mid = midpoint(prev.0, current.0);
                let h1 = GPoint::new(2.0 * mid.x - h2.x, 2.0 * mid.y - h2.y);
                path.push(PathVerb::CubicTo(h1, h2, current.0))?;
            }
            (None, None) => {
                path.push(PathVerb::LineTo(current.0))?;
            }
        }
        prev = current;
    }
    if closed {
        path.push(PathVerb::Close)?;
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rdp_keeps_endpoints_and_drops_collinear() {
        let pts = vec![
            GPoint::new(0.0, 0.0),
            GPoint::new(1.0, 0.01),
            GPoint::new(2.0, -0.01),
            GPoint::new(3.0, 0.0),
        ];
        let out = simplify_rdp(&pts, 0.5);
        assert_eq!(out, vec![GPoint::new(0.0, 0.0), GPoint::new(3.0, 0.0)]);
    }

    #[test]
    fn chaikin_preserves_endpoints() {
        let pts = vec![
            GPoint::new(0.0, 0.0),
            GPoint::new(1.0, 1.0),
            GPoint::new(2.0, 0.0),
        ];
        let out = chaikin_smooth(&pts, 1);
        assert_eq!(out.first(), Some(&GPoint::new(0.0, 0.0)));
        assert_eq!(out.last(), Some(&GPoint::new(2.0, 0.0)));
        assert!(out.len() > pts.len());
    }

    #[test]
    fn midpoint_chain_preserves_endpoints_and_uses_quads() {
        let pts = vec![
            GPoint::new(0.0, 0.0),
            GPoint::new(1.0, 2.0),
            GPoint::new(3.0, 1.0),
            GPoint::new(4.0, 4.0),
        ];
        let path = fit_midpoint_quads(&pts).expect("fit");
        assert!(matches!(path.verbs[0], PathVerb::MoveTo(_)));
        assert!(path.verbs.iter().any(|v| matches!(v, PathVerb::QuadTo(..))));
        let last = match path.verbs.last().unwrap() {
            PathVerb::LineTo(p) => *p,
            _ => panic!("expected trailing LineTo"),
        };
        assert_eq!(last, GPoint::new(4.0, 4.0));
    }

    #[test]
    fn anchors_without_handles_emit_lines() {
        let anchors = vec![
            (GPoint::new(0.0, 0.0), None, None),
            (GPoint::new(1.0, 0.0), None, None),
        ];
        let path = anchors_to_path(&anchors, false).expect("path");
        assert_eq!(path.verbs.len(), 2);
        assert!(matches!(path.verbs[1], PathVerb::LineTo(_)));
    }

    #[test]
    fn anchors_with_handles_emit_cubic() {
        let anchors = vec![
            (GPoint::new(0.0, 0.0), None, Some(GPoint::new(1.0, 0.0))),
            (GPoint::new(2.0, 0.0), Some(GPoint::new(1.0, 0.0)), None),
        ];
        let path = anchors_to_path(&anchors, false).expect("path");
        assert!(matches!(path.verbs[1], PathVerb::CubicTo(..)));
    }
}
