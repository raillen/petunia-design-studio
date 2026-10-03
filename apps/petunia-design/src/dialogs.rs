//! Overlays: command palette and toolbar customization.
//!
//! Both are projections of registry data: the palette lists exactly the
//! enabled menu items, and the customization dialog lists exactly the layout
//! slots the shell owns.

use freya::prelude::*;

use petunia_design_application::{ActionId, ActionRequest};

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
    let mut palette_open = *palette_open;
    let mut palette_query = ui.palette_query;
    let mut active_tool = ui.active_tool;
    let mut tool_rail = ui.tool_rail;
    let mut customize_open = ui.customize_open;
    let mut offset_prompt_open = ui.offset_prompt_open;
    let photo = *ui.persona.read() == petunia_design_application::surfaces::PERSONA_PHOTO;
    rect().width(Size::fill()).child(
        Button::new()
            .on_press(move |_| {
                let action_id = crate::actions::run_ui_token(&ui, &token);
                if let Some(action_id) = action_id {
                    if action_id == ActionId::EDIT_PREFERENCES {
                        customize_open.set(true);
                    }

                    if action_id == "ptnd.action.object.offset_path" {
                        offset_prompt_open.set(true);
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

///// Preferences and toolbar customization dialog.
#[derive(Clone, PartialEq)]
pub struct CustomizeDialog(pub UiShell);

impl Component for CustomizeDialog {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let active_tab = use_state(|| 0usize);

        if !(*ui.customize_open.read()) {
            // Zero-size: a default rect would cover the window and swallow
            // every click meant for the chrome below.
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }

        let mut tab_tb = active_tab;
        let mut tab_gen = active_tab;
        let mut tab_perf = active_tab;
        let mut tab_sc = active_tab;
        let current_tab = *active_tab.read();

        let catalog = ui.shell.peek().bridge.query_toolbar_catalog();
        let title = "Preferências & Customização";
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
            .position(Position::new_absolute().top(60.))
            .width(Size::fill())
            .main_align(Alignment::Center)
            .child(
                Popup::new()
                    .on_close_request(move |_| customize_open.set(false))
                    .child(PopupTitle::new(title.to_string()))
                    .child(
                        PopupContent::new().child(
                            rect()
                                .direction(Direction::Vertical)
                                .width(Size::px(580.))
                                .spacing(theme::SPACE_2)
                                .child(
                                    rect()
                                        .direction(Direction::Horizontal)
                                        .width(Size::fill())
                                        .spacing(theme::SPACE_1)
                                        .child(
                                            Button::new().on_press(move |_| tab_tb.set(0)).child(
                                                label()
                                                    .text(if current_tab == 0 {
                                                        "✓ Ferramentas"
                                                    } else {
                                                        "Ferramentas"
                                                    })
                                                    .font_size(11.),
                                            ),
                                        )
                                        .child(
                                            Button::new().on_press(move |_| tab_gen.set(1)).child(
                                                label()
                                                    .text(if current_tab == 1 {
                                                        "✓ Geral & Idioma"
                                                    } else {
                                                        "Geral & Idioma"
                                                    })
                                                    .font_size(11.),
                                            ),
                                        )
                                        .child(
                                            Button::new().on_press(move |_| tab_perf.set(2)).child(
                                                label()
                                                    .text(if current_tab == 2 {
                                                        "✓ Desempenho"
                                                    } else {
                                                        "Desempenho"
                                                    })
                                                    .font_size(11.),
                                            ),
                                        )
                                        .child(
                                            Button::new().on_press(move |_| tab_sc.set(3)).child(
                                                label()
                                                    .text(if current_tab == 3 {
                                                        "✓ Atalhos"
                                                    } else {
                                                        "Atalhos"
                                                    })
                                                    .font_size(11.),
                                            ),
                                        ),
                                )
                                .child(match current_tab {
                                    1 => rect().child(preferences_general_tab(ui.clone())),
                                    2 => rect().child(preferences_performance_tab()),
                                    3 => rect().child(preferences_shortcuts_tab()),
                                    _ => rect()
                                        .direction(Direction::Vertical)
                                        .spacing(theme::SPACE_1)
                                        .children(catalog.iter().enumerate().map(|(index, row)| {
                                            catalog_row(ui.clone(), index, row)
                                        }))
                                        .child(
                                            rect()
                                                .direction(Direction::Horizontal)
                                                .spacing(theme::SPACE_2)
                                                .main_align(Alignment::End)
                                                .child(divider_adder(ui.clone(), divider_label))
                                                .child(
                                                    Button::new()
                                                        .on_press(move |_| {
                                                            shell_for_reset
                                                                .write()
                                                                .bridge
                                                                .toolbar_reset();
                                                        })
                                                        .child(reset_label.clone()),
                                                ),
                                        )
                                        .child(tool_rail_settings(ui.clone())),
                                }),
                        ),
                    ),
            )
    }
}

fn preferences_general_tab(ui: UiShell) -> impl IntoElement {
    let mut shell_en = ui.shell;
    let mut shell_pt = ui.shell;
    let is_pt = *ui.shell.peek().bridge.locale() == petunia_design_shell::Locale::PtBr;
    rect()
        .direction(Direction::Vertical)
        .width(Size::fill())
        .spacing(theme::SPACE_2)
        .child(
            label()
                .text("IDIOMA DA INTERFACE / INTERFACE LANGUAGE")
                .font_size(10.)
                .color(theme::TEXT_TERTIARY),
        )
        .child(
            rect()
                .direction(Direction::Horizontal)
                .spacing(theme::SPACE_1)
                .child(
                    Button::new()
                        .on_press(move |_| {
                            shell_pt
                                .write()
                                .bridge
                                .set_locale(petunia_design_shell::Locale::PtBr);
                        })
                        .child(
                            label()
                                .text(if is_pt {
                                    "✓ Português (pt-BR)"
                                } else {
                                    "Português (pt-BR)"
                                })
                                .font_size(11.),
                        ),
                )
                .child(
                    Button::new()
                        .on_press(move |_| {
                            shell_en
                                .write()
                                .bridge
                                .set_locale(petunia_design_shell::Locale::EnUs);
                        })
                        .child(
                            label()
                                .text(if !is_pt {
                                    "✓ English (en-US)"
                                } else {
                                    "English (en-US)"
                                })
                                .font_size(11.),
                        ),
                ),
        )
        .child(
            label()
                .text("PALETA DE CORES / THEME ACCENT")
                .font_size(10.)
                .color(theme::TEXT_TERTIARY),
        )
        .child(
            rect()
                .direction(Direction::Horizontal)
                .spacing(theme::SPACE_1)
                .child(
                    rect()
                        .padding(Gaps::new_all(4.))
                        .background(theme::ACCENT_BLOOM)
                        .child(label().text("Bloom").font_size(11.).color(Color::WHITE)),
                )
                .child(
                    rect()
                        .padding(Gaps::new_all(4.))
                        .background(theme::STUDIO_DESIGN)
                        .child(label().text("Design").font_size(11.).color(Color::WHITE)),
                )
                .child(
                    rect()
                        .padding(Gaps::new_all(4.))
                        .background(theme::STUDIO_PHOTO)
                        .child(label().text("Photo").font_size(11.).color(Color::WHITE)),
                ),
        )
        .child(
            label()
                .text("HISTÓRICO E ARQUIVO")
                .font_size(10.)
                .color(theme::TEXT_TERTIARY),
        )
        .child(
            label()
                .text("Limite de histórico não-destrutivo: Ilimitado (Branching Timeline)")
                .font_size(11.)
                .color(theme::TEXT_SECONDARY),
        )
}

fn preferences_performance_tab() -> impl IntoElement {
    rect()
        .direction(Direction::Vertical)
        .width(Size::fill())
        .spacing(theme::SPACE_2)
        .child(
            label()
                .text("MOTOR DE RENDERIZAÇÃO")
                .font_size(10.)
                .color(theme::TEXT_TERTIARY),
        )
        .child(
            label()
                .text("✓ Skia GPU Rasterizer Ativo (RenderCallback Direto)")
                .font_size(11.)
                .color(theme::TEXT_PRIMARY),
        )
        .child(
            label()
                .text("QUALIDADE DO PREVIEW (LOD)")
                .font_size(10.)
                .color(theme::TEXT_TERTIARY),
        )
        .child(
            label()
                .text("✓ Nível de Detalhe Adaptativo (Curvas Bézier & Shaders Skia)")
                .font_size(11.)
                .color(theme::TEXT_SECONDARY),
        )
        .child(
            label()
                .text("SUAVIZAÇÃO (ANTI-ALIASING)")
                .font_size(10.)
                .color(theme::TEXT_TERTIARY),
        )
        .child(
            label()
                .text("✓ Subpixel Antialiasing em Espaço de Cor 32-bit RGBA")
                .font_size(11.)
                .color(theme::TEXT_SECONDARY),
        )
}

fn preferences_shortcuts_tab() -> impl IntoElement {
    let shortcuts = [
        ("V", "Ferramenta de Seleção / Mover"),
        ("A", "Edição de Nós e Âncoras (Node Tool)"),
        ("P", "Caneta Vetorial Bézier (Pen Tool)"),
        ("N", "Lápis de Traçado Livre (Pencil Tool)"),
        ("M", "Formas Paramétricas (Retângulo / Elipse)"),
        ("T", "Texto Artístico e Parágrafos"),
        ("G", "Gradiente Linear e Radial"),
        ("Z", "Zoom / Lupa"),
        ("H / Espaço", "Navegação / Pan"),
        ("Ctrl + Z", "Desfazer Operação (Undo)"),
        ("Ctrl + Shift + Z", "Refazer Operação (Redo)"),
        ("Ctrl + K", "Paleta de Comandos Rápidos"),
        ("Ctrl + N", "Novo Documento"),
        ("Ctrl + E", "Exportar Documento"),
        ("Ctrl + 0", "Enquadrar Superfície na Tela"),
        ("Ctrl + 1", "Zoom Real 100%"),
    ];

    rect()
        .direction(Direction::Vertical)
        .width(Size::fill())
        .spacing(theme::SPACE_1)
        .children(shortcuts.iter().map(|(key, desc)| {
            rect()
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .padding(Gaps::new_all(2.))
                .child(
                    rect()
                        .padding(Gaps::new(2., 6., 2., 6.))
                        .background(theme::SURFACE_CHROME_STRONG)
                        .child(label().text(*key).font_size(11.).color(theme::TEXT_PRIMARY)),
                )
                .child(
                    label()
                        .text(*desc)
                        .font_size(11.)
                        .color(theme::TEXT_SECONDARY),
                )
        }))
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
        let doc_name = use_state(|| "Untitled".to_string());
        let width_val = use_state(|| 1920.0f64);
        let height_val = use_state(|| 1080.0f64);
        let bleed_val = use_state(|| 0.0f64);
        let margin_val = use_state(|| 0.0f64);
        let mut error = use_state(|| None::<String>);
        let observed_open = ui.new_doc_open;
        use_side_effect(move || {
            let _ = *observed_open.read();
            error.set(None);
        });
        let error_text = error.read().clone();
        let failure = ui.text("failed");

        if !(*ui.new_doc_open.read()) {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let mut new_doc_open = ui.new_doc_open;
        let mut shell = ui.shell;

        let mut w_state = width_val;
        let mut h_state = height_val;
        let mut bleed_state = bleed_val;
        let mut margin_state = margin_val;

        let cur_w = *width_val.read();
        let cur_h = *height_val.read();
        let cur_bleed = *bleed_val.read();
        let cur_margin = *margin_val.read();

        rect().child(Popup::new().width(Size::px(560.)).max_width(Size::window_percent(96.)).on_close_request(move |_| new_doc_open.set(false))
            .child(PopupContent::new().child(
                rect()
                    .direction(Direction::Vertical)
                    .width(Size::fill())
                    .spacing(theme::SPACE_2)
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .width(Size::fill())
                            .main_align(Alignment::SpaceBetween)
                            .cross_align(Alignment::Center)
                            .child(
                                label()
                                    .text(ui.text("new"))
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
                        rect()
                            .direction(Direction::Vertical)
                            .width(Size::fill())
                            .spacing(theme::SPACE_1)
                            .child(
                                label()
                                    .text(ui.text("name"))
                                    .font_size(10.)
                                    .color(theme::TEXT_TERTIARY),
                            )
                            .child(
                                Input::new(doc_name).placeholder(ui.text("name")),
                            ),
                    )
                    .child(
                        label()
                            .text(ui.text("presets"))
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
                                        w_state.set(3840.0);
                                        h_state.set(2160.0);
                                    })
                                    .child(label().text("4K UHD").font_size(11.)),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        w_state.set(1080.0);
                                        h_state.set(1080.0);
                                    })
                                    .child(label().text(ui.text("square")).font_size(11.)),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        w_state.set(390.0);
                                        h_state.set(844.0);
                                    })
                                    .child(label().text(ui.text("mobile")).font_size(11.)),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        w_state.set(595.0);
                                        h_state.set(842.0);
                                    })
                                    .child(label().text("A4").font_size(11.)),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        w_state.set(612.0);
                                        h_state.set(792.0);
                                    })
                                    .child(label().text("Letter").font_size(11.)),
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
                                    .text(format!("{}: {:.0} × {:.0} pt", ui.text("dimensions"), cur_w, cur_h))
                                    .font_size(12.)
                                    .color(theme::TEXT_SECONDARY),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        let (w, h) = (*w_state.peek(), *h_state.peek());
                                        w_state.set(h);
                                        h_state.set(w);
                                    })
                                    .child(label().text(ui.text("rotate_orientation")).font_size(11.)),
                            ),
                    )
                    .child(
                        label()
                            .text(ui.text("bleed"))
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
                                    .on_press(move |_| bleed_state.set(0.0))
                                    .child(
                                        label()
                                            .text(format!("{}{}", if cur_bleed == 0.0 {"✓ "} else {""}, ui.text("no_bleed")))
                                            .font_size(11.),
                                    ),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| bleed_state.set(8.5))
                                    .child(
                                        label()
                                            .text(if (cur_bleed - 8.5).abs() < 0.1 {
                                                "✓ 3 mm (8.5 pt)"
                                            } else {
                                                "3 mm (8.5 pt)"
                                            })
                                            .font_size(11.),
                                    ),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| bleed_state.set(14.2))
                                    .child(
                                        label()
                                            .text(if (cur_bleed - 14.2).abs() < 0.1 {
                                                "✓ 5 mm (14.2 pt)"
                                            } else {
                                                "5 mm (14.2 pt)"
                                            })
                                            .font_size(11.),
                                    ),
                            ),
                    )
                    .child(
                        label()
                            .text(ui.text("margins"))
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
                                    .on_press(move |_| margin_state.set(0.0))
                                    .child(
                                        label()
                                            .text(if cur_margin == 0.0 { "✓ 0 pt" } else { "0 pt" })
                                            .font_size(11.),
                                    ),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| margin_state.set(10.0))
                                    .child(
                                        label()
                                            .text(if (cur_margin - 10.0).abs() < 0.1 {
                                                "✓ 10 pt"
                                            } else {
                                                "10 pt"
                                            })
                                            .font_size(11.),
                                    ),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| margin_state.set(20.0))
                                    .child(
                                        label()
                                            .text(if (cur_margin - 20.0).abs() < 0.1 {
                                                "✓ 20 pt"
                                            } else {
                                                "20 pt"
                                            })
                                            .font_size(11.),
                                    ),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| margin_state.set(36.0))
                                    .child(
                                        label()
                                            .text(if (cur_margin - 36.0).abs() < 0.1 {
                                                "✓ 36 pt (0.5\")"
                                            } else {
                                                "36 pt (0.5\")"
                                            })
                                            .font_size(11.),
                                    ),
                            ),
                    )
                    .child(
                        label()
                            .text(ui.text("color_mode"))
                            .font_size(10.)
                            .color(theme::TEXT_TERTIARY),
                    )
                    .child(label().text("RGB").color(theme::TEXT_PRIMARY))
                    .child(label().text(ui.text("color_unavailable")).color(theme::TEXT_SECONDARY))
                    .children(error_text.into_iter().map(|message| label().text(message).color(PROMPT_ERROR)))
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
                                    .child(label().text(ui.text("cancel")).font_size(11.)),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        let name = doc_name.peek().clone();
                                        let w = *width_val.peek();
                                        let h = *height_val.peek();
                                        let bleed_amt = *bleed_val.peek();
                                        let margin_amt = *margin_val.peek();
                                        let result=crate::file_workflows::create_configured_document(&mut shell.write(),&name,[w,h],bleed_amt,margin_amt);
                                        match result {
                                            Ok(())=>{error.set(None);new_doc_open.set(false);},
                                            Err(reason)=>error.set(Some(format!("{failure}: {reason}"))),
                                        }
                                    })
                                    .child(label().text(ui.text("create")).font_size(11.)),
                            ),
                    )
            )))
    }
}

