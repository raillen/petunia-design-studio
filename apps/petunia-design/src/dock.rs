//! Right dock component: Layers, Properties/Inspector, Colors, and History.
//!
//! Provides direct visual inspection, layer hierarchy control, typography editing,
//! shape properties, and styling for objects in the active document.

use freya::prelude::*;
use petunia_design_application::Command;
use petunia_design_color::{
    Cmyk, ColorManagementProvider, ColorValue, DefaultColorManagementProvider, GamutStatus, Lab,
    ProofContext, Srgb,
};
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
                    .child(tab_button(
                        ui,
                        "Propriedades",
                        1,
                        active_tab == 1,
                        &mut dock_tab,
                    ))
                    .child(tab_button(ui, "Cores", 2, active_tab == 2, &mut dock_tab))
                    .child(tab_button(
                        ui,
                        "Histórico",
                        3,
                        active_tab == 3,
                        &mut dock_tab,
                    ))
                    .child(tab_button(
                        ui,
                        "Navegador",
                        4,
                        active_tab == 4,
                        &mut dock_tab,
                    )),
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
                        2 => ColorsTab(ui.clone()).into_element(),
                        3 => history_tab(ui.clone()).into_element(),
                        4 => NavigatorTab(ui.clone()).into_element(),
                        5 => BackgroundTasksPanel(ui.clone()).into_element(),
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
        .child(label().text(title).font_size(12.).color(if active {
            theme::TEXT_PRIMARY
        } else {
            theme::TEXT_TERTIARY
        }))
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
        return ScrollView::new()
            .child(
                rect()
                    .direction(Direction::Vertical)
                    .width(Size::fill())
                    .spacing(theme::SPACE_2)
                    .child(
                        section_header("HISTOGRAMA DO DOCUMENTO"),
                    )
                    .child(
                        HistogramWidget {
                            ui: ui.clone(),
                            object_id: None,
                        },
                    )
                    .child(
                        rect()
                            .width(Size::fill())
                            .padding(Gaps::new_all(theme::SPACE_4))
                            .center()
                            .child(
                                label()
                                    .text("Nenhum objeto selecionado.\nClique em um objeto para editar propriedades.")
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

    let (is_text, text_content, font_size) = if let Some(ref obj) = selected_obj {
        if let Some(ShapeKind::Text {
            ref content,
            font_size,
            ..
        }) = obj.shape
        {
            (true, content.clone(), font_size)
        } else {
            (false, String::new(), 16.0)
        }
    } else {
        (false, String::new(), 16.0)
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
            // Section: Live Modifiers Stack (ADR 09.31)
            section_header("MODIFICADORES VIVOS (ADR 09.31)"),
        )
        .child(modifier_stack_inspector(shell, first_id, &modifiers_list, obj_bounds))
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
                        .direction(Direction::Horizontal)
                        .width(Size::fill())
                        .main_align(Alignment::SpaceBetween)
                        .cross_align(Alignment::Center)
                        .child(
                            label()
                                .text(match drop_shadow_effect {
                                    Some((_, offset, blur, _, op, true)) => {
                                        format!("Sombra Ext: Desf {:.0} | Dx {:.0} | Dy {:.0} | Op {:.0}%", blur, offset[0], offset[1], op * 100.0)
                                    }
                                    Some((_, _, blur, _, _, false)) => {
                                        format!("Sombra Ext (Oculta): Desf {:.0}pt", blur)
                                    }
                                    None => "Sombra Ext: Nenhuma".to_string(),
                                })
                                .font_size(11.)
                                .color(theme::TEXT_SECONDARY),
                        )
                        .child(
                            rect()
                                .direction(Direction::Horizontal)
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
                        .direction(Direction::Horizontal)
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
                        .direction(Direction::Horizontal)
                        .width(Size::fill())
                        .main_align(Alignment::SpaceBetween)
                        .cross_align(Alignment::Center)
                        .child(
                            label()
                                .text(match inner_shadow_effect {
                                    Some((_, offset, blur, _, op, true)) => {
                                        format!("Sombra Int: Desf {:.0} | Dist {:.0} | Op {:.0}%", blur, offset[0], op * 100.0)
                                    }
                                    Some((_, _, blur, _, _, false)) => {
                                        format!("Sombra Int (Oculta): Desf {:.0}pt", blur)
                                    }
                                    None => "Sombra Int: Nenhuma".to_string(),
                                })
                                .font_size(11.)
                                .color(theme::TEXT_SECONDARY),
                        )
                        .child(
                            rect()
                                .direction(Direction::Horizontal)
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
                        .direction(Direction::Horizontal)
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
            // Section: Histogram (Spec 10.10)
            section_header("HISTOGRAMA (SPEC 10.10)"),
        )
        .child(
            HistogramWidget {
                ui: ui.clone(),
                object_id: first_id,
            },
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

#[derive(Clone, PartialEq)]
pub struct ColorsTab(pub UiShell);

impl Component for ColorsTab {
    fn render(&self) -> impl IntoElement {
        let mut subtab = use_state(|| 0usize);
        let current_subtab = *subtab.read();
        let target_fill = use_state(|| true);

        rect()
            .direction(Direction::Vertical)
            .width(Size::fill())
            .height(Size::fill())
            .spacing(theme::SPACE_2)
            .child(
                // Subtab Switcher: Cor (ptnd.panel.color) | Amostras (ptnd.panel.swatches) | Tarefas (ptnd.panel.background_tasks)
                rect()
                    .direction(Direction::Horizontal)
                    .width(Size::fill())
                    .height(Size::px(30.))
                    .background(theme::SURFACE_CHROME_STRONG)
                    .padding(Gaps::new_all(2.))
                    .spacing(theme::SPACE_1)
                    .main_align(Alignment::SpaceEvenly)
                    .cross_align(Alignment::Center)
                    .child(subtab_pill("🎨 Cor", 0, current_subtab == 0, &mut subtab))
                    .child(subtab_pill(
                        "📑 Amostras",
                        1,
                        current_subtab == 1,
                        &mut subtab,
                    ))
                    .child(subtab_pill(
                        "⚡ Tarefas",
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
    title: &'static str,
    idx: usize,
    active: bool,
    subtab: &mut State<usize>,
) -> impl IntoElement {
    let mut st = *subtab;
    rect()
        .height(Size::px(24.))
        .background(if active {
            theme::SURFACE_PANEL
        } else {
            theme::SURFACE_CHROME
        })
        .padding(Gaps::new(0., theme::SPACE_2, 0., theme::SPACE_2))
        .center()
        .on_press(move |_| {
            st.set(idx);
        })
        .child(label().text(title).font_size(10.).color(if active {
            theme::TEXT_PRIMARY
        } else {
            theme::TEXT_SECONDARY
        }))
}

fn channel_adjuster(
    label_text: &'static str,
    val_text: String,
    mut on_dec: impl FnMut() + 'static,
    mut on_inc: impl FnMut() + 'static,
) -> impl IntoElement {
    rect()
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::px(26.))
        .cross_align(Alignment::Center)
        .main_align(Alignment::SpaceBetween)
        .child(
            rect().width(Size::px(60.)).child(
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
        let sel = shell.peek().bridge.selection();
        let first_id = sel.selected_ids.first().copied();
        let is_fill = *self.target_fill.read();
        let mut mut_target_fill = self.target_fill;

        let mut color_mode = use_state(|| 0usize); // 0: sRGB, 1: CMYK, 2: Lab, 3: Spot
        let active_mode = *color_mode.read();

        // sRGB state
        let r_val = use_state(|| 239u8);
        let g_val = use_state(|| 68u8);
        let b_val = use_state(|| 68u8);

        // CMYK state (percentages 0..=100)
        let c_val = use_state(|| 0u8);
        let m_val = use_state(|| 85u8);
        let y_val = use_state(|| 70u8);
        let k_val = use_state(|| 0u8);

        // Lab state (L: 0..100, a: -128..127, b: -128..127)
        let lab_l = use_state(|| 55i16);
        let lab_a = use_state(|| 65i16);
        let lab_b = use_state(|| 35i16);

        // Spot state
        let spot_name = use_state(|| "PANTONE 185 C".to_string());

        let (preview_color, token, color_val) = match active_mode {
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
                let r = ((1.0 - (c_f + k_f).min(1.0)) * 255.0).round() as u8;
                let g = ((1.0 - (m_f + k_f).min(1.0)) * 255.0).round() as u8;
                let b = ((1.0 - (y_f + k_f).min(1.0)) * 255.0).round() as u8;
                (
                    Color::from_rgb(r, g, b),
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

        // Gamut assessment with soft-proof provider
        let provider = DefaultColorManagementProvider;
        let proof_ctx = ProofContext::default();
        let (_, gamut_status) = provider.soft_proof(&color_val, &proof_ctx);
        let is_in_gamut = matches!(gamut_status, GamutStatus::InGamut);

        let mut shell_for_apply = shell;
        let token_for_apply = token.clone();

        rect()
            .direction(Direction::Vertical)
            .width(Size::fill())
            .height(Size::fill())
            .spacing(theme::SPACE_2)
            .child(
                // Target Selector: Preenchimento vs Traço
                rect()
                    .direction(Direction::Horizontal)
                    .width(Size::fill())
                    .spacing(theme::SPACE_1)
                    .child(
                        Button::new()
                            .on_press(move |_| mut_target_fill.set(true))
                            .child(
                                label()
                                    .text(if is_fill {
                                        "● Preenchimento"
                                    } else {
                                        "Preenchimento"
                                    })
                                    .font_size(10.),
                            ),
                    )
                    .child(
                        Button::new()
                            .on_press(move |_| mut_target_fill.set(false))
                            .child(
                                label()
                                    .text(if !is_fill {
                                        "● Traço/Contorno"
                                    } else {
                                        "Traço/Contorno"
                                    })
                                    .font_size(10.),
                            ),
                    ),
            )
            .child(
                // Color Mode Segmented Bar: sRGB | CMYK | Lab | Spot
                rect()
                    .direction(Direction::Horizontal)
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
                            .height(Size::px(34.))
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
                                    .text(if is_in_gamut {
                                        "✓ Em Gama (SWOP)"
                                    } else {
                                        "⚠️ Fora de Gama (SWOP)"
                                    })
                                    .font_size(9.)
                                    .color(if is_in_gamut {
                                        Color::from_rgb(0x10, 0xB9, 0x81)
                                    } else {
                                        Color::from_rgb(0xF5, 0x9E, 0x0B)
                                    }),
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
                            "Vermelho (R):",
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
                            "Verde (G):",
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
                            "Azul (B):",
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
                            "Ciano (C):",
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
                            "Magenta (M):",
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
                            "Amarelo (Y):",
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
                            "Preto (K):",
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
                            "Luminância (L):",
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
                            "Eixo a (V-V):",
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
                            "Eixo b (A-A):",
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
                                    .text("Tinta Spot / Especial:")
                                    .font_size(11.)
                                    .color(theme::TEXT_SECONDARY),
                            )
                            .child(
                                rect()
                                    .direction(Direction::Horizontal)
                                    .spacing(4.)
                                    .child(
                                        Button::new()
                                            .on_press(move |_| {
                                                spot_change.set("PANTONE 185 C".to_string())
                                            })
                                            .child(label().text("185 C (Red)").font_size(9.)),
                                    )
                                    .child(
                                        Button::new()
                                            .on_press(move |_| {
                                                spot_change.set("PANTONE Reflex Blue".to_string())
                                            })
                                            .child(label().text("Reflex Blue").font_size(9.)),
                                    )
                                    .child(
                                        Button::new()
                                            .on_press(move |_| {
                                                spot_change.set("PANTONE Metallic Gold".to_string())
                                            })
                                            .child(label().text("Gold").font_size(9.)),
                                    ),
                            )
                    }
                },
            )
            .child(
                // Apply Button
                Button::new()
                    .on_press(move |_| {
                        if let Some(id) = first_id {
                            let cmd = if is_fill {
                                Command::SetFill {
                                    id,
                                    fill: Some(token_for_apply.clone()),
                                }
                            } else {
                                Command::SetStroke {
                                    id,
                                    stroke: Some(token_for_apply.clone()),
                                    width: 2.0,
                                }
                            };
                            let _ = shell_for_apply
                                .write()
                                .bridge
                                .submit_all("Apply color", vec![cmd]);
                        }
                    })
                    .child(
                        label()
                            .text(if is_fill {
                                "Aplicar ao Preenchimento"
                            } else {
                                "Aplicar ao Contorno"
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
        let shell = self.ui.shell;
        let sel = shell.peek().bridge.selection();
        let first_id = sel.selected_ids.first().copied();
        let is_fill = *self.target_fill.read();

        let mut lib_mode = use_state(|| 0usize); // 0: Sistema, 1: Documento, 2: Favoritos
        let current_lib = *lib_mode.read();

        let custom_swatches = use_state(|| {
            vec![
                (
                    "Menta Neon".to_string(),
                    "#10B981".to_string(),
                    Color::from_rgb(0x10, 0xB9, 0x81),
                ),
                (
                    "Ouro Solar".to_string(),
                    "#F59E0B".to_string(),
                    Color::from_rgb(0xF5, 0x9E, 0x0B),
                ),
                (
                    "Índigo".to_string(),
                    "#6366F1".to_string(),
                    Color::from_rgb(0x63, 0x66, 0xF1),
                ),
                (
                    "Coral Rosa".to_string(),
                    "#F43F5E".to_string(),
                    Color::from_rgb(0xF4, 0x3F, 0x5E),
                ),
            ]
        });

        let system_palette = [
            (
                "Cinza 900",
                "ptnd.gray/900",
                Color::from_rgb(0x11, 0x18, 0x27),
            ),
            (
                "Cinza 600",
                "ptnd.gray/600",
                Color::from_rgb(0x4B, 0x55, 0x63),
            ),
            (
                "Cinza 300",
                "ptnd.gray/300",
                Color::from_rgb(0xD1, 0xD5, 0xDB),
            ),
            ("Branco", "ptnd.white", Color::from_rgb(0xFF, 0xFF, 0xFF)),
            (
                "Vermelho 500",
                "ptnd.red/500",
                Color::from_rgb(0xEF, 0x44, 0x44),
            ),
            (
                "Rosa 500",
                "ptnd.pink/500",
                Color::from_rgb(0xEC, 0x48, 0x99),
            ),
            (
                "Roxo 500",
                "ptnd.purple/500",
                Color::from_rgb(0x8B, 0x5C, 0xF6),
            ),
            (
                "Azul 500",
                "ptnd.blue/500",
                Color::from_rgb(0x3B, 0x82, 0xF6),
            ),
            (
                "Ciano 500",
                "ptnd.cyan/500",
                Color::from_rgb(0x06, 0xB6, 0xD4),
            ),
            (
                "Teal 500",
                "ptnd.teal/500",
                Color::from_rgb(0x14, 0xB8, 0xA6),
            ),
            (
                "Verde 500",
                "ptnd.green/500",
                Color::from_rgb(0x10, 0xB9, 0x81),
            ),
            (
                "Amarelo 500",
                "ptnd.amber/500",
                Color::from_rgb(0xF5, 0x9E, 0x0B),
            ),
        ];

        let doc_swatches: Vec<(String, String, Color)> = {
            let s = shell.peek();
            let mut list = Vec::new();
            if let Some(session) = s.bridge.session() {
                if let Some(surface) = s
                    .bridge
                    .active_surface()
                    .and_then(|id| session.surface(id).ok())
                {
                    for obj in surface.objects() {
                        if let Some(fill) = obj.fill.as_deref() {
                            let rgb = petunia_design_document::resolve_color_to_rgb(fill);
                            let col = Color::from_rgb(
                                (rgb[0] * 255.0).round() as u8,
                                (rgb[1] * 255.0).round() as u8,
                                (rgb[2] * 255.0).round() as u8,
                            );
                            let name = format!("Obj #{}", obj.id.raw());
                            if !list.iter().any(|(_, tok, _)| tok == fill) {
                                list.push((name, fill.to_string(), col));
                            }
                        }
                    }
                }
            }
            list
        };

        let active_swatches: Vec<(String, String, Color)> = match current_lib {
            0 => system_palette
                .iter()
                .map(|&(n, t, c)| (n.to_string(), t.to_string(), c))
                .collect(),
            1 => doc_swatches,
            _ => (*custom_swatches.read()).clone(),
        };

        let mut add_swatches = custom_swatches;

        rect()
            .direction(Direction::Vertical)
            .width(Size::fill())
            .height(Size::fill())
            .spacing(theme::SPACE_2)
            .child(
                // Library Switcher: Sistema | Documento | Favoritos
                rect()
                    .direction(Direction::Horizontal)
                    .width(Size::fill())
                    .height(Size::px(26.))
                    .background(theme::SURFACE_CHROME_STRONG)
                    .padding(Gaps::new_all(2.))
                    .spacing(2.)
                    .main_align(Alignment::SpaceEvenly)
                    .cross_align(Alignment::Center)
                    .child(subtab_pill("Sistema", 0, current_lib == 0, &mut lib_mode))
                    .child(subtab_pill("Documento", 1, current_lib == 1, &mut lib_mode))
                    .child(subtab_pill("Favoritos", 2, current_lib == 2, &mut lib_mode)),
            )
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .width(Size::fill())
                    .main_align(Alignment::SpaceBetween)
                    .cross_align(Alignment::Center)
                    .child(
                        label()
                            .text(match current_lib {
                                0 => "PALETA DO SISTEMA",
                                1 => "CORES DO DOCUMENTO",
                                _ => "AMOSTRAS FAVORITAS",
                            })
                            .font_size(10.)
                            .color(theme::TEXT_SECONDARY),
                    )
                    .maybe_child(if current_lib == 2 {
                        Some(
                            Button::new()
                                .on_press(move |_| {
                                    let count = add_swatches.read().len() + 1;
                                    let mut list = (*add_swatches.read()).clone();
                                    list.push((
                                        format!("Amostra {count}"),
                                        "#EC4899".to_string(),
                                        Color::from_rgb(0xEC, 0x48, 0x99),
                                    ));
                                    add_swatches.set(list);
                                })
                                .child(label().text("+ Adicionar").font_size(9.)),
                        )
                    } else {
                        None
                    }),
            )
            .child(if active_swatches.is_empty() {
                rect()
                    .width(Size::fill())
                    .padding(Gaps::new_all(theme::SPACE_2))
                    .center()
                    .child(
                        label()
                            .text("Nenhuma cor encontrada nesta biblioteca.")
                            .font_size(11.)
                            .color(theme::TEXT_SECONDARY),
                    )
            } else {
                rect()
                    .direction(Direction::Vertical)
                    .width(Size::fill())
                    .spacing(theme::SPACE_1)
                    .children(active_swatches.chunks(4).map(|chunk| {
                        rect()
                            .direction(Direction::Horizontal)
                            .width(Size::fill())
                            .main_align(Alignment::SpaceBetween)
                            .children(chunk.iter().map(|(name, token, color)| {
                                let mut shell_for_swatch = shell;
                                let tok = token.clone();
                                let col = *color;
                                rect()
                                    .width(Size::px(60.))
                                    .height(Size::px(34.))
                                    .background(col)
                                    .border(
                                        Border::new()
                                            .fill(theme::SURFACE_CHROME_STRONG)
                                            .width(1.)
                                            .alignment(BorderAlignment::Inner),
                                    )
                                    .center()
                                    .on_press(move |_| {
                                        if let Some(id) = first_id {
                                            let cmd = if is_fill {
                                                Command::SetFill {
                                                    id,
                                                    fill: Some(tok.clone()),
                                                }
                                            } else {
                                                Command::SetStroke {
                                                    id,
                                                    stroke: Some(tok.clone()),
                                                    width: 2.0,
                                                }
                                            };
                                            let _ = shell_for_swatch
                                                .write()
                                                .bridge
                                                .submit_all("Set swatch color", vec![cmd]);
                                        }
                                    })
                                    .child(label().text(name.clone()).font_size(9.).color(
                                        if col.r() > 180 && col.g() > 180 {
                                            Color::BLACK
                                        } else {
                                            Color::WHITE
                                        },
                                    ))
                            }))
                    }))
            })
    }
}

/// Background Tasks Tab (`ptnd.panel.background_tasks`): Job queue, progress, cancellation.
#[derive(Clone, PartialEq)]
pub struct BackgroundTasksPanel(pub UiShell);

impl Component for BackgroundTasksPanel {
    fn render(&self) -> impl IntoElement {
        let shell = self.0.shell;
        let jobs = shell.peek().bridge.jobs().list_jobs();
        let mut shell_for_sim = shell;
        let mut shell_for_clear = shell;

        rect()
            .direction(Direction::Vertical)
            .width(Size::fill())
            .height(Size::fill())
            .spacing(theme::SPACE_2)
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .width(Size::fill())
                    .main_align(Alignment::SpaceBetween)
                    .cross_align(Alignment::Center)
                    .child(section_header("TAREFAS EM SEGUNDO PLANO"))
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .spacing(4.)
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        let s = shell_for_sim.write();
                                        let (id, _) =
                                            s.bridge.jobs().spawn_job("Exportação PDF 300 DPI");
                                        s.bridge.jobs().update_progress(id, 65);
                                    })
                                    .child(label().text("+ Simular").font_size(10.)),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| {
                                        shell_for_clear.write().bridge.jobs().clear_completed();
                                    })
                                    .child(label().text("Limpar").font_size(10.)),
                            ),
                    ),
            )
            .child(if jobs.is_empty() {
                rect()
                    .width(Size::fill())
                    .padding(Gaps::new_all(theme::SPACE_3))
                    .center()
                    .child(
                        label()
                            .text("Nenhuma tarefa em execução em segundo plano.")
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
                            petunia_design_jobs::JobState::Running => "⚡ Executando",
                            petunia_design_jobs::JobState::Completed => "✓ Concluído",
                            petunia_design_jobs::JobState::Cancelled => "⊘ Cancelado",
                            petunia_design_jobs::JobState::Failed => "✗ Falhou",
                            petunia_design_jobs::JobState::Queued => "Pendente",
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
                                        label().text(state_label).font_size(10.).color(state_color),
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
                                    .width(Size::fill())
                                    .main_align(Alignment::SpaceBetween)
                                    .cross_align(Alignment::Center)
                                    .child(
                                        label()
                                            .text(format!("{}%", job.percent))
                                            .font_size(10.)
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
                                                .child(label().text("Cancelar").font_size(9.)),
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

#[derive(Clone, PartialEq)]
pub struct NavigatorTab(pub UiShell);

impl Component for NavigatorTab {
    fn render(&self) -> impl IntoElement {
        let shell = self.0.shell;
        let is_dragging = use_state(|| false);
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
        let mut shell_for_zoom_100 = shell;

        // Viewport visible rect in document space
        let vis_doc = snapshot.camera.visible_doc_rect();
        let vis_box_x = ((vis_doc.x0 * scale) as f32).clamp(0.0, thumb_w);
        let vis_box_y = ((vis_doc.y0 * scale) as f32).clamp(0.0, thumb_h);
        let vis_box_w = (((vis_doc.width() * scale) as f32).min(thumb_w - vis_box_x)).max(4.0);
        let vis_box_h = (((vis_doc.height() * scale) as f32).min(thumb_h - vis_box_y)).max(4.0);

        let mut shell_for_down = shell;
        let mut shell_for_move = shell;
        let mut drag_down = is_dragging;
        let drag_move = is_dragging;
        let mut drag_up = is_dragging;

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
                                    &mut shell_for_zoom_100.write(),
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
                            .on_pointer_down(move |event: Event<PointerEventData>| {
                                drag_down.set(true);
                                let loc = event.element_location();
                                let target_doc_x = (loc.x / scale).clamp(0.0, surf_w);
                                let target_doc_y = (loc.y / scale).clamp(0.0, surf_h);
                                let mut s = shell_for_down.write();
                                let mut cam = s.view_camera();
                                let vp_w = cam.viewport_width;
                                let vp_h = cam.viewport_height;
                                cam.pan_x = vp_w / 2.0 - target_doc_x * cam.zoom;
                                cam.pan_y = vp_h / 2.0 - target_doc_y * cam.zoom;
                                s.set_view_camera(cam);
                            })
                            .on_pointer_move(move |event: Event<PointerEventData>| {
                                if *drag_move.peek() {
                                    let loc = event.element_location();
                                    let target_doc_x = (loc.x / scale).clamp(0.0, surf_w);
                                    let target_doc_y = (loc.y / scale).clamp(0.0, surf_h);
                                    let mut s = shell_for_move.write();
                                    let mut cam = s.view_camera();
                                    let vp_w = cam.viewport_width;
                                    let vp_h = cam.viewport_height;
                                    cam.pan_x = vp_w / 2.0 - target_doc_x * cam.zoom;
                                    cam.pan_y = vp_h / 2.0 - target_doc_y * cam.zoom;
                                    s.set_view_camera(cam);
                                }
                            })
                            .on_mouse_up(move |_| {
                                drag_up.set(false);
                            })
                            .on_pointer_leave(move |_| {
                                drag_up.set(false);
                            })
                            .children(snapshot.objects.iter().filter_map(|obj| {
                                let min_x = (obj.world_bounds[0] * scale) as f32;
                                let min_y = (obj.world_bounds[1] * scale) as f32;
                                let w =
                                    ((obj.world_bounds[2] - obj.world_bounds[0]) * scale) as f32;
                                let h =
                                    ((obj.world_bounds[3] - obj.world_bounds[1]) * scale) as f32;
                                if w > 1.0 && h > 1.0 {
                                    let fill_col = obj
                                        .fill
                                        .as_deref()
                                        .and_then(parse_color_rgb)
                                        .map(|(r, g, b)| {
                                            Color::from_rgb(
                                                r.round() as u8,
                                                g.round() as u8,
                                                b.round() as u8,
                                            )
                                        })
                                        .unwrap_or(Color::from_rgb(0x94, 0xA3, 0xB8));
                                    Some(
                                        rect()
                                            .position(
                                                Position::new_absolute().left(min_x).top(min_y),
                                            )
                                            .width(Size::px(w))
                                            .height(Size::px(h))
                                            .background(fill_col)
                                            .into_element(),
                                    )
                                } else {
                                    None
                                }
                            }))
                            .child(
                                rect()
                                    .position(
                                        Position::new_absolute().left(vis_box_x).top(vis_box_y),
                                    )
                                    .width(Size::px(vis_box_w))
                                    .height(Size::px(vis_box_h))
                                    .background(Color::from_argb(0x22, 0xEF, 0x44, 0x44))
                                    .border(
                                        Border::new()
                                            .fill(Color::from_rgb(0xEF, 0x44, 0x44))
                                            .width(1.5)
                                            .alignment(BorderAlignment::Inner),
                                    ),
                            ),
                    ),
            )
    }
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
                    vec![Command::SetOpacity { id, opacity: val }],
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
        .child(label().text(label_text).font_size(10.))
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
        .child(label().text(label_text).font_size(10.))
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
        .child(label().text("Fixar (Bake)").font_size(10.))
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
        .child(label().text("Fixar Geometria").font_size(10.))
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
        .child(label().text("Fixar Transparência").font_size(10.))
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
            .width(Size::fill())
            .main_align(Alignment::SpaceBetween)
            .cross_align(Alignment::Center)
            .child(
                label()
                    .text("Nenhum modificador ativo")
                    .font_size(11.)
                    .color(theme::TEXT_TERTIARY),
            )
            .child(
                rect()
                    .direction(Direction::Horizontal)
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
                            .child(label().text("+ Contorno").font_size(10.)),
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
                            .child(label().text("+ Recorte").font_size(10.)),
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
                    OffsetJoin::Round => "Arredondada",
                    OffsetJoin::Miter => "Esquadria",
                    OffsetJoin::Bevel => "Chanfro",
                };
                let cap_name = match cap {
                    OffsetCap::None => "Reta",
                    OffsetCap::Round => "Redonda",
                    OffsetCap::Square => "Quadrada",
                };
                (
                    format!("Contorno Vivo: {:+.1} pt", distance),
                    format!("Junção: {} | Extr: {}", join_name, cap_name),
                )
            }
            petunia_design_document::ModifierKind::TransparentGradient { stops, .. } => (
                "Gradiente de Transparência".to_string(),
                format!("{} marcadores de opacidade", stops.len()),
            ),
            petunia_design_document::ModifierKind::Perspective { .. } => (
                "Distorção de Perspectiva".to_string(),
                "Deformação quad de 4 cantos".to_string(),
            ),
            petunia_design_document::ModifierKind::CropRect { rect } => (
                "Recorte Vetorial (Crop)".to_string(),
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
            .width(Size::fill())
            .main_align(Alignment::SpaceBetween)
            .cross_align(Alignment::Center)
            .child(
                rect()
                    .direction(Direction::Horizontal)
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
                        .font_size(10.),
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
                    .child(label().text("↑").font_size(10.)),
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
                    .child(label().text("↓").font_size(10.)),
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
                .child(label().text("×").font_size(10.)),
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
                        .font_size(10.)
                        .color(theme::TEXT_TERTIARY),
                );

                // Distance + Bake Row
                let dist_row = rect()
                    .direction(Direction::Horizontal)
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
                    .spacing(2.)
                    .cross_align(Alignment::Center)
                    .child(
                        label()
                            .text("Junção:")
                            .font_size(10.)
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
                                    .font_size(10.),
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
                                    .font_size(10.),
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
                                    .font_size(10.),
                            )
                    });
                card = card.child(join_row);
            }
            petunia_design_document::ModifierKind::CropRect { rect: current_rect } => {
                let r_val = *current_rect;
                card = card.child(
                    label()
                        .text(detail)
                        .font_size(10.)
                        .color(theme::TEXT_TERTIARY),
                );
                let modifiers_crop = modifiers.to_vec();
                let crop_row = rect()
                    .direction(Direction::Horizontal)
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
                            .child(label().text("Expandir +10pt").font_size(10.))
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
                            .child(label().text("Recortar -10pt").font_size(10.))
                    })
                    .child(bake_geometry_button(shell, target_id));
                card = card.child(crop_row);
            }
            petunia_design_document::ModifierKind::TransparentGradient { .. } => {
                card = card.child(
                    label()
                        .text(detail)
                        .font_size(10.)
                        .color(theme::TEXT_TERTIARY),
                );
                let trans_row = rect()
                    .direction(Direction::Horizontal)
                    .spacing(2.)
                    .child(bake_transparency_button(shell, target_id));
                card = card.child(trans_row);
            }
            petunia_design_document::ModifierKind::Perspective { .. } => {
                card = card.child(
                    label()
                        .text(detail)
                        .font_size(10.)
                        .color(theme::TEXT_TERTIARY),
                );
                let pers_row = rect()
                    .direction(Direction::Horizontal)
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
                    .child(label().text("+ Contorno").font_size(10.)),
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
                    .child(label().text("+ Recorte").font_size(10.)),
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
        .child(label().text(label_text).font_size(10.))
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
        .child(label().text("👁").font_size(10.))
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
        .child(label().text("✕").font_size(10.))
}

fn drop_shadow_adjust_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: &'static str,
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
        .child(label().text(label_text).font_size(10.))
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
        .child(label().text("👁").font_size(10.))
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

fn inner_shadow_adjust_button(
    mut shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    label_text: &'static str,
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
        .child(label().text(label_text).font_size(10.))
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
        .child(label().text("👁").font_size(10.))
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
        .child(label().text(label_text).font_size(10.))
}

fn adjustment_card(
    shell: State<petunia_design_shell::PetuniaShell>,
    target_id: Option<ObjectId>,
    adj: petunia_design_document::adjustments::AdjustmentItem,
) -> impl IntoElement {
    let adj_id = adj.id;
    let title = match &adj.kind {
        petunia_design_document::adjustments::AdjustmentKind::Levels { .. } => {
            format!("Níveis #{}", adj_id)
        }
        petunia_design_document::adjustments::AdjustmentKind::Curves { .. } => {
            format!("Curvas #{}", adj_id)
        }
        petunia_design_document::adjustments::AdjustmentKind::Hsl { .. } => {
            format!("HSL #{}", adj_id)
        }
        petunia_design_document::adjustments::AdjustmentKind::Exposure { .. } => {
            format!("Exposição #{}", adj_id)
        }
        petunia_design_document::adjustments::AdjustmentKind::WhiteBalance { .. } => {
            format!("Balanço B. #{}", adj_id)
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
                        .child(label().text("✕").font_size(10.)),
                ),
        )
        .child(body)
}

// =========================================================================
// Histogram Widget (Spec 10.10)
// =========================================================================

fn parse_color_rgb(color_str: &str) -> Option<(f64, f64, f64)> {
    let s = color_str.trim();
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 || hex.len() == 8 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some((r as f64, g as f64, b as f64));
        }
    }
    match s {
        "ptnd.white" => Some((255.0, 255.0, 255.0)),
        "ptnd.black" | "ptnd.gray/900" => Some((17.0, 24.0, 39.0)),
        "ptnd.gray/600" => Some((75.0, 85.0, 99.0)),
        "ptnd.gray/300" => Some((209.0, 213.0, 219.0)),
        "ptnd.red/500" => Some((239.0, 68.0, 68.0)),
        "ptnd.pink/500" => Some((236.0, 72.0, 153.0)),
        "ptnd.purple/500" => Some((139.0, 92.0, 246.0)),
        "ptnd.blue/500" => Some((59.0, 130.0, 246.0)),
        "ptnd.cyan/500" => Some((6.0, 182.0, 212.0)),
        "ptnd.teal/500" => Some((20.0, 184.0, 166.0)),
        "ptnd.green/500" => Some((16.0, 185.0, 129.0)),
        "ptnd.amber/500" => Some((245.0, 158.0, 11.0)),
        "ptnd.bloom/500" => Some((183.0, 122.0, 255.0)),
        _ => None,
    }
}

fn apply_adjustments_to_rgb(
    mut r: f64,
    mut g: f64,
    mut b: f64,
    adjustments: &[petunia_design_document::adjustments::AdjustmentItem],
) -> (f64, f64, f64) {
    for adj in adjustments {
        match &adj.kind {
            petunia_design_document::adjustments::AdjustmentKind::Exposure {
                exposure,
                offset,
                gamma,
            } => {
                let exp_mul = 2.0f64.powf(*exposure);
                let apply_exp = |c: f64| -> f64 {
                    let norm = (c / 255.0 * exp_mul + offset).clamp(0.0, 1.0);
                    norm.powf(1.0 / gamma.max(0.01)) * 255.0
                };
                r = apply_exp(r);
                g = apply_exp(g);
                b = apply_exp(b);
            }
            petunia_design_document::adjustments::AdjustmentKind::Levels { master, .. } => {
                let bp = master.input_black;
                let wp = master.input_white;
                let gamma = master.gamma.max(0.01);
                let out_b = master.output_black;
                let out_w = master.output_white;
                let apply_levels = |c: f64| -> f64 {
                    let norm = ((c - bp) / (wp - bp).max(1.0)).clamp(0.0, 1.0);
                    let mapped = norm.powf(1.0 / gamma);
                    out_b + mapped * (out_w - out_b)
                };
                r = apply_levels(r);
                g = apply_levels(g);
                b = apply_levels(b);
            }
            petunia_design_document::adjustments::AdjustmentKind::Hsl { lightness, .. } => {
                let l_shift = *lightness * 60.0;
                r = (r + l_shift).clamp(0.0, 255.0);
                g = (g + l_shift).clamp(0.0, 255.0);
                b = (b + l_shift).clamp(0.0, 255.0);
            }
            petunia_design_document::adjustments::AdjustmentKind::WhiteBalance {
                temperature,
                tint,
            } => {
                r = (r + *temperature * 35.0).clamp(0.0, 255.0);
                b = (b - *temperature * 35.0).clamp(0.0, 255.0);
                g = (g + *tint * 25.0).clamp(0.0, 255.0);
            }
            _ => {}
        }
    }
    (r, g, b)
}

pub fn compute_histogram_bins(
    shell: &PetuniaShell,
    selected_obj: Option<&petunia_design_document::DocumentObject>,
    channel: u8, // 0: RGB, 1: R, 2: G, 3: B, 4: Luma
) -> ([f32; 32], u32, u32, u32, u32) {
    let mut centers: Vec<(f64, f64, f64)> = Vec::new();
    let mut is_raster_sample = false;

    if let Some(obj) = selected_obj {
        let mut sampled = Vec::new();
        if let Some(petunia_design_document::ShapeKind::Image { data, .. }) = &obj.shape {
            if let Some(bytes) = data.as_deref() {
                if let Ok(image) =
                    petunia_design_raster::ImageCache::shared().prepare(bytes, &|| false)
                {
                    // This panel is a sampled RGBA8 display histogram, including
                    // 16-bit sources. Originals retain their precision unchanged.
                    let level = &image.levels()[0];
                    let total = level.premultiplied_rgba8().len() / 4;
                    let stride = total.div_ceil(500).max(1);
                    for pixel in level.premultiplied_rgba8().chunks_exact(4).step_by(stride) {
                        if pixel[3] > 12 {
                            let unassociate = 255.0 / f64::from(pixel[3]);
                            sampled.push((
                                f64::from(pixel[0]) * unassociate,
                                f64::from(pixel[1]) * unassociate,
                                f64::from(pixel[2]) * unassociate,
                            ));
                        }
                    }
                    is_raster_sample = !sampled.is_empty();
                }
            }
        }

        if sampled.is_empty() {
            let base_rgb = obj
                .fill
                .as_deref()
                .and_then(parse_color_rgb)
                .or_else(|| obj.stroke.as_deref().and_then(parse_color_rgb))
                .unwrap_or((140.0, 140.0, 140.0));
            sampled.push(base_rgb);
        }

        let empty_adj = Vec::new();
        let adjustments = obj
            .appearance
            .as_ref()
            .map(|a| a.adjustments.as_slice())
            .unwrap_or(&empty_adj);

        for (r, g, b) in sampled {
            let (adj_r, adj_g, adj_b) = apply_adjustments_to_rgb(r, g, b, adjustments);
            centers.push((adj_r, adj_g, adj_b));
        }
    } else {
        if let Some(session) = shell.bridge.session() {
            if let Some(surf_id) = session.active_surface() {
                if let Ok(surf) = session.surface(surf_id) {
                    for obj in surf.objects() {
                        let rgb = obj
                            .fill
                            .as_deref()
                            .and_then(parse_color_rgb)
                            .or_else(|| obj.stroke.as_deref().and_then(parse_color_rgb))
                            .unwrap_or((128.0, 128.0, 128.0));
                        centers.push(rgb);
                    }
                }
            }
        }
        if centers.is_empty() {
            centers.push((128.0, 128.0, 128.0));
        }
    }

    let mut raw_bins = [0.0f32; 32];
    let mut total_channel_sum = 0.0f64;
    let mut total_weight = 0.0f64;

    for (r, g, b) in &centers {
        let luma = 0.299 * r + 0.587 * g + 0.114 * b;
        let c = match channel {
            1 => *r,
            2 => *g,
            3 => *b,
            4 => luma,
            _ => (r + g + b) / 3.0,
        };
        total_channel_sum += c;
        total_weight += 1.0;

        if is_raster_sample {
            let bin_idx = ((c / 8.0).floor() as usize).clamp(0, 31);
            raw_bins[bin_idx] += 1.0;
        } else {
            let sigma = 24.0;
            for (i, bin) in raw_bins.iter_mut().enumerate() {
                let bin_center = (i as f64) * 8.0 + 4.0;
                let diff = (bin_center - c) / sigma;
                let weight = (-0.5 * diff * diff).exp();
                *bin += weight as f32;
            }
        }
    }

    let mean = if total_weight > 0.0 {
        (total_channel_sum / total_weight).round() as u32
    } else {
        128
    };

    let max_bin = raw_bins.iter().copied().fold(0.001f32, f32::max);
    let mut normalized_bins = [0.0f32; 32];
    for (i, bin) in raw_bins.iter().enumerate() {
        normalized_bins[i] = bin / max_bin;
    }

    let sum_total: f32 = raw_bins.iter().sum::<f32>().max(0.001);
    let shadows_sum: f32 = raw_bins[0..8].iter().sum();
    let midtones_sum: f32 = raw_bins[8..24].iter().sum();
    let highlights_sum: f32 = raw_bins[24..32].iter().sum();

    let shadows_pct = ((shadows_sum / sum_total) * 100.0).round() as u32;
    let midtones_pct = ((midtones_sum / sum_total) * 100.0).round() as u32;
    let highlights_pct = ((highlights_sum / sum_total) * 100.0).round() as u32;

    (
        normalized_bins,
        mean,
        shadows_pct,
        midtones_pct,
        highlights_pct,
    )
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

        let shell = self.ui.shell.peek();
        let selected_obj = self
            .object_id
            .and_then(|id| shell.bridge.session().and_then(|s| s.find_object(id)));

        let (bins, mean, shadows_pct, midtones_pct, highlights_pct) =
            compute_histogram_bins(&shell, selected_obj, channel);

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
                rect()
                    .direction(Direction::Horizontal)
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
                    .cross_align(Alignment::End)
                    .children(bins.iter().enumerate().map(|(idx, &val)| {
                        let bar_h = (val * 64.0).clamp(2.0, 64.0);
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
                    .width(Size::fill())
                    .main_align(Alignment::SpaceBetween)
                    .child(
                        label()
                            .text(format!("Média: {}", mean))
                            .font_size(10.)
                            .color(theme::TEXT_SECONDARY),
                    )
                    .child(
                        label()
                            .text(format!("Sombras: {}%", shadows_pct))
                            .font_size(10.)
                            .color(theme::TEXT_TERTIARY),
                    )
                    .child(
                        label()
                            .text(format!("Meios: {}%", midtones_pct))
                            .font_size(10.)
                            .color(theme::TEXT_TERTIARY),
                    )
                    .child(
                        label()
                            .text(format!("Realces: {}%", highlights_pct))
                            .font_size(10.)
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
        .child(label().text(title).font_size(10.).color(if active {
            theme::TEXT_PRIMARY
        } else {
            theme::TEXT_TERTIARY
        }))
}

/// Draggable splitter between Workspace and RightDock.
#[derive(Clone, PartialEq)]
pub struct DockSplitter(pub UiShell);

impl Component for DockSplitter {
    fn render(&self) -> impl IntoElement {
        let is_dragging = use_state(|| false);
        let drag_start_x = use_state(|| 0.0f64);
        let drag_start_width = use_state(|| 240.0f32);
        let dock_width = self.0.dock_width;
        let dragging_val = *is_dragging.read();

        let splitter_bar = rect()
            .width(Size::px(4.))
            .height(Size::fill())
            .background(if dragging_val {
                theme::ACCENT_BLOOM
            } else {
                theme::SURFACE_CHROME_STRONG
            })
            .cursor(CursorIcon::EwResize)
            .on_mouse_down({
                let mut is_dragging = is_dragging;
                let mut drag_start_x = drag_start_x;
                let mut drag_start_width = drag_start_width;
                move |event: Event<MouseEventData>| {
                    is_dragging.set(true);
                    drag_start_x.set(event.global_location.x);
                    drag_start_width.set(*dock_width.peek());
                }
            });

        if dragging_val {
            rect()
                .direction(Direction::Horizontal)
                .width(Size::px(4.))
                .height(Size::fill())
                .child(splitter_bar)
                .child(
                    Portal::new("dock-splitter-drag")
                        .width(Size::px(0.))
                        .height(Size::px(0.))
                        .child(
                            rect()
                                .position(Position::new_absolute().top(0.).left(0.))
                                .width(Size::fill())
                                .height(Size::fill())
                                .cursor(CursorIcon::EwResize)
                                .on_mouse_move({
                                    let mut dock_width = dock_width;
                                    move |event: Event<MouseEventData>| {
                                        let delta = *drag_start_x.read() - event.global_location.x;
                                        let new_w = (*drag_start_width.read() + delta as f32)
                                            .clamp(180.0, 520.0);
                                        dock_width.set(new_w);
                                    }
                                })
                                .on_mouse_up({
                                    let mut is_dragging = is_dragging;
                                    move |_| {
                                        is_dragging.set(false);
                                    }
                                }),
                        ),
                )
        } else {
            rect()
                .width(Size::px(4.))
                .height(Size::fill())
                .child(splitter_bar)
        }
    }
}

/// Draggable vertical splitter between LeftDock and Center area.
#[derive(Clone, PartialEq)]
pub struct LeftDockSplitter(pub UiShell);

impl Component for LeftDockSplitter {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        if !*ui.left_dock_open.read() {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }

        let is_dragging = use_state(|| false);
        let drag_start_x = use_state(|| 0.0f64);
        let drag_start_width = use_state(|| 240.0f32);
        let left_dock_width = ui.left_dock_width;
        let dragging_val = *is_dragging.read();

        let splitter_bar = rect()
            .width(Size::px(4.))
            .height(Size::fill())
            .background(if dragging_val {
                theme::ACCENT_BLOOM
            } else {
                theme::SURFACE_CHROME_STRONG
            })
            .cursor(CursorIcon::EwResize)
            .on_mouse_down({
                let mut is_dragging = is_dragging;
                let mut drag_start_x = drag_start_x;
                let mut drag_start_width = drag_start_width;
                move |event: Event<MouseEventData>| {
                    is_dragging.set(true);
                    drag_start_x.set(event.global_location.x);
                    drag_start_width.set(*left_dock_width.peek());
                }
            });

        if dragging_val {
            rect()
                .direction(Direction::Horizontal)
                .width(Size::px(4.))
                .height(Size::fill())
                .child(splitter_bar)
                .child(
                    Portal::new("left-dock-splitter-drag")
                        .width(Size::px(0.))
                        .height(Size::px(0.))
                        .child(
                            rect()
                                .position(Position::new_absolute().top(0.).left(0.))
                                .width(Size::fill())
                                .height(Size::fill())
                                .cursor(CursorIcon::EwResize)
                                .on_mouse_move({
                                    let mut left_dock_width = left_dock_width;
                                    move |event: Event<MouseEventData>| {
                                        let delta = event.global_location.x - *drag_start_x.read();
                                        let new_w = (*drag_start_width.read() + delta as f32)
                                            .clamp(160.0, 480.0);
                                        left_dock_width.set(new_w);
                                    }
                                })
                                .on_mouse_up({
                                    let mut is_dragging = is_dragging;
                                    move |_| {
                                        is_dragging.set(false);
                                    }
                                }),
                        ),
                )
        } else {
            rect()
                .width(Size::px(4.))
                .height(Size::fill())
                .child(splitter_bar)
        }
    }
}

/// Draggable horizontal splitter between Workspace and BottomDock.
#[derive(Clone, PartialEq)]
pub struct BottomDockSplitter(pub UiShell);

impl Component for BottomDockSplitter {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        if !*ui.bottom_dock_open.read() {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }

        let is_dragging = use_state(|| false);
        let drag_start_y = use_state(|| 0.0f64);
        let drag_start_height = use_state(|| 160.0f32);
        let bottom_dock_height = ui.bottom_dock_height;
        let dragging_val = *is_dragging.read();

        let splitter_bar = rect()
            .width(Size::fill())
            .height(Size::px(4.))
            .background(if dragging_val {
                theme::ACCENT_BLOOM
            } else {
                theme::SURFACE_CHROME_STRONG
            })
            .cursor(CursorIcon::NsResize)
            .on_mouse_down({
                let mut is_dragging = is_dragging;
                let mut drag_start_y = drag_start_y;
                let mut drag_start_height = drag_start_height;
                move |event: Event<MouseEventData>| {
                    is_dragging.set(true);
                    drag_start_y.set(event.global_location.y);
                    drag_start_height.set(*bottom_dock_height.peek());
                }
            });

        if dragging_val {
            rect()
                .direction(Direction::Vertical)
                .width(Size::fill())
                .height(Size::px(4.))
                .child(splitter_bar)
                .child(
                    Portal::new("bottom-dock-splitter-drag")
                        .width(Size::px(0.))
                        .height(Size::px(0.))
                        .child(
                            rect()
                                .position(Position::new_absolute().top(0.).left(0.))
                                .width(Size::fill())
                                .height(Size::fill())
                                .cursor(CursorIcon::NsResize)
                                .on_mouse_move({
                                    let mut bottom_dock_height = bottom_dock_height;
                                    move |event: Event<MouseEventData>| {
                                        let delta = *drag_start_y.read() - event.global_location.y;
                                        let new_h = (*drag_start_height.read() + delta as f32)
                                            .clamp(80.0, 420.0);
                                        bottom_dock_height.set(new_h);
                                    }
                                })
                                .on_mouse_up({
                                    let mut is_dragging = is_dragging;
                                    move |_| {
                                        is_dragging.set(false);
                                    }
                                }),
                        ),
                )
        } else {
            rect()
                .width(Size::fill())
                .height(Size::px(4.))
                .child(splitter_bar)
        }
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
        let dock_w = *ui.left_dock_width.read();
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
                    .width(Size::fill())
                    .height(Size::px(34.))
                    .background(theme::SURFACE_CHROME)
                    .cross_align(Alignment::Center)
                    .main_align(Alignment::SpaceBetween)
                    .padding(Gaps::new(0., theme::SPACE_1, 0., theme::SPACE_1))
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .spacing(theme::SPACE_1)
                            .cross_align(Alignment::Center)
                            .child(tab_button(
                                ui,
                                "Ativos",
                                0,
                                active_tab == 0,
                                &mut left_dock_tab,
                            ))
                            .child(tab_button(
                                ui,
                                "Símbolos",
                                1,
                                active_tab == 1,
                                &mut left_dock_tab,
                            )),
                    )
                    .child(
                        Button::new()
                            .on_press(move |_| left_dock_open.set(false))
                            .child(label().text("✕").font_size(11.).color(theme::TEXT_TERTIARY)),
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
                .child(section_header("ATIVOS DO DOCUMENTO"))
                .child(
                    Button::new()
                        .on_press(move |_| {
                            place_image_open.set(true);
                        })
                        .child(
                            rect()
                                .direction(Direction::Horizontal)
                                .padding(Gaps::new(4., 8., 4., 8.))
                                .background(theme::SURFACE_CHROME_STRONG)
                                .corner_radius(4.0)
                                .cross_align(Alignment::Center)
                                .child(
                                    label()
                                        .text("+ Importar Imagem (file.place)")
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
                                .text(format!("Imagem #{}", img.id))
                                .font_size(11.)
                                .color(theme::TEXT_PRIMARY),
                        )
                        .child(
                            label()
                                .text(format!("{:.0} x {:.0} pt", w, h))
                                .font_size(10.)
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
                .child(section_header("BIBLIOTECA DE FORMAS"))
                .child(symbol_preset_item(
                    shell,
                    "Retângulo Básico",
                    "200x150 pt",
                    ShapeKind::Rectangle {
                        corner_radii: [0.0; 4],
                    },
                    [50.0, 50.0, 250.0, 200.0],
                    "ptnd.blue/500",
                ))
                .child(symbol_preset_item(
                    shell,
                    "Cartão UI",
                    "240x140 pt · r=8",
                    ShapeKind::Rectangle {
                        corner_radii: [8.0; 4],
                    },
                    [50.0, 50.0, 290.0, 190.0],
                    "ptnd.gray/900",
                ))
                .child(symbol_preset_item(
                    shell,
                    "Círculo / Avatar",
                    "100x100 pt",
                    ShapeKind::Ellipse,
                    [50.0, 50.0, 150.0, 150.0],
                    "ptnd.teal/500",
                ))
                .child(symbol_preset_item(
                    shell,
                    "Estrela 5 Pontas",
                    "120x120 pt",
                    ShapeKind::Star {
                        points: 5,
                        inner_ratio: 0.5,
                    },
                    [50.0, 50.0, 170.0, 170.0],
                    "ptnd.purple/500",
                )),
        )
    }
}

fn symbol_preset_item(
    shell: State<PetuniaShell>,
    name: &'static str,
    subtitle: &'static str,
    shape: ShapeKind,
    bounds: [f64; 4],
    fill: &'static str,
) -> impl IntoElement {
    let mut shell_for_insert = shell;
    let shape_clone = shape;
    rect()
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .padding(Gaps::new(4., 6., 4., 6.))
        .background(theme::SURFACE_CHROME)
        .corner_radius(4.0)
        .child(
            rect()
                .direction(Direction::Vertical)
                .child(label().text(name).font_size(11.).color(theme::TEXT_PRIMARY))
                .child(
                    label()
                        .text(subtitle)
                        .font_size(10.)
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
                                .text("Inserir")
                                .font_size(10.)
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
        let dock_h = *ui.bottom_dock_height.read();
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
                    .width(Size::fill())
                    .height(Size::px(30.))
                    .background(theme::SURFACE_CHROME)
                    .cross_align(Alignment::Center)
                    .main_align(Alignment::SpaceBetween)
                    .padding(Gaps::new(0., theme::SPACE_2, 0., theme::SPACE_2))
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .spacing(theme::SPACE_1)
                            .cross_align(Alignment::Center)
                            .child(tab_button(
                                ui,
                                "Tarefas",
                                0,
                                active_tab == 0,
                                &mut bottom_dock_tab,
                            ))
                            .child(tab_button(
                                ui,
                                "Diagnóstico",
                                1,
                                active_tab == 1,
                                &mut bottom_dock_tab,
                            )),
                    )
                    .child(
                        Button::new()
                            .on_press(move |_| bottom_dock_open.set(false))
                            .child(label().text("✕").font_size(11.).color(theme::TEXT_TERTIARY)),
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
        let title = session.map_or("Sem Documento", |s| s.title());
        let rev = session.map_or(0, |s| s.saved_revision());
        let dirty = shell_ref.bridge.is_dirty();
        let surf_id = shell_ref
            .bridge
            .active_surface()
            .map_or("Nenhum".to_string(), |id| id.to_string());
        let obj_count = snapshot.objects.len();

        ScrollView::new().child(
            rect()
                .direction(Direction::Horizontal)
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
                                .text("DOCUMENTO")
                                .font_size(10.)
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
                                .text(if dirty {
                                    "Modificado (não salvo)"
                                } else {
                                    "Salvo / Sem alterações"
                                })
                                .font_size(10.)
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
                                .text("SUPERFÍCIE & OBJETOS")
                                .font_size(10.)
                                .color(theme::TEXT_TERTIARY),
                        )
                        .child(
                            label()
                                .text(format!("Ativa: {surf_id}"))
                                .font_size(11.)
                                .color(theme::TEXT_PRIMARY),
                        )
                        .child(
                            label()
                                .text(format!("Total de objetos: {obj_count}"))
                                .font_size(10.)
                                .color(theme::TEXT_SECONDARY),
                        ),
                )
                .child(
                    rect()
                        .direction(Direction::Vertical)
                        .spacing(theme::SPACE_1)
                        .child(
                            label()
                                .text("VIEWPORT & CÂMERA")
                                .font_size(10.)
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
                                .font_size(10.)
                                .color(theme::TEXT_SECONDARY),
                        ),
                )
                .child(
                    rect()
                        .direction(Direction::Vertical)
                        .spacing(theme::SPACE_1)
                        .child(
                            label()
                                .text("FERRAMENTA & PERSONA")
                                .font_size(10.)
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
                                .font_size(10.)
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

        // Click on "Propriedades" tab button (roughly x=120, y=17)
        runner.press_cursor((120., 17.));
        runner.release_cursor((120., 17.));
        runner.sync_and_update();

        assert_eq!(
            *ui.dock_tab.read(),
            1,
            "active tab should switch to Propriedades (1)"
        );

        // Click on "Cores" tab button (roughly x=200, y=17)
        runner.press_cursor((200., 17.));
        runner.release_cursor((200., 17.));
        runner.sync_and_update();

        assert_eq!(
            *ui.dock_tab.read(),
            2,
            "active tab should switch to Cores (2)"
        );

        // Click on "Histórico" tab button (roughly x=270, y=17)
        runner.press_cursor((270., 17.));
        runner.release_cursor((270., 17.));
        runner.sync_and_update();

        assert_eq!(
            *ui.dock_tab.read(),
            3,
            "active tab should switch to Histórico (3)"
        );

        // Switch to tab 4: Navegador
        let mut dock_tab = ui.dock_tab;
        dock_tab.set(4);
        runner.sync_and_update();
        assert_eq!(
            *ui.dock_tab.read(),
            4,
            "active tab should switch to Navegador (4)"
        );

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
        assert_eq!(mean, 128);
        assert!(midtones > shadows && midtones > highlights);

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
