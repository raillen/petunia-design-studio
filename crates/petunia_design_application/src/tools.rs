//! Interactive tool taxonomy across Design and Photo personas
//! (08.24, 08.31, 08.33, 10.1–10.10).
//!
//! Moved from the legacy GUI bridge crate: the tool catalog is domain
//! vocabulary (menus, shortcuts, capabilities, MCP), toolkit-free. The
//! gesture state machines stay in the shell crate.

use serde::{Deserialize, Serialize};

use crate::ActionId;

/// Enumeration of interactive tool types across Design and Photo personas (08.24, 08.31, 08.33, 10.1–10.10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolKind {
    // Design Persona (08.24, 10.1 - 10.7)
    /// Object selection, transformation, and marquee.
    Select,
    /// Direct anchor and node segment manipulation.
    Node,
    /// Interactive transform around custom movable pivot origin.
    PointTransform,
    /// Bézier path plotting and curve construction.
    Pen,
    /// Freehand path sketching and fitting.
    Pencil,
    /// Live corner radius editing on nodes.
    Corner,
    /// Live contour outline offset/inset.
    Contour,
    /// Interactive 4-corner perspective warp (live modifier, 10.8).
    Perspective,
    /// Interactive vector cut across paths.
    Knife,
    /// Split path at hit point.
    Scissors,
    /// Parametric rectangle creation.
    Rectangle,
    /// Parametric ellipse creation.
    Ellipse,
    /// Parametric polygon creation.
    Polygon,
    /// Parametric star creation.
    Star,
    /// Interactive region boolean synthesis.
    ShapeBuilder,
    /// Smart Fill of bounded enclosed regions.
    VectorFloodFill,
    /// Artistic typography headline tool.
    ArtisticText,
    /// Paragraph text frame bounding container.
    FrameText,
    /// Interactive canvas gradient vector line and stop editor.
    Gradient,
    /// Interactive transparency gradient stop editor.
    Transparency,
    /// Canvas eyedropper sampling color.
    ColorPicker,
    /// Eyedropper sampling complete AppearanceStack styles.
    StylePicker,
    /// Surface / Artboard creation and resizing.
    Artboard,
    /// Transient measurement of distance, angle, and area.
    Measure,
    /// Viewport zoom tool.
    Zoom,
    /// Viewport pan / hand tool.
    Hand,

    // Photo Persona (08.31, 10.9, 10.10)
    /// Raster rectangular marquee selection.
    MarqueeRect,
    /// Raster elliptical marquee selection.
    MarqueeEllipse,
    /// Freehand / Lasso raster selection.
    Lasso,
    /// Edge-snapping painted raster selection.
    SelectionBrush,
    /// Contiguous color tolerance selection.
    FloodSelect,
    /// Raster painting on PixelLayer or mask.
    PixelPaintBrush,
    /// Raster pixel/alpha eraser.
    PixelEraser,
    /// Raster pixel-layer gradient fill.
    PhotoGradient,
    /// Nondestructive Surface/document crop or destructive PixelLayer crop.
    Crop,
}

