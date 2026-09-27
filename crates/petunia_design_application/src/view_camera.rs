//! Viewport camera with arbitrary zoom, pan, and coordinate mapping (08.6).
//!
//! This module lives in the application layer because it is GUI-agnostic view
//! state (15.B): the camera is pure geometry, so view actions can travel the
//! normal Action -> Command lane and the UI only renders the result.

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

/// Screen-space working zoom step for the `view.zoom_in`/`view.zoom_out`
/// actions (08.35 keeps toolbar zoom discrete rather than continuous).
pub const ZOOM_STEP: f64 = 1.25;

/// Non-document view state owned by an editing session.
///
/// Rulers, snapping and the command palette are view concerns (08.6, 08.2):
/// they change what the user sees and how input is interpreted, never the
/// document itself. Keeping the palette flag here is what lets
/// `ptnd.action.view.command_palette` travel the normal Action lane instead of
/// being a UI-only shortcut.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewState {
    /// Pan/zoom camera for the active viewport.
    pub camera: ViewportCamera,
    /// Whether rulers are shown around the canvas.
    pub rulers_visible: bool,
    /// Whether snapping is armed for interactive transform.
    pub snapping_enabled: bool,
    /// Whether the command palette overlay (Ctrl+K) is open.
    pub command_palette_open: bool,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            camera: ViewportCamera::default(),
            rulers_visible: true,
            snapping_enabled: true,
            command_palette_open: false,
        }
    }
}

impl ViewState {
    /// Zooms in one discrete step around the viewport centre.
    pub fn zoom_in(&mut self) {
        let focus = self.viewport_center();
        self.camera.zoom_at(focus, ZOOM_STEP);
    }

    /// Zooms out one discrete step around the viewport centre.
    pub fn zoom_out(&mut self) {
        let focus = self.viewport_center();
        self.camera.zoom_at(focus, 1.0 / ZOOM_STEP);
    }

    /// Returns the viewport centre in screen space.
    #[must_use]
    pub fn viewport_center(&self) -> GPoint {
        GPoint::new(
            self.camera.viewport_width / 2.0,
            self.camera.viewport_height / 2.0,
        )
    }

    /// Frames `target` (document space) with breathing room.
    pub fn fit_rect(&mut self, target: GRect) {
        self.camera.fit_rect(target, 24.0);
    }

    /// Toggles ruler visibility.
    pub fn toggle_rulers(&mut self) {
        self.rulers_visible = !self.rulers_visible;
    }

    /// Toggles snapping.
    pub fn toggle_snapping(&mut self) {
        self.snapping_enabled = !self.snapping_enabled;
    }

    /// Toggles the command palette overlay (08.2).
    pub fn toggle_command_palette(&mut self) {
        self.command_palette_open = !self.command_palette_open;
    }

    /// Closes the command palette overlay, if open.
    pub fn close_command_palette(&mut self) {
        self.command_palette_open = false;
    }
}

#[cfg(test)]
mod view_state_tests {
    use super::*;

    #[test]
    fn discrete_zoom_steps_are_relative_to_the_centre() {
        let mut view = ViewState::default();
        view.camera.set_zoom(1.0);
        view.zoom_in();
        assert!((view.camera.zoom - ZOOM_STEP).abs() < 1e-9);
        view.zoom_out();
        assert!((view.camera.zoom - 1.0).abs() < 1e-9);
    }

    #[test]
    fn zoom_is_clamped_at_the_documented_bounds() {
        let mut view = ViewState::default();
        for _ in 0..200 {
            view.zoom_in();
        }
        assert!(view.camera.zoom <= MAX_ZOOM);
        for _ in 0..400 {
            view.zoom_out();
        }
        assert!(view.camera.zoom >= MIN_ZOOM);
    }

    #[test]
    fn toggles_flip_without_touching_the_camera() {
        let mut view = ViewState::default();
        let camera = view.camera.clone();
        let rulers = view.rulers_visible;
        let snapping = view.snapping_enabled;
        view.toggle_rulers();
        view.toggle_snapping();
        assert_eq!(view.rulers_visible, !rulers);
        assert_eq!(view.snapping_enabled, !snapping);
        assert_eq!(view.camera, camera);
    }

    #[test]
    fn command_palette_opens_closes_and_closes_idempotently() {
        let mut view = ViewState::default();
        assert!(!view.command_palette_open);
        view.toggle_command_palette();
        assert!(view.command_palette_open);
        view.toggle_command_palette();
        assert!(!view.command_palette_open);
        view.close_command_palette();
        assert!(!view.command_palette_open);
    }

    #[test]
    fn fit_rect_frames_the_target_inside_the_viewport() {
        let mut view = ViewState::default();
        view.fit_rect(GRect::new(0.0, 0.0, 400.0, 300.0));
        assert!(view.camera.zoom > 0.0);
        let visible = view.camera.visible_doc_rect();
        assert!(visible.width() >= 400.0);
        assert!(visible.height() >= 300.0);
    }
}
