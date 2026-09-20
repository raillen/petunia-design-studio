//! Desktop frame buffer and UI chrome renderer (08.7, 08.15, 08.17, 09.24).
//!
//! Renders the complete Aubrieta interface into a 32-bit (0x00RRGGBB) pixel buffer:
//! Menu Bar, Context Toolbar, Tool Rail, Canvas Viewport with Artboards/Surfaces/Bleed,
//! Selection Overlays, Dock Panels (Layers, Properties, History, Data Merge), and Status Bar.

use aubrieta_geometry::{GPoint, GRect};
use aubrieta_ui_gpui::bridge::{
    DataMergePresentationModel, HistoryPresentationModel, LayersPresentationModel,
    PropertiesPresentationModel,
};
use aubrieta_ui_gpui::canvas::{CanvasOverlays, SnapOrientation};
use aubrieta_ui_gpui::shell::AubrietaShell;
use aubrieta_ui_gpui::tools::ToolKind;

use crate::font::draw_text;

// Semantic UI Color Palette (DTCG tokens mapped to 0x00RRGGBB)
pub const COLOR_BG_APP: u32 = 0x18181B; // Zinc 900
pub const COLOR_BG_HEADER: u32 = 0x27272A; // Zinc 800
pub const COLOR_BG_TOOLBAR: u32 = 0x202024; // Zinc 850
pub const COLOR_BG_RAIL: u32 = 0x242428;
pub const COLOR_BG_DOCK: u32 = 0x222226;
pub const COLOR_BG_CANVAS: u32 = 0x141416;
pub const COLOR_BG_PANEL_HEADER: u32 = 0x2A2A30;
pub const COLOR_BORDER: u32 = 0x3F3F46; // Zinc 700
pub const COLOR_BORDER_SUBTLE: u32 = 0x2E2E33;
pub const COLOR_TEXT_PRIMARY: u32 = 0xF4F4F5; // Zinc 100
pub const COLOR_TEXT_SECONDARY: u32 = 0xA1A1AA; // Zinc 400
pub const COLOR_TEXT_MUTED: u32 = 0x71717A; // Zinc 500
pub const COLOR_ACCENT_PRIMARY: u32 = 0x3B82F6; // Blue 500
pub const COLOR_ACCENT_ACTIVE: u32 = 0x2563EB; // Blue 600
pub const COLOR_PAPER: u32 = 0xFFFFFF; // Artboard paper white
pub const COLOR_PAPER_SHADOW: u32 = 0x0A0A0C;
pub const COLOR_GUIDE_BLEED: u32 = 0xF43F5E; // Rose 500
pub const COLOR_GUIDE_MARGIN: u32 = 0x38BDF8; // Sky 400
pub const COLOR_GUIDE_SNAP: u32 = 0xEC4899; // Pink 500
pub const COLOR_SELECTION_HANDLE: u32 = 0x60A5FA; // Light Blue

/// Active inspector panel tab selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActiveDockTab {
    Layers,
    Properties,
    History,
    DataMerge,
}

/// Contiguous 32-bit pixel buffer for desktop window presentation.
#[derive(Clone, Debug)]
pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

impl FrameBuffer {
    /// Creates a fresh frame buffer with dimensions.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![COLOR_BG_APP; width * height],
        }
    }

    /// Clears the entire buffer with a color.
    pub fn clear(&mut self, color: u32) {
        self.pixels.fill(color);
    }

    /// Sets a pixel at (x, y) if in bounds.
    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x] = color;
        }
    }

    /// Draws a solid rectangle.
    pub fn fill_rect(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
        let min_x = x0.min(self.width);
        let max_x = x1.min(self.width);
        let min_y = y0.min(self.height);
        let max_y = y1.min(self.height);

        for y in min_y..max_y {
            let row_start = y * self.width;
            for x in min_x..max_x {
                self.pixels[row_start + x] = color;
            }
        }
    }

    /// Draws a stroked rectangle outline.
    pub fn stroke_rect(
        &mut self,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        color: u32,
        thickness: usize,
    ) {
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        // Top edge
        self.fill_rect(x0, y0, x1, (y0 + thickness).min(y1), color);
        // Bottom edge
        self.fill_rect(x0, y1.saturating_sub(thickness).max(y0), x1, y1, color);
        // Left edge
        self.fill_rect(x0, y0, (x0 + thickness).min(x1), y1, color);
        // Right edge
        self.fill_rect(x1.saturating_sub(thickness).max(x0), y0, x1, y1, color);
    }

    /// Draws a Bresenham line between two integer coordinates.
    pub fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: u32) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            if x0 >= 0 && (x0 as usize) < self.width && y0 >= 0 && (y0 as usize) < self.height {
                self.set_pixel(x0 as usize, y0 as usize, color);
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Draws a button with background, border, and centered text.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_button(
        &mut self,
        x0: usize,
        y0: usize,
        w: usize,
        h: usize,
        text: &str,
        is_active: bool,
        is_hovered: bool,
    ) {
        let bg_color = if is_active {
            COLOR_ACCENT_PRIMARY
        } else if is_hovered {
            0x323238
        } else {
            0x26262B
        };
        let border_color = if is_active {
            COLOR_ACCENT_ACTIVE
        } else {
            COLOR_BORDER
        };
        let text_color = if is_active {
            0xFFFFFF
        } else {
            COLOR_TEXT_PRIMARY
        };

        self.fill_rect(x0, y0, x0 + w, y0 + h, bg_color);
        self.stroke_rect(x0, y0, x0 + w, y0 + h, border_color, 1);

        let text_w = text.len() * 8;
        let text_x = if text_w < w {
            x0 + (w - text_w) / 2
        } else {
            x0 + 2
        };
        let text_y = if h > 8 { y0 + (h - 8) / 2 } else { y0 };
        draw_text(
            &mut self.pixels,
            self.width,
            self.height,
            text_x,
            text_y,
            text,
            text_color,
        );
    }
}

