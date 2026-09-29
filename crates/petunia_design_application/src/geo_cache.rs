//! Session-level evaluated-geometry cache (GAUNTLET Loop 1 / F1).
//!
//! `DocumentObject::evaluated_path` re-flattens and re-runs modifiers on
//! every call; hover loops and overlays call it hundreds of times per
//! gesture. This cache memoizes `(evaluated path, evaluated bounds)` per
//! object, keyed by the session `current_revision` (bumped on every real
//! mutation, undo and redo — never on NoOp). Stale entries are impossible:
//! a revision mismatch recomputes. Memory is bounded by pruning deleted ids
//! on every `prune_selection` and clearing on document open/close.

use std::collections::{HashMap, HashSet};

use petunia_design_document::DocumentObject;
use petunia_design_foundation::ObjectId;
use petunia_design_geometry::{GAffine, GPath, GPoint};

/// One cached evaluation, separated by coordinate domain.
#[derive(Clone, Debug)]
pub struct GeoCacheEntry {
    /// Session revision the entry was computed at.
    pub revision: u64,
    /// Legacy evaluated path retained for v1 consumers.
    pub legacy_path: GPath,
    /// Legacy evaluated bounds retained for v1 consumers.
    pub legacy_bounds: Option<[f64; 4]>,
    /// Evaluated local outline, when the frame is explicit.
    pub local_path: Option<GPath>,
    /// Evaluated local bounds, when the local outline has geometry.
    pub local_bounds: Option<[f64; 4]>,
    /// Composed world transform, when the frame is explicit.
    pub world_transform: Option<GAffine>,
    /// Evaluated world path, when the local frame is explicit.
    pub world_path: Option<GPath>,
    /// Evaluated world bounds, if the world path has geometry.
    pub world_bounds: Option<[f64; 4]>,
    /// Nominal world frame bounds, available even for an unversioned path.
    pub world_frame_bounds: Option<[f64; 4]>,
}

/// Interior-mutable cache so `&self` readers (view-models, overlays,
/// hit-testing) share it without signature churn.
#[derive(Clone, Debug, Default)]
pub struct GeoCache {
    entries: HashMap<ObjectId, GeoCacheEntry>,
    /// Flattened legacy outlines keyed by `(object, tolerance bits)`.
    flats: HashMap<(ObjectId, u64), Vec<Vec<GPoint>>>,
    /// Flattened world outlines keyed by `(object, tolerance bits)`.
    world_flats: HashMap<(ObjectId, u64), Vec<Vec<GPoint>>>,
    /// Total `cached_polygons` / `cached_world_polygons` lookups (F7.3 probe:
    /// `span_points` must serve 25 samples with exactly one lookup).
    flat_lookups: u64,
    /// Actual `to_polygons` computations (cache misses). Hits reuse `flats`
    /// without re-flattening.
    flat_computes: u64,
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

    /// Total flatten lookups served (hits + misses). F7.3 probe.
    #[must_use]
    pub fn flat_lookups(&self) -> u64 {
        self.flat_lookups
    }

    /// Actual flatten computations (misses only). F7.3 probe.
    #[must_use]
    pub fn flat_computes(&self) -> u64 {
        self.flat_computes
    }

    /// Resets the F7.3 flatten probes without dropping cached geometry.
    pub fn reset_flat_stats(&mut self) {
        self.flat_lookups = 0;
        self.flat_computes = 0;
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
        self.world_flats.clear();
    }

    /// Drops entries for ids no longer present.
    pub fn prune(&mut self, valid_ids: &[ObjectId]) {
        let valid: HashSet<ObjectId> = valid_ids.iter().copied().collect();
        self.entries.retain(|id, _| valid.contains(id));
        self.flats.retain(|(id, _), _| valid.contains(id));
        self.world_flats.retain(|(id, _), _| valid.contains(id));
    }

    /// Returns cached geometry when fresh, else `None`.
    fn get(&self, id: ObjectId, revision: u64) -> Option<GeoCacheEntry> {
        self.entries.get(&id).and_then(|entry| {
            if entry.revision == revision {
                Some(entry.clone())
            } else {
                None
            }
        })
    }

    /// Stores a fresh evaluation.
    fn insert(&mut self, id: ObjectId, entry: GeoCacheEntry) {
        self.entries.insert(id, entry);
    }
}

/// Session-side accessors. All take `&self`: the cache is interior-mutable.
impl crate::session::DocumentSession {
    /// Evaluates legacy, local and world domains at the current revision.
    pub fn cached_geometry_entry(&self, id: ObjectId) -> Option<GeoCacheEntry> {
        let revision = self.current_revision();
        if let Some(entry) = self.geo_cache.borrow().get(id, revision) {
            return Some(entry);
        }
        let object = self.find_object(id)?;
        let legacy_path = object.evaluated_path();
        let legacy_bounds = evaluated_bounds_for(object, &legacy_path);
        let explicit = object.evaluated_path_local().ok().and_then(|local_path| {
            let world_transform = self.document().world_transform_checked(id).ok()?;
            let world_path = local_path.transformed(world_transform);
            let local_bounds = local_path.bounding_box().map(|rect| {
                [
                    rect.x0,
                    rect.y0,
                    rect.width().max(1.0),
                    rect.height().max(1.0),
                ]
            });
            let world_bounds = world_path.bounding_box().map(|rect| {
                [
                    rect.x0,
                    rect.y0,
                    rect.width().max(1.0),
                    rect.height().max(1.0),
                ]
            });
            Some((
                local_path,
                local_bounds,
                world_transform,
                world_path,
                world_bounds,
            ))
        });
        let world_frame_bounds = self.document().frame_bounds_world(id).ok();
        let (local_path, local_bounds, world_transform, world_path, world_bounds) = match explicit {
            Some(values) => (
                Some(values.0),
                values.1,
                Some(values.2),
                Some(values.3),
                values.4,
            ),
            None => (None, None, None, None, None),
        };
        let entry = GeoCacheEntry {
            revision,
            legacy_path,
            legacy_bounds,
            local_path,
            local_bounds,
            world_transform,
            world_path,
            world_bounds,
            world_frame_bounds,
        };
        self.geo_cache.borrow_mut().insert(id, entry.clone());
        Some(entry)
    }

