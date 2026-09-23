//! UI tokens for the Petunia Freya shell (08.35).
//!
//! Palette, chrome metrics, icon style and accent colors live here and nowhere
//! else: a theme change is a single edit and no component owns a literal.

use freya::prelude::*;

/// Which Tabler icon set the shell paints.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconStyle {
    /// `assets/icons-outline`: 24px grid, 2px stroke, `currentColor`.
    #[default]
    Outline,
    /// `assets/icons-filled`: solid glyphs, recolored through
    /// `SvgViewer::fill`.
    Filled,
}

impl IconStyle {
    #[must_use]
    pub fn toggled(self) -> Self {
        match self {
            Self::Outline => Self::Filled,
            Self::Filled => Self::Outline,
        }
    }
}

/// Accent colors the shell offers. Names are UI copy resolved from the catalog
/// by the caller; the values are the only literals allowed here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccentColor {
    pub id: &'static str,
    pub value: Color,
}

pub const BLOOM: AccentColor = AccentColor {
    id: "bloom",
    value: Color::from_rgb(0xB7, 0x7A, 0xFF),
};
pub const DESIGN: AccentColor = AccentColor {
    id: "design",
    value: Color::from_rgb(0x35, 0xC7, 0xD4),
};
pub const PHOTO: AccentColor = AccentColor {
    id: "photo",
    value: Color::from_rgb(0xF0, 0x6C, 0x8D),
};
pub const EMBER: AccentColor = AccentColor {
    id: "ember",
    value: Color::from_rgb(0xFF, 0x9B, 0x51),
};
pub const MOSS: AccentColor = AccentColor {
    id: "moss",
    value: Color::from_rgb(0x57, 0xC7, 0x84),
};
pub const GOLD: AccentColor = AccentColor {
    id: "gold",
    value: Color::from_rgb(0xE8, 0xC3, 0x3C),
};

pub const ACCENTS: &[AccentColor] = &[BLOOM, DESIGN, PHOTO, EMBER, MOSS, GOLD];

/// The shell's icon vocab: a semantic name plus the Tabler asset in each set.
///
/// Only entries whose asset exists in **both** sets are listed, so switching
/// [`IconStyle`] can never show a missing glyph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppIcon {
    pub outline: &'static [u8],
    pub filled: &'static [u8],
}

impl AppIcon {
    #[must_use]
    pub fn bytes(self, style: IconStyle) -> &'static [u8] {
        match style {
            IconStyle::Outline => self.outline,
            IconStyle::Filled => self.filled,
        }
    }
}

macro_rules! icon {
    ($name:ident, $outline:literal, $filled:literal) => {
        #[allow(dead_code)]
        pub const $name: AppIcon = AppIcon {
            outline: include_bytes!(concat!("../assets/icons-outline/", $outline, ".svg")),
            filled: include_bytes!(concat!("../assets/icons-filled/", $filled, ".svg")),
        };
    };
}

// Filled Tabler has no undo/redo/fit/pen/text/cut/pan glyphs, so the filled
// set reuses the closest solid metaphor; outline stays the literal one.
icon!(ICON_SETTINGS, "settings", "settings");
icon!(
    ICON_CUSTOMIZE,
    "adjustments-horizontal",
    "adjustments-horizontal"
);
icon!(ICON_OVERFLOW, "dots", "dots");
icon!(ICON_CHEVRON_DOWN, "chevron-down", "chevron-down");
icon!(ICON_CHEVRON_RIGHT, "chevron-right", "chevron-right");
icon!(ICON_CLOSE, "x", "x");
icon!(ICON_PLUS, "plus", "plus");
icon!(ICON_CHECK, "check", "check");
icon!(ICON_UNDO, "undo-2", "circle-dot");
icon!(ICON_REDO, "redo-2", "circle-dot");
icon!(ICON_ZOOM_IN, "zoom-in", "zoom-in");
icon!(ICON_ZOOM_OUT, "zoom-out", "zoom-out");
icon!(ICON_FIT, "maximize", "point");
icon!(ICON_SEARCH, "search", "search");
icon!(ICON_PALETTE, "command", "palette");
icon!(ICON_TRASH, "trash", "trash");
icon!(ICON_COPY, "copy", "copy");
icon!(ICON_LOCK, "lock", "lock");
icon!(ICON_EYE, "eye", "eye");
icon!(ICON_EYE_OFF, "eye-off", "eye");
icon!(ICON_INFO, "info", "info-circle");
icon!(ICON_STAR, "star", "star");
icon!(ICON_PHOTO, "photo", "photo");
icon!(ICON_POINTER, "pointer", "pointer");
icon!(ICON_TEXT, "cursor-text", "file-typography");
icon!(ICON_PEN, "pen-tool", "edit");
icon!(ICON_PENCIL, "pencil", "pencil");
icon!(ICON_SQUARE, "square", "square");
icon!(ICON_CIRCLE, "circle", "circle");
icon!(ICON_TRIANGLE, "triangle", "triangle");
icon!(ICON_HEXAGON, "hexagon", "hexagon");

