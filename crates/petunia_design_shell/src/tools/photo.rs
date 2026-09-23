//! Photo persona tools: raster marquee selections, brush, eraser, and crop (08.31, 10.9).
//!
//! MarqueeRect, MarqueeEllipse, and Lasso commit real masks into the session
//! raster selection (Replace/Add/Subtract/Intersect via Shift/Alt, Photoshop
//! convention). FloodSelect, SelectionBrush, Brush, and Eraser stay stubs:
//! they need pixel layers in the document, which do not exist yet
//! (TOOLS_DECISIONS Batch 13). Crop commits surface geometry as before.

use petunia_design_application::{SelectionMode, SelectionShape};
use petunia_design_document::ChangeSet;
use petunia_design_foundation::PetuniaError;
use petunia_design_geometry::{GPoint, GRect};
use petunia_design_raster::brush::BrushDab;

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{NormalizedPointerEvent, PointerButton, PointerPhase};

/// Click-vs-drag threshold in screen pixels (clicks clear the mask).
const CLICK_THRESHOLD_PX: f64 = 3.0;
/// Minimum screen distance between consecutive lasso samples.
const LASSO_SAMPLE_PX: f64 = 3.0;
/// Minimum lasso length to count as a gesture, not a click.
const LASSO_MIN_LENGTH_PX: f64 = 10.0;

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
    lasso_doc: Vec<GPoint>,
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
            lasso_doc: Vec::new(),
            dabs: Vec::new(),
        }
    }

    /// Current kind.
    #[must_use]
    pub fn kind(&self) -> PhotoToolKind {
        self.kind
    }

    /// Cancels active raster gesture.
    pub fn cancel(&mut self) {
        self.start_doc = None;
        self.current_doc = None;
        self.lasso_doc.clear();
        self.dabs.clear();
    }

    /// True while a gesture is in flight.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.start_doc.is_some()
    }

    /// Handles normalized pointer events for photo operations.
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
        match event.phase {
            PointerPhase::Down => {
                if event.button != PointerButton::Primary {
                    return Ok(ChangeSet::empty());
                }
                snap.reset_hysteresis();
                self.start_doc = Some(event.doc_pos);
                self.current_doc = Some(event.doc_pos);
                self.lasso_doc = vec![event.doc_pos];
                self.dabs.clear();

                if matches!(self.kind, PhotoToolKind::Brush | PhotoToolKind::Eraser) {
                    self.record_dab(event.doc_pos);
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Move => {
                if self.start_doc.is_some() {
                    self.current_doc = Some(event.doc_pos);
                    if self.kind == PhotoToolKind::Lasso {
                        if let Some(last) = self.lasso_doc.last() {
                            if last.distance_to(event.doc_pos) >= LASSO_SAMPLE_PX {
                                self.lasso_doc.push(event.doc_pos);
                            }
                        }
                    }
                    if matches!(self.kind, PhotoToolKind::Brush | PhotoToolKind::Eraser) {
                        self.record_dab(event.doc_pos);
                    }
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Up => {
                let start = self.start_doc.take();
                let current = self.current_doc.take();
                let lasso = std::mem::take(&mut self.lasso_doc);
                snap.reset_hysteresis();

                if let (Some(p0), Some(p1)) = (start, current) {
                    match self.kind {
                        PhotoToolKind::MarqueeRect => {
                            return self.commit_marquee(p0, p1, false, event, bridge, camera);
                        }
                        PhotoToolKind::MarqueeEllipse => {
                            return self.commit_marquee(p0, p1, true, event, bridge, camera);
                        }
                        PhotoToolKind::Lasso => {
                            return self.commit_lasso(&lasso, event, bridge);
                        }
                        PhotoToolKind::Crop => {
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
                        // Pixel sampling/painting needs document pixel layers.
                        PhotoToolKind::SelectionBrush
                        | PhotoToolKind::FloodSelect
                        | PhotoToolKind::Brush
                        | PhotoToolKind::Eraser => {}
                    }
                }
                self.dabs.clear();
                Ok(ChangeSet::empty())
            }
            PointerPhase::Cancel => {
                self.cancel();
                snap.reset_hysteresis();
                Ok(ChangeSet::empty())
            }
        }
    }

    /// Commits a rectangular or elliptical marquee into the session mask.
    /// Clicks clear (Replace) or no-op (Add/Subtract), Photoshop convention.
    fn commit_marquee(
        &mut self,
        p0: GPoint,
        p1: GPoint,
        ellipse: bool,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
    ) -> Result<ChangeSet, PetuniaError> {
        let mode = SelectionMode::from_modifiers(
            event.modifiers.constrain,
            event.modifiers.duplicate,
        );
        if p0.distance_to(p1) * camera.zoom.max(0.1) <= CLICK_THRESHOLD_PX {
            if mode == SelectionMode::Replace {
                bridge.clear_raster_selection();
            }
            return Ok(ChangeSet::empty());
        }
        let shape = if ellipse {
            SelectionShape::Ellipse {
                cx: (p0.x + p1.x) / 2.0,
                cy: (p0.y + p1.y) / 2.0,
                rx: (p1.x - p0.x).abs() / 2.0,
                ry: (p1.y - p0.y).abs() / 2.0,
            }
        } else {
            SelectionShape::Rect {
                x0: p0.x,
                y0: p0.y,
                x1: p1.x,
                y1: p1.y,
            }
        };
        bridge.combine_raster_selection(shape, mode);
        // Selection is session transient state: no undo entry, no ChangeSet.
        Ok(ChangeSet::empty())
    }

    /// Commits a freehand lasso polygon into the session mask.
    fn commit_lasso(
        &mut self,
        points: &[GPoint],
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
    ) -> Result<ChangeSet, PetuniaError> {
        let mode = SelectionMode::from_modifiers(
            event.modifiers.constrain,
            event.modifiers.duplicate,
        );
        let length: f64 = points.windows(2).map(|w| w[0].distance_to(w[1])).sum();
        if points.len() < 3 || length < LASSO_MIN_LENGTH_PX {
            if mode == SelectionMode::Replace {
                bridge.clear_raster_selection();
            }
            return Ok(ChangeSet::empty());
        }
        bridge.combine_raster_selection(
            SelectionShape::Polygon(points.to_vec()),
            mode,
        );
        Ok(ChangeSet::empty())
    }

    fn record_dab(&mut self, pos: GPoint) {
        let is_eraser = self.kind == PhotoToolKind::Eraser;
        self.dabs.push(if is_eraser {
            BrushDab::eraser_dab(pos.x, pos.y)
        } else {
            BrushDab::paint_dab(pos.x, pos.y)
        });
    }

    /// Resolves overlays: in-flight gesture plus the committed mask outline.
    #[must_use]
    pub fn overlays(
        &self,
        camera: &ViewportCamera,
        bridge: &PetuniaDesignGuiBridge,
    ) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        if let (Some(p0), Some(p1)) = (self.start_doc, self.current_doc) {
            match self.kind {
                PhotoToolKind::Lasso => {
                    if self.lasso_doc.len() >= 2 {
                        overlays.lasso_screen = Some(
                            self.lasso_doc
                                .iter()
                                .map(|p| camera.doc_to_screen(*p))
                                .collect(),
                        );
                    }
                }
                PhotoToolKind::MarqueeRect
                | PhotoToolKind::MarqueeEllipse
                | PhotoToolKind::Crop => {
                    let s0 = camera.doc_to_screen(p0);
                    let s1 = camera.doc_to_screen(p1);
                    overlays.marquee_screen = Some(GRect::new(s0.x, s0.y, s1.x, s1.y));
                }
                PhotoToolKind::SelectionBrush
                | PhotoToolKind::FloodSelect
                | PhotoToolKind::Brush
                | PhotoToolKind::Eraser => {}
            }
        }
        // Committed mask (marching ants source) in document space.
        let mask = bridge.raster_selection();
        if !mask.is_empty() {
            overlays.selection_mask = Some(mask.contours.clone());
        }
        overlays
    }
}
