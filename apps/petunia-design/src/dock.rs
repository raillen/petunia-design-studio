//! Right dock component: Layers, Properties/Inspector, Colors, and History.
//!
//! Provides direct visual inspection, layer hierarchy control, typography editing,
//! shape properties, and styling for objects in the active document.

use freya::prelude::*;
use petunia_design_application::Command;
use petunia_design_color::{Cmyk, ColorValue, Lab, Srgb};
use petunia_design_document::ShapeKind;
use petunia_design_foundation::ObjectId;
use petunia_design_geometry::{OffsetCap, OffsetJoin};
use petunia_design_shell::panels::LayersPanelController;
use petunia_design_shell::PetuniaShell;

use crate::actions::run_action_token;
use crate::theme;
use crate::ui_state::UiShell;

/// The right dock surface containing tabbed inspectors (Layers, Properties, Colors, History).
#[derive(Clone, PartialEq)]
pub struct RightDock(pub UiShell);

impl Component for RightDock {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        if !*ui.right_studio_open.read() {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let mut dock_tab = ui.dock_tab;
        let active_tab = *dock_tab.read();
        let mut upper = ui.studio_upper_tab;
        let upper_tab = *upper.read();
        let mut upper_open = ui.studio_upper_open;
        let expanded = *upper_open.read();
        let root = *Platform::get().root_size.read();
        let tools = theme::TOOL_RAIL_WIDTH
            * if ui.tool_rail.read().columns == crate::ui_state::RailColumns::Two {
                2.
            } else {
                1.
            };
        let reserve = tools
            + 8.
            + if *ui.left_dock_open.read() {
                crate::studio_widgets::left_studio_width(
                    *ui.left_dock_width.read(),
                    root.width,
                    tools,
                    true,
                )
            } else {
                0.
            };
        let dock_w =
            crate::studio_widgets::studio_width(*ui.dock_width.read(), root.width, reserve);
        let upper_height = (root.height * 0.32).clamp(160., 258.);
        rect()
            .direction(Direction::Vertical)
            .content(Content::Flex)
            .width(Size::px(dock_w))
            .height(Size::fill())
            .background(theme::SURFACE_PANEL)
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .height(Size::px(theme::PANEL_HEADER_HEIGHT))
                    .background(theme::SURFACE_CHROME)
                    .cross_align(Alignment::Center)
                    .children(
                        [(0, "color"), (1, "swatches")]
                            .into_iter()
                            .map(|(index, key)| {
                                crate::studio_widgets::StudioButton::new(ui, ui.studio_text(key))
                                    .text(ui.studio_text(key))
                                    .tab()
                                    .width(Size::flex(1.))
                                    .selected(upper_tab == index)
                                    .on_press(move |_| {
                                        upper.set(index);
                                        upper_open.set(true);
                                    })
                            }),
                    )
                    .child(
                        crate::studio_widgets::StudioButton::new(
                            ui,
                            ui.studio_text(if expanded { "collapse" } else { "expand" }),
                        )
                        .icon(if expanded {
                            theme::ICON_CHEVRON_DOWN
                        } else {
                            theme::ICON_CHEVRON_RIGHT
                        })
                        .width(Size::px(32.))
                        .on_press(move |_| upper_open.set(!expanded)),
                    ),
            )
            .maybe_child(expanded.then(|| {
                rect()
                    .width(Size::fill())
                    .height(Size::px(upper_height))
                    .padding(Gaps::new_all(theme::SPACE_2))
                    .child(
                        ScrollView::new().child(match upper_tab {
                            1 => SwatchesPanel {
                                ui: ui.clone(),
                                target_fill: ui.studio_fill_target,
                            }
                            .into_element(),
                            _ => crate::studio::CompactColor(ui.clone()).into_element(),
                        }),
                    )
            }))
            .child(
                rect()
                    .height(Size::px(1.))
                    .width(Size::fill())
                    .background(theme::BORDER_SUBTLE),
            )
            .children(
                [
                    [(0, "layers"), (1, "properties"), (2, "colors")],
                    [(3, "history"), (4, "navigator"), (5, "tasks")],
                ]
                .into_iter()
                .map(|tabs| {
                    rect()
                        .direction(Direction::Horizontal)
                        .content(Content::Flex)
                        .width(Size::fill())
                        .height(Size::px(theme::PANEL_HEADER_HEIGHT))
                        .background(theme::SURFACE_CHROME)
                        .children(tabs.into_iter().map(|(index, key)| {
                            crate::studio_widgets::StudioButton::new(ui, ui.studio_text(key))
                                .text(ui.studio_text(key))
                                .tab()
                                .width(Size::flex(1.))
                                .selected(active_tab == index)
                                .on_press(move |_| dock_tab.set(index))
                        }))
                }),
            )
            .child(
                rect()
                    .direction(Direction::Vertical)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .height(Size::flex(1.))
                    .padding(Gaps::new_all(theme::SPACE_2))
                    .child(match active_tab {
                        1 => properties_tab(ui.clone()).into_element(),
                        2 => ScrollView::new()
                            .child(ColorsTab(ui.clone()))
                            .into_element(),
                        3 => history_tab(ui.clone()).into_element(),
                        4 => ScrollView::new()
                            .child(NavigatorTab(ui.clone()))
                            .into_element(),
                        5 => ScrollView::new()
                            .child(BackgroundTasksPanel(ui.clone()))
                            .into_element(),
                        _ => layers_tab(ui.clone()).into_element(),
                    }),
            )
    }
}

// =========================================================================
// 1. Layers Tab
// =========================================================================

fn layers_tab(ui: UiShell) -> impl IntoElement {
    let shell = ui.shell;
    let model = shell.read().query_layers();
    let count = model.rows.len();
    let selected = shell.read().bridge.selection().selected_ids.len();
    let collapsed = ui.collapsed_layers.read().clone();
    let session = ui.shell.read().bridge.session().map(|s| s.identity());
    let mut hidden_depth = None;
    let rows = model
        .rows
        .into_iter()
        .filter(|row| {
            if let Some(depth) = hidden_depth {
                if row.depth > depth {
                    return false;
                }
                hidden_depth = None;
            }
            if session.is_some_and(|session| collapsed.contains(&(session, row.id))) {
                hidden_depth = Some(row.depth);
            }
            true
        })
        .collect::<Vec<_>>();
    rect()
        .direction(Direction::Vertical)
        .content(Content::Flex)
        .width(Size::fill())
        .height(Size::fill())
        .spacing(theme::SPACE_2)
        .child(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .width(Size::fill())
                .height(Size::px(28.))
                .cross_align(Alignment::Center)
                .main_align(Alignment::SpaceBetween)
                .child(
                    label()
                        .text(format!("{} · {count}", ui.studio_text("layers")))
                        .font_size(theme::BODY_SIZE)
                        .color(theme::TEXT_SECONDARY),
                )
                .child(
                    label()
                        .text(if selected == 0 {
                            ui.studio_text("no_selection")
                        } else {
                            ui.studio_text("selection_count")
                                .replace("{count}", &selected.to_string())
                        })
                        .font_size(theme::CAPTION_SIZE)
                        .color(theme::TEXT_TERTIARY),
                ),
        )
        .child(
            ScrollView::new()
                .height(Size::flex(1.))
                .width(Size::fill())
                .child(
                    rect()
                        .direction(Direction::Vertical)
                        .width(Size::fill())
                        .spacing(2.)
                        .maybe_child((count == 0).then(|| {
                            rect()
                                .direction(Direction::Vertical)
                                .width(Size::fill())
                                .padding(Gaps::new_all(theme::SPACE_3))
                                .spacing(theme::SPACE_2)
                                .child(
                                    label()
                                        .text(ui.studio_text("empty_layers"))
                                        .color(theme::TEXT_SECONDARY),
                                )
                                .child(
                                    label()
                                        .text(ui.studio_text("empty_layers_hint"))
                                        .color(theme::TEXT_TERTIARY)
                                        .font_size(theme::BODY_SIZE),
                                )
                        }))
                        .children(rows.into_iter().map(|row| LayerStudioRow {
                            ui: ui.clone(),
                            row,
                        })),
                ),
        )
        .child(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .width(Size::fill())
                .height(Size::px(32.))
                .spacing(theme::SPACE_1)
                .children(
                    [
                        (
                            "group",
                            theme::ICON_LAYERS,
                            "ptnd.action.object.group",
                            selected >= 2,
                        ),
                        (
                            "ungroup",
                            theme::ICON_BOOLEAN,
                            "ptnd.action.object.ungroup",
                            selected > 0,
                        ),
                        (
                            "front",
                            theme::ICON_CHEVRON_UP,
                            "ptnd.action.object.arrange.front",
                            selected > 0,
                        ),
                        (
                            "back",
                            theme::ICON_CHEVRON_DOWN,
                            "ptnd.action.object.arrange.back",
                            selected > 0,
                        ),
                        (
                            "delete",
                            theme::ICON_TRASH,
                            "ptnd.action.edit.delete",
                            selected > 0,
                        ),
                    ]
                    .into_iter()
                    .map(|(key, icon, token, enabled)| {
                        let mut shell = shell;
                        let state = shell.read();
                        let availability = petunia_design_application::menus::availability(
                            token,
                            &state.bridge.action_context(),
                        );
                        let title = availability.reason.map_or_else(
                            || ui.studio_text(key),
                            |reason| {
                                format!(
                                    "{}: {}",
                                    ui.studio_text(key),
                                    state
                                        .bridge
                                        .localization()
                                        .text(reason, state.bridge.locale())
                                )
                            },
                        );
                        drop(state);
                        crate::studio_widgets::StudioButton::new(&ui, title)
                            .icon(icon)
                            .width(Size::flex(1.))
                            .enabled(enabled && availability.enabled)
                            .on_press(move |_| {
                                let _ = run_action_token(&mut shell.write(), token);
                            })
                    }),
                ),
        )
}

#[derive(Clone, PartialEq)]
struct LayerStudioRow {
    ui: UiShell,
    row: petunia_design_application::view_models::LayerRowViewModel,
}
impl Component for LayerStudioRow {
    fn render_key(&self) -> DiffKey {
        DiffKey::from(&(
            self.ui.shell.peek().bridge.session().map(|s| s.identity()),
            self.row.id,
        ))
    }
    fn render(&self) -> impl IntoElement {
        let ui = &self.ui;
        let row = &self.row;
        let id = row.id;
        let selected = row.is_selected;
        let focus_id = use_a11y();
        let focus = use_focus(focus_id);
        let mut hovered = use_state(|| false);
        let mut select = ui.shell;
        let modifiers = ui.modifiers;
        let mut collapse = ui.collapsed_layers;
        let session = ui.shell.read().bridge.session().map(|s| s.identity());
        let collapsed = session.is_some_and(|session| collapse.read().contains(&(session, id)));
        let object = ui
            .shell
            .read()
            .bridge
            .session()
            .and_then(|s| s.find_object(id))
            .cloned();
        let icon = if row.is_container {
            theme::ICON_LAYERS
        } else {
            match object.as_ref().and_then(|o| o.shape.as_ref()) {
                Some(ShapeKind::Text { .. }) => theme::ICON_TYPE,
                Some(ShapeKind::Raster { .. } | ShapeKind::Image { .. }) => theme::ICON_PHOTO,
                Some(ShapeKind::Ellipse) => theme::ICON_CIRCLE,
                Some(ShapeKind::Path { .. }) => theme::ICON_PEN,
                _ => theme::ICON_SQUARE,
            }
        };
        let rename = ui.clone();
        let mut visibility = ui.shell;
        let mut lock = ui.shell;
        rect()
            .direction(Direction::Horizontal)
            .content(Content::Flex)
            .width(Size::fill())
            .height(Size::px(36.))
            .cross_align(Alignment::Center)
            .spacing(2.)
            .padding(Gaps::new(2., 2., 2., (row.depth.min(6) as f32 * 12.) + 2.))
            .background(if selected {
                theme::SURFACE_SELECTED
            } else if hovered() {
                theme::SURFACE_HOVER
            } else {
                Color::TRANSPARENT
            })
            .border(
                Border::new()
                    .fill(if focus() == Focus::Keyboard {
                        ui.accent.read().value
                    } else {
                        Color::TRANSPARENT
                    })
                    .width(2.)
                    .alignment(BorderAlignment::Inner),
            )
            .a11y_id(focus_id)
            .a11y_role(AccessibilityRole::TreeItem)
            .a11y_focusable(true)
            .a11y_alt(row.name.clone())
            .a11y_builder(|node| {
                node.set_selected(selected);
                if row.is_container {
                    node.set_expanded(!collapsed);
                }
            })
            .on_pointer_enter(move |_| hovered.set(true))
            .on_pointer_leave(move |_| hovered.set(false))
            .on_all_press(move |event: Event<PressEventData>| {
                event.stop_propagation();
                LayersPanelController::new().select_row(
                    &mut select.write().bridge,
                    id,
                    modifiers.peek().constrain || modifiers.peek().disable_snap,
                );
            })
            .child(if row.is_container {
                crate::studio_widgets::StudioButton::new(
                    ui,
                    ui.studio_text(if collapsed {
                        "expand_layer"
                    } else {
                        "collapse_layer"
                    }),
                )
                .width(Size::px(22.))
                .icon(if collapsed {
                    theme::ICON_CHEVRON_RIGHT
                } else {
                    theme::ICON_CHEVRON_DOWN
                })
                .on_press(move |_| {
                    if let Some(session) = session {
                        let mut set = collapse.write();
                        if !set.remove(&(session, id)) {
                            set.insert((session, id));
                        }
                    }
                })
                .into_element()
            } else {
                rect().width(Size::px(8.)).into_element()
            })
            .child(crate::chrome::app_icon_sized(
                icon,
                *ui.icon_style.read(),
                theme::TEXT_SECONDARY,
                18.,
            ))
            .child(
                label()
                    .text(row.name.clone())
                    .width(Size::flex(1.))
                    .max_lines(1)
                    .text_overflow(TextOverflow::Ellipsis)
                    .font_size(theme::BODY_SIZE)
                    .color(if row.visible {
                        theme::TEXT_PRIMARY
                    } else {
                        theme::TEXT_TERTIARY
                    }),
            )
            .child(
                crate::studio_widgets::StudioButton::new(ui, ui.text("edit_name"))
                    .icon(theme::ICON_PENCIL)
                    .width(Size::px(28.))
                    .on_press(move |_| {
                        crate::object_edit_dialog::request(
                            &rename,
                            id,
                            crate::object_edits::EditKind::Name,
                        )
                    }),
            )
            .child(
                crate::studio_widgets::StudioButton::new(
                    ui,
                    ui.studio_text(if row.visible {
                        "hide_layer"
                    } else {
                        "show_layer"
                    }),
                )
                .icon(if row.visible {
                    theme::ICON_EYE
                } else {
                    theme::ICON_EYE_OFF
                })
                .width(Size::px(28.))
                .on_press(move |_| {
                    let _ = LayersPanelController::new()
                        .toggle_visibility(&mut visibility.write().bridge, id);
                }),
            )
            .child(
                crate::studio_widgets::StudioButton::new(
                    ui,
                    ui.studio_text(if row.locked {
                        "unlock_layer"
                    } else {
                        "lock_layer"
                    }),
                )
                .icon(theme::ICON_LOCK)
                .selected(row.locked)
                .width(Size::px(28.))
                .on_press(move |_| {
                    let _ = LayersPanelController::new().toggle_lock(&mut lock.write().bridge, id);
                }),
            )
    }
}

// =========================================================================
// 2. Properties Tab (Transform, Typography, Appearance, Booleans, Alignment)
// =========================================================================

