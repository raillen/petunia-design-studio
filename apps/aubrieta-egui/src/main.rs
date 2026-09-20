//! Aubrieta Creative Studio — egui / eframe Secondary Experimental GUI.
//!
//! Evaluates egui immediate-mode GUI with OpenGL/Glow canvas integration,
//! consuming AubrietaGuiBridge presentation models and dispatching interactive commands.

use aubrieta_application::{Command, CommandRequest};
use aubrieta_document::{Bleed, Guide, GuideOrientation, Margins};
use aubrieta_foundation::{AubrietaError, IdGenerator, ObjectId};
use aubrieta_geometry::{GPoint, GRect};
use aubrieta_ui_gpui::bridge::{
    DataMergePresentationModel, HistoryPresentationModel, LayersPresentationModel,
};
use aubrieta_ui_gpui::shell::AubrietaShell;
use aubrieta_ui_gpui::tools::{
    NormalizedPointerEvent, PointerButton, PointerPhase, SemanticModifiers, ToolKind,
};
use eframe::egui;
use std::env;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActiveTab {
    Layers,
    Properties,
    History,
    DataMerge,
}

struct AubrietaEguiApp {
    shell: AubrietaShell,
    active_tab: ActiveTab,
    hovered_doc_point: Option<GPoint>,
    drag_start_doc: Option<GPoint>,
    dragging_object_id: Option<ObjectId>,
    drag_initial_bounds: Option<[f64; 4]>,
}

impl AubrietaEguiApp {
    fn new() -> Self {
        let mut shell = AubrietaShell::new(950.0, 700.0);
        let _ = populate_showcase_document(&mut shell);
        Self {
            shell,
            active_tab: ActiveTab::Properties,
            hovered_doc_point: None,
            drag_start_doc: None,
            dragging_object_id: None,
            drag_initial_bounds: None,
        }
    }

    fn smoke_test(&mut self) -> Result<(), String> {
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
            "OK aubrieta-egui smoke test: surfaces={} title=\"{}\"",
            snap.surface_count, snap.title
        );
        Ok(())
    }
}

