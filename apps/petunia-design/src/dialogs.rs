//! Overlays: command palette and toolbar customization.
//!
//! Both are projections of registry data: the palette lists exactly the
//! enabled menu items, and the customization dialog lists exactly the layout
//! slots the shell owns.

use freya::prelude::*;

use petunia_design_application::ActionId;

use crate::actions::run_action_token;
use crate::chrome::{app_icon, tool_label, tool_shortcut, tool_summary, with_tooltip};
use crate::theme;
use crate::ui_state::{default_tool_rail, RailColumns, ToolGroupConfig, UiShell};

/// Command palette: enabled menu items filtered by the live query.
#[derive(Clone, PartialEq)]
pub struct CommandPalette(pub UiShell);

impl Component for CommandPalette {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let items = use_memo({
            let shell = ui.shell;
            move || shell.read().bridge.query_command_index()
        });
        if !(*ui.palette_open.read()) {
            // Zero-size: a default rect would cover the window and swallow
            // every click meant for the chrome below.
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let query = ui.palette_query.read().clone().to_lowercase();
        let items = items.read().clone();
        let filtered: Vec<_> = items
            .into_iter()
            .filter(|item| {
                query.is_empty()
                    || item.label.to_lowercase().contains(&query)
                    || item.action_id.to_lowercase().contains(&query)
            })
            .take(24)
            .collect();

        let mut palette_open = ui.palette_open;
        let palette_query = ui.palette_query;
        rect()
            .position(Position::new_absolute().top(64.))
            .width(Size::fill())
            .cross_align(Alignment::Center)
            .main_align(Alignment::Center)
            .child(
                rect()
                    .direction(Direction::Vertical)
                    .width(Size::px(480.))
                    .background(theme::SURFACE_PANEL)
                    .padding(Gaps::new_all(theme::SPACE_2))
                    .spacing(theme::SPACE_1)
                    .child(with_tooltip(
                        rect()
                            .width(Size::fill())
                            .child(Input::new(palette_query).placeholder("Type a command…")),
                        ui,
                        "palette:input".to_string(),
                        localized_dialog_text(ui, "ptnd.text.shell.palette"),
                        localized_dialog_text(ui, "ptnd.text.summary.search_commands"),
                        "Ctrl+K".to_string(),
                    ))
                    .children(
                        filtered
                            .iter()
                            .map(|item| palette_row(ui.clone(), item, &mut palette_open)),
                    ),
            )
    }
}

fn palette_row(
    ui: UiShell,
    item: &petunia_design_shell::menu::MenuItemPresentation,
    palette_open: &mut State<bool>,
) -> impl IntoElement {
    let token = item.action_token.clone();
    let mut shell = ui.shell;
    let mut palette_open = *palette_open;
    let mut palette_query = ui.palette_query;
    let mut active_tool = ui.active_tool;
    let mut tool_rail = ui.tool_rail;
    let mut customize_open = ui.customize_open;
    let mut new_doc_open = ui.new_doc_open;
    let mut export_open = ui.export_open;
    let photo = *ui.persona.read() == petunia_design_application::surfaces::PERSONA_PHOTO;
    rect()
        .width(Size::fill())
        .child(
            Button::new()
                .on_press(move |_| {
                    if let Some(action_id) = run_action_token(&mut shell.write(), &token) {
                        if action_id == ActionId::EDIT_PREFERENCES {
                            customize_open.set(true);
                        }
                        if action_id == "ptnd.action.file.new" {
                            new_doc_open.set(true);
                        }
                        if action_id == "ptnd.action.file.export" {
                            export_open.set(true);
                        }
                        if let Some(tool) =
                            petunia_design_application::tools::ToolKind::from_action_id(&action_id)
                        {
                            active_tool.set(tool);
                            tool_rail.write().remember_tool(photo, tool);
                        }
                    }
                    palette_open.set(false);
                    palette_query.set(String::new());
                })
                .child(
                    label()
                        .text(item.label.clone())
                        .color(theme::TEXT_PRIMARY)
                        .font_size(theme::BODY_SIZE),
                ),
        )
}

