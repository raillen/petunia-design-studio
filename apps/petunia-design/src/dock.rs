//! Right dock component: Layers, Properties/Inspector, Colors, and History.
//!
//! Provides direct visual inspection, layer hierarchy control, typography editing,
//! shape properties, and styling for objects in the active document.

use freya::prelude::*;
use petunia_design_application::Command;
use petunia_design_document::ShapeKind;
use petunia_design_foundation::ObjectId;
use petunia_design_shell::panels::LayersPanelController;

use crate::actions::run_action_token;
use crate::theme;
use crate::ui_state::UiShell;

/// The right dock surface containing tabbed inspectors (Layers, Properties, Colors, History).
#[derive(Clone, PartialEq)]
pub struct RightDock(pub UiShell);

impl Component for RightDock {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let mut dock_tab = ui.dock_tab;
        let active_tab = *dock_tab.read();
        let dock_w = *ui.dock_width.read();

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
                    .width(Size::fill())
                    .height(Size::px(34.))
                    .background(theme::SURFACE_CHROME)
                    .cross_align(Alignment::Center)
                    .main_align(Alignment::SpaceEvenly)
                    .child(tab_button(ui, "Camadas", 0, active_tab == 0, &mut dock_tab))
                    .child(tab_button(ui, "Propriedades", 1, active_tab == 1, &mut dock_tab))
                    .child(tab_button(ui, "Cores", 2, active_tab == 2, &mut dock_tab))
                    .child(tab_button(ui, "Histórico", 3, active_tab == 3, &mut dock_tab))
                    .child(tab_button(ui, "Navegador", 4, active_tab == 4, &mut dock_tab)),
            )
            .child(
                // Tab Content Body
                rect()
                    .direction(Direction::Vertical)
                    .width(Size::fill())
                    .height(Size::flex(1.0))
                    .padding(Gaps::new_all(theme::SPACE_2))
                    .child(match active_tab {
                        0 => layers_tab(ui.clone()).into_element(),
                        1 => properties_tab(ui.clone()).into_element(),
                        2 => colors_tab(ui.clone()).into_element(),
                        3 => history_tab(ui.clone()).into_element(),
                        4 => navigator_tab(ui.clone()).into_element(),
                        _ => layers_tab(ui.clone()).into_element(),
                    }),
            )
    }
}

fn tab_button(
    _ui: &UiShell,
    title: &'static str,
    index: usize,
    active: bool,
    dock_tab: &mut State<usize>,
) -> impl IntoElement {
    let mut tab_state = *dock_tab;
    rect()
        .height(Size::fill())
        .padding(Gaps::new(0., theme::SPACE_2, 0., theme::SPACE_2))
        .center()
        .background(if active {
            theme::SURFACE_PANEL
        } else {
            Color::TRANSPARENT
        })
        .on_press(move |_| {
            tab_state.set(index);
        })
        .child(
            label()
                .text(title)
                .font_size(12.)
                .color(if active {
                    theme::TEXT_PRIMARY
                } else {
                    theme::TEXT_TERTIARY
                }),
        )
}

// =========================================================================
// 1. Layers Tab
// =========================================================================