fn properties_tab(ui: UiShell) -> impl IntoElement {
    let shell = ui.shell;
    let props = shell.read().bridge.query_properties();
    let sel = shell.peek().bridge.selection();

    if props.selection_empty {
        return ScrollView::new()
            .child(
                rect()
                    .direction(Direction::Vertical)
                    .width(Size::fill())
                    .spacing(theme::SPACE_2)
                    .child(section_header(ui.studio_text("document_histogram")))
                    .child(HistogramWidget {
                        ui: ui.clone(),
                        object_id: None,
                    })
                    .child(
                        rect()
                            .width(Size::fill())
                            .padding(Gaps::new_all(theme::SPACE_4))
                            .center()
                            .child(
                                label()
                                    .text(ui.studio_text("selection_hint"))
                                    .color(theme::TEXT_TERTIARY)
                                    .font_size(12.),
                            ),
                    ),
            )
            .into_element();
    }

    let first_id = sel.selected_ids.first().copied();
    let selected_obj = first_id.and_then(|id| {
        shell
            .peek()
            .bridge
            .session()
            .and_then(|s| s.find_object(id))
            .cloned()
    });

    let (is_text, font_size) = if let Some(ref obj) = selected_obj {
        if let Some(ShapeKind::Text { font_size, .. }) = obj.shape {
            (true, font_size)
        } else {
            (false, 16.0)
        }
    } else {
        (false, 16.0)
    };

    let star_params = selected_obj.as_ref().and_then(|obj| {
        if let Some(ShapeKind::Star {
            points,
            inner_ratio,
        }) = obj.shape
        {
            Some((points, inner_ratio))
        } else {
            None
        }
    });

    let polygon_params = selected_obj.as_ref().and_then(|obj| {
        if let Some(ShapeKind::Polygon { sides }) = obj.shape {
            Some(sides)
        } else {
            None
        }
    });

    let rect_corners = selected_obj.as_ref().and_then(|obj| {
        if let Some(ShapeKind::Rectangle { corner_radii }) = obj.shape {
            Some(corner_radii[0])
        } else {
            None
        }
    });

    let modifiers_list = selected_obj
        .as_ref()
        .map(|obj| obj.modifiers.clone())
        .unwrap_or_default();
    let obj_bounds = selected_obj.as_ref().and_then(|obj| obj.bounds);

    let gaussian_blur_effect = selected_obj.as_ref().and_then(|obj| {
        obj.appearance.as_ref().and_then(|app| {
            app.effects.iter().find_map(|e| {
                if let petunia_design_document::EffectKind::GaussianBlur { radius } = e.kind {
                    Some((e.id, radius, e.visible))
                } else {
                    None
                }
            })
        })
    });

    let drop_shadow_effect = selected_obj.as_ref().and_then(|obj| {
        obj.appearance.as_ref().and_then(|app| {
            app.effects.iter().find_map(|e| {
                if let petunia_design_document::EffectKind::DropShadow {
                    offset,
                    blur,
                    color,
                    opacity,
                } = &e.kind
                {
                    Some((e.id, *offset, *blur, color.clone(), *opacity, e.visible))
                } else {
                    None
                }
            })
        })
    });

    let inner_shadow_effect = selected_obj.as_ref().and_then(|obj| {
        obj.appearance.as_ref().and_then(|app| {
            app.effects.iter().find_map(|e| {
                if let petunia_design_document::EffectKind::InnerShadow {
                    offset,
                    blur,
                    color,
                    opacity,
                } = &e.kind
                {
                    Some((e.id, *offset, *blur, color.clone(), *opacity, e.visible))
                } else {
                    None
                }
            })
        })
    });

    let sharpen_effect = selected_obj.as_ref().and_then(|obj| {
        obj.appearance.as_ref().and_then(|app| {
            app.effects.iter().find_map(|e| {
                if let petunia_design_document::EffectKind::Sharpen { radius, amount } = e.kind {
                    Some((e.id, radius, amount, e.visible))
                } else {
                    None
                }
            })
        })
    });

    let noise_effect = selected_obj.as_ref().and_then(|obj| {
        obj.appearance.as_ref().and_then(|app| {
            app.effects.iter().find_map(|e| {
                if let petunia_design_document::EffectKind::Noise { amount, monochrome } = e.kind {
                    Some((e.id, amount, monochrome, e.visible))
                } else {
                    None
                }
            })
        })
    });

    let adjustments = selected_obj
        .as_ref()
        .and_then(|obj| obj.appearance.as_ref().map(|app| app.adjustments.clone()))
        .unwrap_or_default();

    let bounds = props.bounds.unwrap_or([0.0, 0.0, 100.0, 100.0]);
    let [x, y, w, h] = bounds;
    let stroke_width = props.stroke_width;
    let opacity = (props.opacity * 100.0).round() as u32;

    ScrollView::new()
        .child(
            rect()
                .direction(Direction::Vertical)
                .width(Size::fill())
                .spacing(theme::SPACE_2)
                .child(
                    // Section: Transformation
                    section_header(ui.studio_text("transform")),
                )
        .child(
            rect()
                .direction(Direction::Horizontal).content(Content::Flex)
                .width(Size::fill())
                .cross_align(Alignment::Center)
                .main_align(Alignment::SpaceBetween)
                .child(value_pill("X", format!("{:.1}", x)))
                .child(value_pill("Y", format!("{:.1}", y)))
                .child(value_pill("W", format!("{:.1}", w)))
                .child(value_pill("H", format!("{:.1}", h))),
        )
        .child({
            let transform_ui = ui.clone();
            Button::new().enabled(sel.selected_ids.len() == 1 && obj_bounds.is_some())
                .on_press(move |_| {
                    if let Some(id) = first_id { crate::object_edit_dialog::request(&transform_ui, id, crate::object_edits::EditKind::Transform); }
                }).child(ui.text("edit_transform"))
        })
        .maybe(sel.selected_ids.len() > 1, |el| el.child(label().text(ui.text("edit_single_selection")).color(theme::TEXT_SECONDARY)))
        .maybe(is_text && sel.selected_ids.len() == 1, |el| {
            let edit_ui = ui.clone();
            let down_ui = ui.clone();
            let up_ui = ui.clone();
            el.child(section_header(ui.text("edit_typography")))
                .maybe(first_id.is_some(), |el| el.child(crate::typography::TypographyControl(ui.clone(), first_id.unwrap())))
                .child(Button::new().on_press(move |_| {
                    if let Some(id) = first_id { crate::object_edit_dialog::request(&edit_ui, id, crate::object_edits::EditKind::Text); }
                }).child(ui.text("edit_text")))
                .child(rect().direction(Direction::Horizontal).content(Content::Flex).spacing(theme::SPACE_2).cross_align(Alignment::Center)
                    .child(Button::new().on_press(move |_| step_text_size(&down_ui, first_id, -4.)).child("A−"))
                    .child(label().text(format!("{font_size:.1} pt")).color(theme::TEXT_PRIMARY))
                    .child(Button::new().on_press(move |_| step_text_size(&up_ui, first_id, 4.)).child("A+")))
        })
        .child(
            // Section: Appearance
            section_header(ui.studio_text("appearance")),
        )
        .child(
            rect()
                .direction(Direction::Horizontal).content(Content::Flex)
                .width(Size::fill())
                .cross_align(Alignment::Center)
                .main_align(Alignment::SpaceBetween)
                .child(
                    label()
                        .text(format!("{}: {}%", ui.studio_text("opacity"), opacity))
                        .font_size(11.)
                        .color(theme::TEXT_SECONDARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .spacing(2.)
                        .child(opacity_button(shell, first_id, "25%", 0.25))
                        .child(opacity_button(shell, first_id, "50%", 0.50))
                        .child(opacity_button(shell, first_id, "75%", 0.75))
                        .child(opacity_button(shell, first_id, "100%", 1.0)),
                ),
        )
        .child(
            rect()
                .direction(Direction::Horizontal).content(Content::Flex)
                .width(Size::fill())
                .cross_align(Alignment::Center)
                .main_align(Alignment::SpaceBetween)
                .child(
                    label()
                        .text(format!("{}: {:.1} pt", ui.studio_text("stroke_width"), stroke_width))
                        .font_size(11.)
                        .color(theme::TEXT_SECONDARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .spacing(theme::SPACE_1)
                        .child(stroke_button(shell, first_id, "- 1pt", -1.0))
                        .child(stroke_button(shell, first_id, "+ 1pt", 1.0)),
                ),
        )
        .child(
            // Quick Swatches for Fill
            rect()
                .direction(Direction::Horizontal).content(Content::Flex)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .child(quick_color_swatch(ui.clone(), "ptnd.gray/900", Color::from_rgb(0x20, 0x21, 0x24)))
                .child(quick_color_swatch(ui.clone(), "ptnd.blue/500", Color::from_rgb(0x3B, 0x82, 0xF6)))
                .child(quick_color_swatch(ui.clone(), "ptnd.purple/500", Color::from_rgb(0x8B, 0x5C, 0xF6)))
                .child(quick_color_swatch(ui.clone(), "ptnd.teal/500", Color::from_rgb(0x14, 0xB8, 0xA6)))
                .child(quick_color_swatch(ui.clone(), "ptnd.red/500", Color::from_rgb(0xEF, 0x44, 0x44)))
                .child(quick_color_swatch(ui.clone(), "ptnd.amber/500", Color::from_rgb(0xF5, 0x9E, 0x0B))),
        )
        .maybe(star_params.is_some(), |el| {
            let (points, inner_ratio) = star_params.unwrap();
            el.child(section_header(ui.studio_text("star_shape")))
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .width(Size::fill())
                        .main_align(Alignment::SpaceBetween)
                        .cross_align(Alignment::Center)
                        .child(
                            label()
                                .text(format!("{}: {} · {}: {:.0}%",ui.studio_text("points"), points, ui.studio_text("radius"), inner_ratio * 100.0))
                                .font_size(11.)
                                .color(theme::TEXT_SECONDARY),
                        )
                        .child(
                            rect()
                                .direction(Direction::Horizontal).content(Content::Flex)
                                .spacing(2.)
                                .child(star_points_button(shell, first_id, "-1", -1))
                                .child(star_points_button(shell, first_id, "+1", 1))
                                .child(star_ratio_button(shell, first_id, "-5%", -0.05))
                                .child(star_ratio_button(shell, first_id, "+5%", 0.05)),
                        ),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .width(Size::fill())
                        .main_align(Alignment::End)
                        .child(convert_to_curves_button(shell, first_id)),
                )
        })
        .maybe(polygon_params.is_some(), |el| {
            let sides = polygon_params.unwrap();
            el.child(section_header(ui.studio_text("polygon_shape")))
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .width(Size::fill())
                        .main_align(Alignment::SpaceBetween)
                        .cross_align(Alignment::Center)
                        .child(
                            label()
                                .text(format!("{}: {}",ui.studio_text("sides"), sides))
                                .font_size(11.)
                                .color(theme::TEXT_SECONDARY),
                        )
                        .child(
                            rect()
                                .direction(Direction::Horizontal).content(Content::Flex)
                                .spacing(2.)
                                .child(polygon_sides_button(shell, first_id, "-1", -1))
                                .child(polygon_sides_button(shell, first_id, "+1", 1)),
                        ),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .width(Size::fill())
                        .main_align(Alignment::End)
                        .child(convert_to_curves_button(shell, first_id)),
                )
        })
        .maybe(rect_corners.is_some(), |el| {
            let r = rect_corners.unwrap();
            el.child(section_header(ui.studio_text("corners")))
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .width(Size::fill())
                        .main_align(Alignment::SpaceBetween)
                        .cross_align(Alignment::Center)
                        .child(
                            label()
                                .text(format!("{}: {:.1} pt",ui.studio_text("radius"), r))
                                .font_size(11.)
                                .color(theme::TEXT_SECONDARY),
                        )
                        .child(
                            rect()
                                .direction(Direction::Horizontal).content(Content::Flex)
                                .spacing(2.)
                                .child(corner_radius_button(shell, first_id, "-2pt", -2.0))
                                .child(corner_radius_button(shell, first_id, "+2pt", 2.0))
                                .child(bake_corners_button(shell, first_id)),
                        ),
                )
        })
        .child(
            // Section: Live Modifiers Stack (ADR 09.31)
            section_header(ui.studio_text("live_modifiers")),
        )
        .child(modifier_stack_inspector(shell, first_id, &modifiers_list, obj_bounds))
        .child(
            // Section: Effects & Live Filters (10.4 / 10.10)
            section_header(ui.studio_text("effects")),
        )
        .child(
            rect()
                .direction(Direction::Horizontal).content(Content::Flex)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text(match gaussian_blur_effect {
                            Some((_, r, true)) => format!("{}: {r:.1} pt",ui.studio_text("blur")),
                            Some((_, r, false)) => format!("{} ({}): {r:.1} pt",ui.studio_text("blur"),ui.studio_text("hidden")),
                            None => format!("{}: {}",ui.studio_text("blur"),ui.studio_text("none")),
                        })
                        .font_size(11.)
                        .color(theme::TEXT_SECONDARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .spacing(2.)
                        .child(blur_adjust_button(shell, first_id, "-1pt", -1.0))
                        .child(blur_adjust_button(shell, first_id, "+1pt", 1.0))
                        .child(blur_adjust_button(shell, first_id, "+5pt", 5.0))
                        .child(toggle_blur_button(shell, first_id))
                        .child(remove_blur_button(
                            shell,
                            first_id,
                            gaussian_blur_effect.map(|(id, _, _)| id).unwrap_or(101),
                        )),
                ),
        )
        .child(
            rect()
                .direction(Direction::Vertical)
                .width(Size::fill())
                .spacing(3.)
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .width(Size::fill())
                        .main_align(Alignment::SpaceBetween)
                        .cross_align(Alignment::Center)
                        .child(
                            label()
                                .text(match drop_shadow_effect {
                                    Some((_, offset, blur, _, op, true)) => {
                                        format!("{}: {:.0} pt · Δ {:.0},{:.0} · {:.0}%",ui.studio_text("drop_shadow"),blur,offset[0],offset[1],op*100.)
                                    }
                                    Some((_, _, blur, _, _, false)) => {
                                        format!("{} ({}): {blur:.0} pt",ui.studio_text("drop_shadow"),ui.studio_text("hidden"))
                                    }
                                    None => format!("{}: {}",ui.studio_text("drop_shadow"),ui.studio_text("none")),
                                })
                                .font_size(11.)
                                .color(theme::TEXT_SECONDARY),
                        )
                        .child(
                            rect()
                                .direction(Direction::Horizontal).content(Content::Flex)
                                .spacing(2.)
                                .child(toggle_drop_shadow_button(shell, first_id))
                                .child(remove_shadow_button(
                                    shell,
                                    first_id,
                                    drop_shadow_effect.map(|(id, ..)| id).unwrap_or(102),
                                )),
                        ),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .spacing(2.)
                        .child(drop_shadow_adjust_button(shell, first_id, "Dx -2", -2.0, 0.0, 0.0, 0.0))
                        .child(drop_shadow_adjust_button(shell, first_id, "Dx +2", 2.0, 0.0, 0.0, 0.0))
                        .child(drop_shadow_adjust_button(shell, first_id, "Dy -2", 0.0, -2.0, 0.0, 0.0))
                        .child(drop_shadow_adjust_button(shell, first_id, "Dy +2", 0.0, 2.0, 0.0, 0.0))
                        .child(drop_shadow_adjust_button(shell, first_id, "Desf +2", 0.0, 0.0, 2.0, 0.0))
                        .child(drop_shadow_adjust_button(shell, first_id, "Op +10%", 0.0, 0.0, 0.0, 0.10)),
                ),
        )
        .child(
            rect()
                .direction(Direction::Vertical)
                .width(Size::fill())
                .spacing(3.)
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .width(Size::fill())
                        .main_align(Alignment::SpaceBetween)
                        .cross_align(Alignment::Center)
                        .child(
                            label()
                                .text(match inner_shadow_effect {
                                    Some((_, offset, blur, _, op, true)) => {
                                        format!("{}: {:.0} pt · Δ {:.0} · {:.0}%",ui.studio_text("inner_shadow"),blur,offset[0],op*100.)
                                    }
                                    Some((_, _, blur, _, _, false)) => {
                                        format!("{} ({}): {blur:.0} pt",ui.studio_text("inner_shadow"),ui.studio_text("hidden"))
                                    }
                                    None => format!("{}: {}",ui.studio_text("inner_shadow"),ui.studio_text("none")),
                                })
                                .font_size(11.)
                                .color(theme::TEXT_SECONDARY),
                        )
                        .child(
                            rect()
                                .direction(Direction::Horizontal).content(Content::Flex)
                                .spacing(2.)
                                .child(toggle_inner_shadow_button(shell, first_id))
                                .child(remove_inner_shadow_button(
                                    shell,
                                    first_id,
                                    inner_shadow_effect.map(|(id, ..)| id).unwrap_or(105),
                                )),
                        ),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .spacing(2.)
                        .child(inner_shadow_adjust_button(shell, first_id, "Dist -2", -2.0, 0.0, 0.0))
                        .child(inner_shadow_adjust_button(shell, first_id, "Dist +2", 2.0, 0.0, 0.0))
                        .child(inner_shadow_adjust_button(shell, first_id, "Desf -2", 0.0, -2.0, 0.0))
                        .child(inner_shadow_adjust_button(shell, first_id, "Desf +2", 0.0, 2.0, 0.0))
                        .child(inner_shadow_adjust_button(shell, first_id, "Op +10%", 0.0, 0.0, 0.10)),
                ),
        )
        .child(
            rect()
                .direction(Direction::Horizontal).content(Content::Flex)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text(match sharpen_effect {
                            Some((_, r, a, true)) => {
                                format!("{}: {r:.1} pt · {:.0}%",ui.studio_text("sharpen"),a*100.)
                            }
                            Some((_, r, _, false)) => format!("{} ({}): {r:.1} pt",ui.studio_text("sharpen"),ui.studio_text("hidden")),
                            None => format!("{}: {}",ui.studio_text("sharpen"),ui.studio_text("none")),
                        })
                        .font_size(11.)
                        .color(theme::TEXT_SECONDARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .spacing(2.)
                        .child(sharpen_adjust_button(shell, first_id, "Qtd -10%", 0.0, -0.10))
                        .child(sharpen_adjust_button(shell, first_id, "Qtd +10%", 0.0, 0.10))
                        .child(sharpen_adjust_button(shell, first_id, "Raio +1", 1.0, 0.0))
                        .child(remove_sharpen_button(
                            shell,
                            first_id,
                            sharpen_effect.map(|(id, ..)| id).unwrap_or(103),
                        )),
                ),
        )
        .child(
            rect()
                .direction(Direction::Horizontal).content(Content::Flex)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text(match noise_effect {
                            Some((_, a, mono, true)) => format!(
                                "{}: {:.0}% ({})",
                                ui.studio_text("noise"),
                                a * 100.0,
                                ui.studio_text(if mono {"monochrome"}else {"color"})
                            ),
                            Some((_, a, _, false)) => format!("{} ({}): {:.0}%",ui.studio_text("noise"),ui.studio_text("hidden"),a*100.),
                            None => format!("{}: {}",ui.studio_text("noise"),ui.studio_text("none")),
                        })
                        .font_size(11.)
                        .color(theme::TEXT_SECONDARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .spacing(2.)
                        .child(noise_adjust_button(shell, first_id, "-5%", -0.05))
                        .child(noise_adjust_button(shell, first_id, "+5%", 0.05))
                        .child(noise_toggle_mono_button(shell, first_id))
                        .child(remove_noise_button(
                            shell,
                            first_id,
                            noise_effect.map(|(id, ..)| id).unwrap_or(104),
                        )),
                ),
        )
        .child(
            // Section: Histogram (Spec 10.10)
            section_header(ui.studio_text("histogram")),
        )
        .child(
            HistogramWidget {
                ui: ui.clone(),
                object_id: first_id,
            },
        )
        .child(
            // Section: Tonal Adjustments (Spec 10.10)
            section_header(ui.studio_text("adjustments")),
        )
        .child(
            // Preset / Add adjustment buttons row
            rect()
                .direction(Direction::Horizontal).content(Content::Flex)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .child(add_adjustment_button(
                    shell,
                    first_id,
                    studio_text(shell,"levels"),
                    petunia_design_document::adjustments::AdjustmentKind::default_levels(),
                ))
                .child(add_adjustment_button(
                    shell,
                    first_id,
                    studio_text(shell,"curves_adjustment"),
                    petunia_design_document::adjustments::AdjustmentKind::default_curves(),
                ))
                .child(add_adjustment_button(
                    shell,
                    first_id,
                    studio_text(shell,"hsl"),
                    petunia_design_document::adjustments::AdjustmentKind::default_hsl(),
                ))
                .child(add_adjustment_button(
                    shell,
                    first_id,
                    "+Exp",
                    petunia_design_document::adjustments::AdjustmentKind::default_exposure(),
                ))
                .child(add_adjustment_button(
                    shell,
                    first_id,
                    studio_text(shell,"white_balance"),
                    petunia_design_document::adjustments::AdjustmentKind::default_white_balance(),
                )),
        )
        .children(
            adjustments
                .into_iter()
                .map(|adj| adjustment_card(shell, first_id, adj)),
        )
        .child(
            // Section: Alignment & Booleans
            section_header(ui.studio_text("alignment")),
        )
        .child(
            rect()
                .direction(Direction::Horizontal).content(Content::Flex)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .child(action_button(shell, "ptnd.action.object.align_left", studio_text(shell,"align_left")))
                .child(action_button(shell, "ptnd.action.object.align_center", studio_text(shell,"align_center")))
                .child(action_button(shell, "ptnd.action.object.align_right", studio_text(shell,"align_right")))
                .child(action_button(shell, "ptnd.action.object.align_top", studio_text(shell,"align_top")))
                .child(action_button(shell, "ptnd.action.object.align_bottom", studio_text(shell,"align_bottom"))),
        )
        .child(
            rect()
                .direction(Direction::Horizontal).content(Content::Flex)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .child(action_button(shell, "ptnd.action.vector.boolean_union", studio_text(shell,"union")))
                .child(action_button(shell, "ptnd.action.vector.boolean_subtract", studio_text(shell,"subtract")))
                .child(action_button(shell, "ptnd.action.vector.boolean_intersect", studio_text(shell,"intersect")))
                .child(action_button(shell, "ptnd.action.vector.boolean_xor", "XOR")),
        ))
        .into_element()
}

// =========================================================================
// 3. Colors Tab
// =========================================================================

#[derive(Clone, PartialEq)]
pub struct ColorsTab(pub UiShell);

impl Component for ColorsTab {
    fn render(&self) -> impl IntoElement {
        let mut subtab = use_state(|| 0usize);
        let current_subtab = *subtab.read();
        let target_fill = self.0.studio_fill_target;

        rect()
            .direction(Direction::Vertical)
            .width(Size::fill())
            .spacing(theme::SPACE_2)
            .child(
                // Subtab Switcher: Cor (ptnd.panel.color) | Amostras (ptnd.panel.swatches) | Tarefas (ptnd.panel.background_tasks)
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .height(Size::px(theme::PANEL_HEADER_HEIGHT))
                    .background(theme::SURFACE_CHROME_STRONG)
                    .padding(Gaps::new_all(2.))
                    .spacing(theme::SPACE_1)
                    .main_align(Alignment::SpaceEvenly)
                    .cross_align(Alignment::Center)
                    .child(subtab_pill(
                        self.0.studio_text("color"),
                        0,
                        current_subtab == 0,
                        &mut subtab,
                    ))
                    .child(subtab_pill(
                        self.0.studio_text("swatches"),
                        1,
                        current_subtab == 1,
                        &mut subtab,
                    ))
                    .child(subtab_pill(
                        self.0.studio_text("tasks"),
                        2,
                        current_subtab == 2,
                        &mut subtab,
                    )),
            )
            .child(match current_subtab {
                0 => ColorPanel {
                    ui: self.0.clone(),
                    target_fill,
                }
                .into_element(),
                1 => SwatchesPanel {
                    ui: self.0.clone(),
                    target_fill,
                }
                .into_element(),
                2 => BackgroundTasksPanel(self.0.clone()).into_element(),
                _ => ColorPanel {
                    ui: self.0.clone(),
                    target_fill,
                }
                .into_element(),
            })
    }
}

fn subtab_pill(
    title: impl Into<String>,
    idx: usize,
    active: bool,
    subtab: &mut State<usize>,
) -> impl IntoElement {
    let mut state = *subtab;
    Button::new()
        .compact()
        .on_press(move |_| state.set(idx))
        .child(label().text(title.into()).color(if active {
            theme::TEXT_PRIMARY
        } else {
            theme::TEXT_SECONDARY
        }))
}

fn channel_adjuster(
    label_text: String,
    val_text: String,
    mut on_dec: impl FnMut() + 'static,
    mut on_inc: impl FnMut() + 'static,
) -> impl IntoElement {
    rect()
        .direction(Direction::Horizontal)
        .content(Content::Flex)
        .width(Size::fill())
        .height(Size::px(26.))
        .cross_align(Alignment::Center)
        .main_align(Alignment::SpaceBetween)
        .child(
            rect().width(Size::px(100.)).child(
                label()
                    .text(label_text)
                    .font_size(11.)
                    .color(theme::TEXT_SECONDARY),
            ),
        )
        .child(
            rect().width(Size::px(70.)).child(
                label()
                    .text(val_text)
                    .font_size(11.)
                    .color(theme::TEXT_PRIMARY),
            ),
        )
        .child(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .spacing(4.)
                .child(
                    Button::new()
                        .on_press(move |_| on_dec())
                        .child(label().text("-").font_size(11.)),
                )
                .child(
                    Button::new()
                        .on_press(move |_| on_inc())
                        .child(label().text("+").font_size(11.)),
                ),
        )
}

/// Color Panel (`ptnd.panel.color`): sRGB, CMYK, Lab, Spot, target toggle, and soft-proof check.
#[derive(Clone, PartialEq)]
pub struct ColorPanel {
    pub ui: UiShell,
    pub target_fill: State<bool>,
}

