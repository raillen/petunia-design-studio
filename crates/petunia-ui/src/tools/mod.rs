//! Tool controllers: interaction contracts without Qt.
//!
//! A controller maps pointer input into intents. It never receives
//! `&mut Document`, never owns geometry algorithms, and never commits
//! directly — it asks the session to run a Transaction. Preview and
//! Commit share the same engine math, so a gesture cannot "jump" on
//! pointer up.

use petunia_core::{NodeKind, ObjectId, PageId, PathNode, Point, VectorPath};
use petunia_engine::DocumentOp;
use serde::{Deserialize, Serialize};

use crate::context::{HandleId, HandleRef, NodeId, SegmentId};

/// One normalized pointer sample, as Qt would deliver it.
///
/// Coordinates are in view space; the tool converts with the view
/// transform before asking the session anything.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PointerSample {
    pub position: Point,
    pub pressure: f32,
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
    pub timestamp_ms: u64,
}

impl PointerSample {
    /// Convenience constructor for tests and headless runs.
    #[must_use]
    pub fn new(position: Point) -> Self {
        Self {
            position,
            pressure: 1.0,
            shift: false,
            alt: false,
            ctrl: false,
            timestamp_ms: 0,
        }
    }

    /// The same sample with modifier flags applied.
    #[must_use]
    pub fn with_shift(mut self, shift: bool) -> Self {
        self.shift = shift;
        self
    }
}

/// What the tool wants the session to do next.
///
/// Only `Commit` changes `DocumentRevision`, and it always carries a
/// full operation list so it stays atomic.
#[derive(Debug, Clone, PartialEq)]
pub enum ToolResponse {
    /// No state change.
    Idle,
    /// Hover or status hint for the status bar.
    Status(String),
    /// Overlay primitives to draw for this gesture.
    Overlay(Vec<OverlayPrimitive>),
    /// Transient preview: Session State only, never persisted.
    Preview(String),
    /// Selection changed in Session State.
    Selection(SelectionDelta),
    /// Commit a transaction with the given operations.
    Commit(Vec<DocumentOp>),
    /// A typed failure with a reason the UI can display.
    Failed(String),
}

/// A selection change requested by a tool.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectionDelta {
    ReplaceObjects(Vec<ObjectId>),
    ToggleObject(ObjectId),
    ClearObjects,
    SelectNode(NodeId),
    ToggleNode(NodeId),
    SelectSegment(SegmentId),
    ToggleHandle(HandleId),
    ClearSub,
}

/// Abstract overlay geometry the render layer styles.
#[derive(Debug, Clone, PartialEq)]
pub enum OverlayPrimitive {
    Line {
        from: Point,
        to: Point,
    },
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
    Handle {
        at: Point,
    },
    Ghost {
        /// Document-space polyline preview (rubber band, node drag).
        points: Vec<Point>,
    },
}

/// Which item kinds can open their own context on double-click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Path,
    Group,
    Text,
    Shape,
    Symbol,
    Other,
}

/// What a hit-test found, already ordered by D1 precedence:
/// `Handle > Node > Segment > Fill`.
#[derive(Debug, Clone, PartialEq)]
pub enum HitTarget {
    Handle {
        object: ObjectId,
        contour: u32,
        node: u32,
        handle: HandleRef,
    },
    Node {
        object: ObjectId,
        contour: u32,
        node: u32,
    },
    Segment {
        object: ObjectId,
        contour: u32,
        from: u32,
        to: u32,
    },
    Fill {
        object: ObjectId,
    },
}

/// View transform: document units to view pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewTransform {
    pub scale: f64,
    pub rotation: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}

impl ViewTransform {
    /// View point from a document point.
    #[must_use]
    pub fn doc_to_view(&self, point: Point) -> Point {
        let (sin, cos) = self.rotation.sin_cos();
        Point::new(
            (point.x * cos - point.y * sin) * self.scale + self.offset_x,
            (point.x * sin + point.y * cos) * self.scale + self.offset_y,
        )
    }

