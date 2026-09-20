//! Stable semantic action identifiers (`aubrieta.*` namespace).

use serde::{Deserialize, Serialize};

/// Namespaced action identifier, e.g. `aubrieta.surface.create`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActionId(pub String);

impl ActionId {
    /// Well-known actions for the P00 MVP slice.
    pub const SURFACE_CREATE: &'static str = "aubrieta.surface.create";
    pub const OBJECT_CREATE: &'static str = "aubrieta.object.create";
    pub const OBJECT_DELETE: &'static str = "aubrieta.object.delete";
    pub const FILL_SET: &'static str = "aubrieta.fill.set";

    // Design Persona Tool Actions (08.24, 08.33, 10.1 - 10.7)
    pub const TOOL_SELECT: &'static str = "aubrieta.tool.select";
    pub const TOOL_NODE: &'static str = "aubrieta.tool.node";
    pub const TOOL_POINT_TRANSFORM: &'static str = "aubrieta.tool.point_transform";
    pub const TOOL_PEN: &'static str = "aubrieta.tool.pen";
    pub const TOOL_PENCIL: &'static str = "aubrieta.tool.pencil";
    pub const TOOL_CORNER: &'static str = "aubrieta.tool.corner";
    pub const TOOL_CONTOUR: &'static str = "aubrieta.tool.contour";
    pub const TOOL_KNIFE: &'static str = "aubrieta.tool.knife";
    pub const TOOL_SCISSORS: &'static str = "aubrieta.tool.scissors";
    pub const TOOL_RECTANGLE: &'static str = "aubrieta.tool.shape.rectangle";
    pub const TOOL_ELLIPSE: &'static str = "aubrieta.tool.shape.ellipse";
    pub const TOOL_POLYGON: &'static str = "aubrieta.tool.shape.polygon";
    pub const TOOL_STAR: &'static str = "aubrieta.tool.shape.star";
    pub const TOOL_SHAPE_BUILDER: &'static str = "aubrieta.tool.shape_builder";
    pub const TOOL_VECTOR_FLOOD_FILL: &'static str = "aubrieta.tool.vector_flood_fill";
    pub const TOOL_ARTISTIC_TEXT: &'static str = "aubrieta.tool.text.artistic";
    pub const TOOL_FRAME_TEXT: &'static str = "aubrieta.tool.text.frame";
    pub const TOOL_GRADIENT: &'static str = "aubrieta.tool.gradient";
    pub const TOOL_TRANSPARENCY: &'static str = "aubrieta.tool.transparency";
    pub const TOOL_COLOR_PICKER: &'static str = "aubrieta.tool.color_picker";
    pub const TOOL_STYLE_PICKER: &'static str = "aubrieta.tool.style_picker";
    pub const TOOL_ARTBOARD: &'static str = "aubrieta.tool.artboard";
    pub const TOOL_MEASURE: &'static str = "aubrieta.tool.measure";
    pub const TOOL_ZOOM: &'static str = "aubrieta.tool.zoom";
    pub const TOOL_PAN: &'static str = "aubrieta.tool.pan";

    // Photo Persona Tool Actions (08.31, 08.33, 10.9, 10.10)
    pub const TOOL_PHOTO_MARQUEE_RECT: &'static str = "aubrieta.tool.photo.marquee_rect";
    pub const TOOL_PHOTO_MARQUEE_ELLIPSE: &'static str = "aubrieta.tool.photo.marquee_ellipse";
    pub const TOOL_PHOTO_LASSO: &'static str = "aubrieta.tool.photo.lasso";
    pub const TOOL_PHOTO_SELECTION_BRUSH: &'static str = "aubrieta.tool.photo.selection_brush";
    pub const TOOL_PHOTO_FLOOD_SELECT: &'static str = "aubrieta.tool.photo.flood_select";
    pub const TOOL_PHOTO_BRUSH: &'static str = "aubrieta.tool.photo.brush";
    pub const TOOL_PHOTO_ERASER: &'static str = "aubrieta.tool.photo.eraser";
    pub const TOOL_PHOTO_GRADIENT: &'static str = "aubrieta.tool.photo.gradient";
    pub const TOOL_PHOTO_CROP: &'static str = "aubrieta.tool.photo.crop";

    // Transformation, Alignment, Geometry Operations (10.1, 10.2, 10.3)
    pub const OBJECT_CONVERT_TO_CURVES: &'static str = "aubrieta.object.convert_to_curves";
    pub const OBJECT_ALIGN: &'static str = "aubrieta.object.align";
    pub const OBJECT_DISTRIBUTE: &'static str = "aubrieta.object.distribute";
    pub const OBJECT_BOOLEAN: &'static str = "aubrieta.object.boolean";
    pub const OBJECT_BAKE_CORNERS: &'static str = "aubrieta.object.bake_corners";
    pub const OBJECT_OFFSET_PATH: &'static str = "aubrieta.object.offset_path";
    pub const OBJECT_SLICE_PATH: &'static str = "aubrieta.object.slice_path";

    /// Creates an action ID. Callers must use the `aubrieta.*` namespace.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Borrows the string identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// User intent before it becomes an undoable [`crate::Command`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionRequest {
    /// Which operation was requested.
    pub action: ActionId,
    /// Opaque JSON payload interpreted by the command layer.
    pub payload: serde_json::Value,
}

impl ActionRequest {
    /// Creates a request with an explicit payload.
    #[must_use]
    pub fn new(action: ActionId, payload: serde_json::Value) -> Self {
        Self { action, payload }
    }

    /// Creates a request without payload.
    #[must_use]
    pub fn without_payload(action: ActionId) -> Self {
        Self {
            action,
            payload: serde_json::Value::Null,
        }
    }
}