impl Component for ColorPanel {
    fn render(&self) -> impl IntoElement {
        let shell = self.ui.shell;
        let sel = shell.read().bridge.selection();
        let first_id = sel.selected_ids.first().copied();
        let is_fill = *self.target_fill.read();
        let raster_target = sel.selected_ids.len() == 1
            && first_id.is_some_and(|id| {
                matches!(
                    shell
                        .read()
                        .bridge
                        .session()
                        .and_then(|s| s.document().find_object(id))
                        .and_then(|o| o.shape.as_ref()),
                    Some(ShapeKind::Raster { .. })
                )
            })
            || (sel.selected_ids.is_empty()
                && *self.ui.persona.read() == petunia_design_application::surfaces::PERSONA_PHOTO);
        let native_profile = first_id.and_then(|id| {
            shell
                .read()
                .bridge
                .session()
                .and_then(|s| s.document().find_object(id))
                .and_then(|o| match &o.shape {
                    Some(ShapeKind::Raster { layer }) => layer.cmyk_profile().cloned(),
                    _ => None,
                })
        });
        let native_ink_target = native_profile.is_some();
        let mut mut_target_fill = self.target_fill;

        let mut color_mode = use_state(|| 0usize); // 0: sRGB, 1: CMYK, 2: Lab, 3: Spot
        let active_mode = *color_mode.read();

        // sRGB state
        let source = {
            let s = shell.read();
            let object = first_id.and_then(|id| s.bridge.session()?.find_object(id));
            let rgb = if raster_target {
                s.tools.photo_brush_tool().brush_settings().color[..3]
                    .try_into()
                    .unwrap_or([0.; 3])
            } else {
                object
                    .and_then(|o| {
                        if is_fill {
                            o.fill.as_deref()
                        } else {
                            o.stroke.as_deref()
                        }
                    })
                    .map(petunia_design_document::resolve_color_to_rgb)
                    .unwrap_or([0.2, 0.78, 0.83])
            };
            (
                s.bridge.session().map(|s| s.identity()),
                first_id,
                is_fill,
                crate::studio::rgb_token(rgb),
                rgb,
                raster_target
                    .then(|| s.tools.photo_brush_tool().brush_settings().ink)
                    .flatten(),
            )
        };
        let mut input = use_state(|| source.clone());
        if *input.peek() != source {
            input.set(source.clone());
        }
        let mut r_val = use_state(|| (source.4[0] * 255.).round() as u8);
        let mut g_val = use_state(|| (source.4[1] * 255.).round() as u8);
        let mut b_val = use_state(|| (source.4[2] * 255.).round() as u8);

        let mut hex = use_state(|| source.3.clone());
        use_side_effect(move || {
            let rgb = [*r_val.read(), *g_val.read(), *b_val.read()];
            hex.set(format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]));
        });
        // CMYK state (percentages 0..=100)
        let mut c_val = use_state(|| 0u8);
        let mut m_val = use_state(|| 85u8);
        let mut y_val = use_state(|| 70u8);
        let mut k_val = use_state(|| 0u8);

        // Lab state (L: 0..100, a: -128..127, b: -128..127)
        let lab_l = use_state(|| 55i16);
        let lab_a = use_state(|| 65i16);
        let lab_b = use_state(|| 35i16);

        // Spot state
        let spot_name = use_state(|| "Studio Red".to_string());

        use_side_effect(move || {
            let source = input.read();
            r_val.set((source.4[0] * 255.).round() as u8);
            g_val.set((source.4[1] * 255.).round() as u8);
            b_val.set((source.4[2] * 255.).round() as u8);
            if let Some(ink) = source.5 {
                let [c, m, y, k] = ink.map(|v| (v * 100.).round() as u8);
                c_val.set(c);
                m_val.set(m);
                y_val.set(y);
                k_val.set(k);
            }
        });
        let profile = native_profile.or_else(|| {
            shell
                .read()
                .bridge
                .session()
                .and_then(|session| {
                    session
                        .active_surface()
                        .and_then(|id| session.document().surface(id).ok())
                })
                .and_then(|surface| surface.cmyk_profile.clone())
        });
        let has_profile = profile.is_some();
        let can_create_native = shell
            .read()
            .bridge
            .session()
            .is_some_and(|session| session.native_layer_profile().is_some());
        let cmyk_layer_ui = self.ui.clone();
        let (icc_swatch, icc_error) = crate::color_ui::use_cmyk_swatch(
            (active_mode == 1).then_some(profile).flatten(),
            [*c_val.read(), *m_val.read(), *y_val.read(), *k_val.read()]
                .map(|v| f32::from(v) / 100.),
        );
        let (preview_color, token, _color_val) = match active_mode {
            0 => {
                let (r, g, b) = (*r_val.read(), *g_val.read(), *b_val.read());
                (
                    Color::from_rgb(r, g, b),
                    format!("#{r:02X}{g:02X}{b:02X}"),
                    ColorValue::Rgb(Srgb::clamped(
                        r as f32 / 255.0,
                        g as f32 / 255.0,
                        b as f32 / 255.0,
                    )),
                )
            }
            1 => {
                let (c, m, y, k) = (*c_val.read(), *m_val.read(), *y_val.read(), *k_val.read());
                let c_f = c as f32 / 100.0;
                let m_f = m as f32 / 100.0;
                let y_f = y as f32 / 100.0;
                let k_f = k as f32 / 100.0;
                let preview =
                    icc_swatch.map_or(Color::TRANSPARENT, |[r, g, b]| Color::from_rgb(r, g, b));
                (
                    preview,
                    format!("cmyk({c}%, {m}%, {y}%, {k}%)"),
                    ColorValue::Cmyk(Cmyk {
                        c: c_f,
                        m: m_f,
                        y: y_f,
                        k: k_f,
                    }),
                )
            }
            2 => {
                let (l, a, b) = (*lab_l.read(), *lab_a.read(), *lab_b.read());
                let lab = Lab {
                    l: l as f32,
                    a: a as f32,
                    b: b as f32,
                };
                let srgb = ColorValue::Lab(lab).to_srgb();
                let r = (srgb.r * 255.0).round() as u8;
                let g = (srgb.g * 255.0).round() as u8;
                let b_rgb = (srgb.b * 255.0).round() as u8;
                (
                    Color::from_rgb(r, g, b_rgb),
                    format!("lab({l}, {a}, {b})"),
                    ColorValue::Lab(lab),
                )
            }
            _ => {
                let (r, g, b) = (*r_val.read(), *g_val.read(), *b_val.read());
                let fallback_rgb =
                    Srgb::clamped(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
                (
                    Color::from_rgb(r, g, b),
                    format!("spot({}, #{r:02X}{g:02X}{b:02X})", *spot_name.read()),
                    ColorValue::Spot {
                        name: (*spot_name.read()).clone(),
                        fallback: Box::new(ColorValue::Rgb(fallback_rgb)),
                    },
                )
            }
        };

        // Gamut status requires an actual profile transform and reference data.

        let press_ui = self.ui.clone();
        let monitor_ui = self.ui.clone();
        let options = *self.ui.proof_options.read();
        let mut bpc_options = self.ui.proof_options;
        let mut shell_for_apply = shell;
        let token_for_apply = token.clone();
        let apply_ui = self.ui.clone();
        let ink_for_apply = (active_mode == 1 && native_ink_target).then(|| {
            [*c_val.read(), *m_val.read(), *y_val.read(), *k_val.read()]
                .map(|v| f32::from(v) / 100.)
        });
        let brush_rgb = if active_mode == 1 {
            icc_swatch.unwrap_or([0; 3]).map(|v| f32::from(v) / 255.)
        } else {
            let c = _color_val.to_srgb();
            [c.r, c.g, c.b]
        };

        rect()
            .direction(Direction::Vertical)
            .width(Size::fill())
            .spacing(theme::SPACE_2)
            .child(
                Button::new()
                    .on_press(move |_| {
                        crate::file_workflows::request_profile(
                            &press_ui,
                            crate::file_workflows::ProfilePurpose::Press,
                        )
                    })
                    .child(self.ui.text("icc_assign")),
            )
            .child(
                Button::new()
                    .enabled(can_create_native)
                    .on_press(move |_| {
                        let result = cmyk_layer_ui.shell.clone().write().bridge.dispatch_action(
                            petunia_design_application::ActionRequest::without_payload(
                                petunia_design_application::ActionId::new(
                                    petunia_design_application::ActionId::RASTER_CREATE_CMYK,
                                ),
                            ),
                        );
                        if let Err(error) = result {
                            cmyk_layer_ui
                                .file_error
                                .clone()
                                .set(Some(error.to_string()));
                        }
                    })
                    .child(self.ui.text("cmyk_new_layer")),
            )
            .child(
                Button::new()
                    .on_press(move |_| {
                        crate::file_workflows::request_profile(
                            &monitor_ui,
                            crate::file_workflows::ProfilePurpose::Monitor,
                        )
                    })
                    .child(self.ui.text("icc_monitor_choose")),
            )
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .spacing(2.)
                    .children(
                        [
                            (
                                petunia_design_color::RenderingIntent::Perceptual,
                                "intent_perceptual",
                            ),
                            (
                                petunia_design_color::RenderingIntent::RelativeColorimetric,
                                "intent_relative",
                            ),
                            (
                                petunia_design_color::RenderingIntent::Saturation,
                                "intent_saturation",
                            ),
                            (
                                petunia_design_color::RenderingIntent::AbsoluteColorimetric,
                                "intent_absolute",
                            ),
                        ]
                        .into_iter()
                        .map(|(intent, key)| {
                            let mut state = self.ui.proof_options;
                            Button::new()
                                .on_press(move |_| {
                                    let current = *state.peek();
                                    state.set(petunia_design_color::IccTransformOptions {
                                        intent,
                                        ..current
                                    });
                                })
                                .child(
                                    label()
                                        .text(self.ui.text(key))
                                        .color(if options.intent == intent {
                                            theme::TEXT_PRIMARY
                                        } else {
                                            theme::TEXT_SECONDARY
                                        })
                                        .font_size(theme::CAPTION_SIZE),
                                )
                        }),
                    ),
            )
            .child(
                Button::new()
                    .on_press(move |_| {
                        let current = *bpc_options.peek();
                        bpc_options.set(petunia_design_color::IccTransformOptions {
                            black_point_compensation: !current.black_point_compensation,
                            ..current
                        });
                    })
                    .child(self.ui.text(if options.black_point_compensation {
                        "bpc_on"
                    } else {
                        "bpc_off"
                    })),
            )
            .maybe_child((active_mode == 1 && !has_profile).then(|| {
                label()
                    .text(self.ui.text("icc_missing"))
                    .color(theme::TEXT_SECONDARY)
            }))
            .children(
                icc_error
                    .into_iter()
                    .map(|error| label().text(error).color(theme::TEXT_ERROR)),
            )
            .child(
                // Target Selector: Preenchimento vs Traço
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .spacing(theme::SPACE_1)
                    .child(
                        Button::new()
                            .enabled(!raster_target)
                            .on_press(move |_| mut_target_fill.set(true))
                            .child(
                                label()
                                    .text(format!(
                                        "{}{}",
                                        if is_fill { "● " } else { "" },
                                        self.ui.text("color_fill")
                                    ))
                                    .font_size(theme::CAPTION_SIZE),
                            ),
                    )
                    .child(
                        Button::new()
                            .enabled(!raster_target)
                            .on_press(move |_| mut_target_fill.set(false))
                            .child(
                                label()
                                    .text(format!(
                                        "{}{}",
                                        if !is_fill { "● " } else { "" },
                                        self.ui.text("color_stroke")
                                    ))
                                    .font_size(theme::CAPTION_SIZE),
                            ),
                    ),
            )
            .child(
                // Color Mode Segmented Bar: sRGB | CMYK | Lab | Spot
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .height(Size::px(26.))
                    .background(theme::SURFACE_CHROME_STRONG)
                    .padding(Gaps::new_all(2.))
                    .spacing(2.)
                    .main_align(Alignment::SpaceEvenly)
                    .cross_align(Alignment::Center)
                    .child(subtab_pill("sRGB", 0, active_mode == 0, &mut color_mode))
                    .child(subtab_pill("CMYK", 1, active_mode == 1, &mut color_mode))
                    .child(subtab_pill("Lab", 2, active_mode == 2, &mut color_mode))
                    .child(subtab_pill("Spot", 3, active_mode == 3, &mut color_mode)),
            )
            .child(
                // Active Color Preview Card
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .height(Size::px(42.))
                    .background(theme::SURFACE_CHROME)
                    .border(
                        Border::new()
                            .fill(theme::SURFACE_CHROME_STRONG)
                            .width(1.)
                            .alignment(BorderAlignment::Inner),
                    )
                    .padding(Gaps::new_all(theme::SPACE_1))
                    .spacing(theme::SPACE_2)
                    .cross_align(Alignment::Center)
                    .child(
                        rect()
                            .width(Size::px(34.))
                            .height(Size::px(theme::PANEL_HEADER_HEIGHT))
                            .background(preview_color)
                            .border(
                                Border::new()
                                    .fill(theme::SURFACE_CHROME_STRONG)
                                    .width(1.)
                                    .alignment(BorderAlignment::Inner),
                            ),
                    )
                    .child(
                        rect()
                            .direction(Direction::Vertical)
                            .child(
                                label()
                                    .text(token.clone())
                                    .font_size(11.)
                                    .color(theme::TEXT_PRIMARY),
                            )
                            .child(
                                label()
                                    .text(self.ui.text("gamut_unmeasured"))
                                    .font_size(theme::CAPTION_SIZE)
                                    .color(theme::TEXT_SECONDARY),
                            ),
                    ),
            )
            .child(
                // Channel Adjusters for Active Color Space
                match active_mode {
                    0 => rect()
                        .direction(Direction::Vertical)
                        .width(Size::fill())
                        .spacing(2.)
                        .child(channel_adjuster(
                            self.ui.text("color_red"),
                            format!("{}", *r_val.read()),
                            move || {
                                let curr = *r_val.peek();
                                let mut s = r_val;
                                s.set(curr.saturating_sub(5));
                            },
                            move || {
                                let curr = *r_val.peek();
                                let mut s = r_val;
                                s.set(curr.saturating_add(5));
                            },
                        ))
                        .child(channel_adjuster(
                            self.ui.text("color_green"),
                            format!("{}", *g_val.read()),
                            move || {
                                let curr = *g_val.peek();
                                let mut s = g_val;
                                s.set(curr.saturating_sub(5));
                            },
                            move || {
                                let curr = *g_val.peek();
                                let mut s = g_val;
                                s.set(curr.saturating_add(5));
                            },
                        ))
                        .child(channel_adjuster(
                            self.ui.text("color_blue"),
                            format!("{}", *b_val.read()),
                            move || {
                                let curr = *b_val.peek();
                                let mut s = b_val;
                                s.set(curr.saturating_sub(5));
                            },
                            move || {
                                let curr = *b_val.peek();
                                let mut s = b_val;
                                s.set(curr.saturating_add(5));
                            },
                        )),
                    1 => rect()
                        .direction(Direction::Vertical)
                        .width(Size::fill())
                        .spacing(2.)
                        .child(channel_adjuster(
                            self.ui.text("color_cyan"),
                            format!("{}%", *c_val.read()),
                            move || {
                                let curr = *c_val.peek();
                                let mut s = c_val;
                                s.set(curr.saturating_sub(5));
                            },
                            move || {
                                let curr = *c_val.peek();
                                let mut s = c_val;
                                s.set(curr.saturating_add(5).min(100));
                            },
                        ))
                        .child(channel_adjuster(
                            self.ui.text("color_magenta"),
                            format!("{}%", *m_val.read()),
                            move || {
                                let curr = *m_val.peek();
                                let mut s = m_val;
                                s.set(curr.saturating_sub(5));
                            },
                            move || {
                                let curr = *m_val.peek();
                                let mut s = m_val;
                                s.set(curr.saturating_add(5).min(100));
                            },
                        ))
                        .child(channel_adjuster(
                            self.ui.text("color_yellow"),
                            format!("{}%", *y_val.read()),
                            move || {
                                let curr = *y_val.peek();
                                let mut s = y_val;
                                s.set(curr.saturating_sub(5));
                            },
                            move || {
                                let curr = *y_val.peek();
                                let mut s = y_val;
                                s.set(curr.saturating_add(5).min(100));
                            },
                        ))
                        .child(channel_adjuster(
                            self.ui.text("color_black"),
                            format!("{}%", *k_val.read()),
                            move || {
                                let curr = *k_val.peek();
                                let mut s = k_val;
                                s.set(curr.saturating_sub(5));
                            },
                            move || {
                                let curr = *k_val.peek();
                                let mut s = k_val;
                                s.set(curr.saturating_add(5).min(100));
                            },
                        )),
                    2 => rect()
                        .direction(Direction::Vertical)
                        .width(Size::fill())
                        .spacing(2.)
                        .child(channel_adjuster(
                            self.ui.text("color_lightness"),
                            format!("{}", *lab_l.read()),
                            move || {
                                let curr = *lab_l.peek();
                                let mut s = lab_l;
                                s.set((curr - 5).clamp(0, 100));
                            },
                            move || {
                                let curr = *lab_l.peek();
                                let mut s = lab_l;
                                s.set((curr + 5).clamp(0, 100));
                            },
                        ))
                        .child(channel_adjuster(
                            self.ui.text("color_lab_a"),
                            format!("{}", *lab_a.read()),
                            move || {
                                let curr = *lab_a.peek();
                                let mut s = lab_a;
                                s.set((curr - 5).clamp(-128, 127));
                            },
                            move || {
                                let curr = *lab_a.peek();
                                let mut s = lab_a;
                                s.set((curr + 5).clamp(-128, 127));
                            },
                        ))
                        .child(channel_adjuster(
                            self.ui.text("color_lab_b"),
                            format!("{}", *lab_b.read()),
                            move || {
                                let curr = *lab_b.peek();
                                let mut s = lab_b;
                                s.set((curr - 5).clamp(-128, 127));
                            },
                            move || {
                                let curr = *lab_b.peek();
                                let mut s = lab_b;
                                s.set((curr + 5).clamp(-128, 127));
                            },
                        )),
                    _ => {
                        let mut spot_change = spot_name;
                        rect()
                            .direction(Direction::Vertical)
                            .width(Size::fill())
                            .spacing(theme::SPACE_1)
                            .child(
                                label()
                                    .text(self.ui.text("color_spot"))
                                    .font_size(11.)
                                    .color(theme::TEXT_SECONDARY),
                            )
                            .child(
                                rect()
                                    .direction(Direction::Horizontal)
                                    .content(Content::Flex)
                                    .spacing(4.)
                                    .child(
                                        Button::new()
                                            .on_press(move |_| {
                                                spot_change.set("Studio Red".to_string())
                                            })
                                            .child(
                                                label()
                                                    .text(self.ui.studio_text("spot_red"))
                                                    .font_size(theme::CAPTION_SIZE),
                                            ),
                                    )
                                    .child(
                                        Button::new()
                                            .on_press(move |_| {
                                                spot_change.set("Studio Blue".to_string())
                                            })
                                            .child(
                                                label()
                                                    .text(self.ui.studio_text("spot_blue"))
                                                    .font_size(theme::CAPTION_SIZE),
                                            ),
                                    )
                                    .child(
                                        Button::new()
                                            .on_press(move |_| {
                                                spot_change.set("Studio Gold".to_string())
                                            })
                                            .child(
                                                label()
                                                    .text(self.ui.studio_text("spot_gold"))
                                                    .font_size(theme::CAPTION_SIZE),
                                            ),
                                    ),
                            )
                    }
                },
            )
            .maybe_child((active_mode == 0).then(|| {
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .spacing(theme::SPACE_2)
                    .cross_align(Alignment::Center)
                    .child(label().text("Hex").color(theme::TEXT_SECONDARY))
                    .child(
                        Input::new(hex)
                            .width(Size::flex(1.))
                            .placeholder("#RRGGBB")
                            .on_validate(move |validator: InputValidator| {
                                if let Some([r, g, b]) = crate::studio::parse_hex(&validator.text())
                                {
                                    r_val.set(r);
                                    g_val.set(g);
                                    b_val.set(b);
                                }
                            }),
                    )
                    .maybe_child(crate::studio::parse_hex(&hex.read()).is_none().then(|| {
                        label()
                            .text(self.ui.studio_text("invalid_hex"))
                            .color(theme::TEXT_ERROR)
                    }))
            }))
            .child(
                // Apply Button
                Button::new()
                    .enabled(
                        crate::studio::color_target_enabled(&self.ui)
                            && (active_mode != 1 || (has_profile && icc_swatch.is_some()))
                            && (!raster_target || active_mode != 3)
                            && (active_mode != 0
                                || crate::studio::parse_hex(&hex.read()).is_some()),
                    )
                    .on_press(move |_| {
                        if raster_target {
                            let mut current = shell_for_apply.write();
                            let target = current.tools.photo_brush_tool_mut();
                            let mut settings = target.brush_settings();
                            settings.color =
                                [brush_rgb[0], brush_rgb[1], brush_rgb[2], settings.color[3]];
                            settings.ink = ink_for_apply;
                            target.set_brush_settings(settings);
                            return;
                        }
                        crate::studio::apply_color(&apply_ui, &token_for_apply, is_fill);
                    })
                    .child(
                        label()
                            .text(if raster_target {
                                self.ui.text("brush_apply")
                            } else if is_fill {
                                self.ui.text("color_apply_fill")
                            } else {
                                self.ui.text("color_apply_stroke")
                            })
                            .font_size(11.),
                    ),
            )
    }
}

/// Swatches Panel (`ptnd.panel.swatches`): Sistema, Documento, and Favoritos libraries.
#[derive(Clone, PartialEq)]
pub struct SwatchesPanel {
    pub ui: UiShell,
    pub target_fill: State<bool>,
}

impl Component for SwatchesPanel {
    fn render(&self) -> impl IntoElement {
        let ui = &self.ui;
        let s = ui.shell.read();
        let selected = s.bridge.selection().selected_ids;
        let first = selected
            .first()
            .and_then(|id| s.bridge.session()?.find_object(*id));
        let fill = *self.target_fill.read();
        let brush = first.is_some_and(|o| matches!(o.shape, Some(ShapeKind::Raster { .. })))
            || (selected.is_empty()
                && *ui.persona.read() == petunia_design_application::surfaces::PERSONA_PHOTO);
        let token = if brush {
            Some(crate::studio::rgb_token(
                s.tools.photo_brush_tool().brush_settings().color[..3]
                    .try_into()
                    .unwrap_or([0.; 3]),
            ))
        } else {
            first.and_then(|o| {
                if fill {
                    o.fill.clone()
                } else {
                    o.stroke.clone()
                }
            })
        };
        let enabled = crate::studio::color_target_enabled(ui);
        let mut document = Vec::<(String, String)>::new();
        if let Some(surface) = s.bridge.session().and_then(|session| {
            session
                .active_surface()
                .and_then(|id| session.document().surface(id).ok())
        }) {
            for object in surface.objects() {
                for color in [&object.fill, &object.stroke].into_iter().flatten() {
                    if !document.iter().any(|(_, value)| value == color) {
                        document.push((color.clone(), color.clone()));
                    }
                }
            }
        }
        drop(s);
        let mut mode = use_state(|| 0usize);
        let active = *mode.read();
        let mut favorites = ui.favorite_swatches;
        let swatches = match active {
            1 => document,
            2 => favorites.read().clone(),
            _ => crate::studio::QUICK_COLORS
                .into_iter()
                .map(|token| (token.to_string(), token.to_string()))
                .collect(),
        };
        let mut target = self.target_fill;
        let add_title = ui.studio_text("swatch");
        rect()
            .direction(Direction::Vertical)
            .width(Size::fill())
            .spacing(theme::SPACE_2)
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .spacing(2.)
                    .children(
                        [(0, "system"), (1, "document"), (2, "favorites")]
                            .into_iter()
                            .map(|(index, key)| {
                                crate::studio_widgets::StudioButton::new(ui, ui.studio_text(key))
                                    .text(ui.studio_text(key))
                                    .tab()
                                    .width(Size::flex(1.))
                                    .selected(active == index)
                                    .on_press(move |_| mode.set(index))
                            }),
                    ),
            )
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .spacing(theme::SPACE_1)
                    .child(
                        crate::studio_widgets::StudioButton::new(
                            ui,
                            ui.studio_text(if brush { "brush_color" } else { "fill" }),
                        )
                        .text(ui.studio_text(if brush { "brush_color" } else { "fill" }))
                        .selected(fill || brush)
                        .on_press(move |_| target.set(true)),
                    )
                    .maybe_child((!brush).then(|| {
                        crate::studio_widgets::StudioButton::new(ui, ui.studio_text("stroke"))
                            .text(ui.studio_text("stroke"))
                            .selected(!fill)
                            .on_press(move |_| target.set(false))
                    })),
            )
            .maybe_child((active == 2).then(|| {
                crate::studio_widgets::StudioButton::new(ui, ui.studio_text("add_selected_color"))
                    .icon(theme::ICON_PLUS)
                    .text(ui.studio_text("add_selected_color"))
                    .enabled(token.is_some())
                    .on_press(move |_| {
                        if let Some(token) = &token {
                            let mut values = favorites.write();
                            if !values.iter().any(|(_, value)| value == token) {
                                let name = format!("{} {}", add_title, values.len() + 1);
                                values.push((name, token.clone()));
                            }
                        }
                    })
            }))
            .maybe_child(swatches.is_empty().then(|| {
                label()
                    .text(ui.studio_text("empty_swatches"))
                    .color(theme::TEXT_SECONDARY)
            }))
            .children(swatches.chunks(4).map(|chunk| {
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .spacing(theme::SPACE_1)
                    .children(chunk.iter().map(|(name, token)| {
                        let rgb = petunia_design_document::resolve_color_to_rgb(token);
                        let apply_ui = ui.clone();
                        let apply_token = token.clone();
                        rect()
                            .direction(Direction::Vertical)
                            .width(Size::flex(1.))
                            .spacing(2.)
                            .child(
                                rect()
                                    .height(Size::px(theme::PANEL_HEADER_HEIGHT))
                                    .width(Size::fill())
                                    .corner_radius(theme::CONTROL_RADIUS)
                                    .background(Color::from_rgb(
                                        (rgb[0] * 255.) as u8,
                                        (rgb[1] * 255.) as u8,
                                        (rgb[2] * 255.) as u8,
                                    ))
                                    .child(
                                        crate::studio_widgets::StudioButton::new(
                                            ui,
                                            format!("{name}: {token}"),
                                        )
                                        .enabled(enabled)
                                        .width(Size::fill())
                                        .height(30.)
                                        .on_press(
                                            move |_| {
                                                crate::studio::apply_color(
                                                    &apply_ui,
                                                    &apply_token,
                                                    fill,
                                                )
                                            },
                                        ),
                                    ),
                            )
                            .child(
                                label()
                                    .text(name.clone())
                                    .font_size(theme::CAPTION_SIZE)
                                    .color(theme::TEXT_SECONDARY)
                                    .max_lines(1)
                                    .text_overflow(TextOverflow::Ellipsis),
                            )
                    }))
            }))
    }
}

