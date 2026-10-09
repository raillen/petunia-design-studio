//! Broad-plus-narrow hit testing.
//!
//! The R-tree answers "what is nearby" and exact geometry answers
//! "what was hit". Paint order comes from scene traversal, never
//! from index order. Per-type narrow phases that need derived data
//! (text layout, decoded alpha, expanded symbols) conservatively
//! fall back to bounds until their engines land.

use crate::geometry::bezier::flatten_contour;
use crate::geometry::bounds::{point_in_polygon, Bounds};
use crate::spatial::index::{SpatialEntry, SpatialIndex};
use petunia_core::{Document, ObjectId, PageId, Point, Rect, SceneItem, Tolerance, VectorPath};

/// What the narrow phase is allowed to cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitTestMode {
    /// Conservative bounds only.
    Bounds,
    /// Exact fill over flattened paths.
    Fill,
    /// Stroke centerline distance over flattened paths.
    Stroke,
    /// Fill first, then stroke.
    Any,
}

/// Screen-space hit request. `tolerance_px` arrives already converted
/// to document units upstream (see the screen-space rule); the view
/// transform itself stays session state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitTestRequest {
    pub page: PageId,
    pub point_document: Point,
    pub tolerance_px: f64,
    pub mode: HitTestMode,
}

/// One hit, topmost first in result lists.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    pub object: ObjectId,
    pub distance_px: f64,
}

/// Depth-first paint order over roots and group children. Object IDs
/// order z; the index never decides it. Nodes reachable twice (root
/// order plus group membership) appear once.
#[must_use]
pub fn paint_order(document: &Document) -> Vec<ObjectId> {
    let mut order = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut stack: Vec<ObjectId> = document.scene.root_order().iter().copied().rev().collect();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        let Some(node) = document.scene.get_node(id) else {
            continue;
        };
        order.push(id);
        if let SceneItem::Group(children) = &node.item {
            stack.extend(children.iter().copied().rev());
        }
    }
    order
}

/// Hit-test the document: broad phase through `index`, narrow phase
/// per item, results topmost first. Groups descend into children;
/// text, images, traces, generated content and symbol instances use
/// conservative bounds until their derived layouts exist.
pub fn hit_test(
    document: &Document,
    index: &dyn SpatialIndex,
    request: HitTestRequest,
) -> Vec<Hit> {
    let _ = request.page;
    let tolerance = Tolerance::new(request.tolerance_px)
        .unwrap_or(Tolerance::new(4.0).expect("constant tolerance"));
    let probe = Rect::new(
        request.point_document.x - request.tolerance_px,
        request.point_document.y - request.tolerance_px,
        request.tolerance_px * 2.0,
        request.tolerance_px * 2.0,
    );
    let mut candidates = Vec::new();
    index.query_aabb(probe, &mut candidates);
    let order = paint_order(document);
    let rank_of = |id: ObjectId| {
        order
            .iter()
            .position(|item| *item == id)
            .unwrap_or(usize::MAX)
    };
    let mut hits: Vec<(usize, Hit)> = candidates
        .into_iter()
        .filter_map(|id| {
            let node = document.scene.get_node(id)?;
            narrow_hit(document, node.item.clone(), id, request, tolerance).map(|distance| {
                (
                    rank_of(id),
                    Hit {
                        object: id,
                        distance_px: distance,
                    },
                )
            })
        })
        .collect();
    // Topmost first: later paint order wins; ties break by distance.
    hits.sort_by(|a, b| {
        b.0.cmp(&a.0).then_with(|| {
            a.1.distance_px
                .partial_cmp(&b.1.distance_px)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });
    hits.into_iter().map(|(_, hit)| hit).collect()
}

fn narrow_hit(
    document: &Document,
    item: SceneItem,
    id: ObjectId,
    request: HitTestRequest,
    tolerance: Tolerance,
) -> Option<f64> {
    match item {
        SceneItem::Path(path) => hit_path(&path, request, tolerance, document, id),
        SceneItem::Group(_) => None,
        _ => hit_bounds(document, id, request),
    }
}

fn hit_bounds(document: &Document, id: ObjectId, request: HitTestRequest) -> Option<f64> {
    let bounds = item_bounds(document, id)?;
    if bounds.contains(request.point_document) {
        Some(0.0)
    } else {
        None
    }
}

/// Conservative bounds per item. Leaves without computable bounds
/// (text layout, decoded images, expanded symbols, generated art)
/// stay out of the index until their engines provide real extents.
fn item_bounds(document: &Document, id: ObjectId) -> Option<Bounds> {
    let node = document.scene.get_node(id)?;
    match &node.item {
        SceneItem::Path(path) => path_bounds(path),
        SceneItem::Group(_)
        | SceneItem::Shape(_)
        | SceneItem::Text(_)
        | SceneItem::Image(_)
        | SceneItem::PixelLayer(_)
        | SceneItem::Trace(_)
        | SceneItem::GeneratedVector(_)
        | SceneItem::SymbolInstance(_) => None,
    }
}

fn path_bounds(path: &VectorPath) -> Option<Bounds> {
    let mut bounds: Option<Bounds> = None;
    for contour in &path.contours {
        for node in &contour.nodes {
            let point = Bounds::point(node.point);
            bounds = Some(match bounds {
                Some(existing) => existing.union(&point),
                None => point,
            });
        }
    }
    bounds
}

fn hit_path(
    path: &VectorPath,
    request: HitTestRequest,
    tolerance: Tolerance,
    document: &Document,
    id: ObjectId,
) -> Option<f64> {
    match request.mode {
        HitTestMode::Bounds => hit_bounds(document, id, request),
        HitTestMode::Fill => hit_fill(path, request),
        HitTestMode::Stroke => hit_stroke(document, id, path, request, tolerance),
        HitTestMode::Any => {
            hit_fill(path, request).or_else(|| hit_stroke(document, id, path, request, tolerance))
        }
    }
}

fn hit_fill(path: &VectorPath, request: HitTestRequest) -> Option<f64> {
    for contour in &path.contours {
        if !contour.closed || contour.nodes.len() < 3 {
            continue;
        }
        let ring: Vec<Point> = contour.nodes.iter().map(|node| node.point).collect();
        if point_in_polygon(request.point_document, &ring, path.fill_rule) {
            return Some(0.0);
        }
    }
    None
}

fn hit_stroke(
    document: &Document,
    id: ObjectId,
    path: &VectorPath,
    request: HitTestRequest,
    tolerance: Tolerance,
) -> Option<f64> {
    let node = document.scene.get_node(id)?;
    let half_width = node
        .stroke
        .as_ref()
        .map(|stroke| stroke.width / 2.0)
        .unwrap_or(0.5);
    let limit = half_width + request.tolerance_px;
    let mut best: Option<f64> = None;
    for contour in &path.contours {
        let flat = flatten_contour(contour, tolerance);
        if flat.is_empty() {
            continue;
        }
        let segments: Vec<(Point, Point)> = if contour.closed && flat.len() > 1 {
            flat.iter()
                .enumerate()
                .map(|(index, point)| (*point, flat[(index + 1) % flat.len()]))
                .collect()
        } else {
            flat.windows(2).map(|pair| (pair[0], pair[1])).collect()
        };
        for (a, b) in segments {
            let distance = point_segment_distance(request.point_document, a, b);
            if distance <= limit && best.is_none_or(|current| distance < current) {
                best = Some(distance);
            }
        }
    }
    best
}

fn point_segment_distance(point: Point, a: Point, b: Point) -> f64 {
    let length_squared = (b.x - a.x) * (b.x - a.x) + (b.y - a.y) * (b.y - a.y);
    if length_squared == 0.0 {
        return (point.x - a.x).hypot(point.y - a.y);
    }
    let t = ((point.x - a.x) * (b.x - a.x) + (point.y - a.y) * (b.y - a.y)) / length_squared;
    let t = t.clamp(0.0, 1.0);
    let near = Point::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y));
    (point.x - near.x).hypot(point.y - near.y)
}