fn layers_tab(ui: UiShell) -> impl IntoElement {
    let shell = ui.shell;
    let layers_model = shell.peek().query_layers();
    let total_rows = layers_model.rows.len();

    let mut shell_for_group = shell;
    let mut shell_for_ungroup = shell;
    let mut shell_for_delete = shell;
    let mut shell_for_front = shell;
    let mut shell_for_back = shell;

    rect()
        .direction(Direction::Vertical)
        .content(Content::Flex)
        .width(Size::fill())
        .height(Size::fill())
        .spacing(theme::SPACE_2)
        .child(
            // Header with action bar
            rect()
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .cross_align(Alignment::Center)
                .main_align(Alignment::SpaceBetween)
                .child(
                    label()
                        .text(format!("Camadas ({total_rows})"))
                        .color(theme::TEXT_PRIMARY)
                        .font_size(13.),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal)
                        .spacing(theme::SPACE_1)
                        .cross_align(Alignment::Center)
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    let _ = run_action_token(
                                        &mut shell_for_group.write(),
                                        "ptnd.action.object.group",
                                    );
                                })
                                .child(label().text("Agrupar").font_size(10.)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    let _ = run_action_token(
                                        &mut shell_for_ungroup.write(),
                                        "ptnd.action.object.ungroup",
                                    );
                                })
                                .child(label().text("Desagrupar").font_size(10.)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    let _ = run_action_token(
                                        &mut shell_for_front.write(),
                                        "ptnd.action.object.arrange.front",
                                    );
                                })
                                .child(label().text("▲ Topo").font_size(10.)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    let _ = run_action_token(
                                        &mut shell_for_back.write(),
                                        "ptnd.action.object.arrange.back",
                                    );
                                })
                                .child(label().text("▼ Fundo").font_size(10.)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    let _ = run_action_token(
                                        &mut shell_for_delete.write(),
                                        "ptnd.action.edit.delete",
                                    );
                                })
                                .child(label().text("Excluir").font_size(10.)),
                        ),
                ),
        )
        .child(
            // List of rows
            rect()
                .direction(Direction::Vertical)
                .width(Size::fill())
                .height(Size::flex(1.0))
                .spacing(2.)
                .children(layers_model.rows.into_iter().map(move |row| {
                    let id = row.id;
                    let is_selected = row.is_selected;
                    let name = row.name.clone();
                    let depth = row.depth;
                    let visible = row.visible;
                    let locked = row.locked;

                    let mut shell_for_select = shell;
                    let mut shell_for_vis = shell;
                    let mut shell_for_lock = shell;
                    let mut shell_for_up = shell;
                    let mut shell_for_down = shell;
                    let mut shell_for_rename = shell;
                    let current_name = name.clone();

                    rect()
                        .direction(Direction::Horizontal)
                        .width(Size::fill())
                        .height(Size::px(28.))
                        .padding(Gaps::new(
                            2.,
                            theme::SPACE_1,
                            2.,
                            (depth as f32 * 14.0) + theme::SPACE_1,
                        ))
                        .cross_align(Alignment::Center)
                        .main_align(Alignment::SpaceBetween)
                        .background(if is_selected {
                            theme::SURFACE_CHROME_STRONG
                        } else {
                            Color::TRANSPARENT
                        })
                        .on_press(move |_| {
                            shell_for_select.write().bridge.set_selection(vec![id]);
                        })
                        .child(
                            rect()
                                .direction(Direction::Horizontal)
                                .cross_align(Alignment::Center)
                                .spacing(theme::SPACE_1)
                                .child(
                                    label()
                                        .text(if row.is_container {
                                            "📁"
                                        } else if name.contains("Text") {
                                            "🔤"
                                        } else {
                                            "🔷"
                                        })
                                        .font_size(12.),
                                )
                                .child(
                                    label()
                                        .text(name)
                                        .font_size(12.)
                                        .color(if is_selected {
                                            theme::TEXT_PRIMARY
                                        } else {
                                            theme::TEXT_SECONDARY
                                        }),
                                ),
                        )
                        .child(
                            rect()
                                .direction(Direction::Horizontal)
                                .cross_align(Alignment::Center)
                                .spacing(theme::SPACE_1)
                                .child(
                                    rect()
                                        .padding(Gaps::new_all(2.))
                                        .on_press(move |_| {
                                            let surf = shell_for_up.peek().bridge.active_surface();
                                            if let Some(surf) = surf {
                                                let _ = LayersPanelController::new().arrange_row(
                                                    &mut shell_for_up.write().bridge,
                                                    surf,
                                                    id,
                                                    petunia_design_document::ArrangePosition::Forward,
                                                );
                                            }
                                        })
                                        .child(
                                            label()
                                                .text("▲")
                                                .font_size(10.)
                                                .color(theme::TEXT_SECONDARY),
                                        ),
                                )
                                .child(
                                    rect()
                                        .padding(Gaps::new_all(2.))
                                        .on_press(move |_| {
                                            let surf = shell_for_down.peek().bridge.active_surface();
                                            if let Some(surf) = surf {
                                                let _ = LayersPanelController::new().arrange_row(
                                                    &mut shell_for_down.write().bridge,
                                                    surf,
                                                    id,
                                                    petunia_design_document::ArrangePosition::Backward,
                                                );
                                            }
                                        })
                                        .child(
                                            label()
                                                .text("▼")
                                                .font_size(10.)
                                                .color(theme::TEXT_SECONDARY),
                                        ),
                                )
                                .child(
                                    rect()
                                        .padding(Gaps::new_all(2.))
                                        .on_press(move |_| {
                                            let new_name = if current_name.ends_with(" *") {
                                                current_name.trim_end_matches(" *").to_string()
                                            } else {
                                                format!("{} *", current_name)
                                            };
                                            let _ = LayersPanelController::new().rename_row(
                                                &mut shell_for_rename.write().bridge,
                                                id,
                                                new_name,
                                            );
                                        })
                                        .child(
                                            label()
                                                .text("✏️")
                                                .font_size(10.),
                                        ),
                                )
                                .child(
                                    rect()
                                        .padding(Gaps::new_all(2.))
                                        .on_press(move |_| {
                                            let _ = LayersPanelController::new().toggle_visibility(
                                                &mut shell_for_vis.write().bridge,
                                                id,
                                            );
                                        })
                                        .child(
                                            label()
                                                .text(if visible { "👁" } else { "🚫" })
                                                .font_size(11.),
                                        ),
                                )
                                .child(
                                    rect()
                                        .padding(Gaps::new_all(2.))
                                        .on_press(move |_| {
                                            let _ = LayersPanelController::new().toggle_lock(
                                                &mut shell_for_lock.write().bridge,
                                                id,
                                            );
                                        })
                                        .child(
                                            label()
                                                .text(if locked { "🔒" } else { "🔓" })
                                                .font_size(11.),
                                        ),
                                ),
                        )
                })),
        )
}

// =========================================================================
// 2. Properties Tab (Transform, Typography, Appearance, Booleans, Alignment)
// =========================================================================

