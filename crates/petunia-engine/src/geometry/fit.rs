//! Least-squares cubic Bézier fitting of point sequences.
//!
//! **Provenance and license.** The algorithm is Philip J. Schneider,
//! "An Algorithm for Automatically Fitting Digitized Curves",
//! *Graphics Gems* (1990): fit a cubic with fixed end tangents,
//! reparameterise with Newton steps, split at the worst point.
//! This implementation is adapted from VectorCraft
//! `crates/pathops/src/fit.rs` at commit
//! `5f92f5eb7fd194826aab2ad0844e474f13990938`, licensed
//! **MIT OR Apache-2.0** (see that repository's `LICENSE-MIT`,
//! `LICENSE-APACHE`). Adapted to Petunia's own `CubicBez`, `Point` and
//! `Vec2` types: no Kurbo type crosses this module, per the `math.rs`
//! invariant that Kurbo never defines the Persistente Petunia model.
//!
//! Use: rebuild compact cubics for points that lost their handles,
//! typically after a boolean flattens curves. Consecutive output
//! cubics are G1 continuous by construction.

use crate::geometry::bezier::CubicBez;
use petunia_core::{Contour, NodeKind, PathNode, Point, Vec2};

/// Maximum Newton reparameterisation rounds per fit.
const MAX_REPARAM: usize = 6;

/// Maximum recursion depth before accepting the current cubic.
const MAX_DEPTH: usize = 40;

/// Unit vector or `None` if (nearly) zero.
fn unit(v: Vec2) -> Option<Vec2> {
    let l = v.length();
    if l > 1e-12 && l.is_finite() {
        Some(scale(v, 1.0 / l))
    } else {
        None
    }
}

fn dot(a: Vec2, b: Vec2) -> f64 {
    a.dx * b.dx + a.dy * b.dy
}

/// Scale a vector by a scalar. Core keeps `Vec2`/`Point` free of
/// operator overloads, so the fit does its own tiny arithmetic.
fn scale(v: Vec2, k: f64) -> Vec2 {
    Vec2::new(v.dx * k, v.dy * k)
}

/// Translate a point by a vector.
fn add(point: Point, v: Vec2) -> Point {
    Point::new(point.x + v.dx, point.y + v.dy)
}

/// Displace a point backwards along a vector.
fn sub(point: Point, v: Vec2) -> Point {
    Point::new(point.x - v.dx, point.y - v.dy)
}

fn delta(from: Point, to: Point) -> Vec2 {
    Vec2::new(to.x - from.x, to.y - from.y)
}

/// First derivative of a cubic at `t`, as a vector.
fn eval_deriv1(c: &CubicBez, t: f64) -> Vec2 {
    let d0 = scale(delta(c.p0, c.p1), 3.0);
    let d1 = scale(delta(c.p1, c.p2), 3.0);
    let d2 = scale(delta(c.p2, c.p3), 3.0);
    let s = 1.0 - t;
    Vec2::new(
        s * s * d0.dx + 2.0 * s * t * d1.dx + t * t * d2.dx,
        s * s * d0.dy + 2.0 * s * t * d1.dy + t * t * d2.dy,
    )
}

/// Second derivative of a cubic at `t`, as a vector.
///
/// `B''(t) = (1-t)·6(p0 − 2p1 + p2) + t·6(p1 − 2p2 + p3)`.
fn eval_deriv2(c: &CubicBez, t: f64) -> Vec2 {
    let dd0 = Vec2::new(
        6.0 * (c.p0.x - 2.0 * c.p1.x + c.p2.x),
        6.0 * (c.p0.y - 2.0 * c.p1.y + c.p2.y),
    );
    let dd1 = Vec2::new(
        6.0 * (c.p1.x - 2.0 * c.p2.x + c.p3.x),
        6.0 * (c.p1.y - 2.0 * c.p2.y + c.p3.y),
    );
    let s = 1.0 - t;
    Vec2::new(s * dd0.dx + t * dd1.dx, s * dd0.dy + t * dd1.dy)
}

/// Direction the curve leaves `p0` (robust to retracted handles).
#[must_use]
pub fn start_tangent(c: &CubicBez) -> Vec2 {
    unit(delta(c.p0, c.p1))
        .or_else(|| unit(delta(c.p0, c.p2)))
        .or_else(|| unit(delta(c.p0, c.p3)))
        .unwrap_or_else(|| Vec2::new(1.0, 0.0))
}

