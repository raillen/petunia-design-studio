//! Session-level spatial index over evaluated bounds (GAUNTLET Loop 3 / F3).
//!
//! Hit-testing and marquee loops scan every object today. This R-tree
//! (`rstar`, bulk-loaded per revision over the active surface) reduces
//! candidate sets before exact tests run. Entries carry z-order sequence so
//! queries return topmost-first, matching the existing reverse iteration.
//! Like `GeoCache`, it is interior-mutable and keyed by `current_revision`,
//! rebuilt lazily on first query after a mutation.

use rstar::{RTree, RTreeObject, AABB};

use petunia_design_foundation::ObjectId;
use petunia_design_geometry::GPoint;

/// One indexed object: stable id, z-order sequence, evaluated bounds.
#[derive(Clone, Debug, PartialEq)]
struct IndexedObj {
    id: ObjectId,
    seq: usize,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

impl RTreeObject for IndexedObj {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners([self.x0, self.y0], [self.x1, self.y1])
    }
}

/// Revision-keyed R-tree over one surface.
#[derive(Clone, Debug, Default)]
pub struct SpatialIndex {
    revision: u64,
    entries: usize,
    tree: RTree<IndexedObj>,
}

impl SpatialIndex {
    /// Creates an empty index.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of indexed objects.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries
    }

    /// True when nothing is indexed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries == 0
    }

    /// Drops the index (document open/new/close prune it implicitly by revision).
    pub fn clear(&mut self) {
        self.tree = RTree::new();
        self.entries = 0;
        self.revision = u64::MAX;
    }
}

/// Session-side queries. All take `&self`: rebuilds are lazy and interior.
impl crate::session::DocumentSession {
    /// Rebuilds the index when the revision moved, then runs `query`.
    fn with_spatial<T>(&self, query: impl FnOnce(&RTree<IndexedObj>) -> T) -> T {
        {
            let index = self.spatial.borrow();
            if index.revision == self.current_revision() {
                return query(&index.tree);
            }
        }
        self.rebuild_spatial();
        query(&self.spatial.borrow().tree)
    }

    /// Bulk-loads the active surface's explicit world AABBs at this revision.
    fn rebuild_spatial(&self) {
        let mut cell = self.spatial.borrow_mut();
        cell.tree = RTree::new();
        cell.entries = 0;
        cell.revision = self.current_revision();
        let Some(surface_id) = self.active_surface() else {
            return;
        };
        let Ok(surface) = self.surface(surface_id) else {
            return;
        };
        // evaluated_bounds is itself memoized (F1): rebuilds stay cheap.
        let items: Vec<IndexedObj> = surface
            .objects()
            .iter()
            .enumerate()
            .filter(|(_, obj)| obj.visible && !obj.locked)
            .filter_map(|(seq, obj)| {
                let [x, y, w, h] = self
                    .cached_world_frame_bounds(obj.id)
                    .or_else(|| self.cached_world_bounds(obj.id))
                    .or_else(|| self.cached_bounds(obj.id))
                    .or(obj.bounds)?;
                Some(IndexedObj {
                    id: obj.id,
                    seq,
                    x0: x,
                    y0: y,
                    x1: x + w,
                    y1: y + h,
                })
            })
            .collect();
        cell.entries = items.len();
        cell.tree = RTree::bulk_load(items);
    }

    /// Candidate ids covering `pt`, topmost-first (for exact hit-testing).
    /// Tolerance expands the query box; exactness stays with the caller.
    #[must_use]
    pub fn spatial_candidates_point(&self, pt: GPoint, tol: f64) -> Vec<ObjectId> {
        let tol = tol.max(0.0);
        let query = AABB::from_corners([pt.x - tol, pt.y - tol], [pt.x + tol, pt.y + tol]);
        let mut hits: Vec<(usize, ObjectId)> = self.with_spatial(|tree| {
            tree.locate_in_envelope_intersecting(&query)
                .map(|o| (o.seq, o.id))
                .collect()
        });
        hits.sort_by_key(|&(seq, _)| std::cmp::Reverse(seq));
        hits.into_iter().map(|(_, id)| id).collect()
    }

    /// Candidate ids intersecting `rect` (`[x0, y0, x1, y1]`), z-ordered.
    #[must_use]
    pub fn spatial_candidates_rect(&self, rect: [f64; 4]) -> Vec<ObjectId> {
        let [x0, y0, x1, y1] = rect;
        let query = AABB::from_corners([x0.min(x1), y0.min(y1)], [x0.max(x1), y0.max(y1)]);
        let mut hits: Vec<(usize, ObjectId)> = self.with_spatial(|tree| {
            tree.locate_in_envelope_intersecting(&query)
                .map(|o| (o.seq, o.id))
                .collect()
        });
        hits.sort_by_key(|&(seq, _)| seq);
        hits.into_iter().map(|(_, id)| id).collect()
    }

    /// Index size for diagnostics and tests.
    #[must_use]
    pub fn spatial_len(&self) -> usize {
        self.with_spatial(|tree| tree.size())
    }
}
