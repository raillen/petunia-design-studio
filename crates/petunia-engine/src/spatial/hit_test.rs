//! Page-scoped broad phase and geometry-aware hit testing.
//! Selection excludes hidden and locked ancestors. Index entries carry
//! conservative world bounds; narrow phase runs in screen coordinates.

use crate::geometry::bounds::Bounds;
use crate::spatial::index::{SpatialEntry, SpatialIndex};
use petunia_core::{
    Appearance, AppearanceKind, Document, FillRule, ObjectId, PageId, Point, Rect, SceneItem,
    Tolerance, Transform2D, VectorPath,
};
use petunia_render_model::RenderPath;

/// Requested selection geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitTestMode {
    Bounds,
    Fill,
    Stroke,
    Any,
}

/// Pointer and scope in document coordinates; tolerance is always pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitTestRequest {
    pub page: PageId,
    pub point_document: Point,
    pub tolerance_px: f64,
    pub mode: HitTestMode,
}

/// One selectable object, reported in front-to-back paint order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    pub object: ObjectId,
    pub distance_px: f64,
}

/// Whole-document traversal for index construction. Cross-page order
/// has no visual meaning; scoped queries use [`paint_order_on_page`].
#[must_use]
pub fn paint_order(document: &Document) -> Vec<ObjectId> {
    document
        .scene
        .page_ids()
        .into_iter()
        .flat_map(|page| paint_order_on_page(document, page))
        .collect()
}

/// Depth-first paint order starts at the requested page's root list.
#[must_use]
pub fn paint_order_on_page(document: &Document, page: PageId) -> Vec<ObjectId> {
    let mut order = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut stack: Vec<_> = document
        .scene
        .page_roots(page)
        .unwrap_or_default()
        .iter()
        .copied()
        .rev()
        .collect();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        let Some(node) = document.scene.get_node(id) else {
            continue;
        };
        order.push(id);
        if let Some(children) = node.item.children() {
            stack.extend(children.iter().copied().rev());
        }
    }
    order
}

fn selectable(document: &Document, id: ObjectId) -> bool {
    std::iter::once(id)
        .chain(document.scene.ancestors(id))
        .all(|ancestor| {
            document
                .scene
                .get_node(ancestor)
                .is_some_and(|node| node.visible && !node.locked)
        })
}

/// Identity-view convenience entry point: one document unit is one pixel.
#[must_use]
pub fn hit_test(
    document: &Document,
    index: &dyn SpatialIndex,
    request: HitTestRequest,
) -> Vec<Hit> {
    hit_test_with_view(document, index, request, Transform2D::IDENTITY)
}

/// Hit-test under an affine document-to-screen view. The inverse maps
/// the screen radius to conservative document extents, while exact
/// distances are computed in screen space even for nonuniform scale.
#[must_use]
pub fn hit_test_with_view(
    document: &Document,
    index: &dyn SpatialIndex,
    request: HitTestRequest,
    view: Transform2D,
) -> Vec<Hit> {
    if !request.tolerance_px.is_finite()
        || request.tolerance_px < 0.0
        || !request.point_document.x.is_finite()
        || !request.point_document.y.is_finite()
        || !view.is_numerically_safe()
    {
        return Vec::new();
    }
    let Some(inverse) = view.inverse() else {
        return Vec::new();
    };
    let rx = request.tolerance_px * inverse.a.hypot(inverse.c);
    let ry = request.tolerance_px * inverse.b.hypot(inverse.d);
    let probe = Rect::new(
        request.point_document.x - rx,
        request.point_document.y - ry,
        2.0 * rx,
        2.0 * ry,
    );
    let mut candidates = Vec::new();
    index.query_aabb(probe, &mut candidates);
    let candidates: std::collections::HashSet<_> = candidates.into_iter().collect();
    paint_order_on_page(document, request.page)
        .into_iter()
        .rev()
        .filter(|id| candidates.contains(id) && selectable(document, *id))
        .filter_map(|id| {
            narrow_hit(document, id, request, view).map(|distance_px| Hit {
                object: id,
                distance_px,
            })
        })
        .collect()
}

