//! Snapping engine with screen-space tolerance, hysteresis, and alignment guides (08.6, 08.27).

use petunia_design_geometry::{GPoint, GRect};

use petunia_design_application::view_camera::ViewportCamera;

/// Orientation for dynamic snapping alignment lines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapOrientation {
    /// Vertical alignment line (snaps X coordinate).
    Vertical,
    /// Horizontal alignment line (snaps Y coordinate).
    Horizontal,
}

/// Visual guide line descriptor for rendering temporary HUD overlays.
#[derive(Clone, Debug, PartialEq)]
pub struct SnapGuideVisual {
    /// Line orientation.
    pub orientation: SnapOrientation,
    /// Position along snapped axis in document coordinates.
    pub position: f64,
    /// Start coordinate along the orthogonal axis.
    pub span_start: f64,
    /// End coordinate along the orthogonal axis.
    pub span_end: f64,
    /// Semantic label or measurement hint.
    pub label: Option<String>,
}

/// Snapping candidate configuration and settings.
#[derive(Clone, Debug, PartialEq)]
pub struct SnapConfig {
    /// Enable document grid snapping.
    pub grid_enabled: bool,
    /// Grid spacing in document points.
    pub grid_spacing: f64,
    /// Enable user guide snapping.
    pub guides_enabled: bool,
    /// Enable snapping to other object boundaries and centers.
    pub objects_enabled: bool,
    /// Screen-space distance tolerance in pixels to trigger snap.
    pub tolerance_px: f64,
    /// Extra screen-space distance in pixels to retain acquired snap (prevents flicker).
    pub hysteresis_px: f64,
}

impl Default for SnapConfig {
    fn default() -> Self {
        Self {
            grid_enabled: true,
            grid_spacing: 10.0,
            guides_enabled: true,
            objects_enabled: true,
            tolerance_px: 8.0,
            hysteresis_px: 4.0,
        }
    }
}

/// Result of evaluating snapping candidates for a moving point or bounds.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SnapResult {
    /// The resulting point (either snapped or unmodified).
    pub point: GPoint,
    /// Whether the X coordinate was snapped.
    pub snapped_x: bool,
    /// Whether the Y coordinate was snapped.
    pub snapped_y: bool,
    /// Overlay guides to render on screen.
    pub guides: Vec<SnapGuideVisual>,
}

/// Snapping service maintaining active snap state for hysteresis tracking.
#[derive(Clone, Debug, Default)]
pub struct SnapEngine {
    /// Configuration settings.
    pub config: SnapConfig,
    /// User guides along X axis (vertical lines).
    pub guides_x: Vec<f64>,
    /// User guides along Y axis (horizontal lines).
    pub guides_y: Vec<f64>,
    /// Previously active snap X candidate for hysteresis.
    active_snap_x: Option<f64>,
    /// Previously active snap Y candidate for hysteresis.
    active_snap_y: Option<f64>,
}

impl SnapEngine {
    /// Creates a new snapping engine with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears any hysteresis state (e.g. at the start or end of a gesture).
    pub fn reset_hysteresis(&mut self) {
        self.active_snap_x = None;
        self.active_snap_y = None;
    }