fn properties_tab(ui: UiShell) -> impl IntoElement {
    let shell = ui.shell;
    let props = shell.peek().bridge.query_properties();
    let sel = shell.peek().bridge.selection();

    if props.selection_empty {
        return rect()
            .width(Size::fill())
            .height(Size::fill())
            .center()
            .child(
                label()
                    .text("Nenhum objeto selecionado.\nClique em um objeto para editar.")
                    .color(theme::TEXT_TERTIARY)
                    .font_size(12.),
            )
            .into_element();
    }

    let first_id = sel.selected_ids.first().copied();
    let selected_obj = first_id.and_then(|id| {
        shell.peek().bridge.session().and_then(|s| s.find_object(id)).cloned()
    });

    let (is_text, text_content, font_size) = if let Some(ref obj) = selected_obj {
        if let Some(ShapeKind::Text { ref content, font_size, .. }) = obj.shape {
            (true, content.clone(), font_size)
        } else {
            (false, String::new(), 16.0)
        }
    } else {
        (false, String::new(), 16.0)
    };

    let star_params = selected_obj.as_ref().and_then(|obj| {
        if let Some(ShapeKind::Star { points, inner_ratio }) = obj.shape {
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

    let contour_distance = selected_obj.as_ref().and_then(|obj| {
        obj.modifiers.iter().find_map(|m| {
            if let petunia_design_document::ModifierKind::ContourOffset { distance, .. } = m.kind {
                Some(distance)
            } else {
                None
            }
        })
    });

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
                if let petunia_design_document::EffectKind::DropShadow { offset, blur, color, opacity } = &e.kind {
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
                    section_header("TRANSFORMAÇÃO"),
                )
        .child(
            rect()
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .cross_align(Alignment::Center)
                .main_align(Alignment::SpaceBetween)
                .child(value_pill("X", format!("{:.1}", x)))
                .child(value_pill("Y", format!("{:.1}", y)))
                .child(value_pill("W", format!("{:.1}", w)))
                .child(value_pill("H", format!("{:.1}", h))),
        )
        .child(
            // Quick nudge/resize buttons
            rect()
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .child(nudge_button(shell, first_id, "W -10", -10.0, 0.0, -10.0, 0.0))
                .child(nudge_button(shell, first_id, "W +10", 0.0, 0.0, 10.0, 0.0))
                .child(nudge_button(shell, first_id, "H -10", 0.0, -10.0, 0.0, -10.0))
                .child(nudge_button(shell, first_id, "H +10", 0.0, 0.0, 0.0, 10.0)),
        )
        .maybe(is_text, |el| {
            let mut edit_content = ui.text_edit_content;
            if edit_content.read().is_empty() && !text_content.is_empty() {
                edit_content.set(text_content.clone());
            }
            let current_input = edit_content.read().clone();
            let mut shell_for_text = shell;
            let mut shell_for_size_down = shell;
            let mut shell_for_size_up = shell;
            let text_for_down = text_content.clone();
            let text_for_up = text_content.clone();

            el.child(section_header("TIPOGRAFIA & TEXTO"))
                .child(
                    rect()
                        .direction(Direction::Vertical)
                        .width(Size::fill())
                        .spacing(theme::SPACE_1)
                        .child(
                            rect()
                                .width(Size::fill())
                                .child(Input::new(edit_content).placeholder("Digite o texto aqui...")),
                        )
                        .child(
                            rect()
                                .direction(Direction::Horizontal)
                                .width(Size::fill())
                                .main_align(Alignment::SpaceBetween)
                                .cross_align(Alignment::Center)
                                .child(
                                    Button::new()
                                        .on_press(move |_| {
                                            if let Some(id) = first_id {
                                                let shape = ShapeKind::Text {
                                                    content: current_input.clone(),
                                                    font_family: "Inter".to_string(),
                                                    font_size,
                                                    line_height: 1.2,
                                                    letter_spacing: 0.0,
                                                    on_path: None,
                                                };
                                                let _ = shell_for_text.write().bridge.submit_all(
                                                    "Set text",
                                                    vec![Command::SetShape {
                                                        id,
                                                        shape: Some(shape),
                                                    }],
                                                );
                                            }
                                        })
                                        .child(label().text("Aplicar Texto").font_size(11.)),
                                )
                                .child(
                                    rect()
                                        .direction(Direction::Horizontal)
                                        .spacing(theme::SPACE_1)
                                        .cross_align(Alignment::Center)
                                        .child(
                                            Button::new()
                                                .on_press(move |_| {
                                                if let Some(id) = first_id {
                                                    let new_size = (font_size - 4.0).max(8.0);
                                                    let shape = ShapeKind::Text {
                                                        content: text_for_down.clone(),
                                                        font_family: "Inter".to_string(),
                                                        font_size: new_size,
                                                        line_height: 1.2,
                                                        letter_spacing: 0.0,
                                                        on_path: None,
                                                    };
                                                    let _ = shell_for_size_down.write().bridge.submit_all(
                                                        "Set font size",
                                                        vec![Command::SetShape {
                                                            id,
                                                            shape: Some(shape),
                                                        }],
                                                    );
                                                }
                                                })
                                                .child(label().text("A-").font_size(11.)),
                                        )
                                        .child(
                                            label()
                                                .text(format!("{:.0}pt", font_size))
                                                .font_size(12.)
                                                .color(theme::TEXT_PRIMARY),
                                        )
                                        .child(
                                            Button::new()
                                                .on_press(move |_| {
                                                if let Some(id) = first_id {
                                                    let new_size = (font_size + 4.0).min(144.0);
                                                    let shape = ShapeKind::Text {
                                                        content: text_for_up.clone(),
                                                        font_family: "Inter".to_string(),
                                                        font_size: new_size,
                                                        line_height: 1.2,
                                                        letter_spacing: 0.0,
                                                        on_path: None,
                                                    };
                                                    let _ = shell_for_size_up.write().bridge.submit_all(
                                                        "Set font size",
                                                        vec![Command::SetShape {
                                                            id,
                                                            shape: Some(shape),
                                                        }],
                                                    );
                                                }
                                                })
                                                .child(label().text("A+").font_size(11.)),
                                        ),
                                ),
                        ),
                )
        })
        .child(
            // Section: Appearance
            section_header("APARÊNCIA & CORES"),
        )
        .child(
            rect()
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .cross_align(Alignment::Center)
                .main_align(Alignment::SpaceBetween)
                .child(
                    label()
                        .text(format!("Opacidade: {}%", opacity))
                        .font_size(11.)
                        .color(theme::TEXT_SECONDARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal)
                        .spacing(2.)
                        .child(opacity_button(shell, first_id, "25%", 0.25))
                        .child(opacity_button(shell, first_id, "50%", 0.50))
                        .child(opacity_button(shell, first_id, "75%", 0.75))
                        .child(opacity_button(shell, first_id, "100%", 1.0)),
                ),
        )
        .child(
            rect()
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .cross_align(Alignment::Center)
                .main_align(Alignment::SpaceBetween)
                .child(
                    label()
                        .text(format!("Traço: {:.1} pt", stroke_width))
                        .font_size(11.)
                        .color(theme::TEXT_SECONDARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal)
                        .spacing(theme::SPACE_1)
                        .child(stroke_button(shell, first_id, "- 1pt", -1.0))
                        .child(stroke_button(shell, first_id, "+ 1pt", 1.0)),
                ),
        )
        .child(
            // Quick Swatches for Fill
            rect()
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .child(quick_color_swatch(shell, first_id, "ptnd.gray/900", Color::from_rgb(0x20, 0x21, 0x24)))
                .child(quick_color_swatch(shell, first_id, "ptnd.blue/500", Color::from_rgb(0x3B, 0x82, 0xF6)))
                .child(quick_color_swatch(shell, first_id, "ptnd.purple/500", Color::from_rgb(0x8B, 0x5C, 0xF6)))
                .child(quick_color_swatch(shell, first_id, "ptnd.teal/500", Color::from_rgb(0x14, 0xB8, 0xA6)))
                .child(quick_color_swatch(shell, first_id, "ptnd.red/500", Color::from_rgb(0xEF, 0x44, 0x44)))
                .child(quick_color_swatch(shell, first_id, "ptnd.amber/500", Color::from_rgb(0xF5, 0x9E, 0x0B))),
        )
        .maybe(star_params.is_some(), |el| {
            let (points, inner_ratio) = star_params.unwrap();
            el.child(section_header("FORMA PARAMÉTRICA — ESTRELA"))
                .child(
                    rect()
                        .direction(Direction::Horizontal)
                        .width(Size::fill())
                        .main_align(Alignment::SpaceBetween)
                        .cross_align(Alignment::Center)
                        .child(
                            label()
                                .text(format!("Pontas: {} | Raio: {:.0}%", points, inner_ratio * 100.0))
                                .font_size(11.)
                                .color(theme::TEXT_SECONDARY),
                        )
                        .child(
                            rect()
                                .direction(Direction::Horizontal)
                                .spacing(2.)
                                .child(star_points_button(shell, first_id, "-1", -1))
                                .child(star_points_button(shell, first_id, "+1", 1))
                                .child(star_ratio_button(shell, first_id, "-5%", -0.05))
                                .child(star_ratio_button(shell, first_id, "+5%", 0.05)),
                        ),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal)
                        .width(Size::fill())
                        .main_align(Alignment::End)
                        .child(convert_to_curves_button(shell, first_id)),
                )
        })
        .maybe(polygon_params.is_some(), |el| {
            let sides = polygon_params.unwrap();
            el.child(section_header("FORMA PARAMÉTRICA — POLÍGONO"))
                .child(
                    rect()
                        .direction(Direction::Horizontal)
                        .width(Size::fill())
                        .main_align(Alignment::SpaceBetween)
                        .cross_align(Alignment::Center)
                        .child(
                            label()
                                .text(format!("Lados: {}", sides))
                                .font_size(11.)
                                .color(theme::TEXT_SECONDARY),
                        )
                        .child(
                            rect()
                                .direction(Direction::Horizontal)
                                .spacing(2.)
                                .child(polygon_sides_button(shell, first_id, "-1", -1))
                                .child(polygon_sides_button(shell, first_id, "+1", 1)),
                        ),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal)
                        .width(Size::fill())
                        .main_align(Alignment::End)
                        .child(convert_to_curves_button(shell, first_id)),
                )
        })
        .maybe(rect_corners.is_some(), |el| {
            let r = rect_corners.unwrap();
            el.child(section_header("FORMA PARAMÉTRICA — CANTOS"))
                .child(
                    rect()
                        .direction(Direction::Horizontal)
                        .width(Size::fill())
                        .main_align(Alignment::SpaceBetween)
                        .cross_align(Alignment::Center)
                        .child(
                            label()
                                .text(format!("Raio: {:.1} pt", r))
                                .font_size(11.)
                                .color(theme::TEXT_SECONDARY),
                        )
                        .child(
                            rect()
                                .direction(Direction::Horizontal)
                                .spacing(2.)
                                .child(corner_radius_button(shell, first_id, "-2pt", -2.0))
                                .child(corner_radius_button(shell, first_id, "+2pt", 2.0))
                                .child(bake_corners_button(shell, first_id)),
                        ),
                )
        })
        .child(
            // Section: Live Modifiers (ADR 09.31)
            section_header("MODIFICADORES VIVOS (ADR 09.31)"),
        )
        .child(
            rect()
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text(match contour_distance {
                            Some(d) => format!("Contorno: {:.1} pt", d),
                            None => "Contorno Vivo: Inativo".to_string(),
                        })
                        .font_size(11.)
                        .color(theme::TEXT_SECONDARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal)
                        .spacing(2.)
                        .child(contour_offset_button(shell, first_id, "-2pt", -2.0))
                        .child(contour_offset_button(shell, first_id, "+2pt", 2.0))
                        .child(bake_contour_button(shell, first_id)),
                ),
        )
        .child(
            // Section: Effects & Live Filters (10.4 / 10.10)
            section_header("EFEITOS (FX) & FILTROS VIVOS"),
        )
        .child(
            rect()
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text(match gaussian_blur_effect {
                            Some((_, r, true)) => format!("Desfoque: {:.1} pt", r),
                            Some((_, r, false)) => format!("Desfoque (Oculto): {:.1} pt", r),
                            None => "Desfoque: Nenhum".to_string(),
                        })
                        .font_size(11.)
                        .color(theme::TEXT_SECONDARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal)
                        .spacing(2.)
                        .child(blur_adjust_button(shell, first_id, "-1pt", -1.0))
                        .child(blur_adjust_button(shell, first_id, "+1pt", 1.0))
                        .child(blur_adjust_button(shell, first_id, "+5pt", 5.0))
                        .child(remove_blur_button(
                            shell,
                            first_id,
                            gaussian_blur_effect.map(|(id, _, _)| id).unwrap_or(101),
                        )),
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
                        .text(match drop_shadow_effect {
                            Some((_, offset, blur, _, _, true)) => {
                                format!("Sombra: Desf {:.0}pt | Dist {:.0}pt", blur, offset[0])
                            }
                            Some((_, _offset, blur, _, _, false)) => {
                                format!("Sombra (Oculta): Desf {:.0}pt", blur)
                            }
                            None => "Sombra: Nenhuma".to_string(),
                        })
                        .font_size(11.)
                        .color(theme::TEXT_SECONDARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal)
                        .spacing(2.)
                        .child(shadow_dist_button(shell, first_id, "Dist -2", -2.0))
                        .child(shadow_dist_button(shell, first_id, "Dist +2", 2.0))
                        .child(shadow_blur_button(shell, first_id, "Desf +2", 2.0))
                        .child(remove_shadow_button(
                            shell,
                            first_id,
                            drop_shadow_effect.map(|(id, ..)| id).unwrap_or(102),
                        )),
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
                        .text(match sharpen_effect {
                            Some((_, r, a, true)) => {
                                format!("Nitidez: Raio {:.1}pt | Qtd {:.0}%", r, a * 100.0)
                            }
                            Some((_, r, _, false)) => format!("Nitidez (Oculta): {:.1}pt", r),
                            None => "Nitidez: Nenhuma".to_string(),
                        })
                        .font_size(11.)
                        .color(theme::TEXT_SECONDARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal)
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
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text(match noise_effect {
                            Some((_, a, mono, true)) => format!(
                                "Ruído: {:.0}% ({})",
                                a * 100.0,
                                if mono { "Mono" } else { "Cor" }
                            ),
                            Some((_, a, _, false)) => format!("Ruído (Oculto): {:.0}%", a * 100.0),
                            None => "Ruído: Nenhum".to_string(),
                        })
                        .font_size(11.)
                        .color(theme::TEXT_SECONDARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal)
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
            // Section: Tonal Adjustments (Spec 10.10)
            section_header("AJUSTES TONAIS (SPEC 10.10)"),
        )
        .child(
            // Preset / Add adjustment buttons row
            rect()
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .child(add_adjustment_button(
                    shell,
                    first_id,
                    "+Níveis",
                    petunia_design_document::adjustments::AdjustmentKind::default_levels(),
                ))
                .child(add_adjustment_button(
                    shell,
                    first_id,
                    "+Curvas",
                    petunia_design_document::adjustments::AdjustmentKind::default_curves(),
                ))
                .child(add_adjustment_button(
                    shell,
                    first_id,
                    "+HSL",
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
                    "+Balanço",
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
            section_header("ALINHAMENTO & BOOLEANOS"),
        )
        .child(
            rect()
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .child(action_button(shell, "ptnd.action.object.align_left", "Esq"))
                .child(action_button(shell, "ptnd.action.object.align_center", "Centro"))
                .child(action_button(shell, "ptnd.action.object.align_right", "Dir"))
                .child(action_button(shell, "ptnd.action.object.align_top", "Topo"))
                .child(action_button(shell, "ptnd.action.object.align_bottom", "Base")),
        )
        .child(
            rect()
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .child(action_button(shell, "ptnd.action.vector.boolean_union", "Unir"))
                .child(action_button(shell, "ptnd.action.vector.boolean_subtract", "Subtrair"))
                .child(action_button(shell, "ptnd.action.vector.boolean_intersect", "Intersec"))
                .child(action_button(shell, "ptnd.action.vector.boolean_xor", "XOR")),
        ))
        .into_element()
}

// =========================================================================
// 3. Colors Tab
// =========================================================================

fn colors_tab(ui: UiShell) -> impl IntoElement {
    let shell = ui.shell;
    let sel = shell.peek().bridge.selection();
    let first_id = sel.selected_ids.first().copied();

    let palette = [
        ("Cinza 900", "ptnd.gray/900", Color::from_rgb(0x11, 0x18, 0x27)),
        ("Cinza 600", "ptnd.gray/600", Color::from_rgb(0x4B, 0x55, 0x63)),
        ("Cinza 300", "ptnd.gray/300", Color::from_rgb(0xD1, 0xD5, 0xDB)),
        ("Branco", "ptnd.white", Color::from_rgb(0xFF, 0xFF, 0xFF)),
        ("Vermelho 500", "ptnd.red/500", Color::from_rgb(0xEF, 0x44, 0x44)),
        ("Rosa 500", "ptnd.pink/500", Color::from_rgb(0xEC, 0x48, 0x99)),
        ("Roxo 500", "ptnd.purple/500", Color::from_rgb(0x8B, 0x5C, 0xF6)),
        ("Azul 500", "ptnd.blue/500", Color::from_rgb(0x3B, 0x82, 0xF6)),
        ("Ciano 500", "ptnd.cyan/500", Color::from_rgb(0x06, 0xB6, 0xD4)),
        ("Teal 500", "ptnd.teal/500", Color::from_rgb(0x14, 0xB8, 0xA6)),
        ("Verde 500", "ptnd.green/500", Color::from_rgb(0x10, 0xB9, 0x81)),
        ("Amarelo 500", "ptnd.amber/500", Color::from_rgb(0xF5, 0x9E, 0x0B)),
    ];

    rect()
        .direction(Direction::Vertical)
        .width(Size::fill())
        .height(Size::fill())
        .spacing(theme::SPACE_2)
        .child(section_header("PALETA DE CORES"))
        .child(
            label()
                .text("Clique em uma cor para aplicar ao preenchimento do objeto selecionado.")
                .font_size(11.)
                .color(theme::TEXT_SECONDARY),
        )
        .child(
            rect()
                .direction(Direction::Vertical)
                .width(Size::fill())
                .spacing(theme::SPACE_1)
                .children(palette.chunks(4).map(|chunk| {
                    rect()
                        .direction(Direction::Horizontal)
                        .width(Size::fill())
                        .main_align(Alignment::SpaceBetween)
                        .children(chunk.iter().map(|&(name, token, color)| {
                            let mut shell_for_swatch = shell;
                            rect()
                                .width(Size::px(60.))
                                .height(Size::px(34.))
                                .background(color)
                                .border(
                                    Border::new()
                                        .fill(theme::SURFACE_CHROME_STRONG)
                                        .width(1.)
                                        .alignment(BorderAlignment::Inner),
                                )
                                .center()
                                .on_press(move |_| {
                                    if let Some(id) = first_id {
                                        let _ = shell_for_swatch.write().bridge.submit_all(
                                            "Set fill",
                                            vec![Command::SetFill {
                                                id,
                                                fill: Some(token.to_string()),
                                            }],
                                        );
                                    }
                                })
                                .child(
                                    label()
                                        .text(name)
                                        .font_size(9.)
                                        .color(if color.r() > 180 && color.g() > 180 {
                                            Color::BLACK
                                        } else {
                                            Color::WHITE
                                        }),
                                )
                        }))
                })),
        )
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
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(section_header("HISTÓRICO DE AÇÕES"))
                .child(
                    rect()
                        .direction(Direction::Horizontal)
                        .spacing(theme::SPACE_1)
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    let _ = run_action_token(
                                        &mut shell_for_undo.write(),
                                        "ptnd.action.edit.undo",
                                    );
                                })
                                .child(label().text("Desfazer").font_size(11.)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    let _ = run_action_token(
                                        &mut shell_for_redo.write(),
                                        "ptnd.action.edit.redo",
                                    );
                                })
                                .child(label().text("Refazer").font_size(11.)),
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