/// Rebuild one object's index entry from its current bounds.
#[must_use]
pub fn entry_for(document: &Document, id: ObjectId) -> Option<SpatialEntry> {
    let bounds = item_bounds(document, id)?;
    Some(SpatialEntry {
        object: id,
        bounds: Rect::new(
            bounds.min.x,
            bounds.min.y,
            bounds.max.x - bounds.min.x,
            bounds.max.y - bounds.min.y,
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::index::RStarIndex;
    use petunia_core::{SceneNode, VectorPath};

    fn document_with_square() -> (Document, ObjectId, ObjectId) {
        let mut document = Document::new("hit");
        let mut group = SceneNode::new_path("group", VectorPath::new());
        group.item = SceneItem::Group(Vec::new());
        let parent = group.id;
        document.scene.insert_node(group);
        let node = SceneNode::new_path("box", VectorPath::rect(0.0, 0.0, 10.0, 10.0));
        let id = node.id;
        document.scene.insert_node(node);
        if let Some(parent_node) = document.scene.get_node_mut(parent) {
            if let SceneItem::Group(children) = &mut parent_node.item {
                children.push(id);
            }
        }
        (document, parent, id)
    }

    fn indexed(document: &Document) -> RStarIndex {
        let mut index = RStarIndex::new();
        for id in paint_order(document) {
            if let Some(entry) = entry_for(document, id) {
                index.insert(entry);
            }
        }
        index
    }

    fn request_at(x: f64, y: f64, mode: HitTestMode) -> HitTestRequest {
        HitTestRequest {
            page: PageId::new_v4(),
            point_document: Point::new(x, y),
            tolerance_px: 2.0,
            mode,
        }
    }

    #[test]
    fn fill_hit_finds_path_topmost_first() {
        let (document, _, id) = document_with_square();
        let index = indexed(&document);
        let hits = hit_test(&document, &index, request_at(5.0, 5.0, HitTestMode::Fill));
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].object, id);
        let miss = hit_test(&document, &index, request_at(50.0, 50.0, HitTestMode::Fill));
        assert!(miss.is_empty());
    }

    #[test]
    fn stroke_hit_uses_centerline_distance() {
        let (document, _, id) = document_with_square();
        let index = indexed(&document);
        // Just outside the fill, within stroke + tolerance of the edge.
        let hits = hit_test(
            &document,
            &index,
            request_at(11.0, 5.0, HitTestMode::Stroke),
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].object, id);
    }

    #[test]
    fn paint_order_never_comes_from_the_index() {
        let (document, parent, id) = document_with_square();
        let order = paint_order(&document);
        assert_eq!(order.len(), 2);
        // Roots precede their children in traversal order.
        assert_eq!(order, vec![parent, id]);
    }
}
