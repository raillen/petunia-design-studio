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
    }

    /// Drops entries for ids no longer present.
    pub fn prune(&mut self, valid_ids: &[ObjectId]) {
        self.entries.retain(|id, _| valid_ids.contains(id));
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
    #[must_use]
    pub fn cached_hit(&self, id: ObjectId, pt: GPoint) -> bool {
        self.cached_path(id)
            .is_some_and(|path| path.contains_point(pt, 0.5))
    }
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
