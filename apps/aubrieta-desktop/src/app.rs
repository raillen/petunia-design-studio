//! Desktop application state and event dispatch coordinator (08.7, 09.24).
//!
//! Orchestrates the `AubrietaShell`, UI layout hit-testing, tool switching,
//! mouse gestures, keyboard shortcuts, and frame buffer presentation.

use aubrieta_application::{Command, CommandRequest};
use aubrieta_document::{Bleed, Guide, GuideOrientation, Margins};
use aubrieta_foundation::{AubrietaError, IdGenerator};
use aubrieta_geometry::{GPoint, GRect};
use aubrieta_ui_gpui::shell::AubrietaShell;
use aubrieta_ui_gpui::tools::{
    NormalizedPointerEvent, PointerButton, PointerPhase, SemanticModifiers, ToolKind,
};

use crate::ui_renderer::{render_desktop_ui, ActiveDockTab, FrameBuffer, UiLayout};

/// Primary desktop application container.
pub struct DesktopApp {
    /// Canonical UI shell coordinating presentation models and commands.
    pub shell: AubrietaShell,
    /// Currently active inspector dock tab.
    pub active_tab: ActiveDockTab,
    /// 32-bit pixel frame buffer.
    pub fb: FrameBuffer,
    /// Layout geometry calculations.
    pub layout: UiLayout,
    /// Current mouse position in window pixels.
    pub mouse_pos: Option<(usize, usize)>,
    /// Whether primary mouse button is currently held down.
    pub is_mouse_down: bool,
    /// Whether canvas pan gesture is active (e.g. Space+Drag).
    pub is_panning: bool,
    /// Last recorded mouse coordinate for pan deltas.
    pub last_mouse: Option<(f64, f64)>,
    /// Semantic Shift modifier state (constrain).
    pub shift_held: bool,
    /// Semantic Alt modifier state (duplicate / from-center).
    pub alt_held: bool,
    /// Semantic Ctrl modifier state (disable snapping).
    pub ctrl_held: bool,
}

impl DesktopApp {
    /// Creates a fresh desktop application instance.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        let mut shell = AubrietaShell::new(width as f64 - 328.0, height as f64 - 88.0);
        let fb = FrameBuffer::new(width, height);
        let layout = UiLayout::new(width, height);

        // Populate initial creative document
        let _ = populate_initial_document(&mut shell);