/// Toolbar customization: every layout slot with show/hide, reorder, divider.
#[derive(Clone, PartialEq)]
pub struct CustomizeDialog(pub UiShell);

impl Component for CustomizeDialog {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        if !(*ui.customize_open.read()) {
            // Zero-size: a default rect would cover the window and swallow
            // every click meant for the chrome below.
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let catalog = ui.shell.peek().bridge.query_toolbar_catalog();
        let title = ui
            .shell
            .peek()
            .bridge
            .localization()
            .text("ptnd.text.shell.customize", ui.shell.peek().bridge.locale());
        let reset_label = ui.shell.peek().bridge.localization().text(
            "ptnd.text.shell.reset_toolbar",
            ui.shell.peek().bridge.locale(),
        );
        let divider_label = ui
            .shell
            .peek()
            .bridge
            .localization()
            .text("ptnd.text.shell.divider", ui.shell.peek().bridge.locale());

        let mut customize_open = ui.customize_open;
        let mut shell_for_reset = ui.shell;
        rect()
            .position(Position::new_absolute().top(74.))
            .width(Size::fill())
            .main_align(Alignment::Center)
            .child(
                Popup::new()
                    .on_close_request(move |_| customize_open.set(false))
                    .child(PopupTitle::new(title))
                    .child(
                        PopupContent::new().child(
                            rect()
                                .direction(Direction::Vertical)
                                .width(Size::px(560.))
                                .spacing(theme::SPACE_1)
                                .children(
                                    catalog
                                        .iter()
                                        .enumerate()
                                        .map(|(index, row)| catalog_row(ui.clone(), index, row)),
                                )
                                .child(
                                    rect()
                                        .direction(Direction::Horizontal)
                                        .spacing(theme::SPACE_2)
                                        .main_align(Alignment::End)
                                        .child(divider_adder(ui.clone(), divider_label))
                                        .child(
                                            Button::new()
                                                .on_press(move |_| {
                                                    shell_for_reset.write().bridge.toolbar_reset();
                                                })
                                                .child(reset_label.clone()),
                                        ),
                                )
                                .child(tool_rail_settings(ui.clone())),
                        ),
                    ),
            )
    }
}

fn catalog_row(
    ui: UiShell,
    index: usize,
    row: &petunia_design_shell::context_toolbar::ToolbarCatalogRow,
) -> impl IntoElement {
    let visible = row.visible;
    let can_hide = row.can_hide;
    let id = row.id.clone();
    let label_text = if row.label.is_empty() {
        "—".to_string()
    } else {
        row.label.clone()
    };
    let mut shell = ui.shell;
    let up_label = localized_dialog_text(&ui, "ptnd.text.shell.move_up");
    let down_label = localized_dialog_text(&ui, "ptnd.text.shell.move_down");
    let row_element = rect()
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .spacing(theme::SPACE_1)
        .cross_align(Alignment::Center)
        .on_press(move |_| {
            if can_hide {
                shell.write().bridge.toolbar_set_visible(&id, !visible);
            }
        })
        .child(app_icon(
            theme::ICON_CHECK,
            *ui.icon_style.read(),
            if visible {
                theme::ACCENT_BLOOM
            } else {
                theme::TEXT_TERTIARY
            },
        ))
        .child(
            label()
                .text(label_text)
                .color(if visible {
                    theme::TEXT_PRIMARY
                } else {
                    theme::TEXT_TERTIARY
                })
                .font_size(theme::BODY_SIZE),
        )
        .child(rect().width(Size::fill()))
        .child(row_mover(ui.clone(), index, -1, up_label))
        .child(row_mover(ui.clone(), index, 1, down_label));

    with_tooltip(
        row_element,
        &ui,
        format!("toolbar-row:{index}"),
        if visible {
            localized_dialog_text(&ui, "ptnd.text.shell.show_toolbar_item")
        } else {
            localized_dialog_text(&ui, "ptnd.text.shell.hide_toolbar_item")
        },
        localized_dialog_text(&ui, "ptnd.text.summary.toggle_toolbar_item"),
        String::new(),
    )
}

