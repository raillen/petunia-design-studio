//! Aubrieta Creative Studio — Slint Primary GUI.
//!
//! Evaluates Slint declarative UI toolkit, consuming AubrietaGuiBridge
//! presentation models and dispatching interactive commands.

slint::include_modules!();

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use aubrieta_application::{Command, CommandRequest};
use aubrieta_document::{Bleed, Guide, GuideOrientation, Margins};
use aubrieta_foundation::{AubrietaError, IdGenerator, ObjectId};
use aubrieta_geometry::{GPoint, GRect};
use aubrieta_ui_gpui::bridge::{
    DataMergePresentationModel, HistoryPresentationModel, LayersPresentationModel,
    PropertiesPresentationModel,
};
use aubrieta_ui_gpui::shell::AubrietaShell;
use aubrieta_ui_gpui::tools::{
    NormalizedPointerEvent, PointerButton, PointerPhase, SemanticModifiers, ToolKind,
};
use slint::{ComponentHandle, VecModel};

pub struct AubrietaSlintState {
    pub shell: AubrietaShell,
    pub drag_start_doc: Option<GPoint>,
    pub dragging_object_id: Option<ObjectId>,
    pub drag_initial_bounds: Option<[f64; 4]>,
}

impl AubrietaSlintState {
    pub fn new() -> Self {
        let mut shell = AubrietaShell::new(950.0, 700.0);
        let _ = populate_showcase_document(&mut shell);
        Self {
            shell,
            drag_start_doc: None,
            dragging_object_id: None,
            drag_initial_bounds: None,
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
            "OK aubrieta-slint smoke test: surfaces={} title=\"{}\"",
            snap.surface_count, snap.title
        );
        Ok(())
    }
}