    /// Legacy `(path, bounds)` compatibility query.
    #[must_use]
    pub fn cached_geometry(&self, id: ObjectId) -> Option<(GPath, Option<[f64; 4]>)> {
        self.cached_geometry_entry(id)
            .map(|entry| (entry.legacy_path, entry.legacy_bounds))
    }

    /// Explicit local evaluated path.
    #[must_use]
    pub fn cached_local_path(&self, id: ObjectId) -> Option<GPath> {
        self.cached_geometry_entry(id)?.local_path
    }

    /// Explicit local evaluated bounds.
    #[must_use]
    pub fn cached_local_bounds(&self, id: ObjectId) -> Option<[f64; 4]> {
        self.cached_geometry_entry(id)?.local_bounds
    }

    /// Explicit world transform.
    #[must_use]
    pub fn cached_world_transform(&self, id: ObjectId) -> Option<GAffine> {
        self.cached_geometry_entry(id)?.world_transform
    }

    /// Explicit world evaluated path.
    #[must_use]
    pub fn cached_world_path(&self, id: ObjectId) -> Option<GPath> {
        self.cached_geometry_entry(id)?.world_path
    }

    /// Explicit world evaluated bounds.
    #[must_use]
    pub fn cached_world_bounds(&self, id: ObjectId) -> Option<[f64; 4]> {
        self.cached_geometry_entry(id)?.world_bounds
    }

    /// Nominal world frame bounds, including unversioned legacy paths.
    #[must_use]
    pub fn cached_world_frame_bounds(&self, id: ObjectId) -> Option<[f64; 4]> {
        self.cached_geometry_entry(id)?.world_frame_bounds
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
        self.geo_cache.borrow_mut().flat_lookups += 1;
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
        {
            let mut cache = self.geo_cache.borrow_mut();
            cache.flat_computes += 1;
            cache.flats.insert(key, polys.clone());
        }
        Some(polys)
    }

    /// Flattened explicit world outline at `tol`, memoized per revision.
    #[must_use]
    pub fn cached_world_polygons(&self, id: ObjectId, tol: f64) -> Option<Vec<Vec<GPoint>>> {
        let tol = tol.max(0.001);
        let key = (id, tol.to_bits());
        let revision = self.current_revision();
        self.geo_cache.borrow_mut().flat_lookups += 1;
        {
            let cache = self.geo_cache.borrow();
            if let Some(flats) = cache.world_flats.get(&key) {
                if cache
                    .entries
                    .get(&id)
                    .is_some_and(|entry| entry.revision == revision)
                {
                    return Some(flats.clone());
                }
            }
        }
        let path = self.cached_world_path(id)?;
        let polys = path.to_polygons(tol);
        {
            let mut cache = self.geo_cache.borrow_mut();
            cache.flat_computes += 1;
            cache.world_flats.insert(key, polys.clone());
        }
        Some(polys)
    }

    /// Exact world-space hit test with the shared spatial tolerance.
    #[must_use]
    pub fn cached_world_hit(&self, id: ObjectId, pt: GPoint, tol: f64) -> bool {
        self.cached_world_polygons(id, tol)
            .is_some_and(|polys| polys.iter().any(|poly| point_in_poly(pt, poly)))
    }

    /// World-space outline sample at normalized fraction `t`.
    #[must_use]
    pub fn cached_world_sample_at(&self, id: ObjectId, t: f64, tol: f64) -> Option<(GPoint, f64)> {
        let polys = self.cached_world_polygons(id, tol)?;
        sample_walk(&polys, t)
    }

    /// World-space normalized fraction nearest to `pt`.
    #[must_use]
    pub fn cached_world_nearest_t(&self, id: ObjectId, pt: GPoint, tol: f64) -> Option<f64> {
        let polys = self.cached_world_polygons(id, tol)?;
        nearest_walk(&polys, pt)
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

    /// Total flatten lookups served (F7.3 probe for shared-flatten spans).
    #[must_use]
    pub fn flatten_lookup_count(&self) -> u64 {
        self.geo_cache.borrow().flat_lookups()
    }

    /// Actual flatten computations, i.e. cache misses (F7.3 probe).
    #[must_use]
    pub fn flatten_compute_count(&self) -> u64 {
        self.geo_cache.borrow().flat_computes()
    }

    /// Resets the F7.3 flatten probes without dropping cached geometry.
    pub fn reset_flatten_stats(&self) {
        self.geo_cache.borrow_mut().reset_flat_stats();
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
        evaluated
            .bounding_box()
            .map(|r| [r.x0, r.y0, r.width().max(1.0), r.height().max(1.0)])
    } else {
        obj.bounds
    }
}
