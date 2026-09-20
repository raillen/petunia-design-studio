//! Aubrieta Creative Studio — Iced Prototype.
//!
//! Evaluates Iced Elm architecture and Canvas graphics, consuming
//! AubrietaGuiBridge presentation models and dispatching commands.

use std::env;

use aubrieta_application::{Command, CommandRequest};
use aubrieta_document::{Bleed, Guide, GuideOrientation, Margins};
use aubrieta_foundation::{AubrietaError, IdGenerator};
use aubrieta_geometry::{GPoint, GRect};
use aubrieta_ui_gpui::bridge::{
    DataMergePresentationModel, HistoryPresentationModel, LayersPresentationModel,
    PropertiesPresentationModel,
};
use aubrieta_ui_gpui::shell::AubrietaShell;
use aubrieta_ui_gpui::tools::{
    NormalizedPointerEvent, PointerButton, PointerPhase, SemanticModifiers, ToolKind,
};

use iced::mouse;
use iced::widget::canvas::{self, Action, Canvas, Event, Frame, Geometry, Path, Stroke, Text};
use iced::widget::{button, checkbox, column, container, row, slider, text, Column};
use iced::{Alignment, Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Layers,
    Properties,
    History,
    DataMerge,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectTool(ToolKind),
    Undo,
    Redo,
    ZoomIn,
    ZoomOut,
    FitCanvas,
    NewDocument,
    ToggleSnap(bool),
    SelectTab(ActiveTab),
    SetOpacity(f32),
    CanvasPointerDown(f32, f32),
    CanvasPointerMoved(f32, f32),
    CanvasPointerUp(f32, f32),
}

pub struct AubrietaIcedApp {
    pub shell: AubrietaShell,
    pub active_tab: ActiveTab,
    pub hovered_doc_point: Option<GPoint>,
    pub selected_opacity: f32,
}

impl AubrietaIcedApp {
    pub fn new() -> Self {
        let mut shell = AubrietaShell::new(950.0, 700.0);
        let _ = populate_showcase_document(&mut shell);
        Self {
            shell,
            active_tab: ActiveTab::Layers,
            hovered_doc_point: None,
            selected_opacity: 100.0,
        }
    }

