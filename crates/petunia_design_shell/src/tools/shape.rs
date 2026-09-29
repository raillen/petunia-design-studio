//! Parametric shape creation tools (10.3).

use petunia_design_document::ChangeSet;
use petunia_design_foundation::PetuniaError;
use petunia_design_geometry::{GPoint, GRect};

use crate::bridge::*;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase,
};

/// Supported parametric shape variants (10.3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShapeKind {
    Rectangle,
    Ellipse,
    Polygon,
    Star,
}

/// Interactive tool for creating parametric shapes via click-and-drag.
#[derive(Clone, Debug)]
pub struct ShapeTool {
    kind: ShapeKind,
    start_doc: Option<GPoint>,
    current_doc: Option<GPoint>,
    constrain: bool,
    from_center: bool,
}

impl ShapeTool {
    /// Creates a shape tool for a specific shape variant.
    #[must_use]
    pub fn new(kind: ShapeKind) -> Self {
        Self {
            kind,
            start_doc: None,
            current_doc: None,
            constrain: false,
            from_center: false,
        }
    }

    /// Resets any active drag.
    pub fn cancel(&mut self) {
        self.start_doc = None;
        self.current_doc = None;
        self.constrain = false;
        self.from_center = false;
    }

    /// Handles normalized pointer events.
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
                let mut pt = event.doc_pos;
                if !event.modifiers.disable_snap {
                    pt = snap.snap_point(pt, camera, &[]).point;
                }
                self.constrain = event.modifiers.constrain;
                self.from_center = event.modifiers.from_center;
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
                    self.constrain = event.modifiers.constrain;
                    self.from_center = event.modifiers.from_center;
                    self.current_doc = Some(pt);
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Up => {
                let start = self.start_doc.take();
                let current = self.current_doc.take();
                self.constrain = false;
                self.from_center = false;

                if let (Some(p0), Some(p1)) = (start, current) {
                    let mut w = (p1.x - p0.x).abs();
                    let mut h = (p1.y - p0.y).abs();

                    // If dragged less than 2px, create standard default size
                    if w < 2.0 && h < 2.0 {
                        w = 100.0;
                        h = 100.0;
                    } else if event.modifiers.constrain {
                        // Constrain 1:1 aspect ratio (square / circle)
                        let max_dim = w.max(h);
                        w = max_dim;
                        h = max_dim;
                    }

                    let (x, y) = if event.modifiers.from_center {
                        (p0.x - w / 2.0, p0.y - h / 2.0)
                    } else {
                        (p0.x.min(p1.x), p0.y.min(p1.y))
                    };

                    return self.commit_shape(bridge, [x, y, w, h]);
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Cancel => {
                self.cancel();
                Ok(ChangeSet::empty())
            }
        }
    }

    fn commit_shape(
        &mut self,
        bridge: &mut PetuniaDesignGuiBridge,
        bounds: [f64; 4],
    ) -> Result<ChangeSet, PetuniaError> {
        let active_surface = bridge
            .session()
            .and_then(|s| s.active_surface())
            .ok_or_else(|| PetuniaError::invalid_input("no active surface for shape creation"))?;

        let obj_id = bridge.next_object_id()?;

        let (name, doc_shape) = match self.kind {
            ShapeKind::Rectangle => petunia_design_document::shape_factory::rectangle(),
            ShapeKind::Ellipse => petunia_design_document::shape_factory::ellipse(),
            ShapeKind::Polygon => petunia_design_document::shape_factory::polygon(),
            ShapeKind::Star => petunia_design_document::shape_factory::star(),
        };

        // One gesture, one undo entry (F-01).
        let changes = bridge.submit_all(
            "Create shape",
            petunia_design_application::create_shape_commands(
                active_surface,
                obj_id,
                name,
                doc_shape,
                Some(bounds),
                Some(petunia_design_document::shape_factory::DEFAULT_SHAPE_FILL.to_string()),
                None,
            ),
        )?;

        // Select the newly created shape
        bridge.set_selection(vec![obj_id]);

        Ok(changes)
    }

    /// Resolves overlays for the Shape tool during drag.
    #[must_use]
    pub fn overlays(&self, camera: &ViewportCamera) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        if let (Some(p0), Some(p1)) = (self.start_doc, self.current_doc) {
            let mut w = (p1.x - p0.x).abs();
            let mut h = (p1.y - p0.y).abs();
            if self.constrain {
                let max_dim = w.max(h);
                w = max_dim;
                h = max_dim;
            }
            let (x, y) = if self.from_center {
                (p0.x - w / 2.0, p0.y - h / 2.0)
            } else {
                (p0.x.min(p1.x), p0.y.min(p1.y))
            };
            let s0 = camera.doc_to_screen(GPoint::new(x, y));
            let s1 = camera.doc_to_screen(GPoint::new(x + w, y + h));
            overlays.marquee_screen = Some(GRect::new(s0.x, s0.y, s1.x, s1.y));
        }
        overlays
    }
}
