//! Aubrieta Creative Studio — egui / eframe Secondary GUI.
//!
//! Evaluates egui immediate-mode GUI with Canvas graphics,
//! consuming AubrietaGuiBridge presentation models and dispatching canonical commands
//! matching the Affinity Designer Dark Studio specifications.

use std::env;

use aubrieta_application::{Command, CommandRequest};
use aubrieta_document::{
    AlignmentMode, Bleed, DistributionAxis, Guide, GuideOrientation, Margins, ShapeKind,
};
use aubrieta_foundation::{AubrietaError, IdGenerator, ObjectId};
use aubrieta_geometry::{BooleanOp, GPoint, GRect};
use aubrieta_ui_gpui::bridge::{
    DataMergePresentationModel, HistoryPresentationModel, LayersPresentationModel,
    PropertiesPresentationModel,
};
use aubrieta_ui_gpui::shell::AubrietaShell;
use aubrieta_ui_gpui::tools::{
    NormalizedPointerEvent, PointerButton, PointerPhase, SemanticModifiers, ToolKind,
};
use eframe::egui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Persona {
    Design,
    Photo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActiveTab {
    Layers,
    Properties,
    History,
    DataMerge,
}

struct AubrietaEguiApp {
    shell: AubrietaShell,
    active_persona: Persona,
    active_tab: ActiveTab,
    hovered_doc_point: Option<GPoint>,
    drag_start_doc: Option<GPoint>,
    dragging_object_id: Option<ObjectId>,
    drag_initial_bounds: Option<[f64; 4]>,
    preview_rect: Option<[f64; 4]>,
    selected_opacity: f32,
}

impl Default for AubrietaEguiApp {
    fn default() -> Self {
        Self::new()
    }
}

impl AubrietaEguiApp {
    fn new() -> Self {
        let mut shell = AubrietaShell::new(950.0, 700.0);
        let _ = populate_showcase_document(&mut shell);
        Self {
            shell,
            active_persona: Persona::Design,
            active_tab: ActiveTab::Layers,
            hovered_doc_point: None,
            drag_start_doc: None,
            dragging_object_id: None,
            drag_initial_bounds: None,
            preview_rect: None,
            selected_opacity: 100.0,
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
            ToolKind::Star,
            ToolKind::ArtisticText,
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

    fn apply_boolean(&mut self, op: BooleanOp) {
        let mut sel_ids = self.shell.bridge.selection().selected_ids.clone();
        if sel_ids.len() < 2 {
            if let Some(session) = self.shell.bridge.session() {
                if let Some(surface) = session.document.surfaces.first() {
                    if surface.objects.len() >= 2 {
                        sel_ids = vec![surface.objects[0].id, surface.objects[1].id];
                    }
                }
            }
        }
        if sel_ids.len() >= 2 {
            let id_a = sel_ids[0];
            let id_b = sel_ids[1];
            let mut id_gen = IdGenerator::new();
            let target_id = id_gen.next_object();
            if let Some(surface) = self
                .shell
                .bridge
                .session()
                .and_then(|s| s.document.surfaces.first().cloned())
            {
                let _ = self
                    .shell
                    .bridge
                    .apply_boolean(surface.id, target_id, id_a, id_b, op);
                self.shell.bridge.set_selection(vec![target_id]);
            }
        }
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
            } else if i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace) {
                let sel = self.shell.bridge.selection();
                for id in sel.selected_ids {
                    let _ = self
                        .shell
                        .bridge
                        .submit_command(CommandRequest::new(Command::DeleteObject { id }));
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
            } else if i.key_pressed(egui::Key::T) {
                self.shell.set_active_tool(ToolKind::ArtisticText);
            }
        });

        // -------------------------------------------------------------
        // SECTION 1 & 2 & 3: TOP PANEL (MENU BAR + PERSONA + CONTEXT HUD)
        // -------------------------------------------------------------
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // 1. Studio Menu Bar & Persona Switcher
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("A")
                        .strong()
                        .color(egui::Color32::WHITE)
                        .background_color(egui::Color32::from_rgb(59, 130, 246)),
                );
                ui.label(
                    egui::RichText::new("Aubrieta Studio")
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                ui.separator();

                ui.menu_button("Arquivo", |ui| {
                    if ui.button("Novo Documento").clicked() {
                        let _ = self.shell.new_document("Sem Título");
                        ui.close_menu();
                    }
                    if ui.button("Abrir...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Aubrieta Design (*.aub)", &["aub"])
                            .add_filter("Gráficos Vetoriais SVG (*.svg)", &["svg"])
                            .add_filter("Todos os arquivos (*.*)", &["*"])
                            .set_title("Abrir Documento Aubrieta")
                            .pick_file()
                        {
                            let title = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Novo Documento");
                            let _ = self.shell.new_document(title);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Salvar Como...").clicked() {
                        let default_name = self
                            .shell
                            .bridge
                            .session()
                            .map(|s| format!("{}.aub", s.title))
                            .unwrap_or_else(|| "projeto.aub".to_string());
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Aubrieta Design (*.aub)", &["aub"])
                            .set_file_name(&default_name)
                            .set_title("Salvar Projeto Aubrieta")
                            .save_file()
                        {
                            println!("RFD: Salvando projeto em: {:?}", path);
                        }
                        ui.close_menu();
                    }
                });

                ui.menu_button("Editar", |ui| {
                    if ui.button("Desfazer (Ctrl+Z)").clicked() {
                        let _ = self.shell.undo();
                        ui.close_menu();
                    }
                    if ui.button("Refazer (Ctrl+Y)").clicked() {
                        let _ = self.shell.redo();
                        ui.close_menu();
                    }
                    if ui.button("Excluir Seleção (Del)").clicked() {
                        let sel = self.shell.bridge.selection();
                        for id in sel.selected_ids {
                            let _ = self
                                .shell
                                .bridge
                                .submit_command(CommandRequest::new(Command::DeleteObject { id }));
                        }
                        self.shell.bridge.clear_selection();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Camada", |ui| {
                    if ui.button("Converter em Curvas").clicked() {
                        let sel = self.shell.bridge.selection().selected_ids.clone();
                        for id in sel {
                            let _ = self.shell.bridge.convert_to_curves(id);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Bake Cantos").clicked() {
                        let sel = self.shell.bridge.selection().selected_ids.clone();
                        for id in sel {
                            let _ = self.shell.bridge.bake_corners(id);
                        }
                        ui.close_menu();
                    }
                });

                ui.menu_button("Selecionar", |ui| {
                    if ui.button("Ver Camadas").clicked() {
                        self.active_tab = ActiveTab::Layers;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Documento", |ui| {
                    if ui.button("Ajustar à Tela").clicked() {
                        if let Some(surface) = self
                            .shell
                            .bridge
                            .session()
                            .and_then(|s| s.document.surfaces.first())
                        {
                            let b = surface.bounds();
                            self.shell.fit_surface(GRect::new(
                                b[0],
                                b[1],
                                b[0] + b[2],
                                b[1] + b[3],
                            ));
                        }
                        ui.close_menu();
                    }
                });

                ui.separator();

                // Persona Switcher (08.2)
                ui.selectable_value(
                    &mut self.active_persona,
                    Persona::Design,
                    "🎨 Design Persona",
                );
                ui.selectable_value(&mut self.active_persona, Persona::Photo, "📷 Photo Persona");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let zoom_pct = (self.shell.camera.zoom * 100.0).round() as i32;
                    let mut snap_enabled = self.shell.snap.config.grid_enabled;
                    if ui.checkbox(&mut snap_enabled, "Snap").changed() {
                        self.shell.snap.config.grid_enabled = snap_enabled;
                        self.shell.snap.config.guides_enabled = snap_enabled;
                    }
                    ui.label(format!("{zoom_pct}%"));
                    if ui.button("+").clicked() {
                        let center = GPoint::new(400.0, 300.0);
                        self.shell.zoom_at(center, 1.2);
                    }
                    if ui.button("−").clicked() {
                        let center = GPoint::new(400.0, 300.0);
                        self.shell.zoom_at(center, 0.8);
                    }
                    if ui.button("Desfazer").clicked() {
                        let _ = self.shell.undo();
                    }
                    if ui.button("Refazer").clicked() {
                        let _ = self.shell.redo();
                    }
                });
            });

            ui.separator();

            // 2. Canonical Context Toolbar (08.23)
            ui.horizontal(|ui| {
                let tool_label = format!("{:?}", self.shell.active_tool());
                ui.label(
                    egui::RichText::new(tool_label)
                        .strong()
                        .color(egui::Color32::from_rgb(96, 165, 250)),
                );
                ui.separator();

                let props: PropertiesPresentationModel = self.shell.query_properties();
                if let Some(b) = props.bounds {
                    ui.label(format!(
                        "X: {:.0} pt  Y: {:.0} pt  W: {:.0} pt  H: {:.0} pt",
                        b[0], b[1], b[2], b[3]
                    ));
                } else {
                    ui.label(
                        egui::RichText::new("(Nenhuma seleção)")
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                }

                ui.separator();

                if ui.button("Curvas").clicked() {
                    let sel = self.shell.bridge.selection().selected_ids.clone();
                    for id in sel {
                        let _ = self.shell.bridge.convert_to_curves(id);
                    }
                }
                if ui.button("Bake").clicked() {
                    let sel = self.shell.bridge.selection().selected_ids.clone();
                    for id in sel {
                        let _ = self.shell.bridge.bake_corners(id);
                    }
                }

                ui.separator();

                // Booleans
                if ui.button("⋃ Unir").clicked() {
                    self.apply_boolean(BooleanOp::Union);
                }
                if ui.button("− Sub").clicked() {
                    self.apply_boolean(BooleanOp::Difference);
                }
                if ui.button("⋂ Inter").clicked() {
                    self.apply_boolean(BooleanOp::Intersection);
                }
                if ui.button("⨁ Xor").clicked() {
                    self.apply_boolean(BooleanOp::Xor);
                }

                ui.separator();

                // Alignments
                if ui.button("Esq").clicked() {
                    let sel = self.shell.bridge.selection().selected_ids.clone();
                    if let Some(surf_id) = self.shell.bridge.active_surface().or_else(|| {
                        self.shell
                            .bridge
                            .session()
                            .and_then(|s| s.document.surfaces.first().map(|sf| sf.id))
                    }) {
                        let _ = self
                            .shell
                            .bridge
                            .align_objects(surf_id, sel, AlignmentMode::Left);
                    }
                }
                if ui.button("Centro").clicked() {
                    let sel = self.shell.bridge.selection().selected_ids.clone();
                    if let Some(surf_id) = self.shell.bridge.active_surface().or_else(|| {
                        self.shell
                            .bridge
                            .session()
                            .and_then(|s| s.document.surfaces.first().map(|sf| sf.id))
                    }) {
                        let _ =
                            self.shell
                                .bridge
                                .align_objects(surf_id, sel, AlignmentMode::Center);
                    }
                }
                if ui.button("Dir").clicked() {
                    let sel = self.shell.bridge.selection().selected_ids.clone();
                    if let Some(surf_id) = self.shell.bridge.active_surface().or_else(|| {
                        self.shell
                            .bridge
                            .session()
                            .and_then(|s| s.document.surfaces.first().map(|sf| sf.id))
                    }) {
                        let _ = self
                            .shell
                            .bridge
                            .align_objects(surf_id, sel, AlignmentMode::Right);
                    }
                }

                if ui.button("Topo").clicked() {
                    let sel = self.shell.bridge.selection().selected_ids.clone();
                    if let Some(surf_id) = self.shell.bridge.active_surface().or_else(|| {
                        self.shell
                            .bridge
                            .session()
                            .and_then(|s| s.document.surfaces.first().map(|sf| sf.id))
                    }) {
                        let _ = self
                            .shell
                            .bridge
                            .align_objects(surf_id, sel, AlignmentMode::Top);
                    }
                }
                if ui.button("Meio").clicked() {
                    let sel = self.shell.bridge.selection().selected_ids.clone();
                    if let Some(surf_id) = self.shell.bridge.active_surface().or_else(|| {
                        self.shell
                            .bridge
                            .session()
                            .and_then(|s| s.document.surfaces.first().map(|sf| sf.id))
                    }) {
                        let _ =
                            self.shell
                                .bridge
                                .align_objects(surf_id, sel, AlignmentMode::Middle);
                    }
                }
                if ui.button("Base").clicked() {
                    let sel = self.shell.bridge.selection().selected_ids.clone();
                    if let Some(surf_id) = self.shell.bridge.active_surface().or_else(|| {
                        self.shell
                            .bridge
                            .session()
                            .and_then(|s| s.document.surfaces.first().map(|sf| sf.id))
                    }) {
                        let _ =
                            self.shell
                                .bridge
                                .align_objects(surf_id, sel, AlignmentMode::Bottom);
                    }
                }

                ui.separator();

                if ui.button("Dist H").clicked() {
                    let sel = self.shell.bridge.selection().selected_ids.clone();
                    if let Some(surf_id) = self.shell.bridge.active_surface().or_else(|| {
                        self.shell
                            .bridge
                            .session()
                            .and_then(|s| s.document.surfaces.first().map(|sf| sf.id))
                    }) {
                        let _ = self.shell.bridge.distribute_objects(
                            surf_id,
                            sel,
                            DistributionAxis::Horizontal,
                        );
                    }
                }
                if ui.button("Dist V").clicked() {
                    let sel = self.shell.bridge.selection().selected_ids.clone();
                    if let Some(surf_id) = self.shell.bridge.active_surface().or_else(|| {
                        self.shell
                            .bridge
                            .session()
                            .and_then(|s| s.document.surfaces.first().map(|sf| sf.id))
                    }) {
                        let _ = self.shell.bridge.distribute_objects(
                            surf_id,
                            sel,
                            DistributionAxis::Vertical,
                        );
                    }
                }

                ui.separator();

                if ui.button("🗑 Excluir").clicked() {
                    let sel = self.shell.bridge.selection();
                    for id in sel.selected_ids {
                        let _ = self
                            .shell
                            .bridge
                            .submit_command(CommandRequest::new(Command::DeleteObject { id }));
                    }
                    self.shell.bridge.clear_selection();
                }
            });
        });

        // -------------------------------------------------------------
        // SECTION 4.1: LEFT TOOL RAIL (48px) WITH 5 GROUPS
        // -------------------------------------------------------------
        egui::SidePanel::left("left_tools")
            .exact_width(52.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    // Group 1: Selection
                    let tools_g1 = [
                        (ToolKind::Select, "V\nSel"),
                        (ToolKind::Node, "A\nNod"),
                        (ToolKind::PointTransform, "F\nPtf"),
                        (ToolKind::Artboard, "H\nArt"),
                    ];
                    for (tool, label) in tools_g1 {
                        let is_active = self.shell.active_tool() == tool;
                        if ui.selectable_label(is_active, label).clicked() {
                            self.shell.set_active_tool(tool);
                        }
                        ui.add_space(2.0);
                    }

                    ui.separator();

                    // Group 2: Drawing
                    let tools_g2 = [
                        (ToolKind::Pen, "P\nPen"),
                        (ToolKind::Pencil, "N\nPcl"),
                        (ToolKind::Rectangle, "M\nRec"),
                        (ToolKind::Ellipse, "E\nEll"),
                        (ToolKind::Star, "S\nStr"),
                    ];
                    for (tool, label) in tools_g2 {
                        let is_active = self.shell.active_tool() == tool;
                        if ui.selectable_label(is_active, label).clicked() {
                            self.shell.set_active_tool(tool);
                        }
                        ui.add_space(2.0);
                    }

                    ui.separator();

                    // Group 3: Content
                    let tools_g3 = [
                        (ToolKind::ArtisticText, "T\nTxt"),
                        (ToolKind::Gradient, "G\nGrd"),
                        (ToolKind::ColorPicker, "I\nPip"),
                    ];
                    for (tool, label) in tools_g3 {
                        let is_active = self.shell.active_tool() == tool;
                        if ui.selectable_label(is_active, label).clicked() {
                            self.shell.set_active_tool(tool);
                        }
                        ui.add_space(2.0);
                    }

                    ui.separator();

                    // Group 4: Editing
                    let tools_g4 = [
                        (ToolKind::Knife, "K\nKnf"),
                        (ToolKind::Scissors, "C\nSci"),
                        (ToolKind::ShapeBuilder, "W\nShp"),
                    ];
                    for (tool, label) in tools_g4 {
                        let is_active = self.shell.active_tool() == tool;
                        if ui.selectable_label(is_active, label).clicked() {
                            self.shell.set_active_tool(tool);
                        }
                        ui.add_space(2.0);
                    }

                    ui.separator();

                    // Group 5: Navigation
                    let tools_g5 = [(ToolKind::Hand, "␣\nHnd"), (ToolKind::Zoom, "Z\nZom")];
                    for (tool, label) in tools_g5 {
                        let is_active = self.shell.active_tool() == tool;
                        if ui.selectable_label(is_active, label).clicked() {
                            self.shell.set_active_tool(tool);
                        }
                        ui.add_space(2.0);
                    }
                });
            });

        // -------------------------------------------------------------
        // SECTION 4.3: RIGHT STUDIO DOCK (300px)
        // -------------------------------------------------------------
        egui::SidePanel::right("right_dock")
            .exact_width(300.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.active_tab, ActiveTab::Layers, "Camadas");
                    ui.selectable_value(
                        &mut self.active_tab,
                        ActiveTab::Properties,
                        "Propriedades",
                    );
                    ui.selectable_value(&mut self.active_tab, ActiveTab::History, "Histórico");
                    ui.selectable_value(&mut self.active_tab, ActiveTab::DataMerge, "Dados");
                });
                ui.separator();

                match self.active_tab {
                    ActiveTab::Layers => {
                        ui.heading("Hierarquia de Objetos");
                        let model: LayersPresentationModel = self.shell.query_layers();
                        for surface in &model.surfaces {
                            ui.label(
                                egui::RichText::new(format!("📄 {} [800 × 600]", surface.name))
                                    .color(egui::Color32::from_rgb(147, 197, 253)),
                            );
                        }
                        ui.add_space(4.0);

                        let mut items = Vec::new();
                        let mut surf_id = None;
                        if let Some(session) = self.shell.bridge.session() {
                            if let Some(surface) = session.document.surfaces.first() {
                                surf_id = Some(surface.id);
                                let sel = self.shell.bridge.selection();
                                for (idx, obj) in surface.objects.iter().enumerate() {
                                    items.push((
                                        idx,
                                        obj.id,
                                        obj.name.clone(),
                                        obj.visible,
                                        obj.locked,
                                        sel.contains(obj.id),
                                    ));
                                }
                            }
                        }

                        let mut action_select = None;
                        let mut action_delete = None;
                        let total_items = items.len();

                        for (idx, id, name, visible, locked, is_selected) in items {
                            let vis_icon = if visible { "👁" } else { " " };
                            let lock_icon = if locked { "🔒" } else { " " };
                            let sel_icon = if is_selected { "●" } else { "○" };

                            ui.horizontal(|ui| {
                                if ui
                                    .selectable_label(
                                        is_selected,
                                        format!("{sel_icon} {vis_icon}{lock_icon} {name}"),
                                    )
                                    .clicked()
                                {
                                    action_select = Some(id);
                                }
                                if ui.small_button("👁").clicked() {
                                    let _ = self.shell.bridge.submit_command(CommandRequest::new(
                                        Command::SetVisibility {
                                            id,
                                            visible: !visible,
                                        },
                                    ));
                                }
                                if ui.small_button("🔒").clicked() {
                                    let _ = self.shell.bridge.submit_command(CommandRequest::new(
                                        Command::SetLocked {
                                            id,
                                            locked: !locked,
                                        },
                                    ));
                                }
                                if let Some(s_id) = surf_id {
                                    if idx > 0 && ui.small_button("▲").clicked() {
                                        let _ = self.shell.bridge.submit_command(
                                            CommandRequest::new(Command::ReorderObject {
                                                surface: s_id,
                                                id,
                                                new_index: idx - 1,
                                            }),
                                        );
                                    }
                                    if idx + 1 < total_items && ui.small_button("▼").clicked() {
                                        let _ = self.shell.bridge.submit_command(
                                            CommandRequest::new(Command::ReorderObject {
                                                surface: s_id,
                                                id,
                                                new_index: idx + 1,
                                            }),
                                        );
                                    }
                                }
                                if ui.small_button("🗑").clicked() {
                                    action_delete = Some(id);
                                }
                            });
                        }

                        if let Some(id) = action_select {
                            self.shell.bridge.set_selection(vec![id]);
                        }
                        if let Some(id) = action_delete {
                            let _ = self
                                .shell
                                .bridge
                                .submit_command(CommandRequest::new(Command::DeleteObject { id }));
                            self.shell.bridge.clear_selection();
                        }

                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("+ Retângulo").clicked() {
                                let mut id_gen = IdGenerator::new();
                                if let Some(surface) = self
                                    .shell
                                    .bridge
                                    .session()
                                    .and_then(|s| s.document.surfaces.first().cloned())
                                {
                                    let new_id = id_gen.next_object();
                                    let count = surface.objects.len() + 1;
                                    let offset = (count as f64 * 35.0) % 250.0;
                                    let _ = self.shell.bridge.create_shape_object(
                                        surface.id,
                                        new_id,
                                        format!("Rectangle {}", count),
                                        ShapeKind::Rectangle {
                                            corner_radii: [4.0; 4],
                                        },
                                        Some([120.0 + offset, 120.0 + offset, 200.0, 130.0]),
                                        Some("aubrieta.green/500".to_string()),
                                        Some("#10b981".to_string()),
                                        1.0,
                                    );
                                    self.shell.bridge.set_selection(vec![new_id]);
                                }
                            }
                            if ui.button("+ Elipse").clicked() {
                                let mut id_gen = IdGenerator::new();
                                if let Some(surface) = self
                                    .shell
                                    .bridge
                                    .session()
                                    .and_then(|s| s.document.surfaces.first().cloned())
                                {
                                    let new_id = id_gen.next_object();
                                    let count = surface.objects.len() + 1;
                                    let offset = (count as f64 * 35.0) % 250.0;
                                    let _ = self.shell.bridge.create_shape_object(
                                        surface.id,
                                        new_id,
                                        format!("Circle {}", count),
                                        ShapeKind::Ellipse,
                                        Some([360.0 + offset, 180.0 + offset, 130.0, 130.0]),
                                        Some("aubrieta.yellow/500".to_string()),
                                        Some("#ca8a04".to_string()),
                                        1.5,
                                    );
                                    self.shell.bridge.set_selection(vec![new_id]);
                                }
                            }
                            if ui.button("+ Estrela").clicked() {
                                let mut id_gen = IdGenerator::new();
                                if let Some(surface) = self
                                    .shell
                                    .bridge
                                    .session()
                                    .and_then(|s| s.document.surfaces.first().cloned())
                                {
                                    let new_id = id_gen.next_object();
                                    let count = surface.objects.len() + 1;
                                    let offset = (count as f64 * 35.0) % 250.0;
                                    let _ = self.shell.bridge.create_shape_object(
                                        surface.id,
                                        new_id,
                                        format!("Star {}", count),
                                        ShapeKind::Star {
                                            points: 5,
                                            inner_ratio: 0.45,
                                        },
                                        Some([220.0 + offset, 160.0 + offset, 130.0, 130.0]),
                                        Some("aubrieta.rose/500".to_string()),
                                        Some("#e11d48".to_string()),
                                        1.5,
                                    );
                                    self.shell.bridge.set_selection(vec![new_id]);
                                }
                            }
                            if ui.button("+ Texto").clicked() {
                                let mut id_gen = IdGenerator::new();
                                if let Some(surface) = self
                                    .shell
                                    .bridge
                                    .session()
                                    .and_then(|s| s.document.surfaces.first().cloned())
                                {
                                    let new_id = id_gen.next_object();
                                    let count = surface.objects.len() + 1;
                                    let offset = (count as f64 * 25.0) % 200.0;
                                    let _ = self.shell.bridge.create_shape_object(
                                        surface.id,
                                        new_id,
                                        format!("Text {}", count),
                                        ShapeKind::Text {
                                            content: format!("Texto Vetorial {}", count),
                                            font_family: "Inter".to_string(),
                                            font_size: 18.0,
                                            line_height: 22.0,
                                            letter_spacing: 0.0,
                                        },
                                        Some([140.0 + offset, 240.0 + offset, 220.0, 40.0]),
                                        Some("aubrieta.purple/500".to_string()),
                                        None,
                                        0.0,
                                    );
                                    self.shell.bridge.set_selection(vec![new_id]);
                                }
                            }
                        });
                    }
                    ActiveTab::Properties => {
                        ui.heading("Inspetor de Propriedades");
                        let sel = self.shell.bridge.selection();
                        let selected_obj = if let Some(session) = self.shell.bridge.session() {
                            if let Some(first_id) = sel.selected_ids.first() {
                                session
                                    .document
                                    .surfaces
                                    .iter()
                                    .flat_map(|s| &s.objects)
                                    .find(|o| o.id == *first_id)
                                    .cloned()
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

                                ui.label(egui::RichText::new("TRANSFORMAÇÃO").strong());
                                let mut changed = false;
                                ui.horizontal(|ui| {
                                    changed |= ui
                                        .add(egui::DragValue::new(&mut x).speed(1.0).prefix("X: "))
                                        .changed();
                                    changed |= ui
                                        .add(egui::DragValue::new(&mut y).speed(1.0).prefix("Y: "))
                                        .changed();
                                });
                                ui.horizontal(|ui| {
                                    changed |= ui
                                        .add(egui::DragValue::new(&mut w).speed(1.0).prefix("W: "))
                                        .changed();
                                    changed |= ui
                                        .add(egui::DragValue::new(&mut h).speed(1.0).prefix("H: "))
                                        .changed();
                                });
                                if changed {
                                    let _ = self.shell.bridge.set_bounds(
                                        obj.id,
                                        Some([x, y, w, h]),
                                        0.0,
                                    );
                                }
                            }

                            ui.separator();
                            ui.label(egui::RichText::new("PREENCHIMENTO (FILL)").strong());
                            ui.horizontal(|ui| {
                                let colors = [
                                    (
                                        "#3b82f6",
                                        egui::Color32::from_rgb(59, 130, 246),
                                        "aubrieta.blue/500",
                                    ),
                                    (
                                        "#eab308",
                                        egui::Color32::from_rgb(234, 179, 8),
                                        "aubrieta.yellow/500",
                                    ),
                                    (
                                        "#e11d48",
                                        egui::Color32::from_rgb(225, 29, 72),
                                        "aubrieta.rose/500",
                                    ),
                                    (
                                        "#10b981",
                                        egui::Color32::from_rgb(16, 185, 129),
                                        "aubrieta.green/500",
                                    ),
                                    (
                                        "#8b5cf6",
                                        egui::Color32::from_rgb(139, 92, 246),
                                        "aubrieta.purple/500",
                                    ),
                                ];
                                for (_hex, color, name) in colors {
                                    let (rect, resp) = ui.allocate_exact_size(
                                        egui::vec2(24.0, 24.0),
                                        egui::Sense::click(),
                                    );
                                    ui.painter().rect_filled(rect, 4.0, color);
                                    ui.painter().rect_stroke(
                                        rect,
                                        4.0,
                                        egui::Stroke::new(1.0_f32, egui::Color32::WHITE),
                                        egui::StrokeKind::Middle,
                                    );
                                    if resp.clicked() {
                                        let _ = self
                                            .shell
                                            .bridge
                                            .set_fill(obj.id, Some(name.to_string()));
                                    }
                                }
                            });

                            ui.add_space(4.0);
                            ui.label("OPACIDADE");
                            if ui
                                .add(
                                    egui::Slider::new(&mut self.selected_opacity, 0.0..=100.0)
                                        .suffix("%"),
                                )
                                .changed()
                            {
                                let opacity =
                                    (self.selected_opacity as f64 / 100.0).clamp(0.0, 1.0);
                                let _ = self.shell.bridge.submit_command(CommandRequest::new(
                                    Command::SetOpacity {
                                        id: obj.id,
                                        opacity,
                                    },
                                ));
                            }

                            ui.separator();
                            ui.label(egui::RichText::new("BOOLEANOS & PATHFINDER").strong());
                            ui.horizontal(|ui| {
                                if ui.button("⋃ Unir").clicked() {
                                    self.apply_boolean(BooleanOp::Union);
                                }
                                if ui.button("− Sub").clicked() {
                                    self.apply_boolean(BooleanOp::Difference);
                                }
                                if ui.button("⋂ Inter").clicked() {
                                    self.apply_boolean(BooleanOp::Intersection);
                                }
                                if ui.button("⨁ Xor").clicked() {
                                    self.apply_boolean(BooleanOp::Xor);
                                }
                            });

                            ui.add_space(6.0);
                            if ui.button("🗑 Excluir Objeto").clicked() {
                                let _ = self.shell.bridge.submit_command(CommandRequest::new(
                                    Command::DeleteObject { id: obj.id },
                                ));
                                self.shell.bridge.clear_selection();
                            }
                        } else {
                            ui.label(
                                egui::RichText::new("(Nenhum objeto selecionado)")
                                    .italics()
                                    .color(egui::Color32::GRAY),
                            );
                        }
                    }
                    ActiveTab::History => {
                        ui.heading("Histórico de Transações");
                        let model: HistoryPresentationModel = self.shell.bridge.query_history();
                        ui.label(format!(
                            "Undo: {} | Redo: {}",
                            model.undo_stack.len(),
                            model.redo_stack.len()
                        ));
                        ui.separator();
                        for (i, item) in model.undo_stack.iter().enumerate().rev() {
                            ui.label(format!("[#{}] {}", i + 1, item.description));
                        }
                    }
                    ActiveTab::DataMerge => {
                        ui.heading("Data Merge de Variáveis");
                        let model: DataMergePresentationModel = self.shell.query_data_merge();
                        let src = model
                            .sources
                            .first()
                            .map(|s| s.name.as_str())
                            .unwrap_or("(Nenhuma fonte)");
                        ui.label(format!("Fonte: {src}"));
                        ui.label(format!(
                            "Registros: {} | Vínculos: {}",
                            model.total_records,
                            model.bindings.len()
                        ));
                        ui.colored_label(egui::Color32::GREEN, "Status: Pronto para Mesclagem");
                    }
                }
            });

        // -------------------------------------------------------------
        // SECTION 5: CANONICAL STATUS BAR (08.2, 08.23)
        // -------------------------------------------------------------
        egui::TopBottomPanel::bottom("bottom_status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let hint = match self.shell.active_tool() {
                    ToolKind::Select => "Select: Clique para selecionar, arraste para mover | Shift: Multi-seleção | Alt: Duplicar",
                    ToolKind::Node => "Node: Clique e arraste pontos de controle e alças Bézier para ajustar curvas.",
                    ToolKind::Pen => "Pen: Clique para criar nós angulares, arraste para nós suaves com tangentes.",
                    ToolKind::Pencil => "Pencil: Desenho vetorial à mão livre com suavização dinâmica.",
                    ToolKind::Rectangle => "Rectangle: Clique e arraste para desenhar retângulos e quadrados.",
                    ToolKind::Ellipse => "Ellipse: Clique e arraste para desenhar elipses ou círculos perfeitos.",
                    ToolKind::Star => "Star: Desenha estrelas vetoriais com raio interno personalizável.",
                    ToolKind::Polygon => "Polygon: Desenha polígonos regulares configuráveis.",
                    ToolKind::ArtisticText | ToolKind::FrameText => "Text: Clique no canvas para criar caixa de texto vetorial.",
                    ToolKind::Gradient => "Gradient: Arraste sobre o objeto para definir gradiente linear ou radial.",
                    ToolKind::ColorPicker => "Color Picker: Clique em qualquer elemento para capturar cor.",
                    ToolKind::Knife | ToolKind::Scissors => "Knife/Scissors: Fatie formas e caminhos vetoriais com uma linha de corte.",
                    ToolKind::ShapeBuilder => "Shape Builder: Combine ou subtraia regiões sobrepostas.",
                    ToolKind::PointTransform => "Point Transform: Transformações afins livres com pivô customizado.",
                    ToolKind::Artboard => "Artboard: Redimensione ou crie novas pranchetas de trabalho.",
                    ToolKind::Hand => "Hand: Arraste para navegar pelo espaço infinito da prancheta.",
                    ToolKind::Zoom => "Zoom: Clique para ampliar, Alt+Clique para reduzir o zoom.",
                    _ => "Aubrieta Studio: Ferramenta pronta para uso.",
                };
                ui.label(egui::RichText::new(hint).color(egui::Color32::from_rgb(161, 161, 170)));
                ui.separator();
                if let Some(pt) = self.hovered_doc_point {
                    ui.label(egui::RichText::new(format!("Doc: X: {:.1} pt  Y: {:.1} pt", pt.x, pt.y)).color(egui::Color32::from_rgb(147, 197, 253)));
                } else {
                    ui.label("Cursor fora do canvas");
                }
                ui.separator();
                let zoom_pct = (self.shell.camera.zoom * 100.0).round() as i32;
                let sel_count = self.shell.bridge.selection().count;
                let snap = if self.shell.snap.config.grid_enabled { "Ativo" } else { "Inativo" };
                ui.label(format!("{sel_count} selecionado(s) | Zoom: {zoom_pct}% | Snap: {snap}"));
            });
        });

        // -------------------------------------------------------------
        // SECTION 4.2: CENTRAL CANVAS VIEWPORT (08.6)
        // -------------------------------------------------------------
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
            let canvas_rect = response.rect;

            // Background canvas dark color
            painter.rect_filled(canvas_rect, 0.0, egui::Color32::from_rgb(17, 17, 19));

            let camera = self.shell.camera.clone();
            let to_screen = |doc_pt: GPoint| -> egui::Pos2 {
                let s = camera.doc_to_screen(doc_pt);
                egui::pos2(
                    canvas_rect.min.x + s.x as f32,
                    canvas_rect.min.y + s.y as f32,
                )
            };

            // Zoom on mouse scroll wheel
            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
            if scroll_delta != 0.0 && response.hovered() {
                let factor = if scroll_delta > 0.0 { 1.15 } else { 0.85 };
                if let Some(pos) = response.hover_pos() {
                    let screen_focus = GPoint::new(
                        (pos.x - canvas_rect.min.x) as f64,
                        (pos.y - canvas_rect.min.y) as f64,
                    );
                    self.shell.zoom_at(screen_focus, factor);
                }
            }

            // Pan on middle mouse button drag or Space+drag
            let is_panning = ui.input(|i| {
                i.pointer.button_down(egui::PointerButton::Middle)
                    || (i.key_down(egui::Key::Space) && i.pointer.primary_down())
            });
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
                self.hovered_doc_point = Some(
                    self.shell
                        .camera
                        .screen_to_doc(GPoint::new(local_x, local_y)),
                );
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
                    let shadow_rect = paper_rect.translate(egui::vec2(6.0, 6.0));
                    painter.rect_filled(shadow_rect, 4.0, egui::Color32::from_black_alpha(120));

                    // Artboard Paper White
                    painter.rect_filled(paper_rect, 0.0, egui::Color32::WHITE);
                    painter.rect_stroke(
                        paper_rect,
                        0.0,
                        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(203, 213, 225)),
                        egui::StrokeKind::Middle,
                    );

                    // Artboard Name header
                    painter.text(
                        egui::pos2(paper_rect.min.x, paper_rect.min.y - 16.0),
                        egui::Align2::LEFT_TOP,
                        format!("📄 {} [{:.0} × {:.0} pt]", surface.name, b[2], b[3]),
                        egui::FontId::proportional(11.0),
                        egui::Color32::GRAY,
                    );

                    // Bleed guide line (magenta) (10.7)
                    if !surface.bleed.is_zero() {
                        let bp0 = to_screen(GPoint::new(
                            b[0] - surface.bleed.left,
                            b[1] - surface.bleed.top,
                        ));
                        let bp1 = to_screen(GPoint::new(
                            b[0] + b[2] + surface.bleed.right,
                            b[1] + b[3] + surface.bleed.bottom,
                        ));
                        painter.rect_stroke(
                            egui::Rect::from_two_pos(bp0, bp1),
                            0.0,
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(244, 63, 94)),
                            egui::StrokeKind::Middle,
                        );
                    }

                    // Margin guide line (cyan) (10.7)
                    if surface.margins != Margins::ZERO {
                        let mp0 = to_screen(GPoint::new(
                            b[0] + surface.margins.left,
                            b[1] + surface.margins.top,
                        ));
                        let mp1 = to_screen(GPoint::new(
                            b[0] + b[2] - surface.margins.right,
                            b[1] + b[3] - surface.margins.bottom,
                        ));
                        painter.rect_stroke(
                            egui::Rect::from_two_pos(mp0, mp1),
                            0.0,
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(6, 182, 212)),
                            egui::StrokeKind::Middle,
                        );
                    }

                    // Vertical guide
                    let gp0 = to_screen(GPoint::new(b[0] + 200.0, b[1]));
                    let gp1 = to_screen(GPoint::new(b[0] + 200.0, b[1] + b[3]));
                    painter.line_segment(
                        [gp0, gp1],
                        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(6, 182, 212)),
                    );

                    // Render objects
                    for obj in &surface.objects {
                        if !obj.visible {
                            continue;
                        }
                        let obj_b = obj.bounds.unwrap_or([0.0, 0.0, 100.0, 80.0]);
                        let op0 = to_screen(GPoint::new(obj_b[0], obj_b[1]));
                        let op1 = to_screen(GPoint::new(obj_b[0] + obj_b[2], obj_b[1] + obj_b[3]));
                        let obj_rect = egui::Rect::from_two_pos(op0, op1);
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
                                egui::Color32::from_rgb(225, 29, 72)
                            } else {
                                egui::Color32::from_rgb(59, 130, 246)
                            }
                        } else {
                            egui::Color32::from_rgb(59, 130, 246)
                        };

                        match &obj.shape {
                            Some(ShapeKind::Ellipse) => {
                                painter.rect_filled(obj_rect, obj_rect.width() / 2.0, color);
                                painter.rect_stroke(
                                    obj_rect,
                                    obj_rect.width() / 2.0,
                                    egui::Stroke::new(
                                        1.5_f32,
                                        egui::Color32::from_rgb(202, 138, 4),
                                    ),
                                    egui::StrokeKind::Middle,
                                );
                            }
                            Some(ShapeKind::Star { inner_ratio, .. }) => {
                                let pts = make_star_points(
                                    obj_rect.center(),
                                    obj_rect.width() / 2.0,
                                    (obj_rect.width() / 2.0) * (*inner_ratio as f32),
                                    5,
                                );
                                painter.add(egui::Shape::convex_polygon(
                                    pts,
                                    color,
                                    egui::Stroke::new(
                                        1.5_f32,
                                        egui::Color32::from_rgb(225, 29, 72),
                                    ),
                                ));
                            }
                            Some(ShapeKind::Text {
                                content, font_size, ..
                            }) => {
                                painter.text(
                                    obj_rect.left_center(),
                                    egui::Align2::LEFT_CENTER,
                                    content,
                                    egui::FontId::proportional(*font_size as f32),
                                    color,
                                );
                            }
                            _ => {
                                painter.rect_filled(obj_rect, 4.0, color);
                                painter.rect_stroke(
                                    obj_rect,
                                    4.0,
                                    egui::Stroke::new(
                                        1.5_f32,
                                        egui::Color32::from_rgb(37, 99, 235),
                                    ),
                                    egui::StrokeKind::Middle,
                                );
                            }
                        }

                        // Selection Affordances (8 Handles + Rotation Pin + HUD) (10.1)
                        if is_selected {
                            painter.rect_stroke(
                                obj_rect,
                                4.0,
                                egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(59, 130, 246)),
                                egui::StrokeKind::Middle,
                            );
                            let handles = [
                                obj_rect.left_top(),
                                egui::pos2(obj_rect.center().x, obj_rect.top()),
                                obj_rect.right_top(),
                                egui::pos2(obj_rect.right(), obj_rect.center().y),
                                obj_rect.right_bottom(),
                                egui::pos2(obj_rect.center().x, obj_rect.bottom()),
                                obj_rect.left_bottom(),
                                egui::pos2(obj_rect.left(), obj_rect.center().y),
                            ];
                            for handle_pos in handles {
                                painter.rect_filled(
                                    egui::Rect::from_center_size(handle_pos, egui::vec2(8.0, 8.0)),
                                    1.0,
                                    egui::Color32::WHITE,
                                );
                                painter.rect_stroke(
                                    egui::Rect::from_center_size(handle_pos, egui::vec2(8.0, 8.0)),
                                    1.0_f32,
                                    egui::Stroke::new(
                                        1.5_f32,
                                        egui::Color32::from_rgb(37, 99, 235),
                                    ),
                                    egui::StrokeKind::Middle,
                                );
                            }

                            // Rotation Pin
                            let pin_top = egui::pos2(obj_rect.center().x, obj_rect.top() - 16.0);
                            painter.line_segment(
                                [egui::pos2(obj_rect.center().x, obj_rect.top()), pin_top],
                                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(59, 130, 246)),
                            );
                            painter.circle_filled(
                                egui::pos2(obj_rect.center().x, obj_rect.top() - 18.0),
                                4.0,
                                egui::Color32::from_rgb(59, 130, 246),
                            );
                            painter.circle_stroke(
                                egui::pos2(obj_rect.center().x, obj_rect.top() - 18.0),
                                4.0,
                                egui::Stroke::new(1.0_f32, egui::Color32::WHITE),
                            );

                            // Dimensions HUD Badge
                            let hud_rect = egui::Rect::from_center_size(
                                egui::pos2(obj_rect.center().x, obj_rect.bottom() + 14.0),
                                egui::vec2(110.0, 18.0),
                            );
                            painter.rect_filled(
                                hud_rect,
                                4.0,
                                egui::Color32::from_black_alpha(220),
                            );
                            painter.text(
                                hud_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("{:.0} × {:.0} pt", obj_b[2], obj_b[3]),
                                egui::FontId::proportional(10.0),
                                egui::Color32::from_rgb(147, 197, 253),
                            );
                        }
                    }
                }
            }

            // Interactive Drag Creation Preview
            if let Some(prev) = self.preview_rect {
                let p0 = to_screen(GPoint::new(prev[0], prev[1]));
                let p1 = to_screen(GPoint::new(prev[0] + prev[2], prev[1] + prev[3]));
                let preview_box = egui::Rect::from_two_pos(p0, p1);
                painter.rect_filled(
                    preview_box,
                    4.0,
                    egui::Color32::from_rgba_unmultiplied(59, 130, 246, 40),
                );
                painter.rect_stroke(
                    preview_box,
                    4.0,
                    egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(59, 130, 246)),
                    egui::StrokeKind::Middle,
                );
            }

            // Interactive Pointer Events inside canvas
            if !is_panning {
                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let screen_pt = GPoint::new(
                            (pos.x - canvas_rect.min.x) as f64,
                            (pos.y - canvas_rect.min.y) as f64,
                        );
                        let doc_pt = self.shell.camera.screen_to_doc(screen_pt);

                        // Hit test objects
                        let mut hit = None;
                        if let Some(session) = self.shell.bridge.session() {
                            if let Some(surface) = session.document.surfaces.first() {
                                for obj in surface.objects.iter().rev() {
                                    if let Some(b) = obj.bounds {
                                        if doc_pt.x >= b[0]
                                            && doc_pt.x <= b[0] + b[2]
                                            && doc_pt.y >= b[1]
                                            && doc_pt.y <= b[1] + b[3]
                                        {
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
                        let screen_pt = GPoint::new(
                            (pos.x - canvas_rect.min.x) as f64,
                            (pos.y - canvas_rect.min.y) as f64,
                        );
                        let doc_pt = self.shell.camera.screen_to_doc(screen_pt);
                        self.drag_start_doc = Some(doc_pt);

                        let mut hit = None;
                        let mut hit_b = None;
                        if let Some(session) = self.shell.bridge.session() {
                            if let Some(surface) = session.document.surfaces.first() {
                                for obj in surface.objects.iter().rev() {
                                    if let Some(b) = obj.bounds {
                                        if doc_pt.x >= b[0]
                                            && doc_pt.x <= b[0] + b[2]
                                            && doc_pt.y >= b[1]
                                            && doc_pt.y <= b[1] + b[3]
                                        {
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

                        let evt = NormalizedPointerEvent::new(
                            PointerPhase::Down,
                            PointerButton::Primary,
                            screen_pt,
                            doc_pt,
                            SemanticModifiers::default(),
                        );
                        let _ = self.shell.handle_pointer_event(&evt);
                    }
                } else if response.dragged() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let screen_pt = GPoint::new(
                            (pos.x - canvas_rect.min.x) as f64,
                            (pos.y - canvas_rect.min.y) as f64,
                        );
                        let doc_pt = self.shell.camera.screen_to_doc(screen_pt);

                        if let Some(start) = self.drag_start_doc {
                            match self.shell.active_tool() {
                                ToolKind::Rectangle
                                | ToolKind::Ellipse
                                | ToolKind::Star
                                | ToolKind::Polygon
                                | ToolKind::Pen => {
                                    let min_x = start.x.min(doc_pt.x);
                                    let min_y = start.y.min(doc_pt.y);
                                    let w = (doc_pt.x - start.x).abs();
                                    let h = (doc_pt.y - start.y).abs();
                                    self.preview_rect = Some([min_x, min_y, w, h]);
                                }
                                ToolKind::Select => {
                                    if let (Some(id), Some(init_b)) =
                                        (self.dragging_object_id, self.drag_initial_bounds)
                                    {
                                        let dx = doc_pt.x - start.x;
                                        let dy = doc_pt.y - start.y;
                                        let new_b =
                                            [init_b[0] + dx, init_b[1] + dy, init_b[2], init_b[3]];
                                        let _ = self.shell.bridge.set_bounds(id, Some(new_b), 0.0);
                                    }
                                }
                                _ => {}
                            }
                        }

                        let evt = NormalizedPointerEvent::new(
                            PointerPhase::Move,
                            PointerButton::Primary,
                            screen_pt,
                            doc_pt,
                            SemanticModifiers::default(),
                        );
                        let _ = self.shell.handle_pointer_event(&evt);
                    }
                } else if response.drag_stopped() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let screen_pt = GPoint::new(
                            (pos.x - canvas_rect.min.x) as f64,
                            (pos.y - canvas_rect.min.y) as f64,
                        );
                        let doc_pt = self.shell.camera.screen_to_doc(screen_pt);
                        self.preview_rect = None;

                        self.drag_start_doc = None;
                        self.dragging_object_id = None;
                        self.drag_initial_bounds = None;

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
        });
    }
}

fn make_star_points(
    center: egui::Pos2,
    r_outer: f32,
    r_inner: f32,
    points: usize,
) -> Vec<egui::Pos2> {
    let mut pts = Vec::with_capacity(points * 2);
    let step = std::f32::consts::PI / points as f32;
    for i in 0..(points * 2) {
        let r = if i % 2 == 0 { r_outer } else { r_inner };
        let angle = i as f32 * step - std::f32::consts::FRAC_PI_2;
        let x = center.x + r * angle.cos();
        let y = center.y + r * angle.sin();
        pts.push(egui::pos2(x, y));
    }
    pts
}

fn populate_showcase_document(shell: &mut AubrietaShell) -> Result<(), AubrietaError> {
    shell.new_document("Aubrieta Showcase Project [egui]")?;
    let mut id_gen = IdGenerator::new();
    let surface_1 = id_gen.next_surface();
    let rect_id = id_gen.next_object();
    let circle_id = id_gen.next_object();
    let star_id = id_gen.next_object();
    let text_id = id_gen.next_object();

    shell
        .bridge
        .submit_command(CommandRequest::new(Command::CreateSurface {
            id: surface_1,
            name: "Main Artboard".to_string(),
        }))?;
    shell
        .bridge
        .set_surface_geometry(surface_1, [60.0, 60.0], [800.0, 600.0])?;
    shell
        .bridge
        .set_surface_bleed(surface_1, Bleed::uniform(10.0))?;
    shell
        .bridge
        .set_surface_margins(surface_1, Margins::uniform(36.0))?;
    shell
        .bridge
        .add_surface_guide(surface_1, Guide::new(1, GuideOrientation::Vertical, 200.0))?;

    // Hero Card (Rectangle)
    shell.bridge.create_shape_object(
        surface_1,
        rect_id,
        "Hero Card".to_string(),
        ShapeKind::Rectangle {
            corner_radii: [8.0; 4],
        },
        Some([100.0, 100.0, 300.0, 180.0]),
        Some("aubrieta.blue/500".to_string()),
        Some("#2563eb".to_string()),
        1.5,
    )?;

    // Accent Circle (Ellipse)
    shell.bridge.create_shape_object(
        surface_1,
        circle_id,
        "Accent Circle".to_string(),
        ShapeKind::Ellipse,
        Some([450.0, 140.0, 140.0, 140.0]),
        Some("aubrieta.yellow/500".to_string()),
        Some("#ca8a04".to_string()),
        1.5,
    )?;

    // Golden Star (Star)
    shell.bridge.create_shape_object(
        surface_1,
        star_id,
        "Golden Star".to_string(),
        ShapeKind::Star {
            points: 5,
            inner_ratio: 0.45,
        },
        Some([450.0, 320.0, 130.0, 130.0]),
        Some("aubrieta.rose/500".to_string()),
        Some("#e11d48".to_string()),
        1.5,
    )?;

    // Banner Text (Artistic Text)
    shell.bridge.create_shape_object(
        surface_1,
        text_id,
        "Banner Text".to_string(),
        ShapeKind::Text {
            content: "Aubrieta Vector Studio".to_string(),
            font_family: "Inter".to_string(),
            font_size: 20.0,
            line_height: 24.0,
            letter_spacing: 0.5,
        },
        Some([100.0, 320.0, 300.0, 50.0]),
        Some("aubrieta.purple/500".to_string()),
        None,
        0.0,
    )?;

    shell.bridge.set_selection(vec![rect_id]);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args
        .iter()
        .any(|a| a == "--smoke-test" || a == "--headless")
    {
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
            .with_title("Aubrieta Creative Studio — egui")
            .with_inner_size([1440.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Aubrieta Creative Studio",
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