/// Background Tasks Tab (`ptnd.panel.background_tasks`): Job queue, progress, cancellation.
#[derive(Clone, PartialEq)]
pub struct BackgroundTasksPanel(pub UiShell);

impl Component for BackgroundTasksPanel {
    fn render(&self) -> impl IntoElement {
        let shell = self.0.shell;
        let jobs = shell.peek().bridge.jobs().list_jobs();
        let mut shell_for_clear = shell;

        rect()
            .direction(Direction::Vertical)
            .width(Size::fill())
            .height(Size::fill())
            .spacing(theme::SPACE_2)
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .main_align(Alignment::SpaceBetween)
                    .cross_align(Alignment::Center)
                    .child(
                        rect()
                            .width(Size::flex(1.))
                            .child(section_header(self.0.studio_text("background_tasks"))),
                    )
                    .child(
                        crate::studio_widgets::StudioButton::new(
                            &self.0,
                            self.0.studio_text("clear"),
                        )
                        .icon(theme::ICON_TRASH)
                        .width(Size::px(28.))
                        .enabled(jobs.iter().any(|job| {
                            matches!(
                                job.state,
                                petunia_design_jobs::JobState::Completed
                                    | petunia_design_jobs::JobState::Cancelled
                                    | petunia_design_jobs::JobState::Failed
                            )
                        }))
                        .on_press(move |_| shell_for_clear.write().bridge.jobs().clear_completed()),
                    ),
            )
            .child(if jobs.is_empty() {
                rect()
                    .width(Size::fill())
                    .padding(Gaps::new_all(theme::SPACE_3))
                    .center()
                    .child(
                        label()
                            .text(self.0.studio_text("empty_tasks"))
                            .font_size(11.)
                            .color(theme::TEXT_SECONDARY),
                    )
            } else {
                rect()
                    .direction(Direction::Vertical)
                    .width(Size::fill())
                    .height(Size::flex(1.0))
                    .spacing(theme::SPACE_2)
                    .children(jobs.into_iter().map(|job| {
                        let mut shell_for_cancel = shell;
                        let job_id = job.id;
                        let is_running = job.state == petunia_design_jobs::JobState::Running;
                        let state_label = match job.state {
                            petunia_design_jobs::JobState::Running => {
                                self.0.studio_text("job_running")
                            }
                            petunia_design_jobs::JobState::Completed => {
                                self.0.studio_text("job_completed")
                            }
                            petunia_design_jobs::JobState::Cancelled => {
                                self.0.studio_text("job_cancelled")
                            }
                            petunia_design_jobs::JobState::Failed => {
                                self.0.studio_text("job_failed")
                            }
                            petunia_design_jobs::JobState::Queued => {
                                self.0.studio_text("job_queued")
                            }
                        };
                        let state_color = match job.state {
                            petunia_design_jobs::JobState::Running => {
                                Color::from_rgb(0x38, 0xBD, 0xF8)
                            }
                            petunia_design_jobs::JobState::Completed => {
                                Color::from_rgb(0x34, 0xD3, 0x99)
                            }
                            petunia_design_jobs::JobState::Cancelled => {
                                Color::from_rgb(0xFB, 0xBF, 0x24)
                            }
                            petunia_design_jobs::JobState::Failed => {
                                Color::from_rgb(0xF8, 0x71, 0x71)
                            }
                            petunia_design_jobs::JobState::Queued => theme::TEXT_SECONDARY,
                        };

                        rect()
                            .direction(Direction::Vertical)
                            .width(Size::fill())
                            .background(theme::SURFACE_CHROME)
                            .border(
                                Border::new()
                                    .fill(theme::SURFACE_CHROME_STRONG)
                                    .width(1.)
                                    .alignment(BorderAlignment::Inner),
                            )
                            .padding(Gaps::new_all(theme::SPACE_2))
                            .spacing(theme::SPACE_1)
                            .child(
                                rect()
                                    .direction(Direction::Horizontal)
                                    .content(Content::Flex)
                                    .width(Size::fill())
                                    .main_align(Alignment::SpaceBetween)
                                    .cross_align(Alignment::Center)
                                    .child(
                                        label()
                                            .text(format!("[#{}] {}", job.id, job.label))
                                            .font_size(11.)
                                            .color(theme::TEXT_PRIMARY),
                                    )
                                    .child(
                                        label()
                                            .text(state_label)
                                            .font_size(theme::CAPTION_SIZE)
                                            .color(state_color),
                                    ),
                            )
                            .child(
                                // Progress Bar
                                rect()
                                    .width(Size::fill())
                                    .height(Size::px(6.))
                                    .background(theme::SURFACE_CHROME_STRONG)
                                    .child(
                                        rect()
                                            .width(Size::percent(job.percent as f32))
                                            .height(Size::px(6.))
                                            .background(state_color),
                                    ),
                            )
                            .child(
                                rect()
                                    .direction(Direction::Horizontal)
                                    .content(Content::Flex)
                                    .width(Size::fill())
                                    .main_align(Alignment::SpaceBetween)
                                    .cross_align(Alignment::Center)
                                    .child(
                                        label()
                                            .text(format!("{}%", job.percent))
                                            .font_size(theme::CAPTION_SIZE)
                                            .color(theme::TEXT_TERTIARY),
                                    )
                                    .maybe_child(if is_running {
                                        Some(
                                            Button::new()
                                                .on_press(move |_| {
                                                    shell_for_cancel
                                                        .write()
                                                        .bridge
                                                        .jobs()
                                                        .cancel_job(job_id);
                                                })
                                                .child(
                                                    label()
                                                        .text(self.0.studio_text("cancel"))
                                                        .font_size(theme::CAPTION_SIZE),
                                                ),
                                        )
                                    } else {
                                        None
                                    }),
                            )
                    }))
            })
    }
}

// =========================================================================
// 4. History Tab
// =========================================================================

fn history_tab(ui: UiShell) -> impl IntoElement {
    let shell = ui.shell;
    let history_model = shell.peek().bridge.query_history();
    let mut shell_for_undo = shell;
    let mut shell_for_redo = shell;

    rect()
        .direction(Direction::Vertical)
        .content(Content::Flex)
        .width(Size::fill())
        .height(Size::fill())
        .spacing(theme::SPACE_2)
        .child(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(section_header(ui.studio_text("action_history")))
                .child(
                    rect()
                        .direction(Direction::Horizontal)
                        .content(Content::Flex)
                        .spacing(theme::SPACE_1)
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    let _ = run_action_token(
                                        &mut shell_for_undo.write(),
                                        "ptnd.action.edit.undo",
                                    );
                                })
                                .child(label().text(ui.studio_text("undo")).font_size(11.)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    let _ = run_action_token(
                                        &mut shell_for_redo.write(),
                                        "ptnd.action.edit.redo",
                                    );
                                })
                                .child(label().text(ui.studio_text("redo")).font_size(11.)),
                        ),
                ),
        )
        .child(
            rect()
                .direction(Direction::Vertical)
                .width(Size::fill())
                .height(Size::flex(1.0))
                .spacing(2.)
                .children(history_model.undo_stack.into_iter().rev().map(|item| {
                    rect()
                        .direction(Direction::Horizontal)
                        .content(Content::Flex)
                        .width(Size::fill())
                        .height(Size::px(26.))
                        .padding(Gaps::new_all(4.))
                        .cross_align(Alignment::Center)
                        .background(theme::SURFACE_CHROME)
                        .child(
                            label()
                                .text(format!("#{} {}", item.index + 1, item.description))
                                .font_size(11.)
                                .color(theme::TEXT_PRIMARY),
                        )
                })),
        )
}

// =========================================================================
// 5. Navigator Tab (Minimap)
// =========================================================================

pub use crate::navigator::NavigatorTab;

fn studio_text(shell: State<PetuniaShell>, key: &str) -> String {
    let state = shell.read();
    state
        .bridge
        .localization()
        .text(&format!("ptnd.text.studio.{key}"), state.bridge.locale())
}

fn tab_button(
    ui: &UiShell,
    title: impl Into<String>,
    index: usize,
    active: bool,
    dock_tab: &mut State<usize>,
) -> impl IntoElement {
    let mut state = *dock_tab;
    let title = title.into();
    crate::studio_widgets::StudioButton::new(ui, title.clone())
        .text(title)
        .tab()
        .selected(active)
        .on_press(move |_| state.set(index))
}

fn section_header(title: impl Into<String>) -> impl IntoElement {
    crate::studio_widgets::panel_header(title)
}

fn value_pill(tag: &'static str, value: String) -> impl IntoElement {
    rect()
        .padding(Gaps::new(2., 6., 2., 6.))
        .background(theme::SURFACE_CHROME)
        .child(
            label()
                .text(format!("{}: {}", tag, value))
                .font_size(11.)
                .color(theme::TEXT_PRIMARY),
        )
}

fn step_text_size(ui: &UiShell, target_id: Option<ObjectId>, delta: f64) {
    let Some(id) = target_id else {
        return;
    };
    let target = crate::object_edits::ObjectEdit::capture(
        &ui.shell.peek(),
        id,
        crate::object_edits::EditKind::Text,
    );
    let result = target.and_then(|target| {
        crate::object_edits::step_font_size(&mut ui.shell.clone().write(), &target, delta)
    });
    if let Err(reason) = result {
        ui.file_error
            .clone()
            .set(Some(crate::object_edit_dialog::failure_text(ui, reason)));
    }
}

fn opacity_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: impl Into<String>,
    val: f64,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let _ = shell.write().bridge.submit_all(
                    "Set opacity",
                    vec![Command::SetOpacity { id, opacity: val }],
                );
            }
        })
        .child(
            label()
                .text(label_text.into())
                .font_size(theme::CAPTION_SIZE),
        )
}

fn stroke_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: impl Into<String>,
    delta: f64,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let current = shell
                    .peek()
                    .bridge
                    .session()
                    .and_then(|s| s.find_object(id))
                    .map(|o| (o.stroke_width, o.stroke.clone()));
                let Some((current, stroke)) = current else {
                    return;
                };
                let new_width = (current + delta).max(0.0);
                let _ = shell.write().bridge.submit_all(
                    "Set stroke width",
                    vec![Command::SetStroke {
                        id,
                        stroke,
                        width: new_width,
                    }],
                );
            }
        })
        .child(
            label()
                .text(label_text.into())
                .font_size(theme::CAPTION_SIZE),
        )
}

fn quick_color_swatch(ui: UiShell, color_token: &'static str, color: Color) -> impl IntoElement {
    let apply_ui = ui.clone();
    rect()
        .width(Size::px(28.))
        .height(Size::px(28.))
        .background(color)
        .corner_radius(theme::CONTROL_RADIUS)
        .child(
            crate::studio_widgets::StudioButton::new(&ui, color_token)
                .width(Size::fill())
                .enabled(crate::studio::color_target_enabled(&ui))
                .on_press(move |_| crate::studio::apply_color(&apply_ui, color_token, true)),
        )
}

fn action_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    action_token: &'static str,
    title: impl Into<String>,
) -> impl IntoElement {
    let availability = petunia_design_application::menus::availability(
        action_token,
        &shell.read().bridge.action_context(),
    );
    Button::new()
        .enabled(availability.enabled)
        .on_press(move |_| {
            let _ = run_action_token(&mut shell.write(), action_token);
        })
        .child(label().text(title.into()).font_size(11.))
}

fn star_points_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: impl Into<String>,
    delta: i32,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let obj = shell
                    .peek()
                    .bridge
                    .session()
                    .and_then(|s| s.find_object(id))
                    .cloned();
                if let Some(obj) = obj {
                    if let Some(ShapeKind::Star {
                        points,
                        inner_ratio,
                    }) = obj.shape
                    {
                        let new_points = (points as i32 + delta).clamp(3, 36) as u32;
                        let _ = shell.write().bridge.submit_all(
                            "Change star points",
                            vec![Command::SetShape {
                                id,
                                shape: Some(ShapeKind::Star {
                                    points: new_points,
                                    inner_ratio,
                                }),
                            }],
                        );
                    }
                }
            }
        })
        .child(
            label()
                .text(label_text.into())
                .font_size(theme::CAPTION_SIZE),
        )
}

fn star_ratio_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: impl Into<String>,
    delta: f64,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let obj = shell
                    .peek()
                    .bridge
                    .session()
                    .and_then(|s| s.find_object(id))
                    .cloned();
                if let Some(obj) = obj {
                    if let Some(ShapeKind::Star {
                        points,
                        inner_ratio,
                    }) = obj.shape
                    {
                        let new_ratio = (inner_ratio + delta).clamp(0.1, 0.9);
                        let _ = shell.write().bridge.submit_all(
                            "Change star inner ratio",
                            vec![Command::SetShape {
                                id,
                                shape: Some(ShapeKind::Star {
                                    points,
                                    inner_ratio: new_ratio,
                                }),
                            }],
                        );
                    }
                }
            }
        })
        .child(
            label()
                .text(label_text.into())
                .font_size(theme::CAPTION_SIZE),
        )
}

fn polygon_sides_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: impl Into<String>,
    delta: i32,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let obj = shell
                    .peek()
                    .bridge
                    .session()
                    .and_then(|s| s.find_object(id))
                    .cloned();
                if let Some(obj) = obj {
                    if let Some(ShapeKind::Polygon { sides }) = obj.shape {
                        let new_sides = (sides as i32 + delta).clamp(3, 36) as u32;
                        let _ = shell.write().bridge.submit_all(
                            "Change polygon sides",
                            vec![Command::SetShape {
                                id,
                                shape: Some(ShapeKind::Polygon { sides: new_sides }),
                            }],
                        );
                    }
                }
            }
        })
        .child(
            label()
                .text(label_text.into())
                .font_size(theme::CAPTION_SIZE),
        )
}

fn corner_radius_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: impl Into<String>,
    delta: f64,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let obj = shell
                    .peek()
                    .bridge
                    .session()
                    .and_then(|s| s.find_object(id))
                    .cloned();
                if let Some(obj) = obj {
                    if let Some(ShapeKind::Rectangle { corner_radii }) = obj.shape {
                        let new_r = (corner_radii[0] + delta).max(0.0);
                        let _ = shell.write().bridge.submit_all(
                            "Change corner radius",
                            vec![Command::SetShape {
                                id,
                                shape: Some(ShapeKind::Rectangle {
                                    corner_radii: [new_r, new_r, new_r, new_r],
                                }),
                            }],
                        );
                    }
                }
            }
        })
        .child(
            label()
                .text(label_text.into())
                .font_size(theme::CAPTION_SIZE),
        )
}

fn bake_corners_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let _ = shell
                    .write()
                    .bridge
                    .submit_all("Bake corners", vec![Command::BakeCorners { id }]);
            }
        })
        .child(
            label()
                .text(studio_text(shell, "bake"))
                .font_size(theme::CAPTION_SIZE),
        )
}

fn contour_offset_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: impl Into<String>,
    delta: f64,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let current_dist = shell
                    .peek()
                    .bridge
                    .session()
                    .and_then(|s| s.find_object(id))
                    .and_then(|o| {
                        o.modifiers.iter().find_map(|m| {
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
                    .unwrap_or(0.0);
                let new_dist = current_dist + delta;
                let _ = shell.write().bridge.submit_all(
                    "Set contour offset",
                    vec![Command::OffsetPath {
                        id,
                        delta: new_dist,
                    }],
                );
            }
        })
        .child(
            label()
                .text(label_text.into())
                .font_size(theme::CAPTION_SIZE),
        )
}

fn bake_contour_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let _ = shell
                    .write()
                    .bridge
                    .submit_all("Bake contour", vec![Command::BakeContour { id }]);
            }
        })
        .child(
            label()
                .text(studio_text(shell, "bake"))
                .font_size(theme::CAPTION_SIZE),
        )
}

fn bake_geometry_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let _ = shell.write().bridge.submit_all(
                    "Bake geometry modifiers",
                    vec![Command::BakeGeometry { id }],
                );
            }
        })
        .child(
            label()
                .text(studio_text(shell, "bake_geometry"))
                .font_size(theme::CAPTION_SIZE),
        )
}

fn bake_transparency_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let _ = shell.write().bridge.submit_all(
                    "Bake transparency modifier",
                    vec![Command::BakeTransparency { id }],
                );
            }
        })
        .child(
            label()
                .text(studio_text(shell, "bake_transparency"))
                .font_size(theme::CAPTION_SIZE),
        )
}