fn navigator_tab(ui: UiShell) -> impl IntoElement {
    let shell = ui.shell;
    let snapshot = shell.peek().canvas_snapshot();
    let zoom_pct = (snapshot.camera.zoom * 100.0) as i32;
    let pan_x = snapshot.camera.pan_x as i32;
    let pan_y = snapshot.camera.pan_y as i32;

    let (surf_w, surf_h) = snapshot
        .surface
        .as_ref()
        .map(|s| (s.bounds[2], s.bounds[3]))
        .unwrap_or((800.0, 600.0));

    // Thumbnail scale: fit within 240px wide, 150px high
    let scale_x = 240.0 / surf_w.max(10.0);
    let scale_y = 150.0 / surf_h.max(10.0);
    let scale = scale_x.min(scale_y);
    let thumb_w = (surf_w * scale) as f32;
    let thumb_h = (surf_h * scale) as f32;

    let mut shell_for_reset = shell;
    let mut shell_for_zoom_in = shell;
    let mut shell_for_zoom_out = shell;

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
                .child(section_header("NAVEGADOR & ZOOM"))
                .child(
                    Button::new()
                        .on_press(move |_| {
                            let _ = run_action_token(
                                &mut shell_for_reset.write(),
                                "ptnd.action.view.fit_surface",
                            );
                        })
                        .child(label().text("Enquadrar").font_size(11.)),
                ),
        )
        .child(
            rect()
                .direction(Direction::Horizontal)
                .spacing(theme::SPACE_1)
                .child(value_pill("Zoom", format!("{}%", zoom_pct)))
                .child(value_pill("X", format!("{} pt", pan_x)))
                .child(value_pill("Y", format!("{} pt", pan_y))),
        )
        .child(
            rect()
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .spacing(theme::SPACE_1)
                .child(
                    Button::new()
                        .on_press(move |_| {
                            let _ = run_action_token(
                                &mut shell_for_zoom_in.write(),
                                "ptnd.action.view.zoom_in",
                            );
                        })
                        .child(label().text("Zoom +").font_size(11.)),
                )
                .child(
                    Button::new()
                        .on_press(move |_| {
                            let _ = run_action_token(
                                &mut shell_for_zoom_out.write(),
                                "ptnd.action.view.zoom_out",
                            );
                        })
                        .child(label().text("Zoom -").font_size(11.)),
                )
                .child(
                    Button::new()
                        .on_press(move |_| {
                            let _ = run_action_token(
                                &mut shell_for_reset.write(),
                                "ptnd.action.view.zoom_100",
                            );
                        })
                        .child(label().text("100%").font_size(11.)),
                ),
        )
        .child(
            // Mini Canvas Viewport Box
            rect()
                .width(Size::fill())
                .height(Size::px(180.))
                .center()
                .background(theme::SURFACE_CHROME)
                .border(
                    Border::new()
                        .fill(theme::SURFACE_CHROME_STRONG)
                        .width(1.)
                        .alignment(BorderAlignment::Inner),
                )
                .child(
                    rect()
                        .width(Size::px(thumb_w.max(20.)))
                        .height(Size::px(thumb_h.max(20.)))
                        .background(Color::WHITE)
                        .border(
                            Border::new()
                                .fill(theme::BLOOM.value)
                                .width(1.)
                                .alignment(BorderAlignment::Inner),
                        )
                        .center()
                        .child(
                            label()
                                .text(format!("{:.0} × {:.0}", surf_w, surf_h))
                                .font_size(9.)
                                .color(Color::from_rgb(0x80, 0x80, 0x80)),
                        ),
                ),
        )
}