    /// Evaluates snapping for a moving point against candidates.
    pub fn snap_point(
        &mut self,
        point: GPoint,
        camera: &ViewportCamera,
        other_bounds: &[GRect],
    ) -> SnapResult {
        let tol_doc = self.config.tolerance_px / camera.zoom;
        let hyst_doc = (self.config.tolerance_px + self.config.hysteresis_px) / camera.zoom;

        let mut best_x = point.x;
        let mut best_dist_x = f64::MAX;
        let mut snapped_x = false;
        let mut guides = Vec::new();

        // 1. Evaluate Hysteresis for X: stick to previous snap if within hysteresis band
        if let Some(prev_x) = self.active_snap_x {
            let dist = (point.x - prev_x).abs();
            if dist <= hyst_doc {
                best_x = prev_x;
                snapped_x = true;
            }
        }

        if !snapped_x {
            // 2. Evaluate Guides for X
            if self.config.guides_enabled {
                for &gx in &self.guides_x {
                    let dist = (point.x - gx).abs();
                    if dist <= tol_doc && dist < best_dist_x {
                        best_x = gx;
                        best_dist_x = dist;
                        snapped_x = true;
                    }
                }
            }

            // 3. Evaluate Object Bounds for X
            if self.config.objects_enabled {
                for b in other_bounds {
                    let candidates = [b.x0, (b.x0 + b.x1) / 2.0, b.x1];
                    for &cx in &candidates {
                        let dist = (point.x - cx).abs();
                        if dist <= tol_doc && dist < best_dist_x {
                            best_x = cx;
                            best_dist_x = dist;
                            snapped_x = true;
                        }
                    }
                }
            }

            // 4. Evaluate Grid for X
            if self.config.grid_enabled && self.config.grid_spacing > 0.0 {
                let nearest =
                    (point.x / self.config.grid_spacing).round() * self.config.grid_spacing;
                let dist = (point.x - nearest).abs();
                if dist <= tol_doc && dist < best_dist_x {
                    best_x = nearest;
                    snapped_x = true;
                }
            }
        }

        // Same for Y
        let mut best_y = point.y;
        let mut best_dist_y = f64::MAX;
        let mut snapped_y = false;

        // 1. Evaluate Hysteresis for Y: stick to previous snap if within hysteresis band
        if let Some(prev_y) = self.active_snap_y {
            let dist = (point.y - prev_y).abs();
            if dist <= hyst_doc {
                best_y = prev_y;
                snapped_y = true;
            }
        }

        if !snapped_y {
            // 2. Evaluate Guides for Y
            if self.config.guides_enabled {
                for &gy in &self.guides_y {
                    let dist = (point.y - gy).abs();
                    if dist <= tol_doc && dist < best_dist_y {
                        best_y = gy;
                        best_dist_y = dist;
                        snapped_y = true;
                    }
                }
            }

            // 3. Evaluate Object Bounds for Y
            if self.config.objects_enabled {
                for b in other_bounds {
                    let candidates = [b.y0, (b.y0 + b.y1) / 2.0, b.y1];
                    for &cy in &candidates {
                        let dist = (point.y - cy).abs();
                        if dist <= tol_doc && dist < best_dist_y {
                            best_y = cy;
                            best_dist_y = dist;
                            snapped_y = true;
                        }
                    }
                }
            }

            // 4. Evaluate Grid for Y
            if self.config.grid_enabled && self.config.grid_spacing > 0.0 {
                let nearest =
                    (point.y / self.config.grid_spacing).round() * self.config.grid_spacing;
                let dist = (point.y - nearest).abs();
                if dist <= tol_doc && dist < best_dist_y {
                    best_y = nearest;
                    snapped_y = true;
                }
            }
        }

        // Update active snap state
        self.active_snap_x = if snapped_x { Some(best_x) } else { None };
        self.active_snap_y = if snapped_y { Some(best_y) } else { None };

        let vis_rect = camera.visible_doc_rect();
        if snapped_x {
            guides.push(SnapGuideVisual {
                orientation: SnapOrientation::Vertical,
                position: best_x,
                span_start: vis_rect.y0,
                span_end: vis_rect.y1,
                label: Some(format!("{:.1} pt", best_x)),
            });
        }
        if snapped_y {
            guides.push(SnapGuideVisual {
                orientation: SnapOrientation::Horizontal,
                position: best_y,
                span_start: vis_rect.x0,
                span_end: vis_rect.x1,
                label: Some(format!("{:.1} pt", best_y)),
            });
        }

        SnapResult {
            point: GPoint::new(best_x, best_y),
            snapped_x,
            snapped_y,
            guides,
        }
    }
}