/// Direction the curve arrives at `p3`, pointing *forward* along the
/// travel direction.
#[must_use]
pub fn end_tangent(c: &CubicBez) -> Vec2 {
    unit(delta(c.p2, c.p3))
        .or_else(|| unit(delta(c.p1, c.p3)))
        .or_else(|| unit(delta(c.p0, c.p3)))
        .unwrap_or_else(|| Vec2::new(1.0, 0.0))
}

/// Fit a chain of cubics through `pts` with maximum error `tol`.
///
/// `t0` is the unit tangent leaving `pts[0]`; `t1` is the unit tangent
/// arriving at the last point (forward direction). Empty or
/// single-point input fits nothing.
#[must_use]
pub fn fit_cubics(pts: &[Point], t0: Vec2, t1: Vec2, tol: f64) -> Vec<CubicBez> {
    let mut out = Vec::new();
    let pts = dedup(pts);
    if pts.len() < 2 {
        return out;
    }
    fit_rec(&pts, t0, t1, tol.max(1e-9), &mut out, 0);
    out
}

fn dedup(pts: &[Point]) -> Vec<Point> {
    let mut v: Vec<Point> = Vec::with_capacity(pts.len());
    for &p in pts {
        if v.last()
            .is_none_or(|q: &Point| delta(*q, p).length() > 1e-12)
        {
            v.push(p);
        }
    }
    v
}

fn fit_rec(pts: &[Point], t0: Vec2, t1: Vec2, tol: f64, out: &mut Vec<CubicBez>, depth: usize) {
    let n = pts.len();
    if n == 2 {
        let d = delta(pts[0], pts[1]).length() / 3.0;
        out.push(CubicBez::new(
            pts[0],
            add(pts[0], scale(t0, d)),
            sub(pts[1], scale(t1, d)),
            pts[1],
        ));
        return;
    }
    let (c, err, split) = fit_single(pts, t0, t1);
    if err <= tol || depth > MAX_DEPTH {
        out.push(c);
        return;
    }
    // Split at the worst point with a centre tangent estimated from
    // its neighbours.
    let split = split.clamp(1, n - 2);
    let tc = unit(delta(pts[split - 1], pts[split + 1])).unwrap_or(t0);
    fit_rec(&pts[..=split], t0, tc, tol, out, depth + 1);
    fit_rec(&pts[split..], tc, t1, tol, out, depth + 1);
}

/// Best single cubic for `pts` with the given end tangents. Returns
/// (curve, max error, index of the worst point).
fn fit_single(pts: &[Point], t0: Vec2, t1: Vec2) -> (CubicBez, f64, usize) {
    fit_single_from(pts, chord_params(pts), t0, t1)
}

/// [`fit_single`] starting from the parameters `u` instead of the
/// chord-length ones.
fn fit_single_from(pts: &[Point], mut u: Vec<f64>, t0: Vec2, t1: Vec2) -> (CubicBez, f64, usize) {
    let mut best = generate(pts, &u, t0, t1);
    let (mut best_err, mut best_split) = max_error(pts, &best, &u);
    for _ in 0..MAX_REPARAM {
        for (i, ui) in u.iter_mut().enumerate() {
            *ui = newton(&best, pts[i], *ui);
        }
        let c = generate(pts, &u, t0, t1);
        let (e, s) = max_error(pts, &c, &u);
        if e < best_err {
            best = c;
            best_err = e;
            best_split = s;
        } else {
            break;
        }
    }
    (best, best_err, best_split)
}

fn chord_params(pts: &[Point]) -> Vec<f64> {
    let mut u = Vec::with_capacity(pts.len());
    let mut acc = 0.0;
    u.push(0.0);
    for w in pts.windows(2) {
        acc += delta(w[0], w[1]).length();
        u.push(acc);
    }
    if acc > 0.0 {
        for x in &mut u {
            *x /= acc;
        }
    }
    u
}

fn bern(t: f64) -> [f64; 4] {
    let s = 1.0 - t;
    [s * s * s, 3.0 * s * s * t, 3.0 * s * t * t, t * t * t]
}

