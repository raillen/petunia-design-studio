use aubrieta_document::ChangeSet;
use aubrieta_foundation::AubrietaError;
use aubrieta_geometry::{GPoint, GRect};

use crate::bridge::{
    AubrietaGuiBridge, DataMergePresentationModel, DialogRequest, LayersPresentationModel,
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
pub struct AubrietaShell {
    /// Semantic bridge to document session and core logic.
    pub bridge: AubrietaGuiBridge,
    /// Viewport camera managing pan, zoom, and coordinate projections.
    pub camera: ViewportCamera,
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

impl Default for AubrietaShell {
    fn default() -> Self {
        Self::new(1280.0, 800.0)
    }
}

impl AubrietaShell {
    /// Creates a fresh desktop shell with viewport dimensions.
    #[must_use]
    pub fn new(viewport_width: f64, viewport_height: f64) -> Self {
        Self {
            bridge: AubrietaGuiBridge::new(),
            camera: ViewportCamera::new(viewport_width, viewport_height),
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
    pub fn new_document(&mut self, title: impl Into<String>) -> Result<(), AubrietaError> {
        self.bridge.new_document(title)?;
        self.camera.reset_100();
        Ok(())
    }

    /// Dispatches a normalized pointer event through active tool and snapping engine.
    /// View-tool navigation is applied to the owned camera afterwards.
    pub fn handle_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
    ) -> Result<ChangeSet, AubrietaError> {
        let changes =
            self.tools
                .on_pointer_event(event, &mut self.bridge, &self.camera, &mut self.snap)?;
        if let Some(action) = self.tools.take_camera_action() {
            use crate::tools::CameraAction;
            match action {
                CameraAction::Pan { dx, dy } => self.camera.pan(dx, dy),
                CameraAction::Zoom { focus, factor } => self.camera.zoom_at(focus, factor),
            }
        }
        Ok(changes)
    }

    /// Switches the active editing tool.
    pub fn set_active_tool(&mut self, tool: ToolKind) {
        self.tools.set_tool(tool);
    }

    /// Returns the currently active editing tool kind.
    #[must_use]
    pub fn active_tool(&self) -> ToolKind {
        self.tools.active_kind()
    }

    /// Triggers pointer-centered zoom in viewport.
    pub fn zoom_at(&mut self, screen_focus: GPoint, factor: f64) {
        self.camera.zoom_at(screen_focus, factor);
    }

    /// Pans the canvas by delta screen pixels.
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.camera.pan(dx, dy);
    }

    /// Fits the active surface or bounds into viewport.
    pub fn fit_surface(&mut self, surface_bounds: GRect) {
        self.camera.fit_rect(surface_bounds, 40.0);
    }

    /// Collects all active visual overlays (handles, guides, pen curve previews).
    pub fn overlays(&mut self) -> CanvasOverlays {
        let mut ov = self.tools.overlays(&self.camera, &self.bridge);
        // Include snap guides if any
        ov.snap_guides = self
            .snap
            .snap_point(GPoint::ORIGIN, &self.camera, &[])
            .guides;
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
    pub fn undo(&mut self) -> Result<bool, AubrietaError> {
        self.history_panel.undo(&mut self.bridge)
    }

    /// Invokes redo via history controller.
    pub fn redo(&mut self) -> Result<bool, AubrietaError> {
        self.history_panel.redo(&mut self.bridge)
    }

    /// Emits a symbolic dialog request.
    pub fn request_dialog(&mut self, req: DialogRequest) {
        self.dialog_queue.push(req);
    }
}
