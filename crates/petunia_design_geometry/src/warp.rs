//! Projective warp and rectangular clipping for live modifiers (10.8).
//!
//! `warp_path` maps an outline through the homography taking its bounding
//! box corners to an arbitrary quad (4-corner perspective, the 10.8 V1
//! baseline). Identity quads return the source untouched (curves preserved);
//! genuine warps flatten at the caller tolerance (F-21; rational-Bézier
//! exactness is POST_V1). Degenerate quads and vanishing-point crossings
//! (`w ≈ 0`) decline (`None`) so evaluation keeps the previous result
//! instead of destroying geometry.

use crate::{GPath, GPoint, GRect, PathVerb};

/// 3x3 homography matrix in row-major order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Homography {
    /// Rows of the matrix; applied as `p' = H * p` in homogeneous coordinates.
    pub rows: [[f64; 3]; 3],
}

impl Homography {
    /// Applies the transform to a point. `None` near the vanishing line.
    #[must_use]
    pub fn apply(&self, p: GPoint) -> Option<GPoint> {
        let w = self.rows[2][0] * p.x + self.rows[2][1] * p.y + self.rows[2][2];
        if w.abs() < 1e-9 {
            return None;
        }
        Some(GPoint::new(
            (self.rows[0][0] * p.x + self.rows[0][1] * p.y + self.rows[0][2]) / w,
            (self.rows[1][0] * p.x + self.rows[1][1] * p.y + self.rows[1][2]) / w,
        ))
    }

    /// True when the matrix is (numerically) the identity.
    #[must_use]
    pub fn is_identity(&self) -> bool {
        let id = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        self.rows
            .iter()
            .zip(id.iter())
            .all(|(a, b)| a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < 1e-9))
    }
}

/// Solves the homography mapping `src` corners to `dst` corners.
/// `None` when the system is singular (degenerate quad).
#[must_use]
pub fn homography_quad_to_quad(src: [GPoint; 4], dst: [GPoint; 4]) -> Option<Homography> {
    // Direct linear system with h33 = 1: two equations per correspondence.
    let mut mat = [[0.0f64; 8]; 8];
    let mut rhs = [0.0f64; 8];
    for (i, (s, d)) in src.iter().zip(dst.iter()).enumerate() {
        let r0 = i * 2;
        let r1 = i * 2 + 1;
        mat[r0][0] = s.x;
        mat[r0][1] = s.y;
        mat[r0][2] = 1.0;
        mat[r0][6] = -d.x * s.x;
        mat[r0][7] = -d.x * s.y;
        rhs[r0] = d.x;
        mat[r1][3] = s.x;
        mat[r1][4] = s.y;
        mat[r1][5] = 1.0;
        mat[r1][6] = -d.y * s.x;
        mat[r1][7] = -d.y * s.y;
        rhs[r1] = d.y;
    }
    let h = solve_8x8(mat, rhs)?;
    if !h.iter().all(|v| v.is_finite()) {
        return None;
    }
    Some(Homography {
        rows: [[h[0], h[1], h[2]], [h[3], h[4], h[5]], [h[6], h[7], 1.0]],
    })
}

/// Gaussian elimination with partial pivoting for an 8x8 system.
/// `None` on singularity.
fn solve_8x8(mut mat: [[f64; 8]; 8], mut rhs: [f64; 8]) -> Option<[f64; 8]> {
    for col in 0..8 {
        let mut pivot = col;
        for row in col..8 {
            if mat[row][col].abs() > mat[pivot][col].abs() {
                pivot = row;
            }
        }
        if mat[pivot][col].abs() < 1e-12 {
            return None;
        }
        mat.swap(col, pivot);
        rhs.swap(col, pivot);
        for row in 0..8 {
            if row == col {
                continue;
            }
            let factor = mat[row][col] / mat[col][col];
            if factor.abs() < 1e-15 {
                continue;
            }
            let pivot_row = mat[col];
            for (k, cell) in mat[row].iter_mut().enumerate().skip(col) {
                *cell -= factor * pivot_row[k];
            }
            rhs[row] -= factor * rhs[col];
        }
    }
    let mut out = [0.0f64; 8];
    for i in 0..8 {
        if mat[i][i].abs() < 1e-12 {
            return None;
        }
        out[i] = rhs[i] / mat[i][i];
    }
    Some(out)
}

