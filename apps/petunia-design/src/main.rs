//! Petunia Design Studio — Slint primary GUI.
//!
//! Evaluates Slint declarative UI toolkit, consuming PetuniaDesignGuiBridge
//! presentation models and dispatching interactive commands.

slint::include_modules!();

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase, SemanticModifiers,
};
use petunia_design_application::menus;
use petunia_design_application::tools::ToolKind;
use petunia_design_application::{ActionId, ActionRequest, Command, CommandRequest};
use petunia_design_document::{Bleed, ContainerRole, Guide, GuideOrientation, Margins};
use petunia_design_foundation::{ObjectId, PetuniaError};
use petunia_design_geometry::{GAffine, GPoint, GRect};
use petunia_design_io::{
    export_document_pdf, export_document_svg, export_raster, PdfExportOptions, RasterExportOptions,
    RasterFormat, RawRasterImage,
};
use petunia_design_shell::bridge::{
    DataMergePresentationModel, HistoryPresentationModel, LayersPresentationModel,
    PropertiesPresentationModel,
};
use petunia_design_shell::canvas::overlay::{hit_test_handle_or_border, SelectionHandleKind};
use petunia_design_shell::context_toolbar::{self, ToolbarEntryKind};
use petunia_design_shell::menu::{MenuItemPresentation, MenuNodePresentation, ShellControlKind};
use petunia_design_shell::shell::PetuniaShell;
use slint::{ComponentHandle, VecModel};

pub struct PetuniaSlintState {
    pub shell: PetuniaShell,
    pub drag_start_doc: Option<GPoint>,
    pub dragging_object_id: Option<ObjectId>,
    pub drag_initial_bounds: Option<[f64; 4]>,
    /// Action tokens currently listed in the palette, in menu order, for
    /// Enter-to-run-first. Tokens, not action ids: two items may share an
    /// action and differ only by payload.
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
        // The product speaks pt-BR today; the canonical source catalog stays
        // en-US and every label resolves through the same service (12.6).
        shell
            .bridge
            .set_locale(petunia_design_resources::i18n::Locale::PtBr);
        let _ = populate_showcase_document(&mut shell);
        // The camera belongs to the session, so it is framed after the
        // document exists (15.B).
        let mut camera = shell.view_camera();
        camera.viewport_width = 950.0;
        camera.viewport_height = 700.0;
        camera.pan_x = 80.0;
        camera.pan_y = 80.0;
        camera.zoom = 1.0;
        shell.set_view_camera(camera);
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

        // The context toolbar the UI paints is scoped to the active tool, and it
        // is never empty and never nameless: the badge is the first entry and it
        // carries the tool's catalog name (08.23).
        for tool in [
            ToolKind::Select,
            ToolKind::Pen,
            ToolKind::Rectangle,
            ToolKind::Crop,
        ] {
            self.shell.set_active_tool(tool);
            let entries = self.shell.bridge.query_context_toolbar(tool, false);
            let Some(first) = entries.first() else {
                return Err(format!("context toolbar is empty for {tool:?}"));
            };
            if first.kind != ToolbarEntryKind::ToolBadge {
                return Err(format!(
                    "context toolbar does not open on the badge for {tool:?}"
                ));
            }
            if first.label.is_empty() {
                return Err(format!("context toolbar badge is nameless for {tool:?}"));
            }
        }

        // 1. Validate Layers Hierarchy Presentation Model
        let layers = self.shell.query_layers();
        if layers.rows.is_empty() {
            return Err("Layers model has no rows".to_string());
        }

        // 2. Validate Grouping & Hierarchy (Step 2)
        let surface_id = snap.active_surface.ok_or("No active surface in snapshot")?;
        let rect_id = layers.rows[0].id;
        let circle_id = layers.rows[1].id;
        let group_id = self
            .shell
            .bridge
            .next_object_id()
            .map_err(|e| e.to_string())?;

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
        let mask_id = self
            .shell
            .bridge
            .next_object_id()
            .map_err(|e| e.to_string())?;
        let clip_group_id = self
            .shell
            .bridge
            .next_object_id()
            .map_err(|e| e.to_string())?;
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

        // 5. The menu/palette Action lane (state-level, no window needed).
        // One token lane serves the menu bar, the command palette, shortcuts
        // and this test, so a broken registry shows up here first.
        if run_action_token(self, "ptnd.action.does.not.exist#null").is_some() {
            return Err("an unknown action token resolved".to_string());
        }
        let zoom_before = self.shell.view_camera().zoom;
        let ran = run_action_token(self, "ptnd.action.view.zoom_in#null");
        if ran.as_deref() != Some("ptnd.action.view.zoom_in") {
            return Err("the action lane did not resolve view.zoom_in".to_string());
        }
        if self.shell.view_camera().zoom <= zoom_before {
            return Err("view.zoom_in did not change the zoom".to_string());
        }

        // The menu and the palette must expose the wired actions and must not
        // offer a blocked capability as runnable (15.F §2).
        let menu = self.shell.bridge.query_menu_bar();
        if menu.item_count() == 0 || menu.enabled_count() == 0 {
            return Err("the menu bar resolved no runnable item".to_string());
        }
        let offered = self.shell.bridge.query_command_index();
        if offered.is_empty() {
            return Err("the command index is empty".to_string());
        }
        if offered
            .iter()
            .any(|item| item.action_id == "ptnd.action.file.place")
        {
            return Err("a blocked action was offered as runnable".to_string());
        }