/// Dialog to export the current document to PNG, SVG, or PDF.
#[derive(Clone, PartialEq)]
pub struct ExportDialog(pub UiShell);

/// Resolves the exact request used for preview, overwrite checks and dispatch.
/// Normalization happens before asking to replace an existing destination.
pub fn desktop_export_payload(
    format: &str,
    path: &str,
    dpi: u32,
    surface: Option<petunia_design_foundation::SurfaceId>,
) -> Result<
    (
        serde_json::Value,
        petunia_design_application::export_service::ExportRequest,
    ),
    petunia_design_foundation::PetuniaError,
> {
    let mut payload = serde_json::json!({"format":format,"path":path.trim(),"dpi":dpi});
    if let Some(surface) = surface {
        payload["surface"] = serde_json::json!(surface.raw());
    }
    let request =
        petunia_design_application::export_service::ExportRequest::from_payload(&payload, surface)?;
    payload["path"] = serde_json::json!(request.path);
    Ok((payload, request))
}

impl Component for ExportDialog {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let mut format = use_state(|| "png".to_string());
        let mut path = use_state(|| "export_output.png".to_string());
        let mut dpi = use_state(|| 72u32);
        let mut error = use_state(|| None::<String>);
        let mut replace = use_state(|| None::<std::path::PathBuf>);
        let mut open = ui.export_open;
        use_side_effect(move || {
            let _ = *open.read();
            error.set(None);
            replace.set(None);
        });
        if !*open.read() {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let active_format = format.read().clone();
        let density = *dpi.read();
        let surf = ui.shell.peek().bridge.active_surface();
        let prepared = desktop_export_payload(&active_format, &path.read(), density, surf);
        let destination = prepared
            .as_ref()
            .ok()
            .map(|(_, request)| request.path.clone());
        let confirmed = destination.is_some() && *replace.read() == destination;
        let surface_info = ui
            .shell
            .peek()
            .bridge
            .session()
            .and_then(|session| surf.and_then(|id| session.document().surface(id).ok()))
            .map(|surface| {
                format!(
                    "{}: {:.0} × {:.0} px",
                    ui.text("export_dimensions"),
                    (surface.dimensions[0] * f64::from(density) / 72.).ceil(),
                    (surface.dimensions[1] * f64::from(density) / 72.).ceil()
                )
            });
        let failure = ui.text("failed");
        let export_ui = ui.clone();
        let cancel_ui = ui.clone();
        let close_ui = ui.clone();
        let busy = ui.file_job.read().is_some();
        let error_text = error.read().clone();
        rect().child(Popup::new().width(Size::px(560.)).max_width(Size::window_percent(96.)).on_close_request(move |_| {crate::file_jobs::cancel_export(&close_ui);open.set(false);error.set(None);replace.set(None);})
            .child(PopupTitle::new(ui.text("export")))
            .child(PopupContent::new().child(rect().direction(Direction::Vertical).width(Size::fill()).spacing(theme::SPACE_2)
                .child(label().text(ui.text("format")))
                .child(rect().direction(Direction::Horizontal).spacing(theme::SPACE_1).children(["png","svg","pdf"].into_iter().map(|fmt| {
                    let selected=active_format==fmt;
                    Button::new().on_press(move |_| {
                        format.set(fmt.to_string());
                        if !path.peek().trim().is_empty() {
                            let parsed=petunia_design_application::export_service::ExportFormat::parse(fmt).expect("listed export format");
                            let target=petunia_design_application::export_service::with_format_suffix(std::path::PathBuf::from(path.peek().trim()),parsed);
                            path.set(target.to_string_lossy().into_owned());
                        }
                        replace.set(None);error.set(None);
                    }).child(format!("{}{}",if selected {"✓ "} else {""},fmt.to_uppercase()))
                })))
                .child(label().text(ui.text(if active_format=="png" {"scope_png"} else {"scope_document"})).color(theme::TEXT_SECONDARY))
                .maybe_child((active_format=="png").then(|| rect().direction(Direction::Vertical).spacing(theme::SPACE_1)
                    .child(label().text(ui.text("dpi")))
                    .child(rect().direction(Direction::Horizontal).spacing(theme::SPACE_1).children([72u32,144,300].into_iter().map(|value| {
                        Button::new().on_press(move |_| {dpi.set(value); replace.set(None);}).child(format!("{}{value} DPI",if density==value {"✓ "} else {""}))
                    })))
                    .maybe_child(surface_info.map(|info| label().text(info)))
                    .child(label().text(ui.text("alpha")).color(theme::TEXT_SECONDARY))))
                .child(label().text(ui.text("pdf_limit")).color(theme::TEXT_SECONDARY))
                .maybe_child((active_format=="png").then(||crate::export_preview::ExportPreview(ui.clone())))
                .child(label().text(ui.text("destination")))
                .child(Input::new(path).width(Size::fill()).placeholder(ui.text("destination")))
                .maybe_child(destination.as_ref().map(|path| label().text(path.display().to_string()).color(theme::TEXT_SECONDARY)))
                .children(error_text.into_iter().map(|message| label().text(message).color(PROMPT_ERROR)))
                .maybe_child(confirmed.then(|| label().text(ui.text("overwrite_reason")).color(theme::TEXT_PRIMARY)))
                .child(rect().direction(Direction::Horizontal).main_align(Alignment::End).spacing(theme::SPACE_1)
                    .child(Button::new().on_press(move |_| {crate::file_jobs::cancel_export(&cancel_ui);open.set(false);error.set(None);replace.set(None);}).child(ui.text("cancel")))
                    .child(Button::new().enabled(!busy).on_press(move |_| {
                        let (_,request)=match &prepared {Ok(value)=>value.clone(),Err(reason)=>{error.set(Some(format!("{failure}: {reason}")));return;}};
                        if request.path.exists() && !confirmed {replace.set(Some(request.path));return;}
                        let result=crate::file_jobs::export(&export_ui,request.clone());
                        match result {
                            Ok(_)=>{error.set(None);replace.set(None);},
                            Err(reason)=>error.set(Some(format!("{failure}: {reason}"))),
                        }
                    }).child(ui.text(if confirmed {"overwrite"} else {"export"})))))))
    }
}