    pub fn smoke_test(&mut self) -> Result<(), String> {
        let snap = self.shell.snapshot();
        if snap.surface_count == 0 {
            return Err("Document missing surfaces".to_string());
        }
        for tool in [
            ToolKind::Select,
            ToolKind::Node,
            ToolKind::Pen,
            ToolKind::Rectangle,
            ToolKind::Ellipse,
            ToolKind::Polygon,
        ] {
            self.shell.set_active_tool(tool);
        }
        self.shell.undo().map_err(|e| format!("undo: {e}"))?;
        self.shell.redo().map_err(|e| format!("redo: {e}"))?;
        let _ = self.shell.query_layers();
        let _ = self.shell.query_properties();
        let _ = self.shell.bridge.query_history();
        let _ = self.shell.query_data_merge();
        println!(
            "OK aubrieta-iced smoke test: surfaces={} title=\"{}\"",
            snap.surface_count, snap.title
        );
        Ok(())
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::SelectTool(tool) => {
                self.shell.set_active_tool(tool);
            }
            Message::Undo => {
                let _ = self.shell.undo();
            }
            Message::Redo => {
                let _ = self.shell.redo();
            }
            Message::ZoomIn => {
                let center = GPoint::new(400.0, 300.0);
                self.shell.zoom_at(center, 1.2);
            }
            Message::ZoomOut => {
                let center = GPoint::new(400.0, 300.0);
                self.shell.zoom_at(center, 0.8);
            }
            Message::FitCanvas => {
                if let Some(surface) = self.shell.bridge.session().and_then(|s| s.document.surfaces.first()) {
                    let b = surface.bounds();
                    self.shell.fit_surface(GRect::new(b[0], b[1], b[0] + b[2], b[1] + b[3]));
                }
            }
            Message::NewDocument => {
                let _ = self.shell.new_document("Untitled");
            }
            Message::ToggleSnap(enabled) => {
                self.shell.snap.config.grid_enabled = enabled;
                self.shell.snap.config.guides_enabled = enabled;
            }
            Message::SelectTab(tab) => {
                self.active_tab = tab;
            }
            Message::SetOpacity(val) => {
                self.selected_opacity = val;
            }
            Message::CanvasPointerDown(x, y) => {
                let screen_pt = GPoint::new(x as f64, y as f64);
                let doc_pt = self.shell.camera.screen_to_doc(screen_pt);
                self.hovered_doc_point = Some(doc_pt);
                let evt = NormalizedPointerEvent::new(
                    PointerPhase::Down,
                    PointerButton::Primary,
                    screen_pt,
                    doc_pt,
                    SemanticModifiers::default(),
                );
                let _ = self.shell.handle_pointer_event(&evt);
            }
            Message::CanvasPointerMoved(x, y) => {
                let screen_pt = GPoint::new(x as f64, y as f64);
                let doc_pt = self.shell.camera.screen_to_doc(screen_pt);
                self.hovered_doc_point = Some(doc_pt);
                let evt = NormalizedPointerEvent::new(
                    PointerPhase::Move,
                    PointerButton::Primary,
                    screen_pt,
                    doc_pt,
                    SemanticModifiers::default(),
                );
                let _ = self.shell.handle_pointer_event(&evt);
            }
            Message::CanvasPointerUp(x, y) => {
                let screen_pt = GPoint::new(x as f64, y as f64);
                let doc_pt = self.shell.camera.screen_to_doc(screen_pt);
                let evt = NormalizedPointerEvent::new(
                    PointerPhase::Up,
                    PointerButton::Primary,
                    screen_pt,
                    doc_pt,
                    SemanticModifiers::default(),
                );
                let _ = self.shell.handle_pointer_event(&evt);
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        // 1. Top Panel: Header & Context Toolbar
        let header_row = row![
            text("🎨 Aubrieta Design Studio [iced]")
                .size(15)
                .color(Color::from_rgb8(96, 165, 250)),
            button("New Document").on_press(Message::NewDocument),
            button("Fit Canvas").on_press(Message::FitCanvas),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let zoom_pct = (self.shell.camera.zoom * 100.0).round() as i32;
        let snap_enabled = self.shell.snap.config.grid_enabled;

        let context_row = row![
            text(format!("Tool: {:?}", self.shell.active_tool()))
                .size(13)
                .color(Color::from_rgb8(147, 197, 253)),
            button("Undo").on_press(Message::Undo),
            button("Redo").on_press(Message::Redo),
            text(format!("Zoom: {zoom_pct}%")).size(12),
            row![
                checkbox(snap_enabled).on_toggle(Message::ToggleSnap),
                text("Magnetic Snap").size(12),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let top_panel = container(
            column![header_row, context_row]
                .spacing(6)
                .padding(8),
        )
        .style(|_| container::Style {
            background: Some(Color::from_rgb8(39, 39, 42).into()),
            border: iced::Border {
                color: Color::from_rgb8(63, 63, 70),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .width(Length::Fill);

        // 2. Left Tool Rail
        let tools = [
            (ToolKind::Select, "V\nSel"),
            (ToolKind::Node, "A\nNod"),
            (ToolKind::Pen, "P\nPen"),
            (ToolKind::Rectangle, "M\nRec"),
            (ToolKind::Ellipse, "E\nEll"),
            (ToolKind::Polygon, "G\nPol"),
        ];

        let mut tool_col = Column::new().spacing(4).padding(4).width(Length::Fixed(56.0));
        for (tool, label) in tools {
            let btn = button(text(label).size(12).align_x(Alignment::Center))
                .width(Length::Fill)
                .on_press(Message::SelectTool(tool));
            tool_col = tool_col.push(btn);
        }

        let left_rail = container(tool_col)
            .style(|_| container::Style {
                background: Some(Color::from_rgb8(32, 32, 35).into()),
                border: iced::Border {
                    color: Color::from_rgb8(63, 63, 70),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .height(Length::Fill);

        // 3. Central Canvas
        let canvas_widget = Canvas::new(AubrietaCanvasProgram {
            shell: &self.shell,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        // 4. Right Dock Inspector
        let tab_buttons = row![
            button("Layers").on_press(Message::SelectTab(ActiveTab::Layers)),
            button("Props").on_press(Message::SelectTab(ActiveTab::Properties)),
            button("History").on_press(Message::SelectTab(ActiveTab::History)),
            button("Merge").on_press(Message::SelectTab(ActiveTab::DataMerge)),
        ]
        .spacing(4);

        let dock_content: Element<'_, Message> = match self.active_tab {
            ActiveTab::Layers => {
                let model: LayersPresentationModel = self.shell.query_layers();
                let mut col = column![
                    text("Object Hierarchy").size(14).color(Color::from_rgb8(96, 165, 250)),
                ]
                .spacing(6);

                for surface in &model.surfaces {
                    col = col.push(text(format!("📄 {}", surface.name)).color(Color::from_rgb8(147, 197, 253)));
                }

                for item in &model.rows {
                    let vis = if item.visible { "👁" } else { " " };
                    let lock = if item.locked { "🔒" } else { " " };
                    let sel = if item.is_selected { "●" } else { " " };
                    col = col.push(text(format!("{sel} [{vis}][{lock}] {}", item.name)).size(13));
                }
                col.into()
            }
            ActiveTab::Properties => {
                let model: PropertiesPresentationModel = self.shell.query_properties();
                let mut col = column![
                    text("Properties Inspector").size(14).color(Color::from_rgb8(96, 165, 250)),
                ]
                .spacing(6);

                if model.selection_empty {
                    col = col.push(text("No selection (Document root)").size(12));
                } else {
                    col = col.push(text(format!("Object: {}", model.name.as_deref().unwrap_or("Item"))).size(13));
                    if let Some(b) = model.bounds {
                        col = col.push(text(format!("X: {:.1} pt   Y: {:.1} pt", b[0], b[1])).size(12));
                        col = col.push(text(format!("W: {:.1} pt   H: {:.1} pt", b[2], b[3])).size(12));
                    }
                    if let Some(f) = &model.fill {
                        col = col.push(text(format!("Fill: {f}")).size(12));
                    }
                    col = col.push(text(format!("Opacity: {:.0}%", self.selected_opacity)).size(12));
                    col = col.push(slider(0.0..=100.0, self.selected_opacity, Message::SetOpacity));
                }
                col.into()
            }
            ActiveTab::History => {
                let model: HistoryPresentationModel = self.shell.bridge.query_history();
                let mut col = column![
                    text("Transaction History").size(14).color(Color::from_rgb8(96, 165, 250)),
                    text(format!("Undo: {} | Redo: {}", model.undo_stack.len(), model.redo_stack.len())).size(12),
                ]
                .spacing(6);

                for (i, item) in model.undo_stack.iter().enumerate().rev() {
                    col = col.push(text(format!("[{}] {}", i + 1, item.description)).size(12));
                }
                col.into()
            }
            ActiveTab::DataMerge => {
                let model: DataMergePresentationModel = self.shell.query_data_merge();
                let src = model.sources.first().map(|s| s.name.as_str()).unwrap_or("(No source)");
                column![
                    text("Variable Data Merge").size(14).color(Color::from_rgb8(96, 165, 250)),
                    text(format!("Source: {src}")).size(13),
                    text(format!("Records: {} | Bindings: {}", model.total_records, model.bindings.len())).size(12),
                    text("Status: Ready to Merge").color(Color::from_rgb8(34, 197, 94)).size(13),
                ]
                .spacing(6)
                .into()
            }
        };

        let right_dock = container(
            column![tab_buttons, dock_content]
                .spacing(8)
                .padding(8)
                .width(Length::Fixed(280.0)),
        )
        .style(|_| container::Style {
            background: Some(Color::from_rgb8(32, 32, 35).into()),
            border: iced::Border {
                color: Color::from_rgb8(63, 63, 70),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .height(Length::Fill);

        let middle_row = row![left_rail, canvas_widget, right_dock]
            .spacing(0)
            .height(Length::Fill);

        // 5. Bottom Status Bar
        let coords_text = if let Some(pt) = self.hovered_doc_point {
            format!("Doc: X: {:.1} pt  Y: {:.1} pt", pt.x, pt.y)
        } else {
            "Cursor outside canvas".to_string()
        };

        let status_bar = container(
            row![
                text(format!("Active Tool: {:?}", self.shell.active_tool())).size(12),
                text(coords_text).size(12),
                text(format!("Zoom: {zoom_pct}% | One-Tree Hierarchy")).size(12),
            ]
            .spacing(20)
            .padding(6)
            .align_y(Alignment::Center),
        )
        .style(|_| container::Style {
            background: Some(Color::from_rgb8(28, 28, 31).into()),
            border: iced::Border {
                color: Color::from_rgb8(63, 63, 70),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .width(Length::Fill);

        column![top_panel, middle_row, status_bar]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

struct AubrietaCanvasProgram<'a> {
    shell: &'a AubrietaShell,
}

impl<'a> canvas::Program<Message> for AubrietaCanvasProgram<'a> {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    return Some(Action::publish(Message::CanvasPointerDown(pos.x, pos.y)));
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    return Some(Action::publish(Message::CanvasPointerMoved(pos.x, pos.y)));
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    return Some(Action::publish(Message::CanvasPointerUp(pos.x, pos.y)));
                }
            }
            _ => {}
        }
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Background canvas dark color
        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            Color::from_rgb8(20, 20, 22),
        );

        let camera = &self.shell.camera;
        let to_screen = |doc_pt: GPoint| -> Point {
            let s = camera.doc_to_screen(doc_pt);
            Point::new(s.x as f32, s.y as f32)
        };

        if let Some(session) = self.shell.bridge.session() {
            for surface in &session.document.surfaces {
                let b = surface.bounds();
                let p0 = to_screen(GPoint::new(b[0], b[1]));
                let w = (b[2] * camera.zoom) as f32;
                let h = (b[3] * camera.zoom) as f32;
                let size = Size::new(w, h);

                // Artboard drop shadow
                frame.fill_rectangle(
                    Point::new(p0.x + 4.0, p0.y + 4.0),
                    size,
                    Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                );

                // Artboard Paper White
                frame.fill_rectangle(p0, size, Color::WHITE);
                let paper_path = Path::rectangle(p0, size);
                frame.stroke(
                    &paper_path,
                    Stroke::default().with_color(Color::from_rgb8(203, 213, 225)).with_width(1.0),
                );

                // Header text
                frame.fill_text(Text {
                    content: format!("📄 {} ({:.0}x{:.0} pt)", surface.name, b[2], b[3]),
                    position: Point::new(p0.x, p0.y - 14.0),
                    color: Color::from_rgb8(148, 163, 184),
                    size: 12.0.into(),
                    ..Default::default()
                });

                // Bleed Guideline (Magenta)
                if !surface.bleed.is_zero() {
                    let bp0 = to_screen(GPoint::new(b[0] - surface.bleed.left, b[1] - surface.bleed.top));
                    let bw = ((b[2] + surface.bleed.left + surface.bleed.right) * camera.zoom) as f32;
                    let bh = ((b[3] + surface.bleed.top + surface.bleed.bottom) * camera.zoom) as f32;
                    let bleed_path = Path::rectangle(bp0, Size::new(bw, bh));
                    frame.stroke(
                        &bleed_path,
                        Stroke::default().with_color(Color::from_rgb8(244, 63, 94)).with_width(1.0),
                    );
                }

                // Margin Guideline (Cyan)
                if surface.margins != Margins::ZERO {
                    let mp0 = to_screen(GPoint::new(b[0] + surface.margins.left, b[1] + surface.margins.top));
                    let mw = ((b[2] - surface.margins.left - surface.margins.right) * camera.zoom) as f32;
                    let mh = ((b[3] - surface.margins.top - surface.margins.bottom) * camera.zoom) as f32;
                    let margin_path = Path::rectangle(mp0, Size::new(mw, mh));
                    frame.stroke(
                        &margin_path,
                        Stroke::default().with_color(Color::from_rgb8(56, 189, 248)).with_width(1.0),
                    );
                }

                // Vertical Guide
                let gp0 = to_screen(GPoint::new(b[0] + 200.0, b[1]));
                let gp1 = to_screen(GPoint::new(b[0] + 200.0, b[1] + b[3]));
                let guide_path = Path::line(gp0, gp1);
                frame.stroke(
                    &guide_path,
                    Stroke::default().with_color(Color::from_rgb8(6, 182, 212)).with_width(1.0),
                );

                // Objects
                for obj in &surface.objects {
                    let obj_b = obj.bounds.unwrap_or([0.0, 0.0, 100.0, 80.0]);
                    let op0 = to_screen(GPoint::new(obj_b[0], obj_b[1]));
                    let ow = (obj_b[2] * camera.zoom) as f32;
                    let oh = (obj_b[3] * camera.zoom) as f32;
                    let osize = Size::new(ow, oh);

                    if obj.name.contains("Hero") {
                        let rect_path = Path::rectangle(op0, osize);
                        frame.fill(&rect_path, Color::from_rgb8(59, 130, 246));
                        frame.stroke(
                            &rect_path,
                            Stroke::default().with_color(Color::WHITE).with_width(2.0),
                        );
                        frame.fill_text(Text {
                            content: obj.name.clone(),
                            position: Point::new(op0.x + ow * 0.5 - 32.0, op0.y + oh * 0.5 - 7.0),
                            color: Color::WHITE,
                            size: 14.0.into(),
                            ..Default::default()
                        });
                    } else {
                        let center = Point::new(op0.x + ow * 0.5, op0.y + oh * 0.5);
                        let circle_path = Path::circle(center, ow * 0.5);
                        frame.fill(&circle_path, Color::from_rgb8(234, 179, 8));
                        frame.stroke(
                            &circle_path,
                            Stroke::default().with_color(Color::WHITE).with_width(2.0),
                        );
                        frame.fill_text(Text {
                            content: obj.name.clone(),
                            position: Point::new(op0.x + ow * 0.5 - 40.0, op0.y + oh * 0.5 - 7.0),
                            color: Color::WHITE,
                            size: 13.0.into(),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        vec![frame.into_geometry()]
    }
}

fn populate_showcase_document(shell: &mut AubrietaShell) -> Result<(), AubrietaError> {
    shell.new_document("Aubrieta Showcase Project [iced]")?;
    let mut id_gen = IdGenerator::new();
    let surface_1 = id_gen.next_surface();
    let rect_id = id_gen.next_object();
    let circle_id = id_gen.next_object();

    shell.bridge.submit_command(CommandRequest::new(Command::CreateSurface {
        id: surface_1,
        name: "Main Artboard".to_string(),
    }))?;
    shell.bridge.set_surface_geometry(surface_1, [40.0, 40.0], [800.0, 600.0])?;
    shell.bridge.set_surface_bleed(surface_1, Bleed::uniform(10.0))?;
    shell.bridge.set_surface_margins(surface_1, Margins::uniform(36.0))?;
    shell.bridge.add_surface_guide(surface_1, Guide::new(1, GuideOrientation::Vertical, 200.0))?;

    shell.bridge.submit_command(CommandRequest::new(Command::CreateObject {
        surface: surface_1,
        id: rect_id,
        name: "Hero Card".to_string(),
    }))?;
    shell.bridge.set_bounds(rect_id, Some([60.0, 60.0, 300.0, 180.0]), 0.0)?;
    shell.bridge.set_fill(rect_id, Some("aubrieta.blue/500".to_string()))?;

    shell.bridge.submit_command(CommandRequest::new(Command::CreateObject {
        surface: surface_1,
        id: circle_id,
        name: "Accent Circle".to_string(),
    }))?;
    shell.bridge.set_bounds(circle_id, Some([400.0, 120.0, 140.0, 140.0]), 0.0)?;
    shell.bridge.set_fill(circle_id, Some("aubrieta.yellow/500".to_string()))?;

    shell.bridge.set_selection(vec![rect_id]);
    Ok(())
}

fn main() -> iced::Result {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--smoke-test" || a == "--headless") {
        println!("aubrieta-iced: Running automated smoke test...");
        let mut app = AubrietaIcedApp::new();
        if let Err(e) = app.smoke_test() {
            eprintln!("aubrieta-iced smoke test failed: {e}");
            std::process::exit(1);
        }
        println!("aubrieta-iced: Smoke test PASSED.");
        return Ok(());
    }

    iced::application(AubrietaIcedApp::new, AubrietaIcedApp::update, AubrietaIcedApp::view)
        .title("Aubrieta Design Studio — Iced Prototype")
        .theme(app_theme)
        .window_size(Size::new(1280.0, 800.0))
        .run()
}

fn app_theme(_: &AubrietaIcedApp) -> Theme {
    Theme::Dark
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iced_app_smoke_test_headless() {
        let mut app = AubrietaIcedApp::new();
        assert!(app.smoke_test().is_ok());
    }
}
