//! Viewport camera with arbitrary zoom, pan, and coordinate mapping (08.6).

use petunia_design_geometry::{GPoint, GRect};

/// Minimum supported zoom level: 0.1% (0.001x).
pub const MIN_ZOOM: f64 = 0.001;
/// Maximum supported zoom level: 25600% (256.0x).
pub const MAX_ZOOM: f64 = 256.0;

/// Camera managing pan, zoom, and coordinate conversions between screen and document space.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewportCamera {
    /// Viewport width in screen pixels.
    pub viewport_width: f64,
    /// Viewport height in screen pixels.
    pub viewport_height: f64,
    /// Horizontal pan offset in screen pixels.
    pub pan_x: f64,
    /// Vertical pan offset in screen pixels.
    pub pan_y: f64,
    /// Zoom scale factor (1.0 = 100%).
    pub zoom: f64,
}

impl Default for ViewportCamera {
    fn default() -> Self {
        Self::new(1280.0, 800.0)
    }
}

impl ViewportCamera {
    /// Creates a camera with explicit screen viewport dimensions.
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            viewport_width: width.max(1.0),
            viewport_height: height.max(1.0),
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
        }
    }

    /// Resizes the screen viewport.
    pub fn resize(&mut self, width: f64, height: f64) {
        self.viewport_width = width.max(1.0);
        self.viewport_height = height.max(1.0);
    }

    /// Converts a screen coordinate (pixels) into document space (points).
    #[must_use]
    pub fn screen_to_doc(&self, screen: GPoint) -> GPoint {
        let doc_x = (screen.x - self.pan_x) / self.zoom;
        let doc_y = (screen.y - self.pan_y) / self.zoom;
        GPoint::new(doc_x, doc_y)
    }

    /// Converts a document coordinate (points) into screen space (pixels).
    #[must_use]
    pub fn doc_to_screen(&self, doc: GPoint) -> GPoint {
        let screen_x = doc.x * self.zoom + self.pan_x;
        let screen_y = doc.y * self.zoom + self.pan_y;
        GPoint::new(screen_x, screen_y)
    }

    /// Translates the camera by a screen pixel offset.
    pub fn pan(&mut self, delta_screen_x: f64, delta_screen_y: f64) {
        self.pan_x += delta_screen_x;
        self.pan_y += delta_screen_y;
    }

    /// Zooms the camera centered at a specific screen point, ensuring the document point
    /// under the cursor remains strictly stationary before and after zooming (08.6).
    pub fn zoom_at(&mut self, screen_focus: GPoint, factor: f64) {
        let target_zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        if (target_zoom - self.zoom).abs() < f64::EPSILON {
            return;
        }

        // 1. Resolve document point under cursor before zoom
        let doc_point = self.screen_to_doc(screen_focus);

        // 2. Apply new zoom
        self.zoom = target_zoom;

        // 3. Adjust pan so doc_point projects back to screen_focus
        self.pan_x = screen_focus.x - doc_point.x * self.zoom;
        self.pan_y = screen_focus.y - doc_point.y * self.zoom;
    }

    /// Sets explicit zoom factor centered at the screen center.
    pub fn set_zoom(&mut self, zoom: f64) {
        let center = GPoint::new(self.viewport_width / 2.0, self.viewport_height / 2.0);
        let factor = zoom / self.zoom;
        self.zoom_at(center, factor);
    }

    /// Resets zoom to 100% (1.0x) centered at the screen center.
    pub fn reset_100(&mut self) {
        self.set_zoom(1.0);
    }

    /// Computes the visible document rectangle currently shown in the viewport.
    #[must_use]
    pub fn visible_doc_rect(&self) -> GRect {
        let top_left = self.screen_to_doc(GPoint::new(0.0, 0.0));
        let bottom_right =
            self.screen_to_doc(GPoint::new(self.viewport_width, self.viewport_height));
        GRect::new(top_left.x, top_left.y, bottom_right.x, bottom_right.y)
    }

    /// Fits a document rectangle comfortably into the viewport with padding.
    pub fn fit_rect(&mut self, target: GRect, padding_px: f64) {
        let avail_w = (self.viewport_width - padding_px * 2.0).max(10.0);
        let avail_h = (self.viewport_height - padding_px * 2.0).max(10.0);

        let scale_x = avail_w / target.width().max(1.0);
        let scale_y = avail_h / target.height().max(1.0);
        let new_zoom = scale_x.min(scale_y).clamp(MIN_ZOOM, MAX_ZOOM);

        self.zoom = new_zoom;

        let center_doc_x = (target.x0 + target.x1) / 2.0;
        let center_doc_y = (target.y0 + target.y1) / 2.0;

        self.pan_x = (self.viewport_width / 2.0) - center_doc_x * self.zoom;
        self.pan_y = (self.viewport_height / 2.0) - center_doc_y * self.zoom;
    }
}
