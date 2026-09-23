//! Overlay geometry and selection handle hit-testing (08.6, 10.1).

use petunia_design_foundation::ObjectId;
use petunia_design_geometry::{GPoint, GRect};

use super::snapping::SnapGuideVisual;
use petunia_design_application::view_camera::ViewportCamera;

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

/// Builder for transform handles around an axis-aligned bounding box.
pub fn compute_selection_handles(
    doc_bounds: GRect,
    camera: &ViewportCamera,
    handle_size_px: f64,
) -> Vec<SelectionHandle> {
    let half_sz = handle_size_px / 2.0;

    let points = [
        (
            SelectionHandleKind::TopLeft,
            GPoint::new(doc_bounds.x0, doc_bounds.y0),
        ),
        (
            SelectionHandleKind::Top,
            GPoint::new(doc_bounds.x0 + doc_bounds.width() / 2.0, doc_bounds.y0),
        ),
        (
            SelectionHandleKind::TopRight,
            GPoint::new(doc_bounds.x1, doc_bounds.y0),
        ),
        (
            SelectionHandleKind::Right,
            GPoint::new(doc_bounds.x1, doc_bounds.y0 + doc_bounds.height() / 2.0),
        ),
        (
            SelectionHandleKind::BottomRight,
            GPoint::new(doc_bounds.x1, doc_bounds.y1),
        ),
        (
            SelectionHandleKind::Bottom,
            GPoint::new(doc_bounds.x0 + doc_bounds.width() / 2.0, doc_bounds.y1),
        ),
        (
            SelectionHandleKind::BottomLeft,
            GPoint::new(doc_bounds.x0, doc_bounds.y1),
        ),
        (
            SelectionHandleKind::Left,
            GPoint::new(doc_bounds.x0, doc_bounds.y0 + doc_bounds.height() / 2.0),
        ),
        // Rotation handle 20px above top center
        (
            SelectionHandleKind::Rotation,
            GPoint::new(
                doc_bounds.x0 + doc_bounds.width() / 2.0,
                doc_bounds.y0 - (20.0 / camera.zoom),
            ),
        ),
    ];

    points
        .into_iter()
        .map(|(kind, doc_pt)| {
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
    // 1. Point handles (8 resize handles + rotation handle)
    let handles = compute_selection_handles(doc_bounds, camera, handle_size_px.max(12.0));
    for h in handles {
        if h.hit_test(screen_pt) {
            return Some(h.kind);
        }
    }

    // 2. Bounding box borders / edges
    let tol = (border_tolerance_px / camera.zoom).max(4.0);
    let x0 = doc_bounds.x0;
    let y0 = doc_bounds.y0;
    let x1 = doc_bounds.x1;
    let y1 = doc_bounds.y1;

    let in_x_range = doc_pt.x >= x0 - tol && doc_pt.x <= x1 + tol;
    let in_y_range = doc_pt.y >= y0 - tol && doc_pt.y <= y1 + tol;

    if in_x_range && (doc_pt.y - y0).abs() <= tol {
        // Near top border
        if (doc_pt.x - x0).abs() <= tol * 2.0 {
            return Some(SelectionHandleKind::TopLeft);
        } else if (doc_pt.x - x1).abs() <= tol * 2.0 {
            return Some(SelectionHandleKind::TopRight);
        } else {
            return Some(SelectionHandleKind::Top);
        }
    }

    if in_x_range && (doc_pt.y - y1).abs() <= tol {
        // Near bottom border
        if (doc_pt.x - x0).abs() <= tol * 2.0 {
            return Some(SelectionHandleKind::BottomLeft);
        } else if (doc_pt.x - x1).abs() <= tol * 2.0 {
            return Some(SelectionHandleKind::BottomRight);
        } else {
            return Some(SelectionHandleKind::Bottom);
        }
    }

    if in_y_range && (doc_pt.x - x0).abs() <= tol {
        // Near left border
        if (doc_pt.y - y0).abs() <= tol * 2.0 {
            return Some(SelectionHandleKind::TopLeft);
        } else if (doc_pt.y - y1).abs() <= tol * 2.0 {
            return Some(SelectionHandleKind::BottomLeft);
        } else {
            return Some(SelectionHandleKind::Left);
        }
    }

    if in_y_range && (doc_pt.x - x1).abs() <= tol {
        // Near right border
        if (doc_pt.y - y0).abs() <= tol * 2.0 {
            return Some(SelectionHandleKind::TopRight);
        } else if (doc_pt.y - y1).abs() <= tol * 2.0 {
            return Some(SelectionHandleKind::BottomRight);
        } else {
            return Some(SelectionHandleKind::Right);
        }
    }

    None
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
    /// Geometry kind being previewed.
    pub kind: GradientOverlayKind,
}

/// Aggregate canvas overlays currently rendered over artwork.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanvasOverlays {
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
    /// Text-on-path span handles in document space (`[start, end]`), if any.
    pub text_path_handles: Option<Vec<GPoint>>,
}
