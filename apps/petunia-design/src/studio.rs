//! Compact color Studio and immediate, semantic color operations (ADR-012).
use crate::{studio_widgets::StudioButton, theme, ui_state::UiShell};
use freya::prelude::*;
use freya_engine::prelude::{Paint, PaintStyle, Rect as SkRect};
use petunia_design_application::Command;
use petunia_design_document::ShapeKind;

pub const QUICK_COLORS: [&str; 12] = [
    "#FFFFFF", "#17191D", "#9299A3", "#F06C8D", "#FF9B51", "#E8C33C", "#57C784", "#35C7D4",
    "#4C8DE8", "#B77AFF", "#E069C4", "#8D6B54",
];

/// Vector selections preserve stroke width. Raster selections edit the brush,
/// never an ignored object fill; choosing RGB clears a previous literal ink.
pub fn apply_color(ui: &UiShell, token: &str, fill: bool) {
    let title = ui.studio_text("color_settings");
    let mut state = ui.shell;
    let mut shell = state.write();
    let ids = shell.bridge.selection().selected_ids;
    let raster = ids.len() == 1
        && shell
            .bridge
            .session()
            .and_then(|s| s.find_object(ids[0]))
            .is_some_and(|o| matches!(o.shape, Some(ShapeKind::Raster { .. })));
    if raster
        || (ids.is_empty()
            && *ui.persona.peek() == petunia_design_application::surfaces::PERSONA_PHOTO)
    {
        let rgb = petunia_design_document::resolve_color_to_rgb(token);
        let brush = shell.tools.photo_brush_tool_mut();
        let mut settings = brush.brush_settings();
        settings.color = [
            rgb[0] as f32,
            rgb[1] as f32,
            rgb[2] as f32,
            settings.color[3],
        ];
        settings.ink = None;
        brush.set_brush_settings(settings);
        return;
    }
    let commands = ids
        .into_iter()
        .filter_map(|id| {
            let object = shell.bridge.session()?.find_object(id)?;
            if object.locked || matches!(object.shape, Some(ShapeKind::Raster { .. })) {
                return None;
            }
            Some(if fill {
                Command::SetFill {
                    id,
                    fill: Some(token.to_owned()),
                }
            } else {
                Command::SetStroke {
                    id,
                    stroke: Some(token.to_owned()),
                    width: object.stroke_width,
                }
            })
        })
        .collect::<Vec<_>>();
    if !commands.is_empty() {
        if let Err(error) = shell.bridge.submit_all(&title, commands) {
            let mut notice = ui.file_error;
            notice.set(Some(error.to_string()));
        }
    }
}

pub fn color_target_enabled(ui: &UiShell) -> bool {
    let state = ui.shell.read();
    let selected = state.bridge.selection().selected_ids;
    let raster = selected.len() == 1
        && state
            .bridge
            .session()
            .and_then(|s| s.find_object(selected[0]))
            .is_some_and(|o| matches!(o.shape, Some(ShapeKind::Raster { .. })));
    raster
        || (selected.is_empty()
            && *ui.persona.read() == petunia_design_application::surfaces::PERSONA_PHOTO)
        || selected.into_iter().any(|id| {
            state
                .bridge
                .session()
                .and_then(|s| s.find_object(id))
                .is_some_and(|o| !o.locked && !matches!(o.shape, Some(ShapeKind::Raster { .. })))
        })
}