/// Dialog to confirm closing a document with unsaved changes.
#[derive(Clone, PartialEq)]
pub struct ConfirmCloseDialog(pub UiShell);

impl Component for ConfirmCloseDialog {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        if !*ui.confirm_close_open.read() {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let mut open = ui.confirm_close_open;
        let mut pending = ui.pending_close;
        let mut shell = ui.shell;
        let save_ui = ui.clone();
        let mut error = ui.file_error;
        let index = *pending.peek();
        let identity = *ui.close_target.peek();
        let target_title = identity.and_then(|id| {
            shell
                .peek()
                .bridge
                .sessions()
                .iter()
                .find(|s| s.identity() == id)
                .map(|s| s.title().to_string())
        });
        let failure = ui.text("failed");
        let save_failure = failure.clone();
        let error_text = error.read().clone();
        rect().child(
            Popup::new()
                .width(Size::px(560.))
                .max_width(Size::window_percent(96.))
                .on_close_request(move |_| {
                    open.set(false);
                    pending.set(None);
                    error.set(None);
                })
                .child(PopupTitle::new(ui.text("close_title")))
                .child(
                    PopupContent::new().child(
                        rect()
                            .direction(Direction::Vertical)
                            .width(Size::fill())
                            .spacing(theme::SPACE_2)
                            .child(label().text(ui.text(if index.is_some() {
                                "close_one"
                            } else {
                                "close_all"
                            })))
                            .maybe_child(
                                target_title
                                    .map(|title| label().text(title).color(theme::TEXT_PRIMARY)),
                            )
                            .children(
                                error_text
                                    .into_iter()
                                    .map(|message| label().text(message).color(PROMPT_ERROR)),
                            )
                            .child(
                                rect()
                                    .direction(Direction::Horizontal)
                                    .main_align(Alignment::End)
                                    .spacing(theme::SPACE_1)
                                    .child(
                                        Button::new()
                                            .on_press(move |_| {
                                                open.set(false);
                                                pending.set(None);
                                                error.set(None);
                                            })
                                            .child(ui.text("cancel")),
                                    )
                                    .child(
                                        Button::new()
                                            .on_press(move |_| {
                                                if index.is_some() && identity.is_none() {
                                                    error.set(Some(failure.clone()));
                                                    return;
                                                }
                                                let result = crate::file_workflows::discard_target(
                                                    &mut shell.write(),
                                                    identity,
                                                );
                                                match result {
                                                    Ok(true) => {
                                                        open.set(false);
                                                        pending.set(None);
                                                        error.set(None);
                                                    }
                                                    Ok(false) => error.set(Some(failure.clone())),
                                                    Err(reason) => error
                                                        .set(Some(format!("{failure}: {reason}"))),
                                                }
                                            })
                                            .child(ui.text("close_discard")),
                                    )
                                    .child(
                                        Button::new()
                                            .on_press(move |_| {
                                                if index.is_some() && identity.is_none() {
                                                    error.set(Some(save_failure.clone()));
                                                    return;
                                                }
                                                match crate::file_jobs::save_before_close(
                                                    &save_ui, identity,
                                                ) {
                                                    Ok(()) => {
                                                        open.set(false);
                                                        pending.set(None);
                                                        error.set(None);
                                                    }
                                                    Err(reason) => error.set(Some(format!(
                                                        "{save_failure}: {reason}"
                                                    ))),
                                                }
                                            })
                                            .child(ui.text("close_save")),
                                    ),
                            ),
                    ),
                ),
        )
    }
}

