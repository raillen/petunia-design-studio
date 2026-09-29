//! Precision measurement tool: distance and area (08.24, 08.27, TOOLS_DECISIONS Batch 8).
//!
//! Transient HUD readouts that never touch the document. Distance mode draws
//! a line (length, delta, angle); Area mode drags a rectangle (area,
//! perimeter) or, idle with a selection, totals selected evaluated areas.

use petunia_design_document::ChangeSet;
use petunia_design_foundation::{ObjectId, PetuniaError};
use petunia_design_geometry::{GPoint, GRect};

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{CanvasOverlays, CursorAffordance, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase,
};

/// Measurement readout data (shared primitive, Table B).
pub use petunia_design_geometry::measure::{AreaReadout, MeasurementReadout};

/// Snaps a 2D vector around p0 to 45-degree angle increments.
fn snap_linear_45(p0: GPoint, p1: GPoint) -> GPoint {
    let dx = p1.x - p0.x;
    let dy = p1.y - p0.y;
    let dist = dx.hypot(dy);
    if dist < 1e-6 {
        return p1;
    }
    let angle = dy.atan2(dx);
    let step = std::f64::consts::FRAC_PI_4;
    let snapped = (angle / step).round() * step;
    GPoint::new(p0.x + dist * snapped.cos(), p0.y + dist * snapped.sin())
}

/// What the measure tool reports.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MeasureMode {
    /// Line readout: distance, delta, angle.
    #[default]
    Distance,
    /// Rectangle readout: area and perimeter (drag), or selected total (idle).
    Area,
}

/// Interactive measurement tool providing transient HUD dimensions without altering document state.
#[derive(Clone, Debug, Default)]
pub struct MeasureTool {
    mode: MeasureMode,
    start_doc: Option<GPoint>,
    current_doc: Option<GPoint>,
}

impl MeasureTool {
    /// Creates a fresh measurement tool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            mode: MeasureMode::Distance,
            start_doc: None,
            current_doc: None,
        }
    }

    /// Current readout mode (future toolbar binding).
    #[must_use]
    pub fn mode(&self) -> MeasureMode {
        self.mode
    }

    /// Switches between distance and area reporting.
    pub fn set_mode(&mut self, mode: MeasureMode) {
        if self.mode != mode {
            self.cancel();
            self.mode = mode;
        }
    }

    /// Resets active measurement.
    pub fn cancel(&mut self) {
        self.start_doc = None;
        self.current_doc = None;
    }

    /// Returns the active distance readout, if currently measuring.
    #[must_use]
    pub fn readout(&self) -> Option<MeasurementReadout> {
        let (p0, p1) = (self.start_doc?, self.current_doc?);
        Some(petunia_design_geometry::measure_readout(p0, p1))
    }

    /// Returns the active area readout for the in-flight drag, if any.
    #[must_use]
    pub fn area_readout(&self) -> Option<AreaReadout> {
        let (p0, p1) = (self.start_doc?, self.current_doc?);
        Some(petunia_design_geometry::area_readout(p0, p1))
    }

    /// Totals evaluated bounds areas over explicit objects.
    #[must_use]
    pub fn measured_area(bridge: &PetuniaDesignGuiBridge, ids: &[ObjectId]) -> f64 {
        let Some(session) = bridge.session() else {
            return 0.0;
        };
        ids.iter()
            .filter_map(|id| session.find_object(*id))
            .filter_map(|obj| obj.evaluated_bounds())
            .map(|[_, _, w, h]| w.max(0.0) * h.max(0.0))
            .sum()
    }

    /// Handles normalized pointer events.
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        _bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
        match event.phase {
            PointerPhase::Down => {
                if event.button != PointerButton::Primary {
                    return Ok(ChangeSet::empty());
                }
                let mut pt = event.doc_pos;
                if !event.modifiers.disable_snap {
                    pt = snap.snap_point(pt, camera, &[]).point;
                }
                self.start_doc = Some(pt);
                self.current_doc = Some(pt);
                Ok(ChangeSet::empty())
            }
            PointerPhase::Move => {
                if let Some(p0) = self.start_doc {
                    let mut pt = event.doc_pos;
                    if !event.modifiers.disable_snap {
                        pt = snap.snap_point(pt, camera, &[]).point;
                    }
                    if event.modifiers.constrain {
                        match self.mode {
                            MeasureMode::Distance => {
                                pt = snap_linear_45(p0, pt);
                            }
                            MeasureMode::Area => {
                                let dx = pt.x - p0.x;
                                let dy = pt.y - p0.y;
                                let max_d = dx.abs().max(dy.abs());
                                let sx = if dx >= 0.0 { 1.0 } else { -1.0 };
                                let sy = if dy >= 0.0 { 1.0 } else { -1.0 };
                                pt = GPoint::new(p0.x + max_d * sx, p0.y + max_d * sy);
                            }
                        }
                    }
                    self.current_doc = Some(pt);
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Up => {
                // Measurements remain visible until next click or cancel
                Ok(ChangeSet::empty())
            }
            PointerPhase::Cancel => {
                self.cancel();
                Ok(ChangeSet::empty())
            }
        }
    }

    /// Resolves overlays displaying the measurement with floating HUD badge.
    /// Distance draws the line with distance/angle badge; Area draws the rectangle with W/H/Area badge.
    #[must_use]
    pub fn overlays(&self, camera: &ViewportCamera) -> CanvasOverlays {
        let mut overlays = CanvasOverlays {
            cursor: CursorAffordance::Crosshair,
            ..Default::default()
        };
        if let (Some(p0), Some(p1)) = (self.start_doc, self.current_doc) {
            match self.mode {
                MeasureMode::Distance => {
                    overlays.pen_preview = Some(vec![p0, p1]);
                    let r = petunia_design_geometry::measure_readout(p0, p1);
                    let mid = GPoint::new((p0.x + p1.x) / 2.0, (p0.y + p1.y) / 2.0);
                    let label = format!("{:.1} px  ({:.1}°)", r.distance, r.angle_deg);
                    overlays.measure_badge = Some((mid, label));
                }
                MeasureMode::Area => {
                    let min_x = p0.x.min(p1.x);
                    let min_y = p0.y.min(p1.y);
                    let max_x = p0.x.max(p1.x);
                    let max_y = p0.y.max(p1.y);
                    let s_tl = camera.doc_to_screen(GPoint::new(min_x, min_y));
                    let s_br = camera.doc_to_screen(GPoint::new(max_x, max_y));
                    overlays.marquee_screen = Some(GRect::new(s_tl.x, s_tl.y, s_br.x, s_br.y));
                    let r = petunia_design_geometry::area_readout(p0, p1);
                    let center = GPoint::new((p0.x + p1.x) / 2.0, (p0.y + p1.y) / 2.0);
                    let label = format!(
                        "W: {:.1}  H: {:.1}  |  Area: {:.1} px²",
                        r.width, r.height, r.area
                    );
                    overlays.measure_badge = Some((center, label));
                }
            }
        }
        overlays
    }
}