        Self {
            shell,
            active_tab: ActiveDockTab::Layers,
            fb,
            layout,
            mouse_pos: None,
            is_mouse_down: false,
            is_panning: false,
            last_mouse: None,
            shift_held: false,
            alt_held: false,
            ctrl_held: false,
        }
    }

    /// Resizes the desktop application window and buffers.
    pub fn resize(&mut self, width: usize, height: usize) {
        self.fb = FrameBuffer::new(width, height);
        self.layout = UiLayout::new(width, height);
        let (cx0, cy0, cx1, cy1) = self.layout.canvas_rect();
        self.shell
            .camera
            .resize((cx1 - cx0) as f64, (cy1 - cy0) as f64);
    }

    /// Handles mouse button press.
    pub fn on_mouse_down(&mut self, x: usize, y: usize, button: PointerButton) {
        self.mouse_pos = Some((x, y));
        self.last_mouse = Some((x as f64, y as f64));

        // 1. Tool Rail click (Left sidebar)
        if x < self.layout.rail_w && y >= self.layout.header_h + self.layout.context_h {
            let y_rel = y - (self.layout.header_h + self.layout.context_h);
            let tool_idx = (y_rel.saturating_sub(10)) / 44;
            let tools = [
                ToolKind::Select,
                ToolKind::Node,
                ToolKind::Pen,
                ToolKind::Rectangle,
                ToolKind::Ellipse,
                ToolKind::Polygon,
            ];
            if let Some(tool) = tools.get(tool_idx) {
                self.shell.set_active_tool(*tool);
            }
            return;
        }

        // 2. Context Toolbar click
        if y >= self.layout.header_h && y < self.layout.header_h + self.layout.context_h {
            // Undo button
            if (190..250).contains(&x) {
                let _ = self.shell.undo();
            }
            // Redo button
            else if (255..315).contains(&x) {
                let _ = self.shell.redo();
            }
            // Fit button
            else if (430..475).contains(&x) {
                let layers = self.shell.query_layers();
                if let Some(surface) = layers.surfaces.first() {
                    let rect = GRect::new(
                        surface.origin[0],
                        surface.origin[1],
                        surface.origin[0] + surface.dimensions[0],
                        surface.origin[1] + surface.dimensions[1],
                    );
                    self.shell.fit_surface(rect);
                }
            }
            // Zoom +
            else if (480..510).contains(&x) {
                let center = GPoint::new(
                    (self.layout.width / 2) as f64,
                    (self.layout.height / 2) as f64,
                );
                self.shell.zoom_at(center, 1.25);
            }
            // Zoom -
            else if (515..545).contains(&x) {
                let center = GPoint::new(
                    (self.layout.width / 2) as f64,
                    (self.layout.height / 2) as f64,
                );
                self.shell.zoom_at(center, 0.8);
            }
            // Snap toggle
            else if (560..640).contains(&x) {
                let enabled = self.shell.snap.config.grid_enabled;
                self.shell.snap.config.grid_enabled = !enabled;
                self.shell.snap.config.guides_enabled = !enabled;
                self.shell.snap.config.objects_enabled = !enabled;
            }
            return;
        }

        // 3. Right Dock Inspector click
        if x >= self.layout.width.saturating_sub(self.layout.dock_w) {
            let dx0 = self.layout.width.saturating_sub(self.layout.dock_w);
            let tab_y0 = self.layout.header_h + self.layout.context_h;
            if y >= tab_y0 && y < tab_y0 + 32 {
                let tab_w = (self.layout.dock_w - 8) / 4;
                let tab_idx = (x.saturating_sub(dx0 + 4)) / tab_w;
                match tab_idx {
                    0 => self.active_tab = ActiveDockTab::Layers,
                    1 => self.active_tab = ActiveDockTab::Properties,
                    2 => self.active_tab = ActiveDockTab::History,
                    3 => self.active_tab = ActiveDockTab::DataMerge,
                    _ => {}
                }
            }
            return;
        }

        // 4. Center Canvas Viewport
        if self.layout.is_in_canvas(x, y) {
            self.is_mouse_down = true;
            if self.is_panning || button == PointerButton::Middle {
                return;
            }
            let (cx0, cy0, _cx1, _cy1) = self.layout.canvas_rect();
            let screen_pt = GPoint::new((x - cx0) as f64, (y - cy0) as f64);
            let doc_pt = self.shell.camera.screen_to_doc(screen_pt);
            let modifiers = SemanticModifiers {
                constrain: self.shift_held,
                from_center: self.alt_held,
                duplicate: self.alt_held,
                disable_snap: self.ctrl_held,
                fine_adjust: false,
            };
            let event = NormalizedPointerEvent::new(
                PointerPhase::Down,
                button,
                screen_pt,
                doc_pt,
                modifiers,
            );
            let _ = self.shell.handle_pointer_event(&event);
        }
    }

    /// Handles mouse movement.
    pub fn on_mouse_move(&mut self, x: usize, y: usize) {
        self.mouse_pos = Some((x, y));

        if self.is_panning {
            if let Some((lx, ly)) = self.last_mouse {
                let dx = x as f64 - lx;
                let dy = y as f64 - ly;
                self.shell.pan(dx, dy);
            }
            self.last_mouse = Some((x as f64, y as f64));
            return;
        }

        if self.is_mouse_down && self.layout.is_in_canvas(x, y) {
            let (cx0, cy0, _cx1, _cy1) = self.layout.canvas_rect();
            let screen_pt = GPoint::new((x - cx0) as f64, (y - cy0) as f64);
            let doc_pt = self.shell.camera.screen_to_doc(screen_pt);
            let modifiers = SemanticModifiers {
                constrain: self.shift_held,
                from_center: self.alt_held,
                duplicate: self.alt_held,
                disable_snap: self.ctrl_held,
                fine_adjust: false,
            };
            let event = NormalizedPointerEvent::new(
                PointerPhase::Move,
                PointerButton::Primary,
                screen_pt,
                doc_pt,
                modifiers,
            );
            let _ = self.shell.handle_pointer_event(&event);
        }
        self.last_mouse = Some((x as f64, y as f64));
    }

    /// Handles mouse button release.
    pub fn on_mouse_up(&mut self, x: usize, y: usize, button: PointerButton) {
        self.mouse_pos = Some((x, y));
        self.is_mouse_down = false;

        if self.layout.is_in_canvas(x, y) {
            let (cx0, cy0, _cx1, _cy1) = self.layout.canvas_rect();
            let screen_pt = GPoint::new((x - cx0) as f64, (y - cy0) as f64);
            let doc_pt = self.shell.camera.screen_to_doc(screen_pt);
            let modifiers = SemanticModifiers {
                constrain: self.shift_held,
                from_center: self.alt_held,
                duplicate: self.alt_held,
                disable_snap: self.ctrl_held,
                fine_adjust: false,
            };
            let event =
                NormalizedPointerEvent::new(PointerPhase::Up, button, screen_pt, doc_pt, modifiers);
            let _ = self.shell.handle_pointer_event(&event);
        }
    }

    /// Handles mouse scroll wheel zoom.
    pub fn on_scroll(&mut self, delta_y: f64) {
        if let Some((mx, my)) = self.mouse_pos {
            let (cx0, cy0, _cx1, _cy1) = self.layout.canvas_rect();
            let screen_pt = GPoint::new(
                (mx.saturating_sub(cx0)) as f64,
                (my.saturating_sub(cy0)) as f64,
            );
            let factor = if delta_y > 0.0 { 1.15 } else { 0.87 };
            self.shell.zoom_at(screen_pt, factor);
        }
    }

    /// Renders the complete frame buffer.
    pub fn render(&mut self) {
        render_desktop_ui(
            &mut self.fb,
            &mut self.shell,
            self.active_tab,
            self.mouse_pos,
        );
    }

    /// End-to-end headless smoke test exercising all interactive shell subsystems.
    pub fn smoke_test(&mut self) -> Result<(), String> {
        // 1. Validate document structure
        let snap = self.shell.snapshot();
        if snap.surface_count == 0 {
            return Err("Expected initial document to have surfaces".to_string());
        }

        // 2. Validate tool switching
        for tool in [
            ToolKind::Select,
            ToolKind::Node,
            ToolKind::Pen,
            ToolKind::Rectangle,
            ToolKind::Ellipse,
            ToolKind::Polygon,
        ] {
            self.shell.set_active_tool(tool);
            if self.shell.active_tool() != tool {
                return Err(format!("Tool switch failed for {tool:?}"));
            }
        }

        // 3. Simulate interactive rectangle creation
        self.shell.set_active_tool(ToolKind::Rectangle);
        let p_start = GPoint::new(100.0, 100.0);
        let p_end = GPoint::new(250.0, 200.0);
        let doc_start = self.shell.camera.screen_to_doc(p_start);
        let doc_end = self.shell.camera.screen_to_doc(p_end);

        let evt_down = NormalizedPointerEvent::new(
            PointerPhase::Down,
            PointerButton::Primary,
            p_start,
            doc_start,
            SemanticModifiers::default(),
        );
        self.shell
            .handle_pointer_event(&evt_down)
            .map_err(|e| format!("rect down: {e}"))?;

        let evt_move = NormalizedPointerEvent::new(
            PointerPhase::Move,
            PointerButton::Primary,
            p_end,
            doc_end,
            SemanticModifiers::default(),
        );
        self.shell
            .handle_pointer_event(&evt_move)
            .map_err(|e| format!("rect move: {e}"))?;

        let evt_up = NormalizedPointerEvent::new(
            PointerPhase::Up,
            PointerButton::Primary,
            p_end,
            doc_end,
            SemanticModifiers::default(),
        );
        self.shell
            .handle_pointer_event(&evt_up)
            .map_err(|e| format!("rect up: {e}"))?;

        // 4. Verify undo / redo
        self.shell.undo().map_err(|e| format!("undo: {e}"))?;
        self.shell.redo().map_err(|e| format!("redo: {e}"))?;

        // 5. Verify presentation models
        let layers = self.shell.query_layers();
        if layers.surfaces.is_empty() {
            return Err("Layers model empty".to_string());
        }

        let props = self.shell.query_properties();
        let _history = self.shell.bridge.query_history();
        let _merge = self.shell.query_data_merge();

        // 6. Verify frame rendering produces valid visual output
        self.render();
        if self.fb.pixels.is_empty() || self.fb.pixels.len() != self.fb.width * self.fb.height {
            return Err("Frame buffer pixel length mismatch".to_string());
        }

        println!(
            "OK DesktopApp Smoke Test: surfaces={} layers={} props_name={:?} zoom={:.1}%",
            snap.surface_count,
            layers.rows.len(),
            props.name,
            self.shell.camera.zoom * 100.0
        );

        Ok(())
    }
}