fn row_mover(ui: UiShell, index: usize, delta: i32, text: String) -> impl IntoElement {
    let mut shell = ui.shell;
    let title = text.clone();
    with_tooltip(
        rect().child(
            Button::new()
                .on_press(move |_| {
                    shell.write().bridge.toolbar_move(index, delta);
                })
                .child(text),
        ),
        &ui,
        format!("toolbar-move:{index}:{delta}"),
        title,
        localized_dialog_text(&ui, "ptnd.text.summary.reorder_toolbar"),
        String::new(),
    )
}

fn divider_adder(ui: UiShell, divider_label: String) -> impl IntoElement {
    let mut shell = ui.shell;
    with_tooltip(
        rect().child(
            Button::new()
                .on_press(move |_| {
                    shell.write().bridge.toolbar_insert_divider(None);
                })
                .child(divider_label.clone()),
        ),
        &ui,
        "toolbar:add-divider".to_string(),
        divider_label,
        localized_dialog_text(&ui, "ptnd.text.summary.add_divider"),
        String::new(),
    )
}

fn tool_rail_settings(ui: UiShell) -> impl IntoElement {
    let photo = *ui.persona.read() == petunia_design_application::surfaces::PERSONA_PHOTO;
    let state = ui.tool_rail.read().clone();
    let groups = state.groups(photo).to_vec();
    let catalog = tool_catalog(photo);
    let used: Vec<_> = groups
        .iter()
        .flat_map(|group| group.tools.iter())
        .copied()
        .collect();
    let missing: Vec<_> = catalog
        .into_iter()
        .filter(|tool| !used.contains(tool))
        .collect();
    let columns_label = localized_dialog_text(&ui, "ptnd.text.shell.tool_rail");
    let one_label = localized_dialog_text(&ui, "ptnd.text.shell.tool_rail_one_column");
    let two_label = localized_dialog_text(&ui, "ptnd.text.shell.tool_rail_two_columns");
    let reset_label = localized_dialog_text(&ui, "ptnd.text.shell.reset_toolbar");
    let one_active = matches!(state.columns, RailColumns::One);
    let two_active = matches!(state.columns, RailColumns::Two);
    let responsive_label = if one_active {
        localized_dialog_text(&ui, "ptnd.text.shell.tool_rail_responsive_one")
    } else if two_active {
        localized_dialog_text(&ui, "ptnd.text.shell.tool_rail_responsive_two")
    } else {
        localized_dialog_text(&ui, "ptnd.text.shell.tool_rail_responsive")
    };
    let add_tool_label = localized_dialog_text(&ui, "ptnd.text.shell.add_tool");
    let mut rail_one = ui.tool_rail;
    let mut rail_two = ui.tool_rail;
    let mut reset_rail = ui.tool_rail;
    let rail = ui.tool_rail;
    rect()
        .direction(Direction::Vertical)
        .width(Size::fill())
        .spacing(theme::SPACE_1)
        .child(label().text(columns_label).color(theme::TEXT_PRIMARY))
        .child(
            rect()
                .direction(Direction::Horizontal)
                .spacing(theme::SPACE_1)
                .child(
                    Button::new()
                        .on_press(move |_| rail_one.write().columns = RailColumns::One)
                        .child(one_label),
                )
                .child(
                    Button::new()
                        .on_press(move |_| rail_two.write().columns = RailColumns::Two)
                        .child(two_label),
                ),
        )
        .child(
            Button::new()
                .on_press(move |_| reset_rail.write().clone_from(&default_tool_rail()))
                .child(reset_label),
        )
        .child(label().text(responsive_label))
        .children(groups.iter().enumerate().map(|(index, group)| {
            tool_group_settings_row(ui.clone(), photo, index, groups.len(), group)
        }))
        .child(label().text(add_tool_label).color(theme::TEXT_PRIMARY))
        .children(missing.into_iter().map(|tool| {
            let mut rail = rail;
            let label_text = format!("+ {}", tool_label(&ui, tool));
            Button::new()
                .on_press(move |_| {
                    if let Some(groups) = rail.write().groups_mut(photo).get_mut(0) {
                        if !groups.tools.contains(&tool) {
                            groups.tools.push(tool);
                        }
                    }
                })
                .child(label_text)
        }))
}

