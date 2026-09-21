//! Parametric shape and text factories with canonical V1 defaults (10.3, 10.6).
//!
//! Centralizes the default geometry, names and fills that interactive tools
//! used to duplicate: every creation path funnels through here so defaults
//! stay consistent across UI, plugins and MCP.

use crate::ShapeKind;

/// Default fill for newly created parametric shapes.
pub const DEFAULT_SHAPE_FILL: &str = "ptnd.blue/500";
/// Default fill for committed freehand paths.
pub const DEFAULT_PATH_STROKE: &str = "ptnd.gray/900";
/// Default fill for committed text objects.
pub const DEFAULT_TEXT_FILL: &str = "ptnd.gray/900";
/// Default freehand stroke width in points.
pub const DEFAULT_PATH_STROKE_WIDTH: f64 = 1.5;
/// Default pencil stroke width in points.
pub const DEFAULT_PENCIL_STROKE_WIDTH: f64 = 2.0;
/// Placeholder text content for new text objects.
pub const DEFAULT_TEXT_CONTENT: &str = "Petunia Typography";
/// Default text font family.
pub const DEFAULT_TEXT_FONT_FAMILY: &str = "Inter";
/// Default artistic headline size in points.
pub const DEFAULT_ARTISTIC_FONT_SIZE: f64 = 24.0;
/// Default frame text size in points.
pub const DEFAULT_FRAME_FONT_SIZE: f64 = 14.0;
/// Default line height factor.
pub const DEFAULT_TEXT_LINE_HEIGHT: f64 = 1.3;

/// Default parametric rectangle with sharp corners.
#[must_use]
pub fn rectangle() -> (&'static str, ShapeKind) {
    (
        "Rectangle",
        ShapeKind::Rectangle {
            corner_radii: [0.0; 4],
        },
    )
}

/// Default parametric ellipse.
#[must_use]
pub fn ellipse() -> (&'static str, ShapeKind) {
    ("Ellipse", ShapeKind::Ellipse)
}

/// Default regular pentagon.
#[must_use]
pub fn polygon() -> (&'static str, ShapeKind) {
    ("Polygon", ShapeKind::Polygon { sides: 5 })
}

/// Default five-point star with 0.5 inner ratio.
#[must_use]
pub fn star() -> (&'static str, ShapeKind) {
    (
        "Star",
        ShapeKind::Star {
            points: 5,
            inner_ratio: 0.5,
        },
    )
}

/// Default artistic headline text shape.
#[must_use]
pub fn artistic_text() -> (&'static str, ShapeKind) {
    (
        "Artistic Text",
        ShapeKind::Text {
            content: DEFAULT_TEXT_CONTENT.to_string(),
            font_family: DEFAULT_TEXT_FONT_FAMILY.to_string(),
            font_size: DEFAULT_ARTISTIC_FONT_SIZE,
            line_height: DEFAULT_TEXT_LINE_HEIGHT,
            letter_spacing: 0.0,
        },
    )
}

/// Default frame (paragraph) text shape.
#[must_use]
pub fn frame_text() -> (&'static str, ShapeKind) {
    (
        "Text Frame",
        ShapeKind::Text {
            content: DEFAULT_TEXT_CONTENT.to_string(),
            font_family: DEFAULT_TEXT_FONT_FAMILY.to_string(),
            font_size: DEFAULT_FRAME_FONT_SIZE,
            line_height: DEFAULT_TEXT_LINE_HEIGHT,
            letter_spacing: 0.0,
        },
    )
}