/// Master UI Layout geometry constants.
pub struct UiLayout {
    pub width: usize,
    pub height: usize,
    pub header_h: usize,
    pub context_h: usize,
    pub rail_w: usize,
    pub dock_w: usize,
    pub status_h: usize,
}

impl UiLayout {
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            header_h: 30,
            context_h: 34,
            rail_w: 48,
            dock_w: 280,
            status_h: 24,
        }
    }

    /// Canvas rectangle in screen pixels: (x0, y0, x1, y1).
    #[must_use]
    pub fn canvas_rect(&self) -> (usize, usize, usize, usize) {
        let x0 = self.rail_w;
        let y0 = self.header_h + self.context_h;
        let x1 = self.width.saturating_sub(self.dock_w);
        let y1 = self.height.saturating_sub(self.status_h);
        (x0, y0, x1, y1)
    }

    /// Returns true if a screen coordinate is inside the canvas viewport.
    #[must_use]
    pub fn is_in_canvas(&self, x: usize, y: usize) -> bool {
        let (x0, y0, x1, y1) = self.canvas_rect();
        x >= x0 && x < x1 && y >= y0 && y < y1
    }
}

/// Renders the complete Aubrieta desktop window into the frame buffer.
pub fn render_desktop_ui(
    fb: &mut FrameBuffer,
    shell: &mut AubrietaShell,
    active_tab: ActiveDockTab,
    hover_coords: Option<(usize, usize)>,
) {
    let layout = UiLayout::new(fb.width, fb.height);
    fb.clear(COLOR_BG_APP);

    // 1. Top Header & Menu Bar (0..30)
    render_header_bar(fb, &layout, shell);

    // 2. Context Toolbar (30..64)
    render_context_toolbar(fb, &layout, shell);

    // 3. Left Tool Rail (0..48 horizontal, 64..776 vertical)
    render_tool_rail(fb, &layout, shell);

    // 4. Center Canvas Viewport (48..1000 horizontal, 64..776 vertical)
    render_canvas_viewport(fb, &layout, shell);

    // 5. Right Inspector Dock (1000..1280 horizontal, 64..776 vertical)
    render_dock_panels(fb, &layout, shell, active_tab);

    // 6. Bottom Status Bar (776..800 vertical)
    render_status_bar(fb, &layout, shell, hover_coords);
}

fn render_header_bar(fb: &mut FrameBuffer, layout: &UiLayout, shell: &AubrietaShell) {
    fb.fill_rect(0, 0, layout.width, layout.header_h, COLOR_BG_HEADER);
    fb.stroke_rect(
        0,
        layout.header_h - 1,
        layout.width,
        layout.header_h,
        COLOR_BORDER,
        1,
    );

    // App Branding
    draw_text(
        &mut fb.pixels,
        fb.width,
        fb.height,
        12,
        10,
        "[A] Aubrieta Design Studio",
        0x60A5FA, // Blue brand color
    );

    // Canonical Menus
    let menus = ["File", "Edit", "Object", "Layer", "View", "Window", "Help"];
    let mut menu_x = 240;
    for menu in menus {
        draw_text(
            &mut fb.pixels,
            fb.width,
            fb.height,
            menu_x,
            10,
            menu,
            COLOR_TEXT_SECONDARY,
        );
        menu_x += menu.len() * 8 + 20;
    }

    // Document Title Tab
    let snap = shell.snapshot();
    let dirty_marker = if snap.is_dirty { " *" } else { "" };
    let doc_label = format!("📄 {}{}", snap.title, dirty_marker);
    let tab_x = layout.width.saturating_sub(320);
    fb.fill_rect(tab_x, 4, tab_x + 220, layout.header_h - 1, 0x1E1E24);
    fb.stroke_rect(tab_x, 4, tab_x + 220, layout.header_h - 1, COLOR_BORDER, 1);
    draw_text(
        &mut fb.pixels,
        fb.width,
        fb.height,
        tab_x + 10,
        10,
        &doc_label,
        COLOR_TEXT_PRIMARY,
    );
}