/// Warps `path` so its bounding-box corners land on `quad`
/// (`[top-left, top-right, bottom-right, bottom-left]`).
/// Identity quads clone the source (curves preserved); real warps flatten
/// at `tolerance` (F-21). `None` on empty outlines and degenerate maps.
#[must_use]
pub fn warp_path_to_quad(path: &GPath, quad: [GPoint; 4], tolerance: f64) -> Option<GPath> {
    let bounds = path.bounding_box()?;
    let src = [
        GPoint::new(bounds.x0, bounds.y0),
        GPoint::new(bounds.x1, bounds.y0),
        GPoint::new(bounds.x1, bounds.y1),
        GPoint::new(bounds.x0, bounds.y1),
    ];
    if src
        .iter()
        .zip(quad.iter())
        .all(|(a, b)| a.distance_to(*b) < 1e-9)
    {
        return Some(path.clone());
    }
    let homography = match homography_quad_to_quad(src, quad) {
        Some(h) => h,
        // Degenerate source (zero-width/height outlines like bare strokes):
        // accept pure translations, decline anything else.
        None => return translated_copy(path, src, quad),
    };
    if homography.is_identity() {
        return Some(path.clone());
    }
    warp_path(path, &homography, tolerance)
}

/// Pure-translation fallback for degenerate sources (zero-area outlines).
/// Returns the translated path when every quad corner is its source corner
/// shifted by one constant offset; `None` otherwise (warping a line by
/// perspective is undefined, so evaluation keeps the previous result).
fn translated_copy(path: &GPath, src: [GPoint; 4], quad: [GPoint; 4]) -> Option<GPath> {
    let (dx, dy) = (quad[0].x - src[0].x, quad[0].y - src[0].y);
    if !quad
        .iter()
        .zip(src.iter())
        .all(|(q, s)| (q.x - s.x - dx).abs() < 1e-9 && (q.y - s.y - dy).abs() < 1e-9)
    {
        return None;
    }
    Some(path.transformed(crate::GAffine::translate(dx, dy)))
}

