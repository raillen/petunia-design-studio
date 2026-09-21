//! Aubrieta Creative Studio — Slint Primary GUI.
//!
//! Evaluates Slint declarative UI toolkit, consuming AubrietaGuiBridge
//! presentation models and dispatching interactive commands.

slint::include_modules!();

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use aubrieta_application::{Command, CommandRequest};
use aubrieta_document::{Bleed, ContainerRole, Guide, GuideOrientation, Margins};
use aubrieta_foundation::{AubrietaError, IdGenerator, ObjectId};
use aubrieta_io::{
    export_document_pdf, export_document_svg, export_raster, PdfExportOptions,
    RasterExportOptions, RasterFormat, RawRasterImage,
};
use aubrieta_geometry::{GAffine, GPoint, GRect};
use aubrieta_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase, SemanticModifiers,
};
use aubrieta_application::tools::ToolKind;
use aubrieta_shell::bridge::{
    DataMergePresentationModel, HistoryPresentationModel, LayersPresentationModel,
    PropertiesPresentationModel,
};
use aubrieta_shell::canvas::overlay::{hit_test_handle_or_border, SelectionHandleKind};
use aubrieta_shell::shell::AubrietaShell;
use slint::{ComponentHandle, VecModel};

pub struct AubrietaSlintState {
    pub shell: AubrietaShell,
    pub drag_start_doc: Option<GPoint>,
    pub dragging_object_id: Option<ObjectId>,
    pub drag_initial_bounds: Option<[f64; 4]>,
}

impl Default for AubrietaSlintState {
    fn default() -> Self {
        Self::new()
    }
}

impl AubrietaSlintState {
    pub fn new() -> Self {
        let mut shell = AubrietaShell::new(950.0, 700.0);
        shell.camera.pan_x = 80.0;
        shell.camera.pan_y = 80.0;
        shell.camera.zoom = 1.0;
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

        // 1. Validate Layers Hierarchy Presentation Model
        let layers = self.shell.query_layers();
        if layers.rows.is_empty() {
            return Err("Layers model has no rows".to_string());
        }

        // 2. Validate Grouping & Hierarchy (Step 2)
        let surface_id = snap.active_surface.ok_or("No active surface in snapshot")?;
        let rect_id = layers.rows[0].id;
        let circle_id = layers.rows[1].id;
        let group_id = self.shell.bridge.next_object_id().map_err(|e| e.to_string())?;

        self.shell
            .bridge
            .group_objects(
                surface_id,
                group_id,
                vec![rect_id, circle_id],
                ContainerRole::Group,
            )
            .map_err(|e| format!("group_objects: {e}"))?;

        let post_group_layers = self.shell.query_layers();
        let group_row = post_group_layers
            .rows
            .iter()
            .find(|r| r.id == group_id)
            .ok_or("Group row not found in layers after grouping")?;
        if !group_row.is_container {
            return Err("Group row is not flagged as container".to_string());
        }

        // 3. Validate Clipping Mask Creation & Release (Step 2)
        let mask_id = self.shell.bridge.next_object_id().map_err(|e| e.to_string())?;
        let clip_group_id = self.shell.bridge.next_object_id().map_err(|e| e.to_string())?;
        self.shell
            .bridge
            .create_shape_object(
                surface_id,
                mask_id,
                "Mask Shape".to_string(),
                aubrieta_document::ShapeKind::Ellipse,
                Some([150.0, 150.0, 100.0, 100.0]),
                Some("aubrieta.purple/500".to_string()),
                None,
                0.0,
            )
            .map_err(|e| format!("create mask shape: {e}"))?;

        self.shell
            .bridge
            .create_clip_group(surface_id, clip_group_id, mask_id, vec![group_id])
            .map_err(|e| format!("create_clip_group: {e}"))?;

        let post_clip_layers = self.shell.query_layers();
        let mask_row = post_clip_layers
            .rows
            .iter()
            .find(|r| r.id == mask_id)
            .ok_or("Mask row not found in layers")?;
        if !mask_row.is_clip_mask {
            return Err("Mask row is not flagged as clip mask".to_string());
        }

        self.shell
            .bridge
            .release_clip_group(clip_group_id)
            .map_err(|e| format!("release_clip_group: {e}"))?;

        // 4. Validate Vector & Raster Export Pipelines (Step 4)
        let session = self.shell.bridge.session().ok_or("No session found")?;
        let svg = export_document_svg(&session.document());
        if !svg.starts_with("<svg") && !svg.contains("<svg") {
            return Err("SVG export produced invalid XML envelope".to_string());
        }

        let (pdf_bytes, report) =
            export_document_pdf(&session.document(), &PdfExportOptions::default())
                .map_err(|e| format!("PDF export failed: {e}"))?;
        if !pdf_bytes.starts_with(b"%PDF") {
            return Err("PDF export produced invalid PDF header".to_string());
        }
        if !report.passed {
            return Err("PDF preflight report marked as failed".to_string());
        }

        let raw = RawRasterImage::from_rgba8(32, 32, vec![200u8; 32 * 32 * 4])
            .map_err(|e| format!("RawRasterImage creation failed: {e}"))?;
        let (png_bytes, _) = export_raster(
            &raw,
            &RasterExportOptions {
                format: RasterFormat::Png,
                jpeg_quality: 90,
                allow_degradations: true,
            },
        )
        .map_err(|e| format!("PNG export failed: {e}"))?;
        if !png_bytes.starts_with(b"\x89PNG") {
            return Err("PNG export produced invalid magic header".to_string());
        }

        let _ = self.shell.query_properties();
        let _ = self.shell.bridge.query_history();
        let _ = self.shell.query_data_merge();
        println!(
            "OK aubrieta-slint smoke test: surfaces={} title=\"{}\" svg_len={} pdf_len={} png_len={}",
            snap.surface_count, snap.title, svg.len(), pdf_bytes.len(), png_bytes.len()
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

    shell
        .bridge
        .submit_command(CommandRequest::new(Command::CreateSurface {
            id: surface_1,
            name: "Main Artboard".to_string(),
        }))?;
    shell
        .bridge
        .set_surface_geometry(surface_1, [0.0, 0.0], [800.0, 600.0])?;
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
        aubrieta_document::ShapeKind::Rectangle {
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
        aubrieta_document::ShapeKind::Star {
            points: 5,
            inner_ratio: 0.45,
        },
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
                        None => {
                            if o.is_container() {
                                "Group"
                            } else {
                                "Rectangle"
                            }
                        }
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
                depth: (r.depth as f32 * 16.0),
                is_container: r.is_container,
                is_clip_mask: r.is_clip_mask,
                is_clipped: r.clip_mask_id.is_some(),
            }
        })
        .collect();
    window.set_layer_rows(Rc::new(VecModel::from(layer_items)).into());