fn local_tolerance(transform: Transform2D) -> Option<Tolerance> {
    // Frobenius norm bounds the largest singular value, so flattening
    // error after transformation is at most 0.1 screen pixel.
    let scale =
        (transform.a.powi(2) + transform.b.powi(2) + transform.c.powi(2) + transform.d.powi(2))
            .sqrt();
    Tolerance::new((0.1 / scale.max(1e-12)).clamp(1e-12, 0.1)).ok()
}

fn geometry(
    document: &Document,
    id: ObjectId,
    tolerance: Tolerance,
) -> Option<(VectorPath, &Appearance)> {
    let node = document.scene.get_node(id)?;
    match &node.item {
        SceneItem::Path(object) => Some((object.path.clone(), &object.appearance)),
        SceneItem::Shape(object) => Some((
            crate::compile::evaluate_shape(object.shape, tolerance),
            &object.appearance,
        )),
        _ => None,
    }
}

fn flattened(path: &VectorPath, tolerance: Tolerance) -> RenderPath {
    let mut result = RenderPath::new();
    result.fill_rule = path.fill_rule;
    for contour in &path.contours {
        result.push_contour(
            crate::geometry::bezier::try_flatten_contour(contour, tolerance, 1 << 18)
                .unwrap_or_default()
                .into_iter()
                .map(|point| (point.x, point.y))
                .collect(),
            contour.closed,
        );
    }
    result
}

fn transformed(path: &RenderPath, transform: Transform2D) -> Vec<Vec<Point>> {
    path.contours
        .iter()
        .map(|ring| {
            ring.iter()
                .map(|&(x, y)| transform.transform_point(Point::new(x, y)))
                .collect()
        })
        .collect()
}

fn narrow_hit(
    document: &Document,
    id: ObjectId,
    request: HitTestRequest,
    view: Transform2D,
) -> Option<f64> {
    if request.mode == HitTestMode::Bounds {
        return item_bounds(document, id)?
            .contains(request.point_document)
            .then_some(0.0);
    }
    let transform = view.concat(document.scene.world_transform(id)?);
    let tolerance = local_tolerance(transform)?;
    let (path, appearance) = geometry(document, id, tolerance)?;
    let point = view.transform_point(request.point_document);
    if matches!(request.mode, HitTestMode::Fill | HitTestMode::Any)
        && appearance.items.iter().any(|item| {
            item.enabled && item.opacity > 0.0 && matches!(item.kind, AppearanceKind::Fill(_))
        })
    {
        let fill = flattened(&path, tolerance);
        let rings = transformed(&fill, transform);
        if in_fill(point, &rings, fill.fill_rule) {
            return Some(0.0);
        }
    }
    if matches!(request.mode, HitTestMode::Stroke | HitTestMode::Any) {
        let mut best: Option<f64> = None;
        for item in appearance
            .items
            .iter()
            .filter(|item| item.enabled && item.opacity > 0.0)
        {
            let AppearanceKind::Stroke(style) = &item.kind else {
                continue;
            };
            let stroke = crate::compile::evaluate_stroke(&path, style, tolerance);
            let rings = transformed(&stroke, transform);
            let distance = if in_fill(point, &rings, stroke.fill_rule) {
                0.0
            } else {
                boundary_distance(point, &rings)
            };
            if distance <= request.tolerance_px && best.is_none_or(|current| distance < current) {
                best = Some(distance);
            }
        }
        return best;
    }
    None
}

fn in_fill(point: Point, rings: &[Vec<Point>], rule: FillRule) -> bool {
    let mut winding = 0i64;
    for ring in rings.iter().filter(|ring| ring.len() >= 3) {
        for (a, b) in ring
            .iter()
            .zip(ring.iter().cycle().skip(1))
            .take(ring.len())
        {
            if point_segment_distance(point, *a, *b) <= 1e-10 {
                return true;
            }
            let cross = (b.x - a.x) * (point.y - a.y) - (point.x - a.x) * (b.y - a.y);
            if a.y <= point.y && b.y > point.y && cross > 0.0 {
                winding += 1;
            } else if a.y > point.y && b.y <= point.y && cross < 0.0 {
                winding -= 1;
            }
        }
    }
    match rule {
        FillRule::NonZero => winding != 0,
        FillRule::EvenOdd => winding % 2 != 0,
    }
}