fn render_context_toolbar(fb: &mut FrameBuffer, layout: &UiLayout, shell: &AubrietaShell) {
    let y0 = layout.header_h;
    let y1 = y0 + layout.context_h;
    fb.fill_rect(0, y0, layout.width, y1, COLOR_BG_TOOLBAR);
    fb.stroke_rect(0, y1 - 1, layout.width, y1, COLOR_BORDER_SUBTLE, 1);

    // Active tool indicator
    let tool_name = match shell.active_tool() {
        ToolKind::Select => "Tool: Select (V)",
        ToolKind::Node => "Tool: Node (A)",
        ToolKind::PointTransform => "Tool: Point Transform (F)",
        ToolKind::Pen => "Tool: Pen (P)",
        ToolKind::Pencil => "Tool: Pencil (N)",
        ToolKind::Corner => "Tool: Corner (C)",
        ToolKind::Contour => "Tool: Contour (J)",
        ToolKind::Knife => "Tool: Knife (K)",
        ToolKind::Scissors => "Tool: Scissors (S)",
        ToolKind::Rectangle => "Tool: Rectangle (M)",
        ToolKind::Ellipse => "Tool: Ellipse (E)",
        ToolKind::Polygon => "Tool: Polygon (G)",
        ToolKind::Star => "Tool: Star",
        ToolKind::ShapeBuilder => "Tool: Shape Builder (W)",
        ToolKind::VectorFloodFill => "Tool: Smart Fill",
        ToolKind::ArtisticText => "Tool: Artistic Text (T)",
        ToolKind::FrameText => "Tool: Frame Text",
        ToolKind::Gradient => "Tool: Gradient (G)",
        ToolKind::Transparency => "Tool: Transparency (Y)",
        ToolKind::ColorPicker => "Tool: Color Picker (I)",
        ToolKind::StylePicker => "Tool: Style Picker",
        ToolKind::Artboard => "Tool: Artboard (H)",
        ToolKind::Measure => "Tool: Measure",
        ToolKind::Zoom => "Tool: Zoom (Z)",
        ToolKind::Hand => "Tool: Hand (Space)",
        ToolKind::MarqueeRect => "Tool: Marquee Rect",
        ToolKind::MarqueeEllipse => "Tool: Marquee Ellipse",
        ToolKind::Lasso => "Tool: Lasso (L)",
        ToolKind::SelectionBrush => "Tool: Selection Brush",
        ToolKind::FloodSelect => "Tool: Flood Select",
        ToolKind::PixelPaintBrush => "Tool: Brush (B)",
        ToolKind::PixelEraser => "Tool: Eraser (E)",
        ToolKind::PhotoGradient => "Tool: Photo Gradient",
        ToolKind::Crop => "Tool: Crop (C)",
    };
    fb.draw_button(12, y0 + 5, 170, 24, tool_name, true, false);

    // Quick Undo / Redo buttons
    fb.draw_button(190, y0 + 5, 60, 24, "Undo", false, false);
    fb.draw_button(255, y0 + 5, 60, 24, "Redo", false, false);

    // Zoom and Viewport controls
    let zoom_pct = (shell.camera.zoom * 100.0).round() as i32;
    let zoom_label = format!("Zoom: {zoom_pct}%");
    fb.draw_button(325, y0 + 5, 100, 24, &zoom_label, false, false);
    fb.draw_button(430, y0 + 5, 45, 24, "Fit", false, false);
    fb.draw_button(480, y0 + 5, 30, 24, "+", false, false);
    fb.draw_button(515, y0 + 5, 30, 24, "-", false, false);

    // Magnetic Snapping Toggle
    let snap_on = shell.snap.config.grid_enabled || shell.snap.config.guides_enabled;
    let snap_label = if snap_on { "Snap: ON" } else { "Snap: OFF" };
    fb.draw_button(560, y0 + 5, 80, 24, snap_label, false, false);
}

