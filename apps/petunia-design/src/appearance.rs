//! Appearance bar: icon style (outline/filled) and accent color pickers.
//!
//! The chrome reads the chosen [`IconStyle`] and [`AccentColor`] from this
//! module's shared state; labels are resolved through the app, so no picker
//! owns a user-visible string beyond its swatch.

use freya::prelude::*;

use crate::chrome::{app_icon, with_tooltip};
use crate::theme::{self, IconStyle, ACCENTS};
use crate::ui_state::UiShell;

/// Trailing controls of the status bar: style toggle plus accent swatches.
#[derive(Clone, PartialEq)]
pub struct AppearanceBar(pub UiShell);

impl Component for AppearanceBar {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let style = *ui.icon_style.read();
        let accent = *ui.accent.read();
        rect()
            .direction(Direction::Horizontal)
            .spacing(theme::SPACE_1)
            .cross_align(Alignment::Center)
            .child(style_toggle(ui.clone(), style))
            .children(
                ACCENTS.iter().map(|candidate| {
                    accent_swatch(ui.clone(), *candidate, candidate.id == accent.id)
                }),
            )
    }
}

fn style_toggle(ui: UiShell, style: IconStyle) -> impl IntoElement {
    let mut icon_style = ui.icon_style;
    let next = style.toggled();
    with_tooltip(
        rect()
            .width(Size::px(26.))
            .height(Size::px(26.))
            .center()
            .on_press(move |_| {
                icon_style.set(next);
            })
            .child(app_icon(theme::ICON_STAR, style, theme::TEXT_SECONDARY)),
        &ui,
        "appearance:style".to_string(),
        appearance_text(&ui, "ptnd.text.appearance.icon_style"),
        appearance_text(&ui, "ptnd.text.summary.toggle_icon_style"),
        String::new(),
    )
}

fn appearance_text(ui: &UiShell, text_id: &str) -> String {
    let shell = ui.shell.peek();
    shell
        .bridge
        .localization()
        .text(text_id, shell.bridge.locale())
}

fn accent_swatch(ui: UiShell, candidate: theme::AccentColor, active: bool) -> impl IntoElement {
    let mut accent = ui.accent;
    with_tooltip(
        rect()
            .width(Size::px(16.))
            .height(Size::px(16.))
            .center()
            .background(candidate.value)
            .on_press(move |_| {
                accent.set(candidate);
            })
            .maybe(active, |el| {
                el.border(
                    Border::new()
                        .fill(theme::TEXT_PRIMARY)
                        .width(1.)
                        .alignment(BorderAlignment::Inner),
                )
            }),
        &ui,
        "appearance:accent".to_string(),
        appearance_text(&ui, "ptnd.text.appearance.accent_color"),
        appearance_text(&ui, "ptnd.text.summary.choose_accent"),
        String::new(),
    )
}
