use petunia_design_application::interaction::PointerPhase;
use petunia_design_application::view_camera::ViewState;
use petunia_design_document::ChangeSet;
use petunia_design_foundation::PetuniaError;
use petunia_design_geometry::{GPoint, GRect};

use crate::bridge::{
    DataMergePresentationModel, DialogRequest, LayersPresentationModel, PetuniaDesignGuiBridge,
    PropertiesPresentationModel, SessionSnapshot,
};
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};
use crate::panels::{
    DataMergePanelController, HistoryPanelController, LayersPanelController,
    PropertiesPanelController,
};
use crate::tools::{NormalizedPointerEvent, ToolKind, ToolManager};

/// Complete desktop application shell coordinating canvas, tools, panels, and bridge.
#[derive(Debug)]
pub struct PetuniaShell {
    /// Semantic bridge to document session and core logic.
    pub bridge: PetuniaDesignGuiBridge,
    /// Camera used only while **no document is open**. Once a session exists,
    /// the authoritative camera is `session.view.camera` (15.B) and this value
    /// is just the last local edit kept for the sessionless window.
    viewport: ViewportCamera,
    /// Snapping service for magnetic alignment.
    pub snap: SnapEngine,
    /// Tool manager coordinating interactive drawing/transform tools.
    pub tools: ToolManager,
    /// Layers panel controller.
    pub layers_panel: LayersPanelController,
    /// Properties inspector controller.
    pub properties_panel: PropertiesPanelController,
    /// History panel controller.
    pub history_panel: HistoryPanelController,
    /// Data merge panel controller.
    pub data_merge_panel: DataMergePanelController,
    /// Pending dialog requests awaiting UI presentation.
    pub dialog_queue: Vec<DialogRequest>,
}

impl Default for PetuniaShell {
    fn default() -> Self {
        Self::new(1280.0, 800.0)
    }
}

impl PetuniaShell {
    /// Creates a fresh desktop shell with viewport dimensions.
    #[must_use]
    pub fn new(viewport_width: f64, viewport_height: f64) -> Self {
        Self {
            bridge: PetuniaDesignGuiBridge::new(),
            viewport: ViewportCamera::new(viewport_width, viewport_height),
            snap: SnapEngine::new(),
            tools: ToolManager::new(),
            layers_panel: LayersPanelController::new(),
            properties_panel: PropertiesPanelController::new(),
            history_panel: HistoryPanelController::new(),
            data_merge_panel: DataMergePanelController::new(),
            dialog_queue: Vec::new(),
        }
    }

    /// Initializes a new empty document.
    pub fn new_document(&mut self, title: impl Into<String>) -> Result<(), PetuniaError> {
        self.bridge.new_document(title)?;
        let mut camera = self.view_camera();
        camera.reset_100();
        self.set_view_camera(camera);
        Ok(())
    }

    /// The authoritative viewport camera.
    #[must_use]
    pub fn view_camera(&self) -> ViewportCamera {
        self.bridge.session().map_or_else(
            || self.viewport.clone(),
            |session| session.view.camera.clone(),
        )
    }

    /// Replaces the session camera, keeping the sessionless fallback in step.
    pub fn set_view_camera(&mut self, camera: ViewportCamera) {
        self.viewport = camera.clone();
        if let Some(view) = self.bridge.view_state_mut() {
            view.camera = camera;
        }
    }

    /// Mutable access to the session's full view state (rulers, snapping, palette).
    pub fn view_state_mut(&mut self) -> Option<&mut ViewState> {
        self.bridge.view_state_mut()
    }

    /// Dispatches a normalized pointer event through active tool and snapping engine.
    /// View-tool navigation is applied to the session camera afterwards.
    pub fn handle_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
    ) -> Result<ChangeSet, PetuniaError> {
        let camera = self.view_camera();
        let changes =
            self.tools
                .on_pointer_event(event, &mut self.bridge, &camera, &mut self.snap)?;
        if event.phase == PointerPhase::Up || event.phase == PointerPhase::Cancel {
            self.snap.reset_hysteresis();
        }
        if let Some(action) = self.tools.take_camera_action() {
            use crate::tools::CameraAction;
            let mut camera = self.view_camera();
            match action {
                CameraAction::Pan { dx, dy } => camera.pan(dx, dy),
                CameraAction::Zoom { focus, factor } => camera.zoom_at(focus, factor),
            }
            self.set_view_camera(camera);
        }
        Ok(changes)
    }

    /// Switches the active editing tool.
    pub fn set_active_tool(&mut self, tool: ToolKind) {
        self.snap.reset_hysteresis();
        self.tools.set_tool(tool);
    }

    /// Returns the currently active editing tool kind.
    #[must_use]
    pub fn active_tool(&self) -> ToolKind {
        self.tools.active_kind()
    }

    /// Triggers pointer-centered zoom in viewport.
    pub fn zoom_at(&mut self, screen_focus: GPoint, factor: f64) {
        let mut camera = self.view_camera();
        camera.zoom_at(screen_focus, factor);
        self.set_view_camera(camera);
    }

    /// Pans the canvas by delta screen pixels.
    pub fn pan(&mut self, dx: f64, dy: f64) {
        let mut camera = self.view_camera();
        camera.pan(dx, dy);
        self.set_view_camera(camera);
    }

    /// Fits the active surface or bounds into viewport.
    pub fn fit_surface(&mut self, surface_bounds: GRect) {
        let mut camera = self.view_camera();
        camera.fit_rect(surface_bounds, 40.0);
        self.set_view_camera(camera);
    }

    /// Collects all active visual overlays (handles, guides, pen curve previews).
    #[must_use]
    pub fn overlays(&self) -> CanvasOverlays {
        let camera = self.view_camera();
        let mut ov = self.tools.overlays(&camera, &self.bridge);
        ov.snap_guides = self.snap.active_guides().to_vec();
        ov
    }

    /// Resolves full session snapshot.
    #[must_use]
    pub fn snapshot(&self) -> SessionSnapshot {
        self.bridge.snapshot()
    }

    /// Resolves layers presentation model.
    #[must_use]
    pub fn query_layers(&self) -> LayersPresentationModel {
        self.layers_panel.query_model(&self.bridge)
    }

    /// Resolves properties presentation model.
    #[must_use]
    pub fn query_properties(&self) -> PropertiesPresentationModel {
        self.properties_panel.query_model(&self.bridge)
    }

    /// Resolves data merge presentation model.
    #[must_use]
    pub fn query_data_merge(&self) -> DataMergePresentationModel {
        self.data_merge_panel.query_model(&self.bridge)
    }

    /// Invokes undo via history controller.
    pub fn undo(&mut self) -> Result<bool, PetuniaError> {
        self.history_panel.undo(&mut self.bridge)
    }

    /// Invokes redo via history controller.
    pub fn redo(&mut self) -> Result<bool, PetuniaError> {
        self.history_panel.redo(&mut self.bridge)
    }

    /// Emits a symbolic dialog request.
    pub fn request_dialog(&mut self, req: DialogRequest) {
        self.dialog_queue.push(req);
    }
}