// =========================================================================
// Helper Widgets
// =========================================================================

fn section_header(title: &'static str) -> impl IntoElement {
    label()
        .text(title)
        .font_size(11.)
        .color(theme::TEXT_TERTIARY)
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

fn nudge_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    text: &'static str,
    _dx: f64,
    _dy: f64,
    dw: f64,
    dh: f64,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let current_bounds = shell
                    .peek()
                    .bridge
                    .session()
                    .and_then(|s| s.find_object(id))
                    .and_then(|o| o.bounds);
                if let Some([x, y, w, h]) = current_bounds {
                    let new_w = (w + dw).max(10.0);
                    let new_h = (h + dh).max(10.0);
                    let _ = shell.write().bridge.submit_all(
                        "Set bounds",
                        vec![Command::SetBounds {
                            id,
                            bounds: Some([x, y, new_w, new_h]),
                            rotation: 0.0,
                        }],
                    );
                }
            }
        })
        .child(label().text(text).font_size(10.))
}

fn opacity_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: &'static str,
    val: f64,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let _ = shell.write().bridge.submit_all(
                    "Set opacity",
                    vec![Command::SetOpacity {
                        id,
                        opacity: val,
                    }],
                );
            }
        })
        .child(label().text(label_text).font_size(10.))
}

fn stroke_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: &'static str,
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
                    .map(|o| o.stroke_width)
                    .unwrap_or(1.0);
                let new_width = (current + delta).max(0.0);
                let _ = shell.write().bridge.submit_all(
                    "Set stroke width",
                    vec![Command::SetStroke {
                        id,
                        stroke: Some("ptnd.gray/900".to_string()),
                        width: new_width,
                    }],
                );
            }
        })
        .child(label().text(label_text).font_size(10.))
}

