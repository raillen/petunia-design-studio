//! Aubrieta Creative Studio — Iced Canonical GUI.
//!
//! Evaluates Iced Elm architecture and Canvas graphics, consuming
//! AubrietaGuiBridge presentation models and dispatching canonical commands
//! matching the Affinity Designer Dark Studio specifications.

use std::env;

use aubrieta_application::{Command, CommandRequest};
use aubrieta_document::{
    AlignmentMode, Bleed, DistributionAxis, Guide, GuideOrientation, Margins, ShapeKind,
};
use aubrieta_foundation::{AubrietaError, IdGenerator, ObjectId};
use aubrieta_geometry::{BooleanOp, GPath, GPoint, GRect};
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
use iced::widget::{button, checkbox, column, container, row, scrollable, slider, text, Column};
use iced::{Alignment, Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Persona {
    Design,
    Photo,
}

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
    SwitchPersona(Persona),
    Undo,
    Redo,
    ZoomIn,
    ZoomOut,
    FitCanvas,
    NewDocument,
    OpenDocument,
    SaveDocument,
    ToggleSnap(bool),
    SelectTab(ActiveTab),
    SetOpacity(f32),
    SelectLayer(usize),
    ToggleLayerVisibility(usize),
    ToggleLayerLock(usize),
    ReorderLayerUp(usize),
    ReorderLayerDown(usize),
    AddRectangle,
    AddCircle,
    AddStar,
    AddText,
    DeleteSelected,
    DuplicateSelected,
    ConvertToCurves,
    BakeCorners,
    BooleanUnion,
    BooleanSubtract,
    BooleanIntersect,
    BooleanXor,
    AlignObjects(AlignmentMode),
    DistributeObjects(DistributionAxis),
    SetColor(String),
    CanvasPointerDown(f32, f32),
    CanvasPointerMoved(f32, f32),
    CanvasPointerUp(f32, f32),
}

pub struct AubrietaIcedApp {
    pub shell: AubrietaShell,
    pub active_persona: Persona,
    pub active_tab: ActiveTab,
    pub hovered_doc_point: Option<GPoint>,
    pub drag_start_doc: Option<GPoint>,
    pub dragging_object_id: Option<ObjectId>,
    pub drag_initial_bounds: Option<[f64; 4]>,
    pub selected_opacity: f32,
    pub preview_rect: Option<[f64; 4]>,
}