        // Export goes through the same Action lane, not through a private
        // renderer in the app: the artifact must be a real PNG.
        let export_dir = std::env::temp_dir().join("petunia-design-smoke-export");
        let _ = std::fs::create_dir_all(&export_dir);
        let export_path = export_dir.join("smoke.png");
        let export_payload = serde_json::json!({
            "path": export_path.to_string_lossy(),
            "format": "png"
        });
        self.shell
            .bridge
            .dispatch_action(ActionRequest::new(
                ActionId::new("ptnd.action.file.export"),
                export_payload,
            ))
            .map_err(|e| format!("export action failed: {e}"))?;
        let exported =
            std::fs::read(&export_path).map_err(|e| format!("exported artifact missing: {e}"))?;
        if !exported.starts_with(b"\x89PNG") {
            return Err("the export action produced a non-PNG artifact".to_string());
        }
        let _ = std::fs::remove_dir_all(&export_dir);
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
        petunia_design_document::Paint::LinearGradient(g) => {
            g.sample_rgba(0.5).map(|(rgb, _)| sample_to_slint(rgb))
        }
        petunia_design_document::Paint::RadialGradient(g) => {
            g.sample_rgba(0.5).map(|(rgb, _)| sample_to_slint(rgb))
        }
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

/// Builds the registry-driven menu bar and pushes it to the UI (15.G).
///
/// The UI contributes nothing to this structure: labels, order, shortcuts and
/// availability all come from the surface registry, so a wired action cannot
/// be missing from the bar and a blocked one cannot look functional.
fn push_menu(win: &MainWindow, st: &PetuniaSlintState) {
    let families: Vec<MenuFamilyEntry> = st
        .shell
        .bridge
        .query_menu_bar()
        .families
        .into_iter()
        .map(|family| {
            // Groups are collected first because a row refers to its submenu by
            // index: the DTO stays flat (arrays of structs) and the popup needs
            // no recursive type to render a branch.
            let groups: Vec<MenuGroupEntry> = family
                .nodes
                .iter()
                .filter_map(|node| match node {
                    MenuNodePresentation::Group(group) => Some(MenuGroupEntry {
                        label: group.label.clone().into(),
                        items: Rc::new(VecModel::from(
                            group.items.iter().map(menu_item_entry).collect::<Vec<_>>(),
                        ))
                        .into(),
                    }),
                    MenuNodePresentation::Item(_) => None,
                })
                .collect();

            let mut group_cursor: i32 = 0;
            let rows: Vec<MenuRowEntry> = family
                .nodes
                .iter()
                .map(|node| match node {
                    MenuNodePresentation::Item(item) => MenuRowEntry {
                        is_group: false,
                        label: item.label.clone().into(),
                        token: item.action_token.clone().into(),
                        shortcut: item.shortcut.clone().into(),
                        enabled: item.enabled,
                        disabled_reason: item.disabled_reason.clone().into(),
                        group_index: -1,
                    },
                    MenuNodePresentation::Group(group) => {
                        let index = group_cursor;
                        group_cursor += 1;
                        // A branch is offered when anything in it can run; an
                        // entirely blocked submenu must not look available.
                        let enabled = group.items.iter().any(|item| item.enabled);
                        MenuRowEntry {
                            is_group: true,
                            label: group.label.clone().into(),
                            token: "".into(),
                            shortcut: "".into(),
                            enabled,
                            disabled_reason: "".into(),
                            group_index: index,
                        }
                    }
                })
                .collect();

            MenuFamilyEntry {
                label: family.label.into(),
                rows: Rc::new(VecModel::from(rows)).into(),
                groups: Rc::new(VecModel::from(groups)).into(),
            }
        })
        .collect();
    win.set_menu_families(Rc::new(VecModel::from(families)).into());
}

/// Fills the popup behind the zoom box (08.2).
///
/// The rows are the registry's zoom levels, presented exactly like menu rows:
/// the box shows the same labels, the same shortcuts and the same availability
/// the View submenu shows, because both read `menus::zoom_levels`.
fn push_zoom_levels(win: &MainWindow, st: &PetuniaSlintState) {
    let rows: Vec<MenuRowEntry> = st
        .shell
        .bridge
        .query_zoom_levels()
        .iter()
        .map(|item| MenuRowEntry {
            is_group: false,
            label: item.label.clone().into(),
            token: item.action_token.clone().into(),
            shortcut: item.shortcut.clone().into(),
            enabled: item.enabled,
            disabled_reason: item.disabled_reason.clone().into(),
            group_index: -1,
        })
        .collect();
    win.set_zoom_rows(Rc::new(VecModel::from(rows)).into());
}

/// Converts one resolved menu item into the Slint row payload.
fn menu_item_entry(item: &MenuItemPresentation) -> MenuItemEntry {
    MenuItemEntry {
        token: item.action_token.clone().into(),
        label: item.label.clone().into(),
        shortcut: item.shortcut.clone().into(),
        enabled: item.enabled,
        disabled_reason: item.disabled_reason.clone().into(),
    }
}

/// Pushes the centred shell control cluster (08.2).
///
/// Order, rendering kind, tooltip and availability all arrive from the shell,
/// which reads them from the registry: the menu bar row paints the list it is
/// handed and declares no control of its own. The zoom readout is the one entry
/// whose value the UI supplies, because it is the live camera state.
fn push_shell_controls(win: &MainWindow, st: &PetuniaSlintState) {
    let controls: Vec<ShellControlEntry> = st
        .shell
        .bridge
        .query_shell_controls()
        .into_iter()
        .map(|control| ShellControlEntry {
            id: control.id.into(),
            kind: match control.kind {
                ShellControlKind::Icon => "icon".into(),
                ShellControlKind::Readout => "readout".into(),
                ShellControlKind::Divider => "divider".into(),
            },
            tooltip: control.tooltip.into(),
            enabled: control.enabled,
        })
        .collect();
    win.set_shell_controls(Rc::new(VecModel::from(controls)).into());

    // The one control that stays in the tab strip, labelled from the catalog.
    win.set_snap_label(
        st.shell
            .bridge
            .surface_label("ptnd.surface.tabs.snapping")
            .unwrap_or_default()
            .into(),
    );
}

/// Pushes the canonical context toolbar for the active tool (08.23).
///
/// Contextuality is decided in the shell: the entries that do not apply to the
/// active tool never reach the UI, so the bar cannot show a vector control while
/// a raster tool is active. Labels, tooltips and blocked reasons arrive
/// localized, from the same catalog and the same registry the menu uses.
fn push_context_toolbar(win: &MainWindow, st: &PetuniaSlintState) {
    let has_selection = st.shell.bridge.action_context().selection_count > 0;
    let entries: Vec<ContextToolbarEntry> = st
        .shell
        .bridge
        .query_context_toolbar(st.shell.active_tool(), has_selection)
        .into_iter()
        .map(|entry| {
            let kind = match entry.kind {
                ToolbarEntryKind::ToolBadge => "tool_badge",
                ToolbarEntryKind::TransformReadout => "transform_readout",
                ToolbarEntryKind::ColorSwatches => "color_swatches",
                ToolbarEntryKind::Command => "command",
                ToolbarEntryKind::Divider => "divider",
                ToolbarEntryKind::Spacer => "spacer",
            };
            ContextToolbarEntry {
                id: entry.id.into(),
                kind: kind.into(),
                label: entry.label.into(),
                tooltip: entry.tooltip.into(),
                tooltip_alt: entry.tooltip_alt.into(),
                enabled: entry.enabled,
                disabled_reason: entry.disabled_reason.into(),
            }
        })
        .collect();
    win.set_context_toolbar(Rc::new(VecModel::from(entries)).into());
    push_toolbar_catalog(win, st);
}

/// Pushes the customization list and the catalog labels the dialog paints.
///
/// The dialog does not invent a string: every button title is a `ptnd.text.*`
/// resolved here, and every row is a slot the shell already owns.
fn push_toolbar_catalog(win: &MainWindow, st: &PetuniaSlintState) {
    let rows: Vec<ToolbarCatalogEntry> = st
        .shell
        .bridge
        .query_toolbar_catalog()
        .into_iter()
        .map(|row| {
            let kind = match row.kind {
                ToolbarEntryKind::ToolBadge => "tool_badge",
                ToolbarEntryKind::TransformReadout => "transform_readout",
                ToolbarEntryKind::ColorSwatches => "color_swatches",
                ToolbarEntryKind::Command => "command",
                ToolbarEntryKind::Divider => "divider",
                ToolbarEntryKind::Spacer => "spacer",
            };
            ToolbarCatalogEntry {
                id: row.id.into(),
                label: row.label.into(),
                kind: kind.into(),
                visible: row.visible,
                can_hide: row.can_hide,
            }
        })
        .collect();
    win.set_toolbar_catalog(Rc::new(VecModel::from(rows)).into());

    let text = |id: &str| {
        st.shell
            .bridge
            .localization()
            .text(id, st.shell.bridge.locale())
            .into()
    };
    win.set_label_overflow(text("ptnd.text.shell.overflow"));
    win.set_label_customize(text("ptnd.text.shell.customize"));
    win.set_label_move_up(text("ptnd.text.shell.move_up"));
    win.set_label_move_down(text("ptnd.text.shell.move_down"));
    win.set_label_reset(text("ptnd.text.shell.reset_toolbar"));
    win.set_label_divider(text("ptnd.text.shell.divider"));
    win.set_label_remove(text("ptnd.text.shell.remove"));
}

/// Pushes the registered personas.
///
/// The switcher renders whatever the registry declares, with catalog labels, so
/// the shell holds no persona name and no persona count of its own.
fn push_personas(win: &MainWindow, st: &PetuniaSlintState) {
    let personas: Vec<PersonaEntry> = st
        .shell
        .bridge
        .personas()
        .into_iter()
        .map(|persona| PersonaEntry {
            id: persona.id.into(),
            label: persona.label.into(),
            hint: persona.hint.into(),
        })
        .collect();
    win.set_personas(Rc::new(VecModel::from(personas)).into());
}

/// Pushes the command palette index, recording the listed tokens in order for
/// Enter-to-run-first.
///
/// The palette is a second *view* of the same registry as the menu, never a
/// second catalog: a command exists here exactly when the menu offers it (08.2).
fn push_palette_items(win: &MainWindow, st: &mut PetuniaSlintState, query: &str) {
    let needle = query.to_lowercase();
    let index = st.shell.bridge.query_command_index();
    let items: Vec<PaletteItem> = index
        .into_iter()
        .filter(|item| {
            needle.is_empty()
                || item.label.to_lowercase().contains(&needle)
                || item.action_id.to_lowercase().contains(&needle)
                || item.shortcut.to_lowercase().contains(&needle)
        })
        .map(|item| {
            st.palette_filtered.push(item.action_token.clone());
            PaletteItem {
                id: item.action_token.into(),
                label: item.label.into(),
                hint: item.shortcut.into(),
            }
        })
        .collect();
    win.set_palette_items(Rc::new(VecModel::from(items)).into());
}

/// Actions whose destination is a user choice, so the UI must collect it
/// before the Action lane can run (see `activate_token`).
fn needs_destination(action_id: &str) -> bool {
    matches!(
        action_id,
        "ptnd.action.file.open" | "ptnd.action.file.save_as" | "ptnd.action.file.export"
    )
}

/// Runs one menu/palette token through the Action lane.
///
/// Returns the action id that resolved, or `None` for an unknown token.
/// A blocked capability is never dispatched, even if a stale token asks for
/// it: the UI's disabled state and this guard agree because both read the same
/// availability rule (15.F §2).
fn run_action_token(state: &mut PetuniaSlintState, token: &str) -> Option<String> {
    let (item, payload) = menus::item_for_token(token)?;
    let action_id = item.surface.to_string();
    if needs_destination(&action_id) {
        return Some(action_id);
    }
    let availability = menus::availability(&action_id, &state.shell.bridge.action_context());
    if !availability.enabled {
        return Some(action_id);
    }
    let _ = state
        .shell
        .bridge
        .dispatch_action(ActionRequest::new(ActionId::new(&action_id), payload));
    Some(action_id)
}

/// Dispatches an action carrying a user-chosen filesystem path.
fn dispatch_path_action(
    state: &mut PetuniaSlintState,
    action_id: &str,
    path: &std::path::Path,
) -> Result<(), String> {
    let payload = serde_json::json!({ "path": path.to_string_lossy() });
    state
        .shell
        .bridge
        .dispatch_action(ActionRequest::new(ActionId::new(action_id), payload))
        .map(|_| ())
        .map_err(|error| error.to_string())
}

/// Runs one menu/palette activation: dispatches inline actions and opens the
/// dialog or file picker for the ones that need a user-chosen destination.
fn activate_token(win: &MainWindow, state: &Rc<RefCell<PetuniaSlintState>>, token: &str) {
    let action_id = {
        let mut st = state.borrow_mut();
        let Some(action_id) = run_action_token(&mut st, token) else {
            return;
        };
        action_id
    };

    match action_id.as_str() {
        "ptnd.action.file.export" => {
            win.set_export_status_message("".into());
            win.set_export_dialog_open(true);
        }
        "ptnd.action.file.open" => {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Petunia Design Studio Project (*.PTND)", &["PTND", "ptnd"])
                .set_title("Abrir documento")
                .pick_file()
            {
                let mut st = state.borrow_mut();
                if let Err(error) = dispatch_path_action(&mut st, &action_id, &path) {
                    win.set_status_hint(error.into());
                }
            }
        }
        "ptnd.action.file.save_as" => {
            let default_name = state.borrow().shell.bridge.session().map_or_else(
                || "Untitled.PTND".to_string(),
                |s| format!("{}.PTND", s.title()),
            );
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Petunia Design Studio Project (*.PTND)", &["PTND", "ptnd"])
                .set_file_name(&default_name)
                .set_title("Salvar documento como")
                .save_file()
            {
                let mut st = state.borrow_mut();
                if let Err(error) = dispatch_path_action(&mut st, &action_id, &path) {
                    win.set_status_hint(error.into());
                }
            }
        }
        _ => {}
    }

    let st = state.borrow();
    // The session owns the palette flag, so the overlay mirrors it instead of
    // keeping a second opinion (15.B).
    win.set_palette_open(
        st.shell
            .bridge
            .session()
            .is_some_and(|session| session.view.command_palette_open),
    );
    sync_ui_from_shell(win, &st);
}

fn sync_ui_from_shell(window: &MainWindow, state: &PetuniaSlintState) {
    let tool_str = format!("{:?}", state.shell.active_tool());
    window.set_active_tool_name(tool_str.into());

    let zoom_pct = (state.shell.view_camera().zoom * 100.0).round() as i32;
    window.set_zoom_pct(zoom_pct);

    // Registry-driven menu bar plus the shell chrome titles resolved from the
    // string catalog. Both are derived: the UI owns neither the structure nor
    // the strings (15.G, 09.16).
    push_menu(window, state);
    push_personas(window, state);
    push_shell_controls(window, state);
    push_zoom_levels(window, state);
    push_context_toolbar(window, state);
    // The OS window title is a catalog string, not a literal in the .slint.
    window.set_window_title(
        state
            .shell
            .bridge
            .localization()
            .text("ptnd.text.shell.brand", state.shell.bridge.locale())
            .into(),
    );
    {
        let localization = state.shell.bridge.localization();
        let locale = state.shell.bridge.locale().clone();
        window.set_title_panel_layers(localization.text("ptnd.text.panel.layers", &locale).into());
        window.set_title_panel_properties(
            localization
                .text("ptnd.text.panel.properties", &locale)
                .into(),
        );
        window
            .set_title_panel_history(localization.text("ptnd.text.panel.history", &locale).into());
        window.set_title_panel_data_merge(
            localization
                .text("ptnd.text.panel.data_merge", &locale)
                .into(),
        );
        // The session owns the palette flag, so the overlay mirrors it rather
        // than keeping a second opinion (15.B).
        window.set_palette_open(
            state
                .shell
                .bridge
                .session()
                .is_some_and(|session| session.view.command_palette_open),
        );
    }

    // Sync Layers
    let layers: LayersPresentationModel = state.shell.query_layers();
    let layer_items: Vec<LayerRowItem> = layers
        .rows
        .iter()
        .map(|r| {
            let kind = if let Some(session) = state.shell.bridge.session() {
                session
                    .document()
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
                    let mut is_circle = matches!(
                        &obj.shape,
                        Some(petunia_design_document::ShapeKind::Ellipse)
                    );
                    let mut is_path = false;
                    let mut is_text = false;
                    let mut text_content = String::new();
                    let mut text_size = 16.0f32;
                    let mut text_color = token_to_slint("ptnd.gray/900");
                    let mut svg_path = String::new();
                    let mut corner_radius = 0.0f32;
                    let (draw_x, draw_y, draw_w, draw_h);
                    if rotated {
                        if matches!(
                            &obj.shape,
                            Some(petunia_design_document::ShapeKind::Text { .. })
                        ) {
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
                            let rp = obj.to_path().transformed(rot);
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
                            Some(petunia_design_document::ShapeKind::Rectangle {
                                corner_radii,
                            }) => {
                                corner_radius = corner_radii[0] as f32;
                            }
                            Some(petunia_design_document::ShapeKind::Path(path)) => {
                                let local = path.transformed(GAffine::translate(-b[0], -b[1]));
                                svg_path = local.to_svg_path_data();
                                is_path = true;
                                is_circle = false;
                            }
                            Some(petunia_design_document::ShapeKind::Polygon { .. })
                            | Some(petunia_design_document::ShapeKind::Star { .. }) => {
                                let local_path =
                                    obj.to_path().transformed(GAffine::translate(-b[0], -b[1]));
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
                                is_circle =
                                    name_lower.contains("circle") || name_lower.contains("ellipse");
                            }
                        }
                    }
                    if is_text {
                        if let Some(fill) = &obj.fill {
                            text_color = token_to_slint(fill);
                        }
                        if let Some(petunia_design_document::ShapeKind::Text {
                            font_size, ..
                        }) = &obj.shape
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

    // Persona switcher (08.2, 15.G). Switching travels by registered id, and
    // the menu bar is re-derived because persona-scoped families come and go
    // with the mode. The hint is a catalog string, not a literal.
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_switch_persona(move |persona| {
            let mut st = state_clone.borrow_mut();
            if !st.shell.bridge.set_persona(persona.as_str()) {
                return;
            }
            let hint = st
                .shell
                .bridge
                .persona_hint(persona.as_str())
                .unwrap_or_default();
            if let Some(win) = win_weak.upgrade() {
                push_menu(&win, &st);
                win.set_status_hint(hint.into());
            }
        });
    }

    // Canonical context toolbar (08.23). An entry is a fourth *view* of the
    // registry: it resolves to the menu token it was declared with and travels
    // the exact lane a menu row travels, so availability, payload and the
    // blocked reason cannot differ between the bar and the menu.
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_context_toolbar_activated(move |id| {
            let Some(entry) = context_toolbar::entry(id.as_str()) else {
                return;
            };
            if entry.kind != ToolbarEntryKind::Command {
                return;
            }
            let token = entry.token.to_string();
            if let Some(win) = win_weak.upgrade() {
                activate_token(&win, &state_clone, token.as_str());
            }
        });
    }

    // Context-toolbar customization. The dialog echoes indexes; the shell is
    // the only place that mutates order and visibility.
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        let refresh = {
            let state_clone = state_clone.clone();
            let win_weak = win_weak.clone();
            move || {
                if let Some(win) = win_weak.upgrade() {
                    push_context_toolbar(&win, &state_clone.borrow());
                }
            }
        };
        {
            let refresh = refresh.clone();
            let state_clone = state_clone.clone();
            main_window.on_toolbar_set_visible(move |index, visible| {
                state_clone
                    .borrow_mut()
                    .shell
                    .bridge
                    .toolbar_set_slot_visible(index as usize, visible);
                refresh();
            });
        }
        {
            let refresh = refresh.clone();
            let state_clone = state_clone.clone();
            main_window.on_toolbar_move(move |index, delta| {
                state_clone
                    .borrow_mut()
                    .shell
                    .bridge
                    .toolbar_move(index as usize, delta);
                refresh();
            });
        }
        {
            let refresh = refresh.clone();
            let state_clone = state_clone.clone();
            main_window.on_toolbar_add_divider(move |index| {
                let after = if index < 0 {
                    None
                } else {
                    Some(index as usize)
                };
                state_clone
                    .borrow_mut()
                    .shell
                    .bridge
                    .toolbar_insert_divider(after);
                refresh();
            });
        }
        {
            let refresh = refresh.clone();
            let state_clone = state_clone.clone();
            main_window.on_toolbar_remove(move |index| {
                state_clone
                    .borrow_mut()
                    .shell
                    .bridge
                    .toolbar_remove_divider(index as usize);
                refresh();
            });
        }
        {
            let state_clone = state_clone.clone();
            main_window.on_toolbar_reset(move || {
                state_clone.borrow_mut().shell.bridge.toolbar_reset();
                refresh();
            });
        }
    }

    // Centred shell control cluster (08.2). A control is a second *view* of the
    // registry, never a second dispatch path: it resolves to an action id, is
    // checked against the same availability rule the menu uses, and only then
    // travels the Action lane.
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_shell_control_activated(move |id| {
            let Some(action_id) = petunia_design_shell::menu::shell_control_action(id.as_str())
            else {
                return;
            };
            {
                let mut st = state_clone.borrow_mut();
                let availability =
                    menus::availability(action_id, &st.shell.bridge.action_context());
                if !availability.enabled {
                    // A blocked control stays blocked: the guard and the UI's
                    // disabled state read the same rule (15.F §2).
                    return;
                }
                let _ = st.shell.bridge.dispatch_action(ActionRequest::new(
                    ActionId::new(action_id),
                    serde_json::json!({}),
                ));
            }
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &state_clone.borrow());
            }
        });
    }

    // Menu bar (15.G): generated from the surface registry, and every
    // activation travels the same Action-token lane as the palette.
    {
        let win_weak = main_window.as_weak();
        main_window.on_menu_family_toggled(move |index, x| {
            if let Some(win) = win_weak.upgrade() {
                let current = win.get_open_menu_index();
                let opening = current != index;
                win.set_open_menu_index(if opening { index } else { -1 });
                if opening {
                    // Anchor the popup under the family that owns it.
                    win.set_open_menu_x(x);
                }
            }
        });
    }
    {
        let win_weak = main_window.as_weak();
        main_window.on_menu_closed(move || {
            if let Some(win) = win_weak.upgrade() {
                win.set_open_menu_index(-1);
            }
        });
    }
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_menu_item_activated(move |token| {
            if let Some(win) = win_weak.upgrade() {
                activate_token(&win, &state_clone, token.as_str());
            }
        });
    }
    {
        // A zoom level is an ordinary registry item, so it travels the same
        // lane a menu row does: token in, Action out, `sync` back.
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_zoom_level_activated(move |token| {
            if let Some(win) = win_weak.upgrade() {
                activate_token(&win, &state_clone, token.as_str());
            }
        });
    }

    // Command palette (Ctrl+K): opening and closing go through the Action lane
    // (`ptnd.action.view.command_palette`), and the overlay mirrors the session.
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_open_palette(move || {
            let mut st = state_clone.borrow_mut();
            let _ = st
                .shell
                .bridge
                .dispatch_action(ActionRequest::without_payload(ActionId::new(
                    "ptnd.action.view.command_palette",
                )));
            let opened = st
                .shell
                .bridge
                .session()
                .is_some_and(|session| session.view.command_palette_open);
            if let Some(win) = win_weak.upgrade() {
                // The palette and an open menu never share the screen.
                win.set_open_menu_index(-1);
            }
            st.palette_filtered.clear();
            if let Some(win) = win_weak.upgrade() {
                win.set_palette_query("".into());
                if opened {
                    push_palette_items(&win, &mut st, "");
                }
                win.set_palette_open(opened);
            }
        });
    }
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_close_palette(move || {
            let mut st = state_clone.borrow_mut();
            // Closing is idempotent: only dispatch while it is actually open,
            // so a stray Escape cannot reopen the overlay.
            if st
                .shell
                .bridge
                .session()
                .is_some_and(|session| session.view.command_palette_open)
            {
                let _ = st
                    .shell
                    .bridge
                    .dispatch_action(ActionRequest::without_payload(ActionId::new(
                        "ptnd.action.view.command_palette",
                    )));
            }
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
                push_palette_items(&win, &mut st, text.as_str());
            }
        });
    }
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_palette_activate(move |token| {
            if let Some(win) = win_weak.upgrade() {
                activate_token(&win, &state_clone, token.as_str());
            }
        });
    }
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_palette_activate_first(move || {
            let first = state_clone.borrow().palette_filtered.first().cloned();
            if let (Some(token), Some(win)) = (first, win_weak.upgrade()) {
                activate_token(&win, &state_clone, &token);
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
            let mut st = state_clone.borrow_mut();
            let _ = st
                .shell
                .bridge
                .dispatch_action(ActionRequest::without_payload(ActionId::new(
                    "ptnd.action.file.new",
                )));
            if let Some(win) = win_weak.upgrade() {
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Open Document via RFD
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_open_doc_clicked(move || {
            // The dialog picks the path; the Action lane does the work. The
            // old handler created an empty document and printed the filename,
            // which is exactly the fake UI the contract forbids (15.F §2).
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Petunia Design Studio Project (*.PTND)", &["PTND", "ptnd"])
                .set_title("Abrir documento")
                .pick_file()
            {
                let mut st = state_clone.borrow_mut();
                let result = dispatch_path_action(&mut st, "ptnd.action.file.open", &path);
                if let Some(win) = win_weak.upgrade() {
                    if let Err(error) = result {
                        win.set_status_hint(format!("Falha ao abrir: {error}").into());
                    }
                    sync_ui_from_shell(&win, &st);
                }
            }
        });
    }

    // Save Document via RFD
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_save_doc_clicked(move || {
            // `file.save` reuses the recorded path; only a session that has
            // never been written needs a destination, and then the same action
            // lane runs `file.save_as` with the chosen path.
            let known_path = state_clone
                .borrow()
                .shell
                .bridge
                .session()
                .and_then(|session| session.path().map(std::path::Path::to_path_buf));
            let mut st = state_clone.borrow_mut();
            let result = if let Some(path) = known_path {
                dispatch_path_action(&mut st, "ptnd.action.file.save", &path)
            } else {
                let default_name = st.shell.bridge.session().map_or_else(
                    || "Untitled.PTND".to_string(),
                    |s| format!("{}.PTND", s.title()),
                );
                match rfd::FileDialog::new()
                    .add_filter("Petunia Design Studio Project (*.PTND)", &["PTND", "ptnd"])
                    .set_file_name(&default_name)
                    .set_title("Salvar documento")
                    .save_file()
                {
                    Some(path) => dispatch_path_action(&mut st, "ptnd.action.file.save_as", &path),
                    None => Ok(()),
                }
            };
            if let Some(win) = win_weak.upgrade() {
                if let Err(error) = result {
                    win.set_status_hint(format!("Falha ao salvar: {error}").into());
                }
                sync_ui_from_shell(&win, &st);
            }
        });
    }

    // Place Image via RFD
    {
        let state_clone = state.clone();
        let win_weak = main_window.as_weak();
        main_window.on_place_image_clicked(move || {
            // `ptnd.action.file.place` is blocked: the document model has no
            // image object yet, so a placed asset has nowhere to live. The old
            // handler created a coloured rectangle named after the file, which
            // is a fake placement. The UI reports the blocker instead (15.F §2).
            let state = state_clone.borrow();
            let reason = menus::availability(
                "ptnd.action.file.place",
                &state.shell.bridge.action_context(),
            )
            .reason
            .unwrap_or("Placing an asset is not available yet");
            if let Some(win) = win_weak.upgrade() {
                win.set_status_hint(reason.into());
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
            if let Some(surface) = st
                .shell
                .bridge
                .session()
                .and_then(|s| s.document().surfaces().first().cloned())
            {
                let Ok(new_id) = st.shell.bridge.next_object_id() else {
                    return;
                };
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
                let Ok(new_id) = st.shell.bridge.next_object_id() else {
                    return;
                };
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
                let Ok(new_id) = st.shell.bridge.next_object_id() else {
                    return;
                };
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
                let Ok(new_id) = st.shell.bridge.next_object_id() else {
                    return;
                };
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
                let Ok(target_id) = st.shell.bridge.next_object_id() else {
                    return;
                };
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
                let Ok(target_id) = st.shell.bridge.next_object_id() else {
                    return;
                };
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
                let Ok(target_id) = st.shell.bridge.next_object_id() else {
                    return;
                };
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
                let Ok(target_id) = st.shell.bridge.next_object_id() else {
                    return;
                };
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
                        let _ = st
                            .shell
                            .bridge
                            .set_bounds(sel_id, Some(b), deg.to_radians());
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
                let Ok(clone_id) = st.shell.bridge.next_object_id() else {
                    return;
                };
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
                    let _ = st.shell.bridge.submit_command(CommandRequest::new(
                        Command::ReorderObject {
                            surface: surf_id,
                            id,
                            new_index,
                        },
                    ));
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
                    let _ = st.shell.bridge.submit_command(CommandRequest::new(
                        Command::ReorderObject {
                            surface: surf_id,
                            id,
                            new_index,
                        },
                    ));
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
                let current = st
                    .shell
                    .bridge
                    .session()
                    .and_then(|s| s.document().find_object(id).map(|o| (o.bounds, o.rotation)));
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
            // Export is one Action, not three private pipelines: the dialog
            // collects format and destination, then `ptnd.action.file.export`
            // runs the engines (SVG/PDF exporters, CPU compositor for PNG).
            let fmt = win_weak.upgrade().map_or_else(
                || "png".to_string(),
                |win| win.get_export_format().to_string(),
            );
            let (filter_label, extension) = match fmt.as_str() {
                "svg" => ("SVG Vector (*.svg)", "svg"),
                "pdf" => ("PDF Document (*.pdf)", "pdf"),
                _ => ("PNG Image (*.png)", "png"),
            };
            let Some(path) = rfd::FileDialog::new()
                .add_filter(filter_label, &[extension])
                .set_file_name(format!("export.{extension}"))
                .save_file()
            else {
                return;
            };

            let mut st = state_clone.borrow_mut();
            let payload = serde_json::json!({
                "path": path.to_string_lossy(),
                "format": fmt,
            });
            let outcome = st.shell.bridge.dispatch_action(ActionRequest::new(
                ActionId::new("ptnd.action.file.export"),
                payload,
            ));
            if let Some(win) = win_weak.upgrade() {
                match outcome {
                    Ok(_) => win
                        .set_export_status_message(format!("Exportado: {}", path.display()).into()),
                    Err(error) => win
                        .set_export_status_message(format!("Falha na exportação: {error}").into()),
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
            let doc_pt = st.shell.view_camera().screen_to_doc(screen_pt);
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
            let camera = st.shell.view_camera();
            let doc_pt = camera.screen_to_doc(screen_pt);

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
                    if let Some(handle) =
                        hit_test_handle_or_border(doc_box, screen_pt, doc_pt, &camera, 14.0, 8.0)
                    {
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
            let doc_pt = st.shell.view_camera().screen_to_doc(screen_pt);

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
    use std::collections::BTreeSet;

    /// Canonical token declarations from `ui/tokens.slint` (08.35).
    const TOKENS_SLINT: &str = include_str!("../ui/tokens.slint");
    /// Shell markup that consumes them (15.F).
    const APP_SLINT: &str = include_str!("../ui/app.slint");

    #[test]
    fn slint_app_smoke_test_headless() {
        let mut state = PetuniaSlintState::new();
        if let Err(failure) = state.smoke_test() {
            panic!("slint smoke test failed: {failure}");
        }
    }

    /// Every `out property` of the `Tokens` global, by name.
    fn declared_tokens(source: &str) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        for line in source.lines() {
            let line = line.trim();
            let Some(rest) = line.strip_prefix("out property <") else {
                continue;
            };
            let Some((_ty, name)) = rest.split_once('>') else {
                continue;
            };
            let name = name.trim();
            let Some(name) = name.split(':').next() else {
                continue;
            };
            let name = name.trim();
            if !name.is_empty() {
                names.insert(name.to_string());
            }
        }
        names
    }

    /// Every `Tokens.<name>` reference in a source file.
    fn referenced_tokens(source: &str) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        let mut rest = source;
        while let Some(position) = rest.find("Tokens.") {
            let after = &rest[position + "Tokens.".len()..];
            let name: String = after
                .chars()
                .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
                .collect();
            if !name.is_empty() {
                names.insert(name.clone());
            }
            rest = &after[name.len()..];
        }
        names
    }

    #[test]
    fn ui_references_only_declared_tokens() {
        let declared = declared_tokens(TOKENS_SLINT);
        let referenced = referenced_tokens(APP_SLINT);
        assert!(
            declared.len() >= 100,
            "tokens.slint should declare the full 08.35 set, found {}",
            declared.len()
        );
        let dangling: Vec<_> = referenced.difference(&declared).collect();
        assert!(
            dangling.is_empty(),
            "ui/app.slint references undeclared tokens: {dangling:?}"
        );
        assert!(
            referenced.len() >= 60,
            "ui/app.slint should consume the token set, found {} uses",
            referenced.len()
        );
    }

    #[test]
    fn ui_color_literals_are_confined_to_document_artwork() {
        let start = APP_SLINT
            .find("[CanvasObjectItem]> canvas_objects: [")
            .expect("canvas object data block should exist");
        let end = APP_SLINT[start..]
            .find("\n    ];")
            .map(|offset| start + offset)
            .expect("canvas object data block should be closed");

        let mut offenders = Vec::new();
        let mut offset = 0usize;
        for (index, line) in APP_SLINT.lines().enumerate() {
            let in_block = offset >= start && offset <= end;
            if !in_block && has_hex_color(line) {
                offenders.push(index + 1);
            }
            offset += line.len() + 1;
        }
        assert!(
            offenders.is_empty(),
            "raw hex colors are only allowed for document artwork (08.21); lines {offenders:?}"
        );
    }

    /// True when the line carries a `#rrggbb`/`#rgb` style literal.
    fn has_hex_color(line: &str) -> bool {
        let mut rest = line;
        while let Some(position) = rest.find('#') {
            let digits: String = rest[position + 1..]
                .chars()
                .take_while(|c| c.is_ascii_hexdigit())
                .collect();
            if matches!(digits.len(), 3 | 4 | 6 | 8) {
                return true;
            }
            rest = &rest[position + 1..];
        }
        false
    }

    #[test]
    fn ui_shell_rows_use_their_canonical_geometry_tokens() {
        // 08.35 names each chrome row; the mapping is by component, not by
        // value, because 28 px serves several different rows. These are the
        // sites the handoff called out as ambiguous.
        for site in [
            "height: Tokens.menu-row-height;",
            "height: Tokens.persona-row-height;",
            "height: Tokens.context-toolbar-height;",
            "height: Tokens.tab-strip-height + Tokens.space-1;",
            "height: Tokens.status-bar-height;",
            "width: Tokens.tool-rail-width;",
            "width: Tokens.right-dock-default-width;",
            "height: Tokens.panel-tab-height + Tokens.space-1;",
            "height: Tokens.layer-row-height;",
            "height: Tokens.property-field-compact;",
            "width: Tokens.layer-tool-hit-size;",
            "height: Tokens.property-row-height;",
            "height: Tokens.dialog-action-height;",
            "height: Tokens.icon-button-size;",
        ] {
            assert!(
                APP_SLINT.contains(site),
                "ui/app.slint should use the canonical token for `{site}`"
            );
        }
    }
}
