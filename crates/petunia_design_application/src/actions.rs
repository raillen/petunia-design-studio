//! Stable semantic action identifiers (`ptnd.*` namespace).

use serde::{Deserialize, Serialize};

/// Namespaced action identifier, e.g. `ptnd.surface.create`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActionId(pub String);

impl ActionId {
    /// Well-known actions for the P00 MVP slice.
    pub const SURFACE_CREATE: &'static str = "ptnd.action.surface.create";
    pub const OBJECT_CREATE: &'static str = "ptnd.action.object.create";
    pub const OBJECT_DELETE: &'static str = "ptnd.action.object.delete";
    pub const EDIT_PREFERENCES: &'static str = "ptnd.action.edit.preferences";
    pub const FILL_SET: &'static str = "ptnd.action.fill.set";
    pub const FILE_PLACE: &'static str = "ptnd.action.file.place";

    // Design Persona Tool Actions (08.24, 08.33, 10.1 - 10.7)
    pub const TOOL_SELECT: &'static str = "ptnd.tool.select";
    pub const TOOL_NODE: &'static str = "ptnd.tool.node";
    pub const TOOL_POINT_TRANSFORM: &'static str = "ptnd.tool.point_transform";
    pub const TOOL_PEN: &'static str = "ptnd.tool.pen";
    pub const TOOL_PENCIL: &'static str = "ptnd.tool.pencil";
    pub const TOOL_CORNER: &'static str = "ptnd.tool.corner";
    pub const TOOL_CONTOUR: &'static str = "ptnd.tool.contour";
    pub const TOOL_PERSPECTIVE: &'static str = "ptnd.tool.perspective";
    pub const TOOL_KNIFE: &'static str = "ptnd.tool.knife";
    pub const TOOL_SCISSORS: &'static str = "ptnd.tool.scissors";
    pub const TOOL_RECTANGLE: &'static str = "ptnd.tool.shape.rectangle";
    pub const TOOL_ELLIPSE: &'static str = "ptnd.tool.shape.ellipse";
    pub const TOOL_POLYGON: &'static str = "ptnd.tool.shape.polygon";
    pub const TOOL_STAR: &'static str = "ptnd.tool.shape.star";
    pub const TOOL_SHAPE_BUILDER: &'static str = "ptnd.tool.shape_builder";
    pub const TOOL_VECTOR_FLOOD_FILL: &'static str = "ptnd.tool.vector_flood_fill";
    pub const TOOL_ARTISTIC_TEXT: &'static str = "ptnd.tool.text.artistic";
    pub const TOOL_FRAME_TEXT: &'static str = "ptnd.tool.text.frame";
    pub const TOOL_GRADIENT: &'static str = "ptnd.tool.gradient";
    pub const TOOL_TRANSPARENCY: &'static str = "ptnd.tool.transparency";
    pub const TOOL_COLOR_PICKER: &'static str = "ptnd.tool.color_picker";
    pub const TOOL_STYLE_PICKER: &'static str = "ptnd.tool.style_picker";
    pub const TOOL_ARTBOARD: &'static str = "ptnd.tool.artboard";
    pub const TOOL_MEASURE: &'static str = "ptnd.tool.measure";
    pub const TOOL_ZOOM: &'static str = "ptnd.tool.zoom";
    pub const TOOL_PAN: &'static str = "ptnd.tool.pan";

    // Photo Persona Tool Actions (08.31, 08.33, 10.9, 10.10)
    pub const TOOL_PHOTO_MARQUEE_RECT: &'static str = "ptnd.tool.photo.marquee_rect";
    pub const TOOL_PHOTO_MARQUEE_ELLIPSE: &'static str = "ptnd.tool.photo.marquee_ellipse";
    pub const TOOL_PHOTO_LASSO: &'static str = "ptnd.tool.photo.lasso";
    pub const TOOL_PHOTO_SELECTION_BRUSH: &'static str = "ptnd.tool.photo.selection_brush";
    pub const TOOL_PHOTO_FLOOD_SELECT: &'static str = "ptnd.tool.photo.flood_select";
    pub const TOOL_PHOTO_BRUSH: &'static str = "ptnd.tool.photo.brush";
    pub const TOOL_PHOTO_ERASER: &'static str = "ptnd.tool.photo.eraser";
    pub const TOOL_PHOTO_GRADIENT: &'static str = "ptnd.tool.photo.gradient";
    pub const TOOL_PHOTO_CROP: &'static str = "ptnd.tool.photo.crop";

    // Transformation, Alignment, Geometry Operations (10.1, 10.2, 10.3)
    pub const OBJECT_CONVERT_TO_CURVES: &'static str = "ptnd.action.object.convert_to_curves";
    pub const OBJECT_ALIGN: &'static str = "ptnd.action.object.align";
    pub const OBJECT_DISTRIBUTE: &'static str = "ptnd.action.object.distribute";
    pub const OBJECT_BOOLEAN: &'static str = "ptnd.action.object.boolean";
    pub const OBJECT_BAKE_CORNERS: &'static str = "ptnd.action.object.bake_corners";
    pub const OBJECT_OFFSET_PATH: &'static str = "ptnd.action.object.offset_path";
    pub const OBJECT_SLICE_PATH: &'static str = "ptnd.action.object.slice_path";

    /// Creates an action ID. Callers must use the `ptnd.*` namespace.
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