fn render_tool_rail(fb: &mut FrameBuffer, layout: &UiLayout, shell: &AubrietaShell) {
    let y0 = layout.header_h + layout.context_h;
    let y1 = layout.height - layout.status_h;
    fb.fill_rect(0, y0, layout.rail_w, y1, COLOR_BG_RAIL);
    fb.stroke_rect(
        layout.rail_w - 1,
        y0,
        layout.rail_w,
        y1,
        COLOR_BORDER_SUBTLE,
        1,
    );

    let tools = [
        (ToolKind::Select, "V", "Sel"),
        (ToolKind::Node, "A", "Nod"),
        (ToolKind::Pen, "P", "Pen"),
        (ToolKind::Rectangle, "M", "Rec"),
        (ToolKind::Ellipse, "E", "Ell"),
        (ToolKind::Polygon, "G", "Pol"),
    ];

    let mut btn_y = y0 + 10;
    for (kind, shortcut, label) in tools {
        let is_active = shell.active_tool() == kind;
        fb.draw_button(6, btn_y, 36, 36, label, is_active, false);
        // Small shortcut key hint
        draw_text(
            &mut fb.pixels,
            fb.width,
            fb.height,
            28,
            btn_y + 24,
            shortcut,
            COLOR_TEXT_MUTED,
        );
        btn_y += 44;
    }
}

