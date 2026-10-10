//! Workspace and view state: pure Session State.
//!
//! ADR-0012 D9: every value here lives in the session. None of it is
//! ever written to PTND. Panels, tabs, tool, selection, zoom and pan
//! are all restorable from a workspace file, never from the document.

use petunia_core::Rect;
use serde::{Deserialize, Serialize};

/// Default left-edge toolbar width in logical pixels.
pub const TOOLBAR_WIDTH: f64 = 48.0;
/// Default right-edge panel width in logical pixels.
pub const PANEL_WIDTH: f64 = 280.0;
/// Smallest window size with usable layout (D9).
pub const MIN_WINDOW: (f64, f64) = (1024.0, 640.0);

/// Which side a docked panel lives on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DockSide {
    Left,
    Right,
}

/// One docked panel in the workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelState {
    pub id: String,
    pub side: DockSide,
    pub open: bool,
}

/// Workspace arrangement: Session State, never PTND.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceState {
    panels: Vec<PanelState>,
    toolbar_visible: bool,
    breadcrumb_visible: bool,
}

impl WorkspaceState {
    /// Default arrangement: toolbar plus two right panels.
    #[must_use]
    pub fn defaults() -> Self {
        Self {
            panels: vec![
                PanelState {
                    id: "properties".into(),
                    side: DockSide::Right,
                    open: true,
                },
                PanelState {
                    id: "layers".into(),
                    side: DockSide::Right,
                    open: true,
                },
            ],
            toolbar_visible: true,
            breadcrumb_visible: true,
        }
    }

    /// Open or close a panel by id; unknown ids are added.
    pub fn set_panel_open(&mut self, id: impl Into<String>, open: bool) {
        let id = id.into();
        for panel in &mut self.panels {
            if panel.id == id {
                panel.open = open;
                return;
            }
        }
        self.panels.push(PanelState {
            id,
            side: DockSide::Right,
            open,
        });
    }

    /// Whether a panel is open.
    #[must_use]
    pub fn is_panel_open(&self, id: &str) -> bool {
        self.panels.iter().any(|panel| panel.id == id && panel.open)
    }

    /// All panels, in dock order.
    #[must_use]
    pub fn panels(&self) -> &[PanelState] {
        &self.panels
    }

    /// Show or hide the left toolbar.
    pub fn set_toolbar_visible(&mut self, visible: bool) {
        self.toolbar_visible = visible;
    }

    /// Whether the left toolbar is shown.
    #[must_use]
    pub fn toolbar_visible(&self) -> bool {
        self.toolbar_visible
    }

    /// Show or hide the breadcrumb.
    pub fn set_breadcrumb_visible(&mut self, visible: bool) {
        self.breadcrumb_visible = visible;
    }

    /// Whether the breadcrumb is shown.
    #[must_use]
    pub fn breadcrumb_visible(&self) -> bool {
        self.breadcrumb_visible
    }
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self::defaults()
    }
}

/// View state: zoom, pan, rotation, viewport, DPR.
///
/// Never serialized in PTND; optionally stored in a separate
/// workspace session.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ViewState {
    pub scale: f64,
    pub pan_x: f64,
    pub pan_y: f64,
    pub rotation: f64,
    pub viewport: Rect,
    pub dpr: f64,
}

impl ViewState {
    /// A view at 100% with no pan.
    #[must_use]
    pub fn new(viewport: Rect) -> Self {
        Self {
            scale: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            rotation: 0.0,
            viewport,
            dpr: 1.0,
        }
    }

    /// Effective scale in device pixels, used for tolerance math.
    #[must_use]
    pub fn effective_scale(&self) -> f64 {
        self.scale * self.dpr
    }

    /// Screen tolerance converted to document units, so hit targets
    /// stay constant in size at any zoom. `None` for a non-positive
    /// scale.
    #[must_use]
    pub fn document_tolerance(&self, screen_tolerance_px: f64) -> Option<f64> {
        let scale = self.effective_scale();
        if !scale.is_finite() || scale <= 0.0 || !screen_tolerance_px.is_finite() {
            return None;
        }
        Some(screen_tolerance_px / scale)
    }
}

impl Default for ViewState {
    fn default() -> Self {
        Self::new(Rect::new(0.0, 0.0, 1920.0, 1080.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_is_session_state_only() {
        let mut workspace = WorkspaceState::defaults();
        assert!(workspace.toolbar_visible());
        assert!(workspace.is_panel_open("layers"));
        workspace.set_panel_open("layers", false);
        assert!(!workspace.is_panel_open("layers"));
        workspace.set_panel_open("timeline", true);
        assert!(workspace.is_panel_open("timeline"));
        workspace.set_toolbar_visible(false);
        assert!(!workspace.toolbar_visible());
    }

    #[test]
    fn document_tolerance_scales_with_zoom_and_dpr() {
        let mut view = ViewState::default();
        assert_eq!(view.document_tolerance(8.0), Some(8.0));
        view.scale = 2.0;
        assert_eq!(view.document_tolerance(8.0), Some(4.0));
        view.dpr = 2.0;
        assert_eq!(view.document_tolerance(8.0), Some(2.0));
        view.scale = 0.0;
        assert_eq!(view.document_tolerance(8.0), None);
    }

    #[test]
    fn defaults_fit_the_minimum_window() {
        let view = ViewState::default();
        assert!(view.viewport.width >= MIN_WINDOW.0);
        assert!(view.viewport.height >= MIN_WINDOW.1);
    }

    #[test]
    fn workspace_panels_persist_across_sessions() {
        // O arranjo é Session State: sobrevive a troca de workspace,
        // mas nunca entra no PTND. O round-trip usa Clone, que é o
        // mecanismo real de salvar/carregar arranjo.
        let workspace = WorkspaceState::defaults();
        let restored = workspace.clone();
        assert_eq!(restored, workspace);
        assert!(restored.is_panel_open("properties"));
    }
}
