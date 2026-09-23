//! Session-level evaluated-geometry cache (GAUNTLET Loop 1 / F1).
//!
//! `DocumentObject::evaluated_path` re-flattens and re-runs modifiers on
//! every call; hover loops and overlays call it hundreds of times per
//! gesture. This cache memoizes `(evaluated path, evaluated bounds)` per
//! object, keyed by the session `current_revision` (bumped on every real
//! mutation, undo and redo — never on NoOp). Stale entries are impossible:
//! a revision mismatch recomputes. Memory is bounded by pruning deleted ids
//! on every `prune_selection` and clearing on document open/close.

use std::collections::HashMap;

use petunia_design_document::DocumentObject;
use petunia_design_foundation::ObjectId;
use petunia_design_geometry::{GPath, GPoint};

/// One cached evaluation.
#[derive(Clone, Debug, Default)]
pub struct GeoCacheEntry {
    /// Session revision the entry was computed at.
    pub revision: u64,
    /// Evaluated outline (modifiers folded).
    pub path: GPath,
    /// Evaluated bounds (None when the object has none).
    pub bounds: Option<[f64; 4]>,
}

/// Interior-mutable cache so `&self` readers (view-models, overlays,
/// hit-testing) share it without signature churn.
#[derive(Clone, Debug, Default)]
pub struct GeoCache {
    entries: HashMap<ObjectId, GeoCacheEntry>,
    /// Flattened outlines keyed by `(object, tolerance bits)`.
    /// Hit-testing, span sampling and previews share one flatten per
    /// tolerance instead of re-flattening per query (F2).
    flats: HashMap<(ObjectId, u64), Vec<Vec<GPoint>>>,
}

impl GeoCache {
    /// Creates an empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of live entries (for tests and diagnostics).
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when no entries are stored.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Drops all entries (document open/new/close).
    pub fn clear(&mut self) {
        self.entries.clear();
        self.flats.clear();
    }

    /// Drops entries for ids no longer present.
    pub fn prune(&mut self, valid_ids: &[ObjectId]) {
        self.entries.retain(|id, _| valid_ids.contains(id));
        self.flats.retain(|(id, _), _| valid_ids.contains(id));
    }

    /// Returns cached geometry when fresh, else `None`.
    fn get(&self, id: ObjectId, revision: u64) -> Option<(GPath, Option<[f64; 4]>)> {
        self.entries.get(&id).and_then(|entry| {
            if entry.revision == revision {
                Some((entry.path.clone(), entry.bounds))
            } else {
                None
            }
        })
    }

    /// Stores a fresh evaluation.
    fn insert(&mut self, id: ObjectId, revision: u64, path: GPath, bounds: Option<[f64; 4]>) {
        self.entries.insert(id, GeoCacheEntry {
            revision,
            path,
            bounds,
        });
    }
}

/// Session-side accessors. All take `&self`: the cache is interior-mutable.
impl crate::session::DocumentSession {
    /// Evaluated `(path, bounds)` for one object, memoized by revision.
    /// Mirrors `evaluated_bounds` semantics exactly: stored base bounds when
    /// no modifier is enabled, outline bbox otherwise.
    #[must_use]
    pub fn cached_geometry(&self, id: ObjectId) -> Option<(GPath, Option<[f64; 4]>)> {
        if let Some(hit) = self.geo_cache.borrow().get(id, self.current_revision()) {
            return Some(hit);
        }
        let obj = self.find_object(id)?;
        let path = obj.evaluated_path();
        let bounds = evaluated_bounds_for(obj, &path);
        self.geo_cache
            .borrow_mut()
            .insert(id, self.current_revision(), path.clone(), bounds);
        Some((path, bounds))
    }

    /// Evaluated outline, memoized.
    #[must_use]
    pub fn cached_path(&self, id: ObjectId) -> Option<GPath> {
        self.cached_geometry(id).map(|(path, _)| path)
    }

    /// Evaluated bounds, memoized.
    #[must_use]
    pub fn cached_bounds(&self, id: ObjectId) -> Option<[f64; 4]> {
        self.cached_geometry(id).and_then(|(_, bounds)| bounds)
    }

    /// Hit-test against the memoized evaluated outline.
    /// Visibility/locking stay at the call site (as with `hit_test` today).
    /// `tol` should come from `zoom_flatten_tol` (F2); one flatten serves
    /// every query at the same tolerance.
    #[must_use]
    pub fn cached_hit(&self, id: ObjectId, pt: GPoint, tol: f64) -> bool {
        self.cached_polygons(id, tol)
            .is_some_and(|polys| polys.iter().any(|poly| point_in_poly(pt, poly)))
    }

