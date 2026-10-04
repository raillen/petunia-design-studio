//! Appearance bar: icon style (outline/filled) and accent color pickers.
//!
//! The chrome reads the chosen [`IconStyle`] and [`AccentColor`] from this
//! module's shared state; labels are resolved through the app, so no picker
//! owns a user-visible string beyond its swatch.

use freya::prelude::*;

use crate::studio_widgets::StudioButton;
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
            .content(Content::Flex)
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
    let mut state = ui.icon_style;
    StudioButton::new(&ui, appearance_text(&ui, "ptnd.text.appearance.icon_style"))
        .icon(theme::ICON_STAR)
        .text(ui.studio_text(if style == IconStyle::Outline {
            "outline_icons"
        } else {
            "filled_icons"
        }))
        .on_press(move |_| state.set(style.toggled()))
}
fn appearance_text(ui: &UiShell, text_id: &str) -> String {
    let shell = ui.shell.read();
    shell
        .bridge
        .localization()
        .text(text_id, shell.bridge.locale())
}
fn accent_swatch(ui: UiShell, candidate: theme::AccentColor, active: bool) -> impl IntoElement {
    let mut state = ui.accent;
    rect()
        .width(Size::px(28.))
        .height(Size::px(28.))
        .corner_radius(theme::CONTROL_RADIUS)
        .background(candidate.value)
        .child(
            StudioButton::new(
                &ui,
                format!("{}: {}", ui.studio_text("accent"), candidate.id),
            )
            .width(Size::fill())
            .selected(active)
            .on_press(move |_| state.set(candidate)),
        )
}
