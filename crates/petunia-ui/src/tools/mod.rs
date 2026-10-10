//! Tool controllers: interaction contracts without Qt.
//!
//! A controller maps pointer input into intents. It never receives
//! `&mut Document`, never owns geometry algorithms, and never commits
//! directly — it asks the session to run a Transaction. Preview and
//! Commit share the same engine math, so a gesture cannot "jump" on
//! pointer up.

use petunia_core::{ObjectId, Point};
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
    pub offset_x: f64,
    pub offset_y: f64,
}

impl ViewTransform {
    /// View point from a document point.
    #[must_use]
    pub fn doc_to_view(&self, point: Point) -> Point {
        Point::new(
            point.x * self.scale + self.offset_x,
            point.y * self.scale + self.offset_y,
        )
    }

    /// Document point from a view point.
    #[must_use]
    pub fn view_to_doc(&self, point: Point) -> Point {
        Point::new(
            (point.x - self.offset_x) / self.scale,
            (point.y - self.offset_y) / self.scale,
        )
    }
}

/// Narrow services a tool may use. No `&mut Document`, no Qt.
pub trait ToolServices {
    /// Hit-test at a view point, honoring D1 precedence.
    fn hit_test(&self, view_point: Point) -> Vec<HitTarget>;

    /// Snap a document point; returns the corrected point and the
    /// candidate kind it snapped to.
    fn snap(&self, point: Point) -> (Point, Option<String>);
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
pub struct SelectTool {
    down_at: Option<Point>,
    dragging: bool,
    additive: bool,
}

impl SelectTool {
    /// Fresh Select tool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            down_at: None,
            dragging: false,
            additive: false,
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
            return ToolResponse::Overlay(vec![marquee(origin, sample.position)]);
        }
        if self.dragging {
            return ToolResponse::Overlay(vec![marquee(origin, sample.position)]);
        }
        ToolResponse::Idle
    }

    fn end(&mut self, ctx: &mut dyn ToolSession, sample: &PointerSample) -> ToolResponse {
        self.down_at = None;
        if self.dragging {
            self.dragging = false;
            ctx.release();
            return ToolResponse::Overlay(vec![marquee_from_end(sample)]);
        }
        ToolResponse::Idle
    }

    fn cancel(&mut self, ctx: &mut dyn ToolSession) -> ToolResponse {
        self.down_at = None;
        self.dragging = false;
        self.additive = false;
        ctx.release();
        ToolResponse::Idle
    }
}

fn marquee_from_end(_sample: &PointerSample) -> OverlayPrimitive {
    OverlayPrimitive::Rect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    }
}

/// Node tool: sub-select nodes and handles; a segment drag never bends.
pub struct NodeTool {
    down_at: Option<Point>,
    dragging: bool,
}

impl NodeTool {
    /// Fresh Node tool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            down_at: None,
            dragging: false,
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

    fn end(&mut self, _ctx: &mut dyn ToolSession, _sample: &PointerSample) -> ToolResponse {
        self.down_at = None;
        self.dragging = false;
        ToolResponse::Idle
    }

    fn cancel(&mut self, ctx: &mut dyn ToolSession) -> ToolResponse {
        self.down_at = None;
        self.dragging = false;
        ctx.release();
        ToolResponse::Idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubServices {
        hits: Vec<HitTarget>,
    }

    impl ToolServices for StubServices {
        fn hit_test(&self, _view_point: Point) -> Vec<HitTarget> {
            self.hits.clone()
        }

        fn snap(&self, point: Point) -> (Point, Option<String>) {
            (point, None)
        }
    }

    struct StubSession {
        services: StubServices,
        view: ViewTransform,
        objects: Vec<ObjectId>,
        selected_node: Option<NodeId>,
        toggled: Vec<NodeId>,
        captured: bool,
        editable: ObjectId,
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
            services: StubServices { hits },
            view: ViewTransform {
                scale: 2.0,
                offset_x: 10.0,
                offset_y: 20.0,
            },
            objects: Vec::new(),
            selected_node: None,
            toggled: Vec::new(),
            captured: false,
            editable: object,
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
        let mut ctx = session(vec![HitTarget::Fill { object }], object);
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
