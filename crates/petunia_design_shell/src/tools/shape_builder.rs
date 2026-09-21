//! Shape Builder and Vector Flood Fill tools (08.24, 10.3).

use petunia_design_application::{Command, CommandRequest};
use petunia_design_document::ChangeSet;
use petunia_design_foundation::PetuniaError;
use petunia_design_geometry::GPoint;

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{NormalizedPointerEvent, PointerButton, PointerPhase};

/// Operational mode for region construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuilderMode {
    /// Interactive shape builder combining or subtracting candidate regions.
    ShapeBuilder,
    /// Smart Fill clicking an enclosed region to create a new filled path.
    SmartFill,
}

/// Interactive tool for constructive geometry region synthesis.
#[derive(Clone, Debug)]
pub struct ShapeBuilderTool {
    mode: BuilderMode,
    start_doc: Option<GPoint>,
    current_doc: Option<GPoint>,
}

impl ShapeBuilderTool {
    /// Creates a shape builder or smart fill tool.
    #[must_use]
    pub fn new(mode: BuilderMode) -> Self {
        Self {
            mode,
            start_doc: None,
            current_doc: None,
        }
    }

    /// Returns the builder mode.
    #[must_use]
    pub fn mode(&self) -> BuilderMode {
        self.mode
    }

    /// Resets active drag.
    pub fn cancel(&mut self) {
        self.start_doc = None;
        self.current_doc = None;
    }

    /// Handles pointer events.
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
                Ok(ChangeSet::empty())
            }
            PointerPhase::Move => {
                if self.start_doc.is_some() {
                    self.current_doc = Some(event.doc_pos);
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Up => {
                let _start = self.start_doc.take();
                let _current = self.current_doc.take();

                let selected = bridge.selection().selected_ids;
                if selected.len() >= 2 {
                    let active_surface = bridge
                        .session()
                        .and_then(|s| s.active_surface())
                        .ok_or_else(|| {
                            PetuniaError::invalid_input("no active surface for boolean")
                        })?;

                    let target_id = bridge.next_object_id()?;
                    let plan = petunia_design_application::boolean_service::plan_boolean(
                        &selected,
                        event.modifiers.from_center,
                    )?;

                    let boolean_cmd = CommandRequest::new(Command::ApplyBoolean {
                        surface: active_surface,
                        target_id,
                        subject_id: plan.subject_id,
                        clip_id: plan.clip_id,
                        op: plan.op,
                    });
                    let changes = bridge.submit_command(boolean_cmd)?;
                    bridge.set_selection(vec![target_id]);
                    return Ok(changes);
                }
                Ok(ChangeSet::empty())
            }
            PointerPhase::Cancel => {
                self.cancel();
                Ok(ChangeSet::empty())
            }
        }
    }

    /// Resolves overlays (region highlights).
    #[must_use]
    pub fn overlays(&self) -> CanvasOverlays {
        let mut overlays = CanvasOverlays::default();
        if let (Some(p0), Some(p1)) = (self.start_doc, self.current_doc) {
            overlays.pen_preview = Some(vec![p0, p1]);
        }
        overlays
    }
}