impl ToolKind {
    /// Returns the canonical semantic action identifier in the `ptnd.tool.*` namespace.
    #[must_use]
    pub fn action_id(&self) -> &'static str {
        match self {
            Self::Select => ActionId::TOOL_SELECT,
            Self::Node => ActionId::TOOL_NODE,
            Self::PointTransform => ActionId::TOOL_POINT_TRANSFORM,
            Self::Pen => ActionId::TOOL_PEN,
            Self::Pencil => ActionId::TOOL_PENCIL,
            Self::Corner => ActionId::TOOL_CORNER,
            Self::Contour => ActionId::TOOL_CONTOUR,
            Self::Perspective => ActionId::TOOL_PERSPECTIVE,
            Self::Knife => ActionId::TOOL_KNIFE,
            Self::Scissors => ActionId::TOOL_SCISSORS,
            Self::Rectangle => ActionId::TOOL_RECTANGLE,
            Self::Ellipse => ActionId::TOOL_ELLIPSE,
            Self::Polygon => ActionId::TOOL_POLYGON,
            Self::Star => ActionId::TOOL_STAR,
            Self::ShapeBuilder => ActionId::TOOL_SHAPE_BUILDER,
            Self::VectorFloodFill => ActionId::TOOL_VECTOR_FLOOD_FILL,
            Self::ArtisticText => ActionId::TOOL_ARTISTIC_TEXT,
            Self::FrameText => ActionId::TOOL_FRAME_TEXT,
            Self::Gradient => ActionId::TOOL_GRADIENT,
            Self::Transparency => ActionId::TOOL_TRANSPARENCY,
            Self::ColorPicker => ActionId::TOOL_COLOR_PICKER,
            Self::StylePicker => ActionId::TOOL_STYLE_PICKER,
            Self::Artboard => ActionId::TOOL_ARTBOARD,
            Self::Measure => ActionId::TOOL_MEASURE,
            Self::Zoom => ActionId::TOOL_ZOOM,
            Self::Hand => ActionId::TOOL_PAN,
            Self::MarqueeRect => ActionId::TOOL_PHOTO_MARQUEE_RECT,
            Self::MarqueeEllipse => ActionId::TOOL_PHOTO_MARQUEE_ELLIPSE,
            Self::Lasso => ActionId::TOOL_PHOTO_LASSO,
            Self::SelectionBrush => ActionId::TOOL_PHOTO_SELECTION_BRUSH,
            Self::FloodSelect => ActionId::TOOL_PHOTO_FLOOD_SELECT,
            Self::PixelPaintBrush => ActionId::TOOL_PHOTO_BRUSH,
            Self::PixelEraser => ActionId::TOOL_PHOTO_ERASER,
            Self::PhotoGradient => ActionId::TOOL_PHOTO_GRADIENT,
            Self::Crop => ActionId::TOOL_PHOTO_CROP,
        }
    }

    #[must_use]
    pub fn from_action_id(action_id: &str) -> Option<Self> {
        if action_id == Self::Select.action_id() {
            return Some(Self::Select);
        }
        if action_id == Self::Node.action_id() {
            return Some(Self::Node);
        }
        if action_id == Self::PointTransform.action_id() {
            return Some(Self::PointTransform);
        }
        if action_id == Self::Pen.action_id() {
            return Some(Self::Pen);
        }
        if action_id == Self::Pencil.action_id() {
            return Some(Self::Pencil);
        }
        if action_id == Self::Corner.action_id() {
            return Some(Self::Corner);
        }
        if action_id == Self::Contour.action_id() {
            return Some(Self::Contour);
        }
        if action_id == Self::Perspective.action_id() {
            return Some(Self::Perspective);
        }
        if action_id == Self::Knife.action_id() {
            return Some(Self::Knife);
        }
        if action_id == Self::Scissors.action_id() {
            return Some(Self::Scissors);
        }
        if action_id == Self::Rectangle.action_id() || action_id == "ptnd.tool.shape.rectangle" {
            return Some(Self::Rectangle);
        }
        if action_id == Self::Ellipse.action_id() || action_id == "ptnd.tool.shape.ellipse" {
            return Some(Self::Ellipse);
        }
        if action_id == Self::Polygon.action_id() || action_id == "ptnd.tool.shape.polygon" {
            return Some(Self::Polygon);
        }
        if action_id == Self::Star.action_id() || action_id == "ptnd.tool.shape.star" {
            return Some(Self::Star);
        }
        if action_id == Self::ShapeBuilder.action_id() {
            return Some(Self::ShapeBuilder);
        }
        if action_id == Self::VectorFloodFill.action_id() {
            return Some(Self::VectorFloodFill);
        }
        if action_id == Self::ArtisticText.action_id() || action_id == "ptnd.tool.text.artistic" {
            return Some(Self::ArtisticText);
        }
        if action_id == Self::FrameText.action_id() || action_id == "ptnd.tool.text.frame" {
            return Some(Self::FrameText);
        }
        if action_id == Self::Gradient.action_id() {
            return Some(Self::Gradient);
        }
        if action_id == Self::Transparency.action_id() {
            return Some(Self::Transparency);
        }
        if action_id == Self::ColorPicker.action_id() {
            return Some(Self::ColorPicker);
        }
        if action_id == Self::StylePicker.action_id() {
            return Some(Self::StylePicker);
        }
        if action_id == Self::Artboard.action_id() {
            return Some(Self::Artboard);
        }
        if action_id == Self::Measure.action_id() {
            return Some(Self::Measure);
        }
        if action_id == Self::Zoom.action_id() {
            return Some(Self::Zoom);
        }
        if action_id == Self::Hand.action_id() {
            return Some(Self::Hand);
        }
        if action_id == Self::MarqueeRect.action_id() {
            return Some(Self::MarqueeRect);
        }
        if action_id == Self::MarqueeEllipse.action_id() {
            return Some(Self::MarqueeEllipse);
        }
        if action_id == Self::Lasso.action_id() {
            return Some(Self::Lasso);
        }
        if action_id == Self::SelectionBrush.action_id() {
            return Some(Self::SelectionBrush);
        }
        if action_id == Self::FloodSelect.action_id() {
            return Some(Self::FloodSelect);
        }
        if action_id == Self::PixelPaintBrush.action_id() {
            return Some(Self::PixelPaintBrush);
        }
        if action_id == Self::PixelEraser.action_id() {
            return Some(Self::PixelEraser);
        }
        if action_id == Self::PhotoGradient.action_id() {
            return Some(Self::PhotoGradient);
        }
        if action_id == Self::Crop.action_id() {
            return Some(Self::Crop);
        }
        None
    }

    #[must_use]
    pub fn is_photo_persona(&self) -> bool {
        matches!(
            self,
            Self::MarqueeRect
                | Self::MarqueeEllipse
                | Self::Lasso
                | Self::SelectionBrush
                | Self::FloodSelect
                | Self::PixelPaintBrush
                | Self::PixelEraser
                | Self::PhotoGradient
                | Self::Crop
        )
    }
}
