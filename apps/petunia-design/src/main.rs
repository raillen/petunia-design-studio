//! Petunia Design Studio desktop app, Freya shell (08.03).
//!
//! The shell is a projection: every mutation goes through the Action lane of
//! the Aubrieta GUI bridge and the UI re-renders from the shell state.

mod theme;

use freya::prelude::*;
use petunia_design_shell::menu::{MenuFamilyPresentation, PersonaPresentation};
use petunia_design_shell::{PetuniaDesignGuiBridge, PetuniaShell};

const WINDOW_WIDTH: f64 = 1280.;
const WINDOW_HEIGHT: f64 = 800.;

/// Default document name: domain data, not UI copy.
const DEFAULT_DOCUMENT_TITLE: &str = "Untitled";

fn main() {
    let bridge = PetuniaDesignGuiBridge::new();
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

    rect()
        .direction(Direction::Vertical)
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::SURFACE_WORKSPACE)
        .child(MenuBarRow(shell))
        .child(DocumentTabStrip(shell))
        .child(Workspace)
        .child(StatusBar(shell))
}

/// Menu bar + persona row: brand slot, menu families, spacer, personas.
#[derive(Clone, PartialEq)]
struct MenuBarRow(State<PetuniaShell>);

impl Component for MenuBarRow {
    fn render(&self) -> impl IntoElement {
        let shell = self.0;
        let bar = shell.peek().bridge.query_menu_bar();
        let personas = shell.peek().bridge.personas();
        let active_persona = shell.peek().bridge.persona();

        rect()
            .direction(Direction::Horizontal)
            .width(Size::fill())
            .height(Size::px(theme::PERSONA_ROW_HEIGHT))
            .background(theme::SURFACE_CHROME)
            .padding(Gaps::new(0., theme::SPACE_2, 0., theme::SPACE_2))
            .spacing(theme::SPACE_2)
            .cross_align(Alignment::Center)
            .child(brand_slot())
            .children(bar.families.iter().map(family_button))
            .child(rect().width(Size::fill()).height(Size::fill()))
            .children(
                personas
                    .iter()
                    .map(|persona| persona_button(shell, persona, persona.id == active_persona)),
            )
    }
}

/// Document tab strip: the open document tab.
#[derive(Clone, PartialEq)]
struct DocumentTabStrip(State<PetuniaShell>);

impl Component for DocumentTabStrip {
    fn render(&self) -> impl IntoElement {
        rect()
            .direction(Direction::Horizontal)
            .width(Size::fill())
            .height(Size::px(theme::TAB_STRIP_HEIGHT))
            .background(theme::SURFACE_CHROME_STRONG)
            .padding(Gaps::new(0., theme::SPACE_2, 0., theme::SPACE_2))
            .cross_align(Alignment::Center)
            .child(
                label()
                    .text(document_title(&self.0))
                    .color(theme::TEXT_PRIMARY)
                    .font_size(theme::CAPTION_SIZE),
            )
    }
}

/// The document workspace. The canvas slice (08.29) draws the artboard here.
#[derive(Clone, PartialEq)]
struct Workspace;

impl Component for Workspace {
    fn render(&self) -> impl IntoElement {
        rect()
            .width(Size::fill())
            .expanded()
            .background(theme::SURFACE_WORKSPACE)
    }
}

/// Status bar: document title and zoom readout, all from catalog and state.
#[derive(Clone, PartialEq)]
struct StatusBar(State<PetuniaShell>);

impl Component for StatusBar {
    fn render(&self) -> impl IntoElement {
        let shell_ref = self.0.peek();
        let title = shell_ref.bridge.session().map_or_else(
            || DEFAULT_DOCUMENT_TITLE.to_string(),
            |session| session.title().to_string(),
        );
        let zoom_pct = shell_ref.view_camera().zoom * 100.;
        let zoom_label = shell_ref
            .bridge
            .localization()
            .text("ptnd.text.shell.zoom_readout", shell_ref.bridge.locale());

        rect()
            .direction(Direction::Horizontal)
            .width(Size::fill())
            .height(Size::px(theme::STATUS_BAR_HEIGHT))
            .background(theme::SURFACE_CHROME_STRONG)
            .padding(Gaps::new(0., theme::SPACE_2, 0., theme::SPACE_2))
            .cross_align(Alignment::Center)
            .child(
                label()
                    .text(format!("{title} · {zoom_label}: {zoom_pct:.0}%"))
                    .color(theme::TEXT_SECONDARY)
                    .font_size(theme::CAPTION_SIZE),
            )
    }
}

fn brand_slot() -> impl IntoElement {
    rect()
        .width(Size::px(theme::BRAND_MARK_SIZE))
        .height(Size::px(theme::BRAND_MARK_SIZE))
        .background(theme::SURFACE_CHROME_STRONG)
}

fn family_button(family: &MenuFamilyPresentation) -> impl IntoElement {
    rect()
        .padding(Gaps::new(0., theme::SPACE_2, 0., theme::SPACE_2))
        .center()
        .child(
            label()
                .text(family.label.clone())
                .color(theme::TEXT_SECONDARY)
                .font_size(theme::BODY_SIZE),
        )
}

fn persona_button(
    shell: State<PetuniaShell>,
    persona: &PersonaPresentation,
    active: bool,
) -> impl IntoElement {
    let id = persona.id.clone();
    let mut shell = shell;
    rect()
        .padding(Gaps::new(0., theme::SPACE_2, 0., theme::SPACE_2))
        .center()
        .background(if active {
            theme::ACCENT_BLOOM
        } else {
            Color::TRANSPARENT
        })
        .on_press(move |_| {
            let _ = shell.write().bridge.set_persona(&id);
        })
        .child(
            label()
                .text(persona.label.clone())
                .color(if active {
                    theme::TEXT_PRIMARY
                } else {
                    theme::TEXT_TERTIARY
                })
                .font_size(theme::CAPTION_SIZE),
        )
}

fn document_title(shell: &State<PetuniaShell>) -> String {
    shell.peek().bridge.session().map_or_else(
        || DEFAULT_DOCUMENT_TITLE.to_string(),
        |session| session.title().to_string(),
    )
}