#[derive(Clone, PartialEq)]
pub struct CompactColor(pub UiShell);
impl Component for CompactColor {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let fill = *ui.studio_fill_target.read();
        let s = ui.shell.read();
        let selection = s.bridge.selection();
        let object = selection
            .selected_ids
            .first()
            .and_then(|id| s.bridge.session()?.find_object(*id));
        let raster = object.is_some_and(|o| matches!(o.shape, Some(ShapeKind::Raster { .. })));
        let brush_mode = raster
            || (*ui.persona.read() == petunia_design_application::surfaces::PERSONA_PHOTO
                && object.is_none());
        let rgb = if brush_mode {
            s.tools.photo_brush_tool().brush_settings().color[..3]
                .try_into()
                .unwrap_or([0., 0., 0.])
        } else {
            object
                .and_then(|o| {
                    if fill {
                        o.fill.as_deref()
                    } else {
                        o.stroke.as_deref()
                    }
                })
                .map(petunia_design_document::resolve_color_to_rgb)
                .unwrap_or([0.2, 0.78, 0.83])
        };
        let hex = rgb_token(rgb);
        let enabled = color_target_enabled(ui);
        drop(s);
        let mut target = ui.studio_fill_target;
        let mut tab = ui.dock_tab;
        let wheel_ui = ui.clone();
        let [hue, saturation, value] = rgb_to_hsv(rgb);
        let focus_id = use_a11y();
        let focus = use_focus(focus_id);
        let on_render = RenderCallback::new(move |context: &mut CanvasContext| {
            let canvas = context.canvas;
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_style(PaintStyle::Stroke);
            paint.set_stroke_width(12.);
            for degree in 0..360 {
                let a = (degree as f32).to_radians();
                let b = ((degree + 1) as f32).to_radians();
                let color = hsv_to_rgb(degree as f32, 1., 1.);
                paint.set_color(Color::from_rgb(
                    (color[0] * 255.) as u8,
                    (color[1] * 255.) as u8,
                    (color[2] * 255.) as u8,
                ));
                canvas.draw_line(
                    (68. + 58. * a.cos(), 68. + 58. * a.sin()),
                    (68. + 58. * b.cos(), 68. + 58. * b.sin()),
                    &paint,
                );
            }
            paint.set_style(PaintStyle::Fill);
            for y in 0..32 {
                for x in 0..32 {
                    let color = hsv_to_rgb(hue, x as f32 / 31., 1. - y as f32 / 31.);
                    paint.set_color(Color::from_rgb(
                        (color[0] * 255.) as u8,
                        (color[1] * 255.) as u8,
                        (color[2] * 255.) as u8,
                    ));
                    canvas.draw_rect(
                        SkRect::from_xywh(
                            31. + x as f32 * 2.3125,
                            31. + y as f32 * 2.3125,
                            2.5,
                            2.5,
                        ),
                        &paint,
                    );
                }
            }
            paint.set_style(PaintStyle::Stroke);
            paint.set_stroke_width(2.);
            paint.set_color(Color::WHITE);
            canvas.draw_circle(
                (
                    68. + 58. * hue.to_radians().cos(),
                    68. + 58. * hue.to_radians().sin(),
                ),
                5.,
                &paint,
            );
            canvas.draw_circle(
                (31. + saturation * 74., 31. + (1. - value) * 74.),
                4.,
                &paint,
            );
        });
        rect()
            .direction(Direction::Vertical)
            .width(Size::fill())
            .spacing(theme::SPACE_2)
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .spacing(theme::SPACE_1)
                    .child(
                        StudioButton::new(
                            ui,
                            ui.studio_text(if brush_mode { "brush_color" } else { "fill" }),
                        )
                        .text(ui.studio_text(if brush_mode { "brush_color" } else { "fill" }))
                        .selected(fill || brush_mode)
                        .on_press(move |_| target.set(true)),
                    )
                    .maybe_child((!brush_mode).then(|| {
                        StudioButton::new(ui, ui.studio_text("stroke"))
                            .text(ui.studio_text("stroke"))
                            .selected(!fill)
                            .on_press(move |_| target.set(false))
                    })),
            )
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .cross_align(Alignment::Center)
                    .spacing(theme::SPACE_2)
                    .child(
                        rect()
                            .width(Size::px(136.))
                            .height(Size::px(136.))
                            .a11y_id(focus_id)
                            .a11y_focusable(enabled)
                            .a11y_role(AccessibilityRole::Button)
                            .a11y_alt(ui.studio_text("color_settings"))
                            .border(
                                Border::new()
                                    .fill(if focus() == Focus::Keyboard {
                                        ui.accent.read().value
                                    } else {
                                        Color::TRANSPARENT
                                    })
                                    .width(2.)
                                    .alignment(BorderAlignment::Inner),
                            )
                            .on_press(move |event: Event<PressEventData>| {
                                event.stop_propagation();
                                if !enabled {
                                    return;
                                }
                                if let PressEventData::Mouse(mouse) = event.data() {
                                    let point = mouse.element_location;
                                    let dx = point.x as f32 - 68.;
                                    let dy = point.y as f32 - 68.;
                                    let radius = dx.hypot(dy);
                                    let color = if (50.0..=68.0).contains(&radius) {
                                        Some(hsv_to_rgb(
                                            dy.atan2(dx).to_degrees().rem_euclid(360.),
                                            saturation,
                                            value,
                                        ))
                                    } else if (31.0..=105.0).contains(&(point.x as f32))
                                        && (31.0..=105.0).contains(&(point.y as f32))
                                    {
                                        Some(hsv_to_rgb(
                                            hue,
                                            (point.x as f32 - 31.) / 74.,
                                            1. - (point.y as f32 - 31.) / 74.,
                                        ))
                                    } else {
                                        None
                                    };
                                    if let Some(color) = color {
                                        apply_color(&wheel_ui, &rgb_token(color), fill);
                                    }
                                }
                            })
                            .on_key_down({
                                let ui = ui.clone();
                                move |event: Event<KeyboardEventData>| {
                                    let next = match &event.key {
                                        Key::Named(NamedKey::ArrowRight) => Some(hsv_to_rgb(
                                            hue + 5.,
                                            saturation.max(0.1),
                                            value.max(0.1),
                                        )),
                                        Key::Named(NamedKey::ArrowLeft) => Some(hsv_to_rgb(
                                            hue - 5.,
                                            saturation.max(0.1),
                                            value.max(0.1),
                                        )),
                                        Key::Named(NamedKey::ArrowUp) => Some(hsv_to_rgb(
                                            hue,
                                            saturation,
                                            (value + 0.05).min(1.),
                                        )),
                                        Key::Named(NamedKey::ArrowDown) => Some(hsv_to_rgb(
                                            hue,
                                            saturation,
                                            (value - 0.05).max(0.),
                                        )),
                                        _ => None,
                                    };
                                    if enabled {
                                        if let Some(color) = next {
                                            event.prevent_default();
                                            event.stop_propagation();
                                            apply_color(&ui, &rgb_token(color), fill);
                                        }
                                    }
                                }
                            })
                            .child(
                                canvas(on_render)
                                    .key((hue.to_bits(), saturation.to_bits(), value.to_bits()))
                                    .width(Size::fill())
                                    .height(Size::fill()),
                            ),
                    )
                    .child(
                        rect()
                            .direction(Direction::Vertical)
                            .width(Size::flex(1.))
                            .spacing(theme::SPACE_2)
                            .child(
                                rect()
                                    .width(Size::px(36.))
                                    .height(Size::px(30.))
                                    .corner_radius(theme::CONTROL_RADIUS)
                                    .background(Color::from_rgb(
                                        (rgb[0] * 255.) as u8,
                                        (rgb[1] * 255.) as u8,
                                        (rgb[2] * 255.) as u8,
                                    )),
                            )
                            .child(
                                label()
                                    .text(hex)
                                    .font_size(theme::BODY_SIZE)
                                    .color(theme::TEXT_PRIMARY)
                                    .max_lines(1),
                            )
                            .child(
                                label()
                                    .text(format!(
                                        "R {:3}   G {:3}   B {:3}",
                                        (rgb[0] * 255.) as u8,
                                        (rgb[1] * 255.) as u8,
                                        (rgb[2] * 255.) as u8
                                    ))
                                    .font_size(theme::CAPTION_SIZE)
                                    .color(theme::TEXT_SECONDARY),
                            )
                            .child(
                                StudioButton::new(ui, ui.studio_text("color_settings"))
                                    .text(ui.studio_text("color_settings"))
                                    .on_press(move |_| tab.set(2)),
                            ),
                    ),
            )
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .spacing(2.)
                    .children(QUICK_COLORS.into_iter().map(|token| {
                        let apply_ui = ui.clone();
                        let rgb = petunia_design_document::resolve_color_to_rgb(token);
                        rect()
                            .width(Size::flex(1.))
                            .height(Size::px(24.))
                            .corner_radius(2.)
                            .background(Color::from_rgb(
                                (rgb[0] * 255.) as u8,
                                (rgb[1] * 255.) as u8,
                                (rgb[2] * 255.) as u8,
                            ))
                            .child(
                                StudioButton::new(
                                    ui,
                                    format!(
                                        "{} {token}",
                                        ui.studio_text(if brush_mode {
                                            "brush_color"
                                        } else if fill {
                                            "fill"
                                        } else {
                                            "stroke"
                                        })
                                    ),
                                )
                                .width(Size::fill())
                                .height(24.)
                                .enabled(enabled)
                                .on_press(move |_| apply_color(&apply_ui, token, fill)),
                            )
                    })),
            )
    }
}

