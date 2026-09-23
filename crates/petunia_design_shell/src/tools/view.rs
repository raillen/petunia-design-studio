//! Viewport navigation tools: Hand (Pan) and Zoom (08.6, 08.23).

use petunia_design_document::ChangeSet;
use petunia_design_foundation::PetuniaError;
use petunia_design_geometry::GPoint;

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use petunia_design_application::interaction::{NormalizedPointerEvent, PointerPhase};

/// Mode for the viewport view tool (08.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewToolMode {
    /// Hand / Pan viewport translation.
    Pan,
    /// Zoom scaling.
    Zoom,
}

/// Camera navigation request produced by the view tools (Table B).
/// The shell owns the camera mutably and applies the drained action after
/// tool dispatch, so tools never need `&mut ViewportCamera`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CameraAction {
    /// Pan by screen-pixel delta.
    Pan { dx: f64, dy: f64 },
    /// Zoom by factor around a screen focus point.
    Zoom { focus: GPoint, factor: f64 },
}

/// Interactive navigation tool supporting camera pan and zoom gestures.
#[derive(Clone, Debug)]
pub struct ViewTool {
    mode: ViewToolMode,
    last_screen: Option<GPoint>,
    pending: Option<CameraAction>,
}

impl ViewTool {
    /// Creates a view tool in Pan or Zoom mode.
    #[must_use]
    pub fn new(mode: ViewToolMode) -> Self {
        Self {
            mode,
            last_screen: None,
            pending: None,
        }
    }

    /// Cancels active navigation gesture.
    pub fn cancel(&mut self) {
        self.last_screen = None;
        self.pending = None;
    }

    /// Takes the pending camera action, if any. The shell drains this after
    /// dispatch and applies it to the owned camera.
    pub fn take_camera_action(&mut self) -> Option<CameraAction> {
        self.pending.take()
    }

    /// Handles normalized pointer events for canvas navigation.
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        _bridge: &mut PetuniaDesignGuiBridge,
        _camera: &ViewportCamera,
        _snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
        match self.mode {
            ViewToolMode::Pan => match event.phase {
                PointerPhase::Down => {
                    self.last_screen = Some(event.screen_pos);
                    Ok(ChangeSet::empty())
                }
                PointerPhase::Move => {
                    if let Some(prev) = self.last_screen {
                        let dx = event.screen_pos.x - prev.x;
                        let dy = event.screen_pos.y - prev.y;
                        self.pending = Some(CameraAction::Pan { dx, dy });
                        self.last_screen = Some(event.screen_pos);
                    }
                    Ok(ChangeSet::empty())
                }
                PointerPhase::Up | PointerPhase::Cancel => {
                    self.last_screen = None;
                    Ok(ChangeSet::empty())
                }
            },
            ViewToolMode::Zoom => match event.phase {
                PointerPhase::Down => {
                    self.last_screen = Some(event.screen_pos);
                    Ok(ChangeSet::empty())
                }
                PointerPhase::Move => {
                    if let Some(prev) = self.last_screen {
                        // Vertical drag drives zoom; horizontal drift is ignored.
                        let dy = event.screen_pos.y - prev.y;
                        let factor = (1.0 - dy * 0.002).clamp(0.5, 2.0);
                        self.pending = Some(CameraAction::Zoom {
                            focus: event.screen_pos,
                            factor,
                        });
                        self.last_screen = Some(event.screen_pos);
                    }
                    Ok(ChangeSet::empty())
                }
                PointerPhase::Up | PointerPhase::Cancel => {
                    self.last_screen = None;
                    Ok(ChangeSet::empty())
                }
            },
        }
    }

    /// Resolves overlays (none for pure view navigation).
    #[must_use]
    pub fn overlays(&self) -> CanvasOverlays {
        CanvasOverlays::default()
    }
}