fn boundary_distance(point: Point, rings: &[Vec<Point>]) -> f64 {
    rings
        .iter()
        .flat_map(|ring| {
            ring.iter()
                .zip(ring.iter().cycle().skip(1))
                .take(ring.len())
        })
        .map(|(a, b)| point_segment_distance(point, *a, *b))
        .fold(f64::INFINITY, f64::min)
}

fn point_segment_distance(point: Point, a: Point, b: Point) -> f64 {
    let length_squared = (b.x - a.x).powi(2) + (b.y - a.y).powi(2);
    if length_squared == 0.0 {
        return point.distance_to(a);
    }
    let t = (((point.x - a.x) * (b.x - a.x) + (point.y - a.y) * (b.y - a.y)) / length_squared)
        .clamp(0.0, 1.0);
    point.distance_to(Point::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y)))
}

fn item_bounds(document: &Document, id: ObjectId) -> Option<Bounds> {
    let transform = document.scene.world_transform(id)?;
    let tolerance = local_tolerance(transform)?;
    let (path, appearance) = geometry(document, id, tolerance)?;
    // Include handles in the convex hull: curve broad phase never loses
    // a bulge between anchors, regardless of flattening precision.
    let mut points: Vec<_> = path
        .contours
        .iter()
        .flat_map(|contour| contour.nodes.iter())
        .flat_map(|node| {
            std::iter::once(node.point)
                .chain(node.handle_in)
                .chain(node.handle_out)
        })
        .map(|point| transform.transform_point(point))
        .collect();
    for item in appearance
        .items
        .iter()
        .filter(|item| item.enabled && item.opacity > 0.0)
    {
        if let AppearanceKind::Stroke(style) = &item.kind {
            let outline = crate::compile::evaluate_stroke(&path, style, tolerance);
            points.extend(transformed(&outline, transform).into_iter().flatten());
        }
    }
    Bounds::of_points(&points).map(|bounds| bounds.padded(0.1))
}

/// Conservative world bounds; metadata filtering stays in the query.
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

/// Index bounds supplied by evaluated snapshot primitives. This covers
/// resource-dependent images and text without inventing authorial extents.
#[must_use]
pub fn entries_for_snapshot(snapshot: &petunia_render_model::RenderSnapshot) -> Vec<SpatialEntry> {
    let mut bounds = std::collections::BTreeMap::<ObjectId, Rect>::new();
    for page in &snapshot.pages {
        snapshot_leaves(&page.primitives, &mut Vec::new(), &mut |primitive, _| {
            let (source, rect) = primitive_bounds(primitive);
            bounds
                .entry(source)
                .and_modify(|old| *old = old.union(rect))
                .or_insert(rect);
        });
    }
    bounds
        .into_iter()
        .map(|(object, bounds)| SpatialEntry { object, bounds })
        .collect()
}