    /// Document point from a view point.
    #[must_use]
    pub fn view_to_doc(&self, point: Point) -> Point {
        let x = (point.x - self.offset_x) / self.scale;
        let y = (point.y - self.offset_y) / self.scale;
        let (sin, cos) = self.rotation.sin_cos();
        Point::new(x * cos + y * sin, -x * sin + y * cos)
    }
}

/// Narrow services a tool may use. No `&mut Document`, no Qt.
pub trait ToolServices {
    /// Hit-test at a view point, honoring D1 precedence.
    fn hit_test(&self, view_point: Point) -> Vec<HitTarget>;

    /// Snap a document point; returns the corrected point and the
    /// candidate kind it snapped to.
    fn snap(&self, point: Point) -> (Point, Option<String>);

    /// Root objects whose geometric bounds sit fully inside the
    /// view-space rect, in z-order. Decision A: containment.
    fn marquee_select(&self, min: Point, max: Point) -> Vec<ObjectId>;
}

/// Narrow session surface a controller drives.
///
/// Implemented directly by the session so its data can be borrowed
/// freely — no indirection, no `unsafe`.
pub trait ToolSession {
    /// Hit-test and snap services.
    fn services(&self) -> &dyn ToolServices;
    /// Current view transform.
    fn view(&self) -> ViewTransform;
    /// Current object selection, topmost first.
    fn selected_objects(&self) -> Vec<ObjectId>;
    /// Replace object selection, dropping stale sub-selection.
    fn select_objects(&mut self, objects: Vec<ObjectId>);
    /// Toggle one object.
    fn toggle_object(&mut self, id: ObjectId);
    /// Select one node, replacing the sub-selection.
    fn select_node(&mut self, id: NodeId);
    /// Toggle one node in the sub-selection.
    fn toggle_node(&mut self, id: NodeId);
    /// Toggle one handle in the sub-selection.
    fn toggle_handle(&mut self, id: HandleId);
    /// Clear only the sub-selection.
    fn clear_sub_selection(&mut self);
    /// Whether the object may be edited (visible, unlocked, in scope).
    fn is_editable(&self, id: ObjectId) -> bool;
    /// Sub-selected nodes by stable id, in selection order.
    fn selected_nodes(&self) -> Vec<NodeId>;
    /// A cloned path for building node-move operations. Read-only:
    /// the tool never writes through it.
    fn path_snapshot(&self, object: ObjectId) -> Option<VectorPath>;
    /// Page new root objects declare.
    fn default_page(&self) -> PageId;
    /// Root object count, for append indices.
    fn root_count(&self) -> usize;
    /// Capture the pointer, holding the base revision.
    fn capture(&mut self);
    /// Release the capture.
    fn release(&mut self);
    /// Whether a pointer interaction is captured.
    fn is_captured(&self) -> bool;
}

/// The controller contract.
pub trait ToolController {
    /// Pointer went down.
    fn begin(&mut self, ctx: &mut dyn ToolSession, sample: &PointerSample) -> ToolResponse;
    /// Pointer moved.
    fn update(&mut self, ctx: &mut dyn ToolSession, sample: &PointerSample) -> ToolResponse;
    /// Pointer released.
    fn end(&mut self, ctx: &mut dyn ToolSession, sample: &PointerSample) -> ToolResponse;
    /// Cancellation (Escape): discard preview, never commit.
    fn cancel(&mut self, ctx: &mut dyn ToolSession) -> ToolResponse;
}

/// Logical pixels a pointer must travel before a click becomes a drag.
/// A calibration parameter, not a frozen constant.
pub const DRAG_THRESHOLD_PX: f64 = 3.0;

/// Distance between two view points.
pub fn distance(a: Point, b: Point) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

fn is_drag(origin: Point, current: Point) -> bool {
    (current.x - origin.x).hypot(current.y - origin.y) > DRAG_THRESHOLD_PX
}

pub fn marquee(origin: Point, current: Point) -> OverlayPrimitive {
    OverlayPrimitive::Rect {
        x: origin.x.min(current.x),
        y: origin.y.min(current.y),
        width: (current.x - origin.x).abs(),
        height: (current.y - origin.y).abs(),
    }
}

/// Select tool: object picking, marquee replace/add, drag preview.
#[derive(Debug, Clone)]
pub struct SelectTool {
    down_at: Option<Point>,
    dragging: bool,
    additive: bool,
    object_drag: bool,
}

