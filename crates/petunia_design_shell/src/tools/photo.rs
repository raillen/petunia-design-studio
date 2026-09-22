//! Photo persona tools: raster marquee selections, brush, eraser, and crop (08.31, 10.9).

use petunia_design_document::ChangeSet;
use petunia_design_foundation::PetuniaError;
use petunia_design_geometry::{GPoint, GRect};
use petunia_design_raster::brush::BrushDab;

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase,
};

/// Operational mode for photo persona tools (10.9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhotoToolKind {
    /// Rectangular raster marquee selection.
    MarqueeRect,
    /// Elliptical raster marquee selection.
    MarqueeEllipse,
    /// Freehand / Lasso raster selection.
    Lasso,
    /// Edge-snapping painted raster selection.
    SelectionBrush,
    /// Flood color tolerance selection.
    FloodSelect,
    /// Raster paint brush stamping dabs onto pixel layer or mask.
    Brush,
    /// Raster alpha eraser.
    Eraser,
    /// Surface/document crop tool.
    Crop,
}

/// Interactive Photo Persona tool handling selections, raster brushes, and cropping.
#[derive(Clone, Debug)]
pub struct PhotoTool {
    kind: PhotoToolKind,
    start_doc: Option<GPoint>,
    current_doc: Option<GPoint>,
    dabs: Vec<BrushDab>,
}

impl PhotoTool {
    /// Creates a photo tool for a specific raster mode.
    #[must_use]
    pub fn new(kind: PhotoToolKind) -> Self {
        Self {
            kind,
            start_doc: None,
            current_doc: None,
            dabs: Vec::new(),
        }
    }

    /// Cancels active raster gesture.
    pub fn cancel(&mut self) {
        self.start_doc = None;
        self.current_doc = None;
        self.dabs.clear();
    }

    /// Handles normalized pointer events for photo operations.
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
        _camera: &ViewportCamera,
        _snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
        match event.phase {
            PointerPhase::Down => {
                if event.button != PointerButton::Primary {
                    return Ok(ChangeSet::empty());
                }
                self.start_doc = Some(event.doc_pos);
                self.current_doc = Some(event.doc_pos);
                self.dabs.clear();

                if matches!(self.kind, PhotoToolKind::Brush | PhotoToolKind::Eraser) {
                    self.record_dab(event.doc_pos);
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Move => {
                if self.start_doc.is_some() {
                    self.current_doc = Some(event.doc_pos);
                    if matches!(self.kind, PhotoToolKind::Brush | PhotoToolKind::Eraser) {
                        self.record_dab(event.doc_pos);
                    }
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Up => {
                let start = self.start_doc.take();
                let current = self.current_doc.take();

                if let (Some(p0), Some(p1)) = (start, current) {
                    if self.kind == PhotoToolKind::Crop {
                        let active_surface = bridge
                            .session()
                            .and_then(|s| s.active_surface())
                            .ok_or_else(|| {
                                PetuniaError::invalid_input("no active surface for crop")
                            })?;

                        return bridge.submit_all(
                            "Crop surface",
                            petunia_design_application::surface_service::crop_commands(
                                active_surface,
                                [p0.x, p0.y],
                                [p1.x, p1.y],
                            ),
                        );
                    }
                }
                self.dabs.clear();
                Ok(ChangeSet::empty())
            }
            PointerPhase::Cancel => {
                self.cancel();
                Ok(ChangeSet::empty())
            }
        }
    }

    fn record_dab(&mut self, pos: GPoint) {
        let is_eraser = self.kind == PhotoToolKind::Eraser;
        self.dabs.push(if is_eraser {
            BrushDab::eraser_dab(pos.x, pos.y)
        } else {
            BrushDab::paint_dab(pos.x, pos.y)
        });
    }

    /// Resolves overlays displaying marquee selection or crop box.
    #[must_use]
    pub fn overlays(&self, camera: &ViewportCamera) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        if let (Some(p0), Some(p1)) = (self.start_doc, self.current_doc) {
            let s0 = camera.doc_to_screen(p0);
            let s1 = camera.doc_to_screen(p1);
            overlays.marquee_screen = Some(GRect::new(s0.x, s0.y, s1.x, s1.y));
        }
        overlays
    }
}
