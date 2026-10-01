//! Uniform basic typography drafts with stable target and property conflict checks.
use crate::{theme, ui_state::UiShell};
use freya::prelude::*;
use petunia_design_application::{session::SessionIdentity, Command};
use petunia_design_document::{ShapeKind, TextAlignment, TextFlow, TextStyle};
use petunia_design_foundation::ObjectId;
use petunia_design_shell::PetuniaShell;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Draft {
    session: SessionIdentity,
    object: ObjectId,
    shape: ShapeKind,
    style: TextStyle,
}
impl Draft {
    fn capture(shell: &PetuniaShell, object: ObjectId) -> Option<Self> {
        let session = shell.bridge.session()?;
        let obj = session.find_object(object)?;
        let shape = obj.shape.as_ref()?;
        if !matches!(shape, ShapeKind::Text { .. }) {
            return None;
        }
        Some(Self {
            session: session.identity(),
            object,
            shape: shape.clone(),
            style: obj.text_style,
        })
    }
    fn fields(&self) -> [String; 5] {
        let ShapeKind::Text {
            font_family,
            font_size,
            line_height,
            letter_spacing,
            ..
        } = &self.shape
        else {
            unreachable!()
        };
        [
            font_family.clone(),
            font_size.to_string(),
            self.style.weight.to_string(),
            line_height.to_string(),
            letter_spacing.to_string(),
        ]
    }
    fn apply(
        &self,
        shell: &mut PetuniaShell,
        fields: [&str; 5],
        style: TextStyle,
    ) -> Result<bool, &'static str> {
        let Some(current) = Self::capture(shell, self.object) else {
            return Err("edit_target_changed");
        };
        if current.session != self.session {
            return Err("edit_target_changed");
        }
        if current != *self {
            return Err("edit_property_changed");
        }
        let mut shape = self.shape.clone();
        let ShapeKind::Text {
            font_family,
            font_size,
            line_height,
            letter_spacing,
            ..
        } = &mut shape
        else {
            unreachable!()
        };
        let family = fields[0].trim();
        if family.is_empty() || family.len() > 1024 || family.chars().any(char::is_control) {
            return Err("type_invalid");
        }
        let number = |value: &str| {
            value
                .trim()
                .replace(',', ".")
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
        };
        let size = number(fields[1])
            .filter(|v| (0.1..=10_000.).contains(v))
            .ok_or("type_invalid")?;
        let weight = fields[2]
            .trim()
            .parse::<u16>()
            .ok()
            .filter(|v| (1..=1000).contains(v))
            .ok_or("type_invalid")?;
        let leading = number(fields[3])
            .filter(|v| (0.1..=100.).contains(v))
            .ok_or("type_invalid")?;
        let tracking = number(fields[4])
            .filter(|v| v.abs() <= 10_000.)
            .ok_or("type_invalid")?;
        *font_family = family.to_owned();
        *font_size = size;
        *line_height = leading;
        *letter_spacing = tracking;
        let style = TextStyle { weight, ..style };
        if shape == self.shape && style == self.style {
            return Ok(false);
        }
        shell
            .bridge
            .submit_all(
                "Set typography",
                vec![
                    Command::SetShape {
                        id: self.object,
                        shape: Some(shape),
                    },
                    Command::SetTextStyle {
                        id: self.object,
                        style,
                    },
                ],
            )
            .map_err(|_| "type_failed")?;
        Ok(true)
    }
}

