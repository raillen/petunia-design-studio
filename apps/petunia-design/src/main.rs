//! Petunia Design Studio — Slint primary GUI.
//!
//! Evaluates Slint declarative UI toolkit, consuming PetuniaDesignGuiBridge
//! presentation models and dispatching interactive commands.

slint::include_modules!();

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use petunia_design_application::{Command, CommandRequest};
use petunia_design_document::{Bleed, ContainerRole, Guide, GuideOrientation, Margins};
use petunia_design_foundation::{PetuniaError, ObjectId};
use petunia_design_io::{
    export_document_pdf, export_document_svg, export_raster, PdfExportOptions,
    RasterExportOptions, RasterFormat, RawRasterImage,
};
use petunia_design_geometry::{GAffine, GPoint, GRect};
use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase, SemanticModifiers,
};
use petunia_design_application::tools::ToolKind;
use petunia_design_shell::tools::MarqueeSelectRule;
use petunia_design_shell::bridge::{
    DataMergePresentationModel, HistoryPresentationModel, LayersPresentationModel,
    PropertiesPresentationModel,
};
use petunia_design_shell::canvas::overlay::{hit_test_handle_or_border, SelectionHandleKind};
use petunia_design_shell::shell::PetuniaShell;
use slint::{ComponentHandle, VecModel};

pub struct PetuniaSlintState {
    pub shell: PetuniaShell,
    pub drag_start_doc: Option<GPoint>,
    pub dragging_object_id: Option<ObjectId>,
    pub drag_initial_bounds: Option<[f64; 4]>,
    /// Command ids currently listed in the palette (for Enter-to-run-first).
    pub palette_filtered: Vec<String>,
}

impl Default for PetuniaSlintState {
    fn default() -> Self {
        Self::new()
    }
}

impl PetuniaSlintState {
    pub fn new() -> Self {
        let mut shell = PetuniaShell::new(950.0, 700.0);
        shell.camera.pan_x = 80.0;
        shell.camera.pan_y = 80.0;
        shell.camera.zoom = 1.0;
        let _ = populate_showcase_document(&mut shell);
        Self {
            shell,
            drag_start_doc: None,
            dragging_object_id: None,
            drag_initial_bounds: None,
            palette_filtered: Vec::new(),
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
                petunia_design_document::ShapeKind::Ellipse,
                Some([150.0, 150.0, 100.0, 100.0]),
                Some("ptnd.purple/500".to_string()),
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
        let svg = export_document_svg(session.document());
        if !svg.starts_with("<svg") && !svg.contains("<svg") {
            return Err("SVG export produced invalid XML envelope".to_string());
        }

        let (pdf_bytes, report) =
            export_document_pdf(session.document(), &PdfExportOptions::default())
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

        // 5. Command palette dispatch (state-level, no window needed).
        if !run_palette_command(self, "no-such-command") {
        } else {
            return Err("palette ran an unknown command".to_string());
        }
        let zoom_before = self.shell.camera.zoom;
        // zoom commands report no resync needed by design; the zoom
        // change itself proves dispatch ran.
        run_palette_command(self, "zoom-in");
        if self.shell.camera.zoom <= zoom_before {
            return Err("palette zoom-in did not change zoom".to_string());
        }
        println!(
            "OK petunia-design smoke test: surfaces={} title=\"{}\" svg_len={} pdf_len={} png_len={}",
            snap.surface_count, snap.title, svg.len(), pdf_bytes.len(), png_bytes.len()
        );
        Ok(())
    }
}

fn populate_showcase_document(shell: &mut PetuniaShell) -> Result<(), PetuniaError> {
    shell.new_document("Petunia Showcase Project [Slint]")?;
    // Allocate every ID from the session lane: parallel local generators
    // collide with the session-owned counter (Canvas takes SurfaceId(1)).
    let surface_1 = shell.bridge.next_surface_id()?;
    let rect_id = shell.bridge.next_object_id()?;
    let circle_id = shell.bridge.next_object_id()?;
    let star_id = shell.bridge.next_object_id()?;
    let text_id = shell.bridge.next_object_id()?;

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
        petunia_design_document::ShapeKind::Rectangle {
            corner_radii: [8.0; 4],
        },
        Some([100.0, 100.0, 300.0, 180.0]),
        Some("ptnd.blue/500".to_string()),
        Some("#2563eb".to_string()),
        1.5,
    )?;

    // Accent Circle (Ellipse)
    shell.bridge.create_shape_object(
        surface_1,
        circle_id,
        "Accent Circle".to_string(),
        petunia_design_document::ShapeKind::Ellipse,
        Some([450.0, 140.0, 140.0, 140.0]),
        Some("ptnd.yellow/500".to_string()),
        Some("#ca8a04".to_string()),
        1.5,
    )?;

    // Golden Star (Star)
    shell.bridge.create_shape_object(
        surface_1,
        star_id,
        "Golden Star".to_string(),
        petunia_design_document::ShapeKind::Star {
            points: 5,
            inner_ratio: 0.45,
        },
        Some([450.0, 320.0, 130.0, 130.0]),
        Some("ptnd.rose/500".to_string()),
        Some("#e11d48".to_string()),
        1.5,
    )?;

    // Title Text (Text)
    shell.bridge.create_shape_object(
        surface_1,
        text_id,
        "Banner Text".to_string(),
        petunia_design_document::ShapeKind::Text {
            content: "Petunia Design Studio".to_string(),
            font_family: "Inter".to_string(),
            font_size: 20.0,
            line_height: 24.0,
            letter_spacing: 0.5,
        },
        Some([100.0, 320.0, 300.0, 50.0]),
        Some("ptnd.purple/500".to_string()),
        None,
        0.0,
    )?;

    shell.bridge.set_selection(vec![rect_id]);
    // Showcase editing happens on the Main Artboard, not the initial canvas.
    shell.bridge.set_active_surface(surface_1)?;
    Ok(())
}

