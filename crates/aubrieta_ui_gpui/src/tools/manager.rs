//! Tool manager coordinating active tool state, switching, and event dispatch (09.27, 10.1).

use aubrieta_document::ChangeSet;
use aubrieta_foundation::AubrietaError;
use serde::{Deserialize, Serialize};

use crate::bridge::AubrietaGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use super::artboard::ArtboardTool;
use super::contour::{ContourMode, ContourTool};
use super::gradient::{GradientTool, GradientToolMode};
use super::input::NormalizedPointerEvent;
use super::knife::{KnifeMode, KnifeTool};
use super::measure::MeasureTool;
use super::node::NodeTool;
use super::pen::PenTool;
use super::pencil::PencilTool;
use super::photo::{PhotoTool, PhotoToolKind};
use super::picker::{PickerMode, PickerTool};
use super::point_transform::PointTransformTool;
use super::select::SelectTool;
use super::shape::{ShapeKind, ShapeTool};
use super::shape_builder::{BuilderMode, ShapeBuilderTool};
use super::text::{TextTool, TextToolMode};
use super::view::{ViewTool, ViewToolMode};

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
    /// Returns the canonical semantic action identifier in the `aubrieta.tool.*` namespace.
    #[must_use]
    pub fn action_id(&self) -> &'static str {
        match self {
            Self::Select => aubrieta_application::ActionId::TOOL_SELECT,
            Self::Node => aubrieta_application::ActionId::TOOL_NODE,
            Self::PointTransform => aubrieta_application::ActionId::TOOL_POINT_TRANSFORM,
            Self::Pen => aubrieta_application::ActionId::TOOL_PEN,
            Self::Pencil => aubrieta_application::ActionId::TOOL_PENCIL,
            Self::Corner => aubrieta_application::ActionId::TOOL_CORNER,
            Self::Contour => aubrieta_application::ActionId::TOOL_CONTOUR,
            Self::Knife => aubrieta_application::ActionId::TOOL_KNIFE,
            Self::Scissors => aubrieta_application::ActionId::TOOL_SCISSORS,
            Self::Rectangle => aubrieta_application::ActionId::TOOL_RECTANGLE,
            Self::Ellipse => aubrieta_application::ActionId::TOOL_ELLIPSE,
            Self::Polygon => aubrieta_application::ActionId::TOOL_POLYGON,
            Self::Star => aubrieta_application::ActionId::TOOL_STAR,
            Self::ShapeBuilder => aubrieta_application::ActionId::TOOL_SHAPE_BUILDER,
            Self::VectorFloodFill => aubrieta_application::ActionId::TOOL_VECTOR_FLOOD_FILL,
            Self::ArtisticText => aubrieta_application::ActionId::TOOL_ARTISTIC_TEXT,
            Self::FrameText => aubrieta_application::ActionId::TOOL_FRAME_TEXT,
            Self::Gradient => aubrieta_application::ActionId::TOOL_GRADIENT,
            Self::Transparency => aubrieta_application::ActionId::TOOL_TRANSPARENCY,
            Self::ColorPicker => aubrieta_application::ActionId::TOOL_COLOR_PICKER,
            Self::StylePicker => aubrieta_application::ActionId::TOOL_STYLE_PICKER,
            Self::Artboard => aubrieta_application::ActionId::TOOL_ARTBOARD,
            Self::Measure => aubrieta_application::ActionId::TOOL_MEASURE,
            Self::Zoom => aubrieta_application::ActionId::TOOL_ZOOM,
            Self::Hand => aubrieta_application::ActionId::TOOL_PAN,
            Self::MarqueeRect => aubrieta_application::ActionId::TOOL_PHOTO_MARQUEE_RECT,
            Self::MarqueeEllipse => aubrieta_application::ActionId::TOOL_PHOTO_MARQUEE_ELLIPSE,
            Self::Lasso => aubrieta_application::ActionId::TOOL_PHOTO_LASSO,
            Self::SelectionBrush => aubrieta_application::ActionId::TOOL_PHOTO_SELECTION_BRUSH,
            Self::FloodSelect => aubrieta_application::ActionId::TOOL_PHOTO_FLOOD_SELECT,
            Self::PixelPaintBrush => aubrieta_application::ActionId::TOOL_PHOTO_BRUSH,
            Self::PixelEraser => aubrieta_application::ActionId::TOOL_PHOTO_ERASER,
            Self::PhotoGradient => aubrieta_application::ActionId::TOOL_PHOTO_GRADIENT,
            Self::Crop => aubrieta_application::ActionId::TOOL_PHOTO_CROP,
        }
    }

    /// True if this tool belongs to the Photo persona.
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

