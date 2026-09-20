//! Eyedropper and style sampling tools (08.24, 09.25, 10.4).

use aubrieta_application::{Command, CommandRequest};
use aubrieta_document::ChangeSet;
use aubrieta_foundation::AubrietaError;

use crate::bridge::AubrietaGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use super::input::{NormalizedPointerEvent, PointerButton, PointerPhase};

/// Operational mode for the picker tool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickerMode {
    /// Color eyedropper sampling fill/stroke color tokens.
    Color,
    /// Style picker copying AppearanceStack and properties.
    Style,
}

/// Eyedropper and style sampler tool.
#[derive(Clone, Debug)]
pub struct PickerTool {
    mode: PickerMode,
}

impl PickerTool {
    /// Creates a picker tool in color or style mode.
    #[must_use]
    pub fn new(mode: PickerMode) -> Self {
        Self { mode }
    }

    /// Resets tool state.
    pub fn cancel(&mut self) {}

    /// Handles normalized pointer events.
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut AubrietaGuiBridge,
        _camera: &ViewportCamera,
        _snap: &mut SnapEngine,
    ) -> Result<ChangeSet, AubrietaError> {
        if event.phase != PointerPhase::Up || event.button != PointerButton::Primary {
            return Ok(ChangeSet::empty());
        }

        let pt = event.doc_pos;
        let session = match bridge.session() {
            Some(s) => s,
            None => return Ok(ChangeSet::empty()),
        };

        let active_surface_id = match session.active_surface {
            Some(id) => id,
            None => return Ok(ChangeSet::empty()),
        };

        let surface = match session.document.surface(active_surface_id) {
            Ok(s) => s,
            Err(_) => return Ok(ChangeSet::empty()),
        };

        // Hit-test in reverse draw order (topmost first)
        let hit_object = surface
            .objects
            .iter()
            .rev()
            .find(|obj| obj.hit_test(pt))
            .cloned();

        let hit = match hit_object {
            Some(obj) => obj,
            None => return Ok(ChangeSet::empty()),
        };

        let selected_ids = bridge.selection().selected_ids;
        let mut combined = ChangeSet::empty();

        match self.mode {
            PickerMode::Color => {
                if let Some(fill) = hit.fill {
                    for sel_id in selected_ids {
                        let cmd = CommandRequest::new(Command::SetFill {
                            id: sel_id,
                            fill: Some(fill.clone()),
                        });
                        let c = bridge.submit_command(cmd)?;
                        combined.extend(c);
                    }
                }
            }
            PickerMode::Style => {
                for sel_id in selected_ids {
                    if let Some(fill) = &hit.fill {
                        let c1 = bridge.submit_command(CommandRequest::new(Command::SetFill {
                            id: sel_id,
                            fill: Some(fill.clone()),
                        }))?;
                        combined.extend(c1);
                    }
                    if let Some(stroke) = &hit.stroke {
                        let c2 = bridge.submit_command(CommandRequest::new(Command::SetStroke {
                            id: sel_id,
                            stroke: Some(stroke.clone()),
                            width: hit.stroke_width,
                        }))?;
                        combined.extend(c2);
                    }
                    if let Some(app) = &hit.appearance {
                        let c3 = bridge.submit_command(CommandRequest::new(Command::SetAppearance {
                            id: sel_id,
                            appearance: Some(app.clone()),
                        }))?;
                        combined.extend(c3);
                    }
                }
            }
        }

        Ok(combined)
    }

    /// Resolves overlays (none for eyedropper sampling).
    #[must_use]
    pub fn overlays(&self) -> CanvasOverlays {
        CanvasOverlays::default()
    }
}
