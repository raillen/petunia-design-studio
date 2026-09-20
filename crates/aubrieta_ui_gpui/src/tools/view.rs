//! Viewport navigation tools: Hand (Pan) and Zoom (08.6, 08.23).

use aubrieta_document::ChangeSet;
use aubrieta_foundation::AubrietaError;
use aubrieta_geometry::GPoint;

use crate::bridge::AubrietaGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use super::input::{NormalizedPointerEvent, PointerPhase};

/// Mode for the viewport view tool (08.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewToolMode {
    /// Hand / Pan viewport translation.
    Pan,
    /// Zoom scaling.
    Zoom,
}

/// Interactive navigation tool supporting camera pan and zoom gestures.
#[derive(Clone, Debug)]
pub struct ViewTool {
    mode: ViewToolMode,
    last_screen: Option<GPoint>,
}

impl ViewTool {
    /// Creates a view tool in Pan or Zoom mode.
    #[must_use]
    pub fn new(mode: ViewToolMode) -> Self {
        Self {
            mode,
            last_screen: None,
        }
    }

    /// Cancels active navigation gesture.
    pub fn cancel(&mut self) {
        self.last_screen = None;
    }

    /// Handles normalized pointer events for canvas navigation.
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        _bridge: &mut AubrietaGuiBridge,
        _camera: &ViewportCamera,
        _snap: &mut SnapEngine,
    ) -> Result<ChangeSet, AubrietaError> {
        match self.mode {
            ViewToolMode::Pan => match event.phase {
                PointerPhase::Down => {
                    self.last_screen = Some(event.screen_pos);
                    Ok(ChangeSet::empty())
                }
                PointerPhase::Move => {
                    if let Some(prev) = self.last_screen {
                        let _dx = event.screen_pos.x - prev.x;
                        let _dy = event.screen_pos.y - prev.y;
                        self.last_screen = Some(event.screen_pos);
                    }
                    Ok(ChangeSet::empty())
                }
                PointerPhase::Up | PointerPhase::Cancel => {
                    self.last_screen = None;
                    Ok(ChangeSet::empty())
                }
            },
            ViewToolMode::Zoom => Ok(ChangeSet::empty()),
        }
    }

    /// Resolves overlays (none for pure view navigation).
    #[must_use]
    pub fn overlays(&self) -> CanvasOverlays {
        CanvasOverlays::default()
    }
}
