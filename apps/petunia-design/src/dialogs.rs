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
        if !*ui.palette_open.read() {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let query = ui.palette_query.read().trim().to_lowercase();
        let filtered = items
            .read()
            .iter()
            .filter(|item| {
                query.is_empty()
                    || item.label.to_lowercase().contains(&query)
                    || item.action_id.to_lowercase().contains(&query)
            })
            .take(48)
            .cloned()
            .collect::<Vec<_>>();
        let empty = filtered.is_empty();
        let mut open = ui.palette_open;
        rect().child(
            Popup::new()
                .width(Size::px(560.))
                .max_width(Size::window_percent(96.))
                .on_close_request(move |_| open.set(false))
                .child(PopupTitle::new(localized_dialog_text(
                    ui,
                    "ptnd.text.shell.palette",
                )))
                .child(
                    PopupContent::new().child(
                        rect()
                            .direction(Direction::Vertical)
                            .width(Size::fill())
                            .spacing(theme::SPACE_2)
                            .child(
                                Input::new(ui.palette_query)
                                    .width(Size::fill())
                                    .auto_focus(true)
                                    .placeholder(ui.studio_text("search_commands")),
                            )
                            .child(
                                ScrollView::new()
                                    .width(Size::fill())
                                    .height(Size::window_percent(55.))
                                    .child(
                                        rect()
                                            .direction(Direction::Vertical)
                                            .width(Size::fill())
                                            .spacing(theme::SPACE_1)
                                            .maybe_child(empty.then(|| {
                                                label()
                                                    .text(ui.studio_text("no_commands"))
                                                    .color(theme::TEXT_SECONDARY)
                                            }))
                                            .children(filtered.iter().map(|item| {
                                                palette_row(ui.clone(), item, &mut open)
                                            })),
                                    ),
                            ),
                    ),
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
    let blocked = (!item.enabled).then(|| item.disabled_reason.clone());
    rect()
        .direction(Direction::Vertical)
        .width(Size::fill())
        .child(
            Button::new()
                .enabled(item.enabled)
                .expanded()
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
        .maybe_child(blocked.map(|reason| {
            label()
                .text(reason)
                .color(theme::TEXT_TERTIARY)
                .font_size(theme::CAPTION_SIZE)
        }))
}

///// Preferences and toolbar customization dialog.
#[derive(Clone, PartialEq)]
pub struct CustomizeDialog(pub UiShell);

impl Component for CustomizeDialog {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let mut tab = use_state(|| 0usize);
        let active = *tab.read();
        if !*ui.customize_open.read() {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let mut close = ui.customize_open;
        let catalog = ui.shell.read().bridge.query_toolbar_catalog();
        let mut reset = ui.shell;
        let reset_title = localized_dialog_text(ui, "ptnd.text.shell.reset_toolbar");
        let divider = localized_dialog_text(ui, "ptnd.text.shell.divider");
        rect().child(
            Popup::new()
                .width(Size::px(640.))
                .max_width(Size::window_percent(96.))
                .on_close_request(move |_| close.set(false))
                .child(PopupTitle::new(ui.studio_text("preferences")))
                .child(
                    PopupContent::new().child(
                        rect()
                            .direction(Direction::Vertical)
                            .width(Size::fill())
                            .spacing(theme::SPACE_3)
                            .child(
                                rect()
                                    .direction(Direction::Horizontal)
                                    .content(Content::Flex)
                                    .width(Size::fill())
                                    .spacing(theme::SPACE_1)
                                    .children(
                                        [
                                            "preferences_tools",
                                            "preferences_general",
                                            "preferences_performance",
                                            "preferences_shortcuts",
                                        ]
                                        .into_iter()
                                        .enumerate()
                                        .map(
                                            |(index, key)| {
                                                crate::studio_widgets::StudioButton::new(
                                                    ui,
                                                    ui.studio_text(key),
                                                )
                                                .text(ui.studio_text(key))
                                                .tab()
                                                .width(Size::flex(1.))
                                                .selected(index == active)
                                                .on_press(move |_| tab.set(index))
                                            },
                                        ),
                                    ),
                            )
                            .child(
                                ScrollView::new()
                                    .width(Size::fill())
                                    .height(Size::window_percent(62.))
                                    .child(match active {
                                        1 => preferences_general_tab(ui.clone()).into_element(),
                                        2 => preferences_performance_tab(ui.clone()).into_element(),
                                        3 => preferences_shortcuts_tab(ui.clone()).into_element(),
                                        _ => rect()
                                            .direction(Direction::Vertical)
                                            .width(Size::fill())
                                            .spacing(theme::SPACE_2)
                                            .children(catalog.iter().enumerate().map(
                                                |(index, row)| catalog_row(ui.clone(), index, row),
                                            ))
                                            .child(
                                                rect()
                                                    .direction(Direction::Horizontal)
                                                    .content(Content::Flex)
                                                    .spacing(theme::SPACE_2)
                                                    .child(divider_adder(ui.clone(), divider))
                                                    .child(
                                                        Button::new()
                                                            .on_press(move |_| {
                                                                reset.write().bridge.toolbar_reset()
                                                            })
                                                            .child(reset_title),
                                                    ),
                                            )
                                            .child(tool_rail_settings(ui.clone()))
                                            .into_element(),
                                    }),
                            ),
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
                .text(ui.studio_text("language"))
                .font_size(theme::CAPTION_SIZE)
                .color(theme::TEXT_TERTIARY),
        )
        .child(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
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
                .text(ui.studio_text("accent"))
                .font_size(theme::CAPTION_SIZE)
                .color(theme::TEXT_TERTIARY),
        )
        .child(crate::appearance::AppearanceBar(ui.clone()))
        .child(
            Button::new()
                .on_press({
                    let reset_ui = ui.clone();
                    move |_| {
                        let mut rail = reset_ui.tool_rail;
                        rail.set(crate::ui_state::default_tool_rail());
                        let mut right = reset_ui.right_studio_open;
                        right.set(true);
                        let mut width = reset_ui.dock_width;
                        width.set(320.);
                        let mut upper = reset_ui.studio_upper_open;
                        upper.set(true);
                        let mut dock = reset_ui.dock_tab;
                        dock.set(0);
                        let mut left = reset_ui.left_dock_open;
                        left.set(false);
                        let mut bottom = reset_ui.bottom_dock_open;
                        bottom.set(false);
                    }
                })
                .child(ui.studio_text("reset_workspace")),
        )
        .child(
            label()
                .text(ui.studio_text("history_file"))
                .font_size(theme::CAPTION_SIZE)
                .color(theme::TEXT_TERTIARY),
        )
        .child(
            label()
                .text(ui.studio_text("history_policy"))
                .font_size(11.)
                .color(theme::TEXT_SECONDARY),
        )
}

fn preferences_performance_tab(ui: UiShell) -> impl IntoElement {
    rect()
        .direction(Direction::Vertical)
        .width(Size::fill())
        .spacing(theme::SPACE_2)
        .child(
            label()
                .text(ui.studio_text("render_engine"))
                .font_size(theme::CAPTION_SIZE)
                .color(theme::TEXT_TERTIARY),
        )
        .child(
            label()
                .text(ui.studio_text("render_description"))
                .font_size(11.)
                .color(theme::TEXT_PRIMARY),
        )
        .child(
            label()
                .text(ui.studio_text("preview_quality"))
                .font_size(theme::CAPTION_SIZE)
                .color(theme::TEXT_TERTIARY),
        )
        .child(
            label()
                .text(ui.studio_text("preview_description"))
                .font_size(11.)
                .color(theme::TEXT_SECONDARY),
        )
        .child(
            label()
                .text(ui.studio_text("antialias"))
                .font_size(theme::CAPTION_SIZE)
                .color(theme::TEXT_TERTIARY),
        )
        .child(
            label()
                .text(ui.studio_text("antialias_description"))
                .font_size(11.)
                .color(theme::TEXT_SECONDARY),
        )
}

fn preferences_shortcuts_tab(ui: UiShell) -> impl IntoElement {
    let shortcuts = [
        ("V", "ptnd.text.tool.select"),
        ("A", "ptnd.text.tool.node"),
        ("P", "ptnd.text.tool.pen"),
        ("N", "ptnd.text.tool.pencil"),
        ("M", "ptnd.text.tool.rectangle"),
        ("T", "ptnd.text.tool.artistic_text"),
        ("G", "ptnd.text.tool.gradient"),
        ("Z", "ptnd.text.tool.zoom"),
        ("H", "ptnd.text.tool.hand"),
        ("B", "ptnd.text.tool.brush"),
        ("E", "ptnd.text.tool.eraser"),
    ];
    rect()
        .direction(Direction::Vertical)
        .width(Size::fill())
        .spacing(theme::SPACE_2)
        .children(shortcuts.into_iter().map(|(key, id)| {
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .child(
                    label()
                        .text(localized_dialog_text(&ui, id))
                        .color(theme::TEXT_SECONDARY),
                )
                .child(label().text(key).color(theme::TEXT_PRIMARY))
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
        .content(Content::Flex)
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
                .content(Content::Flex)
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
                .content(Content::Flex)
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
        .content(Content::Flex)
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
            ToolKind::PixelPaintBrush,
            ToolKind::PixelEraser,
            ToolKind::PixelFill,
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
    let shell = ui.shell.read();
    let bridge = &shell.bridge;
    bridge.localization().text(text_id, bridge.locale())
}

/// Dialog to configure and create a new document with dimension presets.
#[derive(Clone, PartialEq)]
pub struct NewDocumentDialog(pub UiShell);

impl Component for NewDocumentDialog {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let name = use_state(|| ui.studio_text("untitled"));
        let mut width = use_state(|| "1920".to_string());
        let mut height = use_state(|| "1080".to_string());
        let bleed = use_state(|| "0".to_string());
        let margin = use_state(|| "0".to_string());
        let mut error = use_state(|| None::<String>);
        let mut focus_pending = use_state(|| false);
        let observed = ui.new_doc_open;
        use_side_effect(move || {
            let opening = *observed.read();
            error.set(None);
            focus_pending.set(opening);
        });
        if !*ui.new_doc_open.read() {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let focused_role = Platform::get().focused_accessibility_node.read().role();
        if *focus_pending.read()
            && matches!(
                focused_role,
                AccessibilityRole::TextInput | AccessibilityRole::MultilineTextInput
            )
        {
            focus_pending.set(false);
        }
        let initial_focus = *focus_pending.read();
        let mut open = ui.new_doc_open;
        let mut shell = ui.shell;
        let submit_ui = ui.clone();
        let labels = [
            ui.studio_text("width"),
            ui.studio_text("height"),
            ui.text("bleed"),
            ui.text("margins"),
        ];
        let fields = [width, height, bleed, margin];
        let current_error = error.read().clone();
        rect().child(Popup::new().width(Size::px(600.)).max_width(Size::window_percent(96.))
            .on_close_request(move |_|open.set(false)).child(PopupTitle::new(ui.text("new")))
            .child(PopupContent::new().child(rect().direction(Direction::Vertical).width(Size::fill()).spacing(theme::SPACE_3)
                .child(label().text(ui.text("name")).color(theme::TEXT_SECONDARY))
                .child(Input::new(name).width(Size::fill()).auto_focus(initial_focus).placeholder(ui.text("name")).on_validate(move |_|error.set(None)))
                .child(label().text(ui.text("presets")).color(theme::TEXT_SECONDARY))
                .child(rect().direction(Direction::Horizontal).content(Content::Flex).width(Size::fill()).spacing(theme::SPACE_1)
                    .children([("Web 1080p".to_string(),1920.,1080.),("4K UHD".to_string(),3840.,2160.),
                        (ui.text("square"),1080.,1080.),(ui.text("mobile"),390.,844.),("A4".to_string(),595.276,841.89)]
                        .into_iter().map(|(caption,w,h)|Button::new().compact().on_press(move |_|{width.set(w.to_string());height.set(h.to_string());error.set(None);}).child(caption))))
                .child(label().text(ui.studio_text("point_units")).color(theme::TEXT_TERTIARY).font_size(theme::CAPTION_SIZE))
                .children(fields.into_iter().zip(labels).map(|(state,caption)|rect().direction(Direction::Horizontal).content(Content::Flex).width(Size::fill()).spacing(theme::SPACE_2)
                    .cross_align(Alignment::Center).child(label().text(caption.clone()).width(Size::px(110.)).color(theme::TEXT_SECONDARY))
                    .child(Input::new(state).width(Size::flex(1.)).placeholder(caption).on_validate(move |_|error.set(None)))))
                .child(Button::new().on_press(move |_|{let w=width.peek().clone();let h=height.peek().clone();width.set(h);height.set(w);}).child(ui.text("rotate_orientation")))
                .child(label().text(ui.text("color_mode")).color(theme::TEXT_SECONDARY))
                .child(label().text("RGB").color(theme::TEXT_PRIMARY))
                .child(label().text(ui.text("color_unavailable")).color(theme::TEXT_TERTIARY).font_size(theme::CAPTION_SIZE))
                .children(current_error.into_iter().map(|reason|label().text(reason).color(theme::TEXT_ERROR)))
                .child(rect().direction(Direction::Horizontal).content(Content::Flex).width(Size::fill()).spacing(theme::SPACE_2).main_align(Alignment::End)
                    .child(Button::new().on_press(move |_|open.set(false)).child(ui.text("cancel")))
                    .child(Button::new().filled().on_press(move |_|{
                        if name.peek().trim().is_empty(){error.set(Some(submit_ui.studio_text("name_required")));return;}
                        let parse=|state:State<String>,key:&str,positive:bool|parse_new_dimension(&state.peek(),positive)
                            .map_err(|reason|format!("{}: {}",submit_ui.studio_text(key),if *submit_ui.shell.peek().bridge.locale()==petunia_design_shell::Locale::PtBr {
                                reason.message_pt_br()}else {reason.message_en_us()}));
                        let values=(||Ok::<_,String>([parse(width,"width",true)?,parse(height,"height",true)?,parse(bleed,"bleed",false)?,parse(margin,"margins",false)?]))();
                        let values=match values {Ok(v)=>v,Err(reason)=>{error.set(Some(reason));return;}};
                        match crate::file_workflows::create_configured_document(&mut shell.write(),&name.peek(),[values[0],values[1]],values[2],values[3]){
                            Ok(())=>{error.set(None);open.set(false);},Err(reason)=>error.set(Some(reason.to_string())),
                        }
                    }).child(ui.text("create")))))))
    }
}

fn parse_new_dimension(
    raw: &str,
    positive: bool,
) -> Result<f64, petunia_design_foundation::NumericParseError> {
    use petunia_design_foundation::{parse_numeric_input, NumericFieldKind, NumericParseError};
    let raw = raw.trim();
    let lower = raw.to_ascii_lowercase();
    let number = if lower.ends_with("pt") {
        raw[..raw.len() - 2].trim()
    } else {
        raw
    };
    if !number.is_empty() && number.replace(',', ".").parse::<f64>().is_err() {
        return Err(NumericParseError::NotANumber);
    }
    parse_numeric_input(
        number,
        if positive {
            NumericFieldKind::PositiveDimensionPt
        } else {
            NumericFieldKind::NonNegativeDimensionPt
        },
    )
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
                    Button::new().enabled(!busy).on_press(move |_| {
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
                        Button::new().enabled(!busy).on_press(move |_| {dpi.set(value); replace.set(None);}).child(format!("{}{value} DPI",if density==value {"✓ "} else {""}))
                    })))
                    .maybe_child(surface_info.map(|info| label().text(info)))
                    .child(label().text(ui.text("alpha")).color(theme::TEXT_SECONDARY))))
                .maybe_child((active_format=="pdf").then(||label().text(ui.text("pdf_limit")).color(theme::TEXT_SECONDARY)))
                .maybe_child((active_format=="png").then(||crate::export_preview::ExportPreview(ui.clone())))
                .child(label().text(ui.text("destination")))
                .child(Input::new(path).width(Size::fill()).placeholder(ui.text("destination")))
                .maybe_child(destination.as_ref().map(|path| label().text(path.display().to_string()).color(theme::TEXT_SECONDARY)))
                .children(error_text.into_iter().map(|message| label().text(message).color(PROMPT_ERROR)))
                .maybe_child(confirmed.then(|| label().text(ui.text("overwrite_reason")).color(theme::TEXT_PRIMARY)))
                .child(rect().direction(Direction::Horizontal).main_align(Alignment::End).spacing(theme::SPACE_1)
                    .child(Button::new().on_press(move |_| {crate::file_jobs::cancel_export(&cancel_ui);open.set(false);error.set(None);replace.set(None);}).child(ui.text("cancel")))
                    .child(Button::new().filled().enabled(!busy).on_press(move |_| {
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
                                    .content(Content::Flex)
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
        let mut value = use_state(String::new);
        let mut error = use_state(|| None::<String>);
        let mut open = ui.offset_prompt_open;
        let mut shell = ui.shell;
        if !*open.read() {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let current = {
            let state = shell.read();
            state
                .bridge
                .selection()
                .selected_ids
                .first()
                .and_then(|id| state.bridge.session()?.find_object(*id))
                .and_then(|object| {
                    object.modifiers.iter().find_map(|item| match item.kind {
                        petunia_design_document::ModifierKind::ContourOffset {
                            distance, ..
                        } => Some(distance),
                        _ => None,
                    })
                })
        };
        let hint = current.map_or_else(
            || ui.studio_text("no_offset"),
            |distance| {
                format!(
                    "{}: {distance:.2} pt · {}",
                    ui.studio_text("offset_current"),
                    ui.studio_text("offset_zero")
                )
            },
        );
        let error_text = error.read().clone();
        rect().child(
            Popup::new()
                .width(Size::px(440.))
                .max_width(Size::window_percent(96.))
                .on_close_request(move |_| {
                    error.set(None);
                    value.set(String::new());
                    open.set(false);
                })
                .child(PopupTitle::new(ui.studio_text("offset_path")))
                .child(
                    PopupContent::new().child(
                        rect()
                            .direction(Direction::Vertical)
                            .width(Size::fill())
                            .spacing(theme::SPACE_2)
                            .child(
                                label()
                                    .text(ui.studio_text("distance"))
                                    .color(theme::TEXT_SECONDARY),
                            )
                            .child(
                                Input::new(value)
                                    .width(Size::fill())
                                    .auto_focus(true)
                                    .placeholder("pt"),
                            )
                            .child(label().text(hint).color(theme::TEXT_SECONDARY))
                            .children(
                                error_text
                                    .into_iter()
                                    .map(|reason| label().text(reason).color(theme::TEXT_ERROR)),
                            )
                            .child(
                                rect()
                                    .direction(Direction::Horizontal)
                                    .content(Content::Flex)
                                    .width(Size::fill())
                                    .main_align(Alignment::End)
                                    .spacing(theme::SPACE_2)
                                    .child(
                                        Button::new()
                                            .on_press(move |_| {
                                                error.set(None);
                                                value.set(String::new());
                                                open.set(false);
                                            })
                                            .child(ui.studio_text("cancel")),
                                    )
                                    .child(
                                        Button::new()
                                            .filled()
                                            .on_press(move |_| {
                                                let distance =
                                                    match parse_prompt_distance(&value.peek()) {
                                                        Ok(v) => v,
                                                        Err(reason) => {
                                                            error.set(Some(reason.to_string()));
                                                            return;
                                                        }
                                                    };
                                                let result = shell.write().bridge.dispatch_action(
                                                    ActionRequest::new(
                                                        ActionId::new(
                                                            "ptnd.action.object.offset_path",
                                                        ),
                                                        serde_json::json!({"distance":distance}),
                                                    ),
                                                );
                                                match result {
                                                    Ok(_) => {
                                                        error.set(None);
                                                        value.set(String::new());
                                                        open.set(false);
                                                    }
                                                    Err(reason) => {
                                                        error.set(Some(reason.to_string()))
                                                    }
                                                }
                                            })
                                            .child(ui.studio_text("apply")),
                                    ),
                            ),
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
                            .content(Content::Flex)
                            .width(Size::fill())
                            .main_align(Alignment::SpaceBetween)
                            .cross_align(Alignment::Center)
                            .child(
                                label()
                                    .text(ui.studio_text("file_exists"))
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
                            .content(Content::Flex)
                            .width(Size::fill())
                            .main_align(Alignment::End)
                            .spacing(theme::SPACE_1)
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        overwrite_open.set(false);
                                    })
                                    .child(label().text(ui.studio_text("cancel")).font_size(11.)),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        overwrite_open.set(false);
                                    })
                                    .child(label().text(ui.studio_text("overwrite")).font_size(11.)),
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
            .width(Size::fill())
            .cross_align(Alignment::Center)
            .child(
                Popup::new()
                    .width(Size::px(520.))
                    .max_width(Size::window_percent(96.))
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
                                .width(Size::fill())
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
                                        .content(Content::Flex)
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
    fn new_dimensions_accept_points_and_commas_but_reject_wrong_units_and_ranges() {
        use super::parse_new_dimension;
        assert_eq!(parse_new_dimension("612,5 pt", true).unwrap(), 612.5);
        assert_eq!(parse_new_dimension("0", false).unwrap(), 0.);
        for bad in ["", "0", "-1", "NaN", "inf", "1e999", "12 mm", "12 px"] {
            assert!(parse_new_dimension(bad, true).is_err(), "{bad}");
        }
    }
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
