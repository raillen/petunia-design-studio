//! Overlay geometry and selection handle hit-testing (08.6, 10.1).

use petunia_design_foundation::ObjectId;
use petunia_design_geometry::{GAffine, GPoint, GRect};

use super::snapping::SnapGuideVisual;
use petunia_design_application::view_camera::ViewportCamera;

/// Toolkit-neutral cursor affordance requested by canvas tools and overlays.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CursorAffordance {
    #[default]
    Default,
    Pointer,
    Crosshair,
    Move,
    Grab,
    Grabbing,
    ResizeNwse,
    ResizeNesw,
    ResizeCol,
    ResizeRow,
    Rotate,
    Text,
    NotAllowed,
}

/// Handle affordance kind on a selection bounding box.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionHandleKind {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    Rotation,
    NodeCusp,
    NodeCuspSelected,
    NodeSmooth,
    NodeSmoothSelected,
    NodeSymmetric,
    NodeSymmetricSelected,
    NodeControl,
}

/// Interactive selection transform handle.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectionHandle {
    /// Handle role.
    pub kind: SelectionHandleKind,
    /// Handle anchor position in document space.
    pub doc_point: GPoint,
    /// Hit-test box in screen pixels.
    pub screen_hit_box: GRect,
}

impl SelectionHandle {
    /// Tests if a screen coordinate clicks within this handle.
    #[must_use]
    pub fn hit_test(&self, screen_pt: GPoint) -> bool {
        self.screen_hit_box.contains(screen_pt)
    }
}

/// Builder for transform handles around an oriented bounding box (OBB) or axis-aligned box.
pub fn compute_selection_handles_oriented(
    frame_bounds: [f64; 4],
    transform: GAffine,
    camera: &ViewportCamera,
    handle_size_px: f64,
) -> Vec<SelectionHandle> {
    let half_sz = handle_size_px / 2.0;
    let [_, _, w, h] = frame_bounds;

    let points = [
        (SelectionHandleKind::TopLeft, GPoint::new(0.0, 0.0)),
        (SelectionHandleKind::Top, GPoint::new(w / 2.0, 0.0)),
        (SelectionHandleKind::TopRight, GPoint::new(w, 0.0)),
        (SelectionHandleKind::Right, GPoint::new(w, h / 2.0)),
        (SelectionHandleKind::BottomRight, GPoint::new(w, h)),
        (SelectionHandleKind::Bottom, GPoint::new(w / 2.0, h)),
        (SelectionHandleKind::BottomLeft, GPoint::new(0.0, h)),
        (SelectionHandleKind::Left, GPoint::new(0.0, h / 2.0)),
        // Rotation handle 20px above top center in local orientation
        (
            SelectionHandleKind::Rotation,
            GPoint::new(w / 2.0, -(20.0 / camera.zoom)),
        ),
    ];

    points
        .into_iter()
        .map(|(kind, local_pt)| {
            let doc_pt = transform.apply(local_pt);
            let screen_pt = camera.doc_to_screen(doc_pt);
            let screen_hit_box = GRect::new(
                screen_pt.x - half_sz,
                screen_pt.y - half_sz,
                screen_pt.x + half_sz,
                screen_pt.y + half_sz,
            );
            SelectionHandle {
                kind,
                doc_point: doc_pt,
                screen_hit_box,
            }
        })
        .collect()
}

/// Builder for transform handles around an axis-aligned bounding box.
pub fn compute_selection_handles(
    doc_bounds: GRect,
    camera: &ViewportCamera,
    handle_size_px: f64,
) -> Vec<SelectionHandle> {
    compute_selection_handles_oriented(
        [
            doc_bounds.x0,
            doc_bounds.y0,
            doc_bounds.width(),
            doc_bounds.height(),
        ],
        GAffine::translate(doc_bounds.x0, doc_bounds.y0),
        camera,
        handle_size_px,
    )
}

/// Tests whether a pointer coordinate clicks on any selection transform handle or bounding box border of an oriented bounding box.
#[must_use]
pub fn hit_test_handle_or_border_oriented(
    frame_bounds: [f64; 4],
    transform: GAffine,
    screen_pt: GPoint,
    doc_pt: GPoint,
    camera: &ViewportCamera,
    handle_size_px: f64,
    border_tolerance_px: f64,
) -> Option<SelectionHandleKind> {
    // 1. Point handles (8 resize handles + rotation handle)
    let handles = compute_selection_handles_oriented(
        frame_bounds,
        transform,
        camera,
        handle_size_px.max(12.0),
    );
    for h in handles {
        if h.hit_test(screen_pt) {
            return Some(h.kind);
        }
    }

    // 2. Bounding box borders / edges projected into local space
    let inv = transform.inverse()?;
    let local_pt = inv.apply(doc_pt);
    let [_, _, w, h] = frame_bounds;

    let tol = (border_tolerance_px / camera.zoom).max(4.0);
    let in_x_range = local_pt.x >= -tol && local_pt.x <= w + tol;
    let in_y_range = local_pt.y >= -tol && local_pt.y <= h + tol;

    if in_x_range && local_pt.y.abs() <= tol {
        // Near top border
        if local_pt.x.abs() <= tol * 2.0 {
            return Some(SelectionHandleKind::TopLeft);
        } else if (local_pt.x - w).abs() <= tol * 2.0 {
            return Some(SelectionHandleKind::TopRight);
        } else {
            return Some(SelectionHandleKind::Top);
        }
    }

    if in_x_range && (local_pt.y - h).abs() <= tol {
        // Near bottom border
        if local_pt.x.abs() <= tol * 2.0 {
            return Some(SelectionHandleKind::BottomLeft);
        } else if (local_pt.x - w).abs() <= tol * 2.0 {
            return Some(SelectionHandleKind::BottomRight);
        } else {
            return Some(SelectionHandleKind::Bottom);
        }
    }

    if in_y_range && local_pt.x.abs() <= tol {
        // Near left border
        if local_pt.y.abs() <= tol * 2.0 {
            return Some(SelectionHandleKind::TopLeft);
        } else if (local_pt.y - h).abs() <= tol * 2.0 {
            return Some(SelectionHandleKind::BottomLeft);
        } else {
            return Some(SelectionHandleKind::Left);
        }
    }

    if in_y_range && (local_pt.x - w).abs() <= tol {
        // Near right border
        if local_pt.y.abs() <= tol * 2.0 {
            return Some(SelectionHandleKind::TopRight);
        } else if (local_pt.y - h).abs() <= tol * 2.0 {
            return Some(SelectionHandleKind::BottomRight);
        } else {
            return Some(SelectionHandleKind::Right);
        }
    }

    None
}