fn populate_showcase_document(shell: &mut AubrietaShell) -> Result<(), AubrietaError> {
    shell.new_document("Aubrieta Showcase Project [Slint]")?;
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
        aubrieta_document::ShapeKind::Rectangle { corner_radii: [8.0; 4] },
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
        aubrieta_document::ShapeKind::Ellipse,
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
        aubrieta_document::ShapeKind::Star { points: 5, inner_ratio: 0.45 },
        Some([450.0, 320.0, 130.0, 130.0]),
        Some("aubrieta.rose/500".to_string()),
        Some("#e11d48".to_string()),
        1.5,
    )?;

    // Title Text (Text)
    shell.bridge.create_shape_object(
        surface_1,
        text_id,
        "Banner Text".to_string(),
        aubrieta_document::ShapeKind::Text {
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

fn sync_ui_from_shell(window: &MainWindow, state: &AubrietaSlintState) {
    let tool_str = format!("{:?}", state.shell.active_tool());
    window.set_active_tool_name(tool_str.into());

    let zoom_pct = (state.shell.camera.zoom * 100.0).round() as i32;
    window.set_zoom_pct(zoom_pct);

    // Sync Layers
    let layers: LayersPresentationModel = state.shell.query_layers();
    let layer_items: Vec<LayerRowItem> = layers
        .rows
        .iter()
        .map(|r| {
            let kind = if let Some(session) = state.shell.bridge.session() {
                session
                    .document
                    .surfaces
                    .iter()
                    .flat_map(|s| &s.objects)
                    .find(|o| o.id == r.id)
                    .map(|o| match &o.shape {
                        Some(aubrieta_document::ShapeKind::Ellipse) => "Ellipse",
                        Some(aubrieta_document::ShapeKind::Rectangle { .. }) => "Rectangle",
                        Some(aubrieta_document::ShapeKind::Star { .. }) => "Star",
                        Some(aubrieta_document::ShapeKind::Polygon { .. }) => "Star",
                        Some(aubrieta_document::ShapeKind::Text { .. }) => "Text",
                        Some(aubrieta_document::ShapeKind::Path(_)) => "Path",
                        None => "Rectangle",
                    })
                    .unwrap_or("Rectangle")
            } else {
                "Rectangle"
            };

            LayerRowItem {
                name: r.name.clone().into(),
                visible: r.visible,
                locked: r.locked,
                selected: r.is_selected,
                kind: kind.into(),
            }
        })
        .collect();
    window.set_layer_rows(Rc::new(VecModel::from(layer_items)).into());

    // Sync Canvas Objects
    let mut canvas_items = Vec::new();
    if let Some(session) = state.shell.bridge.session() {
        let selection = state.shell.bridge.selection();
        for surface in &session.document.surfaces {
            let sb = surface.bounds();
            for obj in &surface.objects {
                if let Some(b) = obj.bounds {
                    let is_sel = selection.contains(obj.id);
                    let name_lower = obj.name.to_lowercase();
                    let (is_circle, is_path, is_text, text_content, svg_path) = match &obj.shape {
                        Some(aubrieta_document::ShapeKind::Ellipse) => (true, false, false, String::new(), String::new()),
                        Some(aubrieta_document::ShapeKind::Rectangle { .. }) => (false, false, false, String::new(), String::new()),
                        Some(aubrieta_document::ShapeKind::Path(path)) => (false, true, false, String::new(), path.to_svg_path_data()),
                        Some(aubrieta_document::ShapeKind::Polygon { .. }) | Some(aubrieta_document::ShapeKind::Star { .. }) => {
                            (false, true, false, String::new(), obj.to_path().to_svg_path_data())
                        }
                        Some(aubrieta_document::ShapeKind::Text { content, .. }) => (false, false, true, content.clone(), String::new()),
                        None => {
                            let is_c = name_lower.contains("circle") || name_lower.contains("ellipse");
                            (is_c, false, false, String::new(), String::new())
                        }
                    };

                    let bg_color = if let Some(fill) = &obj.fill {
                        if fill.contains("blue") {
                            slint::Color::from_rgb_u8(59, 130, 246)
                        } else if fill.contains("yellow") {
                            slint::Color::from_rgb_u8(234, 179, 8)
                        } else if fill.contains("green") || fill.contains("emerald") {
                            slint::Color::from_rgb_u8(16, 185, 129)
                        } else if fill.contains("purple") {
                            slint::Color::from_rgb_u8(139, 92, 246)
                        } else if fill.contains("rose") || fill.contains("red") {
                            slint::Color::from_rgb_u8(244, 63, 94)
                        } else {
                            slint::Color::from_rgb_u8(59, 130, 246)
                        }
                    } else if is_circle {
                        slint::Color::from_rgb_u8(234, 179, 8)
                    } else {
                        slint::Color::from_rgb_u8(59, 130, 246)
                    };

                    // Relative to the artboard's top-left corner
                    let rel_x = (b[0] - sb[0]).max(0.0) as f32;
                    let rel_y = (b[1] - sb[1]).max(0.0) as f32;
                    let w = b[2].max(10.0) as f32;
                    let h = b[3].max(10.0) as f32;

                    canvas_items.push(CanvasObjectItem {
                        id: obj.id.to_string().into(),
                        name: obj.name.clone().into(),
                        x: rel_x,
                        y: rel_y,
                        w,
                        h,
                        bg_color,
                        selected: is_sel,
                        is_circle,
                        is_path,
                        is_text,
                        text_content: text_content.into(),
                        svg_path: svg_path.into(),
                    });
                }
            }
        }
    }
    window.set_canvas_objects(Rc::new(VecModel::from(canvas_items)).into());

    // Sync Properties
    let props: PropertiesPresentationModel = state.shell.query_properties();
    if !props.selection_empty {
        window.set_has_selection(true);
        window.set_selected_name(props.name.unwrap_or_else(|| "Objeto".to_string()).into());
        if let Some(b) = props.bounds {
            window.set_prop_x(b[0] as f32);
            window.set_prop_y(b[1] as f32);
            window.set_prop_w(b[2] as f32);
            window.set_prop_h(b[3] as f32);
            window.set_selected_bounds(
                format!("X: {:.1} pt   Y: {:.1} pt   W: {:.1} pt   H: {:.1} pt", b[0], b[1], b[2], b[3]).into(),
            );
        }
        if let Some(f) = props.fill {
            window.set_selected_fill(f.into());
        }
        window.set_selected_opacity((props.opacity * 100.0) as f32);
    } else {
        window.set_has_selection(false);
        window.set_selected_name("(Nenhuma seleção)".into());
        window.set_selected_bounds("—".into());
        window.set_selected_fill("—".into());
        window.set_selected_opacity(100.0);
        window.set_prop_x(0.0);
        window.set_prop_y(0.0);
        window.set_prop_w(0.0);
        window.set_prop_h(0.0);
    }

    // Sync History
    let history: HistoryPresentationModel = state.shell.bridge.query_history();
    let history_items: Vec<HistoryRowItem> = history
        .undo_stack
        .iter()
        .enumerate()
        .map(|(i, h)| HistoryRowItem {
            index: (i + 1) as i32,
            description: h.description.clone().into(),
        })
        .collect();
    window.set_history_rows(Rc::new(VecModel::from(history_items)).into());

    // Sync Data Merge
    let merge: DataMergePresentationModel = state.shell.query_data_merge();
    let src = merge.sources.first().map(|s| s.name.clone()).unwrap_or_else(|| "none".to_string());
    window.set_merge_source(src.into());
    window.set_merge_records(merge.total_records as i32);
    window.set_merge_bindings(merge.bindings.len() as i32);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--smoke-test" || a == "--headless") {
        println!("aubrieta-slint: Running automated smoke test...");
        let mut state = AubrietaSlintState::new();
        if let Err(e) = state.smoke_test() {
            eprintln!("aubrieta-slint smoke test failed: {e}");
            std::process::exit(1);
        }
        println!("aubrieta-slint: Smoke test PASSED.");
        return Ok(());
    }

    let main_window = MainWindow::new()?;
    let state = Rc::new(RefCell::new(AubrietaSlintState::new()));

    // Initial sync
    sync_ui_from_shell(&main_window, &state.borrow());

    // Tool selection
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_select_tool(move |tool_name| {
            let tool = match tool_name.as_str() {
                "Node" => ToolKind::Node,
                "Pen" => ToolKind::Pen,
                "Pencil" => ToolKind::Pencil,
                "Rectangle" => ToolKind::Rectangle,
                "Ellipse" => ToolKind::Ellipse,
                "Polygon" => ToolKind::Polygon,
                "Star" => ToolKind::Star,
                "Text" => ToolKind::ArtisticText,
                "Gradient" => ToolKind::Gradient,
                "ColorPicker" => ToolKind::ColorPicker,
                "PointTransform" => ToolKind::PointTransform,
                "Artboard" => ToolKind::Artboard,
                "Knife" => ToolKind::Knife,
                "Scissors" => ToolKind::Scissors,
                "ShapeBuilder" => ToolKind::ShapeBuilder,
                "Hand" => ToolKind::Hand,
                "Zoom" => ToolKind::Zoom,
                _ => ToolKind::Select,
            };
            state_clone.borrow_mut().shell.set_active_tool(tool);
            if let Some(win) = win_weak.upgrade() {
                let hint = match tool {
                    ToolKind::Select => "Select: Clique para selecionar, arraste para mover | Shift: Multi-seleção | Alt: Duplicar",
                    ToolKind::Node => "Node: Clique e arraste pontos de controle e alças Bézier para ajustar curvas.",
                    ToolKind::Pen => "Pen: Clique para criar nós angulares, arraste para nós suaves com tangentes.",
                    ToolKind::Pencil => "Pencil: Desenho vetorial à mão livre com suavização dinâmica.",
                    ToolKind::Rectangle => "Rectangle: Clique e arraste para desenhar retângulos e quadrados com cantos vivos ou arredondados.",
                    ToolKind::Ellipse => "Ellipse: Clique e arraste para desenhar elipses ou círculos perfeitos (com Shift).",
                    ToolKind::Polygon => "Polygon: Desenha polígonos regulares configuráveis.",
                    ToolKind::Star => "Star: Desenha estrelas vetoriais com raio interno personalizável.",
                    ToolKind::ArtisticText | ToolKind::FrameText => "Text: Clique no canvas para criar caixa de texto com tipografia vetorial.",
                    ToolKind::Gradient => "Gradient: Arraste sobre o objeto para definir gradiente linear ou radial.",
                    ToolKind::ColorPicker => "Color Picker: Clique em qualquer elemento para capturar cor de preenchimento.",
                    ToolKind::Knife => "Knife/Scissors: Fatie formas e caminhos vetoriais com uma linha de corte.",
                    ToolKind::ShapeBuilder => "Shape Builder: Combine, una ou subtraia regiões de geometrias sobrepostas.",
                    ToolKind::PointTransform => "Point Transform: Transformações afins livres com ponto de pivô customizado.",
                    ToolKind::Artboard => "Artboard: Redimensione ou crie novas pranchetas de trabalho.",
                    ToolKind::Hand => "Hand: Arraste para navegar pelo espaço infinito da prancheta.",
                    ToolKind::Zoom => "Zoom: Clique para ampliar, Alt+Clique para reduzir o zoom.",
                    _ => "Aubrieta Studio: Ferramenta pronta para uso.",
                };
                win.set_status_hint(hint.into());
                sync_ui_from_shell(&win, &state_clone.borrow());
            }
        });
    }

    // Persona switcher (08.2)
    {
        let win_weak = main_window.as_weak();
        main_window.on_switch_persona(move |p| {
            if let Some(win) = win_weak.upgrade() {
                win.set_active_persona(p);
                let hint = if p == 0 {
                    "🎨 Design Persona: Modo Vetorial ativo. Ferramentas de desenho, nós, preenchimento e curvas."
                } else {
                    "📷 Photo Persona: Modo Raster ativo. Pincéis de pixels, recorte, retoque e máscaras raster."
                };
                win.set_status_hint(hint.into());
            }
        });
    }

    // Undo / Redo
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_undo_clicked(move || {
            let _ = state_clone.borrow_mut().shell.undo();
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &state_clone.borrow());
            }
        });
    }

    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_redo_clicked(move || {
            let _ = state_clone.borrow_mut().shell.redo();
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &state_clone.borrow());
            }
        });
    }

    // Zoom & View
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_zoom_in_clicked(move || {
            let center = GPoint::new(400.0, 300.0);
            state_clone.borrow_mut().shell.zoom_at(center, 1.2);
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &state_clone.borrow());
            }
        });
    }

    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_zoom_out_clicked(move || {
            let center = GPoint::new(400.0, 300.0);
            state_clone.borrow_mut().shell.zoom_at(center, 0.8);
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &state_clone.borrow());
            }
        });
    }

    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_fit_canvas_clicked(move || {
            let mut st = state_clone.borrow_mut();
            if let Some(surface) = st.shell.bridge.session().and_then(|s| s.document.surfaces.first()) {
                let b = surface.bounds();
                st.shell.fit_surface(GRect::new(b[0], b[1], b[0] + b[2], b[1] + b[3]));
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_new_doc_clicked(move || {
            let _ = state_clone.borrow_mut().shell.new_document("Untitled");
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &state_clone.borrow());
            }
        });
    }

    // Open Document via RFD
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_open_doc_clicked(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Aubrieta Design (*.aub)", &["aub"])
                .add_filter("Gráficos Vetoriais SVG (*.svg)", &["svg"])
                .add_filter("Todos os arquivos (*.*)", &["*"])
                .set_title("Abrir Documento Aubrieta")
                .pick_file()
            {
                println!("RFD: Arquivo selecionado para abertura: {:?}", path);
                let title = path.file_name().and_then(|n| n.to_str()).unwrap_or("Novo Documento");
                let _ = state_clone.borrow_mut().shell.new_document(title);
                if let Some(win) = win_weak.upgrade() {
                    sync_ui_from_shell(&win, &state_clone.borrow());
                }
            }
        });
    }

    // Save Document via RFD
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_save_doc_clicked(move || {
            let default_name = state_clone
                .borrow()
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
                if let Some(win) = win_weak.upgrade() {
                    sync_ui_from_shell(&win, &state_clone.borrow());
                }
            }
        });
    }

    // Place Image via RFD
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_place_image_clicked(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Imagens Raster/Vetoriais (*.png, *.jpg, *.jpeg, *.svg)", &["png", "jpg", "jpeg", "svg"])
                .set_title("Inserir Imagem no Documento")
                .pick_file()
            {
                println!("RFD: Inserindo imagem: {:?}", path);
                let mut st = state_clone.borrow_mut();
                let mut id_gen = IdGenerator::new();
                let obj_id = id_gen.next_object();
                if let Some(surface) = st.shell.bridge.session().and_then(|s| s.document.surfaces.first().cloned()) {
                    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Imagem");
                    let _ = st.shell.bridge.submit_command(CommandRequest::new(Command::CreateObject {
                        surface: surface.id,
                        id: obj_id,
                        name: format!("Imagem: {file_stem}"),
                    }));
                    let _ = st.shell.bridge.set_bounds(obj_id, Some([180.0, 180.0, 240.0, 160.0]), 0.0);
                    let _ = st.shell.bridge.set_fill(obj_id, Some("aubrieta.green/500".to_string()));
                    st.shell.bridge.set_selection(vec![obj_id]);
                }
                if let Some(win) = win_weak.upgrade() {
                    sync_ui_from_shell(&win, &st);
                }
            }
        });
    }

    {
        let state_clone = state.clone();
        main_window.on_snap_toggled(move |enabled| {
            let mut st = state_clone.borrow_mut();
            st.shell.snap.config.grid_enabled = enabled;
            st.shell.snap.config.guides_enabled = enabled;
        });
    }

    {
        let win_weak = main_window.as_weak();
        main_window.on_tab_changed(move |tab_idx| {
            if let Some(win) = win_weak.upgrade() {
                win.set_active_tab_index(tab_idx);
            }
        });
    }

    // Select layer by index
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_select_layer_by_index(move |idx| {
            let mut st = state_clone.borrow_mut();
            if let Some(session) = st.shell.bridge.session() {
                if let Some(surface) = session.document.surfaces.first() {
                    if let Some(obj) = surface.objects.get(idx as usize) {
                        let obj_id = obj.id;
                        st.shell.bridge.set_selection(vec![obj_id]);
                    }
                }
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Add rectangle clicked
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_add_rectangle_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let mut id_gen = IdGenerator::new();
            if let Some(surface) = st.shell.bridge.session().and_then(|s| s.document.surfaces.first().cloned()) {
                let new_id = id_gen.next_object();
                let count = surface.objects.len() + 1;
                let _ = st.shell.bridge.submit_command(CommandRequest::new(Command::CreateObject {
                    surface: surface.id,
                    id: new_id,
                    name: format!("Rectangle {}", count),
                }));
                let offset = (count as f64 * 35.0) % 250.0;
                let _ = st.shell.bridge.set_bounds(new_id, Some([120.0 + offset, 120.0 + offset, 200.0, 130.0]), 0.0);
                let _ = st.shell.bridge.set_fill(new_id, Some("aubrieta.green/500".to_string()));
                st.shell.bridge.set_selection(vec![new_id]);
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Add circle clicked
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_add_circle_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let mut id_gen = IdGenerator::new();
            if let Some(surface) = st.shell.bridge.session().and_then(|s| s.document.surfaces.first().cloned()) {
                let new_id = id_gen.next_object();
                let count = surface.objects.len() + 1;
                let _ = st.shell.bridge.submit_command(CommandRequest::new(Command::CreateObject {
                    surface: surface.id,
                    id: new_id,
                    name: format!("Circle {}", count),
                }));
                let offset = (count as f64 * 35.0) % 250.0;
                let _ = st.shell.bridge.set_bounds(new_id, Some([360.0 + offset, 180.0 + offset, 130.0, 130.0]), 0.0);
                let _ = st.shell.bridge.set_fill(new_id, Some("aubrieta.purple/500".to_string()));
                st.shell.bridge.set_selection(vec![new_id]);
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Add star clicked
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_add_star_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let mut id_gen = IdGenerator::new();
            if let Some(surface) = st.shell.bridge.session().and_then(|s| s.document.surfaces.first().cloned()) {
                let new_id = id_gen.next_object();
                let count = surface.objects.len() + 1;
                let offset = (count as f64 * 35.0) % 250.0;
                let _ = st.shell.bridge.create_shape_object(
                    surface.id,
                    new_id,
                    format!("Star {}", count),
                    aubrieta_document::ShapeKind::Star { points: 5, inner_ratio: 0.45 },
                    Some([220.0 + offset, 160.0 + offset, 130.0, 130.0]),
                    Some("aubrieta.rose/500".to_string()),
                    Some("#e11d48".to_string()),
                    1.5,
                );
                st.shell.bridge.set_selection(vec![new_id]);
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Add text clicked
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_add_text_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let mut id_gen = IdGenerator::new();
            if let Some(surface) = st.shell.bridge.session().and_then(|s| s.document.surfaces.first().cloned()) {
                let new_id = id_gen.next_object();
                let count = surface.objects.len() + 1;
                let offset = (count as f64 * 25.0) % 200.0;
                let _ = st.shell.bridge.create_shape_object(
                    surface.id,
                    new_id,
                    format!("Text {}", count),
                    aubrieta_document::ShapeKind::Text {
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
                st.shell.bridge.set_selection(vec![new_id]);
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Boolean Union
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_boolean_union_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let mut sel_ids = st.shell.bridge.selection().selected_ids.clone();
            if sel_ids.len() < 2 {
                if let Some(session) = st.shell.bridge.session() {
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
                if let Some(surface) = st.shell.bridge.session().and_then(|s| s.document.surfaces.first().cloned()) {
                    let _ = st.shell.bridge.apply_boolean(
                        surface.id,
                        target_id,
                        id_a,
                        id_b,
                        aubrieta_geometry::BooleanOp::Union,
                    );
                    st.shell.bridge.set_selection(vec![target_id]);
                }
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Boolean Subtract
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_boolean_subtract_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let mut sel_ids = st.shell.bridge.selection().selected_ids.clone();
            if sel_ids.len() < 2 {
                if let Some(session) = st.shell.bridge.session() {
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
                if let Some(surface) = st.shell.bridge.session().and_then(|s| s.document.surfaces.first().cloned()) {
                    let _ = st.shell.bridge.apply_boolean(
                        surface.id,
                        target_id,
                        id_a,
                        id_b,
                        aubrieta_geometry::BooleanOp::Difference,
                    );
                    st.shell.bridge.set_selection(vec![target_id]);
                }
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Boolean Intersect
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_boolean_intersect_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let mut sel_ids = st.shell.bridge.selection().selected_ids.clone();
            if sel_ids.len() < 2 {
                if let Some(session) = st.shell.bridge.session() {
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
                if let Some(surface) = st.shell.bridge.session().and_then(|s| s.document.surfaces.first().cloned()) {
                    let _ = st.shell.bridge.apply_boolean(
                        surface.id,
                        target_id,
                        id_a,
                        id_b,
                        aubrieta_geometry::BooleanOp::Intersection,
                    );
                    st.shell.bridge.set_selection(vec![target_id]);
                }
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Boolean Xor
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_boolean_xor_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let mut sel_ids = st.shell.bridge.selection().selected_ids.clone();
            if sel_ids.len() < 2 {
                if let Some(session) = st.shell.bridge.session() {
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
                if let Some(surface) = st.shell.bridge.session().and_then(|s| s.document.surfaces.first().cloned()) {
                    let _ = st.shell.bridge.apply_boolean(
                        surface.id,
                        target_id,
                        id_a,
                        id_b,
                        aubrieta_geometry::BooleanOp::Xor,
                    );
                    st.shell.bridge.set_selection(vec![target_id]);
                }
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Convert to curves
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_convert_to_curves_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let sel = st.shell.bridge.selection().selected_ids.clone();
            for id in sel {
                let _ = st.shell.bridge.convert_to_curves(id);
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Bake corners
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_bake_corners_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let sel = st.shell.bridge.selection().selected_ids.clone();
            for id in sel {
                let _ = st.shell.bridge.bake_corners(id);
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Align objects
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_align_objects_clicked(move |mode_str| {
            let mut st = state_clone.borrow_mut();
            let sel = st.shell.bridge.selection().selected_ids.clone();
            if !sel.is_empty() {
                if let Some(surface_id) = st.shell.bridge.active_surface().or_else(|| {
                    st.shell.bridge.session().and_then(|s| s.document.surfaces.first().map(|sf| sf.id))
                }) {
                    let mode = match mode_str.as_str() {
                        "Left" => aubrieta_document::AlignmentMode::Left,
                        "Right" => aubrieta_document::AlignmentMode::Right,
                        "Top" => aubrieta_document::AlignmentMode::Top,
                        "Bottom" => aubrieta_document::AlignmentMode::Bottom,
                        "Middle" => aubrieta_document::AlignmentMode::Middle,
                        _ => aubrieta_document::AlignmentMode::Center,
                    };
                    let _ = st.shell.bridge.align_objects(surface_id, sel, mode);
                }
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Distribute objects
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_distribute_objects_clicked(move |axis_str| {
            let mut st = state_clone.borrow_mut();
            let sel = st.shell.bridge.selection().selected_ids.clone();
            if sel.len() >= 2 {
                if let Some(surface_id) = st.shell.bridge.active_surface().or_else(|| {
                    st.shell.bridge.session().and_then(|s| s.document.surfaces.first().map(|sf| sf.id))
                }) {
                    let axis = match axis_str.as_str() {
                        "Vertical" => aubrieta_document::DistributionAxis::Vertical,
                        _ => aubrieta_document::DistributionAxis::Horizontal,
                    };
                    let _ = st.shell.bridge.distribute_objects(surface_id, sel, axis);
                }
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Delete selected clicked
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_delete_selected_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let sel = st.shell.bridge.selection();
            for id in sel.selected_ids {
                let _ = st.shell.bridge.submit_command(CommandRequest::new(Command::DeleteObject { id }));
            }
            st.shell.bridge.clear_selection();
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Set selected color
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_set_selected_color(move |c| {
            let mut st = state_clone.borrow_mut();
            let sel = st.shell.bridge.selection();
            let color_name = if c.red() > 200 && c.blue() > 200 {
                "aubrieta.purple/500"
            } else if c.red() > 200 && c.green() > 150 {
                "aubrieta.yellow/500"
            } else if c.red() > 200 {
                "aubrieta.rose/500"
            } else if c.green() > 150 {
                "aubrieta.green/500"
            } else {
                "aubrieta.blue/500"
            };
            for id in sel.selected_ids {
                let _ = st.shell.bridge.set_fill(id, Some(color_name.to_string()));
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Adjust properties callbacks
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_adjust_prop_x(move |dx| {
            let mut st = state_clone.borrow_mut();
            if let Some(sel_id) = st.shell.bridge.selection().selected_ids.first().copied() {
                if let Some(session) = st.shell.bridge.session() {
                    if let Some(obj) = session.document.surfaces.iter().flat_map(|s| &s.objects).find(|o| o.id == sel_id) {
                        if let Some(b) = obj.bounds {
                            let new_bounds = [b[0] + dx as f64, b[1], b[2], b[3]];
                            let _ = st.shell.bridge.set_bounds(sel_id, Some(new_bounds), 0.0);
                        }
                    }
                }
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_adjust_prop_y(move |dy| {
            let mut st = state_clone.borrow_mut();
            if let Some(sel_id) = st.shell.bridge.selection().selected_ids.first().copied() {
                if let Some(session) = st.shell.bridge.session() {
                    if let Some(obj) = session.document.surfaces.iter().flat_map(|s| &s.objects).find(|o| o.id == sel_id) {
                        if let Some(b) = obj.bounds {
                            let new_bounds = [b[0], b[1] + dy as f64, b[2], b[3]];
                            let _ = st.shell.bridge.set_bounds(sel_id, Some(new_bounds), 0.0);
                        }
                    }
                }
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_adjust_prop_w(move |dw| {
            let mut st = state_clone.borrow_mut();
            if let Some(sel_id) = st.shell.bridge.selection().selected_ids.first().copied() {
                if let Some(session) = st.shell.bridge.session() {
                    if let Some(obj) = session.document.surfaces.iter().flat_map(|s| &s.objects).find(|o| o.id == sel_id) {
                        if let Some(b) = obj.bounds {
                            let new_bounds = [b[0], b[1], (b[2] + dw as f64).max(10.0), b[3]];
                            let _ = st.shell.bridge.set_bounds(sel_id, Some(new_bounds), 0.0);
                        }
                    }
                }
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_adjust_prop_h(move |dh| {
            let mut st = state_clone.borrow_mut();
            if let Some(sel_id) = st.shell.bridge.selection().selected_ids.first().copied() {
                if let Some(session) = st.shell.bridge.session() {
                    if let Some(obj) = session.document.surfaces.iter().flat_map(|s| &s.objects).find(|o| o.id == sel_id) {
                        if let Some(b) = obj.bounds {
                            let new_bounds = [b[0], b[1], b[2], (b[3] + dh as f64).max(10.0)];
                            let _ = st.shell.bridge.set_bounds(sel_id, Some(new_bounds), 0.0);
                        }
                    }
                }
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_set_opacity_value(move |val| {
            let mut st = state_clone.borrow_mut();
            if let Some(sel_id) = st.shell.bridge.selection().selected_ids.first().copied() {
                let opacity = (val as f64 / 100.0).clamp(0.0, 1.0);
                let _ = st.shell.bridge.submit_command(CommandRequest::new(Command::SetOpacity { id: sel_id, opacity }));
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_duplicate_selected_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let mut new_id = None;
            let clone_info = if let Some(sel_id) = st.shell.bridge.selection().selected_ids.first().copied() {
                if let Some(session) = st.shell.bridge.session() {
                    if let Some(surface) = session.document.surfaces.first() {
                        if let Some(obj) = surface.objects.iter().find(|o| o.id == sel_id) {
                            let b = obj.bounds.unwrap_or([100.0, 100.0, 100.0, 100.0]);
                            let clone_bounds = [b[0] + 20.0, b[1] + 20.0, b[2], b[3]];
                            let fill = obj.fill.clone();
                            let name = format!("{} (cópia)", obj.name);
                            let surf_id = surface.id;
                            Some((surf_id, name, clone_bounds, fill))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            if let Some((surf_id, name, clone_bounds, fill)) = clone_info {
                let mut id_gen = IdGenerator::new();
                let clone_id = id_gen.next_object();
                let _ = st.shell.bridge.submit_command(CommandRequest::new(Command::CreateObject {
                    surface: surf_id,
                    id: clone_id,
                    name,
                }));
                let _ = st.shell.bridge.set_bounds(clone_id, Some(clone_bounds), 0.0);
                if let Some(f) = fill {
                    let _ = st.shell.bridge.set_fill(clone_id, Some(f));
                }
                new_id = Some(clone_id);
            }
            if let Some(id) = new_id {
                st.shell.bridge.set_selection(vec![id]);
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_toggle_layer_visibility(move |idx| {
            let mut st = state_clone.borrow_mut();
            let toggle = if let Some(session) = st.shell.bridge.session() {
                if let Some(surface) = session.document.surfaces.first() {
                    surface.objects.get(idx as usize).map(|obj| (obj.id, !obj.visible))
                } else {
                    None
                }
            } else {
                None
            };
            if let Some((id, visible)) = toggle {
                let _ = st.shell.bridge.submit_command(CommandRequest::new(Command::SetVisibility { id, visible }));
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_toggle_layer_lock(move |idx| {
            let mut st = state_clone.borrow_mut();
            let toggle = if let Some(session) = st.shell.bridge.session() {
                if let Some(surface) = session.document.surfaces.first() {
                    surface.objects.get(idx as usize).map(|obj| (obj.id, !obj.locked))
                } else {
                    None
                }
            } else {
                None
            };
            if let Some((id, locked)) = toggle {
                let _ = st.shell.bridge.submit_command(CommandRequest::new(Command::SetLocked { id, locked }));
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_reorder_layer_up(move |idx| {
            let mut st = state_clone.borrow_mut();
            let reorder = if idx > 0 {
                if let Some(session) = st.shell.bridge.session() {
                    if let Some(surface) = session.document.surfaces.first() {
                        surface.objects.get(idx as usize).map(|obj| (surface.id, obj.id, (idx - 1) as usize))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
            if let Some((surf_id, id, new_index)) = reorder {
                let _ = st.shell.bridge.submit_command(CommandRequest::new(Command::ReorderObject {
                    surface: surf_id,
                    id,
                    new_index,
                }));
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_reorder_layer_down(move |idx| {
            let mut st = state_clone.borrow_mut();
            let reorder = if let Some(session) = st.shell.bridge.session() {
                if let Some(surface) = session.document.surfaces.first() {
                    if (idx as usize) + 1 < surface.objects.len() {
                        surface.objects.get(idx as usize).map(|obj| (surface.id, obj.id, (idx + 1) as usize))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
            if let Some((surf_id, id, new_index)) = reorder {
                let _ = st.shell.bridge.submit_command(CommandRequest::new(Command::ReorderObject {
                    surface: surf_id,
                    id,
                    new_index,
                }));
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Interactive pointer down
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_canvas_pointer_down(move |x, y| {
            let screen_pt = GPoint::new(x as f64, y as f64);
            let mut st = state_clone.borrow_mut();
            let doc_pt = st.shell.camera.screen_to_doc(screen_pt);
            st.drag_start_doc = Some(doc_pt);

            // Hit test against existing objects in active surface
            let mut hit_obj = None;
            let mut hit_bounds = None;
            if let Some(session) = st.shell.bridge.session() {
                if let Some(surface) = session.document.surfaces.first() {
                    for obj in surface.objects.iter().rev() {
                        if let Some(b) = obj.bounds {
                            if doc_pt.x >= b[0] && doc_pt.x <= b[0] + b[2] && doc_pt.y >= b[1] && doc_pt.y <= b[1] + b[3] {
                                hit_obj = Some(obj.id);
                                hit_bounds = Some(b);
                                break;
                            }
                        }
                    }
                }
            }

            if let Some(id) = hit_obj {
                st.shell.bridge.set_selection(vec![id]);
                st.dragging_object_id = Some(id);
                st.drag_initial_bounds = hit_bounds;
            } else if st.shell.active_tool() == ToolKind::Select {
                st.shell.bridge.clear_selection();
                st.dragging_object_id = None;
                st.drag_initial_bounds = None;
            }

            let evt = NormalizedPointerEvent::new(
                PointerPhase::Down,
                PointerButton::Primary,
                screen_pt,
                doc_pt,
                SemanticModifiers::default(),
            );
            let _ = st.shell.handle_pointer_event(&evt);

            if let Some(win) = win_weak.upgrade() {
                win.set_status_coords(format!("Doc: X: {:.1} pt  Y: {:.1} pt", doc_pt.x, doc_pt.y).into());
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Interactive drag
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_canvas_dragged(move |x, y| {
            let screen_pt = GPoint::new(x as f64, y as f64);
            let mut st = state_clone.borrow_mut();
            let doc_pt = st.shell.camera.screen_to_doc(screen_pt);

            if let Some(start_doc) = st.drag_start_doc {
                match st.shell.active_tool() {
                    ToolKind::Rectangle | ToolKind::Ellipse | ToolKind::Polygon | ToolKind::Pen => {
                        let min_x = start_doc.x.min(doc_pt.x);
                        let min_y = start_doc.y.min(doc_pt.y);
                        let w = (doc_pt.x - start_doc.x).abs();
                        let h = (doc_pt.y - start_doc.y).abs();
                        if let Some(win) = win_weak.upgrade() {
                            let artboard_x = win.get_artboard_x();
                            let artboard_y = win.get_artboard_y();
                            win.set_has_preview(true);
                            win.set_preview_x((min_x - artboard_x as f64).max(0.0) as f32);
                            win.set_preview_y((min_y - artboard_y as f64).max(0.0) as f32);
                            win.set_preview_w(w as f32);
                            win.set_preview_h(h as f32);
                        }
                    }
                    ToolKind::Select => {
                        if let (Some(obj_id), Some(init_b)) = (st.dragging_object_id, st.drag_initial_bounds) {
                            let dx = doc_pt.x - start_doc.x;
                            let dy = doc_pt.y - start_doc.y;
                            let new_b = [init_b[0] + dx, init_b[1] + dy, init_b[2], init_b[3]];
                            let _ = st.shell.bridge.set_bounds(obj_id, Some(new_b), 0.0);
                            if let Some(win) = win_weak.upgrade() {
                                sync_ui_from_shell(&win, &st);
                            }
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
            let _ = st.shell.handle_pointer_event(&evt);

            if let Some(win) = win_weak.upgrade() {
                win.set_status_coords(format!("Doc: X: {:.1} pt  Y: {:.1} pt", doc_pt.x, doc_pt.y).into());
            }
        });
    }

    // Interactive pointer up
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_canvas_pointer_up(move |x, y| {
            let screen_pt = GPoint::new(x as f64, y as f64);
            let mut st = state_clone.borrow_mut();
            let doc_pt = st.shell.camera.screen_to_doc(screen_pt);

            if let Some(win) = win_weak.upgrade() {
                win.set_has_preview(false);
            }

            // If drawing a shape, commit the new object
            if let Some(start_doc) = st.drag_start_doc.take() {
                let dx = (doc_pt.x - start_doc.x).abs();
                let dy = (doc_pt.y - start_doc.y).abs();
                if dx > 8.0 && dy > 8.0 {
                    let min_x = start_doc.x.min(doc_pt.x);
                    let min_y = start_doc.y.min(doc_pt.y);
                    let mut id_gen = IdGenerator::new();
                    if let Some(surface) = st.shell.bridge.session().and_then(|s| s.document.surfaces.first().cloned()) {
                        let new_id = id_gen.next_object();
                        let count = surface.objects.len() + 1;
                        let (name, shape, fill) = match st.shell.active_tool() {
                            ToolKind::Ellipse => (
                                format!("Ellipse {}", count),
                                aubrieta_document::ShapeKind::Ellipse,
                                "aubrieta.yellow/500",
                            ),
                            ToolKind::Polygon => (
                                format!("Star {}", count),
                                aubrieta_document::ShapeKind::Star { points: 5, inner_ratio: 0.45 },
                                "aubrieta.rose/500",
                            ),
                            ToolKind::Pen => (
                                format!("Path {}", count),
                                aubrieta_document::ShapeKind::Path(aubrieta_geometry::GPath::rect(
                                    aubrieta_geometry::GRect::new(min_x, min_y, min_x + dx, min_y + dy),
                                    0.0,
                                    0.0,
                                )),
                                "aubrieta.purple/500",
                            ),
                            _ => (
                                format!("Rectangle {}", count),
                                aubrieta_document::ShapeKind::Rectangle { corner_radii: [4.0; 4] },
                                "aubrieta.green/500",
                            ),
                        };
                        let _ = st.shell.bridge.create_shape_object(
                            surface.id,
                            new_id,
                            name,
                            shape,
                            Some([min_x, min_y, dx, dy]),
                            Some(fill.to_string()),
                            Some("#ffffff".to_string()),
                            1.0,
                        );
                        st.shell.bridge.set_selection(vec![new_id]);
                    }
                }
            }

            st.dragging_object_id = None;
            st.drag_initial_bounds = None;

            let evt = NormalizedPointerEvent::new(
                PointerPhase::Up,
                PointerButton::Primary,
                screen_pt,
                doc_pt,
                SemanticModifiers::default(),
            );
            let _ = st.shell.handle_pointer_event(&evt);

            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    main_window.run()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slint_app_smoke_test_headless() {
        let mut state = AubrietaSlintState::new();
        assert!(state.smoke_test().is_ok());
    }
}