impl AubrietaIcedApp {
    pub fn new() -> Self {
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
            selected_opacity: 100.0,
            preview_rect: None,
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
            Message::SwitchPersona(persona) => {
                self.active_persona = persona;
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
                if let Some(surface) = self
                    .shell
                    .bridge
                    .session()
                    .and_then(|s| s.document.surfaces.first())
                {
                    let b = surface.bounds();
                    self.shell.fit_surface(GRect::new(b[0], b[1], b[0] + b[2], b[1] + b[3]));
                }
            }
            Message::NewDocument => {
                let _ = self.shell.new_document("Sem Título");
            }
            Message::OpenDocument => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Aubrieta Design (*.aub)", &["aub"])
                    .add_filter("Gráficos Vetoriais SVG (*.svg)", &["svg"])
                    .add_filter("Todos os arquivos (*.*)", &["*"])
                    .set_title("Abrir Documento Aubrieta")
                    .pick_file()
                {
                    let title = path.file_name().and_then(|n| n.to_str()).unwrap_or("Novo Documento");
                    let _ = self.shell.new_document(title);
                }
            }
            Message::SaveDocument => {
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
                if let Some(sel_id) = self.shell.bridge.selection().selected_ids.first().copied() {
                    let opacity = (val as f64 / 100.0).clamp(0.0, 1.0);
                    let _ = self.shell.bridge.submit_command(CommandRequest::new(Command::SetOpacity {
                        id: sel_id,
                        opacity,
                    }));
                }
            }
            Message::SelectLayer(idx) => {
                if let Some(session) = self.shell.bridge.session() {
                    if let Some(surface) = session.document.surfaces.first() {
                        if let Some(obj) = surface.objects.get(idx) {
                            self.shell.bridge.set_selection(vec![obj.id]);
                        }
                    }
                }
            }
            Message::ToggleLayerVisibility(idx) => {
                let toggle = if let Some(session) = self.shell.bridge.session() {
                    if let Some(surface) = session.document.surfaces.first() {
                        surface.objects.get(idx).map(|obj| (obj.id, !obj.visible))
                    } else {
                        None
                    }
                } else {
                    None
                };
                if let Some((id, visible)) = toggle {
                    let _ = self.shell.bridge.submit_command(CommandRequest::new(Command::SetVisibility {
                        id,
                        visible,
                    }));
                }
            }
            Message::ToggleLayerLock(idx) => {
                let toggle = if let Some(session) = self.shell.bridge.session() {
                    if let Some(surface) = session.document.surfaces.first() {
                        surface.objects.get(idx).map(|obj| (obj.id, !obj.locked))
                    } else {
                        None
                    }
                } else {
                    None
                };
                if let Some((id, locked)) = toggle {
                    let _ = self.shell.bridge.submit_command(CommandRequest::new(Command::SetLocked {
                        id,
                        locked,
                    }));
                }
            }
            Message::ReorderLayerUp(idx) => {
                if idx > 0 {
                    if let Some(session) = self.shell.bridge.session() {
                        if let Some(surface) = session.document.surfaces.first() {
                            if let Some(obj) = surface.objects.get(idx) {
                                let _ = self.shell.bridge.submit_command(CommandRequest::new(
                                    Command::ReorderObject {
                                        surface: surface.id,
                                        id: obj.id,
                                        new_index: idx - 1,
                                    },
                                ));
                            }
                        }
                    }
                }
            }
            Message::ReorderLayerDown(idx) => {
                if let Some(session) = self.shell.bridge.session() {
                    if let Some(surface) = session.document.surfaces.first() {
                        if idx + 1 < surface.objects.len() {
                            if let Some(obj) = surface.objects.get(idx) {
                                let _ = self.shell.bridge.submit_command(CommandRequest::new(
                                    Command::ReorderObject {
                                        surface: surface.id,
                                        id: obj.id,
                                        new_index: idx + 1,
                                    },
                                ));
                            }
                        }
                    }
                }
            }
            Message::AddRectangle => {
                let mut id_gen = IdGenerator::new();
                if let Some(surface) = self.shell.bridge.session().and_then(|s| s.document.surfaces.first().cloned()) {
                    let new_id = id_gen.next_object();
                    let count = surface.objects.len() + 1;
                    let offset = (count as f64 * 35.0) % 250.0;
                    let _ = self.shell.bridge.create_shape_object(
                        surface.id,
                        new_id,
                        format!("Rectangle {}", count),
                        ShapeKind::Rectangle { corner_radii: [4.0; 4] },
                        Some([120.0 + offset, 120.0 + offset, 200.0, 130.0]),
                        Some("aubrieta.green/500".to_string()),
                        Some("#10b981".to_string()),
                        1.0,
                    );
                    self.shell.bridge.set_selection(vec![new_id]);
                }
            }
            Message::AddCircle => {
                let mut id_gen = IdGenerator::new();
                if let Some(surface) = self.shell.bridge.session().and_then(|s| s.document.surfaces.first().cloned()) {
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
            Message::AddStar => {
                let mut id_gen = IdGenerator::new();
                if let Some(surface) = self.shell.bridge.session().and_then(|s| s.document.surfaces.first().cloned()) {
                    let new_id = id_gen.next_object();
                    let count = surface.objects.len() + 1;
                    let offset = (count as f64 * 35.0) % 250.0;
                    let _ = self.shell.bridge.create_shape_object(
                        surface.id,
                        new_id,
                        format!("Star {}", count),
                        ShapeKind::Star { points: 5, inner_ratio: 0.45 },
                        Some([220.0 + offset, 160.0 + offset, 130.0, 130.0]),
                        Some("aubrieta.rose/500".to_string()),
                        Some("#e11d48".to_string()),
                        1.5,
                    );
                    self.shell.bridge.set_selection(vec![new_id]);
                }
            }
            Message::AddText => {
                let mut id_gen = IdGenerator::new();
                if let Some(surface) = self.shell.bridge.session().and_then(|s| s.document.surfaces.first().cloned()) {
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
            Message::DeleteSelected => {
                let sel = self.shell.bridge.selection();
                for id in sel.selected_ids {
                    let _ = self.shell.bridge.submit_command(CommandRequest::new(Command::DeleteObject { id }));
                }
                self.shell.bridge.clear_selection();
            }
            Message::DuplicateSelected => {
                if let Some(sel_id) = self.shell.bridge.selection().selected_ids.first().copied() {
                    if let Some(session) = self.shell.bridge.session() {
                        if let Some(surface) = session.document.surfaces.first() {
                            if let Some(obj) = surface.objects.iter().find(|o| o.id == sel_id) {
                                let b = obj.bounds.unwrap_or([100.0, 100.0, 100.0, 100.0]);
                                let clone_bounds = [b[0] + 20.0, b[1] + 20.0, b[2], b[3]];
                                let fill = obj.fill.clone();
                                let name = format!("{} (cópia)", obj.name);
                                let mut id_gen = IdGenerator::new();
                                let clone_id = id_gen.next_object();
                                let _ = self.shell.bridge.submit_command(CommandRequest::new(Command::CreateObject {
                                    surface: surface.id,
                                    id: clone_id,
                                    name,
                                }));
                                let _ = self.shell.bridge.set_bounds(clone_id, Some(clone_bounds), 0.0);
                                if let Some(f) = fill {
                                    let _ = self.shell.bridge.set_fill(clone_id, Some(f));
                                }
                                self.shell.bridge.set_selection(vec![clone_id]);
                            }
                        }
                    }
                }
            }
            Message::ConvertToCurves => {
                let sel = self.shell.bridge.selection().selected_ids.clone();
                for id in sel {
                    let _ = self.shell.bridge.convert_to_curves(id);
                }
            }
            Message::BakeCorners => {
                let sel = self.shell.bridge.selection().selected_ids.clone();
                for id in sel {
                    let _ = self.shell.bridge.bake_corners(id);
                }
            }
            Message::BooleanUnion => {
                self.apply_boolean(BooleanOp::Union);
            }
            Message::BooleanSubtract => {
                self.apply_boolean(BooleanOp::Difference);
            }
            Message::BooleanIntersect => {
                self.apply_boolean(BooleanOp::Intersection);
            }
            Message::BooleanXor => {
                self.apply_boolean(BooleanOp::Xor);
            }
            Message::AlignObjects(mode) => {
                let sel = self.shell.bridge.selection().selected_ids.clone();
                if !sel.is_empty() {
                    if let Some(surf_id) = self.shell.bridge.active_surface().or_else(|| {
                        self.shell.bridge.session().and_then(|s| s.document.surfaces.first().map(|sf| sf.id))
                    }) {
                        let _ = self.shell.bridge.align_objects(surf_id, sel, mode);
                    }
                }
            }
            Message::DistributeObjects(axis) => {
                let sel = self.shell.bridge.selection().selected_ids.clone();
                if sel.len() >= 2 {
                    if let Some(surf_id) = self.shell.bridge.active_surface().or_else(|| {
                        self.shell.bridge.session().and_then(|s| s.document.surfaces.first().map(|sf| sf.id))
                    }) {
                        let _ = self.shell.bridge.distribute_objects(surf_id, sel, axis);
                    }
                }
            }
            Message::SetColor(color_token) => {
                let sel = self.shell.bridge.selection();
                for id in sel.selected_ids {
                    let _ = self.shell.bridge.set_fill(id, Some(color_token.clone()));
                }
            }
            Message::CanvasPointerDown(x, y) => {
                let screen_pt = GPoint::new(x as f64, y as f64);
                let doc_pt = self.shell.camera.screen_to_doc(screen_pt);
                self.drag_start_doc = Some(doc_pt);
                self.hovered_doc_point = Some(doc_pt);

                // Hit test objects
                let mut hit_obj = None;
                let mut hit_bounds = None;
                if let Some(session) = self.shell.bridge.session() {
                    if let Some(surface) = session.document.surfaces.first() {
                        for obj in surface.objects.iter().rev() {
                            if let Some(b) = obj.bounds {
                                if doc_pt.x >= b[0]
                                    && doc_pt.x <= b[0] + b[2]
                                    && doc_pt.y >= b[1]
                                    && doc_pt.y <= b[1] + b[3]
                                {
                                    hit_obj = Some(obj.id);
                                    hit_bounds = Some(b);
                                    break;
                                }
                            }
                        }
                    }
                }

                if let Some(id) = hit_obj {
                    self.shell.bridge.set_selection(vec![id]);
                    self.dragging_object_id = Some(id);
                    self.drag_initial_bounds = hit_bounds;
                } else if self.shell.active_tool() == ToolKind::Select {
                    self.shell.bridge.clear_selection();
                    self.dragging_object_id = None;
                    self.drag_initial_bounds = None;
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
            Message::CanvasPointerMoved(x, y) => {
                let screen_pt = GPoint::new(x as f64, y as f64);
                let doc_pt = self.shell.camera.screen_to_doc(screen_pt);
                self.hovered_doc_point = Some(doc_pt);

                if let Some(start_doc) = self.drag_start_doc {
                    match self.shell.active_tool() {
                        ToolKind::Rectangle
                        | ToolKind::Ellipse
                        | ToolKind::Polygon
                        | ToolKind::Star
                        | ToolKind::Pen => {
                            let min_x = start_doc.x.min(doc_pt.x);
                            let min_y = start_doc.y.min(doc_pt.y);
                            let w = (doc_pt.x - start_doc.x).abs();
                            let h = (doc_pt.y - start_doc.y).abs();
                            self.preview_rect = Some([min_x, min_y, w, h]);
                        }
                        ToolKind::Select => {
                            if let (Some(obj_id), Some(init_b)) =
                                (self.dragging_object_id, self.drag_initial_bounds)
                            {
                                let dx = doc_pt.x - start_doc.x;
                                let dy = doc_pt.y - start_doc.y;
                                let new_b = [init_b[0] + dx, init_b[1] + dy, init_b[2], init_b[3]];
                                let _ = self.shell.bridge.set_bounds(obj_id, Some(new_b), 0.0);
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
            Message::CanvasPointerUp(x, y) => {
                let screen_pt = GPoint::new(x as f64, y as f64);
                let doc_pt = self.shell.camera.screen_to_doc(screen_pt);
                self.preview_rect = None;

                if let Some(start_doc) = self.drag_start_doc.take() {
                    let dx = (doc_pt.x - start_doc.x).abs();
                    let dy = (doc_pt.y - start_doc.y).abs();
                    if dx > 8.0 && dy > 8.0 {
                        let min_x = start_doc.x.min(doc_pt.x);
                        let min_y = start_doc.y.min(doc_pt.y);
                        let mut id_gen = IdGenerator::new();
                        if let Some(surface) = self
                            .shell
                            .bridge
                            .session()
                            .and_then(|s| s.document.surfaces.first().cloned())
                        {
                            let new_id = id_gen.next_object();
                            let count = surface.objects.len() + 1;
                            let (name, shape, fill) = match self.shell.active_tool() {
                                ToolKind::Ellipse => (
                                    format!("Ellipse {}", count),
                                    ShapeKind::Ellipse,
                                    "aubrieta.yellow/500",
                                ),
                                ToolKind::Star => (
                                    format!("Star {}", count),
                                    ShapeKind::Star { points: 5, inner_ratio: 0.45 },
                                    "aubrieta.rose/500",
                                ),
                                ToolKind::Polygon => (
                                    format!("Polygon {}", count),
                                    ShapeKind::Polygon { sides: 6 },
                                    "aubrieta.blue/500",
                                ),
                                ToolKind::Pen => (
                                    format!("Path {}", count),
                                    ShapeKind::Path(GPath::rect(
                                        GRect::new(min_x, min_y, min_x + dx, min_y + dy),
                                        0.0,
                                        0.0,
                                    )),
                                    "aubrieta.purple/500",
                                ),
                                _ => (
                                    format!("Rectangle {}", count),
                                    ShapeKind::Rectangle { corner_radii: [4.0; 4] },
                                    "aubrieta.green/500",
                                ),
                            };
                            let _ = self.shell.bridge.create_shape_object(
                                surface.id,
                                new_id,
                                name,
                                shape,
                                Some([min_x, min_y, dx, dy]),
                                Some(fill.to_string()),
                                Some("#ffffff".to_string()),
                                1.0,
                            );
                            self.shell.bridge.set_selection(vec![new_id]);
                        }
                    }
                }

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

    pub fn view(&self) -> Element<'_, Message> {
        let zoom_pct = (self.shell.camera.zoom * 100.0).round() as i32;
        let snap_enabled = self.shell.snap.config.grid_enabled;

        // -------------------------------------------------------------
        // SECTION 1: STUDIO MENU BAR & PERSONA SWITCHER (08.2)
        // -------------------------------------------------------------
        let brand_badge = row![
            container(text("A").size(11).color(Color::WHITE))
                .padding([2, 5])
                .style(|_| container::Style {
                    background: Some(Color::from_rgb8(59, 130, 246).into()),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            text("Aubrieta Studio").size(12).color(Color::WHITE),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        let menu_buttons = row![
            brand_badge,
            button(text("Arquivo").size(12)).on_press(Message::NewDocument),
            button(text("Editar").size(12)).on_press(Message::DuplicateSelected),
            button(text("Camada").size(12)).on_press(Message::ConvertToCurves),
            button(text("Selecionar").size(12)).on_press(Message::SelectTab(ActiveTab::Layers)),
            button(text("Documento").size(12)).on_press(Message::FitCanvas),
            button(text("Exibir").size(12)).on_press(Message::ZoomIn),
        ]
        .spacing(4)
        .align_y(Alignment::Center);

        let persona_pills = row![
            button(text("🎨 Design Persona").size(11))
                .style(move |theme, status| if self.active_persona == Persona::Design {
                    button::primary(theme, status)
                } else {
                    button::secondary(theme, status)
                })
                .on_press(Message::SwitchPersona(Persona::Design)),
            button(text("📷 Photo Persona").size(11))
                .style(move |theme, status| if self.active_persona == Persona::Photo {
                    button::primary(theme, status)
                } else {
                    button::secondary(theme, status)
                })
                .on_press(Message::SwitchPersona(Persona::Photo)),
        ]
        .spacing(2)
        .align_y(Alignment::Center);

        let menu_bar = container(
            row![menu_buttons, persona_pills]
                .spacing(12)
                .align_y(Alignment::Center)
                .padding([4, 8]),
        )
        .style(studio_panel_style(Color::from_rgb8(31, 31, 35)))
        .width(Length::Fill);

        // -------------------------------------------------------------
        // SECTION 2: DOCUMENT TAB STRIP & VIEWPORT HUD (08.2)
        // -------------------------------------------------------------
        let doc_title = self
            .shell
            .bridge
            .session()
            .map(|s| format!("{} •", s.title))
            .unwrap_or_else(|| "Showcase Project.aub •".to_string());

        let doc_tab = container(
            row![
                text(doc_title).size(12).color(Color::from_rgb8(244, 244, 245)),
                button(text("×").size(11)).on_press(Message::NewDocument),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([4, 10])
        .style(|_| container::Style {
            background: Some(Color::from_rgb8(32, 32, 36).into()),
            border: iced::Border {
                color: Color::from_rgb8(46, 46, 51),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

        let view_controls = row![
            button(text("Abrir").size(11)).on_press(Message::OpenDocument),
            button(text("Salvar").size(11)).on_press(Message::SaveDocument),
            button(text("Desfazer").size(11)).on_press(Message::Undo),
            button(text("Refazer").size(11)).on_press(Message::Redo),
            button(text("−").size(12)).on_press(Message::ZoomOut),
            container(text(format!("{zoom_pct}%")).size(11).color(Color::from_rgb8(212, 212, 216)))
                .padding([2, 6])
                .style(|_| container::Style {
                    background: Some(Color::from_rgb8(32, 32, 36).into()),
                    border: iced::Border { radius: 3.0.into(), ..Default::default() },
                    ..Default::default()
                }),
            button(text("+").size(12)).on_press(Message::ZoomIn),
            button(text("Ajustar").size(11)).on_press(Message::FitCanvas),
            row![
                checkbox(snap_enabled).on_toggle(Message::ToggleSnap),
                text("Snap").size(11).color(Color::from_rgb8(147, 197, 253)),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        let doc_strip = container(
            row![doc_tab, view_controls]
                .spacing(12)
                .align_y(Alignment::Center)
                .padding([3, 8]),
        )
        .style(studio_panel_style(Color::from_rgb8(24, 24, 27)))
        .width(Length::Fill);

        // -------------------------------------------------------------
        // SECTION 3: CANONICAL CONTEXT TOOLBAR (08.23)
        // -------------------------------------------------------------
        let tool_badge = container(
            text(format!("{:?}", self.shell.active_tool()))
                .size(11)
                .color(Color::from_rgb8(96, 165, 250)),
        )
        .padding([2, 8])
        .style(|_| container::Style {
            background: Some(Color::from_rgb8(39, 39, 43).into()),
            border: iced::Border {
                color: Color::from_rgb8(63, 63, 70),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

        let props: PropertiesPresentationModel = self.shell.query_properties();
        let transform_hud = if let Some(b) = props.bounds {
            row![
                text(format!("X: {:.0} pt", b[0])).size(11).color(Color::from_rgb8(161, 161, 170)),
                text(format!("Y: {:.0} pt", b[1])).size(11).color(Color::from_rgb8(161, 161, 170)),
                text(format!("W: {:.0} pt", b[2])).size(11).color(Color::from_rgb8(161, 161, 170)),
                text(format!("H: {:.0} pt", b[3])).size(11).color(Color::from_rgb8(161, 161, 170)),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
        } else {
            row![text("Nenhum objeto selecionado").size(11).color(Color::from_rgb8(113, 113, 122))]
                .align_y(Alignment::Center)
        };

        let curve_buttons = row![
            button(text("Curvas").size(11)).on_press(Message::ConvertToCurves),
            button(text("Bake").size(11)).on_press(Message::BakeCorners),
        ]
        .spacing(4)
        .align_y(Alignment::Center);

        let boolean_buttons = row![
            button(text("⋃ Unir").size(11)).on_press(Message::BooleanUnion),
            button(text("− Subtrair").size(11)).on_press(Message::BooleanSubtract),
            button(text("⋂ Interseção").size(11)).on_press(Message::BooleanIntersect),
            button(text("⨁ Xor").size(11)).on_press(Message::BooleanXor),
        ]
        .spacing(4)
        .align_y(Alignment::Center);

        let align_buttons = row![
            button(text("Esq").size(11)).on_press(Message::AlignObjects(AlignmentMode::Left)),
            button(text("Centro").size(11)).on_press(Message::AlignObjects(AlignmentMode::Center)),
            button(text("Dir").size(11)).on_press(Message::AlignObjects(AlignmentMode::Right)),
            button(text("Topo").size(11)).on_press(Message::AlignObjects(AlignmentMode::Top)),
            button(text("Meio").size(11)).on_press(Message::AlignObjects(AlignmentMode::Middle)),
            button(text("Base").size(11)).on_press(Message::AlignObjects(AlignmentMode::Bottom)),
        ]
        .spacing(3)
        .align_y(Alignment::Center);

        let context_toolbar = container(
            row![
                tool_badge,
                transform_hud,
                curve_buttons,
                boolean_buttons,
                align_buttons,
                button(text("Excluir").size(11)).on_press(Message::DeleteSelected),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .padding([4, 8]),
        )
        .style(studio_panel_style(Color::from_rgb8(32, 32, 36)))
        .width(Length::Fill);

        // -------------------------------------------------------------
        // SECTION 4: MAIN WORKSPACE (LEFT RAIL + CANVAS + RIGHT DOCK)
        // -------------------------------------------------------------
        // 4.1 Left Tool Rail (56px) with 5 canonical groups
        let tools_g1 = [
            (ToolKind::Select, "V\nSel"),
            (ToolKind::Node, "A\nNod"),
            (ToolKind::PointTransform, "F\nPtf"),
            (ToolKind::Artboard, "H\nArt"),
        ];
        let tools_g2 = [
            (ToolKind::Pen, "P\nPen"),
            (ToolKind::Pencil, "N\nPcl"),
            (ToolKind::Rectangle, "M\nRec"),
            (ToolKind::Ellipse, "E\nEll"),
            (ToolKind::Star, "S\nStr"),
        ];
        let tools_g3 = [
            (ToolKind::ArtisticText, "T\nTxt"),
            (ToolKind::Gradient, "G\nGrd"),
            (ToolKind::ColorPicker, "I\nPip"),
        ];
        let tools_g4 = [
            (ToolKind::Knife, "K\nKnf"),
            (ToolKind::Scissors, "C\nSci"),
            (ToolKind::ShapeBuilder, "W\nShp"),
        ];
        let tools_g5 = [
            (ToolKind::Hand, "␣\nHnd"),
            (ToolKind::Zoom, "Z\nZom"),
        ];

        let mut tool_col = Column::new().spacing(3).padding(4).width(Length::Fixed(56.0));
        for (tool, label) in tools_g1 {
            tool_col = tool_col.push(
                button(text(label).size(11).align_x(Alignment::Center))
                    .width(Length::Fill)
                    .on_press(Message::SelectTool(tool)),
            );
        }
        tool_col = tool_col.push(divider_h());
        for (tool, label) in tools_g2 {
            tool_col = tool_col.push(
                button(text(label).size(11).align_x(Alignment::Center))
                    .width(Length::Fill)
                    .on_press(Message::SelectTool(tool)),
            );
        }
        tool_col = tool_col.push(divider_h());
        for (tool, label) in tools_g3 {
            tool_col = tool_col.push(
                button(text(label).size(11).align_x(Alignment::Center))
                    .width(Length::Fill)
                    .on_press(Message::SelectTool(tool)),
            );
        }
        tool_col = tool_col.push(divider_h());
        for (tool, label) in tools_g4 {
            tool_col = tool_col.push(
                button(text(label).size(11).align_x(Alignment::Center))
                    .width(Length::Fill)
                    .on_press(Message::SelectTool(tool)),
            );
        }
        tool_col = tool_col.push(divider_h());
        for (tool, label) in tools_g5 {
            tool_col = tool_col.push(
                button(text(label).size(11).align_x(Alignment::Center))
                    .width(Length::Fill)
                    .on_press(Message::SelectTool(tool)),
            );
        }

        let left_rail = container(scrollable(tool_col))
            .style(studio_panel_style(Color::from_rgb8(26, 26, 30)))
            .height(Length::Fill);

        // 4.2 Central Canvas
        let canvas_widget = Canvas::new(AubrietaCanvasProgram {
            shell: &self.shell,
            preview_rect: self.preview_rect,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        // 4.3 Right Studio Dock (300px)
        let tab_buttons = row![
            button(text("Camadas").size(11)).on_press(Message::SelectTab(ActiveTab::Layers)),
            button(text("Propriedades").size(11)).on_press(Message::SelectTab(ActiveTab::Properties)),
            button(text("Histórico").size(11)).on_press(Message::SelectTab(ActiveTab::History)),
            button(text("Dados").size(11)).on_press(Message::SelectTab(ActiveTab::DataMerge)),
        ]
        .spacing(3);

        let dock_content: Element<'_, Message> = match self.active_tab {
            ActiveTab::Layers => {
                let model: LayersPresentationModel = self.shell.query_layers();
                let mut col = column![
                    text("Hierarquia de Objetos").size(13).color(Color::from_rgb8(96, 165, 250)),
                ]
                .spacing(4);

                for surface in &model.surfaces {
                    col = col.push(
                        container(text(format!("📄 {} [800 × 600]", surface.name)).size(11).color(Color::from_rgb8(147, 197, 253)))
                            .padding([3, 6])
                            .style(|_| container::Style {
                                background: Some(Color::from_rgb8(36, 36, 40).into()),
                                ..Default::default()
                            })
                            .width(Length::Fill),
                    );
                }

                for (idx, item) in model.rows.iter().enumerate() {
                    let vis = if item.visible { "👁" } else { " " };
                    let lock = if item.locked { "🔒" } else { " " };
                    let sel = if item.is_selected { "●" } else { " " };
                    let row_item = row![
                        button(text(format!("{sel} {vis} {lock} {}", item.name)).size(12))
                            .width(Length::Fill)
                            .on_press(Message::SelectLayer(idx)),
                        button(text("👁").size(10)).on_press(Message::ToggleLayerVisibility(idx)),
                        button(text("🔒").size(10)).on_press(Message::ToggleLayerLock(idx)),
                        button(text("▲").size(10)).on_press(Message::ReorderLayerUp(idx)),
                        button(text("▼").size(10)).on_press(Message::ReorderLayerDown(idx)),
                    ]
                    .spacing(2)
                    .align_y(Alignment::Center);
                    col = col.push(row_item);
                }

                col = col.push(divider_h());
                col = col.push(
                    row![
                        button(text("+ Retângulo").size(10)).on_press(Message::AddRectangle),
                        button(text("+ Elipse").size(10)).on_press(Message::AddCircle),
                        button(text("+ Estrela").size(10)).on_press(Message::AddStar),
                        button(text("+ Texto").size(10)).on_press(Message::AddText),
                    ]
                    .spacing(3),
                );

                scrollable(col.spacing(4)).into()
            }
            ActiveTab::Properties => {
                let model: PropertiesPresentationModel = self.shell.query_properties();
                let mut col = column![
                    text("Inspetor de Propriedades").size(13).color(Color::from_rgb8(96, 165, 250)),
                ]
                .spacing(8);

                if model.selection_empty {
                    col = col.push(text("(Nenhuma seleção)").size(12).color(Color::from_rgb8(161, 161, 170)));
                } else {
                    col = col.push(
                        text(format!("Objeto: {}", model.name.as_deref().unwrap_or("Item")))
                            .size(12)
                            .color(Color::WHITE),
                    );
                    if let Some(b) = model.bounds {
                        col = col.push(text(format!("X: {:.1} pt   Y: {:.1} pt", b[0], b[1])).size(11));
                        col = col.push(text(format!("Largura: {:.1} pt   Altura: {:.1} pt", b[2], b[3])).size(11));
                    }
                    if let Some(f) = &model.fill {
                        col = col.push(text(format!("Cor: {f}")).size(11));
                    }

                    // Quick color palette buttons
                    col = col.push(text("Preenchimento:").size(11).color(Color::from_rgb8(161, 161, 170)));
                    col = col.push(
                        row![
                            button(text("Azul").size(10)).on_press(Message::SetColor("aubrieta.blue/500".to_string())),
                            button(text("Amarelo").size(10)).on_press(Message::SetColor("aubrieta.yellow/500".to_string())),
                            button(text("Vermelho").size(10)).on_press(Message::SetColor("aubrieta.rose/500".to_string())),
                            button(text("Verde").size(10)).on_press(Message::SetColor("aubrieta.green/500".to_string())),
                            button(text("Roxo").size(10)).on_press(Message::SetColor("aubrieta.purple/500".to_string())),
                        ]
                        .spacing(3),
                    );

                    col = col.push(text(format!("Opacidade: {:.0}%", self.selected_opacity)).size(11));
                    col = col.push(slider(0.0..=100.0, self.selected_opacity, Message::SetOpacity));

                    // Pathfinder
                    col = col.push(text("Operações Booleanas:").size(11).color(Color::from_rgb8(161, 161, 170)));
                    col = col.push(
                        row![
                            button(text("⋃ Unir").size(10)).on_press(Message::BooleanUnion),
                            button(text("− Sub").size(10)).on_press(Message::BooleanSubtract),
                            button(text("⋂ Inter").size(10)).on_press(Message::BooleanIntersect),
                            button(text("⨁ Xor").size(10)).on_press(Message::BooleanXor),
                        ]
                        .spacing(3),
                    );

                    // Alignment
                    col = col.push(text("Alinhamento e Distribuição:").size(11).color(Color::from_rgb8(161, 161, 170)));
                    col = col.push(
                        row![
                            button(text("Esq").size(10)).on_press(Message::AlignObjects(AlignmentMode::Left)),
                            button(text("Centro").size(10)).on_press(Message::AlignObjects(AlignmentMode::Center)),
                            button(text("Dir").size(10)).on_press(Message::AlignObjects(AlignmentMode::Right)),
                        ]
                        .spacing(3),
                    );
                    col = col.push(
                        row![
                            button(text("Dist H").size(10)).on_press(Message::DistributeObjects(DistributionAxis::Horizontal)),
                            button(text("Dist V").size(10)).on_press(Message::DistributeObjects(DistributionAxis::Vertical)),
                        ]
                        .spacing(3),
                    );
                }
                scrollable(col).into()
            }
            ActiveTab::History => {
                let model: HistoryPresentationModel = self.shell.bridge.query_history();
                let mut col = column![
                    text("Histórico de Transações").size(13).color(Color::from_rgb8(96, 165, 250)),
                    text(format!("Undo: {} | Redo: {}", model.undo_stack.len(), model.redo_stack.len())).size(11),
                ]
                .spacing(4);

                for (i, item) in model.undo_stack.iter().enumerate().rev() {
                    col = col.push(text(format!("[#{}] {}", i + 1, item.description)).size(11));
                }
                col = col.push(
                    row![
                        button(text("Desfazer").size(11)).on_press(Message::Undo),
                        button(text("Refazer").size(11)).on_press(Message::Redo),
                    ]
                    .spacing(6),
                );
                scrollable(col).into()
            }
            ActiveTab::DataMerge => {
                let model: DataMergePresentationModel = self.shell.query_data_merge();
                let src = model.sources.first().map(|s| s.name.as_str()).unwrap_or("(Nenhuma fonte)");
                column![
                    text("Data Merge de Variáveis").size(13).color(Color::from_rgb8(96, 165, 250)),
                    text(format!("Fonte de Dados: {src}")).size(12),
                    text(format!("Total Registros: {} | Vínculos: {}", model.total_records, model.bindings.len())).size(11),
                    text("Status: Pronto para Mesclagem").color(Color::from_rgb8(34, 197, 94)).size(12),
                ]
                .spacing(6)
                .into()
            }
        };

        let right_dock = container(
            column![tab_buttons, divider_h(), dock_content]
                .spacing(6)
                .padding(8)
                .width(Length::Fixed(300.0)),
        )
        .style(studio_panel_style(Color::from_rgb8(26, 26, 30)))
        .height(Length::Fill);

        let middle_workspace = row![left_rail, canvas_widget, right_dock]
            .spacing(0)
            .height(Length::Fill);

        // -------------------------------------------------------------
        // SECTION 5: CANONICAL STATUS BAR (08.2, 08.23)
        // -------------------------------------------------------------
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

        let coords_text = if let Some(pt) = self.hovered_doc_point {
            format!("Doc: X: {:.1} pt  Y: {:.1} pt", pt.x, pt.y)
        } else {
            "Cursor fora do canvas".to_string()
        };

        let sel_count = self.shell.bridge.selection().count;
        let sel_text = if sel_count > 0 {
            format!("{sel_count} objeto(s) selecionado(s)")
        } else {
            "Nenhum objeto selecionado".to_string()
        };

        let status_bar = container(
            row![
                text(hint).size(11).color(Color::from_rgb8(161, 161, 170)),
                text(coords_text).size(11).color(Color::from_rgb8(147, 197, 253)),
                text(format!("{sel_text}  |  Zoom: {zoom_pct}%  |  Snap: {}", if snap_enabled { "Ativo" } else { "Inativo" })).size(11).color(Color::from_rgb8(113, 113, 122)),
            ]
            .spacing(16)
            .padding([4, 10])
            .align_y(Alignment::Center),
        )
        .style(studio_panel_style(Color::from_rgb8(20, 20, 22)))
        .width(Length::Fill);

        column![menu_bar, doc_strip, context_toolbar, middle_workspace, status_bar]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn divider_h<'a>() -> Element<'a, Message> {
    container(row![])
        .height(Length::Fixed(1.0))
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Color::from_rgb8(45, 45, 50).into()),
            ..Default::default()
        })
        .into()
}

fn studio_panel_style(bg: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(bg.into()),
        border: iced::Border {
            color: Color::from_rgb8(42, 42, 46),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

fn parse_fill_color(fill: Option<&str>) -> Color {
    match fill {
        Some(f) if f.contains("blue") => Color::from_rgb8(59, 130, 246),
        Some(f) if f.contains("yellow") => Color::from_rgb8(234, 179, 8),
        Some(f) if f.contains("rose") || f.contains("red") => Color::from_rgb8(225, 29, 72),
        Some(f) if f.contains("green") || f.contains("emerald") => Color::from_rgb8(16, 185, 129),
        Some(f) if f.contains("purple") => Color::from_rgb8(139, 92, 246),
        Some(f) if f.starts_with('#') && f.len() == 7 => {
            let r = u8::from_str_radix(&f[1..3], 16).unwrap_or(59);
            let g = u8::from_str_radix(&f[3..5], 16).unwrap_or(130);
            let b = u8::from_str_radix(&f[5..7], 16).unwrap_or(246);
            Color::from_rgb8(r, g, b)
        }
        _ => Color::from_rgb8(59, 130, 246),
    }
}

fn make_star_path(cx: f32, cy: f32, r_outer: f32, r_inner: f32, points: usize) -> Path {
    let mut builder = canvas::path::Builder::new();
    let step = std::f32::consts::PI / points as f32;
    for i in 0..(points * 2) {
        let r = if i % 2 == 0 { r_outer } else { r_inner };
        let angle = i as f32 * step - std::f32::consts::FRAC_PI_2;
        let x = cx + r * angle.cos();
        let y = cy + r * angle.sin();
        if i == 0 {
            builder.move_to(Point::new(x, y));
        } else {
            builder.line_to(Point::new(x, y));
        }
    }
    builder.close();
    builder.build()
}

struct AubrietaCanvasProgram<'a> {
    shell: &'a AubrietaShell,
    preview_rect: Option<[f64; 4]>,
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
            Color::from_rgb8(17, 17, 19),
        );

        let camera = &self.shell.camera;
        let to_screen = |doc_pt: GPoint| -> Point {
            let s = camera.doc_to_screen(doc_pt);
            Point::new(s.x as f32, s.y as f32)
        };

        if let Some(session) = self.shell.bridge.session() {
            let selection = self.shell.bridge.selection();

            for surface in &session.document.surfaces {
                let b = surface.bounds();
                let p0 = to_screen(GPoint::new(b[0], b[1]));
                let w = (b[2] * camera.zoom) as f32;
                let h = (b[3] * camera.zoom) as f32;
                let size = Size::new(w, h);

                // Artboard drop shadow
                frame.fill_rectangle(
                    Point::new(p0.x + 6.0, p0.y + 6.0),
                    size,
                    Color::from_rgba(0.0, 0.0, 0.0, 0.5),
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
                    content: format!("📄 {} [{:.0} × {:.0} pt]", surface.name, b[2], b[3]),
                    position: Point::new(p0.x, p0.y - 16.0),
                    color: Color::from_rgb8(148, 163, 184),
                    size: 11.0.into(),
                    ..Default::default()
                });

                // Bleed Guideline (Magenta) (10.7)
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

                // Margin Guideline (Cyan) (10.7)
                if surface.margins != Margins::ZERO {
                    let mp0 = to_screen(GPoint::new(b[0] + surface.margins.left, b[1] + surface.margins.top));
                    let mw = ((b[2] - surface.margins.left - surface.margins.right) * camera.zoom) as f32;
                    let mh = ((b[3] - surface.margins.top - surface.margins.bottom) * camera.zoom) as f32;
                    let margin_path = Path::rectangle(mp0, Size::new(mw, mh));
                    frame.stroke(
                        &margin_path,
                        Stroke::default().with_color(Color::from_rgb8(6, 182, 212)).with_width(1.0),
                    );
                }

                // Vertical Layout Guide
                let gp0 = to_screen(GPoint::new(b[0] + 200.0, b[1]));
                let gp1 = to_screen(GPoint::new(b[0] + 200.0, b[1] + b[3]));
                let guide_path = Path::line(gp0, gp1);
                frame.stroke(
                    &guide_path,
                    Stroke::default().with_color(Color::from_rgb8(6, 182, 212)).with_width(1.0),
                );

                // Render dynamic objects
                for obj in &surface.objects {
                    if !obj.visible {
                        continue;
                    }
                    let obj_b = obj.bounds.unwrap_or([0.0, 0.0, 100.0, 80.0]);
                    let op0 = to_screen(GPoint::new(obj_b[0], obj_b[1]));
                    let ow = (obj_b[2] * camera.zoom) as f32;
                    let oh = (obj_b[3] * camera.zoom) as f32;
                    let osize = Size::new(ow, oh);
                    let is_sel = selection.contains(obj.id);
                    let fill_color = parse_fill_color(obj.fill.as_deref());

                    match &obj.shape {
                        Some(ShapeKind::Ellipse) => {
                            let center = Point::new(op0.x + ow * 0.5, op0.y + oh * 0.5);
                            let circle_path = Path::circle(center, (ow * 0.5).min(oh * 0.5));
                            frame.fill(&circle_path, fill_color);
                            frame.stroke(
                                &circle_path,
                                Stroke::default().with_color(Color::from_rgb8(202, 138, 4)).with_width(1.5),
                            );
                        }
                        Some(ShapeKind::Star { points, inner_ratio }) => {
                            let center = Point::new(op0.x + ow * 0.5, op0.y + oh * 0.5);
                            let star_path = make_star_path(
                                center.x,
                                center.y,
                                ow * 0.5,
                                (ow * 0.5) * (*inner_ratio as f32),
                                *points as usize,
                            );
                            frame.fill(&star_path, fill_color);
                            frame.stroke(
                                &star_path,
                                Stroke::default().with_color(Color::from_rgb8(225, 29, 72)).with_width(1.5),
                            );
                        }
                        Some(ShapeKind::Text { content, font_size, .. }) => {
                            frame.fill_text(Text {
                                content: content.clone(),
                                position: Point::new(op0.x, op0.y + oh * 0.5 - 7.0),
                                color: fill_color,
                                size: (*font_size as f32).into(),
                                ..Default::default()
                            });
                        }
                        _ => {
                            let rect_path = Path::rectangle(op0, osize);
                            frame.fill(&rect_path, fill_color);
                            frame.stroke(
                                &rect_path,
                                Stroke::default().with_color(Color::from_rgb8(37, 99, 235)).with_width(1.5),
                            );
                        }
                    }

                    // Selection Affordances (8 Handles + Rotation Pin + HUD) (10.1)
                    if is_sel {
                        let sel_path = Path::rectangle(op0, osize);
                        frame.stroke(
                            &sel_path,
                            Stroke::default().with_color(Color::from_rgb8(59, 130, 246)).with_width(1.5),
                        );

                        // 8 Handles
                        let handles = [
                            Point::new(op0.x - 4.0, op0.y - 4.0),
                            Point::new(op0.x + ow * 0.5 - 4.0, op0.y - 4.0),
                            Point::new(op0.x + ow - 4.0, op0.y - 4.0),
                            Point::new(op0.x + ow - 4.0, op0.y + oh * 0.5 - 4.0),
                            Point::new(op0.x + ow - 4.0, op0.y + oh - 4.0),
                            Point::new(op0.x + ow * 0.5 - 4.0, op0.y + oh - 4.0),
                            Point::new(op0.x - 4.0, op0.y + oh - 4.0),
                            Point::new(op0.x - 4.0, op0.y + oh * 0.5 - 4.0),
                        ];
                        for hp in handles {
                            frame.fill_rectangle(hp, Size::new(8.0, 8.0), Color::WHITE);
                            let hp_path = Path::rectangle(hp, Size::new(8.0, 8.0));
                            frame.stroke(
                                &hp_path,
                                Stroke::default().with_color(Color::from_rgb8(37, 99, 235)).with_width(1.0),
                            );
                        }

                        // Rotation Pin
                        let pin_top = Point::new(op0.x + ow * 0.5, op0.y - 16.0);
                        let stem_path = Path::line(Point::new(op0.x + ow * 0.5, op0.y), pin_top);
                        frame.stroke(
                            &stem_path,
                            Stroke::default().with_color(Color::from_rgb8(59, 130, 246)).with_width(1.0),
                        );
                        let pin_center = Point::new(op0.x + ow * 0.5, op0.y - 18.0);
                        let pin_path = Path::circle(pin_center, 4.0);
                        frame.fill(&pin_path, Color::from_rgb8(59, 130, 246));
                        frame.stroke(
                            &pin_path,
                            Stroke::default().with_color(Color::WHITE).with_width(1.0),
                        );

                        // Dimensions HUD Badge
                        let hud_p = Point::new(op0.x + ow * 0.5 - 55.0, op0.y + oh + 8.0);
                        frame.fill_rectangle(hud_p, Size::new(110.0, 18.0), Color::from_rgba(0.1, 0.1, 0.14, 0.9));
                        frame.fill_text(Text {
                            content: format!("{:.0} × {:.0} pt", obj_b[2], obj_b[3]),
                            position: Point::new(hud_p.x + 16.0, hud_p.y + 3.0),
                            color: Color::from_rgb8(147, 197, 253),
                            size: 10.0.into(),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        // Render interactive drag creation preview
        if let Some(prev) = self.preview_rect {
            let pp0 = to_screen(GPoint::new(prev[0], prev[1]));
            let pw = (prev[2] * camera.zoom) as f32;
            let ph = (prev[3] * camera.zoom) as f32;
            let prev_size = Size::new(pw, ph);
            frame.fill_rectangle(pp0, prev_size, Color::from_rgba(0.23, 0.51, 0.96, 0.18));
            let prev_path = Path::rectangle(pp0, prev_size);
            frame.stroke(
                &prev_path,
                Stroke::default().with_color(Color::from_rgb8(59, 130, 246)).with_width(1.5),
            );
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
    let star_id = id_gen.next_object();
    let text_id = id_gen.next_object();

    shell.bridge.submit_command(CommandRequest::new(Command::CreateSurface {
        id: surface_1,
        name: "Main Artboard".to_string(),
    }))?;
    shell.bridge.set_surface_geometry(surface_1, [60.0, 60.0], [800.0, 600.0])?;
    shell.bridge.set_surface_bleed(surface_1, Bleed::uniform(10.0))?;
    shell.bridge.set_surface_margins(surface_1, Margins::uniform(36.0))?;
    shell.bridge.add_surface_guide(surface_1, Guide::new(1, GuideOrientation::Vertical, 200.0))?;

    // Hero Card (Rectangle)
    shell.bridge.create_shape_object(
        surface_1,
        rect_id,
        "Hero Card".to_string(),
        ShapeKind::Rectangle { corner_radii: [8.0; 4] },
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
        ShapeKind::Star { points: 5, inner_ratio: 0.45 },
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
        .title("Aubrieta Creative Studio — Iced")
        .theme(app_theme)
        .window_size(Size::new(1440.0, 900.0))
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