fn tool_group_settings_row(
    ui: UiShell,
    photo: bool,
    index: usize,
    group_count: usize,
    group: &ToolGroupConfig,
) -> impl IntoElement {
    let group_label = localized_dialog_text(&ui, group.label_id);
    let visible_label = if group.visible {
        localized_dialog_text(&ui, "ptnd.text.shell.visible")
    } else {
        localized_dialog_text(&ui, "ptnd.text.shell.hidden")
    };
    let up_label = localized_dialog_text(&ui, "ptnd.text.shell.move_up");
    let down_label = localized_dialog_text(&ui, "ptnd.text.shell.move_down");
    let merge_label = localized_dialog_text(&ui, "ptnd.text.shell.merge_group");
    let mut visibility = ui.tool_rail;
    let mut up = ui.tool_rail;
    let mut down = ui.tool_rail;
    let mut merge = ui.tool_rail;
    let tools = group.tools.clone();
    rect()
        .direction(Direction::Vertical)
        .width(Size::fill())
        .spacing(theme::SPACE_1)
        .child(
            rect()
                .direction(Direction::Horizontal)
                .spacing(theme::SPACE_1)
                .cross_align(Alignment::Center)
                .child(label().text(group_label).color(theme::TEXT_PRIMARY))
                .child(rect().width(Size::fill()))
                .child(
                    Button::new()
                        .on_press(move |_| visibility.write().toggle_group(photo, index))
                        .child(visible_label),
                )
                .child(
                    Button::new()
                        .on_press(move |_| up.write().move_group(photo, index, -1))
                        .child(up_label),
                )
                .child(
                    Button::new()
                        .on_press(move |_| down.write().move_group(photo, index, 1))
                        .child(down_label),
                )
                .maybe(index > 0, |el| {
                    el.child(
                        Button::new()
                            .on_press(move |_| merge.write().merge_with_previous(photo, index))
                            .child(merge_label),
                    )
                }),
        )
        .children(tools.iter().enumerate().map(|(tool_index, tool)| {
            tool_member_settings_row(ui.clone(), photo, index, tool_index, group_count, *tool)
        }))
}

fn tool_member_settings_row(
    ui: UiShell,
    photo: bool,
    group_index: usize,
    tool_index: usize,
    group_count: usize,
    tool: petunia_design_application::tools::ToolKind,
) -> impl IntoElement {
    let name = tool_label(&ui, tool);
    let summary = tool_summary(&ui, tool);
    let shortcut = tool_shortcut(tool);
    let mut up = ui.tool_rail;
    let mut down = ui.tool_rail;
    let mut previous_group = ui.tool_rail;
    let mut next_group = ui.tool_rail;
    let mut remove = ui.tool_rail;
    let previous_label = localized_dialog_text(&ui, "ptnd.text.shell.previous_group");
    let next_label = localized_dialog_text(&ui, "ptnd.text.shell.next_group");
    let remove_label = localized_dialog_text(&ui, "ptnd.text.shell.remove");
    rect()
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .spacing(theme::SPACE_1)
        .child(
            label()
                .text(format!("{name} ({shortcut})"))
                .color(theme::TEXT_SECONDARY),
        )
        .child(
            label()
                .text(summary)
                .color(theme::TEXT_TERTIARY)
                .font_size(theme::CAPTION_SIZE),
        )
        .child(rect().width(Size::fill()))
        .child(
            Button::new()
                .on_press(move |_| up.write().move_tool(photo, group_index, tool_index, -1))
                .child("↑"),
        )
        .child(
            Button::new()
                .on_press(move |_| down.write().move_tool(photo, group_index, tool_index, 1))
                .child("↓"),
        )
        .maybe(group_index > 0, |el| {
            el.child(
                Button::new()
                    .on_press(move |_| {
                        let _ = previous_group.write().move_tool_to_group(
                            photo,
                            group_index,
                            tool_index,
                            group_index - 1,
                        );
                    })
                    .child(previous_label),
            )
        })
        .maybe(group_index + 1 < group_count, |el| {
            el.child(
                Button::new()
                    .on_press(move |_| {
                        let _ = next_group.write().move_tool_to_group(
                            photo,
                            group_index,
                            tool_index,
                            group_index + 1,
                        );
                    })
                    .child(next_label),
            )
        })
        .child(
            Button::new()
                .on_press(move |_| remove.write().toggle_tool(photo, group_index, tool))
                .child(remove_label),
        )
}

