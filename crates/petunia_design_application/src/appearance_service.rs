//! Appearance builders and style sampling shared by panels, tools and MCP (10.4).
//!
//! Builders produce `AppearanceStack` values or granular `Command`s; they
//! never touch documents. Callers submit through the command lane.

use petunia_design_document::{
    AppearanceStack, BlendMode, DocumentObject, EffectItem, FillItem, GradientStop, LinearGradient,
    Paint, RadialGradient, StrokeItem,
};
use petunia_design_foundation::ObjectId;

use super::commands::Command;

/// Builds a two-stop linear gradient fill item (10.4).
#[must_use]
pub fn linear_gradient_fill(
    id: u32,
    start: [f64; 2],
    end: [f64; 2],
    color_start: impl Into<String>,
    color_end: impl Into<String>,
) -> FillItem {
    FillItem::linear_gradient(
        id,
        LinearGradient::new(
            start,
            end,
            vec![
                GradientStop::with_id(0.0, color_start, id * 2 + 1),
                GradientStop::with_id(1.0, color_end, id * 2 + 2),
            ],
        ),
    )
}

/// Builds a two-stop radial gradient fill item (10.4).
#[must_use]
pub fn radial_gradient_fill(
    id: u32,
    center: [f64; 2],
    radius: f64,
    color_start: impl Into<String>,
    color_end: impl Into<String>,
) -> FillItem {
    FillItem::radial_gradient(
        id,
        RadialGradient::new(
            center,
            radius,
            vec![
                GradientStop::with_id(0.0, color_start, id * 2 + 1),
                GradientStop::with_id(1.0, color_end, id * 2 + 2),
            ],
        ),
    )
}

/// Applies a gradient paint to the primary fill entry of a stack,
/// appending a new entry when the stack has no fills (F-18).
/// Unlike the legacy bridge behavior, secondary entries are never
/// silently overwritten (10.4).
#[must_use]
pub fn with_primary_gradient(mut stack: AppearanceStack, paint: Paint) -> AppearanceStack {
    match stack.fills.first_mut() {
        Some(first) => {
            first.paint = paint;
        }
        None => {
            stack.fills.push(FillItem {
                id: 1,
                paint,
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
                visible: true,
            });
        }
    }
    stack
}

/// Granular property filter for style sampling (F-18, Table B).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StyleFilter {
    /// Copy fill styling (appearance fills or legacy fill token).
    pub fill: bool,
    /// Copy stroke styling (appearance strokes or legacy stroke token + width).
    pub stroke: bool,
    /// Copy appearance stack effects (drop shadows, blurs, etc.).
    pub effects: bool,
    /// Copy typography properties (font family, font size, line height, letter spacing).
    pub typography: bool,
}

impl Default for StyleFilter {
    fn default() -> Self {
        Self {
            fill: true,
            stroke: true,
            effects: true,
            typography: true,
        }
    }
}

/// Sampled style payload copied by the eyedropper (Table B).
#[derive(Clone, Debug, PartialEq)]
pub struct SampledStyle {
    /// Source fill token, if any.
    pub fill: Option<String>,
    /// Source stroke token and width.
    pub stroke: Option<(String, f64)>,
    /// Full source appearance stack, if any.
    pub appearance: Option<AppearanceStack>,
}

/// Samples the paint style of a source object (Table B).
#[must_use]
pub fn sample_style(source: &DocumentObject) -> SampledStyle {
    SampledStyle {
        fill: source.fill.clone(),
        stroke: source
            .stroke
            .clone()
            .map(|token| (token, source.stroke_width)),
        appearance: source.appearance.clone(),
    }
}

/// Builds granular style-application commands for one target (F-18).
/// Prefers the full appearance stack when present (preserving gradients
/// and multi-entry stacks); otherwise mirrors legacy fill/stroke tokens.
#[must_use]
pub fn style_sample_commands(target_id: ObjectId, style: &SampledStyle) -> Vec<Command> {
    if let Some(stack) = &style.appearance {
        return vec![Command::SetAppearance {
            id: target_id,
            appearance: Some(stack.clone()),
        }];
    }
    let mut cmds = Vec::with_capacity(2);
    cmds.push(Command::SetFill {
        id: target_id,
        fill: style.fill.clone(),
    });
    if let Some((stroke, width)) = &style.stroke {
        cmds.push(Command::SetStroke {
            id: target_id,
            stroke: Some(stroke.clone()),
            width: *width,
        });
    }
    cmds
}

/// Builds granular style-application commands for one target object using a filter.
#[must_use]
pub fn filtered_style_commands(
    target: &DocumentObject,
    source: &DocumentObject,
    filter: &StyleFilter,
) -> Vec<Command> {
    let mut cmds = Vec::new();
    let mut target_stack = target.effective_appearance();
    let source_stack = source.effective_appearance();
    let mut appearance_changed = false;

    if filter.fill {
        target_stack.fills = source_stack.fills.clone();
        appearance_changed = true;
    }
    if filter.stroke {
        target_stack.strokes = source_stack.strokes.clone();
        appearance_changed = true;
    }
    if filter.effects {
        target_stack.effects = source_stack.effects.clone();
        appearance_changed = true;
    }

    if appearance_changed {
        cmds.push(Command::SetAppearance {
            id: target.id,
            appearance: Some(target_stack),
        });
    }

    if filter.typography {
        if let (
            Some(petunia_design_document::ShapeKind::Text {
                font_family,
                font_size,
                line_height,
                letter_spacing,
                ..
            }),
            Some(petunia_design_document::ShapeKind::Text {
                content,
                on_path,
                ..
            }),
        ) = (&source.shape, &target.shape)
        {
            cmds.push(Command::SetShape {
                id: target.id,
                shape: Some(petunia_design_document::ShapeKind::Text {
                    content: content.clone(),
                    font_family: font_family.clone(),
                    font_size: *font_size,
                    line_height: *line_height,
                    letter_spacing: *letter_spacing,
                    on_path: on_path.clone(),
                }),
            });
        }
    }

    cmds
}

/// Builds an effect-toggle command (F-18).
#[must_use]
pub fn toggle_effect_command(id: ObjectId, effect: &EffectItem, visible: bool) -> Command {
    Command::ToggleEffect {
        id,
        effect_id: effect.id,
        visible,
    }
}

/// Builds a solid fill item with an explicit id.
#[must_use]
pub fn solid_fill(id: u32, color: impl Into<String>) -> FillItem {
    FillItem::solid(id, color)
}

/// Builds a solid stroke item with an explicit id and width.
#[must_use]
pub fn solid_stroke(id: u32, color: impl Into<String>, width: f64) -> StrokeItem {
    StrokeItem::solid(id, color, width)
}