    /// Flattened evaluated outline at `tol`, memoized per revision.
    /// Backs hit-testing, span sampling and previews with a single flatten.
    #[must_use]
    pub fn cached_polygons(&self, id: ObjectId, tol: f64) -> Option<Vec<Vec<GPoint>>> {
        let tol = tol.max(0.001);
        let key = (id, tol.to_bits());
        let revision = self.current_revision();
        {
            let cache = self.geo_cache.borrow();
            if let Some(flats) = cache.flats.get(&key) {
                // Fresh only when the geometry entry matches this revision.
                if cache
                    .entries
                    .get(&id)
                    .is_some_and(|entry| entry.revision == revision)
                {
                    return Some(flats.clone());
                }
            }
        }
        let path = self.cached_path(id)?;
        let polys = path.to_polygons(tol);
        self.geo_cache.borrow_mut().flats.insert(key, polys.clone());
        Some(polys)
    }

    /// Outline sample at normalized fraction `t`, walking memoized polygons.
    /// Same math as `GPath::sample_at`, without re-flattening.
    #[must_use]
    pub fn cached_sample_at(&self, id: ObjectId, t: f64, tol: f64) -> Option<(GPoint, f64)> {
        let polys = self.cached_polygons(id, tol)?;
        sample_walk(&polys, t)
    }

    /// Normalized fraction of the outline point nearest `pt`, memoized.
    /// Same math as `GPath::nearest_t`, without re-flattening.
    #[must_use]
    pub fn cached_nearest_t(&self, id: ObjectId, pt: GPoint, tol: f64) -> Option<f64> {
        let polys = self.cached_polygons(id, tol)?;
        nearest_walk(&polys, pt)
    }
}

/// Even-odd point-in-polygon over one contour.
fn point_in_poly(pt: GPoint, poly: &[GPoint]) -> bool {
    let n = poly.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let pi = poly[i];
        let pj = poly[j];
        if (pi.y > pt.y) != (pj.y > pt.y)
            && pt.x < (pj.x - pi.x) * (pt.y - pi.y) / (pj.y - pi.y) + pi.x
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Arc-length walk over flattened contours (mirrors `GPath::sample_at`).
fn sample_walk(polys: &[Vec<GPoint>], t: f64) -> Option<(GPoint, f64)> {
    let total: f64 = polys
        .iter()
        .map(|c| c.windows(2).map(|w| w[0].distance_to(w[1])).sum::<f64>())
        .sum();
    if total < 1e-9 {
        let pt = polys.iter().flatten().next().copied()?;
        return Some((pt, 0.0));
    }
    let mut target = t.clamp(0.0, 1.0) * total;
    for contour in polys {
        for w in contour.windows(2) {
            let seg = w[0].distance_to(w[1]);
            if target <= seg {
                let f = if seg < 1e-12 { 0.0 } else { target / seg };
                let pt = GPoint::new(
                    w[0].x + (w[1].x - w[0].x) * f,
                    w[0].y + (w[1].y - w[0].y) * f,
                );
                return Some((pt, (w[1].y - w[0].y).atan2(w[1].x - w[0].x)));
            }
            target -= seg;
        }
    }
    let last = polys.iter().flatten().next_back().copied()?;
    Some((last, 0.0))
}

/// Nearest-fraction walk over flattened contours (mirrors `GPath::nearest_t`).
fn nearest_walk(polys: &[Vec<GPoint>], pt: GPoint) -> Option<f64> {
    let total: f64 = polys
        .iter()
        .map(|c| c.windows(2).map(|w| w[0].distance_to(w[1])).sum::<f64>())
        .sum();
    if total < 1e-9 {
        return None;
    }
    let mut best = (f64::INFINITY, 0.0);
    let mut acc = 0.0;
    for contour in polys {
        for w in contour.windows(2) {
            let (abx, aby) = (w[1].x - w[0].x, w[1].y - w[0].y);
            let len2 = (abx * abx + aby * aby).max(1e-12);
            let f = (((pt.x - w[0].x) * abx + (pt.y - w[0].y) * aby) / len2).clamp(0.0, 1.0);
            let proj = GPoint::new(w[0].x + abx * f, w[0].y + aby * f);
            let d = proj.distance_to(pt);
            if d < best.0 {
                let seg = abx.hypot(aby);
                best = (d, (acc + seg * f) / total);
            }
            acc += abx.hypot(aby);
        }
    }
    Some(best.1.clamp(0.0, 1.0))
}

/// Replicates `DocumentObject::evaluated_bounds` without re-evaluating.
fn evaluated_bounds_for(obj: &DocumentObject, evaluated: &GPath) -> Option<[f64; 4]> {
    if obj.modifiers.iter().any(|m| m.enabled) {
        evaluated.bounding_box().map(|r| {
            [
                r.x0,
                r.y0,
                r.width().max(1.0),
                r.height().max(1.0),
            ]
        })
    } else {
        obj.bounds
    }
}