fn render_canvas_viewport(fb: &mut FrameBuffer, layout: &UiLayout, shell: &mut AubrietaShell) {
    let (cx0, cy0, cx1, cy1) = layout.canvas_rect();

    // Canvas background
    fb.fill_rect(cx0, cy0, cx1, cy1, COLOR_BG_CANVAS);

    let camera = shell.camera.clone();

    // Render surfaces (artboards) with bleed, margins, guides, and objects
    if let Some(session) = shell.bridge.session() {
        for surface in &session.document.surfaces {
            // Document bounds -> Screen bounds: s_bounds is [x, y, w, h]
            let s_bounds = surface.bounds();
            let sx_min = s_bounds[0];
            let sy_min = s_bounds[1];
            let sx_max = s_bounds[0] + s_bounds[2];
            let sy_max = s_bounds[1] + s_bounds[3];

            let p0_doc = GPoint::new(sx_min, sy_min);
            let p1_doc = GPoint::new(sx_max, sy_max);
            let p0_screen = camera.doc_to_screen(p0_doc);
            let p1_screen = camera.doc_to_screen(p1_doc);

            let sx0 = cx0 as f64 + p0_screen.x;
            let sy0 = cy0 as f64 + p0_screen.y;
            let sx1 = cx0 as f64 + p1_screen.x;
            let sy1 = cy0 as f64 + p1_screen.y;

            // Clip to viewport
            if sx1 < cx0 as f64 || sx0 > cx1 as f64 || sy1 < cy0 as f64 || sy0 > cy1 as f64 {
                continue;
            }

            let px0 = (sx0.max(cx0 as f64) as usize).min(cx1);
            let py0 = (sy0.max(cy0 as f64) as usize).min(cy1);
            let px1 = (sx1.min(cx1 as f64) as usize).max(px0);
            let py1 = (sy1.min(cy1 as f64) as usize).max(py0);

            // Drop shadow for artboard paper
            if px1 + 4 < cx1 && py1 + 4 < cy1 {
                fb.fill_rect(px0 + 4, py0 + 4, px1 + 4, py1 + 4, COLOR_PAPER_SHADOW);
            }

            // Artboard Paper White
            fb.fill_rect(px0, py0, px1, py1, COLOR_PAPER);
            fb.stroke_rect(px0, py0, px1, py1, 0xCCCCCC, 1);

            // Artboard Name header above artboard
            let surface_title = format!(
                "📄 {} ({}x{} pt)",
                surface.name, s_bounds[2] as i32, s_bounds[3] as i32
            );
            if py0 >= 16 + cy0 {
                draw_text(
                    &mut fb.pixels,
                    fb.width,
                    fb.height,
                    px0,
                    py0 - 14,
                    &surface_title,
                    COLOR_TEXT_SECONDARY,
                );
            }

            // Render bleed outline if set
            let bleed = surface.bleed;
            if !bleed.is_zero() {
                let b_p0_doc = GPoint::new(sx_min - bleed.left, sy_min - bleed.top);
                let b_p1_doc = GPoint::new(sx_max + bleed.right, sy_max + bleed.bottom);
                let b_p0 = camera.doc_to_screen(b_p0_doc);
                let b_p1 = camera.doc_to_screen(b_p1_doc);
                let bx0 = ((cx0 as f64 + b_p0.x).max(cx0 as f64) as usize).min(cx1);
                let by0 = ((cy0 as f64 + b_p0.y).max(cy0 as f64) as usize).min(cy1);
                let bx1 = ((cx0 as f64 + b_p1.x).min(cx1 as f64) as usize).max(bx0);
                let by1 = ((cy0 as f64 + b_p1.y).min(cy1 as f64) as usize).max(by0);
                fb.stroke_rect(bx0, by0, bx1, by1, COLOR_GUIDE_BLEED, 1);
            }

            // Render margin guide if set
            let margins = surface.margins;
            if margins != aubrieta_document::Margins::ZERO {
                let m_p0_doc = GPoint::new(sx_min + margins.left, sy_min + margins.top);
                let m_p1_doc = GPoint::new(sx_max - margins.right, sy_max - margins.bottom);
                let m_p0 = camera.doc_to_screen(m_p0_doc);
                let m_p1 = camera.doc_to_screen(m_p1_doc);
                let mx0 = ((cx0 as f64 + m_p0.x).max(cx0 as f64) as usize).min(cx1);
                let my0 = ((cy0 as f64 + m_p0.y).max(cy0 as f64) as usize).min(cy1);
                let mx1 = ((cx0 as f64 + m_p1.x).min(cx1 as f64) as usize).max(mx0);
                let my1 = ((cy0 as f64 + m_p1.y).min(cy1 as f64) as usize).max(my0);
                fb.stroke_rect(mx0, my0, mx1, my1, COLOR_GUIDE_MARGIN, 1);
            }

            // Render guides
            for guide in &surface.guides {
                match guide.orientation {
                    aubrieta_document::GuideOrientation::Vertical => {
                        let gx =
                            camera.doc_to_screen(GPoint::new(guide.position, 0.0)).x + cx0 as f64;
                        if gx >= cx0 as f64 && gx < cx1 as f64 {
                            fb.draw_line(
                                gx as i32,
                                py0 as i32,
                                gx as i32,
                                py1 as i32,
                                COLOR_GUIDE_MARGIN,
                            );
                        }
                    }
                    aubrieta_document::GuideOrientation::Horizontal => {
                        let gy =
                            camera.doc_to_screen(GPoint::new(0.0, guide.position)).y + cy0 as f64;
                        if gy >= cy0 as f64 && gy < cy1 as f64 {
                            fb.draw_line(
                                px0 as i32,
                                gy as i32,
                                px1 as i32,
                                gy as i32,
                                COLOR_GUIDE_MARGIN,
                            );
                        }
                    }
                }
            }

            // Render Objects on this surface
            for obj in &surface.objects {
                let fill_color = obj
                    .fill
                    .as_deref()
                    .map(parse_color_token)
                    .unwrap_or(0x3B82F6);

                let obj_bounds = obj
                    .bounds
                    .map(|b| GRect::new(b[0], b[1], b[0] + b[2], b[1] + b[3]))
                    .unwrap_or(GRect::new(0.0, 0.0, 100.0, 80.0));
                let o_p0 = camera.doc_to_screen(GPoint::new(obj_bounds.x0, obj_bounds.y0));
                let o_p1 = camera.doc_to_screen(GPoint::new(obj_bounds.x1, obj_bounds.y1));

                let ox0 = ((cx0 as f64 + o_p0.x).max(cx0 as f64) as usize).min(cx1);
                let oy0 = ((cy0 as f64 + o_p0.y).max(cy0 as f64) as usize).min(cy1);
                let ox1 = ((cx0 as f64 + o_p1.x).min(cx1 as f64) as usize).max(ox0);
                let oy1 = ((cy0 as f64 + o_p1.y).min(cy1 as f64) as usize).max(oy0);

                if ox1 > ox0 && oy1 > oy0 {
                    fb.fill_rect(ox0, oy0, ox1, oy1, fill_color);
                    let stroke_color = obj
                        .stroke
                        .as_deref()
                        .map(parse_color_token)
                        .unwrap_or(0x1E293B);
                    fb.stroke_rect(ox0, oy0, ox1, oy1, stroke_color, 1);

                    // Object name tag
                    let name_tag = format!("🔷 {}", obj.name);
                    if oy1 - oy0 > 16 {
                        draw_text(
                            &mut fb.pixels,
                            fb.width,
                            fb.height,
                            ox0 + 6,
                            oy0 + 6,
                            &name_tag,
                            0xFFFFFF,
                        );
                    }
                }
            }
        }
    }

    // Overlays (Selection handles, marquee, snapping guides, pen preview)
    let overlays: CanvasOverlays = shell.overlays();
    render_overlays(fb, cx0, cy0, cx1, cy1, &camera, &overlays);
}