/// Error ink for rejected prompt values (no theme error token exists yet).
const PROMPT_ERROR: Color = theme::TEXT_ERROR;

/// Parses a typed numeric prompt value in points.
///
/// Trims whitespace, accepts `,` as the decimal separator, and refuses empty,
/// non-numeric or non-finite input with a bilingual message instead of a
/// silent `0.0` (dossier §13: rejeita-ou-explica).
pub fn parse_prompt_distance(raw: &str) -> Result<f64, &'static str> {
    petunia_design_foundation::parse_numeric_input(
        raw,
        petunia_design_foundation::NumericFieldKind::DistancePt,
    )
    .map_err(|err| match err {
        petunia_design_foundation::NumericParseError::Empty => {
            "Digite um valor em pt / Type a value in pt"
        }
        _ => "Valor inválido: use um número em pt / Invalid value: use a number in pt",
    })
}

/// Generic numeric prompt: title + typed value + unit (10.2 `offset_path`).
///
/// The Object menu and the palette open this instead of dispatching a default
/// distance. Apply dispatches `ptnd.action.object.offset_path` with the typed
/// `distance`, which lands as one undo entry holding live `ContourOffset`
/// modifiers; the hint shows the live distance so the prompt never invents
/// a value, and `0` clears the offset.
#[derive(Clone, PartialEq)]
pub struct OffsetPathDialog(pub UiShell);

