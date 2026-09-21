//! Precision measurement tool for distance, delta, and angle (08.24, 08.27).

use aubrieta_document::ChangeSet;
use aubrieta_foundation::AubrietaError;
use aubrieta_geometry::GPoint;

use crate::bridge::AubrietaGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use aubrieta_application::interaction::{NormalizedPointerEvent, PointerButton, PointerPhase};

/// Measurement readout data (shared primitive, Table B).
pub use aubrieta_geometry::measure::MeasurementReadout;

/// Interactive measurement tool providing transient HUD dimensions without altering document state.
#[derive(Clone, Debug, Default)]
pub struct MeasureTool {
    start_doc: Option<GPoint>,
    current_doc: Option<GPoint>,
}

impl MeasureTool {
    /// Creates a fresh measurement tool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            start_doc: None,
            current_doc: None,
        }
    }

    /// Resets active measurement.
    pub fn cancel(&mut self) {
        self.start_doc = None;
        self.current_doc = None;
    }

    /// Returns the active measurement readout, if currently measuring.
    #[must_use]
    pub fn readout(&self) -> Option<MeasurementReadout> {
        let (p0, p1) = (self.start_doc?, self.current_doc?);
        Some(aubrieta_geometry::measure_readout(p0, p1))
    }

    /// Handles normalized pointer events.
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        _bridge: &mut AubrietaGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, AubrietaError> {
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

    /// Resolves overlays displaying the measurement line.
    #[must_use]
    pub fn overlays(&self) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        if let (Some(p0), Some(p1)) = (self.start_doc, self.current_doc) {
            overlays.pen_preview = Some(vec![p0, p1]);
        }
        overlays
    }
}