/// Controllers stay small and cloneable: the session owns one of
/// each and clones it out per event, so gesture state (press origin,
/// drag flag, pen anchors) survives from Down to Up without shared
/// mutable borrows.
impl SelectTool {
    /// Fresh Select tool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            down_at: None,
            dragging: false,
            additive: false,
            object_drag: false,
        }
    }
}

impl Default for SelectTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolController for SelectTool {
    fn begin(&mut self, ctx: &mut dyn ToolSession, sample: &PointerSample) -> ToolResponse {
        self.down_at = Some(sample.position);
        self.dragging = false;
        self.additive = sample.shift;
        self.object_drag = false;

        let target = ctx
            .services()
            .hit_test(sample.position)
            .into_iter()
            .find(|hit| matches!(hit, HitTarget::Fill { .. } | HitTarget::Segment { .. }));

        match target {
            Some(HitTarget::Fill { object }) | Some(HitTarget::Segment { object, .. }) => {
                if !ctx.is_editable(object) {
                    return ToolResponse::Failed(format!("objeto {object} está travado ou oculto"));
                }
                self.object_drag = !self.additive;
                if self.additive {
                    ToolResponse::Selection(SelectionDelta::ToggleObject(object))
                } else if ctx.selected_objects().contains(&object) {
                    ToolResponse::Idle
                } else {
                    ToolResponse::Selection(SelectionDelta::ReplaceObjects(vec![object]))
                }
            }
            _ => {
                if self.additive || ctx.selected_objects().is_empty() {
                    ToolResponse::Idle
                } else {
                    ToolResponse::Selection(SelectionDelta::ClearObjects)
                }
            }
        }
    }

    fn update(&mut self, ctx: &mut dyn ToolSession, sample: &PointerSample) -> ToolResponse {
        let Some(origin) = self.down_at else {
            return ToolResponse::Idle;
        };
        if !self.dragging && is_drag(origin, sample.position) {
            self.dragging = true;
            ctx.capture();
            if self.object_drag {
                let view = ctx.view();
                return ToolResponse::Overlay(vec![OverlayPrimitive::Line {
                    from: view.view_to_doc(origin),
                    to: view.view_to_doc(sample.position),
                }]);
            }
            return ToolResponse::Overlay(vec![marquee(origin, sample.position)]);
        }
        if self.dragging {
            if self.object_drag {
                let view = ctx.view();
                return ToolResponse::Overlay(vec![OverlayPrimitive::Line {
                    from: view.view_to_doc(origin),
                    to: view.view_to_doc(sample.position),
                }]);
            }
            return ToolResponse::Overlay(vec![marquee(origin, sample.position)]);
        }
        ToolResponse::Idle
    }

    fn end(&mut self, ctx: &mut dyn ToolSession, sample: &PointerSample) -> ToolResponse {
        let origin = self.down_at;
        self.down_at = None;
        if self.dragging {
            self.dragging = false;
            ctx.release();
            // Marquee selects by containment; the click case cleared
            // or replaced the selection back in `begin`.
            if let Some(origin) = origin {
                if self.object_drag {
                    self.object_drag = false;
                    let view = ctx.view();
                    let from = view.view_to_doc(origin);
                    let to = view.view_to_doc(sample.position);
                    let dx = to.x - from.x;
                    let dy = to.y - from.y;
                    let operations = ctx
                        .selected_objects()
                        .iter()
                        .filter(|id| ctx.is_editable(**id))
                        .map(|id| DocumentOp::MoveObjects {
                            object: *id,
                            dx,
                            dy,
                        })
                        .collect();
                    return ToolResponse::Commit(operations);
                }
                let from = origin;
                let to = sample.position;
                let min = Point::new(from.x.min(to.x), from.y.min(to.y));
                let max = Point::new(from.x.max(to.x), from.y.max(to.y));
                let objects = ctx.services().marquee_select(min, max);
                return ToolResponse::Selection(SelectionDelta::ReplaceObjects(objects));
            }
        }
        ToolResponse::Idle
    }