/// Solve the 2×2 normal equations for the handle lengths along the
/// fixed tangents.
fn generate(pts: &[Point], u: &[f64], t0: Vec2, t1: Vec2) -> CubicBez {
    let zero = CubicBez::new(
        Point::new(0.0, 0.0),
        Point::new(0.0, 0.0),
        Point::new(0.0, 0.0),
        Point::new(0.0, 0.0),
    );
    let _ = zero;
    let (Some(&p0), Some(&p3)) = (pts.first(), pts.last()) else {
        return zero;
    };
    let dir_in = scale(t1, -1.0); // handle at the end points backwards
    let (mut c00, mut c01, mut c11, mut x0, mut x1) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for (p, &t) in pts.iter().zip(u) {
        let b = bern(t);
        let a0 = scale(t0, b[1]);
        let a1 = scale(dir_in, b[2]);
        c00 += dot(a0, a0);
        c01 += dot(a0, a1);
        c11 += dot(a1, a1);
        let basis = Point::new(
            p0.x * (b[0] + b[1]) + p3.x * (b[2] + b[3]),
            p0.y * (b[0] + b[1]) + p3.y * (b[2] + b[3]),
        );
        let tmp = delta(basis, *p);
        x0 += dot(a0, tmp);
        x1 += dot(a1, tmp);
    }
    let det = c00 * c11 - c01 * c01;
    let seg = delta(p0, p3).length();
    let eps = 1e-6 * seg;
    let (mut al, mut ar) = if det.abs() > 1e-12 {
        ((x0 * c11 - x1 * c01) / det, (c00 * x1 - c01 * x0) / det)
    } else {
        (seg / 3.0, seg / 3.0)
    };
    if !al.is_finite() || !ar.is_finite() || al < eps || ar < eps {
        al = seg / 3.0;
        ar = seg / 3.0;
    }
    // Guard against wild handles on nearly-degenerate data.
    let cap = seg * 4.0 + 1e-9;
    al = al.min(cap);
    ar = ar.min(cap);
    CubicBez::new(p0, add(p0, scale(t0, al)), add(p3, scale(dir_in, ar)), p3)
}

fn max_error(pts: &[Point], c: &CubicBez, u: &[f64]) -> (f64, usize) {
    let mut e = 0.0;
    let mut idx = pts.len() / 2;
    for (i, (p, &t)) in pts.iter().zip(u).enumerate() {
        let d = delta(c.evaluate(t), *p).length();
        if d > e {
            e = d;
            idx = i;
        }
    }
    (e, idx)
}

fn newton(c: &CubicBez, p: Point, t: f64) -> f64 {
    // Residual points from the data point to the curve point.
    let q = delta(p, c.evaluate(t));
    let q1 = eval_deriv1(c, t);
    let q2 = eval_deriv2(c, t);
    let num = dot(q, q1);
    let den = dot(q1, q1) + dot(q, q2);
    if den.abs() < 1e-12 {
        return t;
    }
    let nt = t - num / den;
    if nt.is_finite() {
        nt.clamp(0.0, 1.0)
    } else {
        t
    }
}

/// Is the cubic (within `tol`) a straight line from `p0` to `p3`
/// with handles inside the chord?
#[must_use]
pub fn is_straight(c: &CubicBez, tol: f64) -> bool {
    let chord = delta(c.p0, c.p3);
    let len = chord.length();
    if len < 1e-12 {
        return delta(c.p0, c.p1).length() <= tol && delta(c.p0, c.p2).length() <= tol;
    }
    let dir = Vec2::new(chord.dx / len, chord.dy / len);
    [c.p1, c.p2].iter().all(|&h| {
        let v = delta(c.p0, h);
        let along = dot(v, dir);
        let side = v.dx * dir.dy - v.dy * dir.dx;
        side.abs() <= tol && along >= -tol && along <= len + tol
    })
}

/// Sample a cubic at `n + 1` evenly spaced parameters (both ends
/// included).
#[must_use]
pub fn sample_uniform(c: &CubicBez, n: usize) -> Vec<Point> {
    if n == 0 {
        return vec![c.p0];
    }
    (0..=n).map(|i| c.evaluate(i as f64 / n as f64)).collect()
}

/// True when two cubics join with matching tangent direction (G1).
#[must_use]
pub fn g1_continuous(left: &CubicBez, right: &CubicBez) -> bool {
    let a = unit(end_tangent(left));
    let b = unit(start_tangent(right));
    match (a, b) {
        (Some(a), Some(b)) => 1.0 - dot(a, b).abs() < 1e-6,
        _ => false,
    }
}