/// Resource-dependent hit testing consumes the same evaluated snapshot
/// used by rendering. Hidden/locked/page rules still come from Document.
/// Image selection uses the transformed source rectangle; alpha-aware
/// pixel selection and post-effect coverage are separate future policies.
#[must_use]
pub fn hit_test_with_snapshot(
    document: &Document,
    snapshot: &petunia_render_model::RenderSnapshot,
    index: &dyn SpatialIndex,
    request: HitTestRequest,
    view: Transform2D,
) -> Vec<Hit> {
    let Some(page) = snapshot.pages.iter().find(|page| page.page == request.page) else {
        return Vec::new();
    };
    if !request.tolerance_px.is_finite()
        || request.tolerance_px < 0.0
        || !request.point_document.x.is_finite()
        || !request.point_document.y.is_finite()
        || !view.is_numerically_safe()
    {
        return Vec::new();
    }
    let Some(inverse) = view.inverse() else {
        return Vec::new();
    };
    let rx = request.tolerance_px * inverse.a.hypot(inverse.c);
    let ry = request.tolerance_px * inverse.b.hypot(inverse.d);
    let mut ids = Vec::new();
    index.query_aabb(
        Rect::new(
            request.point_document.x - rx,
            request.point_document.y - ry,
            rx * 2.0,
            ry * 2.0,
        ),
        &mut ids,
    );
    let candidates: std::collections::HashSet<_> = ids.into_iter().collect();
    let mut found = std::collections::HashMap::<ObjectId, f64>::new();
    snapshot_leaves(
        &page.primitives,
        &mut Vec::new(),
        &mut |primitive, clips| {
            let (id, bounds) = primitive_bounds(primitive);
            if !candidates.contains(&id)
                || !selectable(document, id)
                || clips
                    .iter()
                    .any(|clip| !inside_clip(request.point_document, clip))
            {
                return;
            }
            let distance =
                if request.mode == HitTestMode::Bounds {
                    bounds.contains_point(request.point_document).then_some(0.0)
                } else if document.scene.get_node(id).is_some_and(|node| {
                    matches!(node.item, SceneItem::Path(_) | SceneItem::Shape(_))
                }) {
                    narrow_hit(document, id, request, view)
                } else {
                    snapshot_narrow_hit(primitive, snapshot, request, view)
                };
            if let Some(distance) = distance {
                found
                    .entry(id)
                    .and_modify(|old| *old = old.min(distance))
                    .or_insert(distance);
            }
        },
    );
    paint_order_on_page(document, request.page)
        .into_iter()
        .rev()
        .filter_map(|object| {
            found.get(&object).map(|&distance_px| Hit {
                object,
                distance_px,
            })
        })
        .collect()
}

fn primitive_bounds(primitive: &petunia_render_model::RenderPrimitive) -> (ObjectId, Rect) {
    use petunia_render_model::RenderPrimitive;
    match primitive {
        RenderPrimitive::Vector(value) => (value.source, value.bounds),
        RenderPrimitive::Image(value) => (value.source, value.bounds),
        RenderPrimitive::Text(value) => (value.source, value.bounds),
        RenderPrimitive::Raster(value) => (value.source, value.bounds),
        RenderPrimitive::Group(value) => (value.source, value.bounds),
    }
}

fn snapshot_leaves<'a>(
    primitives: &'a [petunia_render_model::RenderPrimitive],
    clips: &mut Vec<&'a petunia_render_model::RenderClip>,
    visit: &mut impl FnMut(
        &'a petunia_render_model::RenderPrimitive,
        &[&'a petunia_render_model::RenderClip],
    ),
) {
    for primitive in primitives {
        if let petunia_render_model::RenderPrimitive::Group(group) = primitive {
            if group.opacity <= 0.0 {
                continue;
            }
            if let Some(clip) = &group.clip {
                clips.push(clip);
            }
            snapshot_leaves(&group.children, clips, visit);
            if group.clip.is_some() {
                clips.pop();
            }
        } else {
            visit(primitive, clips);
        }
    }
}

fn inside_clip(point: Point, clip: &petunia_render_model::RenderClip) -> bool {
    use petunia_render_model::RenderClip;
    match clip {
        RenderClip::Rect(rect) => rect.contains_point(point),
        RenderClip::Polygon(ring) => in_fill(
            point,
            &[ring.iter().map(|&(x, y)| Point::new(x, y)).collect()],
            FillRule::NonZero,
        ),
        RenderClip::Path(path) => in_fill(
            point,
            &transformed(path, Transform2D::IDENTITY),
            path.fill_rule,
        ),
    }
}