impl Component for OffsetPathDialog {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let value = use_state(String::new);
        let error = use_state(|| None::<String>);

        if !(*ui.offset_prompt_open.read()) {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let mut prompt_open = ui.offset_prompt_open;
        let mut shell = ui.shell;
        let mut value_state = value;
        let mut error_state = error;

        let current: Option<f64> = {
            let guard = shell.peek();
            guard
                .bridge
                .selection()
                .selected_ids
                .first()
                .copied()
                .and_then(|id| guard.bridge.session()?.find_object(id))
                .and_then(|object| {
                    object.modifiers.iter().find_map(|m| {
                        if let petunia_design_document::ModifierKind::ContourOffset {
                            distance,
                            ..
                        } = m.kind
                        {
                            Some(distance)
                        } else {
                            None
                        }
                    })
                })
        };
        let hint = match current {
            Some(distance) => {
                format!("Atual / Current: {distance:.2} pt (0 remove / clears)")
            }
            None => "Sem deslocamento / No offset yet".to_string(),
        };
        let error_text = error.read().clone();

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
                        rect()
                            .direction(Direction::Horizontal)
                            .width(Size::fill())
                            .main_align(Alignment::SpaceBetween)
                            .cross_align(Alignment::Center)
                            .child(
                                label()
                                    .text("Deslocar caminho / Offset Path")
                                    .font_size(14.)
                                    .color(theme::TEXT_PRIMARY),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        prompt_open.set(false);
                                    })
                                    .child(label().text("✕").font_size(12.)),
                            ),
                    )
                    .child(
                        label()
                            .text("DISTÂNCIA / DISTANCE")
                            .font_size(10.)
                            .color(theme::TEXT_TERTIARY),
                    )
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .width(Size::fill())
                            .spacing(theme::SPACE_1)
                            .cross_align(Alignment::Center)
                            .child(
                                rect()
                                    .width(Size::flex(1.0))
                                    .child(Input::new(value).placeholder("ex. 6")),
                            )
                            .child(
                                label()
                                    .text("pt")
                                    .font_size(12.)
                                    .color(theme::TEXT_SECONDARY),
                            ),
                    )
                    .child(
                        label()
                            .text(hint)
                            .font_size(11.)
                            .color(theme::TEXT_SECONDARY),
                    )
                    .child(if let Some(message) = error_text {
                        rect().child(label().text(message).font_size(11.).color(PROMPT_ERROR))
                    } else {
                        rect()
                    })
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .width(Size::fill())
                            .main_align(Alignment::End)
                            .spacing(theme::SPACE_1)
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        error_state.set(None);
                                        value_state.set(String::new());
                                        prompt_open.set(false);
                                    })
                                    .child(label().text("Cancelar").font_size(11.)),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        let typed = value_state.peek().clone();
                                        match parse_prompt_distance(&typed) {
                                            Err(message) => {
                                                error_state.set(Some(message.to_string()));
                                            }
                                            Ok(distance) => {
                                                let res = shell.write().bridge.dispatch_action(
                                                    ActionRequest::new(
                                                        ActionId::new(
                                                            "ptnd.action.object.offset_path",
                                                        ),
                                                        serde_json::json!({
                                                            "distance": distance,
                                                        }),
                                                    ),
                                                );
                                                match res {
                                                    Ok(_) => {
                                                        error_state.set(None);
                                                        value_state.set(String::new());
                                                        prompt_open.set(false);
                                                    }
                                                    Err(err) => {
                                                        error_state.set(Some(err.to_string()));
                                                    }
                                                }
                                            }
                                        }
                                    })
                                    .child(label().text("Aplicar / Apply").font_size(11.)),
                            ),
                    ),
            )
    }
}

