//! Spatial index behind a narrow trait.
//!
//! Production queries run on an R-tree through this adapter; the
//! external tree type never crosses into Petunia APIs. A linear scan
//! exists as reference and benchmark baseline only.

use petunia_core::{ObjectId, Point, Rect};
use rstar::{PointDistance, RTree, RTreeObject, AABB};
use std::collections::HashMap;

/// One indexed entry: identity plus conservative document-space
/// bounds. Paint order, visibility and kind stay out; query them on
/// the snapshot or derived metadata instead.
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialEntry {
    pub object: ObjectId,
    pub bounds: Rect,
}

impl SpatialEntry {
    fn envelope(&self) -> AABB<[f64; 2]> {
        AABB::from_corners(
            [self.bounds.x, self.bounds.y],
            [
                self.bounds.x + self.bounds.width,
                self.bounds.y + self.bounds.height,
            ],
        )
    }
}

impl RTreeObject for SpatialEntry {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope()
    }
}

impl PointDistance for SpatialEntry {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        // Delegate to the envelope so the tree traversal and the
        // reported metric can never disagree.
        self.envelope().distance_2(point)
    }
}

/// Narrow query contract over any backing store.
pub trait SpatialIndex {
    /// Objects whose bounds touch `rect`.
    fn query_aabb(&self, rect: Rect, out: &mut Vec<ObjectId>);

    /// Objects within `max_distance` of `point`, nearest first.
    fn nearest(&self, point: Point, max_distance: f64, out: &mut Vec<ObjectId>);
}

/// Production index: R-tree behind the trait.
#[derive(Debug, Default)]
pub struct RStarIndex {
    tree: RTree<SpatialEntry>,
}

impl RStarIndex {
    /// Empty index.
    #[must_use]
    pub fn new() -> Self {
        Self { tree: RTree::new() }
    }

    /// Bulk load for large structural changes; incremental updates
    /// use [`Self::insert`] and [`Self::remove`] instead.
    #[must_use]
    pub fn bulk_load(entries: Vec<SpatialEntry>) -> Self {
        Self {
            tree: RTree::bulk_load(entries),
        }
    }

    /// Insert or replace the entry for its object.
    pub fn insert(&mut self, entry: SpatialEntry) {
        self.tree.remove(&entry);
        self.tree.insert(entry);
    }

    /// Drop the entry for `object` when present.
    pub fn remove(&mut self, object: ObjectId) {
        let found = self
            .tree
            .locate_in_envelope(&AABB::from_corners(
                [-f64::INFINITY, -f64::INFINITY],
                [f64::INFINITY, f64::INFINITY],
            ))
            .find(|entry| entry.object == object)
            .cloned();
        if let Some(entry) = found {
            self.tree.remove(&entry);
        }
    }

    /// Number of indexed entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tree.size()
    }

    /// True when nothing is indexed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tree.size() == 0
    }
}

fn envelope_of(rect: Rect) -> AABB<[f64; 2]> {
    AABB::from_corners(
        [rect.x, rect.y],
        [rect.x + rect.width, rect.y + rect.height],
    )
}

fn distance_to_envelope(point: Point, entry: &SpatialEntry) -> f64 {
    let dx = if point.x < entry.bounds.x {
        entry.bounds.x - point.x
    } else if point.x > entry.bounds.x + entry.bounds.width {
        point.x - (entry.bounds.x + entry.bounds.width)
    } else {
        0.0
    };
    let dy = if point.y < entry.bounds.y {
        entry.bounds.y - point.y
    } else if point.y > entry.bounds.y + entry.bounds.height {
        point.y - (entry.bounds.y + entry.bounds.height)
    } else {
        0.0
    };
    dx.hypot(dy)
}

fn order_by_distance_then_id(scored: &mut [(ObjectId, f64)]) {
    scored.sort_by(|a, b| {
        a.1.partial_cmp(&b.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.as_uuid().cmp(&b.0.as_uuid()))
    });
}

impl SpatialIndex for RStarIndex {
    fn query_aabb(&self, rect: Rect, out: &mut Vec<ObjectId>) {
        out.clear();
        // Intersecting, not contained: broad phase must keep every
        // candidate the narrow phase could confirm.
        out.extend(
            self.tree
                .locate_in_envelope_intersecting(&envelope_of(rect))
                .map(|entry| entry.object),
        );
        out.sort();
    }

