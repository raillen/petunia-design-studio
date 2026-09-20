//! Tool manager coordinating active tool state, switching, and event dispatch (09.27, 10.1).

use aubrieta_document::ChangeSet;
use aubrieta_foundation::AubrietaError;
use serde::{Deserialize, Serialize};

use crate::bridge::AubrietaGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use super::input::NormalizedPointerEvent;
use super::node::NodeTool;
use super::pen::PenTool;
use super::select::SelectTool;
use super::shape::{ShapeKind, ShapeTool};

/// Enumeration of interactive tool types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolKind {
    /// Object selection, transformation, and marquee.
    Select,
    /// Bézier path plotting and curve construction.
    Pen,
    /// Direct anchor and node segment manipulation.
    Node,
    /// Parametric rectangle creation.
    Rectangle,
    /// Parametric ellipse creation.
    Ellipse,
    /// Parametric polygon creation.
    Polygon,
}

/// Central manager orchestrating vector tools and event routing.
#[derive(Debug)]
pub struct ToolManager {
    active_kind: ToolKind,
    select_tool: SelectTool,
    pen_tool: PenTool,
    node_tool: NodeTool,
    rectangle_tool: ShapeTool,
    ellipse_tool: ShapeTool,
    polygon_tool: ShapeTool,
}

impl Default for ToolManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolManager {
    /// Creates a fresh tool manager with Select tool active.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_kind: ToolKind::Select,
            select_tool: SelectTool::new(),
            pen_tool: PenTool::new(),
            node_tool: NodeTool::new(),
            rectangle_tool: ShapeTool::new(ShapeKind::Rectangle),
            ellipse_tool: ShapeTool::new(ShapeKind::Ellipse),
            polygon_tool: ShapeTool::new(ShapeKind::Polygon),
        }
    }

    /// Returns the currently active tool kind.
    #[must_use]
    pub fn active_kind(&self) -> ToolKind {
        self.active_kind
    }

    /// Switches the active tool, gracefully cancelling any pending gesture on the previous tool.
    pub fn set_tool(&mut self, new_tool: ToolKind) {
        if self.active_kind == new_tool {
            return;
        }
        self.cancel_active();
        self.active_kind = new_tool;
    }

    /// Cancels any active gesture in the current tool.
    pub fn cancel_active(&mut self) {
        match self.active_kind {
            ToolKind::Select => self.select_tool.cancel(),
            ToolKind::Pen => self.pen_tool.cancel(),
            ToolKind::Node => self.node_tool.cancel(),
            ToolKind::Rectangle => self.rectangle_tool.cancel(),
            ToolKind::Ellipse => self.ellipse_tool.cancel(),
            ToolKind::Polygon => self.polygon_tool.cancel(),
        }
    }

    /// Dispatches a normalized pointer event to the active tool.
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut AubrietaGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, AubrietaError> {
        match self.active_kind {
            ToolKind::Select => self
                .select_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Pen => self.pen_tool.on_pointer_event(event, bridge, camera, snap),
            ToolKind::Node => self.node_tool.on_pointer_event(event, bridge, camera, snap),
            ToolKind::Rectangle => self
                .rectangle_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Ellipse => self
                .ellipse_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Polygon => self
                .polygon_tool
                .on_pointer_event(event, bridge, camera, snap),
        }
    }

    /// Resolves overlays produced by the active tool.
    #[must_use]
    pub fn overlays(&self, camera: &ViewportCamera, bridge: &AubrietaGuiBridge) -> CanvasOverlays {
        match self.active_kind {
            ToolKind::Select => self.select_tool.overlays(camera, bridge),
            ToolKind::Pen => self.pen_tool.overlays(),
            ToolKind::Node => self.node_tool.overlays(),
            ToolKind::Rectangle => self.rectangle_tool.overlays(camera),
            ToolKind::Ellipse => self.ellipse_tool.overlays(camera),
            ToolKind::Polygon => self.polygon_tool.overlays(camera),
        }
    }
}