/// Overwrite Conflict Dialog (`ptnd.dialog.overwrite_conflict`).
///
/// Prompts the user before destructive overwrite of an existing file (Figma/Photoshop convention).
#[derive(Clone, PartialEq)]
pub struct OverwriteConflictDialog(pub UiShell);

impl Component for OverwriteConflictDialog {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        if !(*ui.overwrite_conflict_open.read()) {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let mut overwrite_open = ui.overwrite_conflict_open;
        let conflict_path = (*ui.overwrite_conflict_path.read()).clone();

        rect()
            .position(Position::new_absolute().top(140.))
            .width(Size::fill())
            .cross_align(Alignment::Center)
            .main_align(Alignment::Center)
            .child(
                rect()
                    .direction(Direction::Vertical)
                    .width(Size::px(420.))
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
                                    .text("O arquivo já existe / File already exists")
                                    .font_size(13.)
                                    .color(theme::TEXT_PRIMARY),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        overwrite_open.set(false);
                                    })
                                    .child(label().text("✕").font_size(11.)),
                            ),
                    )
                    .child(
                        label()
                            .text(format!(
                                "Um arquivo chamado \"{}\" já existe neste local. Deseja substituí-lo?",
                                conflict_path
                            ))
                            .font_size(11.)
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
                                        overwrite_open.set(false);
                                    })
                                    .child(label().text("Cancelar").font_size(11.)),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        overwrite_open.set(false);
                                    })
                                    .child(label().text("Substituir / Overwrite").font_size(11.)),
                            ),
                    ),
            )
    }
}