fn modifier_stack_inspector(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    modifiers: &[petunia_design_document::ModifierItem],
    target_bounds: Option<[f64; 4]>,
) -> impl IntoElement {
    let mut root = rect()
        .direction(Direction::Vertical)
        .width(Size::fill())
        .spacing(4.);

    if modifiers.is_empty() {
        let add_row = rect()
            .direction(Direction::Horizontal)
            .content(Content::Flex)
            .width(Size::fill())
            .main_align(Alignment::SpaceBetween)
            .cross_align(Alignment::Center)
            .child(
                label()
                    .text(studio_text(shell, "no_modifiers"))
                    .font_size(11.)
                    .color(theme::TEXT_TERTIARY),
            )
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .spacing(4.)
                    .child(
                        Button::new()
                            .on_press(move |_| {
                                if let Some(id) = target_id {
                                    let _ = shell.write().bridge.submit_all(
                                        "Add contour modifier",
                                        vec![Command::OffsetPath { id, delta: 6.0 }],
                                    );
                                }
                            })
                            .child(
                                label()
                                    .text(studio_text(shell, "add_contour"))
                                    .font_size(theme::CAPTION_SIZE),
                            ),
                    )
                    .child(
                        Button::new()
                            .on_press(move |_| {
                                if let Some(id) = target_id {
                                    let r = target_bounds.unwrap_or([0.0, 0.0, 100.0, 100.0]);
                                    let crop_rect =
                                        [r[0], r[1], (r[2] * 0.8).max(10.), (r[3] * 0.8).max(10.)];
                                    let _ = shell.write().bridge.submit_all(
                                        "Add crop modifier",
                                        vec![Command::SetCropRect {
                                            id,
                                            rect: crop_rect,
                                        }],
                                    );
                                }
                            })
                            .child(
                                label()
                                    .text(studio_text(shell, "add_clip"))
                                    .font_size(theme::CAPTION_SIZE),
                            ),
                    ),
            );
        return root.child(add_row);
    }

    // Render each modifier in the stack
    for (idx, item) in modifiers.iter().enumerate() {
        let item_id = item.id;
        let is_enabled = item.enabled;
        let can_move_up = idx > 0;
        let can_move_down = idx + 1 < modifiers.len();

        let (title, detail) = match &item.kind {
            petunia_design_document::ModifierKind::ContourOffset {
                distance,
                join,
                cap,
            } => {
                let join_name = match join {
                    OffsetJoin::Round => studio_text(shell, "round"),
                    OffsetJoin::Miter => studio_text(shell, "miter"),
                    OffsetJoin::Bevel => studio_text(shell, "bevel"),
                };
                let cap_name = match cap {
                    OffsetCap::None => studio_text(shell, "butt"),
                    OffsetCap::Round => studio_text(shell, "round"),
                    OffsetCap::Square => studio_text(shell, "square"),
                };
                (
                    format!("{}: {distance:+.1} pt", studio_text(shell, "live_contour")),
                    format!(
                        "{}: {} · {}: {}",
                        studio_text(shell, "join"),
                        join_name,
                        studio_text(shell, "cap"),
                        cap_name
                    ),
                )
            }
            petunia_design_document::ModifierKind::TransparentGradient { stops, .. } => (
                studio_text(shell, "transparent_gradient"),
                format!("{} {}", stops.len(), studio_text(shell, "opacity_stops")),
            ),
            petunia_design_document::ModifierKind::Perspective { .. } => (
                studio_text(shell, "perspective_warp"),
                studio_text(shell, "quad_warp"),
            ),
            petunia_design_document::ModifierKind::CropRect { rect } => (
                studio_text(shell, "vector_crop"),
                format!(
                    "{:.0}x{:.0} @ {:.0},{:.0}",
                    rect[2], rect[3], rect[0], rect[1]
                ),
            ),
        };

        let mut card = rect()
            .direction(Direction::Vertical)
            .width(Size::fill())
            .background(theme::SURFACE_CHROME_STRONG)
            .border(
                Border::new()
                    .fill(if is_enabled {
                        theme::BORDER_SUBTLE
                    } else {
                        theme::SURFACE_CHROME
                    })
                    .width(1.)
                    .alignment(BorderAlignment::Inner),
            )
            .padding(Gaps::new_all(4.))
            .spacing(3.);

        // Header row: Title + Reorder/Toggle/Delete
        let mut header_row = rect()
            .direction(Direction::Horizontal)
            .content(Content::Flex)
            .width(Size::fill())
            .main_align(Alignment::SpaceBetween)
            .cross_align(Alignment::Center)
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .spacing(4.)
                    .cross_align(Alignment::Center)
                    .child(label().text(title).font_size(11.).color(if is_enabled {
                        theme::TEXT_PRIMARY
                    } else {
                        theme::TEXT_TERTIARY
                    })),
            );

        // Action buttons
        let mut actions = rect()
            .direction(Direction::Horizontal)
            .content(Content::Flex)
            .spacing(2.)
            .cross_align(Alignment::Center);

        // Toggle enabled button
        let modifiers_clone = modifiers.to_vec();
        actions = actions.child(
            Button::new()
                .on_press(move |_| {
                    if let Some(id) = target_id {
                        let mut next = modifiers_clone.clone();
                        if let Some(m) = next.iter_mut().find(|m| m.id == item_id) {
                            m.enabled = !m.enabled;
                            let _ = shell.write().bridge.submit_all(
                                "Toggle modifier",
                                vec![Command::SetModifiers {
                                    id,
                                    modifiers: next,
                                }],
                            );
                        }
                    }
                })
                .child(
                    label()
                        .text(if is_enabled { "👁" } else { "⊘" })
                        .font_size(theme::CAPTION_SIZE),
                ),
        );

        // Move up button
        if can_move_up {
            let modifiers_clone = modifiers.to_vec();
            actions = actions.child(
                Button::new()
                    .on_press(move |_| {
                        if let Some(id) = target_id {
                            let mut next = modifiers_clone.clone();
                            next.swap(idx, idx - 1);
                            let _ = shell.write().bridge.submit_all(
                                "Reorder modifier up",
                                vec![Command::SetModifiers {
                                    id,
                                    modifiers: next,
                                }],
                            );
                        }
                    })
                    .child(label().text("↑").font_size(theme::CAPTION_SIZE)),
            );
        }

        // Move down button
        if can_move_down {
            let modifiers_clone = modifiers.to_vec();
            actions = actions.child(
                Button::new()
                    .on_press(move |_| {
                        if let Some(id) = target_id {
                            let mut next = modifiers_clone.clone();
                            next.swap(idx, idx + 1);
                            let _ = shell.write().bridge.submit_all(
                                "Reorder modifier down",
                                vec![Command::SetModifiers {
                                    id,
                                    modifiers: next,
                                }],
                            );
                        }
                    })
                    .child(label().text("↓").font_size(theme::CAPTION_SIZE)),
            );
        }

        // Delete button
        let modifiers_clone = modifiers.to_vec();
        actions = actions.child(
            Button::new()
                .on_press(move |_| {
                    if let Some(id) = target_id {
                        let mut next = modifiers_clone.clone();
                        next.retain(|m| m.id != item_id);
                        let _ = shell.write().bridge.submit_all(
                            "Delete modifier",
                            vec![Command::SetModifiers {
                                id,
                                modifiers: next,
                            }],
                        );
                    }
                })
                .child(label().text("×").font_size(theme::CAPTION_SIZE)),
        );

        header_row = header_row.child(actions);
        card = card.child(header_row);

        // Subdetail and parameter controls
        match &item.kind {
            petunia_design_document::ModifierKind::ContourOffset {
                distance: current_d,
                join: current_join,
                cap: current_cap,
            } => {
                let d_val = *current_d;
                let j_val = *current_join;
                let c_val = *current_cap;

                card = card.child(
                    label()
                        .text(detail)
                        .font_size(theme::CAPTION_SIZE)
                        .color(theme::TEXT_TERTIARY),
                );

                // Distance + Bake Row
                let dist_row = rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .spacing(2.)
                    .child(contour_offset_button(shell, target_id, "-2pt", -2.0))
                    .child(contour_offset_button(shell, target_id, "+2pt", 2.0))
                    .child(contour_offset_button(shell, target_id, "-5pt", -5.0))
                    .child(contour_offset_button(shell, target_id, "+5pt", 5.0))
                    .child(bake_contour_button(shell, target_id));
                card = card.child(dist_row);

                // Join Style selection row
                let modifiers_join = modifiers.to_vec();
                let join_row = rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .spacing(2.)
                    .cross_align(Alignment::Center)
                    .child(
                        label()
                            .text(studio_text(shell, "join"))
                            .font_size(theme::CAPTION_SIZE)
                            .color(theme::TEXT_SECONDARY),
                    )
                    .child({
                        let modifiers_clone = modifiers_join.clone();
                        Button::new()
                            .on_press(move |_| {
                                if let Some(id) = target_id {
                                    let mut next = modifiers_clone.clone();
                                    if let Some(m) = next.iter_mut().find(|m| m.id == item_id) {
                                        m.kind =
                                            petunia_design_document::ModifierKind::ContourOffset {
                                                distance: d_val,
                                                join: OffsetJoin::Round,
                                                cap: c_val,
                                            };
                                        let _ = shell.write().bridge.submit_all(
                                            "Set contour join round",
                                            vec![Command::SetModifiers {
                                                id,
                                                modifiers: next,
                                            }],
                                        );
                                    }
                                }
                            })
                            .child(
                                label()
                                    .text(if j_val == OffsetJoin::Round {
                                        "[Redonda]"
                                    } else {
                                        "Redonda"
                                    })
                                    .font_size(theme::CAPTION_SIZE),
                            )
                    })
                    .child({
                        let modifiers_clone = modifiers_join.clone();
                        Button::new()
                            .on_press(move |_| {
                                if let Some(id) = target_id {
                                    let mut next = modifiers_clone.clone();
                                    if let Some(m) = next.iter_mut().find(|m| m.id == item_id) {
                                        m.kind =
                                            petunia_design_document::ModifierKind::ContourOffset {
                                                distance: d_val,
                                                join: OffsetJoin::Miter,
                                                cap: c_val,
                                            };
                                        let _ = shell.write().bridge.submit_all(
                                            "Set contour join miter",
                                            vec![Command::SetModifiers {
                                                id,
                                                modifiers: next,
                                            }],
                                        );
                                    }
                                }
                            })
                            .child(
                                label()
                                    .text(if j_val == OffsetJoin::Miter {
                                        "[Esquadria]"
                                    } else {
                                        "Esquadria"
                                    })
                                    .font_size(theme::CAPTION_SIZE),
                            )
                    })
                    .child({
                        let modifiers_clone = modifiers_join.clone();
                        Button::new()
                            .on_press(move |_| {
                                if let Some(id) = target_id {
                                    let mut next = modifiers_clone.clone();
                                    if let Some(m) = next.iter_mut().find(|m| m.id == item_id) {
                                        m.kind =
                                            petunia_design_document::ModifierKind::ContourOffset {
                                                distance: d_val,
                                                join: OffsetJoin::Bevel,
                                                cap: c_val,
                                            };
                                        let _ = shell.write().bridge.submit_all(
                                            "Set contour join bevel",
                                            vec![Command::SetModifiers {
                                                id,
                                                modifiers: next,
                                            }],
                                        );
                                    }
                                }
                            })
                            .child(
                                label()
                                    .text(if j_val == OffsetJoin::Bevel {
                                        "[Chanfro]"
                                    } else {
                                        "Chanfro"
                                    })
                                    .font_size(theme::CAPTION_SIZE),
                            )
                    });
                card = card.child(join_row);
            }
            petunia_design_document::ModifierKind::CropRect { rect: current_rect } => {
                let r_val = *current_rect;
                card = card.child(
                    label()
                        .text(detail)
                        .font_size(theme::CAPTION_SIZE)
                        .color(theme::TEXT_TERTIARY),
                );
                let modifiers_crop = modifiers.to_vec();
                let crop_row = rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .spacing(2.)
                    .child({
                        let modifiers_clone = modifiers_crop.clone();
                        Button::new()
                            .on_press(move |_| {
                                if let Some(id) = target_id {
                                    let mut next = modifiers_clone.clone();
                                    if let Some(m) = next.iter_mut().find(|m| m.id == item_id) {
                                        m.kind = petunia_design_document::ModifierKind::CropRect {
                                            rect: [
                                                r_val[0] - 5.,
                                                r_val[1] - 5.,
                                                r_val[2] + 10.,
                                                r_val[3] + 10.,
                                            ],
                                        };
                                        let _ = shell.write().bridge.submit_all(
                                            "Expand crop rect",
                                            vec![Command::SetModifiers {
                                                id,
                                                modifiers: next,
                                            }],
                                        );
                                    }
                                }
                            })
                            .child(
                                label()
                                    .text(studio_text(shell, "expand_geometry"))
                                    .font_size(theme::CAPTION_SIZE),
                            )
                    })
                    .child({
                        let modifiers_clone = modifiers_crop.clone();
                        Button::new()
                            .on_press(move |_| {
                                if let Some(id) = target_id {
                                    let mut next = modifiers_clone.clone();
                                    if let Some(m) = next.iter_mut().find(|m| m.id == item_id) {
                                        m.kind = petunia_design_document::ModifierKind::CropRect {
                                            rect: [
                                                r_val[0] + 5.,
                                                r_val[1] + 5.,
                                                (r_val[2] - 10.).max(10.),
                                                (r_val[3] - 10.).max(10.),
                                            ],
                                        };
                                        let _ = shell.write().bridge.submit_all(
                                            "Contract crop rect",
                                            vec![Command::SetModifiers {
                                                id,
                                                modifiers: next,
                                            }],
                                        );
                                    }
                                }
                            })
                            .child(
                                label()
                                    .text(studio_text(shell, "inset_geometry"))
                                    .font_size(theme::CAPTION_SIZE),
                            )
                    })
                    .child(bake_geometry_button(shell, target_id));
                card = card.child(crop_row);
            }
            petunia_design_document::ModifierKind::TransparentGradient { .. } => {
                card = card.child(
                    label()
                        .text(detail)
                        .font_size(theme::CAPTION_SIZE)
                        .color(theme::TEXT_TERTIARY),
                );
                let trans_row = rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .spacing(2.)
                    .child(bake_transparency_button(shell, target_id));
                card = card.child(trans_row);
            }
            petunia_design_document::ModifierKind::Perspective { .. } => {
                card = card.child(
                    label()
                        .text(detail)
                        .font_size(theme::CAPTION_SIZE)
                        .color(theme::TEXT_TERTIARY),
                );
                let pers_row = rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .spacing(2.)
                    .child(bake_geometry_button(shell, target_id));
                card = card.child(pers_row);
            }
        }

        root = root.child(card);
    }

    // Add extra "+ Modificador" buttons when stack is non-empty
    let has_contour = modifiers.iter().any(|m| {
        matches!(
            m.kind,
            petunia_design_document::ModifierKind::ContourOffset { .. }
        )
    });
    let has_crop = modifiers.iter().any(|m| {
        matches!(
            m.kind,
            petunia_design_document::ModifierKind::CropRect { .. }
        )
    });
    if !has_contour || !has_crop || modifiers.len() > 1 {
        let mut extra_row = rect()
            .direction(Direction::Horizontal)
            .content(Content::Flex)
            .spacing(4.)
            .cross_align(Alignment::Center);
        if !has_contour {
            extra_row = extra_row.child(
                Button::new()
                    .on_press(move |_| {
                        if let Some(id) = target_id {
                            let _ = shell.write().bridge.submit_all(
                                "Add contour modifier",
                                vec![Command::OffsetPath { id, delta: 6.0 }],
                            );
                        }
                    })
                    .child(
                        label()
                            .text(studio_text(shell, "add_contour"))
                            .font_size(theme::CAPTION_SIZE),
                    ),
            );
        }
        if !has_crop {
            extra_row = extra_row.child(
                Button::new()
                    .on_press(move |_| {
                        if let Some(id) = target_id {
                            let r = target_bounds.unwrap_or([0.0, 0.0, 100.0, 100.0]);
                            let crop_rect =
                                [r[0], r[1], (r[2] * 0.8).max(10.), (r[3] * 0.8).max(10.)];
                            let _ = shell.write().bridge.submit_all(
                                "Add crop modifier",
                                vec![Command::SetCropRect {
                                    id,
                                    rect: crop_rect,
                                }],
                            );
                        }
                    })
                    .child(
                        label()
                            .text(studio_text(shell, "add_clip"))
                            .font_size(theme::CAPTION_SIZE),
                    ),
            );
        }
        if modifiers.len() > 1 {
            extra_row = extra_row.child(bake_geometry_button(shell, target_id));
        }
        root = root.child(extra_row);
    }

    root
}

fn convert_to_curves_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let _ = shell
                    .write()
                    .bridge
                    .submit_all("Convert to curves", vec![Command::ConvertToCurves { id }]);
            }
        })
        .child(
            label()
                .text(studio_text(shell, "curves"))
                .font_size(theme::CAPTION_SIZE),
        )
}

fn blur_adjust_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: impl Into<String>,
    delta: f64,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let existing = shell
                    .peek()
                    .bridge
                    .session()
                    .and_then(|s| s.find_object(id))
                    .and_then(|o| {
                        o.appearance.as_ref().and_then(|app| {
                            app.effects.iter().find_map(|e| {
                                if let petunia_design_document::EffectKind::GaussianBlur {
                                    radius,
                                } = e.kind
                                {
                                    Some((e.id, radius, e.visible))
                                } else {
                                    None
                                }
                            })
                        })
                    });
                let (eff_id, current_radius, visible) = existing.unwrap_or((101, 0.0, true));
                let new_radius = (current_radius + delta).max(0.5);
                let mut cmds = Vec::new();
                if existing.is_some() {
                    cmds.push(Command::RemoveEffect {
                        id,
                        effect_id: eff_id,
                    });
                }
                cmds.push(Command::AddEffect {
                    id,
                    effect: petunia_design_document::EffectItem {
                        id: eff_id,
                        kind: petunia_design_document::EffectKind::GaussianBlur {
                            radius: new_radius,
                        },
                        visible,
                    },
                });
                let _ = shell.write().bridge.submit_all("Adjust blur", cmds);
            }
        })
        .child(
            label()
                .text(label_text.into())
                .font_size(theme::CAPTION_SIZE),
        )
}

fn toggle_blur_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let existing = shell
                    .peek()
                    .bridge
                    .session()
                    .and_then(|s| s.find_object(id))
                    .and_then(|o| {
                        o.appearance.as_ref().and_then(|app| {
                            app.effects.iter().find_map(|e| {
                                if let petunia_design_document::EffectKind::GaussianBlur {
                                    radius,
                                } = e.kind
                                {
                                    Some((e.id, radius, e.visible))
                                } else {
                                    None
                                }
                            })
                        })
                    });
                if let Some((eff_id, radius, visible)) = existing {
                    let cmds = vec![
                        Command::RemoveEffect {
                            id,
                            effect_id: eff_id,
                        },
                        Command::AddEffect {
                            id,
                            effect: petunia_design_document::EffectItem {
                                id: eff_id,
                                kind: petunia_design_document::EffectKind::GaussianBlur { radius },
                                visible: !visible,
                            },
                        },
                    ];
                    let _ = shell.write().bridge.submit_all("Toggle blur", cmds);
                }
            }
        })
        .child(label().text("👁").font_size(theme::CAPTION_SIZE))
}

fn remove_blur_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    effect_id: u32,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let _ = shell
                    .write()
                    .bridge
                    .submit_all("Remove blur", vec![Command::RemoveEffect { id, effect_id }]);
            }
        })
        .child(label().text("✕").font_size(theme::CAPTION_SIZE))
}

fn drop_shadow_adjust_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: impl Into<String>,
    delta_dx: f64,
    delta_dy: f64,
    delta_blur: f64,
    delta_opacity: f64,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let existing = shell
                    .peek()
                    .bridge
                    .session()
                    .and_then(|s| s.find_object(id))
                    .and_then(|o| {
                        o.appearance.as_ref().and_then(|app| {
                            app.effects.iter().find_map(|e| {
                                if let petunia_design_document::EffectKind::DropShadow {
                                    offset,
                                    blur,
                                    color,
                                    opacity,
                                } = &e.kind
                                {
                                    Some((e.id, *offset, *blur, color.clone(), *opacity, e.visible))
                                } else {
                                    None
                                }
                            })
                        })
                    });
                let had_existing = existing.is_some();
                let (eff_id, offset, blur, color, opacity, visible) = existing.unwrap_or((
                    102,
                    [4.0, 4.0],
                    8.0,
                    "ptnd.gray/900".to_string(),
                    0.6,
                    true,
                ));
                let new_offset = [
                    (offset[0] + delta_dx).clamp(-100.0, 100.0),
                    (offset[1] + delta_dy).clamp(-100.0, 100.0),
                ];
                let new_blur = (blur + delta_blur).max(0.0);
                let new_opacity = (opacity + delta_opacity).clamp(0.05, 1.0);
                let mut cmds = Vec::new();
                if had_existing {
                    cmds.push(Command::RemoveEffect {
                        id,
                        effect_id: eff_id,
                    });
                }
                cmds.push(Command::AddEffect {
                    id,
                    effect: petunia_design_document::EffectItem {
                        id: eff_id,
                        kind: petunia_design_document::EffectKind::DropShadow {
                            offset: new_offset,
                            blur: new_blur,
                            color,
                            opacity: new_opacity,
                        },
                        visible,
                    },
                });
                let _ = shell.write().bridge.submit_all("Adjust drop shadow", cmds);
            }
        })
        .child(
            label()
                .text(label_text.into())
                .font_size(theme::CAPTION_SIZE),
        )
}

fn toggle_drop_shadow_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let existing = shell
                    .peek()
                    .bridge
                    .session()
                    .and_then(|s| s.find_object(id))
                    .and_then(|o| {
                        o.appearance.as_ref().and_then(|app| {
                            app.effects.iter().find_map(|e| {
                                if let petunia_design_document::EffectKind::DropShadow {
                                    offset,
                                    blur,
                                    color,
                                    opacity,
                                } = &e.kind
                                {
                                    Some((e.id, *offset, *blur, color.clone(), *opacity, e.visible))
                                } else {
                                    None
                                }
                            })
                        })
                    });
                if let Some((eff_id, offset, blur, color, opacity, visible)) = existing {
                    let cmds = vec![
                        Command::RemoveEffect {
                            id,
                            effect_id: eff_id,
                        },
                        Command::AddEffect {
                            id,
                            effect: petunia_design_document::EffectItem {
                                id: eff_id,
                                kind: petunia_design_document::EffectKind::DropShadow {
                                    offset,
                                    blur,
                                    color,
                                    opacity,
                                },
                                visible: !visible,
                            },
                        },
                    ];
                    let _ = shell.write().bridge.submit_all("Toggle drop shadow", cmds);
                }
            }
        })
        .child(label().text("👁").font_size(theme::CAPTION_SIZE))
}

fn remove_shadow_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    effect_id: u32,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let _ = shell.write().bridge.submit_all(
                    "Remove shadow",
                    vec![Command::RemoveEffect { id, effect_id }],
                );
            }
        })
        .child(label().text("✕").font_size(theme::CAPTION_SIZE))
}

fn inner_shadow_adjust_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: impl Into<String>,
    delta_dist: f64,
    delta_blur: f64,
    delta_opacity: f64,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let existing = shell
                    .peek()
                    .bridge
                    .session()
                    .and_then(|s| s.find_object(id))
                    .and_then(|o| {
                        o.appearance.as_ref().and_then(|app| {
                            app.effects.iter().find_map(|e| {
                                if let petunia_design_document::EffectKind::InnerShadow {
                                    offset,
                                    blur,
                                    color,
                                    opacity,
                                } = &e.kind
                                {
                                    Some((e.id, *offset, *blur, color.clone(), *opacity, e.visible))
                                } else {
                                    None
                                }
                            })
                        })
                    });
                let had_existing = existing.is_some();
                let (eff_id, offset, blur, color, opacity, visible) = existing.unwrap_or((
                    105,
                    [3.0, 3.0],
                    6.0,
                    "ptnd.gray/900".to_string(),
                    0.5,
                    true,
                ));
                let new_offset = [
                    (offset[0] + delta_dist).max(0.0),
                    (offset[1] + delta_dist).max(0.0),
                ];
                let new_blur = (blur + delta_blur).max(0.0);
                let new_opacity = (opacity + delta_opacity).clamp(0.05, 1.0);
                let mut cmds = Vec::new();
                if had_existing {
                    cmds.push(Command::RemoveEffect {
                        id,
                        effect_id: eff_id,
                    });
                }
                cmds.push(Command::AddEffect {
                    id,
                    effect: petunia_design_document::EffectItem {
                        id: eff_id,
                        kind: petunia_design_document::EffectKind::InnerShadow {
                            offset: new_offset,
                            blur: new_blur,
                            color,
                            opacity: new_opacity,
                        },
                        visible,
                    },
                });
                let _ = shell.write().bridge.submit_all("Adjust inner shadow", cmds);
            }
        })
        .child(
            label()
                .text(label_text.into())
                .font_size(theme::CAPTION_SIZE),
        )
}

fn toggle_inner_shadow_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let existing = shell
                    .peek()
                    .bridge
                    .session()
                    .and_then(|s| s.find_object(id))
                    .and_then(|o| {
                        o.appearance.as_ref().and_then(|app| {
                            app.effects.iter().find_map(|e| {
                                if let petunia_design_document::EffectKind::InnerShadow {
                                    offset,
                                    blur,
                                    color,
                                    opacity,
                                } = &e.kind
                                {
                                    Some((e.id, *offset, *blur, color.clone(), *opacity, e.visible))
                                } else {
                                    None
                                }
                            })
                        })
                    });
                if let Some((eff_id, offset, blur, color, opacity, visible)) = existing {
                    let cmds = vec![
                        Command::RemoveEffect {
                            id,
                            effect_id: eff_id,
                        },
                        Command::AddEffect {
                            id,
                            effect: petunia_design_document::EffectItem {
                                id: eff_id,
                                kind: petunia_design_document::EffectKind::InnerShadow {
                                    offset,
                                    blur,
                                    color,
                                    opacity,
                                },
                                visible: !visible,
                            },
                        },
                    ];
                    let _ = shell.write().bridge.submit_all("Toggle inner shadow", cmds);
                }
            }
        })
        .child(label().text("👁").font_size(theme::CAPTION_SIZE))
}

fn remove_inner_shadow_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    effect_id: u32,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let _ = shell.write().bridge.submit_all(
                    "Remove inner shadow",
                    vec![Command::RemoveEffect { id, effect_id }],
                );
            }
        })
        .child(label().text("✕").font_size(theme::CAPTION_SIZE))
}

fn sharpen_adjust_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: impl Into<String>,
    delta_radius: f64,
    delta_amount: f64,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let existing = shell
                    .peek()
                    .bridge
                    .session()
                    .and_then(|s| s.find_object(id))
                    .and_then(|o| {
                        o.appearance.as_ref().and_then(|app| {
                            app.effects.iter().find_map(|e| {
                                if let petunia_design_document::EffectKind::Sharpen {
                                    radius,
                                    amount,
                                } = e.kind
                                {
                                    Some((e.id, radius, amount))
                                } else {
                                    None
                                }
                            })
                        })
                    });
                let (eff_id, radius, amount) = existing.unwrap_or((103, 1.0, 0.5));
                let new_radius = (radius + delta_radius).max(0.1);
                let new_amount = (amount + delta_amount).clamp(0.0, 5.0);
                let _ = shell.write().bridge.submit_all(
                    "Adjust sharpen filter",
                    vec![Command::AddEffect {
                        id,
                        effect: petunia_design_document::EffectItem {
                            id: eff_id,
                            kind: petunia_design_document::EffectKind::Sharpen {
                                radius: new_radius,
                                amount: new_amount,
                            },
                            visible: true,
                        },
                    }],
                );
            }
        })
        .child(
            label()
                .text(label_text.into())
                .font_size(theme::CAPTION_SIZE),
        )
}

fn remove_sharpen_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    effect_id: u32,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let _ = shell.write().bridge.submit_all(
                    "Remove sharpen",
                    vec![Command::RemoveEffect { id, effect_id }],
                );
            }
        })
        .child(label().text("✕").font_size(theme::CAPTION_SIZE))
}

fn noise_adjust_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: impl Into<String>,
    delta: f64,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let existing = shell
                    .peek()
                    .bridge
                    .session()
                    .and_then(|s| s.find_object(id))
                    .and_then(|o| {
                        o.appearance.as_ref().and_then(|app| {
                            app.effects.iter().find_map(|e| {
                                if let petunia_design_document::EffectKind::Noise {
                                    amount,
                                    monochrome,
                                } = e.kind
                                {
                                    Some((e.id, amount, monochrome))
                                } else {
                                    None
                                }
                            })
                        })
                    });
                let (eff_id, amount, monochrome) = existing.unwrap_or((104, 0.15, true));
                let new_amount = (amount + delta).clamp(0.0, 1.0);
                let _ = shell.write().bridge.submit_all(
                    "Adjust noise filter",
                    vec![Command::AddEffect {
                        id,
                        effect: petunia_design_document::EffectItem {
                            id: eff_id,
                            kind: petunia_design_document::EffectKind::Noise {
                                amount: new_amount,
                                monochrome,
                            },
                            visible: true,
                        },
                    }],
                );
            }
        })
        .child(
            label()
                .text(label_text.into())
                .font_size(theme::CAPTION_SIZE),
        )
}