    fn nearest(&self, point: Point, max_distance: f64, out: &mut Vec<ObjectId>) {
        out.clear();
        if !(max_distance.is_finite() && max_distance >= 0.0) {
            return;
        }
        // The tree yields increasing envelope distance; keep the
        // prefix inside range, then stabilize ties by identity.
        let mut scored: Vec<(ObjectId, f64)> = self
            .tree
            .nearest_neighbor_iter_with_distance_2(&[point.x, point.y])
            .map(|(entry, squared)| (entry.object, squared.sqrt()))
            .take_while(|(_, distance)| *distance <= max_distance)
            .collect();
        order_by_distance_then_id(&mut scored);
        out.extend(scored.into_iter().map(|(id, _)| id));
    }
}

/// Linear reference implementation: same contract, no tree. Exists
/// for tests and benchmark baselines, never for production queries.
#[derive(Debug, Default)]
pub struct LinearIndex {
    entries: HashMap<ObjectId, Rect>,
}

impl LinearIndex {
    /// Empty reference index.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Insert or replace the entry for its object.
    pub fn insert(&mut self, entry: SpatialEntry) {
        self.entries.insert(entry.object, entry.bounds);
    }

    /// Drop the entry for `object` when present.
    pub fn remove(&mut self, object: ObjectId) {
        self.entries.remove(&object);
    }
}

impl SpatialIndex for LinearIndex {
    fn query_aabb(&self, rect: Rect, out: &mut Vec<ObjectId>) {
        out.clear();
        let mut hits: Vec<ObjectId> = self
            .entries
            .iter()
            .filter(|(_, bounds)| {
                rect.x <= bounds.x + bounds.width
                    && rect.x + rect.width >= bounds.x
                    && rect.y <= bounds.y + bounds.height
                    && rect.y + rect.height >= bounds.y
            })
            .map(|(id, _)| *id)
            .collect();
        hits.sort();
        out.extend(hits);
    }

    fn nearest(&self, point: Point, max_distance: f64, out: &mut Vec<ObjectId>) {
        out.clear();
        if !(max_distance.is_finite() && max_distance >= 0.0) {
            return;
        }
        let mut scored: Vec<(ObjectId, f64)> = self
            .entries
            .iter()
            .map(|(id, bounds)| {
                let entry = SpatialEntry {
                    object: *id,
                    bounds: *bounds,
                };
                (*id, distance_to_envelope(point, &entry))
            })
            .filter(|(_, distance)| *distance <= max_distance)
            .collect();
        order_by_distance_then_id(&mut scored);
        out.extend(scored.into_iter().map(|(id, _)| id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(x: f64, y: f64) -> SpatialEntry {
        SpatialEntry {
            object: ObjectId::new_v4(),
            bounds: Rect::new(x, y, 10.0, 10.0),
        }
    }

    /// Deterministic pseudo-random points without extra dependencies.
    fn pseudo_random(count: usize) -> Vec<(f64, f64)> {
        let mut state = 0x1234_5678_9abc_def0u64;
        (0..count)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let x = ((state >> 33) as f64 / u32::MAX as f64) * 1000.0;
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let y = ((state >> 33) as f64 / u32::MAX as f64) * 1000.0;
                (x, y)
            })
            .collect()
    }

    #[test]
    fn rstar_and_linear_agree_on_queries() {
        let mut tree = RStarIndex::new();
        let mut linear = LinearIndex::new();
        for (x, y) in pseudo_random(200) {
            let item = entry(x, y);
            tree.insert(item.clone());
            linear.insert(item);
        }
        assert_eq!(tree.len(), 200);
        for (x, y) in pseudo_random(25) {
            let window = Rect::new(x - 30.0, y - 30.0, 60.0, 60.0);
            let mut from_tree = Vec::new();
            let mut from_linear = Vec::new();
            tree.query_aabb(window, &mut from_tree);
            linear.query_aabb(window, &mut from_linear);
            assert_eq!(from_tree, from_linear, "window {window:?}");
        }
    }

    #[test]
    fn remove_drops_entries_from_both() {
        let mut tree = RStarIndex::new();
        let item = entry(0.0, 0.0);
        let id = item.object;
        tree.insert(item);
        let mut out = Vec::new();
        tree.query_aabb(Rect::new(-5.0, -5.0, 20.0, 20.0), &mut out);
        assert_eq!(out, vec![id]);
        tree.remove(id);
        assert!(tree.is_empty());
        tree.query_aabb(Rect::new(-5.0, -5.0, 20.0, 20.0), &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn nearest_respects_max_distance() {
        let mut tree = RStarIndex::new();
        tree.insert(entry(100.0, 100.0));
        let mut out = Vec::new();
        tree.nearest(Point::new(0.0, 0.0), 10.0, &mut out);
        assert!(out.is_empty());
        tree.nearest(Point::new(105.0, 105.0), 10.0, &mut out);
        assert_eq!(out.len(), 1);
    }
}