fn tool_catalog(photo: bool) -> Vec<petunia_design_application::tools::ToolKind> {
    use petunia_design_application::tools::ToolKind;
    if photo {
        vec![
            ToolKind::Select,
            ToolKind::MarqueeRect,
            ToolKind::MarqueeEllipse,
            ToolKind::Lasso,
            ToolKind::SelectionBrush,
            ToolKind::FloodSelect,
            ToolKind::Crop,
            ToolKind::PhotoGradient,
            ToolKind::Zoom,
            ToolKind::Hand,
        ]
    } else {
        vec![
            ToolKind::Select,
            ToolKind::Node,
            ToolKind::Perspective,
            ToolKind::Pen,
            ToolKind::Pencil,
            ToolKind::Corner,
            ToolKind::Contour,
            ToolKind::Knife,
            ToolKind::Scissors,
            ToolKind::Rectangle,
            ToolKind::Ellipse,
            ToolKind::Polygon,
            ToolKind::Star,
            ToolKind::ShapeBuilder,
            ToolKind::VectorFloodFill,
            ToolKind::ArtisticText,
            ToolKind::FrameText,
            ToolKind::Gradient,
            ToolKind::Transparency,
            ToolKind::ColorPicker,
            ToolKind::StylePicker,
            ToolKind::Artboard,
            ToolKind::Measure,
            ToolKind::Zoom,
            ToolKind::Hand,
        ]
    }
}

fn localized_dialog_text(ui: &UiShell, text_id: &str) -> String {
    let shell = ui.shell.peek();
    let bridge = &shell.bridge;
    bridge.localization().text(text_id, bridge.locale())
}

/// Dialog to configure and create a new document with dimension presets.
#[derive(Clone, PartialEq)]
pub struct NewDocumentDialog(pub UiShell);

impl Component for NewDocumentDialog {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let doc_name = use_state(|| "Novo Documento".to_string());
        let width_val = use_state(|| 1920.0f64);
        let height_val = use_state(|| 1080.0f64);