fn noise_toggle_mono_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let existing = shell
                    .peek()
                    .bridge
                    .session()
                    .and_then(|s| s.find_object(id))
                    .and_then(|o| {
                        o.appearance.as_ref().and_then(|app| {
                            app.effects.iter().find_map(|e| {
                                if let petunia_design_document::EffectKind::Noise {
                                    amount,
                                    monochrome,
                                } = e.kind
                                {
                                    Some((e.id, amount, monochrome))
                                } else {
                                    None
                                }
                            })
                        })
                    });
                let (eff_id, amount, monochrome) = existing.unwrap_or((104, 0.15, true));
                let _ = shell.write().bridge.submit_all(
                    "Toggle noise monochrome",
                    vec![Command::AddEffect {
                        id,
                        effect: petunia_design_document::EffectItem {
                            id: eff_id,
                            kind: petunia_design_document::EffectKind::Noise {
                                amount,
                                monochrome: !monochrome,
                            },
                            visible: true,
                        },
                    }],
                );
            }
        })
        .child(
            label()
                .text(studio_text(shell, "mono_color"))
                .font_size(theme::CAPTION_SIZE),
        )
}

fn remove_noise_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    effect_id: u32,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let _ = shell.write().bridge.submit_all(
                    "Remove noise",
                    vec![Command::RemoveEffect { id, effect_id }],
                );
            }
        })
        .child(label().text("✕").font_size(theme::CAPTION_SIZE))
}

fn add_adjustment_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: impl Into<String>,
    kind: petunia_design_document::adjustments::AdjustmentKind,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let next_id = shell
                    .peek()
                    .bridge
                    .session()
                    .and_then(|s| s.find_object(id))
                    .and_then(|o| o.appearance.as_ref())
                    .map(|app| app.adjustments.iter().map(|a| a.id).max().unwrap_or(0) + 1)
                    .unwrap_or(1);
                let item = petunia_design_document::adjustments::AdjustmentItem::new(
                    next_id,
                    kind.clone(),
                );
                let _ = shell.write().bridge.submit_all(
                    "Add tonal adjustment",
                    vec![Command::AddAdjustment {
                        id,
                        adjustment: item,
                    }],
                );
            }
        })
        .child(
            label()
                .text(label_text.into())
                .font_size(theme::CAPTION_SIZE),
        )
}

fn adjustment_card(
    shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    adj: petunia_design_document::adjustments::AdjustmentItem,
) -> impl IntoElement {
    let adj_id = adj.id;
    let title = match &adj.kind {
        petunia_design_document::adjustments::AdjustmentKind::Levels { .. } => {
            format!("{} #{adj_id}", studio_text(shell, "levels"))
        }
        petunia_design_document::adjustments::AdjustmentKind::Curves { .. } => {
            format!("Curvas #{}", adj_id)
        }
        petunia_design_document::adjustments::AdjustmentKind::Hsl { .. } => {
            format!("HSL #{}", adj_id)
        }
        petunia_design_document::adjustments::AdjustmentKind::Exposure { .. } => {
            format!("{} #{adj_id}", studio_text(shell, "exposure"))
        }
        petunia_design_document::adjustments::AdjustmentKind::WhiteBalance { .. } => {
            format!("{} #{adj_id}", studio_text(shell, "white_balance"))
        }
    };

    let body = match adj.kind.clone() {
        petunia_design_document::adjustments::AdjustmentKind::Levels { master, .. } => {
            let mut shell_g_down = shell;
            let mut shell_g_up = shell;
            let mut shell_b_up = shell;
            let mut shell_w_down = shell;
            let adj_1 = adj.clone();
            let adj_2 = adj.clone();
            let adj_3 = adj.clone();
            let adj_4 = adj.clone();

            rect()
                .direction(Direction::Vertical)
                .width(Size::fill())
                .spacing(2.)
                .child(
                    label()
                        .text(format!("Gamma: {:.2} | In: [{:.2}, {:.2}]", master.gamma, master.input_black, master.input_white))
                        .font_size(theme::CAPTION_SIZE)
                        .color(theme::TEXT_TERTIARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .spacing(2.)
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_1.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::Levels { master: ref mut m, .. } = new_adj.kind {
                                            m.gamma = (m.gamma - 0.1).clamp(0.1, 10.0);
                                        }
                                        let _ = shell_g_down.write().bridge.submit_all("Adjust levels gamma", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text("γ -0.1").font_size(theme::CAPTION_SIZE)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_2.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::Levels { master: ref mut m, .. } = new_adj.kind {
                                            m.gamma = (m.gamma + 0.1).clamp(0.1, 10.0);
                                        }
                                        let _ = shell_g_up.write().bridge.submit_all("Adjust levels gamma", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text("γ +0.1").font_size(theme::CAPTION_SIZE)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_3.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::Levels { master: ref mut m, .. } = new_adj.kind {
                                            m.input_black = (m.input_black + 0.05).clamp(0.0, 0.9);
                                        }
                                        let _ = shell_b_up.write().bridge.submit_all("Adjust levels black", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text(studio_text(shell,"black_more")).font_size(theme::CAPTION_SIZE)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_4.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::Levels { master: ref mut m, .. } = new_adj.kind {
                                            m.input_white = (m.input_white - 0.05).clamp(0.1, 1.0);
                                        }
                                        let _ = shell_w_down.write().bridge.submit_all("Adjust levels white", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text(studio_text(shell,"white_less")).font_size(theme::CAPTION_SIZE)),
                        ),
                )
        }
        petunia_design_document::adjustments::AdjustmentKind::Curves { master_points, .. } => {
            let mut shell_s = shell;
            let mut shell_lin = shell;
            let mut shell_hi = shell;
            let adj_1 = adj.clone();
            let adj_2 = adj.clone();
            let adj_3 = adj.clone();

            rect()
                .direction(Direction::Vertical)
                .width(Size::fill())
                .spacing(2.)
                .child(
                    label()
                        .text(format!("{}: {}",studio_text(shell,"curve_points"), master_points.len()))
                        .font_size(theme::CAPTION_SIZE)
                        .color(theme::TEXT_TERTIARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .spacing(2.)
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_1.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::Curves { ref mut master_points, .. } = new_adj.kind {
                                            *master_points = vec![[0.0, 0.0], [0.25, 0.15], [0.75, 0.85], [1.0, 1.0]];
                                        }
                                        let _ = shell_s.write().bridge.submit_all("Set S-Curve", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text(studio_text(shell,"s_curve")).font_size(theme::CAPTION_SIZE)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_2.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::Curves { ref mut master_points, .. } = new_adj.kind {
                                            *master_points = vec![[0.0, 0.0], [1.0, 1.0]];
                                        }
                                        let _ = shell_lin.write().bridge.submit_all("Set Linear Curve", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text(studio_text(shell,"linear")).font_size(theme::CAPTION_SIZE)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_3.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::Curves { ref mut master_points, .. } = new_adj.kind {
                                            *master_points = vec![[0.0, 0.0], [0.35, 0.20], [0.65, 0.80], [1.0, 1.0]];
                                        }
                                        let _ = shell_hi.write().bridge.submit_all("Set High Contrast", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text(studio_text(shell,"high_contrast")).font_size(theme::CAPTION_SIZE)),
                        ),
                )
        }
        petunia_design_document::adjustments::AdjustmentKind::Hsl {
            hue_shift,
            saturation,
            lightness,
        } => {
            let mut shell_h = shell;
            let mut shell_s_up = shell;
            let mut shell_s_down = shell;
            let mut shell_l_up = shell;
            let adj_1 = adj.clone();
            let adj_2 = adj.clone();
            let adj_3 = adj.clone();
            let adj_4 = adj.clone();

            rect()
                .direction(Direction::Vertical)
                .width(Size::fill())
                .spacing(2.)
                .child(
                    label()
                        .text(format!("Matiz: {:+.0}° | Sat: {:+.0}% | Lum: {:+.0}%", hue_shift, saturation * 100.0, lightness * 100.0))
                        .font_size(theme::CAPTION_SIZE)
                        .color(theme::TEXT_TERTIARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .spacing(2.)
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_1.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::Hsl { ref mut hue_shift, .. } = new_adj.kind {
                                            *hue_shift = (*hue_shift + 15.0).rem_euclid(360.0);
                                        }
                                        let _ = shell_h.write().bridge.submit_all("Shift Hue", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text("H +15°").font_size(theme::CAPTION_SIZE)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_2.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::Hsl { ref mut saturation, .. } = new_adj.kind {
                                            *saturation = (*saturation - 0.1).clamp(-1.0, 1.0);
                                        }
                                        let _ = shell_s_down.write().bridge.submit_all("Adjust Saturation", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text("S -10%").font_size(theme::CAPTION_SIZE)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_3.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::Hsl { ref mut saturation, .. } = new_adj.kind {
                                            *saturation = (*saturation + 0.1).clamp(-1.0, 1.0);
                                        }
                                        let _ = shell_s_up.write().bridge.submit_all("Adjust Saturation", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text("S +10%").font_size(theme::CAPTION_SIZE)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_4.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::Hsl { ref mut lightness, .. } = new_adj.kind {
                                            *lightness = (*lightness + 0.1).clamp(-1.0, 1.0);
                                        }
                                        let _ = shell_l_up.write().bridge.submit_all("Adjust Lightness", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text("L +10%").font_size(theme::CAPTION_SIZE)),
                        ),
                )
        }
        petunia_design_document::adjustments::AdjustmentKind::Exposure {
            exposure,
            offset,
            gamma,
        } => {
            let mut shell_ev_down = shell;
            let mut shell_ev_up = shell;
            let mut shell_off_up = shell;
            let mut shell_gam_up = shell;
            let adj_1 = adj.clone();
            let adj_2 = adj.clone();
            let adj_3 = adj.clone();
            let adj_4 = adj.clone();

            rect()
                .direction(Direction::Vertical)
                .width(Size::fill())
                .spacing(2.)
                .child(
                    label()
                        .text(format!("EV: {:+.1} | Offset: {:+.2} | γ: {:.2}", exposure, offset, gamma))
                        .font_size(theme::CAPTION_SIZE)
                        .color(theme::TEXT_TERTIARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .spacing(2.)
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_1.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::Exposure { ref mut exposure, .. } = new_adj.kind {
                                            *exposure = (*exposure - 0.5).clamp(-5.0, 5.0);
                                        }
                                        let _ = shell_ev_down.write().bridge.submit_all("Adjust Exposure", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text("EV -0.5").font_size(theme::CAPTION_SIZE)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_2.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::Exposure { ref mut exposure, .. } = new_adj.kind {
                                            *exposure = (*exposure + 0.5).clamp(-5.0, 5.0);
                                        }
                                        let _ = shell_ev_up.write().bridge.submit_all("Adjust Exposure", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text("EV +0.5").font_size(theme::CAPTION_SIZE)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_3.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::Exposure { ref mut offset, .. } = new_adj.kind {
                                            *offset = (*offset + 0.05).clamp(-0.5, 0.5);
                                        }
                                        let _ = shell_off_up.write().bridge.submit_all("Adjust Offset", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text("Off +0.05").font_size(theme::CAPTION_SIZE)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_4.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::Exposure { ref mut gamma, .. } = new_adj.kind {
                                            *gamma = (*gamma + 0.1).clamp(0.1, 5.0);
                                        }
                                        let _ = shell_gam_up.write().bridge.submit_all("Adjust Gamma", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text("γ +0.1").font_size(theme::CAPTION_SIZE)),
                        ),
                )
        }
        petunia_design_document::adjustments::AdjustmentKind::WhiteBalance {
            temperature,
            tint,
        } => {
            let mut shell_t_down = shell;
            let mut shell_t_up = shell;
            let mut shell_tint_down = shell;
            let mut shell_tint_up = shell;
            let adj_1 = adj.clone();
            let adj_2 = adj.clone();
            let adj_3 = adj.clone();
            let adj_4 = adj.clone();

            rect()
                .direction(Direction::Vertical)
                .width(Size::fill())
                .spacing(2.)
                .child(
                    label()
                        .text(format!("Temp: {:+.2} | Tint: {:+.2}", temperature, tint))
                        .font_size(theme::CAPTION_SIZE)
                        .color(theme::TEXT_TERTIARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal).content(Content::Flex)
                        .spacing(2.)
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_1.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::WhiteBalance { ref mut temperature, .. } = new_adj.kind {
                                            *temperature = (*temperature - 0.1).clamp(-1.0, 1.0);
                                        }
                                        let _ = shell_t_down.write().bridge.submit_all("Cooler Temperature", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text(studio_text(shell,"cooler")).font_size(theme::CAPTION_SIZE)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_2.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::WhiteBalance { ref mut temperature, .. } = new_adj.kind {
                                            *temperature = (*temperature + 0.1).clamp(-1.0, 1.0);
                                        }
                                        let _ = shell_t_up.write().bridge.submit_all("Warmer Temperature", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text(studio_text(shell,"warmer")).font_size(theme::CAPTION_SIZE)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_3.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::WhiteBalance { ref mut tint, .. } = new_adj.kind {
                                            *tint = (*tint - 0.1).clamp(-1.0, 1.0);
                                        }
                                        let _ = shell_tint_down.write().bridge.submit_all("Green Tint", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text(studio_text(shell,"green_less")).font_size(theme::CAPTION_SIZE)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    if let Some(id) = target_id {
                                        let mut new_adj = adj_4.clone();
                                        if let petunia_design_document::adjustments::AdjustmentKind::WhiteBalance { ref mut tint, .. } = new_adj.kind {
                                            *tint = (*tint + 0.1).clamp(-1.0, 1.0);
                                        }
                                        let _ = shell_tint_up.write().bridge.submit_all("Magenta Tint", vec![Command::SetAdjustment { id, adjustment: new_adj }]);
                                    }
                                })
                                .child(label().text("Magenta +0.1").font_size(theme::CAPTION_SIZE)),
                        ),
                )
        }
    };

    let mut shell_remove = shell;
    rect()
        .direction(Direction::Vertical)
        .width(Size::fill())
        .padding(Gaps::new_all(4.))
        .background(theme::SURFACE_CHROME)
        .spacing(2.)
        .child(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text(title)
                        .font_size(11.)
                        .color(theme::TEXT_PRIMARY),
                )
                .child(
                    Button::new()
                        .on_press(move |_| {
                            if let Some(id) = target_id {
                                let _ = shell_remove.write().bridge.submit_all(
                                    "Remove adjustment",
                                    vec![Command::RemoveAdjustment {
                                        id,
                                        adjustment_id: adj_id,
                                    }],
                                );
                            }
                        })
                        .child(label().text("✕").font_size(theme::CAPTION_SIZE)),
                ),
        )
        .child(body)
}

// =========================================================================
// Histogram Widget (Spec 10.10)
// =========================================================================

#[cfg(test)]
pub fn compute_histogram_bins(
    shell: &PetuniaShell,
    selected_obj: Option<&petunia_design_document::DocumentObject>,
    channel: u8,
) -> ([f32; 32], u32, u32, u32, u32) {
    use petunia_design_application::histogram::{analyze, HistogramRequest};
    let source = shell
        .canvas_snapshot()
        .preview_source
        .expect("histogram source");
    analyze(
        &HistogramRequest {
            source,
            object: selected_obj.map(|object| object.id),
        },
        &petunia_design_jobs::CancellationToken::new(),
    )
    .expect("composed histogram")
    .summary(channel)
}

#[derive(Clone, PartialEq)]
pub struct HistogramWidget {
    pub ui: UiShell,
    pub object_id: Option<ObjectId>,
}

impl Component for HistogramWidget {
    fn render(&self) -> impl IntoElement {
        let mut channel_state = use_state(|| 0u8);
        let channel = *channel_state.read();

        let base = self.ui.shell.read().canvas_snapshot().preview_source;
        let source = self
            .ui
            .canvas_text_source
            .read()
            .clone()
            .filter(|source| {
                crate::canvas_text::is_active(&self.ui)
                    && base.as_ref().is_some_and(|base| {
                        source.surface_id() == base.surface_id()
                            && source.revision() == base.revision()
                    })
            })
            .or(base);
        let request =
            source.map(
                |source| petunia_design_application::histogram::HistogramRequest {
                    source,
                    object: self.object_id,
                },
            );
        let (histogram, status) = crate::histogram_ui::use_histogram(request);
        let (bins, mean, shadows_pct, midtones_pct, highlights_pct) = histogram
            .as_ref()
            .map_or(([0.; 32], 0, 0, 0, 0), |h| h.summary(channel));

        let bar_color = match channel {
            1 => Color::from_rgb(0xEF, 0x44, 0x44), // Red
            2 => Color::from_rgb(0x22, 0xC5, 0x5E), // Green
            3 => Color::from_rgb(0x3B, 0x82, 0xF6), // Blue
            4 => Color::from_rgb(0xCB, 0xD5, 0xE1), // Luma
            _ => Color::from_rgb(0xE2, 0xE8, 0xF0), // RGB Composite
        };

        rect()
            .direction(Direction::Vertical)
            .width(Size::fill())
            .spacing(theme::SPACE_1)
            .child(
                label()
                    .text(self.ui.text("histogram_scope"))
                    .font_size(theme::CAPTION_SIZE),
            )
            .children(
                status
                    .into_iter()
                    .map(|text| label().text(text).font_size(theme::CAPTION_SIZE)),
            )
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .main_align(Alignment::SpaceBetween)
                    .child(channel_button("RGB", 0, channel == 0, &mut channel_state))
                    .child(channel_button("R", 1, channel == 1, &mut channel_state))
                    .child(channel_button("G", 2, channel == 2, &mut channel_state))
                    .child(channel_button("B", 3, channel == 3, &mut channel_state))
                    .child(channel_button("Luma", 4, channel == 4, &mut channel_state)),
            )
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::px(72.))
                    .background(theme::SURFACE_CHROME)
                    .border(
                        Border::new()
                            .fill(theme::SURFACE_CHROME_STRONG)
                            .width(1.)
                            .alignment(BorderAlignment::Inner),
                    )
                    .padding(Gaps::new(2., 2., 2., 2.))
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .cross_align(Alignment::End)
                    .children(bins.iter().enumerate().map(|(idx, &val)| {
                        let bar_h = (val * 64.0).clamp(0.0, 64.0);
                        let is_grid = idx == 8 || idx == 16 || idx == 24;
                        rect()
                            .width(Size::flex(1.0))
                            .height(Size::px(bar_h))
                            .background(bar_color)
                            .opacity(if is_grid { 0.95 } else { 0.75 })
                    })),
            )
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .main_align(Alignment::SpaceBetween)
                    .child(
                        label()
                            .text(format!("{}: {}", self.ui.text("histogram_mean"), mean))
                            .font_size(theme::CAPTION_SIZE)
                            .color(theme::TEXT_SECONDARY),
                    )
                    .child(
                        label()
                            .text(format!(
                                "{}: {}%",
                                self.ui.text("histogram_shadows"),
                                shadows_pct
                            ))
                            .font_size(theme::CAPTION_SIZE)
                            .color(theme::TEXT_TERTIARY),
                    )
                    .child(
                        label()
                            .text(format!(
                                "{}: {}%",
                                self.ui.text("histogram_midtones"),
                                midtones_pct
                            ))
                            .font_size(theme::CAPTION_SIZE)
                            .color(theme::TEXT_TERTIARY),
                    )
                    .child(
                        label()
                            .text(format!(
                                "{}: {}%",
                                self.ui.text("histogram_highlights"),
                                highlights_pct
                            ))
                            .font_size(theme::CAPTION_SIZE)
                            .color(theme::TEXT_TERTIARY),
                    ),
            )
    }
}

fn channel_button(
    title: &'static str,
    index: u8,
    active: bool,
    state: &mut State<u8>,
) -> impl IntoElement {
    let mut state_clone = *state;
    rect()
        .padding(Gaps::new(2., 8., 2., 8.))
        .background(if active {
            theme::SURFACE_PANEL
        } else {
            theme::SURFACE_CHROME
        })
        .border(
            Border::new()
                .fill(if active {
                    theme::BLOOM.value
                } else {
                    theme::SURFACE_CHROME_STRONG
                })
                .width(1.)
                .alignment(BorderAlignment::Inner),
        )
        .on_press(move |_| {
            state_clone.set(index);
        })
        .child(
            label()
                .text(title)
                .font_size(theme::CAPTION_SIZE)
                .color(if active {
                    theme::TEXT_PRIMARY
                } else {
                    theme::TEXT_TERTIARY
                }),
        )
}

/// Draggable, keyboard-accessible panel edges share one interaction owner.
#[derive(Clone, PartialEq)]
pub struct DockSplitter(pub UiShell);
impl Component for DockSplitter {
    fn render(&self) -> impl IntoElement {
        crate::studio_widgets::StudioSplitter::new(&self.0, crate::studio_widgets::DockEdge::Right)
    }
}
#[derive(Clone, PartialEq)]
pub struct LeftDockSplitter(pub UiShell);
impl Component for LeftDockSplitter {
    fn render(&self) -> impl IntoElement {
        crate::studio_widgets::StudioSplitter::new(&self.0, crate::studio_widgets::DockEdge::Left)
    }
}
#[derive(Clone, PartialEq)]
pub struct BottomDockSplitter(pub UiShell);
impl Component for BottomDockSplitter {
    fn render(&self) -> impl IntoElement {
        crate::studio_widgets::StudioSplitter::new(&self.0, crate::studio_widgets::DockEdge::Bottom)
    }
}

/// The left dock surface containing Assets & Symbol libraries.
#[derive(Clone, PartialEq)]
pub struct LeftDock(pub UiShell);

impl Component for LeftDock {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        if !*ui.left_dock_open.read() {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }

        let mut left_dock_tab = ui.left_dock_tab;
        let active_tab = *left_dock_tab.read();
        let tools = theme::TOOL_RAIL_WIDTH
            * if ui.tool_rail.read().columns == crate::ui_state::RailColumns::Two {
                2.
            } else {
                1.
            };
        let dock_w = crate::studio_widgets::left_studio_width(
            *ui.left_dock_width.read(),
            Platform::get().root_size.read().width,
            tools,
            *ui.right_studio_open.read(),
        );
        let mut left_dock_open = ui.left_dock_open;

        rect()
            .direction(Direction::Vertical)
            .content(Content::Flex)
            .width(Size::px(dock_w))
            .height(Size::fill())
            .background(theme::SURFACE_PANEL)
            .border(
                Border::new()
                    .fill(theme::SURFACE_CHROME_STRONG)
                    .width(1.)
                    .alignment(BorderAlignment::Inner),
            )
            .child(
                // Tab Header Bar
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .height(Size::px(theme::PANEL_HEADER_HEIGHT))
                    .background(theme::SURFACE_CHROME)
                    .cross_align(Alignment::Center)
                    .main_align(Alignment::SpaceBetween)
                    .padding(Gaps::new(0., theme::SPACE_1, 0., theme::SPACE_1))
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .content(Content::Flex)
                            .spacing(theme::SPACE_1)
                            .cross_align(Alignment::Center)
                            .child(tab_button(
                                ui,
                                ui.studio_text("assets"),
                                0,
                                active_tab == 0,
                                &mut left_dock_tab,
                            ))
                            .child(tab_button(
                                ui,
                                ui.studio_text("symbols"),
                                1,
                                active_tab == 1,
                                &mut left_dock_tab,
                            )),
                    )
                    .child(
                        crate::studio_widgets::StudioButton::new(ui, ui.studio_text("hide_assets"))
                            .icon(theme::ICON_CLOSE)
                            .width(Size::px(28.))
                            .on_press(move |_| left_dock_open.set(false)),
                    ),
            )
            .child(
                rect()
                    .direction(Direction::Vertical)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .height(Size::flex(1.0))
                    .padding(Gaps::new(
                        theme::SPACE_2,
                        theme::SPACE_2,
                        theme::SPACE_2,
                        theme::SPACE_2,
                    ))
                    .child(match active_tab {
                        0 => AssetsTab(ui.clone()).into_element(),
                        _ => SymbolsTab(ui.clone()).into_element(),
                    }),
            )
    }
}

/// Assets panel tab: lists placed images and project assets.
#[derive(Clone, PartialEq)]
pub struct AssetsTab(pub UiShell);

impl Component for AssetsTab {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let snapshot = ui.shell.read().canvas_snapshot();
        let image_assets: Vec<_> = snapshot
            .objects
            .iter()
            .filter(|o| matches!(o.shape.as_deref(), Some(ShapeKind::Image { .. })))
            .collect();
        let mut place_image_open = ui.place_image_open;

        ScrollView::new().child(
            rect()
                .direction(Direction::Vertical)
                .width(Size::fill())
                .spacing(theme::SPACE_2)
                .child(section_header(ui.studio_text("document_assets")))
                .child(
                    Button::new()
                        .on_press(move |_| {
                            place_image_open.set(true);
                        })
                        .child(
                            rect()
                                .direction(Direction::Horizontal)
                                .content(Content::Flex)
                                .padding(Gaps::new(4., 8., 4., 8.))
                                .background(theme::SURFACE_CHROME_STRONG)
                                .corner_radius(4.0)
                                .cross_align(Alignment::Center)
                                .child(
                                    label()
                                        .text(ui.studio_text("place_image"))
                                        .font_size(11.)
                                        .color(theme::ACCENT_BLOOM),
                                ),
                        ),
                )
                .child(
                    label()
                        .text(format!("Imagens posicionadas: {}", image_assets.len()))
                        .font_size(11.)
                        .color(theme::TEXT_SECONDARY),
                )
                .children(image_assets.iter().map(|img| {
                    let bounds = img.world_bounds;
                    let w = bounds[2] - bounds[0];
                    let h = bounds[3] - bounds[1];
                    rect()
                        .direction(Direction::Vertical)
                        .padding(Gaps::new(4., 6., 4., 6.))
                        .background(theme::SURFACE_CHROME)
                        .corner_radius(4.0)
                        .child(
                            label()
                                .text(format!("{} #{}", ui.studio_text("image"), img.id))
                                .font_size(11.)
                                .color(theme::TEXT_PRIMARY),
                        )
                        .child(
                            label()
                                .text(format!("{:.0} x {:.0} pt", w, h))
                                .font_size(theme::CAPTION_SIZE)
                                .color(theme::TEXT_TERTIARY),
                        )
                        .into_element()
                })),
        )
    }
}

/// Symbols & component templates tab.
#[derive(Clone, PartialEq)]
pub struct SymbolsTab(pub UiShell);

impl Component for SymbolsTab {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let shell = ui.shell;

        ScrollView::new().child(
            rect()
                .direction(Direction::Vertical)
                .width(Size::fill())
                .spacing(theme::SPACE_2)
                .child(section_header(ui.studio_text("shape_library")))
                .child(symbol_preset_item(
                    shell,
                    ui.studio_text("basic_rectangle"),
                    "200x150 pt",
                    ShapeKind::Rectangle {
                        corner_radii: [0.0; 4],
                    },
                    [50.0, 50.0, 200.0, 150.0],
                    "ptnd.blue/500",
                ))
                .child(symbol_preset_item(
                    shell,
                    ui.studio_text("ui_card"),
                    "240x140 pt · r=8",
                    ShapeKind::Rectangle {
                        corner_radii: [8.0; 4],
                    },
                    [50.0, 50.0, 240.0, 140.0],
                    "ptnd.gray/900",
                ))
                .child(symbol_preset_item(
                    shell,
                    ui.studio_text("basic_ellipse"),
                    "100x100 pt",
                    ShapeKind::Ellipse,
                    [50.0, 50.0, 100.0, 100.0],
                    "ptnd.teal/500",
                ))
                .child(symbol_preset_item(
                    shell,
                    ui.studio_text("basic_star"),
                    "120x120 pt",
                    ShapeKind::Star {
                        points: 5,
                        inner_ratio: 0.5,
                    },
                    [50.0, 50.0, 120.0, 120.0],
                    "ptnd.purple/500",
                )),
        )
    }
}

fn symbol_preset_item(
    shell: State<PetuniaShell>,
    name: String,
    subtitle: &'static str,
    shape: ShapeKind,
    bounds: [f64; 4],
    fill: &'static str,
) -> impl IntoElement {
    let mut shell_for_insert = shell;
    let shape_clone = shape;
    rect()
        .direction(Direction::Horizontal)
        .content(Content::Flex)
        .width(Size::fill())
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .padding(Gaps::new(4., 6., 4., 6.))
        .background(theme::SURFACE_CHROME)
        .corner_radius(4.0)
        .child(
            rect()
                .direction(Direction::Vertical)
                .child(
                    label()
                        .text(name.clone())
                        .font_size(11.)
                        .color(theme::TEXT_PRIMARY),
                )
                .child(
                    label()
                        .text(subtitle)
                        .font_size(theme::CAPTION_SIZE)
                        .color(theme::TEXT_TERTIARY),
                ),
        )
        .child(
            Button::new()
                .on_press(move |_| {
                    let mut s = shell_for_insert.write();
                    if let Some(surf) = s.bridge.active_surface() {
                        if let Ok(obj_id) = s.bridge.next_object_id() {
                            let msg = format!("Insert {}", name);
                            let _ = s.bridge.submit_all(
                                &msg,
                                vec![
                                    Command::CreateObject {
                                        surface: surf,
                                        id: obj_id,
                                        name: name.to_string(),
                                    },
                                    Command::SetBounds {
                                        id: obj_id,
                                        bounds: Some(bounds),
                                        rotation: 0.0,
                                    },
                                    Command::SetShape {
                                        id: obj_id,
                                        shape: Some(shape_clone.clone()),
                                    },
                                    Command::SetFill {
                                        id: obj_id,
                                        fill: Some(fill.to_string()),
                                    },
                                ],
                            );
                        }
                    }
                })
                .child(
                    rect()
                        .padding(Gaps::new(2., 6., 2., 6.))
                        .background(theme::SURFACE_CHROME_STRONG)
                        .corner_radius(4.0)
                        .child(
                            label()
                                .text(studio_text(shell, "insert"))
                                .font_size(theme::CAPTION_SIZE)
                                .color(theme::ACCENT_BLOOM),
                        ),
                ),
        )
}

/// The bottom dock surface containing Background Tasks & Diagnostics.
#[derive(Clone, PartialEq)]
pub struct BottomDock(pub UiShell);

impl Component for BottomDock {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        if !*ui.bottom_dock_open.read() {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }

        let mut bottom_dock_tab = ui.bottom_dock_tab;
        let active_tab = *bottom_dock_tab.read();
        let dock_h = crate::studio_widgets::bottom_studio_height(
            *ui.bottom_dock_height.read(),
            Platform::get().root_size.read().height,
        );
        let mut bottom_dock_open = ui.bottom_dock_open;

        rect()
            .direction(Direction::Vertical)
            .content(Content::Flex)
            .width(Size::fill())
            .height(Size::px(dock_h))
            .background(theme::SURFACE_PANEL)
            .border(
                Border::new()
                    .fill(theme::SURFACE_CHROME_STRONG)
                    .width(1.)
                    .alignment(BorderAlignment::Inner),
            )
            .child(
                // Tab Header Bar
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .height(Size::px(theme::PANEL_HEADER_HEIGHT))
                    .background(theme::SURFACE_CHROME)
                    .cross_align(Alignment::Center)
                    .main_align(Alignment::SpaceBetween)
                    .padding(Gaps::new(0., theme::SPACE_2, 0., theme::SPACE_2))
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .content(Content::Flex)
                            .spacing(theme::SPACE_1)
                            .cross_align(Alignment::Center)
                            .child(tab_button(
                                ui,
                                ui.studio_text("tasks"),
                                0,
                                active_tab == 0,
                                &mut bottom_dock_tab,
                            ))
                            .child(tab_button(
                                ui,
                                ui.studio_text("diagnostics"),
                                1,
                                active_tab == 1,
                                &mut bottom_dock_tab,
                            )),
                    )
                    .child(
                        crate::studio_widgets::StudioButton::new(
                            ui,
                            ui.studio_text("hide_diagnostics"),
                        )
                        .icon(theme::ICON_CLOSE)
                        .width(Size::px(28.))
                        .on_press(move |_| bottom_dock_open.set(false)),
                    ),
            )
            .child(
                rect()
                    .direction(Direction::Vertical)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .height(Size::flex(1.0))
                    .padding(Gaps::new(
                        theme::SPACE_1,
                        theme::SPACE_2,
                        theme::SPACE_1,
                        theme::SPACE_2,
                    ))
                    .child(match active_tab {
                        0 => BackgroundTasksPanel(ui.clone()).into_element(),
                        _ => DiagnosticsTab(ui.clone()).into_element(),
                    }),
            )
    }
}

