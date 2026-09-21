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

    /// Updates appearance stack on all currently selected objects.
    pub fn set_appearance(
        &self,
        bridge: &mut AubrietaGuiBridge,
        appearance: Option<aubrieta_document::AppearanceStack>,
    ) -> Result<ChangeSet, AubrietaError> {
        let sel_ids = bridge.selection().selected_ids;
        let mut combined = ChangeSet::empty();
        for id in sel_ids {
            let changes = bridge.set_appearance(id, appearance.clone())?;
            for c in changes.changes {
                combined.push(c);
            }
        }
        Ok(combined)
    }

    /// Updates active surface dimensions and origin (10.7).
    pub fn set_surface_geometry(
        &self,
        bridge: &mut AubrietaGuiBridge,
        origin: [f64; 2],
        dimensions: [f64; 2],
    ) -> Result<ChangeSet, AubrietaError> {
        let surface = bridge
            .active_surface()
            .ok_or_else(|| AubrietaError::invalid_input("no active surface"))?;
        bridge.set_surface_geometry(surface, origin, dimensions)
    }

    /// Updates active surface bleed insets (10.7).
    pub fn set_surface_bleed(
        &self,
        bridge: &mut AubrietaGuiBridge,
        bleed: aubrieta_document::Bleed,
    ) -> Result<ChangeSet, AubrietaError> {
        let surface = bridge
            .active_surface()
            .ok_or_else(|| AubrietaError::invalid_input("no active surface"))?;
        bridge.set_surface_bleed(surface, bleed)
    }

    /// Updates active surface safe margins (10.7).
    pub fn set_surface_margins(
        &self,
        bridge: &mut AubrietaGuiBridge,
        margins: aubrieta_document::Margins,
    ) -> Result<ChangeSet, AubrietaError> {
        let surface = bridge
            .active_surface()
            .ok_or_else(|| AubrietaError::invalid_input("no active surface"))?;
        bridge.set_surface_margins(surface, margins)
    }

    /// Updates active surface background (10.7).
    pub fn set_surface_background(
        &self,
        bridge: &mut AubrietaGuiBridge,
        background: Option<String>,
    ) -> Result<ChangeSet, AubrietaError> {
        let surface = bridge
            .active_surface()
            .ok_or_else(|| AubrietaError::invalid_input("no active surface"))?;
        bridge.set_surface_background(surface, background)
    }
}
