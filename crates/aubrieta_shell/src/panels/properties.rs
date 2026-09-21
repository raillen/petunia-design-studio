//! Properties inspector panel controller (09.25, 10.1, 10.4).

use aubrieta_application::Command;
use aubrieta_document::ChangeSet;
use aubrieta_foundation::{AubrietaError, ObjectId};

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
    /// One panel gesture commits exactly one undo entry (F-01).
    pub fn set_fill(
        &self,
        bridge: &mut AubrietaGuiBridge,
        fill_token: Option<String>,
    ) -> Result<ChangeSet, AubrietaError> {
        let sel_ids = bridge.selection().selected_ids;
        let cmds = sel_ids
            .into_iter()
            .map(|id| Command::SetFill {
                id,
                fill: fill_token.clone(),
            })
            .collect();
        bridge.submit_all("Set fill", cmds)
    }

    /// Updates stroke token and width on all currently selected objects.
    pub fn set_stroke(
        &self,
        bridge: &mut AubrietaGuiBridge,
        stroke_token: Option<String>,
        width: f64,
    ) -> Result<ChangeSet, AubrietaError> {
        let sel_ids = bridge.selection().selected_ids;
        let cmds = sel_ids
            .into_iter()
            .map(|id| Command::SetStroke {
                id,
                stroke: stroke_token.clone(),
                width,
            })
            .collect();
        bridge.submit_all("Set stroke", cmds)
    }

    /// Updates opacity factor on all currently selected objects.
    pub fn set_opacity(
        &self,
        bridge: &mut AubrietaGuiBridge,
        opacity: f64,
    ) -> Result<ChangeSet, AubrietaError> {
        let sel_ids = bridge.selection().selected_ids;
        let cmds = sel_ids
            .into_iter()
            .map(|id| Command::SetOpacity { id, opacity })
            .collect();
        bridge.submit_all("Set opacity", cmds)
    }

    /// Updates bounds on the selection (10.1).
    /// Single selection: exact bounds + rotation. Multi-selection: the
    /// incoming rect is treated as the new position of the selection's
    /// top-left — each object translates by that delta, preserving its own
    /// size (never collapses distinct objects into one rect). Rotation is
    /// applied per object as explicitly requested.
    pub fn set_bounds(
        &self,
        bridge: &mut AubrietaGuiBridge,
        bounds: [f64; 4],
        rotation: f64,
    ) -> Result<ChangeSet, AubrietaError> {
        let sel_ids = bridge.selection().selected_ids;
        if sel_ids.len() <= 1 {
            let cmds = sel_ids
                .into_iter()
                .map(|id| Command::SetBounds {
                    id,
                    bounds: Some(bounds),
                    rotation,
                })
                .collect();
            return bridge.submit_all("Set bounds", cmds);
        }
        let current = combined_selection_bounds(bridge, &sel_ids);
        let (dx, dy) = match current {
            Some([cx, cy, _, _]) => (bounds[0] - cx, bounds[1] - cy),
            None => (0.0, 0.0),
        };
        let cmds = sel_ids
            .into_iter()
            .filter_map(|id| {
                moved_bounds(bridge, id, dx, dy).map(|next| Command::SetBounds {
                    id,
                    bounds: Some(next),
                    rotation,
                })
            })
            .collect();
        bridge.submit_all("Move selection", cmds)
    }

    /// Updates appearance stack on all currently selected objects.
    pub fn set_appearance(
        &self,
        bridge: &mut AubrietaGuiBridge,
        appearance: Option<aubrieta_document::AppearanceStack>,
    ) -> Result<ChangeSet, AubrietaError> {
        let sel_ids = bridge.selection().selected_ids;
        let cmds = sel_ids
            .into_iter()
            .map(|id| Command::SetAppearance {
                id,
                appearance: appearance.clone(),
            })
            .collect();
        bridge.submit_all("Set appearance", cmds)
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

/// Combined `[x, y, w, h]` of the given objects, ignoring unbounded ones.
fn combined_selection_bounds(
    bridge: &AubrietaGuiBridge,
    ids: &[ObjectId],
) -> Option<[f64; 4]> {
    let session = bridge.session()?;
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut any = false;
    for id in ids {
        if let Some(b) = session.find_object(*id).and_then(|o| o.bounds) {
            any = true;
            min_x = min_x.min(b[0]);
            min_y = min_y.min(b[1]);
            max_x = max_x.max(b[0] + b[2]);
            max_y = max_y.max(b[1] + b[3]);
        }
    }
    any.then_some([min_x, min_y, max_x - min_x, max_y - min_y])
}

/// Current bounds of one object translated by `(dx, dy)`.
fn moved_bounds(bridge: &AubrietaGuiBridge, id: ObjectId, dx: f64, dy: f64) -> Option<[f64; 4]> {
    bridge
        .session()?
        .find_object(id)
        .and_then(|o| o.bounds)
        .map(|b| [b[0] + dx, b[1] + dy, b[2], b[3]])
}