/// Warps every subpath run through a homography, preserving open/closed
/// topology. Curves flatten at `tolerance`; `None` on vanishing crossings.
#[must_use]
pub fn warp_path(path: &GPath, homography: &Homography, tolerance: f64) -> Option<GPath> {
    let mut out = GPath::new();
    let mut run: Vec<PathVerb> = Vec::new();
    let mut runs: Vec<(Vec<PathVerb>, bool)> = Vec::new();
    for verb in &path.verbs {
        match verb {
            PathVerb::MoveTo(_) => {
                if !run.is_empty() {
                    let closed = false;
                    runs.push((std::mem::take(&mut run), closed));
                }
                run.push(*verb);
            }
            PathVerb::Close => {
                runs.push((std::mem::take(&mut run), true));
            }
            _ => run.push(*verb),
        }
    }
    if !run.is_empty() {
        runs.push((run, false));
    }
    if runs.is_empty() {
        return None;
    }
    for (verbs, closed) in runs {
        let mut probe = GPath::new();
        for verb in verbs {
            let _ = probe.push(verb);
        }
        let polys = probe.to_polygons(tolerance.max(0.001));
        let mut first = true;
        for poly in polys {
            for pt in poly {
                let mapped = homography.apply(pt)?;
                if first {
                    let _ = out.push(PathVerb::MoveTo(mapped));
                    first = false;
                } else {
                    let _ = out.push(PathVerb::LineTo(mapped));
                }
            }
        }
        if closed && !out.is_empty() {
            let _ = out.push(PathVerb::Close);
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Clips `path` to an axis-aligned rectangle (CropRect evaluation).
/// Closed runs intersect (boolean); open runs clip per half-plane so
/// strokes trim instead of vanishing. `None` when nothing survives.
#[must_use]
pub fn clip_path_to_rect(path: &GPath, rect: GRect, tolerance: f64) -> Option<GPath> {
    let mut out = GPath::new();
    let mut run: Vec<PathVerb> = Vec::new();
    let mut runs: Vec<(Vec<PathVerb>, bool)> = Vec::new();
    for verb in &path.verbs {
        match verb {
            PathVerb::MoveTo(_) => {
                if !run.is_empty() {
                    runs.push((std::mem::take(&mut run), false));
                }
                run.push(*verb);
            }
            PathVerb::Close => {
                runs.push((std::mem::take(&mut run), true));
            }
            _ => run.push(*verb),
        }
    }
    if !run.is_empty() {
        runs.push((run, false));
    }
    let frame = vec![
        GPoint::new(rect.x0, rect.y0),
        GPoint::new(rect.x1, rect.y0),
        GPoint::new(rect.x1, rect.y1),
        GPoint::new(rect.x0, rect.y1),
    ];
    for (verbs, closed) in runs {
        let mut probe = GPath::new();
        for verb in verbs {
            let _ = probe.push(verb);
        }
        if closed {
            let subject = probe.to_polygons(tolerance.max(0.001));
            if subject.is_empty() {
                continue;
            }
            let clipped = crate::boolean_op(
                &crate::BooleanInput::new(subject),
                &crate::BooleanInput::single(frame.clone()),
                crate::BooleanOp::Intersection,
            );
            if !clipped.is_empty() {
                let piece = GPath::from_polygons(&clipped);
                for verb in piece.verbs {
                    let _ = out.push(verb);
                }
            }
        } else {
            for part in clip_open_runs(&probe, rect, tolerance) {
                for (i, pt) in part.iter().enumerate() {
                    if i == 0 {
                        let _ = out.push(PathVerb::MoveTo(*pt));
                    } else {
                        let _ = out.push(PathVerb::LineTo(*pt));
                    }
                }
            }
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Clips open runs against the rect, keeping inside pieces.
/// Segments crossing through (out-out) contribute their middle piece.
fn clip_open_runs(path: &GPath, rect: GRect, tolerance: f64) -> Vec<Vec<GPoint>> {
    let mut pieces: Vec<Vec<GPoint>> = Vec::new();
    for poly in path.to_polygons(tolerance.max(0.001)) {
        // Clipped sub-segment per source edge, then chained into runs.
        let mut clipped: Vec<(GPoint, GPoint)> = Vec::new();
        for w in poly.windows(2) {
            if let Some((t0, t1)) = liang_barsky(w[0], w[1], rect) {
                clipped.push((
                    GPoint::new(
                        w[0].x + (w[1].x - w[0].x) * t0,
                        w[0].y + (w[1].y - w[0].y) * t0,
                    ),
                    GPoint::new(
                        w[0].x + (w[1].x - w[0].x) * t1,
                        w[0].y + (w[1].y - w[0].y) * t1,
                    ),
                ));
            }
        }
        let mut current: Vec<GPoint> = Vec::new();
        for (e0, e1) in clipped {
            if current.is_empty() {
                current.push(e0);
            } else if current.last().is_some_and(|l| l.distance_to(e0) > 1e-9) {
                if current.len() >= 2 {
                    pieces.push(std::mem::take(&mut current));
                } else {
                    current.clear();
                }
                current.push(e0);
            }
            current.push(e1);
        }
        if current.len() >= 2 {
            pieces.push(current);
        }
    }
    pieces
}

/// Liang–Barsky interval of segment `a->b` inside `rect`, if any.
fn liang_barsky(a: GPoint, b: GPoint, rect: GRect) -> Option<(f64, f64)> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let mut t0: f64 = 0.0;
    let mut t1: f64 = 1.0;
    let edges = [
        (-dx, a.x - rect.x0),
        (dx, rect.x1 - a.x),
        (-dy, a.y - rect.y0),
        (dy, rect.y1 - a.y),
    ];
    for (p, q) in edges {
        if p.abs() < 1e-12 {
            if q < 0.0 {
                return None;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                t0 = t0.max(r);
            } else {
                t1 = t1.min(r);
            }
            if t0 > t1 {
                return None;
            }
        }
    }
    if t1 - t0 < 1e-12 {
        return None;
    }
    Some((t0.clamp(0.0, 1.0), t1.clamp(0.0, 1.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_rect() -> GPath {
        GPath::rect(GRect::new(0.0, 0.0, 100.0, 100.0), 0.0, 0.0)
    }

    fn quad(x0: f64, y0: f64, x1: f64, y1: f64) -> [GPoint; 4] {
        [
            GPoint::new(x0, y0),
            GPoint::new(x1, y0),
            GPoint::new(x1, y1),
            GPoint::new(x0, y1),
        ]
    }

    #[test]
    fn identity_quad_clones_curves() {
        let mut path = unit_rect();
        path.verbs.clear();
        path.push(PathVerb::MoveTo(GPoint::new(0.0, 0.0))).unwrap();
        path.push(PathVerb::CubicTo(
            GPoint::new(30.0, 0.0),
            GPoint::new(70.0, 100.0),
            GPoint::new(100.0, 100.0),
        ))
        .unwrap();
        let q = quad(0.0, 0.0, 100.0, 100.0);
        let out = warp_path_to_quad(&path, q, 0.25).expect("warp");
        assert_eq!(out.verbs, path.verbs);
    }

    #[test]
    fn translation_quad_shifts_bounds_exactly() {
        let out =
            warp_path_to_quad(&unit_rect(), quad(50.0, 25.0, 150.0, 125.0), 0.25).expect("warp");
        let bounds = out.bounding_box().expect("bounds");
        assert!((bounds.x0 - 50.0).abs() < 1e-6, "got {bounds:?}");
        assert!((bounds.y0 - 25.0).abs() < 1e-6, "got {bounds:?}");
        assert!((bounds.width() - 100.0).abs() < 1e-6, "got {bounds:?}");
    }

    #[test]
    fn perspective_maps_corners_onto_quad() {
        // Vanishing-right trapezoid: right edge half height.
        let q = [
            GPoint::new(0.0, 0.0),
            GPoint::new(100.0, 25.0),
            GPoint::new(100.0, 75.0),
            GPoint::new(0.0, 100.0),
        ];
        let out = warp_path_to_quad(&unit_rect(), q, 0.25).expect("warp");
        for corner in q {
            let hit = out
                .to_polygons(0.25)
                .iter()
                .flatten()
                .any(|p| p.distance_to(corner) < 1.0);
            assert!(hit, "corner {corner:?} missing");
        }
    }

    #[test]
    fn degenerate_quad_declines() {
        // Collinear quad: unsolvable.
        let q = [
            GPoint::new(0.0, 0.0),
            GPoint::new(50.0, 0.0),
            GPoint::new(100.0, 0.0),
            GPoint::new(150.0, 0.0),
        ];
        assert!(warp_path_to_quad(&unit_rect(), q, 0.25).is_none());
        assert!(warp_path_to_quad(&GPath::new(), quad(0.0, 0.0, 1.0, 1.0), 0.25).is_none());
    }

    #[test]
    fn open_path_stays_open_through_warp() {
        let line = GPath::line(GPoint::new(0.0, 0.0), GPoint::new(100.0, 0.0));
        let out = warp_path_to_quad(&line, quad(0.0, 10.0, 100.0, 10.0), 0.25).expect("warp");
        assert!(!out.verbs.contains(&PathVerb::Close));
        let bounds = out.bounding_box().expect("bounds");
        assert!((bounds.y0 - 10.0).abs() < 1e-6, "got {bounds:?}");
    }

    #[test]
    fn clip_keeps_inside_and_trims_crossing() {
        let rect = GRect::new(25.0, 25.0, 75.0, 75.0);
        let out = clip_path_to_rect(&unit_rect(), rect, 0.25).expect("clip");
        let bounds = out.bounding_box().expect("bounds");
        assert!((bounds.x0 - 25.0).abs() < 1.0, "got {bounds:?}");
        assert!((bounds.width() - 50.0).abs() < 1.0, "got {bounds:?}");
        // Open stroke trims instead of vanishing.
        let line = GPath::line(GPoint::new(0.0, 50.0), GPoint::new(100.0, 50.0));
        let out = clip_path_to_rect(&line, rect, 0.25).expect("clip");
        assert!(!out.verbs.contains(&PathVerb::Close));
        assert!((out.bounding_box().expect("b").width() - 50.0).abs() < 1.0);
        // Fully outside clips to nothing.
        let far = GPath::line(GPoint::new(200.0, 200.0), GPoint::new(300.0, 300.0));
        assert!(clip_path_to_rect(&far, rect, 0.25).is_none());
    }
}