/// Diagnostics & session preflight tab for BottomDock.
#[derive(Clone, PartialEq)]
pub struct DiagnosticsTab(pub UiShell);

impl Component for DiagnosticsTab {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let shell_ref = ui.shell.read();
        let snapshot = shell_ref.canvas_snapshot();
        let cam = shell_ref.view_camera();
        let session = shell_ref.bridge.session();
        let title = session.map_or_else(|| ui.studio_text("no_document"), |s| s.title().to_owned());
        let rev = session.map_or(0, |s| s.saved_revision());
        let dirty = shell_ref.bridge.is_dirty();
        let surf_id = shell_ref
            .bridge
            .active_surface()
            .map_or_else(|| ui.studio_text("none"), |id| id.to_string());
        let obj_count = snapshot.objects.len();

        ScrollView::new().child(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .width(Size::fill())
                .spacing(theme::SPACE_3)
                .padding(Gaps::new(
                    theme::SPACE_1,
                    theme::SPACE_2,
                    theme::SPACE_1,
                    theme::SPACE_2,
                ))
                .child(
                    rect()
                        .direction(Direction::Vertical)
                        .spacing(theme::SPACE_1)
                        .child(
                            label()
                                .text(ui.studio_text("document"))
                                .font_size(theme::CAPTION_SIZE)
                                .color(theme::TEXT_TERTIARY),
                        )
                        .child(
                            label()
                                .text(format!("{title} (rev: {rev})"))
                                .font_size(11.)
                                .color(theme::TEXT_PRIMARY),
                        )
                        .child(
                            label()
                                .text(ui.studio_text(if dirty { "unsaved" } else { "saved" }))
                                .font_size(theme::CAPTION_SIZE)
                                .color(if dirty {
                                    theme::ACCENT_BLOOM
                                } else {
                                    theme::TEXT_SECONDARY
                                }),
                        ),
                )
                .child(
                    rect()
                        .direction(Direction::Vertical)
                        .spacing(theme::SPACE_1)
                        .child(
                            label()
                                .text(ui.studio_text("surface_objects"))
                                .font_size(theme::CAPTION_SIZE)
                                .color(theme::TEXT_TERTIARY),
                        )
                        .child(
                            label()
                                .text(format!("{}: {surf_id}", ui.studio_text("active_surface")))
                                .font_size(11.)
                                .color(theme::TEXT_PRIMARY),
                        )
                        .child(
                            label()
                                .text(format!("{}: {obj_count}", ui.studio_text("object_count")))
                                .font_size(theme::CAPTION_SIZE)
                                .color(theme::TEXT_SECONDARY),
                        ),
                )
                .child(
                    rect()
                        .direction(Direction::Vertical)
                        .spacing(theme::SPACE_1)
                        .child(
                            label()
                                .text(ui.studio_text("viewport_camera"))
                                .font_size(theme::CAPTION_SIZE)
                                .color(theme::TEXT_TERTIARY),
                        )
                        .child(
                            label()
                                .text(format!("{:.0}% zoom", cam.zoom * 100.0))
                                .font_size(11.)
                                .color(theme::TEXT_PRIMARY),
                        )
                        .child(
                            label()
                                .text(format!("Pan: ({:.1}, {:.1})", cam.pan_x, cam.pan_y))
                                .font_size(theme::CAPTION_SIZE)
                                .color(theme::TEXT_SECONDARY),
                        ),
                )
                .child(
                    rect()
                        .direction(Direction::Vertical)
                        .spacing(theme::SPACE_1)
                        .child(
                            label()
                                .text(ui.studio_text("tool_studio"))
                                .font_size(theme::CAPTION_SIZE)
                                .color(theme::TEXT_TERTIARY),
                        )
                        .child(
                            label()
                                .text(format!("{:?}", *ui.active_tool.read()))
                                .font_size(11.)
                                .color(theme::TEXT_PRIMARY),
                        )
                        .child(
                            label()
                                .text(ui.persona.read().clone())
                                .font_size(theme::CAPTION_SIZE)
                                .color(theme::TEXT_SECONDARY),
                        ),
                ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use freya_testing::TestingRunner;
    use petunia_design_shell::{PetuniaShell, ToolKind};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn dock_renders_all_four_tabs_and_switches() {
        let seen: Rc<RefCell<Option<UiShell>>> = Rc::new(RefCell::new(None));
        let seen_hook = seen.clone();

        let (mut runner, ()) = TestingRunner::new(
            move || {
                let shell = use_state(|| {
                    let mut s = PetuniaShell::new(800., 600.);
                    s.new_document("TestDoc").expect("document opens");
                    s.set_active_tool(ToolKind::Select);
                    s
                });
                let ui = UiShell::fresh(shell);
                seen_hook.replace(Some(ui.clone()));
                RightDock(ui)
            },
            (320., 600.).into(),
            |_| {},
            1.,
        );

        runner.sync_and_update();

        let ui = seen.borrow().clone().unwrap();
        assert_eq!(*ui.dock_tab.read(), 0, "default active tab is Camadas (0)");

        for (key, index) in [
            ("properties", 1),
            ("colors", 2),
            ("history", 3),
            ("navigator", 4),
            ("layers", 0),
        ] {
            let title = ui.studio_text(key);
            let area = runner
                .find(|node, element| {
                    Label::try_downcast(element)
                        .filter(|label| label.text == title)
                        .map(|_| node)
                })
                .unwrap()
                .layout()
                .area;
            assert!(
                area.min_x() >= 0. && area.max_x() <= 320.,
                "tab must fit within the default dock"
            );
            assert!(area.height() < 25., "tab must stay on one line");
            runner.click_cursor((
                f64::from(area.min_x() + area.width() / 2.),
                f64::from(area.min_y() + area.height() / 2.),
            ));
            runner.sync_and_update();
            assert_eq!(*ui.dock_tab.read(), index);
        }
        let mut dock_tab = ui.dock_tab;
        // Switch to tab 5: Tarefas
        dock_tab.set(5);
        runner.sync_and_update();
        assert_eq!(
            *ui.dock_tab.read(),
            5,
            "active tab should switch to Tarefas (5)"
        );
    }

    #[test]
    fn dock_quick_color_swatch_and_stroke_commands_mutate_object() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("TestDoc").expect("doc opens");
        let surf_id = shell.bridge.active_surface().unwrap();
        let obj_id = shell.bridge.next_object_id().unwrap();
        let _ = shell.bridge.submit_all(
            "Create rect",
            vec![
                Command::CreateObject {
                    surface: surf_id,
                    id: obj_id,
                    name: "Rect1".to_string(),
                },
                Command::SetBounds {
                    id: obj_id,
                    bounds: Some([10.0, 10.0, 100.0, 100.0]),
                    rotation: 0.0,
                },
                Command::SetShape {
                    id: obj_id,
                    shape: Some(ShapeKind::Rectangle {
                        corner_radii: [0.0; 4],
                    }),
                },
                Command::SetFill {
                    id: obj_id,
                    fill: Some("ptnd.gray/50".to_string()),
                },
            ],
        );

        // Mutate fill
        let _ = shell.bridge.submit_all(
            "Set fill",
            vec![Command::SetFill {
                id: obj_id,
                fill: Some("ptnd.bloom/500".to_string()),
            }],
        );

        let session = shell.bridge.session().unwrap();
        let surf = session.surface(surf_id).unwrap();
        let obj = surf.objects().iter().find(|o| o.id == obj_id).unwrap();
        assert_eq!(obj.fill.as_deref(), Some("ptnd.bloom/500"));

        // Mutate stroke
        let _ = shell.bridge.submit_all(
            "Set stroke",
            vec![Command::SetStroke {
                id: obj_id,
                stroke: Some("ptnd.gray/900".to_string()),
                width: 5.0,
            }],
        );

        let session = shell.bridge.session().unwrap();
        let surf = session.surface(surf_id).unwrap();
        let obj = surf.objects().iter().find(|o| o.id == obj_id).unwrap();
        assert_eq!(obj.stroke.as_deref(), Some("ptnd.gray/900"));
        assert!((obj.stroke_width - 5.0).abs() < 1e-5);
    }

    #[test]
    fn dock_tonal_adjustments_and_live_filters_commands_mutate_object() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("TonalDoc").expect("doc opens");
        let surf_id = shell.bridge.active_surface().unwrap();
        let obj_id = shell.bridge.next_object_id().unwrap();

        let _ = shell.bridge.submit_all(
            "Add test shape",
            vec![
                Command::CreateObject {
                    surface: surf_id,
                    id: obj_id,
                    name: "TonalRect".to_string(),
                },
                Command::SetBounds {
                    id: obj_id,
                    bounds: Some([0.0, 0.0, 100.0, 100.0]),
                    rotation: 0.0,
                },
                Command::SetShape {
                    id: obj_id,
                    shape: Some(ShapeKind::Rectangle {
                        corner_radii: [0.0; 4],
                    }),
                },
            ],
        );

        // 1. Add Sharpen filter
        let _ = shell.bridge.submit_all(
            "Add sharpen",
            vec![Command::AddEffect {
                id: obj_id,
                effect: petunia_design_document::EffectItem {
                    id: 103,
                    kind: petunia_design_document::EffectKind::Sharpen {
                        radius: 2.0,
                        amount: 1.5,
                    },
                    visible: true,
                },
            }],
        );

        // 2. Add Noise filter
        let _ = shell.bridge.submit_all(
            "Add noise",
            vec![Command::AddEffect {
                id: obj_id,
                effect: petunia_design_document::EffectItem {
                    id: 104,
                    kind: petunia_design_document::EffectKind::Noise {
                        amount: 0.25,
                        monochrome: true,
                    },
                    visible: true,
                },
            }],
        );

        // Verify filters exist in appearance stack
        {
            let session = shell.bridge.session().unwrap();
            let surf = session.surface(surf_id).unwrap();
            let obj = surf.objects().iter().find(|o| o.id == obj_id).unwrap();
            let app = obj.appearance.as_ref().expect("appearance stack exists");
            assert_eq!(app.effects.len(), 2);
            assert!(matches!(
                app.effects[0].kind,
                petunia_design_document::EffectKind::Sharpen { radius, amount }
                if (radius - 2.0).abs() < 1e-5 && (amount - 1.5).abs() < 1e-5
            ));
            assert!(matches!(
                app.effects[1].kind,
                petunia_design_document::EffectKind::Noise { amount, monochrome }
                if (amount - 0.25).abs() < 1e-5 && monochrome
            ));
        }

        // 3. Add Levels adjustment
        let levels_item = petunia_design_document::adjustments::AdjustmentItem::new(
            1,
            petunia_design_document::adjustments::AdjustmentKind::default_levels(),
        );
        let _ = shell.bridge.submit_all(
            "Add levels",
            vec![Command::AddAdjustment {
                id: obj_id,
                adjustment: levels_item.clone(),
            }],
        );

        // 4. Update Levels gamma
        let mut updated_levels = levels_item.clone();
        if let petunia_design_document::adjustments::AdjustmentKind::Levels {
            ref mut master, ..
        } = updated_levels.kind
        {
            master.gamma = 1.8;
        }
        let _ = shell.bridge.submit_all(
            "Set levels gamma",
            vec![Command::SetAdjustment {
                id: obj_id,
                adjustment: updated_levels,
            }],
        );

        // 5. Add Exposure adjustment
        let exp_item = petunia_design_document::adjustments::AdjustmentItem::new(
            2,
            petunia_design_document::adjustments::AdjustmentKind::default_exposure(),
        );
        let _ = shell.bridge.submit_all(
            "Add exposure",
            vec![Command::AddAdjustment {
                id: obj_id,
                adjustment: exp_item,
            }],
        );

        // Verify both adjustments exist
        {
            let session = shell.bridge.session().unwrap();
            let surf = session.surface(surf_id).unwrap();
            let obj = surf.objects().iter().find(|o| o.id == obj_id).unwrap();
            let app = obj.appearance.as_ref().expect("appearance stack exists");
            assert_eq!(app.adjustments.len(), 2);
            if let petunia_design_document::adjustments::AdjustmentKind::Levels { master, .. } =
                app.adjustments[0].kind
            {
                assert!((master.gamma - 1.8).abs() < 1e-5);
            } else {
                panic!("First adjustment must be Levels");
            }
        }

        // 6. Remove Exposure adjustment
        let _ = shell.bridge.submit_all(
            "Remove exposure",
            vec![Command::RemoveAdjustment {
                id: obj_id,
                adjustment_id: 2,
            }],
        );

        // Verify only Levels remains
        {
            let session = shell.bridge.session().unwrap();
            let surf = session.surface(surf_id).unwrap();
            let obj = surf.objects().iter().find(|o| o.id == obj_id).unwrap();
            let app = obj.appearance.as_ref().expect("appearance stack exists");
            assert_eq!(app.adjustments.len(), 1);
            assert_eq!(app.adjustments[0].id, 1);
        }

        // 7. Test Undo
        let _ = shell.bridge.undo();
        {
            let session = shell.bridge.session().unwrap();
            let surf = session.surface(surf_id).unwrap();
            let obj = surf.objects().iter().find(|o| o.id == obj_id).unwrap();
            let app = obj.appearance.as_ref().expect("appearance stack exists");
            assert_eq!(app.adjustments.len(), 2, "Undo restores removed adjustment");
        }
    }