// Sizes are the chrome geometry of 08.35 expressed in Freya pixels.
#[allow(dead_code)]
pub const ICON_TOOL_RAIL: f32 = 20.;
pub const ICON_TOOLBAR: f32 = 18.;
pub const ICON_INLINE: f32 = 16.;
pub const TOOL_RAIL_WIDTH: f32 = 44.;
pub const TOOL_BUTTON: f32 = 34.;
pub const TOOL_GAP: f32 = 2.;
pub const PERSONA_ROW_HEIGHT: f32 = 40.;
pub const TAB_STRIP_HEIGHT: f32 = 30.;
pub const STATUS_BAR_HEIGHT: f32 = 26.;
pub const TOOLBAR_HEIGHT: f32 = 34.;
#[allow(dead_code)]
pub const MENU_ROW_HEIGHT: f32 = 28.;
pub const BODY_SIZE: f32 = 13.;
pub const CAPTION_SIZE: f32 = 11.;
pub const SPACE_1: f32 = 4.;
pub const SPACE_2: f32 = 8.;
#[allow(dead_code)]
pub const SPACE_4: f32 = 16.;
pub const BRAND_MARK_SIZE: f32 = 20.;
#[allow(dead_code)]
pub const PERSONA_TAB_HEIGHT: f32 = 30.;

// Canvas and workspace ground.
pub const SURFACE_WORKSPACE: Color = Color::from_rgb(0x20, 0x21, 0x24);
// Menu bar ground.
pub const SURFACE_CHROME: Color = Color::from_rgb(0x27, 0x28, 0x2B);
// Recessed chrome: tab strip and status bar.
pub const SURFACE_CHROME_STRONG: Color = Color::from_rgb(0x22, 0x23, 0x26);
// Floating surfaces: dropdowns, popovers, dialogs.
pub const SURFACE_PANEL: Color = Color::from_rgb(0x30, 0x32, 0x36);
pub const BORDER_SUBTLE: Color = Color::from_rgb(0x41, 0x44, 0x4A);
pub const TEXT_PRIMARY: Color = Color::from_rgb(0xF2, 0xF3, 0xF5);
pub const TEXT_SECONDARY: Color = Color::from_rgb(0xC2, 0xC6, 0xCC);
pub const TEXT_TERTIARY: Color = Color::from_rgb(0x8E, 0x94, 0x9D);
pub const ACCENT_BLOOM: Color = Color::from_rgb(0xB7, 0x7A, 0xFF);
pub const STUDIO_DESIGN: Color = Color::from_rgb(0x35, 0xC7, 0xD4);
pub const STUDIO_PHOTO: Color = Color::from_rgb(0xF0, 0x6C, 0x8D);

/// The dark theme carrying the Petunia palette.
pub fn petunia_theme() -> Theme {
    let mut theme = dark_theme();
    theme.name = "petunia";
    theme.colors = ColorsSheet {
        primary: ACCENT_BLOOM,
        secondary: STUDIO_DESIGN,
        tertiary: STUDIO_PHOTO,
        background: SURFACE_WORKSPACE,
        surface_primary: SURFACE_CHROME,
        surface_secondary: SURFACE_PANEL,
        surface_tertiary: SURFACE_CHROME_STRONG,
        border: BORDER_SUBTLE,
        border_focus: ACCENT_BLOOM,
        text_primary: TEXT_PRIMARY,
        text_secondary: TEXT_SECONDARY,
        text_placeholder: TEXT_TERTIARY,
        ..DARK_COLORS
    };
    theme
}
