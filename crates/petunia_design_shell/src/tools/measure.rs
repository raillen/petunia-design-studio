//! Precision measurement tool: distance and area (08.24, 08.27, TOOLS_DECISIONS Batch 8).
//!
//! Transient HUD readouts that never touch the document. Distance mode draws
//! a line (length, delta, angle); Area mode drags a rectangle (area,
//! perimeter) or, idle with a selection, totals selected evaluated areas.

use petunia_design_document::ChangeSet;
use petunia_design_foundation::{ObjectId, PetuniaError};
use petunia_design_geometry::{GPoint, GRect};

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase,
};

/// Measurement readout data (shared primitive, Table B).
pub use petunia_design_geometry::measure::{AreaReadout, MeasurementReadout};

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
                if self.start_doc.is_some() {
                    let mut pt = event.doc_pos;
                    if !event.modifiers.disable_snap {
                        pt = snap.snap_point(pt, camera, &[]).point;
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

    /// Resolves overlays displaying the measurement.
    /// Distance draws the line; Area draws the rectangle marquee.
    #[must_use]
    pub fn overlays(&self) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        if let (Some(p0), Some(p1)) = (self.start_doc, self.current_doc) {
            match self.mode {
                MeasureMode::Distance => {
                    overlays.pen_preview = Some(vec![p0, p1]);
                }
                MeasureMode::Area => {
                    overlays.marquee_screen = Some(GRect::new(p0.x, p0.y, p1.x, p1.y));
                }
            }
        }
        overlays
    }
}