fn render_overlays(
    fb: &mut FrameBuffer,
    cx0: usize,
    cy0: usize,
    cx1: usize,
    cy1: usize,
    camera: &aubrieta_ui_gpui::canvas::ViewportCamera,
    overlays: &CanvasOverlays,
) {
    // 1. Selection handles
    for handle in &overlays.handles {
        let hx = cx0 as f64 + handle.screen_hit_box.x0;
        let hy = cy0 as f64 + handle.screen_hit_box.y0;
        let hw = handle.screen_hit_box.width();
        let hh = handle.screen_hit_box.height();

        let px0 = (hx.max(cx0 as f64) as usize).min(cx1);
        let py0 = (hy.max(cy0 as f64) as usize).min(cy1);
        let px1 = ((hx + hw).min(cx1 as f64) as usize).max(px0);
        let py1 = ((hy + hh).min(cy1 as f64) as usize).max(py0);

        if px1 > px0 && py1 > py0 {
            fb.fill_rect(px0, py0, px1, py1, 0xFFFFFF);
            fb.stroke_rect(px0, py0, px1, py1, COLOR_ACCENT_PRIMARY, 1);
        }
    }

    // 2. Selection Marquee
    if let Some(marquee) = overlays.marquee_screen {
        let mx0 = ((cx0 as f64 + marquee.x0).max(cx0 as f64) as usize).min(cx1);
        let my0 = ((cy0 as f64 + marquee.y0).max(cy0 as f64) as usize).min(cy1);
        let mx1 = ((cx0 as f64 + marquee.x1).min(cx1 as f64) as usize).max(mx0);
        let my1 = ((cy0 as f64 + marquee.y1).min(cy1 as f64) as usize).max(my0);

        if mx1 > mx0 && my1 > my0 {
            fb.stroke_rect(mx0, my0, mx1, my1, COLOR_SELECTION_HANDLE, 1);
        }
    }

    // 3. Snap guides
    for guide in &overlays.snap_guides {
        match guide.orientation {
            SnapOrientation::Vertical => {
                let gx =
                    (cx0 as f64 + camera.doc_to_screen(GPoint::new(guide.position, 0.0)).x) as i32;
                let gy0 = (cy0 as f64 + camera.doc_to_screen(GPoint::new(0.0, guide.span_start)).y)
                    as i32;
                let gy1 =
                    (cy0 as f64 + camera.doc_to_screen(GPoint::new(0.0, guide.span_end)).y) as i32;
                fb.draw_line(gx, gy0, gx, gy1, COLOR_GUIDE_SNAP);
            }
            SnapOrientation::Horizontal => {
                let gy =
                    (cy0 as f64 + camera.doc_to_screen(GPoint::new(0.0, guide.position)).y) as i32;
                let gx0 = (cx0 as f64 + camera.doc_to_screen(GPoint::new(guide.span_start, 0.0)).x)
                    as i32;
                let gx1 =
                    (cx0 as f64 + camera.doc_to_screen(GPoint::new(guide.span_end, 0.0)).x) as i32;
                fb.draw_line(gx0, gy, gx1, gy, COLOR_GUIDE_SNAP);
            }
        }
    }
}

fn render_dock_panels(
    fb: &mut FrameBuffer,
    layout: &UiLayout,
    shell: &AubrietaShell,
    active_tab: ActiveDockTab,
) {
    let dx0 = layout.width.saturating_sub(layout.dock_w);
    let dy0 = layout.header_h + layout.context_h;
    let dx1 = layout.width;
    let dy1 = layout.height - layout.status_h;

    // Dock container background
    fb.fill_rect(dx0, dy0, dx1, dy1, COLOR_BG_DOCK);
    fb.stroke_rect(dx0, dy0, dx0 + 1, dy1, COLOR_BORDER, 1);

    // Tab switcher header
    let tab_h = 32;
    fb.fill_rect(dx0, dy0, dx1, dy0 + tab_h, COLOR_BG_PANEL_HEADER);
    fb.stroke_rect(dx0, dy0 + tab_h - 1, dx1, dy0 + tab_h, COLOR_BORDER, 1);

    let tabs = [
        (ActiveDockTab::Layers, "Layers"),
        (ActiveDockTab::Properties, "Props"),
        (ActiveDockTab::History, "History"),
        (ActiveDockTab::DataMerge, "Merge"),
    ];

    let tab_w = (layout.dock_w - 8) / 4;
    for (i, (tab, label)) in tabs.iter().enumerate() {
        let tx0 = dx0 + 4 + i * tab_w;
        let is_active = *tab == active_tab;
        fb.draw_button(tx0, dy0 + 4, tab_w - 2, 24, label, is_active, false);
    }

    // Tab content area
    let cy0 = dy0 + tab_h + 10;
    match active_tab {
        ActiveDockTab::Layers => render_layers_tab(fb, dx0 + 10, cy0, dx1 - 10, dy1, shell),
        ActiveDockTab::Properties => render_properties_tab(fb, dx0 + 10, cy0, dx1 - 10, dy1, shell),
        ActiveDockTab::History => render_history_tab(fb, dx0 + 10, cy0, dx1 - 10, dy1, shell),
        ActiveDockTab::DataMerge => render_data_merge_tab(fb, dx0 + 10, cy0, dx1 - 10, dy1, shell),
    }
}

