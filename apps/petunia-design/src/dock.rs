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
                                .child(label().text("Agrupar").font_size(11.)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    let _ = run_action_token(
                                        &mut shell_for_ungroup.write(),
                                        "ptnd.action.object.ungroup",
                                    );
                                })
                                .child(label().text("Desagrupar").font_size(11.)),
                        )
                        .child(
                            Button::new()
                                .on_press(move |_| {
                                    let _ = run_action_token(
                                        &mut shell_for_delete.write(),
                                        "ptnd.action.edit.delete",
                                    );
                                })
                                .child(label().text("Excluir").font_size(11.)),
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

    let bounds = props.bounds.unwrap_or([0.0, 0.0, 100.0, 100.0]);
    let [x, y, w, h] = bounds;
    let stroke_width = props.stroke_width;
    let opacity = (props.opacity * 100.0).round() as u32;

    rect()
        .direction(Direction::Vertical)
        .width(Size::fill())
        .height(Size::fill())
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
        )
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
}