    fn cancel(&mut self, ctx: &mut dyn ToolSession) -> ToolResponse {
        self.down_at = None;
        self.dragging = false;
        self.additive = false;
        self.object_drag = false;
        ctx.release();
        ToolResponse::Idle
    }
}

/// Node tool: sub-select nodes and handles; a segment drag never bends.
/// Dragging moves the sub-selected nodes and commits exactly one
/// transaction on release.
#[derive(Debug, Clone)]
pub struct NodeTool {
    down_at: Option<Point>,
    dragging: bool,
    grabbed: Option<NodeId>,
}

impl NodeTool {
    /// Fresh Node tool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            down_at: None,
            dragging: false,
            grabbed: None,
        }
    }
}

impl Default for NodeTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolController for NodeTool {
    fn begin(&mut self, ctx: &mut dyn ToolSession, sample: &PointerSample) -> ToolResponse {
        self.down_at = Some(sample.position);
        self.dragging = false;

        let hits = ctx.services().hit_test(sample.position);
        match hits.first() {
            // D1: handles outrank nodes.
            Some(HitTarget::Handle {
                object,
                contour,
                node,
                handle,
            }) => {
                let id = HandleId {
                    object: *object,
                    contour: *contour,
                    node: *node,
                    handle: *handle,
                };
                ToolResponse::Selection(SelectionDelta::ToggleHandle(id))
            }
            Some(HitTarget::Node {
                object,
                contour,
                node,
            }) => {
                let id = NodeId {
                    object: *object,
                    contour: *contour,
                    node: *node,
                };
                self.grabbed = Some(id);
                if sample.shift {
                    ToolResponse::Selection(SelectionDelta::ToggleNode(id))
                } else {
                    ToolResponse::Selection(SelectionDelta::SelectNode(id))
                }
            }
            Some(HitTarget::Segment { .. }) => {
                ToolResponse::Status("segmento: ative Bend para deformar".to_string())
            }
            _ => {
                if sample.shift {
                    ToolResponse::Idle
                } else {
                    ToolResponse::Selection(SelectionDelta::ClearSub)
                }
            }
        }
    }

    fn update(&mut self, ctx: &mut dyn ToolSession, sample: &PointerSample) -> ToolResponse {
        let Some(origin) = self.down_at else {
            return ToolResponse::Idle;
        };
        if !self.dragging && is_drag(origin, sample.position) {
            self.dragging = true;
            ctx.capture();
            return ToolResponse::Preview("movendo node".to_string());
        }
        ToolResponse::Idle
    }

    fn end(&mut self, ctx: &mut dyn ToolSession, sample: &PointerSample) -> ToolResponse {
        let origin = self.down_at;
        let grabbed = self.grabbed;
        self.down_at = None;
        self.dragging = false;
        self.grabbed = None;
        let (Some(origin), Some(grabbed)) = (origin, grabbed) else {
            return ToolResponse::Idle;
        };
        if !is_drag(origin, sample.position) {
            return ToolResponse::Idle;
        }
        ctx.release();
        // Move the sub-selection of the grabbed contour; a bare grab
        // without sub-selection moves the grabbed node itself.
        let view = ctx.view();
        let from = view.view_to_doc(origin);
        let to = view.view_to_doc(sample.position);
        let (dx, dy) = (to.x - from.x, to.y - from.y);
        if dx == 0.0 && dy == 0.0 {
            return ToolResponse::Idle;
        }
        let mut targets: Vec<u32> = ctx
            .selected_nodes()
            .into_iter()
            .filter(|node| node.object == grabbed.object && node.contour == grabbed.contour)
            .map(|node| node.node)
            .collect();
        if targets.is_empty() {
            targets.push(grabbed.node);
        }
        targets.sort_unstable();
        targets.dedup();
        let Some(mut path) = ctx.path_snapshot(grabbed.object) else {
            return ToolResponse::Failed(format!("object {} has no path", grabbed.object));
        };
        let Some(contour) = path.contours.get_mut(grabbed.contour as usize) else {
            return ToolResponse::Failed("contour is gone".to_string());
        };
        for index in targets {
            let Some(node) = contour.nodes.get_mut(index as usize) else {
                continue;
            };
            node.point = Point::new(node.point.x + dx, node.point.y + dy);
            if let Some(handle) = node.handle_in.as_mut() {
                *handle = Point::new(handle.x + dx, handle.y + dy);
            }
            if let Some(handle) = node.handle_out.as_mut() {
                *handle = Point::new(handle.x + dx, handle.y + dy);
            }
        }
        ToolResponse::Commit(vec![petunia_engine::DocumentOp::ReplacePath {
            object: grabbed.object,
            path,
        }])
    }