        if !(*ui.new_doc_open.read()) {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let mut new_doc_open = ui.new_doc_open;
        let mut shell = ui.shell;

        let mut w_state = width_val;
        let mut h_state = height_val;

        rect()
            .position(Position::new_absolute().top(80.))
            .width(Size::fill())
            .cross_align(Alignment::Center)
            .main_align(Alignment::Center)
            .child(
                rect()
                    .direction(Direction::Vertical)
                    .width(Size::px(460.))
                    .background(theme::SURFACE_PANEL)
                    .border(
                        Border::new()
                            .fill(theme::SURFACE_CHROME_STRONG)
                            .width(1.)
                            .alignment(BorderAlignment::Inner),
                    )
                    .padding(Gaps::new_all(theme::SPACE_3))
                    .spacing(theme::SPACE_2)
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .width(Size::fill())
                            .main_align(Alignment::SpaceBetween)
                            .cross_align(Alignment::Center)
                            .child(
                                label()
                                    .text("Novo Documento")
                                    .font_size(14.)
                                    .color(theme::TEXT_PRIMARY),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        new_doc_open.set(false);
                                    })
                                    .child(label().text("✕").font_size(12.)),
                            ),
                    )
                    .child(
                        label()
                            .text("PRESETS DE TAMANHO")
                            .font_size(10.)
                            .color(theme::TEXT_TERTIARY),
                    )
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .width(Size::fill())
                            .spacing(theme::SPACE_1)
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        w_state.set(1920.0);
                                        h_state.set(1080.0);
                                    })
                                    .child(label().text("Web 1080p").font_size(11.)),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        w_state.set(1080.0);
                                        h_state.set(1080.0);
                                    })
                                    .child(label().text("Square (1080p)").font_size(11.)),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        w_state.set(390.0);
                                        h_state.set(844.0);
                                    })
                                    .child(label().text("Mobile").font_size(11.)),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        w_state.set(595.0);
                                        h_state.set(842.0);
                                    })
                                    .child(label().text("A4").font_size(11.)),
                            ),
                    )
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .width(Size::fill())
                            .main_align(Alignment::SpaceBetween)
                            .cross_align(Alignment::Center)
                            .child(
                                label()
                                    .text(format!(
                                        "Dimensões: {:.0} × {:.0} pt",
                                        *width_val.read(),
                                        *height_val.read()
                                    ))
                                    .font_size(12.)
                                    .color(theme::TEXT_SECONDARY),
                            ),
                    )
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .width(Size::fill())
                            .main_align(Alignment::End)
                            .spacing(theme::SPACE_1)
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        new_doc_open.set(false);
                                    })
                                    .child(label().text("Cancelar").font_size(11.)),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        let name = doc_name.peek().clone();
                                        let w = *width_val.peek();
                                        let h = *height_val.peek();
                                        let mut s = shell.write();
                                        if s.new_document(&name).is_ok() {
                                            if let Some(surf_id) = s.bridge.active_surface() {
                                                let _ = s.bridge.submit_all(
                                                    "Set surface geometry",
                                                    vec![petunia_design_application::Command::SetSurfaceGeometry {
                                                        surface: surf_id,
                                                        origin: [0.0, 0.0],
                                                        dimensions: [w, h],
                                                    }],
                                                );
                                            }
                                        }
                                        new_doc_open.set(false);
                                    })
                                    .child(label().text("Criar").font_size(11.)),
                            ),
                    ),
            )
    }
}

/// Dialog to export the current document to PNG, SVG, or PDF.
#[derive(Clone, PartialEq)]
pub struct ExportDialog(pub UiShell);

impl Component for ExportDialog {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let export_format = use_state(|| "png".to_string());
        let export_path = use_state(|| "export_output.png".to_string());

