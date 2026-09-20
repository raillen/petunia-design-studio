//! Overlay geometry and selection handle hit-testing (08.6, 10.1).

use aubrieta_geometry::{GPoint, GRect};

use super::camera::ViewportCamera;
use super::snapping::SnapGuideVisual;

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
}