impl eframe::App for AubrietaEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle global keyboard shortcuts
        ctx.input(|i| {
            if i.modifiers.ctrl || i.modifiers.command {
                if i.key_pressed(egui::Key::Z) {
                    let _ = self.shell.undo();
                } else if i.key_pressed(egui::Key::Y) {
                    let _ = self.shell.redo();
                }
            } else {
                if i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace) {
                    let sel = self.shell.bridge.selection();
                    for id in sel.selected_ids {
                        let _ = self.shell.bridge.submit_command(CommandRequest::new(Command::DeleteObject { id }));
                    }
                    self.shell.bridge.clear_selection();
                } else if i.key_pressed(egui::Key::V) {
                    self.shell.set_active_tool(ToolKind::Select);
                } else if i.key_pressed(egui::Key::M) {
                    self.shell.set_active_tool(ToolKind::Rectangle);
                } else if i.key_pressed(egui::Key::E) {
                    self.shell.set_active_tool(ToolKind::Ellipse);
                } else if i.key_pressed(egui::Key::P) {
                    self.shell.set_active_tool(ToolKind::Pen);
                } else if i.key_pressed(egui::Key::A) {
                    self.shell.set_active_tool(ToolKind::Node);
                }
            }
        });

        // 1. Top Panel: Menus & Context Toolbar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.heading("🎨 Aubrieta Design Studio [egui Experimental]");
                ui.separator();
                ui.menu_button("File", |ui| {
                    if ui.button("New Document").clicked() {
                        let _ = self.shell.new_document("New Document");
                        ui.close_menu();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo (Ctrl+Z)").clicked() {
                        let _ = self.shell.undo();
                        ui.close_menu();
                    }
                    if ui.button("Redo (Ctrl+Y)").clicked() {
                        let _ = self.shell.redo();
                        ui.close_menu();
                    }
                    if ui.button("Delete Selected (Del)").clicked() {
                        let sel = self.shell.bridge.selection();
                        for id in sel.selected_ids {
                            let _ = self.shell.bridge.submit_command(CommandRequest::new(Command::DeleteObject { id }));
                        }
                        self.shell.bridge.clear_selection();
                        ui.close_menu();
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Fit Canvas").clicked() {
                        if let Some(surface) = self.shell.bridge.session().and_then(|s| s.document.surfaces.first()) {
                            let b = surface.bounds();
                            self.shell.fit_surface(GRect::new(b[0], b[1], b[0] + b[2], b[1] + b[3]));
                        }
                        ui.close_menu();
                    }
                });
            });

            ui.separator();

            // Context toolbar with interactive quick actions
            ui.horizontal(|ui| {
                let tool_label = format!("Tool: {:?}", self.shell.active_tool());
                ui.label(egui::RichText::new(tool_label).strong().color(egui::Color32::LIGHT_BLUE));
                ui.separator();

                if ui.button("+ Retângulo").clicked() {
                    let mut id_gen = IdGenerator::new();
                    if let Some(surface) = self.shell.bridge.session().and_then(|s| s.document.surfaces.first().cloned()) {
                        let new_id = id_gen.next_object();
                        let count = surface.objects.len() + 1;
                        let _ = self.shell.bridge.submit_command(CommandRequest::new(Command::CreateObject {
                            surface: surface.id,
                            id: new_id,
                            name: format!("Rectangle {}", count),
                        }));
                        let offset = (count as f64 * 35.0) % 250.0;
                        let _ = self.shell.bridge.set_bounds(new_id, Some([120.0 + offset, 120.0 + offset, 200.0, 130.0]), 0.0);
                        let _ = self.shell.bridge.set_fill(new_id, Some("aubrieta.green/500".to_string()));
                        self.shell.bridge.set_selection(vec![new_id]);
                    }
                }

                if ui.button("+ Elipse").clicked() {
                    let mut id_gen = IdGenerator::new();
                    if let Some(surface) = self.shell.bridge.session().and_then(|s| s.document.surfaces.first().cloned()) {
                        let new_id = id_gen.next_object();
                        let count = surface.objects.len() + 1;
                        let _ = self.shell.bridge.submit_command(CommandRequest::new(Command::CreateObject {
                            surface: surface.id,
                            id: new_id,
                            name: format!("Circle {}", count),
                        }));
                        let offset = (count as f64 * 35.0) % 250.0;
                        let _ = self.shell.bridge.set_bounds(new_id, Some([360.0 + offset, 180.0 + offset, 130.0, 130.0]), 0.0);
                        let _ = self.shell.bridge.set_fill(new_id, Some("aubrieta.purple/500".to_string()));
                        self.shell.bridge.set_selection(vec![new_id]);
                    }
                }

                if ui.button("🗑 Deletar").clicked() {
                    let sel = self.shell.bridge.selection();
                    for id in sel.selected_ids {
                        let _ = self.shell.bridge.submit_command(CommandRequest::new(Command::DeleteObject { id }));
                    }
                    self.shell.bridge.clear_selection();
                }

                ui.separator();

                if ui.button("Undo").clicked() {
                    let _ = self.shell.undo();
                }
                if ui.button("Redo").clicked() {
                    let _ = self.shell.redo();
                }
                if ui.button("Fit Canvas").clicked() {
                    if let Some(surface) = self.shell.bridge.session().and_then(|s| s.document.surfaces.first()) {
                        let b = surface.bounds();
                        self.shell.fit_surface(GRect::new(b[0], b[1], b[0] + b[2], b[1] + b[3]));
                    }
                }

                ui.separator();

                let zoom_pct = (self.shell.camera.zoom * 100.0).round() as i32;
                ui.label(format!("Zoom: {zoom_pct}%"));
                if ui.button("+").clicked() {
                    let center = GPoint::new(400.0, 300.0);
                    self.shell.zoom_at(center, 1.2);
                }
                if ui.button("-").clicked() {
                    let center = GPoint::new(400.0, 300.0);
                    self.shell.zoom_at(center, 0.8);
                }

                ui.separator();

                let mut snap_enabled = self.shell.snap.config.grid_enabled;
                if ui.checkbox(&mut snap_enabled, "Magnetic Snap").changed() {
                    self.shell.snap.config.grid_enabled = snap_enabled;
                    self.shell.snap.config.guides_enabled = snap_enabled;
                }
            });
        });

        // 2. Left Side Panel: Tool Rail
        egui::SidePanel::left("left_tools").exact_width(48.0).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let tools = [
                    (ToolKind::Select, "V\nSel"),
                    (ToolKind::Node, "A\nNod"),
                    (ToolKind::Pen, "P\nPen"),
                    (ToolKind::Rectangle, "M\nRec"),
                    (ToolKind::Ellipse, "E\nEll"),
                    (ToolKind::Polygon, "G\nPol"),
                ];

                for (tool, label) in tools {
                    let is_active = self.shell.active_tool() == tool;
                    if ui.selectable_label(is_active, label).clicked() {
                        self.shell.set_active_tool(tool);
                    }
                    ui.add_space(4.0);
                }
            });
        });

        // 3. Right Side Panel: Dock Inspector
        egui::SidePanel::right("right_dock").exact_width(290.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, ActiveTab::Layers, "Layers");
                ui.selectable_value(&mut self.active_tab, ActiveTab::Properties, "Props");
                ui.selectable_value(&mut self.active_tab, ActiveTab::History, "History");
                ui.selectable_value(&mut self.active_tab, ActiveTab::DataMerge, "Merge");
            });
            ui.separator();

            match self.active_tab {
                ActiveTab::Layers => {
                    ui.heading("Object Hierarchy");
                    let model: LayersPresentationModel = self.shell.query_layers();
                    for surface in &model.surfaces {
                        ui.label(egui::RichText::new(format!("📄 {}", surface.name)).color(egui::Color32::LIGHT_BLUE));
                    }
                    ui.add_space(4.0);

                    let mut action_select = None;
                    let mut action_delete = None;
                    if let Some(session) = self.shell.bridge.session() {
                        if let Some(surface) = session.document.surfaces.first() {
                            let sel = self.shell.bridge.selection();
                            for obj in &surface.objects {
                                let is_selected = sel.contains(obj.id);
                                ui.horizontal(|ui| {
                                    let icon = if is_selected { "●" } else { "○" };
                                    if ui.selectable_label(is_selected, format!("{icon} {}", obj.name)).clicked() {
                                        action_select = Some(obj.id);
                                    }
                                    if ui.small_button("🗑").clicked() {
                                        action_delete = Some(obj.id);
                                    }
                                });
                            }
                        }
                    }
                    if let Some(id) = action_select {
                        self.shell.bridge.set_selection(vec![id]);
                    }
                    if let Some(id) = action_delete {
                        let _ = self.shell.bridge.submit_command(CommandRequest::new(Command::DeleteObject { id }));
                        self.shell.bridge.clear_selection();
                    }
                }
                ActiveTab::Properties => {
                    ui.heading("Properties Inspector");
                    let sel = self.shell.bridge.selection();
                    let selected_obj = if let Some(session) = self.shell.bridge.session() {
                        if let Some(first_id) = sel.selected_ids.first() {
                            session.document.surfaces.iter().flat_map(|s| &s.objects).find(|o| o.id == *first_id).cloned()
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some(obj) = selected_obj {
                        ui.label(egui::RichText::new(format!("Objeto: {}", obj.name)).strong());

                        if let Some(bounds) = obj.bounds {
                            let mut x = bounds[0];
                            let mut y = bounds[1];
                            let mut w = bounds[2];
                            let mut h = bounds[3];

                            ui.label(egui::RichText::new("Dimensões & Posição:").strong());
                            let mut changed = false;
                            ui.horizontal(|ui| {
                                changed |= ui.add(egui::DragValue::new(&mut x).speed(1.0).prefix("X: ")).changed();
                                changed |= ui.add(egui::DragValue::new(&mut y).speed(1.0).prefix("Y: ")).changed();
                            });
                            ui.horizontal(|ui| {
                                changed |= ui.add(egui::DragValue::new(&mut w).speed(1.0).prefix("W: ")).changed();
                                changed |= ui.add(egui::DragValue::new(&mut h).speed(1.0).prefix("H: ")).changed();
                            });
                            if changed {
                                let _ = self.shell.bridge.set_bounds(obj.id, Some([x, y, w, h]), 0.0);
                            }
                        }

                        ui.separator();
                        ui.label(egui::RichText::new("Cores Rápidas:").strong());
                        ui.horizontal(|ui| {
                            let colors = [
                                ("#3b82f6", egui::Color32::from_rgb(59, 130, 246), "aubrieta.blue/500"),
                                ("#10b981", egui::Color32::from_rgb(16, 185, 129), "aubrieta.green/500"),
                                ("#eab308", egui::Color32::from_rgb(234, 179, 8), "aubrieta.yellow/500"),
                                ("#8b5cf6", egui::Color32::from_rgb(139, 92, 246), "aubrieta.purple/500"),
                                ("#f43f5e", egui::Color32::from_rgb(244, 63, 94), "aubrieta.rose/500"),
                            ];
                            for (_hex, color, name) in colors {
                                let (rect, resp) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());
                                ui.painter().rect_filled(rect, 4.0, color);
                                ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0_f32, egui::Color32::WHITE), egui::StrokeKind::Middle);
                                if resp.clicked() {
                                    let _ = self.shell.bridge.set_fill(obj.id, Some(name.to_string()));
                                }
                            }
                        });

                        ui.add_space(8.0);
                        if ui.button("🗑 Excluir Objeto").clicked() {
                            let _ = self.shell.bridge.submit_command(CommandRequest::new(Command::DeleteObject { id: obj.id }));
                            self.shell.bridge.clear_selection();
                        }
                    } else {
                        ui.label(egui::RichText::new("(Nenhum objeto selecionado)").italics().color(egui::Color32::GRAY));
                        ui.label("Selecione um objeto no canvas ou na lista de camadas para editar suas propriedades.");
                    }
                }
                ActiveTab::History => {
                    ui.heading("Transaction History");
                    let model: HistoryPresentationModel = self.shell.bridge.query_history();
                    ui.label(format!("Undo: {} | Redo: {}", model.undo_stack.len(), model.redo_stack.len()));
                    ui.separator();
                    for (i, item) in model.undo_stack.iter().enumerate().rev() {
                        ui.label(format!("[{}] {}", i + 1, item.description));
                    }
                }
                ActiveTab::DataMerge => {
                    ui.heading("Variable Data Merge");
                    let model: DataMergePresentationModel = self.shell.query_data_merge();
                    let src = model.sources.first().map(|s| s.name.as_str()).unwrap_or("(No source loaded)");
                    ui.label(format!("Source: {src}"));
                    ui.label(format!("Records: {} | Bindings: {}", model.total_records, model.bindings.len()));
                    ui.colored_label(egui::Color32::GREEN, "Status: Ready to Merge");
                }
            }
        });

        // 4. Bottom Status Bar
        egui::TopBottomPanel::bottom("bottom_status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Active Tool: {:?}", self.shell.active_tool()));
                ui.separator();
                if let Some(pt) = self.hovered_doc_point {
                    ui.label(format!("Doc: X: {:.1} pt  Y: {:.1} pt", pt.x, pt.y));
                } else {
                    ui.label("Cursor outside canvas");
                }
                ui.separator();
                let zoom_pct = (self.shell.camera.zoom * 100.0).round() as i32;
                ui.label(format!("Zoom: {zoom_pct}% | egui Experimental Shell"));
            });
        });

        // 5. Central Panel: Interactive Canvas Viewport
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
            let canvas_rect = response.rect;

            // Background canvas dark color
            painter.rect_filled(canvas_rect, 0.0, egui::Color32::from_rgb(20, 20, 22));

            let camera = self.shell.camera.clone();
            let to_screen = |doc_pt: GPoint| -> egui::Pos2 {
                let s = camera.doc_to_screen(doc_pt);
                egui::pos2(canvas_rect.min.x + s.x as f32, canvas_rect.min.y + s.y as f32)
            };

            // Zoom on mouse scroll wheel
            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
            if scroll_delta != 0.0 && response.hovered() {
                let factor = if scroll_delta > 0.0 { 1.15 } else { 0.85 };
                if let Some(pos) = response.hover_pos() {
                    let screen_focus = GPoint::new((pos.x - canvas_rect.min.x) as f64, (pos.y - canvas_rect.min.y) as f64);
                    self.shell.zoom_at(screen_focus, factor);
                }
            }

            // Pan on middle mouse button drag or Space+drag
            let is_panning = ui.input(|i| i.pointer.button_down(egui::PointerButton::Middle) || (i.key_down(egui::Key::Space) && i.pointer.primary_down()));
            if is_panning {
                let delta = ui.input(|i| i.pointer.delta());
                if delta.length_sq() > 0.0 {
                    self.shell.pan(delta.x as f64, delta.y as f64);
                }
            }

            // Track pointer hover
            if let Some(mouse_pos) = response.hover_pos() {
                let local_x = (mouse_pos.x - canvas_rect.min.x) as f64;
                let local_y = (mouse_pos.y - canvas_rect.min.y) as f64;
                self.hovered_doc_point = Some(self.shell.camera.screen_to_doc(GPoint::new(local_x, local_y)));
            }

            // Draw Artboards/Surfaces & Objects
            if let Some(session) = self.shell.bridge.session() {
                let sel = self.shell.bridge.selection();

                for surface in &session.document.surfaces {
                    let b = surface.bounds();
                    let p0 = to_screen(GPoint::new(b[0], b[1]));
                    let p1 = to_screen(GPoint::new(b[0] + b[2], b[1] + b[3]));
                    let paper_rect = egui::Rect::from_two_pos(p0, p1);

                    // Artboard drop shadow
                    let shadow_rect = paper_rect.translate(egui::vec2(4.0, 4.0));
                    painter.rect_filled(shadow_rect, 2.0, egui::Color32::from_black_alpha(100));

                    // Artboard Paper White
                    painter.rect_filled(paper_rect, 0.0, egui::Color32::WHITE);
                    painter.rect_stroke(paper_rect, 0.0, egui::Stroke::new(1.0_f32, egui::Color32::LIGHT_GRAY), egui::StrokeKind::Middle);

                    // Artboard Name header
                    painter.text(
                        egui::pos2(paper_rect.min.x, paper_rect.min.y - 14.0),
                        egui::Align2::LEFT_TOP,
                        format!("📄 {} ({:.0}x{:.0} pt) • Arraste para desenhar / mover", surface.name, b[2], b[3]),
                        egui::FontId::proportional(12.0),
                        egui::Color32::GRAY,
                    );

                    // Bleed guide line (magenta)
                    if !surface.bleed.is_zero() {
                        let bp0 = to_screen(GPoint::new(b[0] - surface.bleed.left, b[1] - surface.bleed.top));
                        let bp1 = to_screen(GPoint::new(b[0] + b[2] + surface.bleed.right, b[1] + b[3] + surface.bleed.bottom));
                        painter.rect_stroke(egui::Rect::from_two_pos(bp0, bp1), 0.0, egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(244, 63, 94)), egui::StrokeKind::Middle);
                    }

                    // Margin guide line (sky blue)
                    if surface.margins != Margins::ZERO {
                        let mp0 = to_screen(GPoint::new(b[0] + surface.margins.left, b[1] + surface.margins.top));
                        let mp1 = to_screen(GPoint::new(b[0] + b[2] - surface.margins.right, b[1] + b[3] - surface.margins.bottom));
                        painter.rect_stroke(egui::Rect::from_two_pos(mp0, mp1), 0.0, egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(56, 189, 248)), egui::StrokeKind::Middle);
                    }

                    // Render objects
                    for obj in &surface.objects {
                        let obj_b = obj.bounds.unwrap_or([0.0, 0.0, 100.0, 80.0]);
                        let op0 = to_screen(GPoint::new(obj_b[0], obj_b[1]));
                        let op1 = to_screen(GPoint::new(obj_b[0] + obj_b[2], obj_b[1] + obj_b[3]));
                        let obj_rect = egui::Rect::from_two_pos(op0, op1);

                        let is_circle = obj.name.to_lowercase().contains("circle") || obj.name.to_lowercase().contains("ellipse");
                        let is_selected = sel.contains(obj.id);

                        let color = if let Some(fill) = &obj.fill {
                            if fill.contains("blue") {
                                egui::Color32::from_rgb(59, 130, 246)
                            } else if fill.contains("yellow") {
                                egui::Color32::from_rgb(234, 179, 8)
                            } else if fill.contains("green") {
                                egui::Color32::from_rgb(16, 185, 129)
                            } else if fill.contains("purple") {
                                egui::Color32::from_rgb(139, 92, 246)
                            } else if fill.contains("rose") || fill.contains("red") {
                                egui::Color32::from_rgb(244, 63, 94)
                            } else {
                                egui::Color32::from_rgb(59, 130, 246)
                            }
                        } else if is_circle {
                            egui::Color32::from_rgb(234, 179, 8)
                        } else {
                            egui::Color32::from_rgb(59, 130, 246)
                        };

                        let rounding = if is_circle { obj_rect.width() / 2.0 } else { 4.0 };
                        painter.rect_filled(obj_rect, rounding, color);
                        painter.rect_stroke(obj_rect, rounding, egui::Stroke::new(if is_selected { 2.5_f32 } else { 1.0_f32 }, if is_selected { egui::Color32::from_rgb(96, 165, 250) } else { egui::Color32::WHITE }), egui::StrokeKind::Middle);

                        // If selected, draw selection handles
                        if is_selected {
                            let ring = obj_rect.expand(4.0);
                            painter.rect_stroke(ring, rounding + 4.0, egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(59, 130, 246)), egui::StrokeKind::Middle);
                            for handle_pos in [obj_rect.left_top(), obj_rect.right_top(), obj_rect.left_bottom(), obj_rect.right_bottom()] {
                                painter.rect_filled(egui::Rect::from_center_size(handle_pos, egui::vec2(8.0, 8.0)), 1.0, egui::Color32::WHITE);
                                painter.rect_stroke(egui::Rect::from_center_size(handle_pos, egui::vec2(8.0, 8.0)), 1.0_f32, egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(59, 130, 246)), egui::StrokeKind::Middle);
                            }
                        }

                        painter.text(
                            obj_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            &obj.name,
                            egui::FontId::proportional(13.0),
                            egui::Color32::WHITE,
                        );
                    }
                }
            }

            // Interactive Drawing Preview (Rectangle / Ellipse tool)
            if let (Some(start_doc), Some(current_doc)) = (self.drag_start_doc, self.hovered_doc_point) {
                if matches!(self.shell.active_tool(), ToolKind::Rectangle | ToolKind::Ellipse) {
                    let p0 = to_screen(start_doc);
                    let p1 = to_screen(current_doc);
                    let preview_rect = egui::Rect::from_two_pos(p0, p1);
                    let is_ellipse = self.shell.active_tool() == ToolKind::Ellipse;
                    let rounding = if is_ellipse { preview_rect.width() / 2.0 } else { 4.0 };

                    painter.rect_filled(preview_rect, rounding, egui::Color32::from_rgba_unmultiplied(59, 130, 246, 50));
                    painter.rect_stroke(preview_rect, rounding, egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(59, 130, 246)), egui::StrokeKind::Middle);
                }
            }

            // Interactive Pointer Events inside canvas
            if !is_panning {
                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let screen_pt = GPoint::new((pos.x - canvas_rect.min.x) as f64, (pos.y - canvas_rect.min.y) as f64);
                        let doc_pt = self.shell.camera.screen_to_doc(screen_pt);

                        // Hit test objects
                        let mut hit = None;
                        if let Some(session) = self.shell.bridge.session() {
                            if let Some(surface) = session.document.surfaces.first() {
                                for obj in surface.objects.iter().rev() {
                                    if let Some(b) = obj.bounds {
                                        if doc_pt.x >= b[0] && doc_pt.x <= b[0] + b[2] && doc_pt.y >= b[1] && doc_pt.y <= b[1] + b[3] {
                                            hit = Some(obj.id);
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(id) = hit {
                            self.shell.bridge.set_selection(vec![id]);
                            self.active_tab = ActiveTab::Properties;
                        } else if self.shell.active_tool() == ToolKind::Select {
                            self.shell.bridge.clear_selection();
                        }
                    }
                } else if response.drag_started() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let screen_pt = GPoint::new((pos.x - canvas_rect.min.x) as f64, (pos.y - canvas_rect.min.y) as f64);
                        let doc_pt = self.shell.camera.screen_to_doc(screen_pt);
                        self.drag_start_doc = Some(doc_pt);

                        // If select tool, check if dragging an object
                        let mut hit = None;
                        let mut hit_b = None;
                        if let Some(session) = self.shell.bridge.session() {
                            if let Some(surface) = session.document.surfaces.first() {
                                for obj in surface.objects.iter().rev() {
                                    if let Some(b) = obj.bounds {
                                        if doc_pt.x >= b[0] && doc_pt.x <= b[0] + b[2] && doc_pt.y >= b[1] && doc_pt.y <= b[1] + b[3] {
                                            hit = Some(obj.id);
                                            hit_b = Some(b);
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(id) = hit {
                            self.shell.bridge.set_selection(vec![id]);
                            self.dragging_object_id = Some(id);
                            self.drag_initial_bounds = hit_b;
                            self.active_tab = ActiveTab::Properties;
                        }

                        let evt = NormalizedPointerEvent::new(PointerPhase::Down, PointerButton::Primary, screen_pt, doc_pt, SemanticModifiers::default());
                        let _ = self.shell.handle_pointer_event(&evt);
                    }
                } else if response.dragged() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let screen_pt = GPoint::new((pos.x - canvas_rect.min.x) as f64, (pos.y - canvas_rect.min.y) as f64);
                        let doc_pt = self.shell.camera.screen_to_doc(screen_pt);

                        if self.shell.active_tool() == ToolKind::Select {
                            if let (Some(start), Some(id), Some(init_b)) = (self.drag_start_doc, self.dragging_object_id, self.drag_initial_bounds) {
                                let dx = doc_pt.x - start.x;
                                let dy = doc_pt.y - start.y;
                                let new_b = [init_b[0] + dx, init_b[1] + dy, init_b[2], init_b[3]];
                                let _ = self.shell.bridge.set_bounds(id, Some(new_b), 0.0);
                            }
                        }

                        let evt = NormalizedPointerEvent::new(PointerPhase::Move, PointerButton::Primary, screen_pt, doc_pt, SemanticModifiers::default());
                        let _ = self.shell.handle_pointer_event(&evt);
                    }
                } else if response.drag_stopped() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let screen_pt = GPoint::new((pos.x - canvas_rect.min.x) as f64, (pos.y - canvas_rect.min.y) as f64);
                        let doc_pt = self.shell.camera.screen_to_doc(screen_pt);

                        // If drawing a shape, create the object on release
                        if let Some(start) = self.drag_start_doc.take() {
                            let dx = (doc_pt.x - start.x).abs();
                            let dy = (doc_pt.y - start.y).abs();
                            if matches!(self.shell.active_tool(), ToolKind::Rectangle | ToolKind::Ellipse) && dx > 8.0 && dy > 8.0 {
                                let min_x = start.x.min(doc_pt.x);
                                let min_y = start.y.min(doc_pt.y);
                                let mut id_gen = IdGenerator::new();
                                if let Some(surface) = self.shell.bridge.session().and_then(|s| s.document.surfaces.first().cloned()) {
                                    let new_id = id_gen.next_object();
                                    let count = surface.objects.len() + 1;
                                    let (name, fill) = match self.shell.active_tool() {
                                        ToolKind::Ellipse => (format!("Ellipse {}", count), "aubrieta.purple/500"),
                                        _ => (format!("Rectangle {}", count), "aubrieta.green/500"),
                                    };
                                    let _ = self.shell.bridge.submit_command(CommandRequest::new(Command::CreateObject {
                                        surface: surface.id,
                                        id: new_id,
                                        name,
                                    }));
                                    let _ = self.shell.bridge.set_bounds(new_id, Some([min_x, min_y, dx, dy]), 0.0);
                                    let _ = self.shell.bridge.set_fill(new_id, Some(fill.to_string()));
                                    self.shell.bridge.set_selection(vec![new_id]);
                                    self.active_tab = ActiveTab::Properties;
                                }
                            }
                        }

                        self.dragging_object_id = None;
                        self.drag_initial_bounds = None;

                        let evt = NormalizedPointerEvent::new(PointerPhase::Up, PointerButton::Primary, screen_pt, doc_pt, SemanticModifiers::default());
                        let _ = self.shell.handle_pointer_event(&evt);
                    }
                }
            }
        });
    }
}