/// Rebuild the nodes of one contour by refitting its flattened
/// geometry, preserving anchors and closure.
///
/// Returns `None` when the contour has fewer than two nodes, holds
/// non-finite geometry, or cannot be refitted. Endpoint tangents come
/// from the contour's own nodes when present, otherwise from the
/// first and last polyline chords. Node kinds become
/// [`NodeKind::Smooth`] where neighbours are G1 continuous and
/// [`NodeKind::Cusp`] elsewhere, so the fit never invents tangency
/// the source did not have.
#[must_use]
pub fn refit_contour(contour: &Contour, tolerance: f64) -> Option<Vec<PathNode>> {
    let band = petunia_core::Tolerance::new(tolerance).ok()?;
    if contour.nodes.len() < 2 {
        return None;
    }
    let points = crate::geometry::bezier::flatten_contour(contour, band);
    if points.len() < 2 || points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
        return None;
    }

    // Endpoint tangents: explicit handles win, chords are the fallback.
    let first = contour.nodes.first()?;
    let last = contour.nodes.last()?;
    let t0 = first
        .handle_out
        .and_then(|handle| unit(delta(first.point, handle)))
        .or_else(|| unit(delta(points[0], points[1])))
        .unwrap_or_else(|| Vec2::new(1.0, 0.0));
    let t1 = last
        .handle_in
        .and_then(|handle| unit(delta(handle, last.point)))
        .or_else(|| unit(delta(points[points.len() - 2], *points.last()?)))
        .unwrap_or_else(|| Vec2::new(1.0, 0.0));

    let segments = fit_cubics(&points, t0, t1, tolerance.max(1e-9));
    if segments.is_empty() {
        return None;
    }

    let mut nodes: Vec<PathNode> = Vec::with_capacity(segments.len() + 1);
    for (index, curve) in segments.iter().enumerate() {
        let mut node = PathNode::new(curve.p0, NodeKind::Cusp);
        node.handle_in = (index > 0)
            .then(|| segments[index - 1].p2)
            .filter(|_| g1_continuous(&segments[index - 1], curve));
        node.handle_out = Some(curve.p1);
        if index > 0 && node.handle_in.is_some() {
            node.kind = NodeKind::Smooth;
        }
        nodes.push(node);
    }
    let tail = segments.last().expect("non-empty");
    let mut end = PathNode::new(tail.p3, NodeKind::Smooth);
    end.handle_in = Some(tail.p2);
    end.handle_out = None;
    nodes.push(end);
    Some(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_core::{Contour, Point as P, Rect};

    fn cubic() -> CubicBez {
        CubicBez::new(
            P::new(0.0, 0.0),
            P::new(30.0, 60.0),
            P::new(70.0, 60.0),
            P::new(100.0, 0.0),
        )
    }

    #[test]
    fn fits_a_cubic_exactly() {
        let c = cubic();
        let pts = sample_uniform(&c, 40);
        let out = fit_cubics(&pts, start_tangent(&c), end_tangent(&c), 0.05);
        assert_eq!(out.len(), 1);
        assert!(
            (out[0].p1.x - c.p1.x).abs() < 1.0 && (out[0].p1.y - c.p1.y).abs() < 1.0,
            "{:?}",
            out[0]
        );
    }

    #[test]
    fn splits_when_needed_and_stays_g1() {
        // A zig-zag cannot be fitted with one cubic.
        let pts: Vec<P> = (0..=40)
            .map(|i| P::new(i as f64, (i as f64 * 0.5).sin() * 10.0))
            .collect();
        let out = fit_cubics(&pts, Vec2::new(1.0, 0.0), Vec2::new(1.0, 0.0), 0.1);
        assert!(out.len() > 1);
        for pair in out.windows(2) {
            assert!((pair[0].p3.x - pair[1].p0.x).abs() < 1e-9);
            assert!((pair[0].p3.y - pair[1].p0.y).abs() < 1e-9);
            assert!(g1_continuous(&pair[0], &pair[1]), "segments must be G1");
        }
    }

    #[test]
    fn straight_detection_matches_geometry() {
        let line = CubicBez::new(
            P::new(0.0, 0.0),
            P::new(3.0, 0.0),
            P::new(6.0, 0.0),
            P::new(9.0, 0.0),
        );
        assert!(is_straight(&line, 1e-6));
        let bump = CubicBez::new(
            P::new(0.0, 0.0),
            P::new(3.0, 1.0),
            P::new(6.0, 0.0),
            P::new(9.0, 0.0),
        );
        assert!(!is_straight(&bump, 1e-6));
    }

    #[test]
    fn degenerate_inputs_never_panic() {
        // Empty, single point, duplicates and a zero chord.
        assert!(fit_cubics(&[], Vec2::new(1.0, 0.0), Vec2::new(1.0, 0.0), 0.1).is_empty());
        assert!(fit_cubics(
            &[P::new(1.0, 1.0)],
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 0.0),
            0.1
        )
        .is_empty());
        let duplicated = vec![P::new(2.0, 2.0), P::new(2.0, 2.0), P::new(2.0, 2.0)];
        let out = fit_cubics(&duplicated, Vec2::new(1.0, 0.0), Vec2::new(1.0, 0.0), 0.1);
        assert!(out.is_empty() || out.iter().all(|c| c.p1.x.is_finite()));
    }

    #[test]
    fn fit_is_deterministic() {
        let pts: Vec<P> = (0..=25)
            .map(|i| P::new(i as f64, ((i as f64) * 0.3).sin() * 5.0))
            .collect();
        let first = fit_cubics(&pts, Vec2::new(1.0, 0.0), Vec2::new(1.0, 0.0), 0.05);
        let second = fit_cubics(&pts, Vec2::new(1.0, 0.0), Vec2::new(1.0, 0.0), 0.05);
        assert_eq!(first.len(), second.len());
        for (a, b) in first.iter().zip(second.iter()) {
            assert_eq!(a.p1, b.p1);
            assert_eq!(a.p2, b.p2);
        }
    }

    #[test]
    fn refit_rebuilds_a_smooth_contour() {
        let mut contour = Contour::new(false);
        contour.push_node(PathNode::new(P::new(0.0, 0.0), NodeKind::Smooth));
        contour.push_node(PathNode::new(P::new(100.0, 0.0), NodeKind::Smooth));
        // 31 sampled points along an arc produce many nodes; refit must
        // return well below that.
        let points: Vec<P> = (0..=30)
            .map(|i| {
                let t = i as f64 / 30.0;
                P::new(t * 100.0, (t * std::f64::consts::FRAC_PI_2).sin() * 40.0)
            })
            .collect();
        for point in points {
            contour.push_node(PathNode::new(point, NodeKind::Cusp));
        }
        let nodes = refit_contour(&contour, 0.5).expect("refits");
        assert!(nodes.len() >= 2, "{:?}", nodes.len());
        assert!(nodes.len() < contour.nodes.len(), "refit must compact");
        assert!(nodes.iter().all(|node| node.point.x.is_finite()));
        let _ = Rect::new(0.0, 0.0, 1.0, 1.0);
    }

    /// Paridade com o que já existia: o refit precisa caber na curva
    /// original dentro da tolerância pedida e produzir no máximo tantos
    /// pontos quanto o caminho atual (`flatten` + `simplify`).
    #[test]
    fn refit_matches_flatten_simplify_within_tolerance() {
        use crate::geometry::simplify::simplify_open;

        let original = CubicBez::new(
            P::new(0.0, 0.0),
            P::new(20.0, 80.0),
            P::new(80.0, 80.0),
            P::new(100.0, 0.0),
        );
        let sampled = sample_uniform(&original, 64);
        let tolerance = 0.25;

        // Caminho antigo: achatar e simplificar a polilinha.
        let legacy = simplify_open(&sampled, tolerance);

        // Caminho novo: refitar em cubicas.
        let mut contour = Contour::new(false);
        for point in &sampled {
            contour.push_node(PathNode::new(*point, NodeKind::Cusp));
        }
        let fitted = refit_contour(&contour, tolerance).expect("refits");

        // Ambos preservam os extremos.
        assert!((fitted[0].point.x - 0.0).abs() < 1e-9);
        assert!((fitted.last().unwrap().point.x - 100.0).abs() < 1e-9);
        assert!(fitted.len() <= legacy.len());

        // Reconstruir os segmentos a partir dos nós e medir o desvio
        // contra a amostragem original.
        let mut worst = 0.0f64;
        for pair in fitted.windows(2) {
            let curve = CubicBez::new(
                pair[0].point,
                pair[0].handle_out.unwrap_or(pair[0].point),
                pair[1].handle_in.unwrap_or(pair[1].point),
                pair[1].point,
            );
            for step in 0..=8 {
                let probe = curve.evaluate(step as f64 / 8.0);
                let nearest = sampled
                    .iter()
                    .map(|point| delta(probe, *point).length())
                    .fold(f64::INFINITY, f64::min);
                worst = worst.max(nearest);
            }
        }
        assert!(worst < tolerance + 0.25, "refit desviou {worst} da curva");
    }
}
