//! Photo persona tools: raster marquee selections, brush, eraser, and crop (08.31, 10.9).

use aubrieta_application::{Command, CommandRequest};
use aubrieta_document::ChangeSet;
use aubrieta_foundation::AubrietaError;
use aubrieta_geometry::{GPoint, GRect};
use aubrieta_raster::brush::{BlendMode, BrushDab};

use crate::bridge::AubrietaGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use super::input::{NormalizedPointerEvent, PointerButton, PointerPhase};

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
        bridge: &mut AubrietaGuiBridge,
        _camera: &ViewportCamera,
        _snap: &mut SnapEngine,
    ) -> Result<ChangeSet, AubrietaError> {
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
                            .and_then(|s| s.active_surface)
                            .ok_or_else(|| {
                                AubrietaError::invalid_input("no active surface for crop")
                            })?;

                        let x = p0.x.min(p1.x);
                        let y = p0.y.min(p1.y);
                        let w = (p1.x - p0.x).abs().max(10.0);
                        let h = (p1.y - p0.y).abs().max(10.0);

                        let crop_cmd = CommandRequest::new(Command::SetSurfaceGeometry {
                            surface: active_surface,
                            origin: [x, y],
                            dimensions: [w, h],
                        });
                        return bridge.submit_command(crop_cmd);
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
        self.dabs.push(BrushDab {
            center_x: pos.x,
            center_y: pos.y,
            radius: 12.0,
            hardness: 0.8,
            opacity: 1.0,
            color: if is_eraser {
                [0.0, 0.0, 0.0, 0.0]
            } else {
                [0.1, 0.1, 0.1, 1.0]
            },
            blend_mode: BlendMode::Normal,
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