fn render_layers_tab(
    fb: &mut FrameBuffer,
    x0: usize,
    mut y: usize,
    x1: usize,
    y_max: usize,
    shell: &AubrietaShell,
) {
    let model: LayersPresentationModel = shell.query_layers();

    draw_text(
        &mut fb.pixels,
        fb.width,
        fb.height,
        x0,
        y,
        "OBJECT HIERARCHY",
        COLOR_TEXT_SECONDARY,
    );
    y += 20;

    for surface in &model.surfaces {
        if y + 20 > y_max {
            break;
        }
        let surface_label = format!("📄 {}", surface.name);
        draw_text(
            &mut fb.pixels,
            fb.width,
            fb.height,
            x0,
            y,
            &surface_label,
            0x60A5FA,
        );
        y += 18;
    }

    for item in &model.rows {
        if y + 20 > y_max {
            break;
        }
        let vis = if item.visible { "👁" } else { " " };
        let lock = if item.locked { "🔒" } else { " " };
        let sel = if item.is_selected { "*" } else { " " };
        let indent = "  ".repeat(item.depth);
        let row = format!("{indent}{sel}[{vis}][{lock}] {}", item.name);

        let row_color = if item.is_selected {
            0xFFFFFF
        } else {
            COLOR_TEXT_PRIMARY
        };
        if item.is_selected {
            fb.fill_rect(x0, y - 2, x1, y + 14, 0x2A3E5C);
        }
        draw_text(&mut fb.pixels, fb.width, fb.height, x0, y, &row, row_color);
        y += 18;
    }
}

fn render_properties_tab(
    fb: &mut FrameBuffer,
    x0: usize,
    mut y: usize,
    _x1: usize,
    _y_max: usize,
    shell: &AubrietaShell,
) {
    let model: PropertiesPresentationModel = shell.query_properties();

    draw_text(
        &mut fb.pixels,
        fb.width,
        fb.height,
        x0,
        y,
        "PROPERTIES INSPECTOR",
        COLOR_TEXT_SECONDARY,
    );
    y += 20;

    let sel_text = if model.selection_empty {
        "No selection (Document)".to_string()
    } else {
        format!("Selected: {}", model.name.as_deref().unwrap_or("Object"))
    };
    draw_text(
        &mut fb.pixels,
        fb.width,
        fb.height,
        x0,
        y,
        &sel_text,
        COLOR_TEXT_PRIMARY,
    );
    y += 20;

    if let Some(bounds) = model.bounds {
        draw_text(
            &mut fb.pixels,
            fb.width,
            fb.height,
            x0,
            y,
            "Transform",
            COLOR_TEXT_SECONDARY,
        );
        y += 16;
        let t1 = format!(" X: {:<6.1}  Y: {:<6.1}", bounds[0], bounds[1]);
        let t2 = format!(" W: {:<6.1}  H: {:<6.1}", bounds[2], bounds[3]);
        draw_text(
            &mut fb.pixels,
            fb.width,
            fb.height,
            x0 + 4,
            y,
            &t1,
            COLOR_TEXT_PRIMARY,
        );
        y += 14;
        draw_text(
            &mut fb.pixels,
            fb.width,
            fb.height,
            x0 + 4,
            y,
            &t2,
            COLOR_TEXT_PRIMARY,
        );
        y += 24;
    }

    draw_text(
        &mut fb.pixels,
        fb.width,
        fb.height,
        x0,
        y,
        "Appearance Stack",
        COLOR_TEXT_SECONDARY,
    );
    y += 16;
    let fill_desc = model.fill.as_deref().unwrap_or("None / Default");
    let fill_text = format!(" Fill: {fill_desc}");
    draw_text(
        &mut fb.pixels,
        fb.width,
        fb.height,
        x0 + 4,
        y,
        &fill_text,
        0x34D399,
    );
    y += 16;

    let op_pct = (model.opacity * 100.0).round() as i32;
    let op_text = format!(" Opacity: {op_pct}%");
    draw_text(
        &mut fb.pixels,
        fb.width,
        fb.height,
        x0 + 4,
        y,
        &op_text,
        COLOR_TEXT_PRIMARY,
    );
}

