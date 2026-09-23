//! Petunia Design Studio desktop app, Freya shell (08.03).
//!
//! The shell is a projection: every mutation goes through the Action lane of
//! the Aubrieta GUI bridge and the UI re-renders from the shell state.
//! Domain crates never import toolkit types; the UI receives DTOs and sends
//! activation tokens back across the bridge.

mod actions;
mod appearance;
mod chrome;
mod dialogs;
mod theme;
mod ui_state;

use freya::prelude::*;
use petunia_design_shell::PetuniaShell;

use crate::appearance::AppearanceBar;
use crate::chrome::{ContextToolbar, DocumentTabStrip, FamilyPopup, MenuBarRow, ToolRail};
use crate::dialogs::{CommandPalette, CustomizeDialog};
use crate::ui_state::UiShell;

const WINDOW_WIDTH: f64 = 1280.;
const WINDOW_HEIGHT: f64 = 800.;

/// Default document name: domain data, not UI copy.
const DEFAULT_DOCUMENT_TITLE: &str = "Untitled";

fn main() {
    let bridge = petunia_design_shell::PetuniaDesignGuiBridge::new();
    let title = bridge
        .localization()
        .text("ptnd.text.shell.brand", bridge.locale());

    launch(
        LaunchConfig::new().with_window(
            WindowConfig::new(app)
                .with_size(WINDOW_WIDTH, WINDOW_HEIGHT)
                .with_window_attributes(move |attributes, _| attributes.with_title(title)),
        ),
    );
}

fn app() -> impl IntoElement {
    use_init_theme(theme::petunia_theme);

    let shell = use_state(|| {
        let mut shell = PetuniaShell::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        shell
            .new_document(DEFAULT_DOCUMENT_TITLE)
            .expect("a fresh document opens");
        shell
    });
    let ui = UiShell::fresh(shell);

    rect()
        .direction(Direction::Vertical)
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::SURFACE_WORKSPACE)
        .child(MenuBarRow(ui.clone()))
        .child(DocumentTabStrip(ui.clone()))
        .child(ContextToolbar(ui.clone()))
        .child(
            rect()
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .expanded()
                .child(ToolRail(ui.clone()))
                .child(Workspace(ui.clone())),
        )
        .child(StatusBar(ui.clone()))
        .child(FamilyPopup(ui.clone()))
        .child(CommandPalette(ui.clone()))
        .child(CustomizeDialog(ui))
}

/// The document workspace. The canvas slice (08.29) draws the artboard here.
#[derive(Clone, PartialEq)]
struct Workspace(UiShell);

impl Component for Workspace {
    fn render(&self) -> impl IntoElement {
        rect()
            .width(Size::fill())
            .expanded()
            .background(theme::SURFACE_WORKSPACE)
    }
}

/// Status bar: document title, zoom and persona hint, all catalog-resolved.
#[derive(Clone, PartialEq)]
struct StatusBar(UiShell);

impl Component for StatusBar {
    fn render(&self) -> impl IntoElement {
        let shell_ref = self.0.shell.peek();
        let title = shell_ref.bridge.session().map_or_else(
            || DEFAULT_DOCUMENT_TITLE.to_string(),
            |s| s.title().to_string(),
        );
        let zoom_pct = (shell_ref.view_camera().zoom * 100.).round() as i32;
        let zoom_label = shell_ref
            .bridge
            .localization()
            .text("ptnd.text.shell.zoom_readout", shell_ref.bridge.locale());
        // A hovered control explains itself here: tooltips live in the status
        // bar so buttons stay exactly where the registry put them.
        let hovered = self.0.hovered.read().clone();
        let hint = hovered.map_or_else(
            || {
                shell_ref
                    .bridge
                    .persona_hint(shell_ref.bridge.persona())
                    .unwrap_or_default()
            },
            |(_, text)| text,
        );

        rect()
            .direction(Direction::Horizontal)
            .width(Size::fill())
            .height(Size::px(theme::STATUS_BAR_HEIGHT))
            .background(theme::SURFACE_CHROME_STRONG)
            .padding(Gaps::new(0., theme::SPACE_2, 0., theme::SPACE_2))
            .spacing(theme::SPACE_2)
            .cross_align(Alignment::Center)
            .child(
                label()
                    .text(format!("{title} · {zoom_label}: {zoom_pct}%"))
                    .color(theme::TEXT_SECONDARY)
                    .font_size(theme::CAPTION_SIZE),
            )
            .child(rect().width(Size::fill()))
            .child(
                label()
                    .text(hint)
                    .color(theme::TEXT_TERTIARY)
                    .font_size(theme::CAPTION_SIZE),
            )
            .child(AppearanceBar(self.0.clone()))
    }
}