fn snapshot_narrow_hit(
    primitive: &petunia_render_model::RenderPrimitive,
    snapshot: &petunia_render_model::RenderSnapshot,
    request: HitTestRequest,
    view: Transform2D,
) -> Option<f64> {
    use petunia_render_model::RenderPrimitive;
    if request.mode == HitTestMode::Stroke {
        return None;
    }
    let point = view.transform_point(request.point_document);
    match primitive {
        RenderPrimitive::Vector(vector) => {
            if vector.appearance.opacity <= 0.0 {
                return None;
            }
            in_fill(
                point,
                &transformed(&vector.geometry, view.concat(vector.transform)),
                vector.geometry.fill_rule,
            )
            .then_some(0.0)
        }
        RenderPrimitive::Image(image) => {
            if image.opacity <= 0.0 {
                return None;
            }
            let resolved = snapshot.resources.images.get(&image.resource)?;
            let (min_x, min_y, max_x, max_y) = image.source_rect.map_or(
                (0.0, 0.0, resolved.width as f64, resolved.height as f64),
                |crop| {
                    (
                        resolved.width as f64 * crop.min.x,
                        resolved.height as f64 * crop.min.y,
                        resolved.width as f64 * crop.max.x,
                        resolved.height as f64 * crop.max.y,
                    )
                },
            );
            let transform = view.concat(image.transform);
            let ring = [
                Point::new(min_x, min_y),
                Point::new(max_x, min_y),
                Point::new(max_x, max_y),
                Point::new(min_x, max_y),
            ]
            .into_iter()
            .map(|point| transform.transform_point(point))
            .collect();
            in_fill(point, &[ring], FillRule::NonZero).then_some(0.0)
        }
        RenderPrimitive::Text(text) => text
            .bounds
            .contains_point(request.point_document)
            .then_some(0.0),
        RenderPrimitive::Raster(raster) => (raster.opacity > 0.0
            && raster.bounds.contains_point(request.point_document))
        .then_some(0.0),
        RenderPrimitive::Group(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::index::RStarIndex;
    use petunia_core::{SceneNode, VectorPath};

    fn document_with_square() -> (Document, ObjectId, ObjectId) {
        let mut document = Document::new("hit");
        let page = document.scene.default_page();
        let mut group = SceneNode::new_path(
            "group",
            VectorPath::new(),
            petunia_core::ParentRef::Page(page),
        );
        group.item = SceneItem::Group(Vec::new());
        let parent = group.id;
        document.scene.insert_node(group);
        let mut node = SceneNode::new_path(
            "box",
            VectorPath::rect(0.0, 0.0, 10.0, 10.0),
            petunia_core::ParentRef::Page(page),
        );
        if let SceneItem::Path(object) = &mut node.item {
            object.appearance.items.push(petunia_core::AppearanceItem {
                id: petunia_core::AppearanceItemId::new_v4(),
                enabled: true,
                opacity: 1.0,
                blend_mode: petunia_core::BlendMode::Normal,
                kind: AppearanceKind::Stroke(
                    petunia_core::StrokeStyle::new(
                        1.0,
                        petunia_core::StrokeCap::Butt,
                        petunia_core::StrokeJoin::Miter,
                        petunia_core::ColorSource::Value(petunia_core::ColorValue::Process(
                            petunia_core::ProcessColor {
                                value: petunia_core::ProcessColorValue::Rgb(petunia_core::Rgba {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    alpha: 1.0,
                                }),
                                space: petunia_core::ColorSpaceRef::Builtin(
                                    petunia_core::BuiltinColorSpace::Srgb,
                                ),
                            },
                        )),
                    )
                    .expect("stroke"),
                ),
            });
        }
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

    fn request_at(page: PageId, x: f64, y: f64, mode: HitTestMode) -> HitTestRequest {
        HitTestRequest {
            page,
            point_document: Point::new(x, y),
            tolerance_px: 2.0,
            mode,
        }
    }

    #[test]
    fn fill_hit_finds_path_topmost_first() {
        let (document, _, id) = document_with_square();
        let index = indexed(&document);
        let hits = hit_test(
            &document,
            &index,
            request_at(document.scene.default_page(), 5.0, 5.0, HitTestMode::Fill),
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].object, id);
        let miss = hit_test(
            &document,
            &index,
            request_at(document.scene.default_page(), 50.0, 50.0, HitTestMode::Fill),
        );
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
            request_at(
                document.scene.default_page(),
                11.0,
                5.0,
                HitTestMode::Stroke,
            ),
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
