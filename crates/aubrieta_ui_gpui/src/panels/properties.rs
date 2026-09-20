//! Properties inspector panel controller (09.25, 10.1, 10.4).

use aubrieta_document::ChangeSet;
use aubrieta_foundation::AubrietaError;

use crate::bridge::{AubrietaGuiBridge, PropertiesPresentationModel};

/// Controller managing the Properties Inspector panel.
#[derive(Debug, Default)]
pub struct PropertiesPanelController;

impl PropertiesPanelController {
    /// Creates a fresh properties panel controller.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Resolves the current properties presentation model for active selection.
    #[must_use]
    pub fn query_model(&self, bridge: &AubrietaGuiBridge) -> PropertiesPresentationModel {
        bridge.query_properties()
    }

    /// Updates fill token on all currently selected objects.
    pub fn set_fill(
        &self,
        bridge: &mut AubrietaGuiBridge,
        fill_token: Option<String>,
    ) -> Result<ChangeSet, AubrietaError> {
        let sel_ids = bridge.selection().selected_ids;
        let mut combined = ChangeSet::empty();
        for id in sel_ids {
            let changes = bridge.set_fill(id, fill_token.clone())?;
            for c in changes.changes {
                combined.push(c);
            }
        }
        Ok(combined)
    }

    /// Updates stroke token and width on all currently selected objects.
    pub fn set_stroke(
        &self,
        bridge: &mut AubrietaGuiBridge,
        stroke_token: Option<String>,
        width: f64,
    ) -> Result<ChangeSet, AubrietaError> {
        let sel_ids = bridge.selection().selected_ids;
        let mut combined = ChangeSet::empty();
        for id in sel_ids {
            let changes = bridge.set_stroke(id, stroke_token.clone(), width)?;
            for c in changes.changes {
                combined.push(c);
            }
        }
        Ok(combined)
    }

    /// Updates opacity factor on all currently selected objects.
    pub fn set_opacity(
        &self,
        bridge: &mut AubrietaGuiBridge,
        opacity: f64,
    ) -> Result<ChangeSet, AubrietaError> {
        let sel_ids = bridge.selection().selected_ids;
        let mut combined = ChangeSet::empty();
        for id in sel_ids {
            let changes = bridge.set_opacity(id, opacity)?;
            for c in changes.changes {
                combined.push(c);
            }
        }
        Ok(combined)
    }

    /// Updates bounds coordinate on all currently selected objects.
    pub fn set_bounds(
        &self,
        bridge: &mut AubrietaGuiBridge,
        bounds: [f64; 4],
        rotation: f64,
    ) -> Result<ChangeSet, AubrietaError> {
        let sel_ids = bridge.selection().selected_ids;
        let mut combined = ChangeSet::empty();
        for id in sel_ids {
            let changes = bridge.set_bounds(id, Some(bounds), rotation)?;
            for c in changes.changes {
                combined.push(c);
            }
        }
        Ok(combined)
    }
}