fn render_history_tab(
    fb: &mut FrameBuffer,
    x0: usize,
    mut y: usize,
    _x1: usize,
    y_max: usize,
    shell: &AubrietaShell,
) {
    let model: HistoryPresentationModel = shell.bridge.query_history();

    draw_text(
        &mut fb.pixels,
        fb.width,
        fb.height,
        x0,
        y,
        "TRANSACTION HISTORY",
        COLOR_TEXT_SECONDARY,
    );
    y += 20;

    let h_info = format!(
        "Undo: {} | Redo: {}",
        model.undo_stack.len(),
        model.redo_stack.len()
    );
    draw_text(
        &mut fb.pixels,
        fb.width,
        fb.height,
        x0,
        y,
        &h_info,
        COLOR_TEXT_PRIMARY,
    );
    y += 20;

    for (i, entry) in model.undo_stack.iter().enumerate().rev() {
        if y + 16 > y_max {
            break;
        }
        let line = format!("  [{}] {}", i + 1, entry.description);
        draw_text(
            &mut fb.pixels,
            fb.width,
            fb.height,
            x0,
            y,
            &line,
            COLOR_TEXT_PRIMARY,
        );
        y += 16;
    }
}

fn render_data_merge_tab(
    fb: &mut FrameBuffer,
    x0: usize,
    mut y: usize,
    _x1: usize,
    _y_max: usize,
    shell: &AubrietaShell,
) {
    let model: DataMergePresentationModel = shell.query_data_merge();

    draw_text(
        &mut fb.pixels,
        fb.width,
        fb.height,
        x0,
        y,
        "VARIABLE DATA MERGE",
        COLOR_TEXT_SECONDARY,
    );
    y += 20;

    let src_name = model
        .sources
        .first()
        .map(|s| s.name.as_str())
        .unwrap_or("(No source loaded)");
    let src_label = format!("Source: {src_name}");
    draw_text(
        &mut fb.pixels,
        fb.width,
        fb.height,
        x0,
        y,
        &src_label,
        COLOR_TEXT_PRIMARY,
    );
    y += 18;

    let rec_label = format!(
        "Records: {} | Bindings: {}",
        model.total_records,
        model.bindings.len()
    );
    draw_text(
        &mut fb.pixels,
        fb.width,
        fb.height,
        x0,
        y,
        &rec_label,
        COLOR_TEXT_SECONDARY,
    );
    y += 24;

    let status = if model.total_records > 0 {
        "Status: Ready to Merge"
    } else {
        "Status: Awaiting Source"
    };
    draw_text(&mut fb.pixels, fb.width, fb.height, x0, y, status, 0x10B981);
}

fn render_status_bar(
    fb: &mut FrameBuffer,
    layout: &UiLayout,
    shell: &AubrietaShell,
    hover_coords: Option<(usize, usize)>,
) {
    let y0 = layout.height - layout.status_h;
    fb.fill_rect(0, y0, layout.width, layout.height, 0x18181C);
    fb.stroke_rect(0, y0, layout.width, y0 + 1, COLOR_BORDER, 1);

    // Left info
    let left_info = "Ready | Mode: Vector Layout";
    draw_text(
        &mut fb.pixels,
        fb.width,
        fb.height,
        12,
        y0 + 8,
        left_info,
        COLOR_TEXT_MUTED,
    );

    // Coordinates info
    let (cx0, cy0, _cx1, _cy1) = layout.canvas_rect();
    let coords_str = if let Some((mx, my)) = hover_coords {
        if layout.is_in_canvas(mx, my) {
            let screen_pt = GPoint::new((mx - cx0) as f64, (my - cy0) as f64);
            let doc_pt = shell.camera.screen_to_doc(screen_pt);
            format!("Doc: X: {:<5.1} pt  Y: {:<5.1} pt", doc_pt.x, doc_pt.y)
        } else {
            "Pointer outside canvas".to_string()
        }
    } else {
        "No cursor".to_string()
    };
    draw_text(
        &mut fb.pixels,
        fb.width,
        fb.height,
        340,
        y0 + 8,
        &coords_str,
        COLOR_TEXT_SECONDARY,
    );

    // Zoom and Document Status
    let zoom_pct = (shell.camera.zoom * 100.0).round() as i32;
    let right_info = format!("Zoom: {zoom_pct}% | One-Tree Hierarchy");
    let right_x = layout.width.saturating_sub(300);
    draw_text(
        &mut fb.pixels,
        fb.width,
        fb.height,
        right_x,
        y0 + 8,
        &right_info,
        COLOR_TEXT_MUTED,
    );
}

fn parse_color_token(token: &str) -> u32 {
    if token.starts_with('#') {
        let hex = token.trim_start_matches('#');
        if hex.len() == 6 {
            if let Ok(val) = u32::from_str_radix(hex, 16) {
                return val;
            }
        }
    }
    match token {
        "aubrieta.red/500" => 0xEF4444,
        "aubrieta.blue/500" => 0x3B82F6,
        "aubrieta.green/500" => 0x22C55E,
        "aubrieta.yellow/500" => 0xEAB308,
        "aubrieta.purple/500" => 0xA855F7,
        _ => 0x94A3B8,
    }
}
