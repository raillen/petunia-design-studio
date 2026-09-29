//! Shared stroke-proximity helpers (Select P1 + Knife).
//!
//! Both the Select tool (click/hover) and the Knife/Scissors tools need
//! "near the evaluated outline" checks because open paths have no interior
//! for fill-only hit-testing. The segment-distance math lives here so the
//! two tools cannot drift; each tool keeps its own `near_*` wrapper because
//! polygon sources differ (Select uses memoized `GeoCache` world/legacy
//! flattens with adaptive `zoom_flatten_tol`, Knife uses direct
//! `evaluated_path().to_polygons(0.5)` for cut targeting).
//! Node/text/pencil keep their local copies (out of P1 scope).

use petunia_design_geometry::GPoint;

/// Shortest distance from `pt` to segment `a->b`.
#[must_use]
pub fn dist_to_segment(pt: GPoint, a: GPoint, b: GPoint) -> f64 {
    let abx = b.x - a.x;
    let aby = b.y - a.y;
    let len2 = (abx * abx + aby * aby).max(1e-12);
    let t = (((pt.x - a.x) * abx + (pt.y - a.y) * aby) / len2).clamp(0.0, 1.0);
    pt.distance_to(GPoint::new(a.x + abx * t, a.y + aby * t))
}

/// True when `pt` lies within `tol` of any contour segment.
#[must_use]
pub fn contours_near_point(polys: &[Vec<GPoint>], pt: GPoint, tol: f64) -> bool {
    polys
        .iter()
        .flat_map(|contour| contour.windows(2))
        .any(|w| dist_to_segment(pt, w[0], w[1]) <= tol)
}