    fn cancel(&mut self, ctx: &mut dyn ToolSession) -> ToolResponse {
        self.down_at = None;
        self.dragging = false;
        self.grabbed = None;
        ctx.release();
        ToolResponse::Idle
    }
}

/// Pen tool: click anchors into a polyline, click the first anchor to
/// close, Escape to cancel. Clicks only stage Session State; closing
/// commits exactly one transaction.
#[derive(Debug, Clone)]
pub struct PenTool {
    points: Vec<Point>,
}

impl PenTool {
    /// Fresh Pen tool.
    #[must_use]
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }

    /// Anchors staged so far, in document space.
    #[must_use]
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// Explicit Enter completes an open path, without inventing a closure.
    pub fn finish(&mut self, ctx: &mut dyn ToolSession) -> ToolResponse {
        if self.points.len() < 2 {
            return ToolResponse::Failed(
                "adicione pelo menos dois pontos antes de confirmar".into(),
            );
        }
        self.commit_path(ctx, false)
    }

    fn close_tolerance(view: ViewTransform) -> f64 {
        8.0 / view.scale.max(1e-9)
    }

    fn commit_path(&mut self, ctx: &mut dyn ToolSession, closed: bool) -> ToolResponse {
        use petunia_core::{Appearance, SceneItem, SceneNode, Transform2D};
        let mut contour = petunia_core::Contour::new(closed);
        for point in &self.points {
            contour.push_node(PathNode::line(*point, NodeKind::Cusp));
        }
        self.points.clear();
        ctx.release();
        ToolResponse::Commit(vec![petunia_engine::DocumentOp::InsertRoot {
            index: ctx.root_count(),
            node: Box::new(SceneNode {
                id: petunia_core::ObjectId::new_v4(),
                name: "Path".to_string(),
                parent: petunia_core::ParentRef::Page(ctx.default_page()),
                visible: true,
                locked: false,
                transform: Transform2D::IDENTITY,
                opacity: 1.0,
                geometry_effects: Default::default(),
                post_effects: Default::default(),
                clip: None,
                mask: None,
                item: SceneItem::Path(petunia_core::PathObject {
                    path: {
                        let mut path = VectorPath::new();
                        path.push_contour(contour);
                        path
                    },
                    appearance: Appearance::solid_black(),
                }),
            }),
        }])
    }
}

impl Default for PenTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolController for PenTool {
    fn begin(&mut self, ctx: &mut dyn ToolSession, sample: &PointerSample) -> ToolResponse {
        let view = ctx.view();
        let (snapped, _) = ctx.services().snap(view.view_to_doc(sample.position));
        if self.points.len() >= 3
            && distance(snapped, self.points[0]) <= Self::close_tolerance(view)
        {
            return self.commit_path(ctx, true);
        }
        if let Some(last) = self.points.last() {
            if distance(snapped, *last) <= 1e-9 {
                return ToolResponse::Idle;
            }
        }
        self.points.push(snapped);
        ctx.capture();
        ToolResponse::Status(format!("path with {} anchors", self.points.len()))
    }

    fn update(&mut self, ctx: &mut dyn ToolSession, sample: &PointerSample) -> ToolResponse {
        if self.points.is_empty() {
            return ToolResponse::Idle;
        }
        let view = ctx.view();
        let (snapped, _) = ctx.services().snap(view.view_to_doc(sample.position));
        let mut preview = self.points.clone();
        preview.push(snapped);
        ToolResponse::Overlay(vec![OverlayPrimitive::Ghost { points: preview }])
    }

    fn end(&mut self, _ctx: &mut dyn ToolSession, _sample: &PointerSample) -> ToolResponse {
        ToolResponse::Idle
    }