pub fn rgb_token(rgb: [f32; 3]) -> String {
    let [r, g, b] = rgb.map(|v| (v.clamp(0., 1.) * 255.).round() as u8);
    format!("#{r:02X}{g:02X}{b:02X}")
}
pub fn rgb_to_hsv([r, g, b]: [f32; 3]) -> [f32; 3] {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let hue = if d < 1e-6 {
        0.
    } else if max == r {
        60. * ((g - b) / d).rem_euclid(6.)
    } else if max == g {
        60. * ((b - r) / d + 2.)
    } else {
        60. * ((r - g) / d + 4.)
    };
    [hue, if max == 0. { 0. } else { d / max }, max]
}
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let c = v * s;
    let h = h.rem_euclid(360.) / 60.;
    let x = c * (1. - (h % 2. - 1.).abs());
    let m = v - c;
    let base = match h as u8 {
        0 => [c, x, 0.],
        1 => [x, c, 0.],
        2 => [0., c, x],
        3 => [0., x, c],
        4 => [x, 0., c],
        _ => [c, 0., x],
    };
    base.map(|a| a + m)
}

/// Hex fields accept exactly one sRGB triplet; no partial or suffix input is applied.
pub fn parse_hex(raw: &str) -> Option<[u8; 3]> {
    let raw = raw.trim().strip_prefix('#')?;
    if raw.len() != 6 || !raw.is_ascii() {
        return None;
    }
    Some([
        u8::from_str_radix(&raw[..2], 16).ok()?,
        u8::from_str_radix(&raw[2..4], 16).ok()?,
        u8::from_str_radix(&raw[4..], 16).ok()?,
    ])
}