fn quick_color_swatch(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    token: &'static str,
    color: Color,
) -> impl IntoElement {
    rect()
        .width(Size::px(26.))
        .height(Size::px(26.))
        .background(color)
        .border(
            Border::new()
                .fill(theme::SURFACE_CHROME_STRONG)
                .width(1.)
                .alignment(BorderAlignment::Inner),
        )
        .on_press(move |_| {
            if let Some(id) = target_id {
                let _ = shell.write().bridge.submit_all(
                    "Set fill",
                    vec![Command::SetFill {
                        id,
                        fill: Some(token.to_string()),
                    }],
                );
            }
        })
}

fn action_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    action_token: &'static str,
    title: &'static str,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            let _ = run_action_token(&mut shell.write(), action_token);
        })
        .child(label().text(title).font_size(11.))
}

fn star_points_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: &'static str,
    delta: i32,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let obj = shell.peek().bridge.session().and_then(|s| s.find_object(id)).cloned();
                if let Some(obj) = obj {
                    if let Some(ShapeKind::Star { points, inner_ratio }) = obj.shape {
                        let new_points = (points as i32 + delta).clamp(3, 36) as u32;
                        let _ = shell.write().bridge.submit_all(
                            "Change star points",
                            vec![Command::SetShape {
                                id,
                                shape: Some(ShapeKind::Star { points: new_points, inner_ratio }),
                            }],
                        );
                    }
                }
            }
        })
        .child(label().text(label_text).font_size(10.))
}