/// Tests whether a pointer coordinate clicks on any selection transform handle or bounding box border.
/// Returns the detected handle affordance kind if hit.
#[must_use]
pub fn hit_test_handle_or_border(
    doc_bounds: GRect,
    screen_pt: GPoint,
    doc_pt: GPoint,
    camera: &ViewportCamera,
    handle_size_px: f64,
    border_tolerance_px: f64,
) -> Option<SelectionHandleKind> {
    hit_test_handle_or_border_oriented(
        [
            doc_bounds.x0,
            doc_bounds.y0,
            doc_bounds.width(),
            doc_bounds.height(),
        ],
        GAffine::translate(doc_bounds.x0, doc_bounds.y0),
        screen_pt,
        doc_pt,
        camera,
        handle_size_px,
        border_tolerance_px,
    )
}

/// Gradient overlay kind for the gradient line preview.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GradientOverlayKind {
    Linear,
    Radial,
}

/// Committed gradient line plus stop handles, in screen space.
#[derive(Clone, Debug, PartialEq)]
pub struct GradientOverlay {
    /// Gradient vector start (line start or radial center).
    pub start: GPoint,
    /// Gradient vector end (line end or radial edge).
    pub end: GPoint,
    /// `(offset, handle position)` per stop.
    pub stops: Vec<(f64, GPoint)>,
    /// Resolved RGB color `[r, g, b]` in [0.0, 1.0] for each stop.
    pub stop_colors: Vec<[f32; 3]>,
    /// Geometry kind being previewed.
    pub kind: GradientOverlayKind,
}

/// Preview of one object during an uncommitted Select transform.
#[derive(Clone, Debug, PartialEq)]
pub struct TransformPreviewObject {
    /// Stable object identity.
    pub id: ObjectId,
    /// Proposed document-space bounds.
    pub bounds: [f64; 4],
    /// Proposed rotation in radians.
    pub rotation: f64,
}

/// Uncommitted transform state published to the canvas without mutating the document.
#[derive(Clone, Debug, PartialEq)]
pub struct TransformPreview {
    /// Document revision captured when the gesture began.
    pub base_revision: u64,
    /// Proposed selection frame in document space.
    pub frame: [f64; 4],
    /// Proposed object transforms.
    pub objects: Vec<TransformPreviewObject>,
}

/// Aggregate canvas overlays currently rendered over artwork.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanvasOverlays {
    /// Active uncommitted Select transform.
    pub transform_preview: Option<TransformPreview>,
    /// Active selection handles.
    pub handles: Vec<SelectionHandle>,
    /// Selection marquee rectangle in screen space, if active.
    pub marquee_screen: Option<GRect>,
    /// Snapping guide lines.
    pub snap_guides: Vec<SnapGuideVisual>,
    /// Pen tool provisional path preview points in document space.
    pub pen_preview: Option<Vec<GPoint>>,
    /// Object currently hovered by the Select tool (click feedback, hover outline).
    pub hovered_object: Option<ObjectId>,
    /// Object pressed on pointer-down by the Select tool (click feedback).
    pub pressed_object: Option<ObjectId>,
    /// Freehand lasso path in screen space, if a lasso gesture is active.
    pub lasso_screen: Option<Vec<GPoint>>,
    /// True while the active marquee/lasso adds to the selection (Shift).
    pub marquee_additive: bool,
    /// True while the active marquee/lasso removes from the selection (Alt).
    pub marquee_subtractive: bool,
    /// Committed gradient line plus stop handles, if a gradient is selected.
    pub gradient: Option<GradientOverlay>,
    /// Pending shape-builder region outline in document space, if any.
    pub region_preview: Option<Vec<GPoint>>,
    /// True while shape-builder preview is subtractive (Alt carve).
    pub region_subtractive: bool,
    /// Text-on-path span handles in document space (`[start, end]`), if any.
    pub text_path_handles: Option<Vec<GPoint>>,
    /// Committed raster selection mask contours in document space, if any.
    pub selection_mask: Option<Vec<Vec<GPoint>>>,
    /// Bézier control handle connecting lines in document space (`(anchor, control)`).
    pub node_control_lines: Vec<(GPoint, GPoint)>,
    /// In-flight path preview during vector editing.
    pub path_preview: Option<petunia_design_geometry::GPath>,
    /// In-flight raster brush stroke preview dabs (`BrushDab`).
    pub brush_preview: Option<Vec<petunia_design_raster::BrushDab>>,
    /// Immutable real-pixel draft composed by the preview worker.
    pub raster_preview_source:
        Option<std::sync::Arc<petunia_design_application::preview::PreviewSource>>,
    /// Floating measurement badge text and document position (`(doc_point, label)`), if active.
    pub measure_badge: Option<(GPoint, String)>,
    /// Active cursor affordance requested by canvas tools.
    pub cursor: CursorAffordance,
}