    fn cancel(&mut self, ctx: &mut dyn ToolSession) -> ToolResponse {
        self.points.clear();
        ctx.release();
        ToolResponse::Idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubServices {
        hits: Vec<HitTarget>,
        marquee: Vec<ObjectId>,
    }

    impl ToolServices for StubServices {
        fn hit_test(&self, _view_point: Point) -> Vec<HitTarget> {
            self.hits.clone()
        }

        fn snap(&self, point: Point) -> (Point, Option<String>) {
            (point, None)
        }

        fn marquee_select(&self, _min: Point, _max: Point) -> Vec<ObjectId> {
            self.marquee.clone()
        }
    }

    struct StubSession {
        services: StubServices,
        view: ViewTransform,
        objects: Vec<ObjectId>,
        selected_node: Option<NodeId>,
        sub_nodes: Vec<NodeId>,
        toggled: Vec<NodeId>,
        captured: bool,
        editable: ObjectId,
        paths: Vec<(ObjectId, VectorPath)>,
        page: PageId,
        roots: usize,
    }

    impl ToolSession for StubSession {
        fn services(&self) -> &dyn ToolServices {
            &self.services
        }

        fn view(&self) -> ViewTransform {
            self.view
        }

        fn selected_objects(&self) -> Vec<ObjectId> {
            self.objects.clone()
        }

        fn select_objects(&mut self, objects: Vec<ObjectId>) {
            self.objects = objects;
        }

        fn toggle_object(&mut self, id: ObjectId) {
            self.objects.push(id);
        }

        fn select_node(&mut self, id: NodeId) {
            self.selected_node = Some(id);
        }

        fn toggle_node(&mut self, id: NodeId) {
            self.toggled.push(id);
        }

        fn toggle_handle(&mut self, id: HandleId) {
            self.toggled.push(NodeId {
                object: id.object,
                contour: id.contour,
                node: id.node,
            });
        }

        fn clear_sub_selection(&mut self) {}

        fn is_editable(&self, id: ObjectId) -> bool {
            id == self.editable
        }

        fn selected_nodes(&self) -> Vec<NodeId> {
            self.sub_nodes.clone()
        }

        fn path_snapshot(&self, object: ObjectId) -> Option<VectorPath> {
            self.paths
                .iter()
                .find(|(id, _)| *id == object)
                .map(|(_, path)| path.clone())
        }

        fn default_page(&self) -> PageId {
            self.page
        }

        fn root_count(&self) -> usize {
            self.roots
        }

        fn capture(&mut self) {
            self.captured = true;
        }

        fn release(&mut self) {
            self.captured = false;
        }

        fn is_captured(&self) -> bool {
            self.captured
        }
    }

    fn session(hits: Vec<HitTarget>, object: ObjectId) -> StubSession {
        StubSession {
            services: StubServices {
                hits,
                marquee: Vec::new(),
            },
            view: ViewTransform {
                scale: 2.0,
                rotation: 0.0,
                offset_x: 10.0,
                offset_y: 20.0,
            },
            objects: Vec::new(),
            selected_node: None,
            sub_nodes: Vec::new(),
            toggled: Vec::new(),
            captured: false,
            editable: object,
            paths: Vec::new(),
            page: PageId::new_v4(),
            roots: 0,
        }
    }

    #[test]
    fn select_replaces_selection_on_click() {
        let object = ObjectId::new_v4();
        let mut ctx = session(vec![HitTarget::Fill { object }], object);
        let mut tool = SelectTool::new();
        let response = tool.begin(&mut ctx, &PointerSample::new(Point::new(30.0, 40.0)));
        assert_eq!(
            response,
            ToolResponse::Selection(SelectionDelta::ReplaceObjects(vec![object]))
        );
    }

    #[test]
    fn select_shift_toggles_and_locked_targets_fail_loudly() {
        let object = ObjectId::new_v4();
        let mut ctx = session(vec![HitTarget::Fill { object }], object);
        let mut tool = SelectTool::new();
        let toggled = tool.begin(
            &mut ctx,
            &PointerSample::new(Point::new(0.0, 0.0)).with_shift(true),
        );
        assert!(matches!(
            toggled,
            ToolResponse::Selection(SelectionDelta::ToggleObject(_))
        ));

        let other = ObjectId::new_v4();
        let mut locked = session(vec![HitTarget::Fill { object: other }], object);
        let mut tool = SelectTool::new();
        let response = tool.begin(&mut locked, &PointerSample::new(Point::new(0.0, 0.0)));
        assert!(matches!(response, ToolResponse::Failed(_)), "{response:?}");
    }

    #[test]
    fn select_marquee_commits_nothing() {
        let object = ObjectId::new_v4();
        // A marquee starts on empty canvas. A hit object starts object drag.
        let mut ctx = session(Vec::new(), object);
        let mut tool = SelectTool::new();
        tool.begin(&mut ctx, &PointerSample::new(Point::new(0.0, 0.0)));
        let response = tool.update(&mut ctx, &PointerSample::new(Point::new(40.0, 50.0)));
        assert!(ctx.is_captured(), "drag must capture the pointer");
        assert!(matches!(response, ToolResponse::Overlay(_)));
        let end = tool.end(&mut ctx, &PointerSample::new(Point::new(40.0, 50.0)));
        assert!(
            !matches!(end, ToolResponse::Commit(_)),
            "marquee never commits"
        );
        assert!(!ctx.is_captured());
    }

    #[test]
    fn node_prefers_handles_over_nodes() {
        let object = ObjectId::new_v4();
        let hits = vec![
            HitTarget::Handle {
                object,
                contour: 0,
                node: 2,
                handle: HandleRef::Out,
            },
            HitTarget::Node {
                object,
                contour: 0,
                node: 2,
            },
        ];
        let mut ctx = session(hits, object);
        let mut tool = NodeTool::new();
        let response = tool.begin(&mut ctx, &PointerSample::new(Point::new(0.0, 0.0)));
        // Handle wins the tie, and the delta carries it.
        assert!(matches!(
            response,
            ToolResponse::Selection(SelectionDelta::ToggleHandle(_))
        ));
    }

    #[test]
    fn node_segment_drag_never_bends() {
        let object = ObjectId::new_v4();
        let hits = vec![HitTarget::Segment {
            object,
            contour: 0,
            from: 1,
            to: 2,
        }];
        let mut ctx = session(hits, object);
        let mut tool = NodeTool::new();
        let response = tool.begin(&mut ctx, &PointerSample::new(Point::new(0.0, 0.0)));
        assert!(matches!(response, ToolResponse::Status(_)), "{response:?}");
        // Dragging over the segment still never produces a Commit.
        let dragged = tool.update(&mut ctx, &PointerSample::new(Point::new(30.0, 30.0)));
        assert!(!matches!(dragged, ToolResponse::Commit(_)));
    }

    #[test]
    fn node_drag_commits_one_moved_path() {
        use petunia_core::{Contour, NodeKind, PathNode};
        let object = ObjectId::new_v4();
        let mut contour = Contour::new(false);
        contour.push_node(PathNode::new(Point::new(10.0, 10.0), NodeKind::Cusp));
        contour.push_node(PathNode::new(Point::new(30.0, 10.0), NodeKind::Cusp));
        let mut path = VectorPath::new();
        path.push_contour(contour);
        let mut ctx = session(
            vec![HitTarget::Node {
                object,
                contour: 0,
                node: 1,
            }],
            object,
        );
        ctx.paths.push((object, path));
        ctx.sub_nodes.push(NodeId {
            object,
            contour: 0,
            node: 1,
        });
        let mut tool = NodeTool::new();
        // View scale is 2 with offset (10, 20): doc = (view - off) / 2.
        // Node 1 sits at doc (30, 10) -> view (70, 40).
        tool.begin(&mut ctx, &PointerSample::new(Point::new(70.0, 40.0)));
        tool.update(&mut ctx, &PointerSample::new(Point::new(90.0, 60.0)));
        let end = tool.end(&mut ctx, &PointerSample::new(Point::new(90.0, 60.0)));
        // Drag delta is doc (10, 10): the node travels with handles.
        let petunia_engine::DocumentOp::ReplacePath {
            object: moved,
            path,
        } = single_commit(end)
        else {
            panic!("expected one ReplacePath commit");
        };
        assert_eq!(moved, object);
        assert_eq!(path.contours[0].nodes[1].point, Point::new(40.0, 20.0));
        assert_eq!(path.contours[0].nodes[0].point, Point::new(10.0, 10.0));
    }

    #[test]
    fn node_click_without_drag_commits_nothing() {
        let object = ObjectId::new_v4();
        let mut ctx = session(
            vec![HitTarget::Node {
                object,
                contour: 0,
                node: 0,
            }],
            object,
        );
        let mut tool = NodeTool::new();
        tool.begin(&mut ctx, &PointerSample::new(Point::new(0.0, 0.0)));
        let end = tool.end(&mut ctx, &PointerSample::new(Point::new(1.0, 1.0)));
        assert_eq!(end, ToolResponse::Idle);
    }

    #[test]
    fn marquee_end_selects_containment() {
        let first = ObjectId::new_v4();
        let second = ObjectId::new_v4();
        let mut ctx = session(vec![], first);
        ctx.services.marquee = vec![first, second];
        let mut tool = SelectTool::new();
        tool.begin(&mut ctx, &PointerSample::new(Point::new(0.0, 0.0)));
        tool.update(&mut ctx, &PointerSample::new(Point::new(40.0, 50.0)));
        let end = tool.end(&mut ctx, &PointerSample::new(Point::new(40.0, 50.0)));
        assert_eq!(
            end,
            ToolResponse::Selection(SelectionDelta::ReplaceObjects(vec![first, second]))
        );
    }

    #[test]
    fn pen_closes_on_first_anchor_with_one_commit() {
        let object = ObjectId::new_v4();
        let mut ctx = session(vec![], object);
        let mut tool = PenTool::new();
        // View scale 2, offset (10, 20): doc points are distinct.
        for view in [
            Point::new(10.0, 20.0),
            Point::new(110.0, 20.0),
            Point::new(110.0, 120.0),
        ] {
            tool.begin(&mut ctx, &PointerSample::new(view));
            tool.end(&mut ctx, &PointerSample::new(view));
        }
        assert_eq!(tool.points().len(), 3);
        // Click back on the first anchor closes the path.
        let closed = tool.begin(&mut ctx, &PointerSample::new(Point::new(10.0, 20.0)));
        let petunia_engine::DocumentOp::InsertRoot { index, node } = single_commit(closed) else {
            panic!("expected one InsertRoot commit");
        };
        assert_eq!(index, 0);
        assert_eq!(node.item_path().expect("path").contours[0].nodes.len(), 3);
        assert!(node.item_path().expect("path").contours[0].closed);
        assert!(tool.points().is_empty(), "points clear after commit");
    }

    #[test]
    fn pen_escape_cancels_without_commit() {
        let object = ObjectId::new_v4();
        let mut ctx = session(vec![], object);
        let mut tool = PenTool::new();
        tool.begin(&mut ctx, &PointerSample::new(Point::new(0.0, 0.0)));
        tool.begin(&mut ctx, &PointerSample::new(Point::new(30.0, 30.0)));
        let cancelled = tool.cancel(&mut ctx);
        assert_eq!(cancelled, ToolResponse::Idle);
        assert!(tool.points().is_empty());
    }

    fn single_commit(response: ToolResponse) -> petunia_engine::DocumentOp {
        let ToolResponse::Commit(mut operations) = response else {
            panic!("expected a commit, got {response:?}");
        };
        assert_eq!(operations.len(), 1);
        operations.pop().expect("one operation")
    }

    #[test]
    fn node_click_selects_node_id_not_index() {
        let object = ObjectId::new_v4();
        let hits = vec![HitTarget::Node {
            object,
            contour: 1,
            node: 7,
        }];
        let mut ctx = session(hits, object);
        let mut tool = NodeTool::new();
        let response = tool.begin(&mut ctx, &PointerSample::new(Point::new(0.0, 0.0)));
        // The tool reports the stable id; the session applies it.
        assert_eq!(
            response,
            ToolResponse::Selection(SelectionDelta::SelectNode(NodeId {
                object,
                contour: 1,
                node: 7
            }))
        );
    }
}
