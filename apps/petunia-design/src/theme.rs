//! UI tokens for the Petunia shell (08.35).
//!
//! Palette and chrome metrics live here and nowhere else: a theme change is a
//! single edit and no component owns a literal.

use freya::prelude::*;

/// Canvas and workspace ground.
pub const SURFACE_WORKSPACE: Color = Color::from_rgb(0x20, 0x21, 0x24);
/// Menu bar ground.
pub const SURFACE_CHROME: Color = Color::from_rgb(0x27, 0x28, 0x2B);
/// Recessed chrome: tab strip and status bar.
pub const SURFACE_CHROME_STRONG: Color = Color::from_rgb(0x22, 0x23, 0x26);
/// Floating surfaces: dropdowns, popovers, dialogs.
pub const SURFACE_PANEL: Color = Color::from_rgb(0x30, 0x32, 0x36);
pub const BORDER_SUBTLE: Color = Color::from_rgb(0x41, 0x44, 0x4A);
pub const TEXT_PRIMARY: Color = Color::from_rgb(0xF2, 0xF3, 0xF5);
pub const TEXT_SECONDARY: Color = Color::from_rgb(0xC2, 0xC6, 0xCC);
pub const TEXT_TERTIARY: Color = Color::from_rgb(0x8E, 0x94, 0x9D);
pub const ACCENT_BLOOM: Color = Color::from_rgb(0xB7, 0x7A, 0xFF);
pub const STUDIO_DESIGN: Color = Color::from_rgb(0x35, 0xC7, 0xD4);
pub const STUDIO_PHOTO: Color = Color::from_rgb(0xF0, 0x6C, 0x8D);

pub const PERSONA_ROW_HEIGHT: f32 = 40.;
pub const TAB_STRIP_HEIGHT: f32 = 30.;
pub const STATUS_BAR_HEIGHT: f32 = 26.;
pub const BRAND_MARK_SIZE: f32 = 20.;
pub const BODY_SIZE: f32 = 13.;
pub const CAPTION_SIZE: f32 = 11.;
pub const SPACE_2: f32 = 8.;

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