fn star_ratio_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: &'static str,
    delta: f64,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let obj = shell.peek().bridge.session().and_then(|s| s.find_object(id)).cloned();
                if let Some(obj) = obj {
                    if let Some(ShapeKind::Star { points, inner_ratio }) = obj.shape {
                        let new_ratio = (inner_ratio + delta).clamp(0.1, 0.9);
                        let _ = shell.write().bridge.submit_all(
                            "Change star inner ratio",
                            vec![Command::SetShape {
                                id,
                                shape: Some(ShapeKind::Star { points, inner_ratio: new_ratio }),
                            }],
                        );
                    }
                }
            }
        })
        .child(label().text(label_text).font_size(10.))
}

fn polygon_sides_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: &'static str,
    delta: i32,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let obj = shell.peek().bridge.session().and_then(|s| s.find_object(id)).cloned();
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
        .child(label().text(label_text).font_size(10.))
}

fn corner_radius_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: &'static str,
    delta: f64,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let obj = shell.peek().bridge.session().and_then(|s| s.find_object(id)).cloned();
                if let Some(obj) = obj {
                    if let Some(ShapeKind::Rectangle { corner_radii }) = obj.shape {
                        let new_r = (corner_radii[0] + delta).max(0.0);
                        let _ = shell.write().bridge.submit_all(
                            "Change corner radius",
                            vec![Command::SetShape {
                                id,
                                shape: Some(ShapeKind::Rectangle { corner_radii: [new_r, new_r, new_r, new_r] }),
                            }],
                        );
                    }
                }
            }
        })
        .child(label().text(label_text).font_size(10.))
}

fn bake_corners_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let _ = shell.write().bridge.submit_all(
                    "Bake corners",
                    vec![Command::BakeCorners { id }],
                );
            }
        })
        .child(label().text("Fixar").font_size(10.))
}

fn contour_offset_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: &'static str,
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
                            if let petunia_design_document::ModifierKind::ContourOffset { distance, .. } = m.kind {
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
                    vec![Command::OffsetPath { id, delta: new_dist }],
                );
            }
        })
        .child(label().text(label_text).font_size(10.))
}

fn bake_contour_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let _ = shell.write().bridge.submit_all(
                    "Bake contour",
                    vec![Command::BakeContour { id }],
                );
            }
        })
        .child(label().text("Fixar (Bake)").font_size(10.))
}

fn convert_to_curves_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let _ = shell.write().bridge.submit_all(
                    "Convert to curves",
                    vec![Command::ConvertToCurves { id }],
                );
            }
        })
        .child(label().text("Para Curvas").font_size(10.))
}

fn blur_adjust_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: &'static str,
    delta: f64,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let current_radius = shell
                    .peek()
                    .bridge
                    .session()
                    .and_then(|s| s.find_object(id))
                    .and_then(|o| {
                        o.appearance.as_ref().and_then(|app| {
                            app.effects.iter().find_map(|e| {
                                if let petunia_design_document::EffectKind::GaussianBlur { radius } = e.kind {
                                    Some(radius)
                                } else {
                                    None
                                }
                            })
                        })
                    })
                    .unwrap_or(0.0);
                let new_radius = (current_radius + delta).max(0.5);
                let _ = shell.write().bridge.submit_all(
                    "Adjust blur",
                    vec![Command::AddEffect {
                        id,
                        effect: petunia_design_document::EffectItem {
                            id: 101,
                            kind: petunia_design_document::EffectKind::GaussianBlur { radius: new_radius },
                            visible: true,
                        },
                    }],
                );
            }
        })
        .child(label().text(label_text).font_size(10.))
}

fn remove_blur_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    effect_id: u32,
) -> impl IntoElement {
    Button::new()
        .on_press(move |_| {
            if let Some(id) = target_id {
                let _ = shell.write().bridge.submit_all(
                    "Remove blur",
                    vec![Command::RemoveEffect { id, effect_id }],
                );
            }
        })
        .child(label().text("✕").font_size(10.))
}

fn shadow_dist_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: &'static str,
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
                                if let petunia_design_document::EffectKind::DropShadow { offset, blur, color, opacity } = &e.kind {
                                    Some((e.id, *offset, *blur, color.clone(), *opacity))
                                } else {
                                    None
                                }
                            })
                        })
                    });
                let (eff_id, offset, blur, color, opacity) = existing.unwrap_or((102, [4.0, 4.0], 8.0, "ptnd.gray/900".to_string(), 0.6));
                let new_offset = [(offset[0] + delta).max(0.0), (offset[1] + delta).max(0.0)];
                let _ = shell.write().bridge.submit_all(
                    "Adjust shadow distance",
                    vec![Command::AddEffect {
                        id,
                        effect: petunia_design_document::EffectItem {
                            id: eff_id,
                            kind: petunia_design_document::EffectKind::DropShadow {
                                offset: new_offset,
                                blur,
                                color,
                                opacity,
                            },
                            visible: true,
                        },
                    }],
                );
            }
        })
        .child(label().text(label_text).font_size(10.))
}

fn shadow_blur_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: &'static str,
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
                                if let petunia_design_document::EffectKind::DropShadow { offset, blur, color, opacity } = &e.kind {
                                    Some((e.id, *offset, *blur, color.clone(), *opacity))
                                } else {
                                    None
                                }
                            })
                        })
                    });
                let (eff_id, offset, blur, color, opacity) = existing.unwrap_or((102, [4.0, 4.0], 8.0, "ptnd.gray/900".to_string(), 0.6));
                let new_blur = (blur + delta).max(0.0);
                let _ = shell.write().bridge.submit_all(
                    "Adjust shadow blur",
                    vec![Command::AddEffect {
                        id,
                        effect: petunia_design_document::EffectItem {
                            id: eff_id,
                            kind: petunia_design_document::EffectKind::DropShadow {
                                offset,
                                blur: new_blur,
                                color,
                                opacity,
                            },
                            visible: true,
                        },
                    }],
                );
            }
        })
        .child(label().text(label_text).font_size(10.))
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
        .child(label().text("✕").font_size(10.))
}