/// Populates an expressive starter document displaying artboards, bleed, appearance stacks, and shapes.
fn populate_initial_document(shell: &mut AubrietaShell) -> Result<(), AubrietaError> {
    shell.new_document("Aubrieta Showcase Project")?;

    let mut id_gen = IdGenerator::new();
    let surface_1 = id_gen.next_surface();
    let surface_2 = id_gen.next_surface();
    let rect_id = id_gen.next_object();
    let circle_id = id_gen.next_object();

    // 1. Surface 1: Primary Artboard (800 x 600) with Bleed, Margins and Guides
    shell
        .bridge
        .submit_command(CommandRequest::new(Command::CreateSurface {
            id: surface_1,
            name: "Main Artboard".to_string(),
        }))?;

    shell
        .bridge
        .set_surface_geometry(surface_1, [40.0, 40.0], [800.0, 600.0])?;

    shell
        .bridge
        .set_surface_bleed(surface_1, Bleed::uniform(10.0))?;

    shell
        .bridge
        .set_surface_margins(surface_1, Margins::uniform(36.0))?;

    shell
        .bridge
        .add_surface_guide(surface_1, Guide::new(1, GuideOrientation::Vertical, 200.0))?;

    // 2. Hero Card Object with Appearance Stack (Solid fill and stroke)
    shell
        .bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_1,
            id: rect_id,
            name: "Hero Card".to_string(),
        }))?;

    shell
        .bridge
        .set_bounds(rect_id, Some([60.0, 60.0, 300.0, 180.0]), 0.0)?;
    shell
        .bridge
        .set_fill(rect_id, Some("aubrieta.blue/500".to_string()))?;
    shell
        .bridge
        .set_stroke(rect_id, Some("#ffffff".to_string()), 2.0)?;

    let app_stack = aubrieta_document::AppearanceStack::new()
        .with_fill("aubrieta.blue/500")
        .with_stroke("#ffffff", 2.0);
    shell.bridge.set_appearance(rect_id, Some(app_stack))?;

    // 3. Accent Circle Object
    shell
        .bridge
        .submit_command(CommandRequest::new(Command::CreateObject {
            surface: surface_1,
            id: circle_id,
            name: "Accent Circle".to_string(),
        }))?;

    shell
        .bridge
        .set_bounds(circle_id, Some([400.0, 120.0, 140.0, 140.0]), 0.0)?;
    shell
        .bridge
        .set_fill(circle_id, Some("aubrieta.yellow/500".to_string()))?;

    // 4. Surface 2: Mobile Story Artboard (360 x 640)
    shell
        .bridge
        .submit_command(CommandRequest::new(Command::CreateSurface {
            id: surface_2,
            name: "Mobile Story 9:16".to_string(),
        }))?;

    shell
        .bridge
        .set_surface_geometry(surface_2, [900.0, 40.0], [360.0, 640.0])?;

    // Select Hero Card initially
    shell.bridge.set_selection(vec![rect_id]);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_app_smoke_test_headless() {
        let mut app = DesktopApp::new(1280, 800);
        assert!(app.smoke_test().is_ok());
    }

    #[test]
    fn desktop_app_layout_and_resizing() {
        let mut app = DesktopApp::new(1280, 800);
        app.resize(1920, 1080);
        assert_eq!(app.fb.width, 1920);
        assert_eq!(app.fb.height, 1080);
        assert_eq!(app.layout.width, 1920);
        assert_eq!(app.layout.height, 1080);
        app.render();
        assert_eq!(app.fb.pixels.len(), 1920 * 1080);
    }
}