/// Central manager orchestrating vector tools and event routing.
#[derive(Debug)]
pub struct ToolManager {
    active_kind: ToolKind,
    select_tool: SelectTool,
    pen_tool: PenTool,
    node_tool: NodeTool,
    point_transform_tool: PointTransformTool,
    pencil_tool: PencilTool,
    corner_tool: ContourTool,
    contour_tool: ContourTool,
    knife_tool: KnifeTool,
    scissors_tool: KnifeTool,
    rectangle_tool: ShapeTool,
    ellipse_tool: ShapeTool,
    polygon_tool: ShapeTool,
    star_tool: ShapeTool,
    shape_builder_tool: ShapeBuilderTool,
    smart_fill_tool: ShapeBuilderTool,
    artistic_text_tool: TextTool,
    frame_text_tool: TextTool,
    gradient_tool: GradientTool,
    transparency_tool: GradientTool,
    color_picker_tool: PickerTool,
    style_picker_tool: PickerTool,
    artboard_tool: ArtboardTool,
    measure_tool: MeasureTool,
    zoom_tool: ViewTool,
    hand_tool: ViewTool,
    photo_marquee_rect_tool: PhotoTool,
    photo_marquee_ellipse_tool: PhotoTool,
    photo_lasso_tool: PhotoTool,
    photo_selection_brush_tool: PhotoTool,
    photo_flood_select_tool: PhotoTool,
    photo_brush_tool: PhotoTool,
    photo_eraser_tool: PhotoTool,
    photo_gradient_tool: GradientTool,
    photo_crop_tool: PhotoTool,
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
            point_transform_tool: PointTransformTool::new(),
            pencil_tool: PencilTool::new(),
            corner_tool: ContourTool::new(ContourMode::Corner),
            contour_tool: ContourTool::new(ContourMode::Contour),
            knife_tool: KnifeTool::new(KnifeMode::Knife),
            scissors_tool: KnifeTool::new(KnifeMode::Scissors),
            rectangle_tool: ShapeTool::new(ShapeKind::Rectangle),
            ellipse_tool: ShapeTool::new(ShapeKind::Ellipse),
            polygon_tool: ShapeTool::new(ShapeKind::Polygon),
            star_tool: ShapeTool::new(ShapeKind::Star),
            shape_builder_tool: ShapeBuilderTool::new(BuilderMode::ShapeBuilder),
            smart_fill_tool: ShapeBuilderTool::new(BuilderMode::SmartFill),
            artistic_text_tool: TextTool::new(TextToolMode::Artistic),
            frame_text_tool: TextTool::new(TextToolMode::Frame),
            gradient_tool: GradientTool::new(GradientToolMode::Fill),
            transparency_tool: GradientTool::new(GradientToolMode::Transparency),
            color_picker_tool: PickerTool::new(PickerMode::Color),
            style_picker_tool: PickerTool::new(PickerMode::Style),
            artboard_tool: ArtboardTool::new(),
            measure_tool: MeasureTool::new(),
            zoom_tool: ViewTool::new(ViewToolMode::Zoom),
            hand_tool: ViewTool::new(ViewToolMode::Pan),
            photo_marquee_rect_tool: PhotoTool::new(PhotoToolKind::MarqueeRect),
            photo_marquee_ellipse_tool: PhotoTool::new(PhotoToolKind::MarqueeEllipse),
            photo_lasso_tool: PhotoTool::new(PhotoToolKind::Lasso),
            photo_selection_brush_tool: PhotoTool::new(PhotoToolKind::SelectionBrush),
            photo_flood_select_tool: PhotoTool::new(PhotoToolKind::FloodSelect),
            photo_brush_tool: PhotoTool::new(PhotoToolKind::Brush),
            photo_eraser_tool: PhotoTool::new(PhotoToolKind::Eraser),
            photo_gradient_tool: GradientTool::new(GradientToolMode::Fill),
            photo_crop_tool: PhotoTool::new(PhotoToolKind::Crop),
        }
    }

    /// Returns the currently active tool kind.
    #[must_use]
    pub fn active_kind(&self) -> ToolKind {
        self.active_kind
    }

    /// Switches the active tool, canceling any in-progress gesture on the previous tool.
    pub fn set_active_tool(&mut self, kind: ToolKind) {
        if self.active_kind != kind {
            self.cancel_active();
            self.active_kind = kind;
        }
    }

    /// Alias for `set_active_tool`.
    pub fn set_tool(&mut self, kind: ToolKind) {
        self.set_active_tool(kind);
    }

    /// Cancels any active gesture in the current tool.
    pub fn cancel_active(&mut self) {
        match self.active_kind {
            ToolKind::Select => self.select_tool.cancel(),
            ToolKind::Pen => self.pen_tool.cancel(),
            ToolKind::Node => self.node_tool.cancel(),
            ToolKind::PointTransform => self.point_transform_tool.cancel(),
            ToolKind::Pencil => self.pencil_tool.cancel(),
            ToolKind::Corner => self.corner_tool.cancel(),
            ToolKind::Contour => self.contour_tool.cancel(),
            ToolKind::Knife => self.knife_tool.cancel(),
            ToolKind::Scissors => self.scissors_tool.cancel(),
            ToolKind::Rectangle => self.rectangle_tool.cancel(),
            ToolKind::Ellipse => self.ellipse_tool.cancel(),
            ToolKind::Polygon => self.polygon_tool.cancel(),
            ToolKind::Star => self.star_tool.cancel(),
            ToolKind::ShapeBuilder => self.shape_builder_tool.cancel(),
            ToolKind::VectorFloodFill => self.smart_fill_tool.cancel(),
            ToolKind::ArtisticText => self.artistic_text_tool.cancel(),
            ToolKind::FrameText => self.frame_text_tool.cancel(),
            ToolKind::Gradient => self.gradient_tool.cancel(),
            ToolKind::Transparency => self.transparency_tool.cancel(),
            ToolKind::ColorPicker => self.color_picker_tool.cancel(),
            ToolKind::StylePicker => self.style_picker_tool.cancel(),
            ToolKind::Artboard => self.artboard_tool.cancel(),
            ToolKind::Measure => self.measure_tool.cancel(),
            ToolKind::Zoom => self.zoom_tool.cancel(),
            ToolKind::Hand => self.hand_tool.cancel(),
            ToolKind::MarqueeRect => self.photo_marquee_rect_tool.cancel(),
            ToolKind::MarqueeEllipse => self.photo_marquee_ellipse_tool.cancel(),
            ToolKind::Lasso => self.photo_lasso_tool.cancel(),
            ToolKind::SelectionBrush => self.photo_selection_brush_tool.cancel(),
            ToolKind::FloodSelect => self.photo_flood_select_tool.cancel(),
            ToolKind::PixelPaintBrush => self.photo_brush_tool.cancel(),
            ToolKind::PixelEraser => self.photo_eraser_tool.cancel(),
            ToolKind::PhotoGradient => self.photo_gradient_tool.cancel(),
            ToolKind::Crop => self.photo_crop_tool.cancel(),
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
            ToolKind::PointTransform => self
                .point_transform_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Pencil => self
                .pencil_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Corner => self
                .corner_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Contour => self
                .contour_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Knife => self
                .knife_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Scissors => self
                .scissors_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Rectangle => self
                .rectangle_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Ellipse => self
                .ellipse_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Polygon => self
                .polygon_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Star => self.star_tool.on_pointer_event(event, bridge, camera, snap),
            ToolKind::ShapeBuilder => self
                .shape_builder_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::VectorFloodFill => self
                .smart_fill_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::ArtisticText => self
                .artistic_text_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::FrameText => self
                .frame_text_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Gradient => self
                .gradient_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Transparency => self
                .transparency_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::ColorPicker => self
                .color_picker_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::StylePicker => self
                .style_picker_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Artboard => self
                .artboard_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Measure => self
                .measure_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Zoom => self.zoom_tool.on_pointer_event(event, bridge, camera, snap),
            ToolKind::Hand => self.hand_tool.on_pointer_event(event, bridge, camera, snap),
            ToolKind::MarqueeRect => self
                .photo_marquee_rect_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::MarqueeEllipse => self
                .photo_marquee_ellipse_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Lasso => self
                .photo_lasso_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::SelectionBrush => self
                .photo_selection_brush_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::FloodSelect => self
                .photo_flood_select_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::PixelPaintBrush => self
                .photo_brush_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::PixelEraser => self
                .photo_eraser_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::PhotoGradient => self
                .photo_gradient_tool
                .on_pointer_event(event, bridge, camera, snap),
            ToolKind::Crop => self
                .photo_crop_tool
                .on_pointer_event(event, bridge, camera, snap),
        }
    }

    /// Resolves overlays produced by the active tool.
    #[must_use]
    pub fn overlays(&self, camera: &ViewportCamera, bridge: &AubrietaGuiBridge) -> CanvasOverlays {
        match self.active_kind {
            ToolKind::Select => self.select_tool.overlays(camera, bridge),
            ToolKind::Pen => self.pen_tool.overlays(),
            ToolKind::Node => self.node_tool.overlays(camera, bridge),
            ToolKind::PointTransform => self.point_transform_tool.overlays(),
            ToolKind::Pencil => self.pencil_tool.overlays(),
            ToolKind::Corner => self.corner_tool.overlays(),
            ToolKind::Contour => self.contour_tool.overlays(),
            ToolKind::Knife => self.knife_tool.overlays(),
            ToolKind::Scissors => self.scissors_tool.overlays(),
            ToolKind::Rectangle => self.rectangle_tool.overlays(camera),
            ToolKind::Ellipse => self.ellipse_tool.overlays(camera),
            ToolKind::Polygon => self.polygon_tool.overlays(camera),
            ToolKind::Star => self.star_tool.overlays(camera),
            ToolKind::ShapeBuilder => self.shape_builder_tool.overlays(),
            ToolKind::VectorFloodFill => self.smart_fill_tool.overlays(),
            ToolKind::ArtisticText => self.artistic_text_tool.overlays(camera),
            ToolKind::FrameText => self.frame_text_tool.overlays(camera),
            ToolKind::Gradient => self.gradient_tool.overlays(),
            ToolKind::Transparency => self.transparency_tool.overlays(),
            ToolKind::ColorPicker => self.color_picker_tool.overlays(),
            ToolKind::StylePicker => self.style_picker_tool.overlays(),
            ToolKind::Artboard => self.artboard_tool.overlays(camera),
            ToolKind::Measure => self.measure_tool.overlays(),
            ToolKind::Zoom => self.zoom_tool.overlays(),
            ToolKind::Hand => self.hand_tool.overlays(),
            ToolKind::MarqueeRect => self.photo_marquee_rect_tool.overlays(camera),
            ToolKind::MarqueeEllipse => self.photo_marquee_ellipse_tool.overlays(camera),
            ToolKind::Lasso => self.photo_lasso_tool.overlays(camera),
            ToolKind::SelectionBrush => self.photo_selection_brush_tool.overlays(camera),
            ToolKind::FloodSelect => self.photo_flood_select_tool.overlays(camera),
            ToolKind::PixelPaintBrush => self.photo_brush_tool.overlays(camera),
            ToolKind::PixelEraser => self.photo_eraser_tool.overlays(camera),
            ToolKind::PhotoGradient => self.photo_gradient_tool.overlays(),
            ToolKind::Crop => self.photo_crop_tool.overlays(camera),
        }
    }
}