fn sharpen_adjust_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: &'static str,
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
                                if let petunia_design_document::EffectKind::Sharpen { radius, amount } = e.kind {
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
        .child(label().text(label_text).font_size(10.))
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
        .child(label().text("✕").font_size(10.))
}

fn noise_adjust_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: &'static str,
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
                                if let petunia_design_document::EffectKind::Noise { amount, monochrome } = e.kind {
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
        .child(label().text(label_text).font_size(10.))
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
                                if let petunia_design_document::EffectKind::Noise { amount, monochrome } = e.kind {
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
        .child(label().text("Mono/Cor").font_size(10.))
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
        .child(label().text("✕").font_size(10.))
}

fn add_adjustment_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: &'static str,
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
                let item = petunia_design_document::adjustments::AdjustmentItem::new(next_id, kind.clone());
                let _ = shell.write().bridge.submit_all(
                    "Add tonal adjustment",
                    vec![Command::AddAdjustment {
                        id,
                        adjustment: item,
                    }],
                );
            }
        })
        .child(label().text(label_text).font_size(10.))
}

fn adjustment_card(
    shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    adj: petunia_design_document::adjustments::AdjustmentItem,
) -> impl IntoElement {
    let adj_id = adj.id;
    let title = match &adj.kind {
        petunia_design_document::adjustments::AdjustmentKind::Levels { .. } => format!("Níveis #{}", adj_id),
        petunia_design_document::adjustments::AdjustmentKind::Curves { .. } => format!("Curvas #{}", adj_id),
        petunia_design_document::adjustments::AdjustmentKind::Hsl { .. } => format!("HSL #{}", adj_id),
        petunia_design_document::adjustments::AdjustmentKind::Exposure { .. } => format!("Exposição #{}", adj_id),
        petunia_design_document::adjustments::AdjustmentKind::WhiteBalance { .. } => format!("Balanço B. #{}", adj_id),
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
                        .font_size(10.)
                        .color(theme::TEXT_TERTIARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal)
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
                                .child(label().text("γ -0.1").font_size(10.)),
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
                                .child(label().text("γ +0.1").font_size(10.)),
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
                                .child(label().text("Preto +").font_size(10.)),
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
                                .child(label().text("Branco -").font_size(10.)),
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
                        .text(format!("Pontos da Curva: {}", master_points.len()))
                        .font_size(10.)
                        .color(theme::TEXT_TERTIARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal)
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
                                .child(label().text("Curva S").font_size(10.)),
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
                                .child(label().text("Linear").font_size(10.)),
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
                                .child(label().text("Alto Contraste").font_size(10.)),
                        ),
                )
        }
        petunia_design_document::adjustments::AdjustmentKind::Hsl { hue_shift, saturation, lightness } => {
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
                        .font_size(10.)
                        .color(theme::TEXT_TERTIARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal)
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
                                .child(label().text("H +15°").font_size(10.)),
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
                                .child(label().text("S -10%").font_size(10.)),
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
                                .child(label().text("S +10%").font_size(10.)),
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
                                .child(label().text("L +10%").font_size(10.)),
                        ),
                )
        }
        petunia_design_document::adjustments::AdjustmentKind::Exposure { exposure, offset, gamma } => {
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
                        .font_size(10.)
                        .color(theme::TEXT_TERTIARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal)
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
                                .child(label().text("EV -0.5").font_size(10.)),
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
                                .child(label().text("EV +0.5").font_size(10.)),
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
                                .child(label().text("Off +0.05").font_size(10.)),
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
                                .child(label().text("γ +0.1").font_size(10.)),
                        ),
                )
        }
        petunia_design_document::adjustments::AdjustmentKind::WhiteBalance { temperature, tint } => {
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
                        .font_size(10.)
                        .color(theme::TEXT_TERTIARY),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal)
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
                                .child(label().text("Frio -0.1").font_size(10.)),
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
                                .child(label().text("Quente +0.1").font_size(10.)),
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
                                .child(label().text("Verde -0.1").font_size(10.)),
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
                                .child(label().text("Magenta +0.1").font_size(10.)),
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
                .width(Size::fill())
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(label().text(title).font_size(11.).color(theme::TEXT_PRIMARY))
                .child(
                    Button::new()
                        .on_press(move |_| {
                            if let Some(id) = target_id {
                                let _ = shell_remove.write().bridge.submit_all(
                                    "Remove adjustment",
                                    vec![Command::RemoveAdjustment { id, adjustment_id: adj_id }],
                                );
                            }
                        })
                        .child(label().text("✕").font_size(10.)),
                ),
        )
        .child(body)
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

        // Click on "Propriedades" tab button (roughly x=120, y=17)
        runner.press_cursor((120., 17.));
        runner.release_cursor((120., 17.));
        runner.sync_and_update();

        assert_eq!(*ui.dock_tab.read(), 1, "active tab should switch to Propriedades (1)");

        // Click on "Cores" tab button (roughly x=200, y=17)
        runner.press_cursor((200., 17.));
        runner.release_cursor((200., 17.));
        runner.sync_and_update();

        assert_eq!(*ui.dock_tab.read(), 2, "active tab should switch to Cores (2)");

        // Click on "Histórico" tab button (roughly x=270, y=17)
        runner.press_cursor((270., 17.));
        runner.release_cursor((270., 17.));
        runner.sync_and_update();

        assert_eq!(*ui.dock_tab.read(), 3, "active tab should switch to Histórico (3)");
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
                Command::SetShape {
                    id: obj_id,
                    shape: Some(ShapeKind::Rectangle {
                        corner_radii: [0.0; 4],
                    }),
                },
                Command::SetBounds {
                    id: obj_id,
                    bounds: Some([10.0, 10.0, 100.0, 100.0]),
                    rotation: 0.0,
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
                Command::SetShape {
                    id: obj_id,
                    shape: Some(ShapeKind::Rectangle {
                        corner_radii: [0.0; 4],
                    }),
                },
                Command::SetBounds {
                    id: obj_id,
                    bounds: Some([0.0, 0.0, 100.0, 100.0]),
                    rotation: 0.0,
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
        if let petunia_design_document::adjustments::AdjustmentKind::Levels { ref mut master, .. } = updated_levels.kind {
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
            if let petunia_design_document::adjustments::AdjustmentKind::Levels { master, .. } = app.adjustments[0].kind {
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
}

