//! Eyedropper and style sampling tools (08.24, 09.25, 10.4).

use aubrieta_document::ChangeSet;
use aubrieta_foundation::AubrietaError;

use crate::bridge::AubrietaGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use aubrieta_application::interaction::{NormalizedPointerEvent, PointerButton, PointerPhase};

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

        let active_surface_id = match session.active_surface() {
            Some(id) => id,
            None => return Ok(ChangeSet::empty()),
        };

        let surface = match session.surface(active_surface_id) {
            Ok(s) => s,
            Err(_) => return Ok(ChangeSet::empty()),
        };

        // Hit-test in reverse draw order (topmost first)
        let hit_object = surface
            .objects()
            .iter()
            .rev()
            .find(|obj| obj.hit_test(pt))
            .cloned();

        let hit = match hit_object {
            Some(obj) => obj,
            None => return Ok(ChangeSet::empty()),
        };

        let selected_ids = bridge.selection().selected_ids;
        let mut all_cmds = Vec::new();

        match self.mode {
            PickerMode::Color => {
                if let Some(fill) = hit.fill {
                    for sel_id in selected_ids {
                        all_cmds.push(aubrieta_application::Command::SetFill {
                            id: sel_id,
                            fill: Some(fill.clone()),
                        });
                    }
                }
            }
            PickerMode::Style => {
                let style = aubrieta_application::appearance_service::sample_style(&hit);
                for sel_id in selected_ids {
                    all_cmds.extend(
                        aubrieta_application::appearance_service::style_sample_commands(
                            sel_id, &style,
                        ),
                    );
                }
            }
        }

        if all_cmds.is_empty() {
            return Ok(ChangeSet::empty());
        }
        let label = match self.mode {
            PickerMode::Color => "Pick color",
            PickerMode::Style => "Pick style",
        };
        bridge.submit_all(label, all_cmds)
    }

    /// Resolves overlays (none for eyedropper sampling).
    #[must_use]
    pub fn overlays(&self) -> CanvasOverlays {
        CanvasOverlays::default()
    }
}
