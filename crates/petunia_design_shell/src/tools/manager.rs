//! Tool manager coordinating active tool state, switching, and event dispatch (09.27, 10.1).

use petunia_design_application::tools::ToolKind;
use petunia_design_document::ChangeSet;
use petunia_design_foundation::PetuniaError;

use crate::bridge::PetuniaDesignGuiBridge;
use crate::canvas::{CanvasOverlays, SnapEngine, ViewportCamera};

use super::artboard::ArtboardTool;
use super::contour::{ContourMode, ContourTool};
use super::gradient::{GradientTool, GradientToolMode};
use super::knife::{KnifeMode, KnifeTool};
use super::measure::MeasureTool;
use super::node::NodeTool;
use super::pen::PenTool;
use super::pencil::PencilTool;
use super::perspective::PerspectiveTool;
use super::photo::{PhotoTool, PhotoToolKind};
use super::picker::{PickerMode, PickerTool};
use super::point_transform::PointTransformTool;
use super::select::SelectTool;
use super::shape::{ShapeKind, ShapeTool};
use super::shape_builder::{BuilderMode, ShapeBuilderTool};
use super::text::{TextTool, TextToolMode};
use super::view::{ViewTool, ViewToolMode};
use petunia_design_application::interaction::NormalizedPointerEvent;

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
    perspective_tool: PerspectiveTool,
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
            perspective_tool: PerspectiveTool::new(),
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

    /// Borrows the Select tool for settings/gesture configuration.
    #[must_use]
    pub fn select_tool(&self) -> &super::select::SelectTool {
        &self.select_tool
    }

    /// Mutably borrows the Select tool (gesture mode, marquee rule).
    pub fn select_tool_mut(&mut self) -> &mut super::select::SelectTool {
        &mut self.select_tool
    }

    /// Switches the Select empty-canvas gesture (rectangle vs. lasso).
    pub fn set_select_gesture_mode(&mut self, mode: super::select::SelectGestureMode) {
        self.select_tool.set_gesture_mode(mode);
    }

    /// Sets the Select marquee rule backing the settings menu option
    /// (overlap vs. fully contained vs. directional).
    pub fn set_select_marquee_rule(&mut self, rule: super::select::MarqueeSelectRule) {
        self.select_tool.set_marquee_rule(rule);
    }

    /// Borrows the Measure tool.
    #[must_use]
    pub fn measure_tool(&self) -> &super::measure::MeasureTool {
        &self.measure_tool
    }

    /// Mutably borrows the Measure tool.
    pub fn measure_tool_mut(&mut self) -> &mut super::measure::MeasureTool {
        &mut self.measure_tool
    }

    /// Borrows the Gradient tool.
    #[must_use]
    pub fn gradient_tool(&self) -> &super::gradient::GradientTool {
        &self.gradient_tool
    }

    /// Mutably borrows the Gradient tool.
    pub fn gradient_tool_mut(&mut self) -> &mut super::gradient::GradientTool {
        &mut self.gradient_tool
    }

    /// Borrows the Node tool.
    #[must_use]
    pub fn node_tool(&self) -> &super::node::NodeTool {
        &self.node_tool
    }

    /// Mutably borrows the Node tool.
    pub fn node_tool_mut(&mut self) -> &mut super::node::NodeTool {
        &mut self.node_tool
    }

    /// Borrows the Pen tool.
    #[must_use]
    pub fn pen_tool(&self) -> &super::pen::PenTool {
        &self.pen_tool
    }

    /// Mutably borrows the Pen tool.
    pub fn pen_tool_mut(&mut self) -> &mut super::pen::PenTool {
        &mut self.pen_tool
    }

    /// Borrows the Pencil tool.
    #[must_use]
    pub fn pencil_tool(&self) -> &super::pencil::PencilTool {
        &self.pencil_tool
    }

    /// Mutably borrows the Pencil tool.
    pub fn pencil_tool_mut(&mut self) -> &mut super::pencil::PencilTool {
        &mut self.pencil_tool
    }

    /// Borrows the Photo raster brush tool.
    #[must_use]
    pub fn photo_brush_tool(&self) -> &super::photo::PhotoTool {
        &self.photo_brush_tool
    }

    /// Mutably borrows the Photo raster brush tool.
    pub fn photo_brush_tool_mut(&mut self) -> &mut super::photo::PhotoTool {
        &mut self.photo_brush_tool
    }

    /// Borrows the Photo raster eraser tool.
    #[must_use]
    pub fn photo_eraser_tool(&self) -> &super::photo::PhotoTool {
        &self.photo_eraser_tool
    }

    /// Mutably borrows the Photo raster eraser tool.
    pub fn photo_eraser_tool_mut(&mut self) -> &mut super::photo::PhotoTool {
        &mut self.photo_eraser_tool
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
            ToolKind::Perspective => self.perspective_tool.cancel(),
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
        bridge: &mut PetuniaDesignGuiBridge,
        camera: &ViewportCamera,
        snap: &mut SnapEngine,
    ) -> Result<ChangeSet, PetuniaError> {
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
            ToolKind::Perspective => self
                .perspective_tool
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

    /// Takes a pending camera action from the active view tool, if any.
    /// The shell drains this after dispatch and applies it to the camera.
    pub fn take_camera_action(&mut self) -> Option<super::view::CameraAction> {
        match self.active_kind {
            ToolKind::Zoom => self.zoom_tool.take_camera_action(),
            ToolKind::Hand => self.hand_tool.take_camera_action(),
            _ => None,
        }
    }

    /// Resolves overlays produced by the active tool.
    #[must_use]
    pub fn overlays(
        &self,
        camera: &ViewportCamera,
        bridge: &PetuniaDesignGuiBridge,
    ) -> CanvasOverlays {
        match self.active_kind {
            ToolKind::Select => self.select_tool.overlays(camera, bridge),
            ToolKind::Pen => self.pen_tool.overlays(),
            ToolKind::Node => self.node_tool.overlays(camera, bridge),
            ToolKind::PointTransform => self.point_transform_tool.overlays(),
            ToolKind::Pencil => self.pencil_tool.overlays(),
            ToolKind::Corner => self.corner_tool.overlays(bridge, camera),
            ToolKind::Contour => self.contour_tool.overlays(bridge, camera),
            ToolKind::Perspective => self.perspective_tool.overlays(bridge, camera),
            ToolKind::Knife => self.knife_tool.overlays(camera, bridge),
            ToolKind::Scissors => self.scissors_tool.overlays(camera, bridge),
            ToolKind::Rectangle => self.rectangle_tool.overlays(camera),
            ToolKind::Ellipse => self.ellipse_tool.overlays(camera),
            ToolKind::Polygon => self.polygon_tool.overlays(camera),
            ToolKind::Star => self.star_tool.overlays(camera),
            ToolKind::ShapeBuilder => self.shape_builder_tool.overlays(bridge),
            ToolKind::VectorFloodFill => self.smart_fill_tool.overlays(bridge),
            ToolKind::ArtisticText => self.artistic_text_tool.overlays(camera, bridge),
            ToolKind::FrameText => self.frame_text_tool.overlays(camera, bridge),
            ToolKind::Gradient => self.gradient_tool.overlays(bridge, camera),
            ToolKind::Transparency => self.transparency_tool.overlays(bridge, camera),
            ToolKind::ColorPicker => self.color_picker_tool.overlays(bridge),
            ToolKind::StylePicker => self.style_picker_tool.overlays(bridge),
            ToolKind::Artboard => self.artboard_tool.overlays(camera),
            ToolKind::Measure => self.measure_tool.overlays(camera),
            ToolKind::Zoom => self.zoom_tool.overlays(),
            ToolKind::Hand => self.hand_tool.overlays(),
            ToolKind::MarqueeRect => self.photo_marquee_rect_tool.overlays(camera, bridge),
            ToolKind::MarqueeEllipse => self.photo_marquee_ellipse_tool.overlays(camera, bridge),
            ToolKind::Lasso => self.photo_lasso_tool.overlays(camera, bridge),
            ToolKind::SelectionBrush => self.photo_selection_brush_tool.overlays(camera, bridge),
            ToolKind::FloodSelect => self.photo_flood_select_tool.overlays(camera, bridge),
            ToolKind::PixelPaintBrush => self.photo_brush_tool.overlays(camera, bridge),
            ToolKind::PixelEraser => self.photo_eraser_tool.overlays(camera, bridge),
            ToolKind::PhotoGradient => self.photo_gradient_tool.overlays(bridge, camera),
            ToolKind::Crop => self.photo_crop_tool.overlays(camera, bridge),
        }
    }
}