fn populate_showcase_document(shell: &mut AubrietaShell) -> Result<(), AubrietaError> {
    shell.new_document("Aubrieta Showcase Project [egui]")?;
    let mut id_gen = IdGenerator::new();
    let surface_1 = id_gen.next_surface();
    let rect_id = id_gen.next_object();
    let circle_id = id_gen.next_object();

    shell.bridge.submit_command(CommandRequest::new(Command::CreateSurface {
        id: surface_1,
        name: "Main Artboard".to_string(),
    }))?;
    shell.bridge.set_surface_geometry(surface_1, [60.0, 60.0], [800.0, 600.0])?;
    shell.bridge.set_surface_bleed(surface_1, Bleed::uniform(10.0))?;
    shell.bridge.set_surface_margins(surface_1, Margins::uniform(36.0))?;
    shell.bridge.add_surface_guide(surface_1, Guide::new(1, GuideOrientation::Vertical, 200.0))?;

    shell.bridge.submit_command(CommandRequest::new(Command::CreateObject {
        surface: surface_1,
        id: rect_id,
        name: "Hero Card".to_string(),
    }))?;
    shell.bridge.set_bounds(rect_id, Some([100.0, 100.0, 300.0, 180.0]), 0.0)?;
    shell.bridge.set_fill(rect_id, Some("aubrieta.blue/500".to_string()))?;

    shell.bridge.submit_command(CommandRequest::new(Command::CreateObject {
        surface: surface_1,
        id: circle_id,
        name: "Accent Circle".to_string(),
    }))?;
    shell.bridge.set_bounds(circle_id, Some([450.0, 160.0, 140.0, 140.0]), 0.0)?;
    shell.bridge.set_fill(circle_id, Some("aubrieta.yellow/500".to_string()))?;

    shell.bridge.set_selection(vec![rect_id]);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--smoke-test" || a == "--headless") {
        println!("aubrieta-egui: Running automated smoke test...");
        let mut app = AubrietaEguiApp::new();
        if let Err(e) = app.smoke_test() {
            eprintln!("aubrieta-egui smoke test failed: {e}");
            std::process::exit(1);
        }
        println!("aubrieta-egui: Smoke test PASSED.");
        return Ok(());
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Aubrieta Design Studio — egui Secondary Experimental")
            .with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Aubrieta Design Studio",
        native_options,
        Box::new(|_cc| Ok(Box::new(AubrietaEguiApp::new()))),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn egui_app_smoke_test_headless() {
        let mut app = AubrietaEguiApp::new();
        assert!(app.smoke_test().is_ok());
    }
}
