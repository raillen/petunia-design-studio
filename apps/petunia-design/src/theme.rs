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

/// Product meaning stays attached to these semantic IDs, not raw SVG names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppIcon {
    pub outline: &'static [u8],
    pub filled: Option<&'static [u8]>,
}

impl AppIcon {
    #[must_use]
    pub fn bytes(self, style: IconStyle) -> &'static [u8] {
        match style {
            IconStyle::Outline => self.outline,
            IconStyle::Filled => self.filled.unwrap_or(self.outline),
        }
    }
}

macro_rules! icon {
    ($name:ident, $outline:literal, $filled:literal) => {
        #[allow(dead_code)]
        pub const $name: AppIcon = AppIcon {
            outline: include_bytes!(concat!("../assets/icons-outline/", $outline, ".svg")),
            filled: Some(include_bytes!(concat!(
                "../assets/icons-filled/",
                $filled,
                ".svg"
            ))),
        };
    };
}

macro_rules! outline_icon {
    ($name:ident, $outline:literal) => {
        #[allow(dead_code)]
        pub const $name: AppIcon = AppIcon {
            outline: include_bytes!(concat!("../assets/icons-outline/", $outline, ".svg")),
            filled: None,
        };
    };
}

// Domain tools without an equivalent filled glyph retain the outline asset.
icon!(ICON_SETTINGS, "settings", "settings");
icon!(
    ICON_CUSTOMIZE,
    "adjustments-horizontal",
    "adjustments-horizontal"
);
icon!(ICON_OVERFLOW, "dots", "dots");
outline_icon!(ICON_CHEVRON_UP, "chevron-up");
icon!(ICON_CHEVRON_DOWN, "chevron-down", "chevron-down");
icon!(ICON_CHEVRON_RIGHT, "chevron-right", "chevron-right");
icon!(ICON_CLOSE, "x", "x");
icon!(ICON_PLUS, "plus", "plus");
icon!(ICON_CHECK, "check", "check");
outline_icon!(ICON_UNDO, "undo-2");
outline_icon!(ICON_REDO, "redo-2");
icon!(ICON_ZOOM_IN, "zoom-in", "zoom-in");
icon!(ICON_ZOOM_OUT, "zoom-out", "zoom-out");
outline_icon!(ICON_FIT, "maximize");
icon!(ICON_SEARCH, "search", "search");
icon!(ICON_PALETTE, "command", "palette");
icon!(ICON_TRASH, "trash", "trash");
icon!(ICON_COPY, "copy", "copy");
icon!(ICON_LOCK, "lock", "lock");
icon!(ICON_EYE, "eye", "eye");
outline_icon!(ICON_EYE_OFF, "eye-off");
icon!(ICON_INFO, "info", "info-circle");
icon!(ICON_STAR, "star", "star");
icon!(ICON_PHOTO, "photo", "photo");
icon!(ICON_POINTER, "pointer", "pointer");
outline_icon!(ICON_TEXT, "cursor-text");
outline_icon!(ICON_PEN, "pen-tool");
icon!(ICON_PENCIL, "pencil", "pencil");
icon!(ICON_SQUARE, "square", "square");
icon!(ICON_CIRCLE, "circle", "circle");
icon!(ICON_TRIANGLE, "triangle", "triangle");
icon!(ICON_HEXAGON, "hexagon", "hexagon");
outline_icon!(ICON_MOVE, "move");
outline_icon!(ICON_HAND, "hand");
outline_icon!(ICON_NODE, "vector-bezier-2");
outline_icon!(ICON_BRUSH, "brush");
outline_icon!(ICON_ERASER, "eraser");
outline_icon!(ICON_PAINT, "brush");
outline_icon!(ICON_WAND, "wand");
outline_icon!(ICON_GRADIENT, "gradient");
outline_icon!(ICON_TRANSPARENCY, "droplet-half-2");
outline_icon!(ICON_CROP, "crop");
outline_icon!(ICON_EYEDROPPER, "pipette");
outline_icon!(ICON_LAYERS, "layers");
outline_icon!(ICON_PERSPECTIVE, "perspective");
outline_icon!(ICON_CORNER, "radius-top-left");
outline_icon!(ICON_CONTOUR, "circle-dashed");
outline_icon!(ICON_KNIFE, "knife");
outline_icon!(ICON_SCISSORS, "scissors");
outline_icon!(ICON_FILL, "bucket-droplet");
outline_icon!(ICON_BOOLEAN, "layers-intersect");
outline_icon!(ICON_COLOR_PICKER, "pipette");
outline_icon!(ICON_ATTRIBUTE_PICKER, "color-swatch");
icon!(ICON_ARTBOARD, "artboard", "artboard");
outline_icon!(ICON_MARQUEE_RECT, "marquee");
outline_icon!(ICON_MARQUEE_ELLIPSE, "marquee-2");
outline_icon!(ICON_LASSO, "lasso-polygon");
outline_icon!(ICON_MEASURE, "ruler-measure");
outline_icon!(ICON_SELECTION_BRUSH, "brush");
outline_icon!(ICON_CROP_PHOTO, "crop");
outline_icon!(ICON_SHAPE_BUILDER, "layers-intersect");
outline_icon!(ICON_FRAME_TEXT, "text-wrap");
outline_icon!(ICON_FLOOD_SELECT, "wand");
outline_icon!(ICON_TYPE, "type");
outline_icon!(ICON_ROTATE, "rotate");
icon!(ICON_MENU, "menu-2", "menu-2");
icon!(ICON_WARNING, "info", "alert-triangle");
icon!(ICON_SPARKLES, "sparkles", "sparkles");
icon!(ICON_GRIP, "dots", "dots-vertical");