/// Resolves any color token/literal to an exact Slint color via the
/// canonical engine parser (no fuzzy substring matching).
fn token_to_slint(token: &str) -> slint::Color {
    let rgb = petunia_design_document::resolve_color_to_rgb(token);
    slint::Color::from_rgb_u8(
        (rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

/// Samples a paint to a flat Slint color (gradients at center stop).
fn paint_to_slint(paint: &petunia_design_document::Paint) -> Option<slint::Color> {
    match paint {
        petunia_design_document::Paint::None => None,
        petunia_design_document::Paint::Solid(token) => Some(token_to_slint(token)),
        petunia_design_document::Paint::LinearGradient(g) => g
            .sample_rgba(0.5)
            .map(|(rgb, _)| sample_to_slint(rgb)),
        petunia_design_document::Paint::RadialGradient(g) => g
            .sample_rgba(0.5)
            .map(|(rgb, _)| sample_to_slint(rgb)),
    }
}

fn sample_to_slint(rgb: [f32; 3]) -> slint::Color {
    slint::Color::from_rgb_u8(
        (rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

/// Cursor kind for the active tool when not hovering a selection handle.
/// Selection handles keep their resize cursors only under Select/Node.
fn tool_cursor_kind(tool: ToolKind) -> &'static str {
    match tool {
        ToolKind::Select | ToolKind::Node => "default",
        ToolKind::Pen
        | ToolKind::Pencil
        | ToolKind::Knife
        | ToolKind::Scissors
        | ToolKind::Rectangle
        | ToolKind::Ellipse
        | ToolKind::Polygon
        | ToolKind::Star
        | ToolKind::ShapeBuilder
        | ToolKind::VectorFloodFill
        | ToolKind::Contour
        | ToolKind::Corner
        | ToolKind::Zoom
        | ToolKind::ColorPicker
        | ToolKind::StylePicker
        | ToolKind::Measure => "crosshair",
        ToolKind::ArtisticText | ToolKind::FrameText => "text",
        ToolKind::Hand => "grab",
        _ => "default",
    }
}

/// Static command-palette catalog: (id, label, hint).
fn palette_catalog() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("undo", "Desfazer", "Ctrl+Z"),
        ("redo", "Refazer", "Ctrl+Y"),
        ("group", "Agrupar seleção", "Ctrl+G"),
        ("ungroup", "Desagrupar", "Ctrl+Shift+G"),
        ("clip-mask", "Criar máscara de recorte", ""),
        ("clip-release", "Liberar máscara", ""),
        ("align-left", "Alinhar à esquerda", ""),
        ("align-center", "Alinhar ao centro", ""),
        ("align-right", "Alinhar à direita", ""),
        ("align-top", "Alinhar ao topo", ""),
        ("align-middle", "Alinhar ao meio", ""),
        ("align-bottom", "Alinhar à base", ""),
        ("distribute-h", "Distribuir horizontalmente", ""),
        ("distribute-v", "Distribuir verticalmente", ""),
        ("bool-union", "Booleano: União", ""),
        ("bool-subtract", "Booleano: Subtração", ""),
        ("bool-intersect", "Booleano: Intersecção", ""),
        ("bool-xor", "Booleano: Exclusão", ""),
        ("convert-curves", "Converter em curvas", ""),
        ("bake-corners", "Bake cantos", ""),
        ("duplicate", "Duplicar seleção", "Ctrl+D"),
        ("delete", "Excluir seleção", "Delete"),
        ("select-all", "Selecionar tudo", "Ctrl+A"),
        ("zoom-in", "Aproximar zoom", ""),
        ("zoom-out", "Afastar zoom", ""),
        ("zoom-fit", "Ajustar à tela", ""),
        ("export", "Abrir exportação", "Ctrl+E"),
    ]
}

/// Pushes the filtered palette catalog to the UI, recording the listed
/// ids in state order for Enter-to-run-first.
fn push_palette_items(win: &MainWindow, query: &str, st: &mut PetuniaSlintState) {
    let q = query.to_lowercase();
    let items: Vec<PaletteItem> = palette_catalog()
        .into_iter()
        .filter(|(id, label, hint)| {
            q.is_empty()
                || label.to_lowercase().contains(&q)
                || id.contains(&q)
                || hint.to_lowercase().contains(&q)
        })
        .map(|(id, label, hint)| {
            st.palette_filtered.push(id.to_string());
            PaletteItem {
                id: id.into(),
                label: label.into(),
                hint: hint.into(),
            }
        })
        .collect();
    win.set_palette_items(Rc::new(VecModel::from(items)).into());
}

/// Runs one palette command id against the live shell. Returns true when
/// the UI should resync afterwards.
fn run_palette_command(state: &mut PetuniaSlintState, id: &str) -> bool {
    let st = state;
    match id {
        "undo" => {
            let _ = st.shell.undo();
            true
        }
        "redo" => {
            let _ = st.shell.redo();
            true
        }
        "group" => {
            let _ = st.shell.layers_panel.group_selection(
                &mut st.shell.bridge,
                petunia_design_document::ContainerRole::Group,
            );
            true
        }
        "ungroup" => {
            let _ = st
                .shell
                .layers_panel
                .ungroup_selection(&mut st.shell.bridge);
            true
        }
        "clip-mask" => {
            let _ = st
                .shell
                .layers_panel
                .create_clipping_mask(&mut st.shell.bridge);
            true
        }
        "clip-release" => {
            // Release selected clip groups; with an empty relevant
            // selection, release all clip groups on the active surface.
            let targets: Vec<ObjectId> = st
                .shell
                .bridge
                .session()
                .map(|s| {
                    let sel = st.shell.bridge.selection().selected_ids.clone();
                    let scope: Vec<ObjectId> = if sel.is_empty() {
                        match s.active_surface() {
                            Some(surf) => s
                                .surface(surf)
                                .map(|sf| {
                                    sf.objects().iter().map(|o| o.id).collect()
                                })
                                .unwrap_or_default(),
                            None => vec![],
                        }
                    } else {
                        sel
                    };
                    scope
                        .into_iter()
                        .filter(|oid| {
                            s.find_object(*oid).is_some_and(|o| {
                                o.is_container()
                                    && o.role
                                        == Some(petunia_design_document::ContainerRole::ClipGroup)
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            for gid in targets {
                let _ = st
                    .shell
                    .layers_panel
                    .release_clipping_mask(&mut st.shell.bridge, gid);
            }
            true
        }
        "delete" => {
            let ids = st.shell.bridge.selection().selected_ids.clone();
            for id in ids {
                let _ = st.shell.bridge.submit_command(CommandRequest::new(
                    Command::DeleteObject { id },
                ));
            }
            st.shell.bridge.set_selection(vec![]);
            true
        }
        "select-all" => {
            if let Some(session) = st.shell.bridge.session() {
                let _ = session;
            }
            // Selection lives in the session; select via surface objects.
            let all: Vec<ObjectId> = st
                .shell
                .bridge
                .session()
                .map(|s| {
                    s.document()
                        .surfaces()
                        .iter()
                        .flat_map(|sf| sf.objects())
                        .map(|o| o.id)
                        .collect()
                })
                .unwrap_or_default();
            st.shell.bridge.set_selection(all);
            true
        }
        "duplicate" => {
            // Offset duplicate of the primary selection via panel lane.
            let sel = st.shell.bridge.selection().selected_ids.clone();
            if let Some(first) = sel.first().copied() {
                if let Some(session) = st.shell.bridge.session() {
                    if let Some(obj) = session
                        .document()
                        .surfaces()
                        .iter()
                        .flat_map(|s| s.objects())
                        .find(|o| o.id == first)
                        .cloned()
                    {
                        if let Some(surface_id) = session
                            .document()
                            .surfaces()
                            .iter()
                            .find(|sf| sf.objects().iter().any(|o| o.id == first))
                            .map(|sf| sf.id)
                        {
                            if let Ok(new_id) = st.shell.bridge.next_object_id() {
                                let b = obj.bounds.unwrap_or([0.0, 0.0, 100.0, 100.0]);
                                let nb =
                                    [b[0] + 20.0, b[1] + 20.0, b[2], b[3]];
                                let cmds = vec![
                                    Command::CreateShapeObject {
                                        surface: surface_id,
                                        id: new_id,
                                        name: format!("{} Copy", obj.name),
                                        shape: obj.shape.clone().unwrap_or(
                                            petunia_design_document::ShapeKind::Rectangle {
                                                corner_radii: [0.0; 4],
                                            },
                                        ),
                                        bounds: Some(nb),
                                        fill: obj.fill.clone(),
                                        stroke: obj.stroke.clone(),
                                        stroke_width: obj.stroke_width,
                                    },
                                    Command::SetBounds {
                                        id: new_id,
                                        bounds: Some(nb),
                                        rotation: obj.rotation,
                                    },
                                ];
                                let _ = st.shell.bridge.submit_all("Duplicate object", cmds);
                                st.shell.bridge.set_selection(vec![new_id]);
                            }
                        }
                    }
                }
            }
            true
        }
        "zoom-in" => {
            st.shell.camera.set_zoom(st.shell.camera.zoom * 1.25);
            false
        }
        "zoom-out" => {
            st.shell.camera.set_zoom(st.shell.camera.zoom / 1.25);
            false
        }
        "zoom-fit" => {
            if let Some(session) = st.shell.bridge.session() {
                if let Some(surface) = session.document().surfaces().first() {
                    let b = surface.bounds();
                    st.shell.fit_surface(petunia_design_geometry::GRect::new(
                        b[0], b[1], b[0] + b[2], b[1] + b[3],
                    ));
                }
            }
            false
        }
        "export" => false,
        mode if mode.starts_with("align-") => {
            let sel = st.shell.bridge.selection().selected_ids.clone();
            if let Some(surface_id) = st.shell.bridge.active_surface().or_else(|| {
                st.shell
                    .bridge
                    .session()
                    .and_then(|s| s.document().surfaces().first().map(|sf| sf.id))
            }) {
                let amode = match mode {
                    "align-left" => petunia_design_document::AlignmentMode::Left,
                    "align-right" => petunia_design_document::AlignmentMode::Right,
                    "align-top" => petunia_design_document::AlignmentMode::Top,
                    "align-bottom" => petunia_design_document::AlignmentMode::Bottom,
                    "align-middle" => petunia_design_document::AlignmentMode::Middle,
                    _ => petunia_design_document::AlignmentMode::Center,
                };
                let _ = st.shell.bridge.align_objects(surface_id, sel, amode);
            }
            true
        }
        mode if mode.starts_with("distribute-") => {
            let sel = st.shell.bridge.selection().selected_ids.clone();
            if let Some(surface_id) = st.shell.bridge.active_surface().or_else(|| {
                st.shell
                    .bridge
                    .session()
                    .and_then(|s| s.document().surfaces().first().map(|sf| sf.id))
            }) {
                let axis = if mode == "distribute-v" {
                    petunia_design_document::DistributionAxis::Vertical
                } else {
                    petunia_design_document::DistributionAxis::Horizontal
                };
                let _ = st.shell.bridge.distribute_objects(surface_id, sel, axis);
            }
            true
        }
        mode if mode.starts_with("bool-") => {
            let mut sel = st.shell.bridge.selection().selected_ids.clone();
            if sel.len() < 2 {
                if let Some(session) = st.shell.bridge.session() {
                    if let Some(surface) = session.document().surfaces().first() {
                        if surface.objects().len() >= 2 {
                            sel = vec![surface.objects()[0].id, surface.objects()[1].id];
                        }
                    }
                }
            }
            if sel.len() >= 2 {
                if let Ok(target_id) = st.shell.bridge.next_object_id() {
                    if let Some(surface) = st
                        .shell
                        .bridge
                        .session()
                        .and_then(|s| s.document().surfaces().first().cloned())
                    {
                        let op = match mode {
                            "bool-subtract" => petunia_design_geometry::BooleanOp::Difference,
                            "bool-intersect" => petunia_design_geometry::BooleanOp::Intersection,
                            "bool-xor" => petunia_design_geometry::BooleanOp::Xor,
                            _ => petunia_design_geometry::BooleanOp::Union,
                        };
                        let _ = st.shell.bridge.apply_boolean(
                            surface.id,
                            target_id,
                            sel[0],
                            sel[1],
                            op,
                        );
                        st.shell.bridge.set_selection(vec![target_id]);
                    }
                }
            }
            true
        }
        "convert-curves" => {
            let sel = st.shell.bridge.selection().selected_ids.clone();
            for id in sel {
                let _ = st.shell.bridge.convert_to_curves(id);
            }
            true
        }
        "bake-corners" => {
            let sel = st.shell.bridge.selection().selected_ids.clone();
            for id in sel {
                let _ = st.shell.bridge.bake_corners(id);
            }
            true
        }
        _ => false,
    }
}

fn sync_ui_from_shell(window: &MainWindow, state: &PetuniaSlintState) {
    let tool_str = format!("{:?}", state.shell.active_tool());
    window.set_active_tool_name(tool_str.into());

    // Batch 1: keep the contextual marquee-rule control in sync with the tool.
    let rule_str = match state.shell.tools.select_tool().marquee_rule() {
        MarqueeSelectRule::Intersect => "Intersect",
        MarqueeSelectRule::Contained => "Contained",
        MarqueeSelectRule::Directional => "Directional",
    };
    window.set_marquee_rule(rule_str.into());

    let zoom_pct = (state.shell.camera.zoom * 100.0).round() as i32;
    window.set_zoom_pct(zoom_pct);

    // Sync Layers
    let layers: LayersPresentationModel = state.shell.query_layers();
    let layer_items: Vec<LayerRowItem> = layers
        .rows
        .iter()
        .map(|r| {
            let kind = if let Some(session) = state.shell.bridge.session() {
                session.document()
                    .surfaces()
                    .iter()
                    .flat_map(|s| s.objects())
                    .find(|o| o.id == r.id)
                    .map(|o| match &o.shape {
                        Some(petunia_design_document::ShapeKind::Ellipse) => "Ellipse",
                        Some(petunia_design_document::ShapeKind::Rectangle { .. }) => "Rectangle",
                        Some(petunia_design_document::ShapeKind::Star { .. }) => "Star",
                        Some(petunia_design_document::ShapeKind::Polygon { .. }) => "Polygon",
                        Some(petunia_design_document::ShapeKind::Text { .. }) => "Text",
                        Some(petunia_design_document::ShapeKind::Path(_)) => "Path",
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

    // Sync Canvas Objects: exact colors via the engine parser, rotation
    // baked into path geometry (Slint has no rotate transform), effective
    // opacity, real text metrics and center-sampled gradients.
    let mut canvas_items = Vec::new();
    if let Some(session) = state.shell.bridge.session() {
        let selection = state.shell.bridge.selection();
        for surface in session.document().surfaces() {
            let sb = surface.bounds();
            for obj in surface.objects() {
                if let Some(b) = obj.bounds {
                    let is_sel = selection.contains(obj.id);
                    let eff = obj.effective_appearance();
                    let opacity = eff.opacity.clamp(0.0, 1.0) as f32;
                    let rotated = obj.rotation.abs() > f64::EPSILON;

                    // Fill: primary entry first, legacy token fallback.
                    let bg_color = eff
                        .primary_fill()
                        .and_then(|f| paint_to_slint(&f.paint))
                        .or_else(|| obj.fill.as_deref().map(token_to_slint))
                        .unwrap_or_else(|| token_to_slint("ptnd.blue/500"));
                    let (stroke_color, stroke_width) = eff
                        .primary_stroke()
                        .and_then(|s| match &s.paint {
                            petunia_design_document::Paint::None => None,
                            _ => paint_to_slint(&s.paint).map(|c| (c, s.width.max(0.0) as f32)),
                        })
                        .or_else(|| {
                            obj.stroke
                                .as_deref()
                                .map(|t| (token_to_slint(t), (obj.stroke_width as f32).max(0.0)))
                        })
                        .unwrap_or((slint::Color::from_argb_u8(0, 0, 0, 0), 0.0));

                    // Geometry: rotated objects bake rotation about the
                    // bounds top-left (document model) into path data and
                    // report the rotated bounding box.
                    let mut is_circle = matches!(&obj.shape, Some(petunia_design_document::ShapeKind::Ellipse));
                    let mut is_path = false;
                    let mut is_text = false;
                    let mut text_content = String::new();
                    let mut text_size = 16.0f32;
                    let mut text_color = token_to_slint("ptnd.gray/900");
                    let mut svg_path = String::new();
                    let mut corner_radius = 0.0f32;
                    let (draw_x, draw_y, draw_w, draw_h);
                    if rotated {
                        if matches!(&obj.shape, Some(petunia_design_document::ShapeKind::Text { .. })) {
                            // Text has no vector outline: draw unrotated
                            // (known fidelity gap, documented).
                            is_text = true;
                            draw_x = b[0];
                            draw_y = b[1];
                            draw_w = b[2];
                            draw_h = b[3];
                        } else {
                            let rot = GAffine::translate(b[0], b[1])
                                .after(GAffine::rotate(obj.rotation))
                                .after(GAffine::translate(-b[0], -b[1]));
                            let rp = obj.evaluated_path().transformed(rot);
                            if let Some(bb) = rp.bounding_box() {
                                draw_x = bb.x0;
                                draw_y = bb.y0;
                                draw_w = bb.width().max(1.0);
                                draw_h = bb.height().max(1.0);
                                let local = rp.transformed(GAffine::translate(-bb.x0, -bb.y0));
                                svg_path = local.to_svg_path_data();
                                is_path = true;
                                is_circle = false;
                            } else {
                                draw_x = b[0];
                                draw_y = b[1];
                                draw_w = b[2];
                                draw_h = b[3];
                            }
                        }
                    } else {
                        draw_x = b[0];
                        draw_y = b[1];
                        draw_w = b[2];
                        draw_h = b[3];
                        match &obj.shape {
                            Some(petunia_design_document::ShapeKind::Ellipse) => {
                                is_circle = true;
                            }
                            Some(petunia_design_document::ShapeKind::Rectangle { corner_radii }) => {
                                corner_radius = corner_radii[0] as f32;
                            }
                            Some(petunia_design_document::ShapeKind::Path(_)) => {
                                let local = obj
                                    .evaluated_path()
                                    .transformed(GAffine::translate(-b[0], -b[1]));
                                svg_path = local.to_svg_path_data();
                                is_path = true;
                                is_circle = false;
                            }
                            Some(petunia_design_document::ShapeKind::Polygon { .. })
                            | Some(petunia_design_document::ShapeKind::Star { .. }) => {
                                let local_path = obj
                                    .evaluated_path()
                                    .transformed(GAffine::translate(-b[0], -b[1]));
                                svg_path = local_path.to_svg_path_data();
                                is_path = true;
                                is_circle = false;
                            }
                            Some(petunia_design_document::ShapeKind::Text {
                                content,
                                font_size,
                                ..
                            }) => {
                                is_text = true;
                                text_content = content.clone();
                                text_size = *font_size as f32;
                            }
                            None => {
                                let name_lower = obj.name.to_lowercase();
                                is_circle = name_lower.contains("circle")
                                    || name_lower.contains("ellipse");
                            }
                        }
                    }
                    if is_text {
                        if let Some(fill) = &obj.fill {
                            text_color = token_to_slint(fill);
                        }
                        if let Some(petunia_design_document::ShapeKind::Text { font_size, .. }) =
                            &obj.shape
                        {
                            text_size = *font_size as f32;
                        }
                    }

                    // Relative to the artboard's top-left corner
                    let rel_x = (draw_x - sb[0]).max(0.0) as f32;
                    let rel_y = (draw_y - sb[1]).max(0.0) as f32;
                    let w = draw_w.max(10.0) as f32;
                    let h = draw_h.max(10.0) as f32;

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
                        text_size,
                        text_color,
                        svg_path: svg_path.into(),
                        opacity,
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
            window.set_prop_rot(props.rotation.to_degrees() as f32);
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
                        petunia_design_document::Paint::Solid(c) => ("Solid", c.clone()),
                        petunia_design_document::Paint::LinearGradient(_) => {
                            ("Linear", "Linear Gradient".to_string())
                        }
                        petunia_design_document::Paint::RadialGradient(_) => {
                            ("Radial", "Radial Gradient".to_string())
                        }
                        petunia_design_document::Paint::None => ("None", "None".to_string()),
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
                        petunia_design_document::Paint::Solid(c) => ("Solid", c.clone()),
                        petunia_design_document::Paint::LinearGradient(_) => {
                            ("Linear", "Linear Gradient".to_string())
                        }
                        petunia_design_document::Paint::RadialGradient(_) => {
                            ("Radial", "Radial Gradient".to_string())
                        }
                        petunia_design_document::Paint::None => ("None", "None".to_string()),
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
        if let Some(app) = &props.appearance {
            if let Some(stroke) = app.primary_stroke() {
                window.set_prop_stroke_width(stroke.width as f32);
            }
        }
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
        window.set_prop_rot(0.0);
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
        println!("petunia-design: Running automated smoke test...");
        let mut state = PetuniaSlintState::new();
        if let Err(e) = state.smoke_test() {
            eprintln!("petunia-design smoke test failed: {e}");
            std::process::exit(1);
        }
        println!("petunia-design: Smoke test PASSED.");
        return Ok(());
    }

    let main_window = MainWindow::new()?;
    let state = Rc::new(RefCell::new(PetuniaSlintState::new()));

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
                win.set_canvas_cursor_kind(tool_cursor_kind(tool).into());
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
                    _ => "Petunia Design Studio: ferramenta pronta para uso.",
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

    // Command palette (Ctrl+K) (08.2)
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_open_palette(move || {
            let mut st = state_clone.borrow_mut();
            st.palette_filtered.clear();
            if let Some(win) = win_weak.upgrade() {
                win.set_palette_query("".into());
                push_palette_items(&win, "", &mut st);
                win.set_palette_open(true);
            }
        });
    }
    {
        let win_weak = main_window.as_weak();
        main_window.on_close_palette(move || {
            if let Some(win) = win_weak.upgrade() {
                win.set_palette_open(false);
            }
        });
    }
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_palette_query_changed(move |text| {
            let mut st = state_clone.borrow_mut();
            st.palette_filtered.clear();
            if let Some(win) = win_weak.upgrade() {
                push_palette_items(&win, text.as_str(), &mut st);
            }
        });
    }
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        let activate = move |id: String| {
            let mut st = state_clone.borrow_mut();
            let should_sync = run_palette_command(&mut st, id.as_str());
            if id == "export" {
                if let Some(win) = win_weak.upgrade() {
                    win.set_palette_open(false);
                    win.set_export_dialog_open(true);
                }
                return;
            }
            if let Some(win) = win_weak.upgrade() {
                win.set_palette_open(false);
                if should_sync {
                    sync_ui_from_shell(&win, &st);
                }
            }
        };
        let state_clone2 = state.clone();
        let win_weak2 = main_window.as_weak();
        main_window.on_palette_activate(move |id| {
            let _ = &state_clone2;
            let _ = &win_weak2;
            activate(id.to_string());
        });
        // Enter-to-run-first needs its own closure over fresh clones.
        let state_clone3 = state.clone();
        let win_weak3 = main_window.as_weak();
        main_window.on_palette_activate_first(move || {
            let first = state_clone3
                .borrow()
                .palette_filtered
                .first()
                .cloned();
            if let Some(id) = first {
                let mut st = state_clone3.borrow_mut();
                let should_sync = run_palette_command(&mut st, id.as_str());
                if let Some(win) = win_weak3.upgrade() {
                    win.set_palette_open(false);
                    if should_sync {
                        sync_ui_from_shell(&win, &st);
                    }
                }
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
                .and_then(|s| s.document().surfaces().first())
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
                .add_filter("Petunia Design Studio Project (*.PTND)", &["PTND", "ptnd"])
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
                .map(|s| format!("{}.aub", s.title()))
                .unwrap_or_else(|| "projeto.aub".to_string());
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Petunia Design Studio Project (*.PTND)", &["PTND", "ptnd"])
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
                let Ok(obj_id) = st.shell.bridge.next_object_id() else { return };
                if let Some(surface) = st
                    .shell
                    .bridge
                    .session()
                    .and_then(|s| s.document().surfaces().first().cloned())
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
                        .set_fill(obj_id, Some("ptnd.green/500".to_string()));
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

    // Batch 1: Select marquee rule backing the settings option
    // (Sobrepor = Intersect, Completa = Contained, Auto = Directional).
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_marquee_rule_changed(move |rule_name| {
            let rule = match rule_name.as_str() {
                "Intersect" => MarqueeSelectRule::Intersect,
                "Contained" => MarqueeSelectRule::Contained,
                _ => MarqueeSelectRule::Directional,
            };
            let mut st = state_clone.borrow_mut();
            st.shell.tools.set_select_marquee_rule(rule);
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
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
            if let Some(surface) = st
                .shell
                .bridge
                .session()
                .and_then(|s| s.document().surfaces().first().cloned())
            {
                let Ok(new_id) = st.shell.bridge.next_object_id() else { return };
                let count = surface.objects().len() + 1;
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
                    .set_fill(new_id, Some("ptnd.green/500".to_string()));
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
            if let Some(surface) = st
                .shell
                .bridge
                .session()
                .and_then(|s| s.document().surfaces().first().cloned())
            {
                let Ok(new_id) = st.shell.bridge.next_object_id() else { return };
                let count = surface.objects().len() + 1;
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
                    .set_fill(new_id, Some("ptnd.purple/500".to_string()));
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
            if let Some(surface) = st
                .shell
                .bridge
                .session()
                .and_then(|s| s.document().surfaces().first().cloned())
            {
                let Ok(new_id) = st.shell.bridge.next_object_id() else { return };
                let count = surface.objects().len() + 1;
                let offset = (count as f64 * 35.0) % 250.0;
                let _ = st.shell.bridge.create_shape_object(
                    surface.id,
                    new_id,
                    format!("Star {}", count),
                    petunia_design_document::ShapeKind::Star {
                        points: 5,
                        inner_ratio: 0.45,
                    },
                    Some([220.0 + offset, 160.0 + offset, 130.0, 130.0]),
                    Some("ptnd.rose/500".to_string()),
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
            if let Some(surface) = st
                .shell
                .bridge
                .session()
                .and_then(|s| s.document().surfaces().first().cloned())
            {
                let Ok(new_id) = st.shell.bridge.next_object_id() else { return };
                let count = surface.objects().len() + 1;
                let offset = (count as f64 * 25.0) % 200.0;
                let _ = st.shell.bridge.create_shape_object(
                    surface.id,
                    new_id,
                    format!("Text {}", count),
                    petunia_design_document::ShapeKind::Text {
                        content: format!("Texto Vetorial {}", count),
                        font_family: "Inter".to_string(),
                        font_size: 18.0,
                        line_height: 22.0,
                        letter_spacing: 0.0,
                    },
                    Some([140.0 + offset, 240.0 + offset, 220.0, 40.0]),
                    Some("ptnd.purple/500".to_string()),
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
                    if let Some(surface) = session.document().surfaces().first() {
                        if surface.objects().len() >= 2 {
                            sel_ids = vec![surface.objects()[0].id, surface.objects()[1].id];
                        }
                    }
                }
            }
            if sel_ids.len() >= 2 {
                let id_a = sel_ids[0];
                let id_b = sel_ids[1];
                let Ok(target_id) = st.shell.bridge.next_object_id() else { return };
                if let Some(surface) = st
                    .shell
                    .bridge
                    .session()
                    .and_then(|s| s.document().surfaces().first().cloned())
                {
                    let _ = st.shell.bridge.apply_boolean(
                        surface.id,
                        target_id,
                        id_a,
                        id_b,
                        petunia_design_geometry::BooleanOp::Union,
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
                    if let Some(surface) = session.document().surfaces().first() {
                        if surface.objects().len() >= 2 {
                            sel_ids = vec![surface.objects()[0].id, surface.objects()[1].id];
                        }
                    }
                }
            }
            if sel_ids.len() >= 2 {
                let id_a = sel_ids[0];
                let id_b = sel_ids[1];
                let Ok(target_id) = st.shell.bridge.next_object_id() else { return };
                if let Some(surface) = st
                    .shell
                    .bridge
                    .session()
                    .and_then(|s| s.document().surfaces().first().cloned())
                {
                    let _ = st.shell.bridge.apply_boolean(
                        surface.id,
                        target_id,
                        id_a,
                        id_b,
                        petunia_design_geometry::BooleanOp::Difference,
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
                    if let Some(surface) = session.document().surfaces().first() {
                        if surface.objects().len() >= 2 {
                            sel_ids = vec![surface.objects()[0].id, surface.objects()[1].id];
                        }
                    }
                }
            }
            if sel_ids.len() >= 2 {
                let id_a = sel_ids[0];
                let id_b = sel_ids[1];
                let Ok(target_id) = st.shell.bridge.next_object_id() else { return };
                if let Some(surface) = st
                    .shell
                    .bridge
                    .session()
                    .and_then(|s| s.document().surfaces().first().cloned())
                {
                    let _ = st.shell.bridge.apply_boolean(
                        surface.id,
                        target_id,
                        id_a,
                        id_b,
                        petunia_design_geometry::BooleanOp::Intersection,
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
                    if let Some(surface) = session.document().surfaces().first() {
                        if surface.objects().len() >= 2 {
                            sel_ids = vec![surface.objects()[0].id, surface.objects()[1].id];
                        }
                    }
                }
            }
            if sel_ids.len() >= 2 {
                let id_a = sel_ids[0];
                let id_b = sel_ids[1];
                let Ok(target_id) = st.shell.bridge.next_object_id() else { return };
                if let Some(surface) = st
                    .shell
                    .bridge
                    .session()
                    .and_then(|s| s.document().surfaces().first().cloned())
                {
                    let _ = st.shell.bridge.apply_boolean(
                        surface.id,
                        target_id,
                        id_a,
                        id_b,
                        petunia_design_geometry::BooleanOp::Xor,
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

    // Bake contour (explicit commit of live offsets, 09.31)
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_bake_contour_clicked(move || {
            let mut st = state_clone.borrow_mut();
            let sel = st.shell.bridge.selection().selected_ids.clone();
            for id in sel {
                let _ = st.shell.bridge.bake_contour(id);
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
                        .and_then(|s| s.document().surfaces().first().map(|sf| sf.id))
                }) {
                    let mode = match mode_str.as_str() {
                        "Left" => petunia_design_document::AlignmentMode::Left,
                        "Right" => petunia_design_document::AlignmentMode::Right,
                        "Top" => petunia_design_document::AlignmentMode::Top,
                        "Bottom" => petunia_design_document::AlignmentMode::Bottom,
                        "Middle" => petunia_design_document::AlignmentMode::Middle,
                        _ => petunia_design_document::AlignmentMode::Center,
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
                        .and_then(|s| s.document().surfaces().first().map(|sf| sf.id))
                }) {
                    let axis = match axis_str.as_str() {
                        "Vertical" => petunia_design_document::DistributionAxis::Vertical,
                        _ => petunia_design_document::DistributionAxis::Horizontal,
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
                "ptnd.purple/500"
            } else if c.red() > 200 && c.green() > 150 {
                "ptnd.yellow/500"
            } else if c.red() > 200 {
                "ptnd.rose/500"
            } else if c.green() > 150 {
                "ptnd.green/500"
            } else {
                "ptnd.blue/500"
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
        main_window.on_commit_prop_x(move |text| {
            let mut st = state_clone.borrow_mut();
            if let Ok(v) = text.parse::<f64>() {
                if let Some(sel_id) = st.shell.bridge.selection().selected_ids.first().copied() {
                    let current = st.shell.bridge.session().and_then(|s| {
                        s.document()
                            .surfaces()
                            .iter()
                            .flat_map(|s| s.objects())
                            .find(|o| o.id == sel_id)
                            .and_then(|o| o.bounds.map(|b| (b, o.rotation)))
                    });
                    if let Some((b, rot)) = current {
                        let mut nb = b;
                        nb[0] = v;
                        let _ = st.shell.bridge.set_bounds(sel_id, Some(nb), rot);
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
        main_window.on_commit_prop_y(move |text| {
            let mut st = state_clone.borrow_mut();
            if let Ok(v) = text.parse::<f64>() {
                if let Some(sel_id) = st.shell.bridge.selection().selected_ids.first().copied() {
                    let current = st.shell.bridge.session().and_then(|s| {
                        s.document()
                            .surfaces()
                            .iter()
                            .flat_map(|s| s.objects())
                            .find(|o| o.id == sel_id)
                            .and_then(|o| o.bounds.map(|b| (b, o.rotation)))
                    });
                    if let Some((b, rot)) = current {
                        let mut nb = b;
                        nb[1] = v;
                        let _ = st.shell.bridge.set_bounds(sel_id, Some(nb), rot);
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
        main_window.on_commit_prop_w(move |text| {
            let mut st = state_clone.borrow_mut();
            if let Ok(v) = text.parse::<f64>() {
                if let Some(sel_id) = st.shell.bridge.selection().selected_ids.first().copied() {
                    let current = st.shell.bridge.session().and_then(|s| {
                        s.document()
                            .surfaces()
                            .iter()
                            .flat_map(|s| s.objects())
                            .find(|o| o.id == sel_id)
                            .and_then(|o| o.bounds.map(|b| (b, o.rotation)))
                    });
                    if let Some((b, rot)) = current {
                        let mut nb = b;
                        nb[2] = v.max(1.0);
                        let _ = st.shell.bridge.set_bounds(sel_id, Some(nb), rot);
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
        main_window.on_commit_prop_h(move |text| {
            let mut st = state_clone.borrow_mut();
            if let Ok(v) = text.parse::<f64>() {
                if let Some(sel_id) = st.shell.bridge.selection().selected_ids.first().copied() {
                    let current = st.shell.bridge.session().and_then(|s| {
                        s.document()
                            .surfaces()
                            .iter()
                            .flat_map(|s| s.objects())
                            .find(|o| o.id == sel_id)
                            .and_then(|o| o.bounds.map(|b| (b, o.rotation)))
                    });
                    if let Some((b, rot)) = current {
                        let mut nb = b;
                        nb[3] = v.max(1.0);
                        let _ = st.shell.bridge.set_bounds(sel_id, Some(nb), rot);
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
        main_window.on_commit_prop_rot(move |text| {
            let mut st = state_clone.borrow_mut();
            if let Ok(deg) = text.parse::<f64>() {
                if let Some(sel_id) = st.shell.bridge.selection().selected_ids.first().copied() {
                    let current = st.shell.bridge.session().and_then(|s| {
                        s.document()
                            .surfaces()
                            .iter()
                            .flat_map(|s| s.objects())
                            .find(|o| o.id == sel_id)
                            .and_then(|o| o.bounds)
                    });
                    if let Some(b) = current {
                        let _ = st.shell.bridge.set_bounds(
                            sel_id,
                            Some(b),
                            deg.to_radians(),
                        );
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
        main_window.on_set_stroke_width_value(move |val| {
            let mut st = state_clone.borrow_mut();
            if let Some(sel_id) = st.shell.bridge.selection().selected_ids.first().copied() {
                let target = st.shell.bridge.session().and_then(|s| {
                    s.document()
                        .surfaces()
                        .iter()
                        .flat_map(|s| s.objects())
                        .find(|o| o.id == sel_id)
                        .map(|o| {
                            o.effective_appearance()
                                .primary_stroke()
                                .map(|entry| entry.id)
                        })
                });
                if let Some(Some(stroke_id)) = target {
                    let _ = st.shell.bridge.submit_command(CommandRequest::new(
                        Command::SetStrokeItemWidth {
                            id: sel_id,
                            stroke_id,
                            width: (val as f64).max(0.0),
                        },
                    ));
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
                "Multiply" => petunia_design_document::BlendMode::Multiply,
                "Screen" => petunia_design_document::BlendMode::Screen,
                "Overlay" => petunia_design_document::BlendMode::Overlay,
                "Darken" => petunia_design_document::BlendMode::Darken,
                "Lighten" => petunia_design_document::BlendMode::Lighten,
                "Difference" => petunia_design_document::BlendMode::Difference,
                "Exclusion" => petunia_design_document::BlendMode::Exclusion,
                "ColorDodge" => petunia_design_document::BlendMode::ColorDodge,
                "ColorBurn" => petunia_design_document::BlendMode::ColorBurn,
                "HardLight" => petunia_design_document::BlendMode::HardLight,
                "SoftLight" => petunia_design_document::BlendMode::SoftLight,
                "Hue" => petunia_design_document::BlendMode::Hue,
                "Saturation" => petunia_design_document::BlendMode::Saturation,
                "Color" => petunia_design_document::BlendMode::Color,
                "Luminosity" => petunia_design_document::BlendMode::Luminosity,
                _ => petunia_design_document::BlendMode::Normal,
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
                    petunia_design_document::Paint::Solid("ptnd.rose/500".to_string()),
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
                    petunia_design_document::Paint::Solid("ptnd.blue/500".to_string()),
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
                            "ptnd.blue/500",
                            "ptnd.purple/500",
                        );
                    }
                    "radial" => {
                        let _ = st.shell.bridge.set_radial_gradient_fill(
                            id,
                            "ptnd.yellow/500",
                            "ptnd.rose/500",
                        );
                    }
                    _ => {
                        let _ = st
                            .shell
                            .bridge
                            .set_fill(id, Some("ptnd.blue/500".to_string()));
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
                        if let Some(surface) = session.document().surfaces().first() {
                            if let Some(obj) = surface.objects().iter().find(|o| o.id == sel_id) {
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
                let Ok(clone_id) = st.shell.bridge.next_object_id() else { return };
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
                    .and_then(|s| s.document().surfaces().first().map(|surf| surf.id))
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
                        s.document()
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
                        .and_then(|s| s.document().find_object(id).and_then(|o| o.parent));
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
                    .and_then(|s| s.document().surfaces().first().map(|surf| surf.id))
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
                    let obj = s.document().find_object(id)?;
                    if obj.role == Some(ContainerRole::ClipGroup) {
                        Some(obj.id)
                    } else if let Some(pid) = obj.parent {
                        let parent = s.document().find_object(pid)?;
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
                    s.document().find_object(id).map(|o| (o.bounds, o.rotation))
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
                    .and_then(|sid| session.document().surfaces().iter().find(|s| s.id == sid))
                    .or_else(|| session.document().surfaces().first());
                surf.map(|s| s.objects().iter().map(|o| o.id).collect())
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
                    .and_then(|sid| session.document().surfaces().iter().find(|s| s.id == sid))
                    .or_else(|| session.document().surfaces().first());
                if let Some(surface) = surf {
                    let objects: Vec<ObjectId> = surface.objects().iter().map(|o| o.id).collect();
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
                        let svg_content = export_document_svg(session.document());
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
                        match export_document_pdf(session.document(), &options) {
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
                        if let Some(surface) = session.document().surfaces().first() {
                            let w = (surface.dimensions[0].round() as usize).max(10);
                            let h = (surface.dimensions[1].round() as usize).max(10);
                            let mut buffer = vec![255u8; w * h * 4];
                            for obj in surface.objects() {
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
                                        Some(petunia_design_document::ShapeKind::Ellipse)
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

            // Handle affordances win only under selection tools; any
            // other active tool owns the cursor everywhere on canvas.
            let cursor_kind = match st.shell.active_tool() {
                ToolKind::Select | ToolKind::Node
                    if st.shell.bridge.selection().combined_bounds.is_some() =>
                {
                    let [bx, by, bw, bh] = st
                        .shell
                        .bridge
                        .selection()
                        .combined_bounds
                        .unwrap_or([0.0, 0.0, 0.0, 0.0]);
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
                        tool_cursor_kind(st.shell.active_tool())
                    }
                }
                tool => tool_cursor_kind(tool),
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
        let mut state = PetuniaSlintState::new();
        assert!(state.smoke_test().is_ok());
    }
}