/// Places one original through the action/command lane. Failures remain visible
/// in the dialog; no placeholder or undo entry is created for rejected sources.
#[derive(Clone, PartialEq)]
pub struct PlaceImageDialog(pub UiShell);

impl Component for PlaceImageDialog {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let path = use_state(String::new);
        let mut picker_busy = use_state(|| false);
        let error = use_state(|| None::<String>);
        if !*ui.place_image_open.read() {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let mut open = ui.place_image_open;
        let mut close_open = open;
        let place_ui = ui.clone();
        let browse_ui = ui.clone();
        let busy = ui.file_job.read().is_some() || *picker_busy.read();
        let mut path_state = path;
        let mut error_state = error;
        let mut close_error = error;
        let error_text = error.read().clone();
        let failure_label = localized_dialog_text(ui, "ptnd.text.image.place_failed");
        rect()
            .position(Position::new_absolute().top(100.))
            .width(Size::fill())
            .cross_align(Alignment::Center)
            .child(
                Popup::new()
                    .on_close_request(move |_| {
                        if busy {
                            return;
                        }
                        close_error.set(None);
                        close_open.set(false);
                    })
                    .child(PopupTitle::new(localized_dialog_text(
                        ui,
                        "ptnd.text.file.place",
                    )))
                    .child(
                        PopupContent::new().child(
                            rect()
                                .direction(Direction::Vertical)
                                .width(Size::px(480.))
                                .spacing(theme::SPACE_2)
                                .child(
                                    label()
                                        .text(localized_dialog_text(ui, "ptnd.text.image.formats"))
                                        .color(theme::TEXT_SECONDARY),
                                )
                                .child(Input::new(path).placeholder(localized_dialog_text(
                                    ui,
                                    "ptnd.text.image.source_path",
                                )))
                                .child(
                                    Button::new()
                                        .enabled(!busy)
                                        .on_press(move |_| {
                                            picker_busy.set(true);
                                            let browse_ui = browse_ui.clone();
                                            spawn(async move {
                                                let file = rfd::AsyncFileDialog::new()
                                                    .add_filter(
                                                        "Images",
                                                        &[
                                                            "png", "jpg", "jpeg", "tif", "tiff",
                                                            "webp",
                                                        ],
                                                    )
                                                    .pick_file()
                                                    .await;
                                                picker_busy.set(false);
                                                if *browse_ui.place_image_open.peek() {
                                                    if let Some(file) = file {
                                                        path_state.set(
                                                            file.path()
                                                                .to_string_lossy()
                                                                .into_owned(),
                                                        );
                                                        error_state.set(None);
                                                    }
                                                }
                                            });
                                        })
                                        .child(ui.text("browse")),
                                )
                                .children(
                                    error_text
                                        .into_iter()
                                        .map(|message| label().text(message).color(PROMPT_ERROR)),
                                )
                                .child(
                                    rect()
                                        .direction(Direction::Horizontal)
                                        .main_align(Alignment::End)
                                        .spacing(theme::SPACE_1)
                                        .child(
                                            Button::new()
                                                .on_press(move |_| {
                                                    if busy {
                                                        return;
                                                    }
                                                    error_state.set(None);
                                                    open.set(false);
                                                })
                                                .child(localized_dialog_text(
                                                    ui,
                                                    "ptnd.text.image.cancel",
                                                )),
                                        )
                                        .child(
                                            Button::new()
                                                .enabled(!busy)
                                                .on_press(move |_| {
                                                    let chosen =
                                                        path_state.read().trim().to_string();
                                                    let result = crate::file_jobs::place(
                                                        &place_ui,
                                                        std::path::Path::new(&chosen),
                                                    );
                                                    match result {
                                                        Ok(_) => {
                                                            path_state.set(String::new());
                                                            error_state.set(None);
                                                        }
                                                        Err(reason) => error_state.set(Some(
                                                            format!("{failure_label}: {reason}"),
                                                        )),
                                                    }
                                                })
                                                .child(localized_dialog_text(
                                                    ui,
                                                    "ptnd.text.image.place",
                                                )),
                                        ),
                                ),
                        ),
                    ),
            )
    }
}

#[cfg(test)]
mod prompt_tests {
    use super::parse_prompt_distance;

    #[test]
    fn prompt_parses_plain_and_signed_values() {
        assert_eq!(parse_prompt_distance("6").unwrap(), 6.0);
        assert_eq!(parse_prompt_distance("  -2.5 ").unwrap(), -2.5);
        assert_eq!(parse_prompt_distance("3,5").unwrap(), 3.5);
        assert_eq!(parse_prompt_distance("0").unwrap(), 0.0);
    }

    #[test]
    fn prompt_rejects_empty_non_numeric_and_non_finite() {
        assert!(parse_prompt_distance("").is_err());
        assert!(parse_prompt_distance("   ").is_err());
        assert!(parse_prompt_distance("longe").is_err());
        assert!(parse_prompt_distance("NaN").is_err());
        assert!(parse_prompt_distance("inf").is_err());
        assert!(parse_prompt_distance("1e999").is_err());
    }
}