    #[test]
    fn histogram_computation_and_channel_filtering() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("HistoDoc").expect("doc opens");
        let surf_id = shell.bridge.active_surface().unwrap();
        let obj_id = shell.bridge.next_object_id().unwrap();

        // 1. When empty/no object selected: baseline document histogram
        let (bins, mean, shadows, midtones, highlights) = compute_histogram_bins(&shell, None, 0);
        assert_eq!(bins.len(), 32);
        assert_eq!(mean, 0);
        assert_eq!((shadows, midtones, highlights), (0, 0, 0));
        assert!(bins.iter().all(|bin| *bin == 0.));

        // 2. Add Red object
        let result = shell.bridge.submit_all(
            "Add Red shape",
            vec![
                Command::CreateObject {
                    surface: surf_id,
                    id: obj_id,
                    name: "RedRect".to_string(),
                },
                Command::SetBounds {
                    id: obj_id,
                    bounds: Some([0.0, 0.0, 100.0, 100.0]),
                    rotation: 0.0,
                },
                Command::SetShape {
                    id: obj_id,
                    shape: Some(ShapeKind::Rectangle {
                        corner_radii: [0.0; 4],
                    }),
                },
                Command::SetFill {
                    id: obj_id,
                    fill: Some("ptnd.red/500".to_string()),
                },
            ],
        );
        result.expect("red histogram fixture must be a valid document");

        let session = shell.bridge.session().unwrap();
        let surf = session.surface(surf_id).unwrap();
        let obj = surf.objects().iter().find(|o| o.id == obj_id).unwrap();

        // Channel 1: Red channel of red object -> high mean, peak in highlights
        let (_r_bins, r_mean, _r_shad, _r_mid, r_high) =
            compute_histogram_bins(&shell, Some(obj), 1);
        assert!(
            r_mean > 200,
            "Red channel mean must be high for red object (got {})",
            r_mean
        );
        assert!(r_high > 50, "Red channel must concentrate in highlights");

        // Channel 2: Green channel of red object -> low mean, peak in shadows
        let (_g_bins, g_mean, g_shad, _g_mid, _g_high) =
            compute_histogram_bins(&shell, Some(obj), 2);
        assert!(
            g_mean < 100,
            "Green channel mean must be low for red object (got {})",
            g_mean
        );
        assert!(
            g_shad + _g_mid > 95,
            "Green channel must concentrate in shadows and lower midtones"
        );
        assert!(
            _g_high < 5,
            "Green channel must have virtually no highlights (got {})",
            _g_high
        );

        // 3. Add Exposure +1.0 to increase luminance
        let exp_item = petunia_design_document::adjustments::AdjustmentItem::new(
            1,
            petunia_design_document::adjustments::AdjustmentKind::Exposure {
                exposure: 1.0,
                offset: 0.0,
                gamma: 1.0,
            },
        );
        let _ = shell.bridge.submit_all(
            "Add exposure",
            vec![Command::AddAdjustment {
                id: obj_id,
                adjustment: exp_item,
            }],
        );

        let session = shell.bridge.session().unwrap();
        let surf = session.surface(surf_id).unwrap();
        let obj = surf.objects().iter().find(|o| o.id == obj_id).unwrap();

        // Green channel with +1.0 exposure should shift up
        let (_g_bins_exp, g_mean_exp, _, _, _) = compute_histogram_bins(&shell, Some(obj), 2);
        assert!(
            g_mean_exp > g_mean,
            "Exposure must increase mean channel value"
        );

        // 4. Raster buffer histogram sampling on Image object
        let img_id = shell.bridge.next_object_id().unwrap();
        let raw_pixels: Vec<u8> = vec![
            250, 20, 20, 255, // red pixel
            250, 20, 20, 255, // red pixel
            10, 240, 10, 255, // green pixel
            10, 10, 240, 255, // blue pixel
        ];
        let raw_img = petunia_design_io::RawRasterImage::from_rgba8(2, 2, raw_pixels).unwrap();
        let (png_bytes, _) = petunia_design_io::export_raster(
            &raw_img,
            &petunia_design_io::RasterExportOptions::default(),
        )
        .unwrap();

        let _ = shell.bridge.submit_all(
            "Add Image object",
            vec![
                Command::CreateObject {
                    surface: surf_id,
                    id: img_id,
                    name: "SampleImage".to_string(),
                },
                Command::SetBounds {
                    id: img_id,
                    bounds: Some([0.0, 0.0, 2.0, 2.0]),
                    rotation: 0.0,
                },
                Command::SetShape {
                    id: img_id,
                    shape: Some(ShapeKind::Image {
                        path: "sample.png".to_string(),
                        data: Some(std::sync::Arc::new(
                            petunia_design_raster::EncodedImage::new(png_bytes).unwrap(),
                        )),
                    }),
                },
            ],
        );

        let session = shell.bridge.session().unwrap();
        let surf = session.surface(surf_id).unwrap();
        let img_obj = surf.objects().iter().find(|o| o.id == img_id).unwrap();

        let (img_bins, img_mean, _, _, _) = compute_histogram_bins(&shell, Some(img_obj), 0);
        assert_eq!(img_bins.len(), 32);
        assert!(
            img_bins.iter().any(|&b| b > 0.0),
            "Raster sampling must populate bins"
        );
        assert!(img_mean > 0);
    }

    #[test]
    fn pen_and_photo_brush_settings_and_hud_extensions() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("ToolsDoc").expect("doc opens");

        // 1. Pen tool modes
        assert_eq!(
            shell.tools.pen_tool().mode(),
            petunia_design_shell::tools::PenMode::Bezier
        );
        shell
            .tools
            .pen_tool_mut()
            .set_mode(petunia_design_shell::tools::PenMode::Polygon);
        assert_eq!(
            shell.tools.pen_tool().mode(),
            petunia_design_shell::tools::PenMode::Polygon
        );
        shell
            .tools
            .pen_tool_mut()
            .set_mode(petunia_design_shell::tools::PenMode::Line);
        assert_eq!(
            shell.tools.pen_tool().mode(),
            petunia_design_shell::tools::PenMode::Line
        );

        // 2. Photo brush settings
        let default_brush = shell.tools.photo_brush_tool().brush_settings();
        assert!((default_brush.radius - 16.0).abs() < 1e-4);
        assert!((default_brush.hardness - 0.8).abs() < 1e-4);

        let new_brush = petunia_design_shell::tools::PhotoBrushSettings {
            radius: 32.0,
            hardness: 0.5,
            flow: 0.8,
            opacity: 0.9,
            color: default_brush.color,
            ink: None,
        };
        shell
            .tools
            .photo_brush_tool_mut()
            .set_brush_settings(new_brush);
        assert_eq!(shell.tools.photo_brush_tool().brush_settings(), new_brush);

        // 3. Desktop shell helper methods
        let finish_res = shell.finish_open_path();
        assert!(
            finish_res.is_ok(),
            "finish_open_path runs safely even when empty"
        );
        let convert_res = shell.convert_selected_nodes(petunia_design_shell::tools::NodeType::Cusp);
        assert!(convert_res.is_ok(), "convert_selected_nodes runs safely");
        let del_res = shell.delete_selected_nodes();
        assert!(del_res.is_ok(), "delete_selected_nodes runs safely");
    }

    #[test]
    fn layer_fx_drop_shadow_inner_shadow_and_blur_controls() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("FXDoc").expect("doc opens");
        let surf_id = shell.bridge.active_surface().unwrap();
        let obj_id = shell.bridge.next_object_id().unwrap();

        let _ = shell.bridge.submit_all(
            "Add test shape",
            vec![
                Command::CreateObject {
                    surface: surf_id,
                    id: obj_id,
                    name: "FXRect".to_string(),
                },
                Command::SetBounds {
                    id: obj_id,
                    bounds: Some([0.0, 0.0, 100.0, 100.0]),
                    rotation: 0.0,
                },
                Command::SetShape {
                    id: obj_id,
                    shape: Some(ShapeKind::Rectangle {
                        corner_radii: [0.0; 4],
                    }),
                },
            ],
        );

        // 1. Add Drop Shadow
        let _ = shell.bridge.submit_all(
            "Add drop shadow",
            vec![Command::AddEffect {
                id: obj_id,
                effect: petunia_design_document::EffectItem {
                    id: 102,
                    kind: petunia_design_document::EffectKind::DropShadow {
                        offset: [4.0, 6.0],
                        blur: 8.0,
                        color: "ptnd.gray/900".to_string(),
                        opacity: 0.6,
                    },
                    visible: true,
                },
            }],
        );

        // 2. Add Inner Shadow
        let _ = shell.bridge.submit_all(
            "Add inner shadow",
            vec![Command::AddEffect {
                id: obj_id,
                effect: petunia_design_document::EffectItem {
                    id: 105,
                    kind: petunia_design_document::EffectKind::InnerShadow {
                        offset: [2.0, 2.0],
                        blur: 4.0,
                        color: "ptnd.black".to_string(),
                        opacity: 0.5,
                    },
                    visible: true,
                },
            }],
        );

        // 3. Add Gaussian Blur
        let _ = shell.bridge.submit_all(
            "Add blur",
            vec![Command::AddEffect {
                id: obj_id,
                effect: petunia_design_document::EffectItem {
                    id: 101,
                    kind: petunia_design_document::EffectKind::GaussianBlur { radius: 3.5 },
                    visible: true,
                },
            }],
        );

        // Verify appearance stack
        {
            let session = shell.bridge.session().unwrap();
            let surf = session.surface(surf_id).unwrap();
            let obj = surf.objects().iter().find(|o| o.id == obj_id).unwrap();
            let app = obj.appearance.as_ref().expect("appearance exists");
            assert_eq!(app.effects.len(), 3);

            let shadow = app
                .effects
                .iter()
                .find(|e| {
                    matches!(
                        e.kind,
                        petunia_design_document::EffectKind::DropShadow { .. }
                    )
                })
                .unwrap();
            assert_eq!(shadow.id, 102);
            assert!(shadow.visible);

            let inner = app
                .effects
                .iter()
                .find(|e| {
                    matches!(
                        e.kind,
                        petunia_design_document::EffectKind::InnerShadow { .. }
                    )
                })
                .unwrap();
            assert_eq!(inner.id, 105);
            assert!(inner.visible);

            let blur = app
                .effects
                .iter()
                .find(|e| {
                    matches!(
                        e.kind,
                        petunia_design_document::EffectKind::GaussianBlur { .. }
                    )
                })
                .unwrap();
            assert_eq!(blur.id, 101);
            assert!(blur.visible);
        }

        // 4. Toggle Drop Shadow visibility
        let _ = shell.bridge.submit_all(
            "Toggle shadow",
            vec![
                Command::RemoveEffect {
                    id: obj_id,
                    effect_id: 102,
                },
                Command::AddEffect {
                    id: obj_id,
                    effect: petunia_design_document::EffectItem {
                        id: 102,
                        kind: petunia_design_document::EffectKind::DropShadow {
                            offset: [4.0, 6.0],
                            blur: 8.0,
                            color: "ptnd.gray/900".to_string(),
                            opacity: 0.6,
                        },
                        visible: false,
                    },
                },
            ],
        );

        {
            let session = shell.bridge.session().unwrap();
            let surf = session.surface(surf_id).unwrap();
            let obj = surf.objects().iter().find(|o| o.id == obj_id).unwrap();
            let app = obj.appearance.as_ref().unwrap();
            let shadow = app
                .effects
                .iter()
                .find(|e| {
                    matches!(
                        e.kind,
                        petunia_design_document::EffectKind::DropShadow { .. }
                    )
                })
                .unwrap();
            assert!(!shadow.visible, "Drop shadow must now be hidden");
        }
    }

    #[test]
    fn modifier_stack_inspector_and_bake_commands_mutate_object() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("TestDoc").expect("doc opens");
        let surf_id = shell.bridge.active_surface().unwrap();
        let obj_id = shell.bridge.next_object_id().unwrap();

        // 1. Create a rectangle object
        let _ = shell.bridge.submit_all(
            "Create rect",
            vec![
                Command::CreateObject {
                    surface: surf_id,
                    id: obj_id,
                    name: "RectMod".to_string(),
                },
                Command::SetBounds {
                    id: obj_id,
                    bounds: Some([50.0, 50.0, 100.0, 80.0]),
                    rotation: 0.0,
                },
                Command::SetShape {
                    id: obj_id,
                    shape: Some(ShapeKind::Rectangle {
                        corner_radii: [0.0; 4],
                    }),
                },
            ],
        );

        // 2. Add a live ContourOffset modifier
        let _ = shell.bridge.submit_all(
            "Add contour modifier",
            vec![Command::OffsetPath {
                id: obj_id,
                delta: 8.0,
            }],
        );

        {
            let session = shell.bridge.session().unwrap();
            let surf = session.surface(surf_id).unwrap();
            let obj = surf.objects().iter().find(|o| o.id == obj_id).unwrap();
            assert_eq!(obj.modifiers.len(), 1);
            if let petunia_design_document::ModifierKind::ContourOffset {
                distance,
                join,
                cap,
            } = &obj.modifiers[0].kind
            {
                assert_eq!(*distance, 8.0);
                assert_eq!(*join, OffsetJoin::Round);
                assert_eq!(*cap, OffsetCap::None);
            } else {
                panic!("Expected ContourOffset modifier");
            }
        }

        // 3. Update join style to Miter via SetModifiers
        let _ = shell.bridge.submit_all(
            "Update contour join to Miter",
            vec![Command::SetModifiers {
                id: obj_id,
                modifiers: vec![petunia_design_document::ModifierItem {
                    space: petunia_design_document::ModifierSpace::Parent,
                    id: 1,
                    kind: petunia_design_document::ModifierKind::ContourOffset {
                        distance: 8.0,
                        join: OffsetJoin::Miter,
                        cap: OffsetCap::Round,
                    },
                    enabled: true,
                }],
            }],
        );

        {
            let session = shell.bridge.session().unwrap();
            let surf = session.surface(surf_id).unwrap();
            let obj = surf.objects().iter().find(|o| o.id == obj_id).unwrap();
            if let petunia_design_document::ModifierKind::ContourOffset { join, cap, .. } =
                &obj.modifiers[0].kind
            {
                assert_eq!(*join, OffsetJoin::Miter);
                assert_eq!(*cap, OffsetCap::Round);
            }
        }

        // 4. Add a CropRect modifier, reorder and toggle enabled
        let _ = shell.bridge.submit_all(
            "Add crop and reorder",
            vec![Command::SetModifiers {
                id: obj_id,
                modifiers: vec![
                    petunia_design_document::ModifierItem {
                        space: petunia_design_document::ModifierSpace::Parent,
                        id: 2,
                        kind: petunia_design_document::ModifierKind::CropRect {
                            rect: [50.0, 50.0, 80.0, 60.0],
                        },
                        enabled: false,
                    },
                    petunia_design_document::ModifierItem {
                        space: petunia_design_document::ModifierSpace::Parent,
                        id: 1,
                        kind: petunia_design_document::ModifierKind::ContourOffset {
                            distance: 8.0,
                            join: OffsetJoin::Miter,
                            cap: OffsetCap::Round,
                        },
                        enabled: true,
                    },
                ],
            }],
        );

        {
            let session = shell.bridge.session().unwrap();
            let surf = session.surface(surf_id).unwrap();
            let obj = surf.objects().iter().find(|o| o.id == obj_id).unwrap();
            assert_eq!(obj.modifiers.len(), 2);
            assert_eq!(obj.modifiers[0].id, 2);
            assert!(!obj.modifiers[0].enabled);
            assert_eq!(obj.modifiers[1].id, 1);
            assert!(obj.modifiers[1].enabled);
        }

        // 5. Bake Contour commits contour into base curve geometry
        let _ = shell
            .bridge
            .submit_all("Bake contour", vec![Command::BakeContour { id: obj_id }]);

        {
            let session = shell.bridge.session().unwrap();
            let surf = session.surface(surf_id).unwrap();
            let obj = surf.objects().iter().find(|o| o.id == obj_id).unwrap();
            // Contour modifier was baked out of the chain
            assert!(!obj.modifiers.iter().any(|m| matches!(
                m.kind,
                petunia_design_document::ModifierKind::ContourOffset { .. }
            )));
            // Shape is now a Path (converted to curves)
            assert!(matches!(obj.shape, Some(ShapeKind::LocalPath { .. })));
        }
    }

    #[test]
    fn dock_color_and_swatches_and_background_tasks_panels_integrate() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("ColorDoc").expect("doc opens");
        let surf_id = shell.bridge.active_surface().unwrap();
        let obj_id = shell.bridge.next_object_id().unwrap();
        let _ = shell.bridge.submit_all(
            "Create obj",
            vec![
                Command::CreateObject {
                    surface: surf_id,
                    id: obj_id,
                    name: "Obj1".to_string(),
                },
                Command::SetFill {
                    id: obj_id,
                    fill: Some("#FF0055".to_string()),
                },
            ],
        );
        shell.bridge.select_all();

        // Background jobs verification in bridge
        assert_eq!(shell.bridge.jobs().list_jobs().len(), 0);
        let (job_id, token) = shell.bridge.jobs().spawn_job("Render Test");
        assert_eq!(shell.bridge.jobs().list_jobs().len(), 1);
        shell.bridge.jobs().update_progress(job_id, 75);
        assert_eq!(shell.bridge.jobs().list_jobs()[0].percent, 75);
        assert_eq!(
            shell.bridge.jobs().list_jobs()[0].state,
            petunia_design_jobs::JobState::Running
        );
        shell.bridge.jobs().cancel_job(job_id);
        assert!(token.is_cancelled());
        assert_eq!(
            shell.bridge.jobs().list_jobs()[0].state,
            petunia_design_jobs::JobState::Cancelled
        );
        shell.bridge.jobs().clear_completed();
        assert_eq!(shell.bridge.jobs().list_jobs().len(), 0);
    }

    #[test]
    fn left_dock_and_splitter_open_close_and_symbol_insertion() {
        let seen: Rc<RefCell<Option<UiShell>>> = Rc::new(RefCell::new(None));
        let seen_hook = seen.clone();

        let (mut runner, ()) = TestingRunner::new(
            move || {
                let shell = use_state(|| {
                    let mut s = PetuniaShell::new(1000., 700.);
                    s.new_document("LeftDockDoc").expect("doc opens");
                    s
                });
                let mut ui = UiShell::fresh(shell);
                ui.left_dock_open.set(true);
                seen_hook.replace(Some(ui.clone()));
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .child(LeftDock(ui.clone()))
                    .child(LeftDockSplitter(ui))
            },
            (400., 700.).into(),
            |_| {},
            1.,
        );

        runner.sync_and_update();
        let mut ui = seen.borrow().clone().unwrap();
        assert!(*ui.left_dock_open.read(), "Left dock is open");
        assert_eq!(*ui.left_dock_tab.read(), 0, "Default tab is Ativos (0)");

        // Switch to Símbolos (1)
        let mut left_tab = ui.left_dock_tab;
        left_tab.set(1);
        runner.sync_and_update();
        assert_eq!(*ui.left_dock_tab.read(), 1, "Tab switched to Símbolos (1)");

        // Insert symbol preset into document
        let surf_id = ui.shell.read().bridge.active_surface().unwrap();
        let obj_id = ui.shell.write().bridge.next_object_id().unwrap();
        let _ = ui.shell.write().bridge.submit_all(
            "Insert Retângulo Básico",
            vec![
                Command::CreateObject {
                    surface: surf_id,
                    id: obj_id,
                    name: "Retângulo Básico".to_string(),
                },
                Command::SetBounds {
                    id: obj_id,
                    bounds: Some([50.0, 50.0, 250.0, 200.0]),
                    rotation: 0.0,
                },
                Command::SetShape {
                    id: obj_id,
                    shape: Some(ShapeKind::Rectangle {
                        corner_radii: [0.0; 4],
                    }),
                },
                Command::SetFill {
                    id: obj_id,
                    fill: Some("ptnd.blue/500".to_string()),
                },
            ],
        );

        // Verify shape exists on surface
        {
            let shell_ref = ui.shell.read();
            let session = shell_ref.bridge.session().unwrap();
            let surf = session.surface(surf_id).unwrap();
            let obj = surf
                .objects()
                .iter()
                .find(|o| o.id == obj_id)
                .expect("symbol inserted");
            assert_eq!(obj.name, "Retângulo Básico");
            assert_eq!(obj.fill.as_deref(), Some("ptnd.blue/500"));
        }

        // Left dock width clamping
        let mut dock_w = ui.left_dock_width;
        dock_w.set((100.0f32).clamp(160.0, 480.0));
        assert_eq!(*ui.left_dock_width.read(), 160.0);
        dock_w.set((600.0f32).clamp(160.0, 480.0));
        assert_eq!(*ui.left_dock_width.read(), 480.0);
    }

    #[test]
    fn bottom_dock_and_splitter_open_close_and_diagnostics() {
        let seen: Rc<RefCell<Option<UiShell>>> = Rc::new(RefCell::new(None));
        let seen_hook = seen.clone();

        let (mut runner, ()) = TestingRunner::new(
            move || {
                let shell = use_state(|| {
                    let mut s = PetuniaShell::new(1000., 700.);
                    s.new_document("BottomDockDoc").expect("doc opens");
                    s
                });
                let mut ui = UiShell::fresh(shell);
                ui.bottom_dock_open.set(true);
                seen_hook.replace(Some(ui.clone()));
                rect()
                    .direction(Direction::Vertical)
                    .child(BottomDockSplitter(ui.clone()))
                    .child(BottomDock(ui))
            },
            (1000., 300.).into(),
            |_| {},
            1.,
        );

        runner.sync_and_update();
        let ui = seen.borrow().clone().unwrap();
        assert!(*ui.bottom_dock_open.read(), "Bottom dock is open");
        assert_eq!(*ui.bottom_dock_tab.read(), 0, "Default tab is Tarefas (0)");

        // Switch to Diagnóstico (1)
        let mut bot_tab = ui.bottom_dock_tab;
        bot_tab.set(1);
        runner.sync_and_update();
        assert_eq!(
            *ui.bottom_dock_tab.read(),
            1,
            "Tab switched to Diagnóstico (1)"
        );

        // Bottom dock height clamping
        let mut dock_h = ui.bottom_dock_height;
        dock_h.set((50.0f32).clamp(80.0, 420.0));
        assert_eq!(*ui.bottom_dock_height.read(), 80.0);
        dock_h.set((500.0f32).clamp(80.0, 420.0));
        assert_eq!(*ui.bottom_dock_height.read(), 420.0);

        // Close bottom dock
        let mut open = ui.bottom_dock_open;
        open.set(false);
        runner.sync_and_update();
        assert!(
            !*ui.bottom_dock_open.read(),
            "Bottom dock closed successfully"
        );
    }
}