        if !(*ui.export_open.read()) {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let mut export_open = ui.export_open;
        let mut shell = ui.shell;

        let mut fmt_png = export_format;
        let mut fmt_svg = export_format;
        let mut fmt_pdf = export_format;
        let mut p_png = export_path;
        let mut p_svg = export_path;
        let mut p_pdf = export_path;

        let active_fmt = export_format.read().clone();

        rect()
            .position(Position::new_absolute().top(80.))
            .width(Size::fill())
            .cross_align(Alignment::Center)
            .main_align(Alignment::Center)
            .child(
                rect()
                    .direction(Direction::Vertical)
                    .width(Size::px(440.))
                    .background(theme::SURFACE_PANEL)
                    .border(
                        Border::new()
                            .fill(theme::SURFACE_CHROME_STRONG)
                            .width(1.)
                            .alignment(BorderAlignment::Inner),
                    )
                    .padding(Gaps::new_all(theme::SPACE_3))
                    .spacing(theme::SPACE_2)
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .width(Size::fill())
                            .main_align(Alignment::SpaceBetween)
                            .cross_align(Alignment::Center)
                            .child(
                                label()
                                    .text("Exportar Arte / Documento")
                                    .font_size(14.)
                                    .color(theme::TEXT_PRIMARY),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        export_open.set(false);
                                    })
                                    .child(label().text("✕").font_size(12.)),
                            ),
                    )
                    .child(
                        label()
                            .text("FORMATO DE EXPORTAÇÃO")
                            .font_size(10.)
                            .color(theme::TEXT_TERTIARY),
                    )
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .width(Size::fill())
                            .spacing(theme::SPACE_1)
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        fmt_png.set("png".to_string());
                                        p_png.set("export_output.png".to_string());
                                    })
                                    .child(
                                        label()
                                            .text(if active_fmt == "png" {
                                                "✓ PNG (Bitmap)"
                                            } else {
                                                "PNG (Bitmap)"
                                            })
                                            .font_size(11.),
                                    ),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        fmt_svg.set("svg".to_string());
                                        p_svg.set("export_output.svg".to_string());
                                    })
                                    .child(
                                        label()
                                            .text(if active_fmt == "svg" {
                                                "✓ SVG (Vetorial)"
                                            } else {
                                                "SVG (Vetorial)"
                                            })
                                            .font_size(11.),
                                    ),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        fmt_pdf.set("pdf".to_string());
                                        p_pdf.set("export_output.pdf".to_string());
                                    })
                                    .child(
                                        label()
                                            .text(if active_fmt == "pdf" {
                                                "✓ PDF (Documento)"
                                            } else {
                                                "PDF (Documento)"
                                            })
                                            .font_size(11.),
                                    ),
                            ),
                    )
                    .child(
                        rect()
                            .width(Size::fill())
                            .child(Input::new(export_path).placeholder("Caminho do arquivo...")),
                    )
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .width(Size::fill())
                            .main_align(Alignment::End)
                            .spacing(theme::SPACE_1)
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        export_open.set(false);
                                    })
                                    .child(label().text("Cancelar").font_size(11.)),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        let path = export_path.peek().clone();
                                        let fmt = export_format.peek().clone();
                                        let payload = serde_json::json!({
                                            "path": path,
                                            "format": fmt,
                                        });
                                        let res = shell.write().bridge.dispatch_action(
                                            petunia_design_application::ActionRequest::new(
                                                petunia_design_application::ActionId::new(
                                                    "ptnd.action.file.export",
                                                ),
                                                payload,
                                            ),
                                        );
                                        if res.is_ok() {
                                            export_open.set(false);
                                        }
                                    })
                                    .child(label().text("Exportar").font_size(11.)),
                            ),
                    ),
            )
    }
}

/// Dialog to confirm closing a document with unsaved changes.
#[derive(Clone, PartialEq)]
pub struct ConfirmCloseDialog(pub UiShell);

impl Component for ConfirmCloseDialog {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        if !(*ui.confirm_close_open.read()) {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let mut confirm_close_open = ui.confirm_close_open;
        let mut shell = ui.shell;

        rect()
            .position(Position::new_absolute().top(120.))
            .width(Size::fill())
            .cross_align(Alignment::Center)
            .main_align(Alignment::Center)
            .child(
                rect()
                    .direction(Direction::Vertical)
                    .width(Size::px(400.))
                    .background(theme::SURFACE_PANEL)
                    .border(
                        Border::new()
                            .fill(theme::SURFACE_CHROME_STRONG)
                            .width(1.)
                            .alignment(BorderAlignment::Inner),
                    )
                    .padding(Gaps::new_all(theme::SPACE_3))
                    .spacing(theme::SPACE_2)
                    .child(
                        label()
                            .text("Alterações Não Salvas")
                            .font_size(14.)
                            .color(theme::TEXT_PRIMARY),
                    )
                    .child(
                        label()
                            .text("O documento possui alterações não salvas. Deseja fechar e descartar as alterações?")
                            .font_size(12.)
                            .color(theme::TEXT_SECONDARY),
                    )
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .width(Size::fill())
                            .main_align(Alignment::End)
                            .spacing(theme::SPACE_1)
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        confirm_close_open.set(false);
                                    })
                                    .child(label().text("Cancelar").font_size(11.)),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        confirm_close_open.set(false);
                                        let _ = shell.write().new_document("Untitled");
                                    })
                                    .child(label().text("Fechar Sem Salvar").font_size(11.)),
                            ),
                    ),
            )
    }
}

