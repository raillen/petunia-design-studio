//! Overlays: command palette and toolbar customization.
//!
//! Both are projections of registry data: the palette lists exactly the
//! enabled menu items, and the customization dialog lists exactly the layout
//! slots the shell owns.

use freya::prelude::*;

use crate::actions::run_action_token;
use crate::chrome::app_icon;
use crate::theme;
use crate::ui_state::UiShell;

/// Command palette: enabled menu items filtered by the live query.
#[derive(Clone, PartialEq)]
pub struct CommandPalette(pub UiShell);

impl Component for CommandPalette {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        if !(*ui.palette_open.read()) {
            // Zero-size: a default rect would cover the window and swallow
            // every click meant for the chrome below.
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let query = ui.palette_query.read().clone().to_lowercase();
        let items = ui.shell.peek().bridge.query_command_index();
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
                    .child(Input::new(palette_query).placeholder("Type a command…"))
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
    MenuItem::new()
        .on_press(move |_| {
            run_action_token(&mut shell.write(), &token);
            palette_open.set(false);
            palette_query.set(String::new());
        })
        .child(
            label()
                .text(item.label.clone())
                .color(theme::TEXT_PRIMARY)
                .font_size(theme::BODY_SIZE),
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
                                .width(Size::px(360.))
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
                                ),
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
    let label_text = if row.label.is_empty() {
        "—".to_string()
    } else {
        row.label.clone()
    };
    rect()
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .spacing(theme::SPACE_1)
        .cross_align(Alignment::Center)
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
        .child(row_mover(ui.clone(), index, -1, "Up"))
        .child(row_mover(ui.clone(), index, 1, "Down"))
}

fn row_mover(ui: UiShell, index: usize, delta: i32, text: &str) -> impl IntoElement {
    let mut shell = ui.shell;
    Button::new()
        .on_press(move |_| {
            shell.write().bridge.toolbar_move(index, delta);
        })
        .child(text)
}

fn divider_adder(ui: UiShell, divider_label: String) -> impl IntoElement {
    let mut shell = ui.shell;
    Button::new()
        .on_press(move |_| {
            shell.write().bridge.toolbar_insert_divider(None);
        })
        .child(divider_label)
}