    // Sync Canvas Objects
    let mut canvas_items = Vec::new();
    if let Some(session) = state.shell.bridge.session() {
        let selection = state.shell.bridge.selection();
        for surface in &session.document().surfaces {
            let sb = surface.bounds();
            for obj in &surface.objects {
                if let Some(b) = obj.bounds {
                    let is_sel = selection.contains(obj.id);
                    let name_lower = obj.name.to_lowercase();
                    let (is_circle, is_path, is_text, text_content, svg_path, corner_radius) =
                        match &obj.shape {
                            Some(aubrieta_document::ShapeKind::Ellipse) => {
                                (true, false, false, String::new(), String::new(), 0.0)
                            }
                            Some(aubrieta_document::ShapeKind::Rectangle { corner_radii }) => (
                                false,
                                false,
                                false,
                                String::new(),
                                String::new(),
                                corner_radii[0] as f32,
                            ),
                            Some(aubrieta_document::ShapeKind::Path(path)) => {
                                let local_path = path.transformed(GAffine::translate(-b[0], -b[1]));
                                (
                                    false,
                                    true,
                                    false,
                                    String::new(),
                                    local_path.to_svg_path_data(),
                                    0.0,
                                )
                            }
                            Some(aubrieta_document::ShapeKind::Polygon { .. })
                            | Some(aubrieta_document::ShapeKind::Star { .. }) => {
                                let local_path =
                                    obj.to_path().transformed(GAffine::translate(-b[0], -b[1]));
                                (
                                    false,
                                    true,
                                    false,
                                    String::new(),
                                    local_path.to_svg_path_data(),
                                    0.0,
                                )
                            }
                            Some(aubrieta_document::ShapeKind::Text { content, .. }) => {
                                (false, false, true, content.clone(), String::new(), 0.0)
                            }
                            None => {
                                let is_c =
                                    name_lower.contains("circle") || name_lower.contains("ellipse");
                                (is_c, false, false, String::new(), String::new(), 0.0)
                            }
                        };

                    let stroke_color = if let Some(stroke) = &obj.stroke {
                        if stroke.contains("blue") {
                            slint::Color::from_rgb_u8(37, 99, 235)
                        } else if stroke.contains("yellow") {
                            slint::Color::from_rgb_u8(202, 138, 4)
                        } else if stroke.contains("red") || stroke.contains("rose") {
                            slint::Color::from_rgb_u8(225, 29, 72)
                        } else if stroke.contains("gray") {
                            slint::Color::from_rgb_u8(39, 39, 42)
                        } else if stroke.starts_with('#') && stroke.len() == 7 {
                            let r = u8::from_str_radix(&stroke[1..3], 16).unwrap_or(37);
                            let g = u8::from_str_radix(&stroke[3..5], 16).unwrap_or(99);
                            let b_val = u8::from_str_radix(&stroke[5..7], 16).unwrap_or(235);
                            slint::Color::from_rgb_u8(r, g, b_val)
                        } else {
                            slint::Color::from_rgb_u8(37, 99, 235)
                        }
                    } else {
                        slint::Color::from_argb_u8(0, 0, 0, 0)
                    };
                    let stroke_width = (obj.stroke_width as f32).max(0.0);

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
                        } else if fill.contains("gray") {
                            slint::Color::from_rgb_u8(113, 113, 122)
                        } else if fill.starts_with('#') && fill.len() == 7 {
                            let r = u8::from_str_radix(&fill[1..3], 16).unwrap_or(59);
                            let g = u8::from_str_radix(&fill[3..5], 16).unwrap_or(130);
                            let b_val = u8::from_str_radix(&fill[5..7], 16).unwrap_or(246);
                            slint::Color::from_rgb_u8(r, g, b_val)
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
                    let is_gradient = obj.appearance.as_ref().is_some_and(|app| {
                        app.fills.iter().any(|f| {
                            matches!(
                                f.paint,
                                aubrieta_document::Paint::LinearGradient(_)
                                    | aubrieta_document::Paint::RadialGradient(_)
                            )
                        })
                    });

                    canvas_items.push(CanvasObjectItem {
                        id: obj.id.to_string().into(),
                        name: obj.name.clone().into(),
                        x: rel_x,
                        y: rel_y,
                        w,
                        h,
                        bg_color,
                        stroke_color,
                        stroke_width,
                        corner_radius,
                        selected: is_sel,
                        is_circle,
                        is_path,
                        is_text,
                        text_content: text_content.into(),
                        svg_path: svg_path.into(),
                        is_gradient,
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
                format!(
                    "X: {:.1} pt   Y: {:.1} pt   W: {:.1} pt   H: {:.1} pt",
                    b[0], b[1], b[2], b[3]
                )
                .into(),
            );
        }
        if let Some(f) = props.fill {
            window.set_selected_fill(f.into());
        }
        window.set_selected_opacity((props.opacity * 100.0) as f32);

        // Sync Appearance Stack (10.4)
        let (blend_mode_str, fill_items, stroke_items) = if let Some(app) = &props.appearance {
            let bm = format!("{:?}", app.blend_mode);
            let fills: Vec<AppearanceFillItem> = app
                .fills
                .iter()
                .map(|f| {
                    let (ptype, col) = match &f.paint {
                        aubrieta_document::Paint::Solid(c) => ("Solid", c.clone()),
                        aubrieta_document::Paint::LinearGradient(_) => {
                            ("Linear", "Linear Gradient".to_string())
                        }
                        aubrieta_document::Paint::RadialGradient(_) => {
                            ("Radial", "Radial Gradient".to_string())
                        }
                        aubrieta_document::Paint::None => ("None", "None".to_string()),
                    };
                    AppearanceFillItem {
                        id: f.id as i32,
                        paint_type: ptype.into(),
                        color_label: col.into(),
                        opacity_pct: (f.opacity * 100.0).round() as i32,
                        visible: f.visible,
                    }
                })
                .collect();
            let strokes: Vec<AppearanceStrokeItem> = app
                .strokes
                .iter()
                .map(|s| {
                    let (ptype, col) = match &s.paint {
                        aubrieta_document::Paint::Solid(c) => ("Solid", c.clone()),
                        aubrieta_document::Paint::LinearGradient(_) => {
                            ("Linear", "Linear Gradient".to_string())
                        }
                        aubrieta_document::Paint::RadialGradient(_) => {
                            ("Radial", "Radial Gradient".to_string())
                        }
                        aubrieta_document::Paint::None => ("None", "None".to_string()),
                    };
                    AppearanceStrokeItem {
                        id: s.id as i32,
                        paint_type: ptype.into(),
                        color_label: col.into(),
                        width_pt: s.width as f32,
                        opacity_pct: (s.opacity * 100.0).round() as i32,
                        visible: s.visible,
                    }
                })
                .collect();
            (bm, fills, strokes)
        } else {
            ("Normal".to_string(), Vec::new(), Vec::new())
        };

        window.set_prop_blend_mode(blend_mode_str.into());
        window.set_prop_fills(Rc::new(VecModel::from(fill_items)).into());
        window.set_prop_strokes(Rc::new(VecModel::from(stroke_items)).into());
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
        window.set_prop_blend_mode("Normal".into());
        window.set_prop_fills(Rc::new(VecModel::from(Vec::new())).into());
        window.set_prop_strokes(Rc::new(VecModel::from(Vec::new())).into());
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
    let src = merge
        .sources
        .first()
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "none".to_string());
    window.set_merge_source(src.into());
    window.set_merge_records(merge.total_records as i32);
    window.set_merge_bindings(merge.bindings.len() as i32);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args
        .iter()
        .any(|a| a == "--smoke-test" || a == "--headless")
    {
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
                "Corner" => ToolKind::Corner,
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
                    ToolKind::Corner => "Corner: Arraste sobre vértices para ajustar o raio de arredondamento.",
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
            if let Some(surface) = st
                .shell
                .bridge
                .session()
                .and_then(|s| s.document.surfaces.first())
            {
                let b = surface.bounds();
                st.shell
                    .fit_surface(GRect::new(b[0], b[1], b[0] + b[2], b[1] + b[3]));
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
                let title = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Novo Documento");
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
                .add_filter(
                    "Imagens Raster/Vetoriais (*.png, *.jpg, *.jpeg, *.svg)",
                    &["png", "jpg", "jpeg", "svg"],
                )
                .set_title("Inserir Imagem no Documento")
                .pick_file()
            {
                println!("RFD: Inserindo imagem: {:?}", path);
                let mut st = state_clone.borrow_mut();
                let mut id_gen = IdGenerator::new();
                let obj_id = id_gen.next_object();
                if let Some(surface) = st
                    .shell
                    .bridge
                    .session()
                    .and_then(|s| s.document.surfaces.first().cloned())
                {
                    let file_stem = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Imagem");
                    let _ = st.shell.bridge.submit_command(CommandRequest::new(
                        Command::CreateObject {
                            surface: surface.id,
                            id: obj_id,
                            name: format!("Imagem: {file_stem}"),
                        },
                    ));
                    let _ =
                        st.shell
                            .bridge
                            .set_bounds(obj_id, Some([180.0, 180.0, 240.0, 160.0]), 0.0);
                    let _ = st
                        .shell
                        .bridge
                        .set_fill(obj_id, Some("aubrieta.green/500".to_string()));
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
            let layers = st.shell.query_layers();
            if let Some(r) = layers.rows.get(idx as usize) {
                let obj_id = r.id;
                st.shell.bridge.set_selection(vec![obj_id]);
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
            if let Some(surface) = st
                .shell
                .bridge
                .session()
                .and_then(|s| s.document.surfaces.first().cloned())
            {
                let new_id = id_gen.next_object();
                let count = surface.objects.len() + 1;
                let _ =
                    st.shell
                        .bridge
                        .submit_command(CommandRequest::new(Command::CreateObject {
                            surface: surface.id,
                            id: new_id,
                            name: format!("Rectangle {}", count),
                        }));
                let offset = (count as f64 * 35.0) % 250.0;
                let _ = st.shell.bridge.set_bounds(
                    new_id,
                    Some([120.0 + offset, 120.0 + offset, 200.0, 130.0]),
                    0.0,
                );
                let _ = st
                    .shell
                    .bridge
                    .set_fill(new_id, Some("aubrieta.green/500".to_string()));
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
            if let Some(surface) = st
                .shell
                .bridge
                .session()
                .and_then(|s| s.document.surfaces.first().cloned())
            {
                let new_id = id_gen.next_object();
                let count = surface.objects.len() + 1;
                let _ =
                    st.shell
                        .bridge
                        .submit_command(CommandRequest::new(Command::CreateObject {
                            surface: surface.id,
                            id: new_id,
                            name: format!("Circle {}", count),
                        }));
                let offset = (count as f64 * 35.0) % 250.0;
                let _ = st.shell.bridge.set_bounds(
                    new_id,
                    Some([360.0 + offset, 180.0 + offset, 130.0, 130.0]),
                    0.0,
                );
                let _ = st
                    .shell
                    .bridge
                    .set_fill(new_id, Some("aubrieta.purple/500".to_string()));
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
            if let Some(surface) = st
                .shell
                .bridge
                .session()
                .and_then(|s| s.document.surfaces.first().cloned())
            {
                let new_id = id_gen.next_object();
                let count = surface.objects.len() + 1;
                let offset = (count as f64 * 35.0) % 250.0;
                let _ = st.shell.bridge.create_shape_object(
                    surface.id,
                    new_id,
                    format!("Star {}", count),
                    aubrieta_document::ShapeKind::Star {
                        points: 5,
                        inner_ratio: 0.45,
                    },
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
            if let Some(surface) = st
                .shell
                .bridge
                .session()
                .and_then(|s| s.document.surfaces.first().cloned())
            {
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
                    if let Some(surface) = session.document().surfaces.first() {
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
                if let Some(surface) = st
                    .shell
                    .bridge
                    .session()
                    .and_then(|s| s.document.surfaces.first().cloned())
                {
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
                    if let Some(surface) = session.document().surfaces.first() {
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
                if let Some(surface) = st
                    .shell
                    .bridge
                    .session()
                    .and_then(|s| s.document.surfaces.first().cloned())
                {
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
                    if let Some(surface) = session.document().surfaces.first() {
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
                if let Some(surface) = st
                    .shell
                    .bridge
                    .session()
                    .and_then(|s| s.document.surfaces.first().cloned())
                {
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
                    if let Some(surface) = session.document().surfaces.first() {
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
                if let Some(surface) = st
                    .shell
                    .bridge
                    .session()
                    .and_then(|s| s.document.surfaces.first().cloned())
                {
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
                    st.shell
                        .bridge
                        .session()
                        .and_then(|s| s.document.surfaces.first().map(|sf| sf.id))
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
                    st.shell
                        .bridge
                        .session()
                        .and_then(|s| s.document.surfaces.first().map(|sf| sf.id))
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
                let _ = st
                    .shell
                    .bridge
                    .submit_command(CommandRequest::new(Command::DeleteObject { id }));
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
                    if let Some(obj) = session
                        .document
                        .surfaces
                        .iter()
                        .flat_map(|s| &s.objects)
                        .find(|o| o.id == sel_id)
                    {
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
                    if let Some(obj) = session
                        .document
                        .surfaces
                        .iter()
                        .flat_map(|s| &s.objects)
                        .find(|o| o.id == sel_id)
                    {
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
                    if let Some(obj) = session
                        .document
                        .surfaces
                        .iter()
                        .flat_map(|s| &s.objects)
                        .find(|o| o.id == sel_id)
                    {
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
                    if let Some(obj) = session
                        .document
                        .surfaces
                        .iter()
                        .flat_map(|s| &s.objects)
                        .find(|o| o.id == sel_id)
                    {
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
                let _ = st
                    .shell
                    .bridge
                    .submit_command(CommandRequest::new(Command::SetOpacity {
                        id: sel_id,
                        opacity,
                    }));
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Blend mode clicked
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_set_blend_mode_clicked(move |mode_str| {
            let mut st = state_clone.borrow_mut();
            let mode = match mode_str.as_str() {
                "Multiply" => aubrieta_document::BlendMode::Multiply,
                "Screen" => aubrieta_document::BlendMode::Screen,
                "Overlay" => aubrieta_document::BlendMode::Overlay,
                "Darken" => aubrieta_document::BlendMode::Darken,
                "Lighten" => aubrieta_document::BlendMode::Lighten,
                "Difference" => aubrieta_document::BlendMode::Difference,
                "Exclusion" => aubrieta_document::BlendMode::Exclusion,
                "ColorDodge" => aubrieta_document::BlendMode::ColorDodge,
                "ColorBurn" => aubrieta_document::BlendMode::ColorBurn,
                "HardLight" => aubrieta_document::BlendMode::HardLight,
                "SoftLight" => aubrieta_document::BlendMode::SoftLight,
                "Hue" => aubrieta_document::BlendMode::Hue,
                "Saturation" => aubrieta_document::BlendMode::Saturation,
                "Color" => aubrieta_document::BlendMode::Color,
                "Luminosity" => aubrieta_document::BlendMode::Luminosity,
                _ => aubrieta_document::BlendMode::Normal,
            };
            let sel_ids = st.shell.bridge.selection().selected_ids;
            for id in sel_ids {
                let _ = st.shell.bridge.set_blend_mode(id, mode);
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Add fill clicked
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_add_fill_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let sel_ids = st.shell.bridge.selection().selected_ids;
            for id in sel_ids {
                let _ = st.shell.bridge.add_fill(
                    id,
                    aubrieta_document::Paint::Solid("aubrieta.rose/500".to_string()),
                );
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Remove fill clicked
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_remove_fill_clicked(move |fill_id| {
            let mut st = state_clone.borrow_mut();
            let sel_ids = st.shell.bridge.selection().selected_ids;
            for id in sel_ids {
                let _ = st.shell.bridge.remove_fill(id, fill_id as u32);
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Add stroke clicked
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_add_stroke_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let sel_ids = st.shell.bridge.selection().selected_ids;
            for id in sel_ids {
                let _ = st.shell.bridge.add_stroke(
                    id,
                    aubrieta_document::Paint::Solid("aubrieta.blue/500".to_string()),
                    2.0,
                );
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Remove stroke clicked
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_remove_stroke_clicked(move |stroke_id| {
            let mut st = state_clone.borrow_mut();
            let sel_ids = st.shell.bridge.selection().selected_ids;
            for id in sel_ids {
                let _ = st.shell.bridge.remove_stroke(id, stroke_id as u32);
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Gradient style clicked
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_set_gradient_clicked(move |kind| {
            let mut st = state_clone.borrow_mut();
            let sel_ids = st.shell.bridge.selection().selected_ids;
            for id in sel_ids {
                match kind.as_str() {
                    "linear" => {
                        let _ = st.shell.bridge.set_linear_gradient_fill(
                            id,
                            "aubrieta.blue/500",
                            "aubrieta.purple/500",
                        );
                    }
                    "radial" => {
                        let _ = st.shell.bridge.set_radial_gradient_fill(
                            id,
                            "aubrieta.yellow/500",
                            "aubrieta.rose/500",
                        );
                    }
                    _ => {
                        let _ = st
                            .shell
                            .bridge
                            .set_fill(id, Some("aubrieta.blue/500".to_string()));
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
        main_window.on_duplicate_selected_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let mut new_id = None;
            let clone_info =
                if let Some(sel_id) = st.shell.bridge.selection().selected_ids.first().copied() {
                    if let Some(session) = st.shell.bridge.session() {
                        if let Some(surface) = session.document().surfaces.first() {
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
                let _ =
                    st.shell
                        .bridge
                        .submit_command(CommandRequest::new(Command::CreateObject {
                            surface: surf_id,
                            id: clone_id,
                            name,
                        }));
                let _ = st
                    .shell
                    .bridge
                    .set_bounds(clone_id, Some(clone_bounds), 0.0);
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

    // Layer visibility toggle
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_toggle_layer_visibility(move |idx| {
            let mut st = state_clone.borrow_mut();
            let layers = st.shell.query_layers();
            if let Some(r) = layers.rows.get(idx as usize) {
                let id = r.id;
                let visible = !r.visible;
                let _ = st
                    .shell
                    .bridge
                    .submit_command(CommandRequest::new(Command::SetVisibility { id, visible }));
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Layer lock toggle
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_toggle_layer_lock(move |idx| {
            let mut st = state_clone.borrow_mut();
            let layers = st.shell.query_layers();
            if let Some(r) = layers.rows.get(idx as usize) {
                let id = r.id;
                let locked = !r.locked;
                let _ = st
                    .shell
                    .bridge
                    .submit_command(CommandRequest::new(Command::SetLocked { id, locked }));
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Reorder layer up
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_reorder_layer_up(move |idx| {
            let mut st = state_clone.borrow_mut();
            if idx > 0 {
                let layers = st.shell.query_layers();
                if let Some(r) = layers.rows.get(idx as usize) {
                    let surf_id = r.surface_id;
                    let id = r.id;
                    let new_index = (idx - 1) as usize;
                    let _ = st
                        .shell
                        .bridge
                        .submit_command(CommandRequest::new(Command::ReorderObject {
                            surface: surf_id,
                            id,
                            new_index,
                        }));
                }
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Reorder layer down
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_reorder_layer_down(move |idx| {
            let mut st = state_clone.borrow_mut();
            let layers = st.shell.query_layers();
            if (idx as usize) + 1 < layers.rows.len() {
                if let Some(r) = layers.rows.get(idx as usize) {
                    let surf_id = r.surface_id;
                    let id = r.id;
                    let new_index = (idx + 1) as usize;
                    let _ = st
                        .shell
                        .bridge
                        .submit_command(CommandRequest::new(Command::ReorderObject {
                            surface: surf_id,
                            id,
                            new_index,
                        }));
                }
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Hierarchy & Grouping Callbacks (10.5 - Step 2)
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_group_selected_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let surface_opt = st.shell.bridge.active_surface().or_else(|| {
                st.shell
                    .bridge
                    .session()
                    .and_then(|s| s.document.surfaces.first().map(|surf| surf.id))
            });
            let selected_ids = st.shell.bridge.selection().selected_ids;
            if let (Some(surface), false) = (surface_opt, selected_ids.is_empty()) {
                if let Ok(group_id) = st.shell.bridge.next_object_id() {
                    let _ = st.shell.bridge.group_objects(
                        surface,
                        group_id,
                        selected_ids,
                        ContainerRole::Group,
                    );
                    st.shell.bridge.set_selection(vec![group_id]);
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
        main_window.on_ungroup_selected_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let selected_ids = st.shell.bridge.selection().selected_ids;
            for id in selected_ids {
                let is_group = st
                    .shell
                    .bridge
                    .session()
                    .and_then(|s| {
                        s.document
                            .find_object(id)
                            .map(|o| o.is_container() || o.role.is_some())
                    })
                    .unwrap_or(false);

                if is_group {
                    let _ = st.shell.bridge.ungroup(id);
                } else {
                    let parent_opt = st
                        .shell
                        .bridge
                        .session()
                        .and_then(|s| s.document.find_object(id).and_then(|o| o.parent));
                    if let Some(parent_id) = parent_opt {
                        let _ = st.shell.bridge.ungroup(parent_id);
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
        main_window.on_create_clip_mask_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let surface_opt = st.shell.bridge.active_surface().or_else(|| {
                st.shell
                    .bridge
                    .session()
                    .and_then(|s| s.document.surfaces.first().map(|surf| surf.id))
            });
            let selected_ids = st.shell.bridge.selection().selected_ids;
            if let (Some(surface), true) = (surface_opt, selected_ids.len() >= 2) {
                if let Ok(group_id) = st.shell.bridge.next_object_id() {
                    let mask_id = selected_ids[0];
                    let content_ids = selected_ids[1..].to_vec();
                    let _ =
                        st.shell
                            .bridge
                            .create_clip_group(surface, group_id, mask_id, content_ids);
                    st.shell.bridge.set_selection(vec![group_id]);
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
        main_window.on_release_clip_mask_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let selected_ids = st.shell.bridge.selection().selected_ids;
            for id in selected_ids {
                let clip_group = st.shell.bridge.session().and_then(|s| {
                    let obj = s.document.find_object(id)?;
                    if obj.role == Some(ContainerRole::ClipGroup) {
                        Some(obj.id)
                    } else if let Some(pid) = obj.parent {
                        let parent = s.document.find_object(pid)?;
                        if parent.role == Some(ContainerRole::ClipGroup) {
                            Some(pid)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });
                if let Some(cg_id) = clip_group {
                    let _ = st.shell.bridge.release_clip_group(cg_id);
                }
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Keyboard Shortcuts & Selection Navigation (Step 3)
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_nudge_selected(move |dx, dy| {
            let mut st = state_clone.borrow_mut();
            let sel_ids = st.shell.bridge.selection().selected_ids;
            for id in sel_ids {
                let current = st.shell.bridge.session().and_then(|s| {
                    s.document.find_object(id).map(|o| (o.bounds, o.rotation))
                });
                if let Some((Some(b), rot)) = current {
                    let new_b = [b[0] + dx as f64, b[1] + dy as f64, b[2], b[3]];
                    let _ = st.shell.bridge.set_bounds(id, Some(new_b), rot);
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
        main_window.on_select_all_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let all_ids: Vec<ObjectId> = if let Some(session) = st.shell.bridge.session() {
                let surf = st
                    .shell
                    .bridge
                    .active_surface()
                    .and_then(|sid| session.document().surfaces.iter().find(|s| s.id == sid))
                    .or_else(|| session.document().surfaces.first());
                surf.map(|s| s.objects.iter().map(|o| o.id).collect())
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            if !all_ids.is_empty() {
                st.shell.bridge.set_selection(all_ids);
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_cycle_selection_clicked(move |forward| {
            let mut st = state_clone.borrow_mut();
            if let Some(session) = st.shell.bridge.session() {
                let surf = st
                    .shell
                    .bridge
                    .active_surface()
                    .and_then(|sid| session.document().surfaces.iter().find(|s| s.id == sid))
                    .or_else(|| session.document().surfaces.first());
                if let Some(surface) = surf {
                    let objects: Vec<ObjectId> = surface.objects.iter().map(|o| o.id).collect();
                    if !objects.is_empty() {
                        let current_sel = st.shell.bridge.selection().selected_ids.first().copied();
                        let current_idx =
                            current_sel.and_then(|id| objects.iter().position(|&o| o == id));
                        let next_idx = match current_idx {
                            None => 0,
                            Some(idx) => {
                                if forward {
                                    (idx + 1) % objects.len()
                                } else {
                                    (idx + objects.len() - 1) % objects.len()
                                }
                            }
                        };
                        st.shell.bridge.set_selection(vec![objects[next_idx]]);
                    }
                }
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Direct Export Flow in UI (Step 4)
    {
        let win_weak = main_window.as_weak();
        main_window.on_open_export_dialog_clicked(move || {
            if let Some(win) = win_weak.upgrade() {
                win.set_export_status_message("".into());
                win.set_export_dialog_open(true);
            }
        });
    }

    {
        let win_weak = main_window.as_weak();
        main_window.on_close_export_dialog_clicked(move || {
            if let Some(win) = win_weak.upgrade() {
                win.set_export_dialog_open(false);
            }
        });
    }

    {
        let win_weak = main_window.as_weak();
        main_window.on_set_export_format(move |fmt| {
            if let Some(win) = win_weak.upgrade() {
                win.set_export_format(fmt);
            }
        });
    }

    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_do_export_clicked(move || {
            let st = state_clone.borrow();
            let fmt = if let Some(win) = win_weak.upgrade() {
                win.get_export_format().to_string()
            } else {
                "png".to_string()
            };

            let session = match st.shell.bridge.session() {
                Some(s) => s,
                None => return,
            };

            match fmt.as_str() {
                "svg" => {
                    let dialog = rfd::FileDialog::new()
                        .add_filter("SVG Vector (*.svg)", &["svg"])
                        .set_file_name("export.svg");
                    if let Some(path) = dialog.save_file() {
                        let svg_content = export_document_svg(&session.document());
                        match std::fs::write(&path, svg_content.as_bytes()) {
                            Ok(()) => {
                                if let Some(win) = win_weak.upgrade() {
                                    win.set_export_status_message(
                                        format!("SVG exportado: {}", path.display()).into(),
                                    );
                                }
                            }
                            Err(e) => {
                                if let Some(win) = win_weak.upgrade() {
                                    win.set_export_status_message(
                                        format!("Erro SVG: {e}").into(),
                                    );
                                }
                            }
                        }
                    }
                }
                "pdf" => {
                    let dialog = rfd::FileDialog::new()
                        .add_filter("PDF Document (*.pdf)", &["pdf"])
                        .set_file_name("export.pdf");
                    if let Some(path) = dialog.save_file() {
                        let options = PdfExportOptions::default();
                        match export_document_pdf(&session.document(), &options) {
                            Ok((pdf_bytes, _)) => {
                                match std::fs::write(&path, &pdf_bytes) {
                                    Ok(()) => {
                                        if let Some(win) = win_weak.upgrade() {
                                            win.set_export_status_message(
                                                format!("PDF exportado: {}", path.display()).into(),
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        if let Some(win) = win_weak.upgrade() {
                                            win.set_export_status_message(
                                                format!("Erro ao gravar PDF: {e}").into(),
                                            );
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                if let Some(win) = win_weak.upgrade() {
                                    win.set_export_status_message(
                                        format!("Erro PDF: {e}").into(),
                                    );
                                }
                            }
                        }
                    }
                }
                _ => {
                    // PNG (Raster)
                    let dialog = rfd::FileDialog::new()
                        .add_filter("PNG Image (*.png)", &["png"])
                        .set_file_name("export.png");
                    if let Some(path) = dialog.save_file() {
                        if let Some(surface) = session.document().surfaces.first() {
                            let w = (surface.dimensions[0].round() as usize).max(10);
                            let h = (surface.dimensions[1].round() as usize).max(10);
                            let mut buffer = vec![255u8; w * h * 4];
                            for obj in &surface.objects {
                                if !obj.visible {
                                    continue;
                                }
                                if let Some(b) = obj.bounds {
                                    let ox = (b[0] - surface.origin[0]).max(0.0) as usize;
                                    let oy = (b[1] - surface.origin[1]).max(0.0) as usize;
                                    let ow = (b[2] as usize).min(w.saturating_sub(ox));
                                    let oh = (b[3] as usize).min(h.saturating_sub(oy));
                                    let (cr, cg, cb, ca) = if let Some(fill) = &obj.fill {
                                        if fill.contains("blue") {
                                            (59u8, 130u8, 246u8, 255u8)
                                        } else if fill.contains("yellow") {
                                            (234, 179, 8, 255)
                                        } else if fill.contains("green") {
                                            (16, 185, 129, 255)
                                        } else if fill.contains("purple") {
                                            (139, 92, 246, 255)
                                        } else if fill.contains("rose") || fill.contains("red") {
                                            (244, 63, 94, 255)
                                        } else {
                                            (100, 116, 139, 255)
                                        }
                                    } else {
                                        (59, 130, 246, 255)
                                    };
                                    let is_circle = matches!(
                                        obj.shape,
                                        Some(aubrieta_document::ShapeKind::Ellipse)
                                    );
                                    for py in 0..oh {
                                        for px in 0..ow {
                                            if is_circle {
                                                let rx = ow as f64 / 2.0;
                                                let ry = oh as f64 / 2.0;
                                                let dx = (px as f64 - rx) / rx.max(1.0);
                                                let dy = (py as f64 - ry) / ry.max(1.0);
                                                if dx * dx + dy * dy > 1.0 {
                                                    continue;
                                                }
                                            }
                                            let idx = ((oy + py) * w + (ox + px)) * 4;
                                            if idx + 3 < buffer.len() {
                                                let alpha = ca as f32 / 255.0;
                                                buffer[idx] = (cr as f32 * alpha
                                                    + buffer[idx] as f32 * (1.0 - alpha))
                                                    as u8;
                                                buffer[idx + 1] = (cg as f32 * alpha
                                                    + buffer[idx + 1] as f32 * (1.0 - alpha))
                                                    as u8;
                                                buffer[idx + 2] = (cb as f32 * alpha
                                                    + buffer[idx + 2] as f32 * (1.0 - alpha))
                                                    as u8;
                                                buffer[idx + 3] = 255;
                                            }
                                        }
                                    }
                                }
                            }
                            if let Ok(raw) =
                                RawRasterImage::from_rgba8(w as u32, h as u32, buffer)
                            {
                                let opts = RasterExportOptions {
                                    format: RasterFormat::Png,
                                    jpeg_quality: 90,
                                    allow_degradations: true,
                                };
                                match export_raster(&raw, &opts) {
                                    Ok((png_bytes, _)) => match std::fs::write(&path, &png_bytes)
                                    {
                                        Ok(()) => {
                                            if let Some(win) = win_weak.upgrade() {
                                                win.set_export_status_message(
                                                    format!("PNG exportado: {}", path.display())
                                                        .into(),
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            if let Some(win) = win_weak.upgrade() {
                                                win.set_export_status_message(
                                                    format!("Erro ao gravar PNG: {e}").into(),
                                                );
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        if let Some(win) = win_weak.upgrade() {
                                            win.set_export_status_message(
                                                format!("Erro PNG: {e}").into(),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
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

            let evt = NormalizedPointerEvent::new(
                PointerPhase::Down,
                PointerButton::Primary,
                screen_pt,
                doc_pt,
                SemanticModifiers::default(),
            );
            let _ = st.shell.handle_pointer_event(&evt);

            if let Some(win) = win_weak.upgrade() {
                win.set_status_coords(
                    format!("Doc: X: {:.1} pt  Y: {:.1} pt", doc_pt.x, doc_pt.y).into(),
                );
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

            let evt = NormalizedPointerEvent::new(
                PointerPhase::Move,
                PointerButton::Primary,
                screen_pt,
                doc_pt,
                SemanticModifiers::default(),
            );
            let _ = st.shell.handle_pointer_event(&evt);

            let cursor_kind =
                if let Some([bx, by, bw, bh]) = st.shell.bridge.selection().combined_bounds {
                    let doc_box = GRect::new(bx, by, bx + bw, by + bh);
                    if let Some(handle) = hit_test_handle_or_border(
                        doc_box,
                        screen_pt,
                        doc_pt,
                        &st.shell.camera,
                        14.0,
                        8.0,
                    ) {
                        match handle {
                            SelectionHandleKind::TopLeft | SelectionHandleKind::BottomRight => {
                                "nwse-resize"
                            }
                            SelectionHandleKind::TopRight | SelectionHandleKind::BottomLeft => {
                                "nesw-resize"
                            }
                            SelectionHandleKind::Top | SelectionHandleKind::Bottom => "ns-resize",
                            SelectionHandleKind::Left | SelectionHandleKind::Right => "ew-resize",
                            SelectionHandleKind::Rotation => "crosshair",
                        }
                    } else if doc_box.contains(doc_pt) {
                        "grab"
                    } else {
                        "default"
                    }
                } else {
                    match st.shell.active_tool() {
                        ToolKind::Pen
                        | ToolKind::Pencil
                        | ToolKind::Knife
                        | ToolKind::Rectangle
                        | ToolKind::Ellipse
                        | ToolKind::Star
                        | ToolKind::Polygon => "crosshair",
                        ToolKind::ArtisticText | ToolKind::FrameText => "text",
                        ToolKind::Hand => "grab",
                        _ => "default",
                    }
                };

            if let Some(win) = win_weak.upgrade() {
                win.set_status_coords(
                    format!("Doc: X: {:.1} pt  Y: {:.1} pt", doc_pt.x, doc_pt.y).into(),
                );
                win.set_canvas_cursor_kind(cursor_kind.into());

                let overlays = st.shell.overlays();
                if let Some(marquee) = overlays.marquee_screen {
                    let ax = win.get_artboard_x() as f64;
                    let ay = win.get_artboard_y() as f64;
                    win.set_has_preview(true);
                    win.set_preview_x((marquee.x0.min(marquee.x1) - ax).max(0.0) as f32);
                    win.set_preview_y((marquee.y0.min(marquee.y1) - ay).max(0.0) as f32);
                    win.set_preview_w(marquee.width().abs() as f32);
                    win.set_preview_h(marquee.height().abs() as f32);
                } else {
                    win.set_has_preview(false);
                }

                sync_ui_from_shell(&win, &st);
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

            st.drag_start_doc = None;
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