#[derive(Clone, PartialEq)]
pub struct TypographyControl(pub UiShell, pub ObjectId);
impl Component for TypographyControl {
    fn render(&self) -> impl IntoElement {
        let ui = self.0.clone();
        let object = self.1;
        let label = ui.text("type_edit");
        Button::new()
            .on_press(move |_| {
                ui.typography_edit
                    .clone()
                    .set(Draft::capture(&ui.shell.peek(), object));
            })
            .child(label)
    }
}
#[derive(Clone, PartialEq)]
pub struct TypographyDialog(pub UiShell);
impl Component for TypographyDialog {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let mut draft = ui.typography_edit;
        let mut family = use_state(String::new);
        let mut size = use_state(String::new);
        let mut weight = use_state(String::new);
        let mut leading = use_state(String::new);
        let mut tracking = use_state(String::new);
        let mut style = use_state(TextStyle::default);
        let mut error = use_state(|| None::<String>);
        use_side_effect(move || {
            if let Some(value) = draft.read().as_ref() {
                let fields = value.fields();
                family.set(fields[0].clone());
                size.set(fields[1].clone());
                weight.set(fields[2].clone());
                leading.set(fields[3].clone());
                tracking.set(fields[4].clone());
                style.set(value.style);
            }
            error.set(None);
        });
        let submit_ui = ui.clone();
        let current_style = *style.read();
        let popup = draft.read().clone();
        let reason = error.read().clone();
        rect()
            .direction(Direction::Vertical)
            .width(Size::fill())
            .maybe(popup.is_some(), |el| {
                el.child(
                    Popup::new()
                        .width(Size::px(520.))
                        .max_width(Size::window_percent(96.))
                        .on_close_request(move |_| draft.set(None))
                        .child(PopupTitle::new(ui.text("type_edit")))
                        .child(
                            PopupContent::new().child(
                                rect()
                                    .direction(Direction::Vertical)
                                    .width(Size::fill())
                                    .spacing(theme::SPACE_2)
                                    .children(
                                        [
                                            ("type_family", family),
                                            ("type_size", size),
                                            ("type_weight", weight),
                                            ("type_leading", leading),
                                            ("type_tracking", tracking),
                                        ]
                                        .into_iter()
                                        .map(
                                            |(key, state)| {
                                                rect()
                                                    .direction(Direction::Horizontal)
                                                    .width(Size::fill())
                                                    .spacing(theme::SPACE_2)
                                                    .child(
                                                        label()
                                                            .text(ui.text(key))
                                                            .width(Size::px(130.))
                                                            .color(theme::TEXT_PRIMARY),
                                                    )
                                                    .child(Input::new(state).width(Size::flex(1.)))
                                            },
                                        ),
                                    )
                                    .child(
                                        rect()
                                            .direction(Direction::Horizontal)
                                            .spacing(theme::SPACE_2)
                                            .child(
                                                Button::new()
                                                    .on_press(move |_| {
                                                        let mut value = *style.peek();
                                                        value.italic = !value.italic;
                                                        style.set(value);
                                                    })
                                                    .child(ui.text(if current_style.italic {
                                                        "type_italic_on"
                                                    } else {
                                                        "type_italic_off"
                                                    })),
                                            )
                                            .child(
                                                Button::new()
                                                    .on_press(move |_| {
                                                        let mut value = *style.peek();
                                                        value.flow =
                                                            if value.flow == TextFlow::Frame {
                                                                TextFlow::Artistic
                                                            } else {
                                                                TextFlow::Frame
                                                            };
                                                        style.set(value);
                                                    })
                                                    .child(ui.text(
                                                        if current_style.flow == TextFlow::Frame {
                                                            "type_frame"
                                                        } else {
                                                            "type_artistic"
                                                        },
                                                    )),
                                            ),
                                    )
                                    .child(
                                        rect()
                                            .direction(Direction::Horizontal)
                                            .spacing(theme::SPACE_2)
                                            .children(
                                                [
                                                    (TextAlignment::Start, "type_start"),
                                                    (TextAlignment::Center, "type_center"),
                                                    (TextAlignment::End, "type_end"),
                                                ]
                                                .into_iter()
                                                .map(|(align, key)| {
                                                    let text = format!(
                                                        "{}{}",
                                                        if current_style.alignment == align {
                                                            "✓ "
                                                        } else {
                                                            ""
                                                        },
                                                        ui.text(key)
                                                    );
                                                    Button::new()
                                                        .on_press(move |_| {
                                                            let mut value = *style.peek();
                                                            value.alignment = align;
                                                            style.set(value);
                                                        })
                                                        .child(text)
                                                }),
                                            ),
                                    )
                                    .child(
                                        label()
                                            .text(ui.text("type_limits"))
                                            .color(theme::TEXT_SECONDARY),
                                    )
                                    .children(reason.into_iter().map(|reason| {
                                        label().text(reason).color(theme::TEXT_ERROR)
                                    }))
                                    .child(
                                        rect()
                                            .direction(Direction::Horizontal)
                                            .spacing(theme::SPACE_2)
                                            .main_align(Alignment::End)
                                            .child(
                                                Button::new()
                                                    .on_press(move |_| draft.set(None))
                                                    .child(ui.text("cancel")),
                                            )
                                            .child(
                                                Button::new()
                                                    .on_press(move |_| {
                                                        let Some(target) = draft.peek().clone()
                                                        else {
                                                            return;
                                                        };
                                                        let fields = [
                                                            family.peek().clone(),
                                                            size.peek().clone(),
                                                            weight.peek().clone(),
                                                            leading.peek().clone(),
                                                            tracking.peek().clone(),
                                                        ];
                                                        let result = target.apply(
                                                            &mut submit_ui.shell.clone().write(),
                                                            std::array::from_fn(|i| {
                                                                fields[i].as_str()
                                                            }),
                                                            *style.peek(),
                                                        );
                                                        match result {
                                                            Ok(_) => draft.set(None),
                                                            Err(key) => {
                                                                error.set(Some(submit_ui.text(key)))
                                                            }
                                                        }
                                                    })
                                                    .child(ui.text("edit_apply")),
                                            ),
                                    ),
                            ),
                        ),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typography_preserves_content_and_replays_as_one_transaction() {
        let (mut shell, id, _) = crate::object_edits::tests::fixture();
        let before = Draft::capture(&shell, id).unwrap();
        let mut style = before.style;
        style.italic = true;
        style.alignment = TextAlignment::Center;
        assert!(before
            .apply(
                &mut shell,
                ["DejaVu Sans", "32", "700", "1.4", "1.5"],
                style
            )
            .unwrap());
        let after = Draft::capture(&shell, id).unwrap();
        let ShapeKind::Text {
            content, on_path, ..
        } = &after.shape
        else {
            panic!()
        };
        assert_eq!(content, "First\nSecond");
        assert!(on_path.is_some());
        assert_eq!(after.style.weight, 700);
        shell.bridge.undo().unwrap();
        assert_eq!(Draft::capture(&shell, id).unwrap(), before);
        shell.bridge.redo().unwrap();
        assert_eq!(Draft::capture(&shell, id).unwrap(), after);
    }
    #[test]
    fn invalid_or_conflicting_typography_does_not_publish() {
        let (mut shell, id, _) = crate::object_edits::tests::fixture();
        let before = Draft::capture(&shell, id).unwrap();
        assert!(before
            .apply(&mut shell, ["font", "NaN", "400", "1.2", "0"], before.style)
            .is_err());
        assert_eq!(Draft::capture(&shell, id).unwrap(), before);
        let edit = crate::object_edits::ObjectEdit::capture(
            &shell,
            id,
            crate::object_edits::EditKind::Text,
        )
        .unwrap();
        crate::object_edits::step_font_size(&mut shell, &edit, 2.).unwrap();
        assert_eq!(
            before.apply(&mut shell, ["font", "24", "400", "1.2", "0"], before.style),
            Err("edit_property_changed")
        );
    }
}