// Sizes are the chrome geometry of 08.35 expressed in Freya pixels.
#[allow(dead_code)]
pub const ICON_TOOL_RAIL: f32 = 20.;
pub const ICON_TOOLBAR: f32 = 18.;
pub const ICON_INLINE: f32 = 16.;
pub const TOOL_RAIL_WIDTH: f32 = 56.;
pub const TOOL_BUTTON: f32 = 34.;
pub const TOOL_GAP: f32 = 2.;
pub const PERSONA_ROW_HEIGHT: f32 = 40.;
pub const TAB_STRIP_HEIGHT: f32 = 30.;
pub const STATUS_BAR_HEIGHT: f32 = 26.;
pub const TOOLBAR_HEIGHT: f32 = 34.;
#[allow(dead_code)]
pub const MENU_ROW_HEIGHT: f32 = 28.;
pub const BODY_SIZE: f32 = 12.;
pub const CAPTION_SIZE: f32 = 11.;
pub const SPACE_1: f32 = 4.;
pub const SPACE_2: f32 = 8.;
pub const SPACE_3: f32 = 12.;
#[allow(dead_code)]
pub const SPACE_4: f32 = 16.;
pub const BRAND_MARK_SIZE: f32 = 20.;
#[allow(dead_code)]
pub const PERSONA_TAB_HEIGHT: f32 = 30.;

pub const CONTROL_HEIGHT: f32 = 28.;
pub const CONTROL_RADIUS: f32 = 4.;
pub const PANEL_HEADER_HEIGHT: f32 = 32.;
pub const SURFACE_HOVER: Color = Color::from_rgb(0x3B, 0x3E, 0x43);
pub const SURFACE_SELECTED: Color = Color::from_rgb(0x3F, 0x49, 0x5C);
pub const BORDER_ACTIVE: Color = Color::from_rgb(0x7E, 0x8B, 0xA0);
pub const TEXT_ON_ACCENT: Color = Color::from_rgb(0x18, 0x19, 0x1B);
pub const STATE_WARNING: Color = Color::from_rgb(0xE8, 0xC3, 0x3C);
pub const STATE_SUCCESS: Color = Color::from_rgb(0x57, 0xC7, 0x84);

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
/// Recoverable error feedback shared by desktop prompts.
pub const TEXT_ERROR: Color = Color::from_rgb(0xE5, 0x6B, 0x6B);
pub const TEXT_SECONDARY: Color = Color::from_rgb(0xC2, 0xC6, 0xCC);
pub const TEXT_TERTIARY: Color = Color::from_rgb(0xB4, 0xB8, 0xBF);
// Controls the registry blocks: visible, but plainly not actionable.
pub const TEXT_DISABLED: Color = Color::from_rgb(0x83, 0x88, 0x90);
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
        text_inverse: TEXT_ON_ACCENT,
        text_secondary: TEXT_SECONDARY,
        text_placeholder: TEXT_TERTIARY,
        ..DARK_COLORS
    };
    for (key, height, padding) in [
        ("button_layout", 28., Gaps::new_symmetric(0., 10.)),
        ("compact_button_layout", 24., Gaps::new_symmetric(0., 6.)),
        ("expanded_button_layout", 32., Gaps::new_symmetric(0., 12.)),
    ] {
        theme.set(
            key,
            ButtonLayoutThemePreference {
                height: Preference::Specific(Size::px(height)),
                width: Preference::Specific(Size::Inner),
                padding: Preference::Specific(padding),
                margin: Preference::Specific(Gaps::new_all(0.)),
                corner_radius: Preference::Specific(CornerRadius::new_all(CONTROL_RADIUS)),
            },
        );
    }
    theme.set(
        "input_layout",
        InputLayoutThemePreference {
            padding: Preference::Specific(Gaps::new_symmetric(6., 8.)),
            corner_radius: Preference::Specific(CornerRadius::new_all(CONTROL_RADIUS)),
        },
    );
    theme.set(
        "typography",
        TypographyThemePreference {
            title: Preference::Specific(20.),
            subtitle: Preference::Specific(16.),
            body: Preference::Specific(BODY_SIZE),
            caption: Preference::Specific(CAPTION_SIZE),
            overline: Preference::Specific(CAPTION_SIZE),
        },
    );
    theme
}
