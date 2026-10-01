//! Explicit, cancellable drafts for text, layer names and placement.
use crate::{
    object_edits::{EditFailure, EditKind, EditValue, ObjectEdit},
    theme,
    ui_state::UiShell,
};
use freya::prelude::*;
use petunia_design_foundation::ObjectId;

pub fn failure_text(ui: &UiShell, failure: EditFailure) -> String {
    match failure {
        EditFailure::TargetChanged => ui.text("edit_target_changed"),
        EditFailure::PropertyChanged => ui.text("edit_property_changed"),
        EditFailure::InvalidName => ui.text("edit_invalid_name"),
        EditFailure::Number { field, reason } => {
            let message = if *ui.shell.peek().bridge.locale() == petunia_design_shell::Locale::PtBr
            {
                reason.message_pt_br()
            } else {
                reason.message_en_us()
            };
            format!("{field}: {message}")
        }
        EditFailure::Command(reason) => format!("{}: {reason}", ui.text("failed")),
    }
}

pub fn request(ui: &UiShell, object: ObjectId, kind: EditKind) {
    match ObjectEdit::capture(&ui.shell.peek(), object, kind) {
        Ok(edit) => {
            let mut prompt = ui.object_edit;
            prompt.set(Some(edit));
        }
        Err(reason) => ui.file_error.clone().set(Some(failure_text(ui, reason))),
    }
}

#[derive(Clone, PartialEq)]
pub struct ObjectEditDialog(pub UiShell);
impl Component for ObjectEditDialog {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let mut prompt = ui.object_edit;
        let mut content = use_state(String::new);
        let mut x = use_state(String::new);
        let mut y = use_state(String::new);
        let mut width = use_state(String::new);
        let mut height = use_state(String::new);
        let mut rotation = use_state(String::new);
        let mut error = use_state(|| None::<String>);
        use_side_effect(move || {
            let current = prompt.read().clone();
            error.set(None);
            match current.map(|draft| draft.original) {
                Some(EditValue::Name(name)) => content.set(name),
                Some(EditValue::Text(petunia_design_document::ShapeKind::Text {
                    content: value,
                    ..
                })) => content.set(value),
                Some(EditValue::Transform {
                    bounds,
                    rotation: radians,
                }) => {
                    x.set(bounds[0].to_string());
                    y.set(bounds[1].to_string());
                    width.set(bounds[2].to_string());
                    height.set(bounds[3].to_string());
                    rotation.set(radians.to_degrees().to_string());
                }
                _ => content.set(String::new()),
            }
        });
        let Some(edit) = prompt.read().clone() else {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        };
        let is_text = matches!(edit.original, EditValue::Text(_));
        let is_transform = matches!(edit.original, EditValue::Transform { .. });
        let title = ui.text(if is_transform {
            "edit_transform"
        } else if is_text {
            "edit_text"
        } else {
            "edit_name"
        });
        let hint = ui.text(if is_transform {
            "edit_frame_hint"
        } else if is_text {
            "edit_text_hint"
        } else {
            "edit_name_hint"
        });
        let reason = error.read().clone();
        let submit_ui = ui.clone();
        let mut dismiss = prompt;
        rect().child(
            Popup::new()
                .width(Size::px(520.))
                .max_width(Size::window_percent(96.))
                .on_close_request(move |_| dismiss.set(None))
                .child(PopupTitle::new(title))
                .child(
                    PopupContent::new().child(
                        rect()
                            .direction(Direction::Vertical)
                            .width(Size::fill())
                            .spacing(theme::SPACE_2)
                            .child(label().text(hint).color(theme::TEXT_SECONDARY))
                            .maybe(!is_transform, |el| {
                                el.child(
                                    Input::new(content)
                                        .on_validate(move |_| error.set(None))
                                        .width(Size::fill())
                                        .auto_focus(true)
                                        .multiline(is_text)
                                        .height(if is_text {
                                            Size::px(140.)
                                        } else {
                                            Size::px(38.)
                                        }),
                                )
                            })
                            .maybe(is_transform, |el| {
                                el.children(
                                    [
                                        ("X (pt)", x),
                                        ("Y (pt)", y),
                                        ("W (pt)", width),
                                        ("H (pt)", height),
                                        ("°", rotation),
                                    ]
                                    .into_iter()
                                    .map(|(tag, state)| {
                                        rect()
                                            .direction(Direction::Horizontal)
                                            .width(Size::fill())
                                            .cross_align(Alignment::Center)
                                            .spacing(theme::SPACE_2)
                                            .child(
                                                label()
                                                    .text(tag)
                                                    .width(Size::px(70.))
                                                    .color(theme::TEXT_PRIMARY),
                                            )
                                            .child(Input::new(state).width(Size::flex(1.)))
                                    }),
                                )
                            })
                            .children(
                                reason
                                    .into_iter()
                                    .map(|text| label().text(text).color(theme::TEXT_ERROR)),
                            )
                            .child(
                                rect()
                                    .direction(Direction::Horizontal)
                                    .width(Size::fill())
                                    .main_align(Alignment::End)
                                    .spacing(theme::SPACE_2)
                                    .child(
                                        Button::new()
                                            .on_press(move |_| prompt.set(None))
                                            .child(ui.text("cancel")),
                                    )
                                    .child(
                                        Button::new()
                                            .on_press(move |_| {
                                                // Read current states on the gesture, not values captured by a prior render.
                                                let Some(draft) =
                                                    submit_ui.object_edit.peek().clone()
                                                else {
                                                    return;
                                                };
                                                let fields = [
                                                    x.peek().clone(),
                                                    y.peek().clone(),
                                                    width.peek().clone(),
                                                    height.peek().clone(),
                                                    rotation.peek().clone(),
                                                ];
                                                let result = draft.apply(
                                                    &mut submit_ui.shell.clone().write(),
                                                    &content.peek(),
                                                    std::array::from_fn(|i| fields[i].as_str()),
                                                );
                                                match result {
                                                    Ok(changed) => {
                                                        prompt.set(None);
                                                        if changed {
                                                            submit_ui.file_notice.clone().set(
                                                                Some(
                                                                    submit_ui.text("edit_applied"),
                                                                ),
                                                            );
                                                        }
                                                    }
                                                    Err(reason) => error.set(Some(failure_text(
                                                        &submit_ui, reason,
                                                    ))),
                                                }
                                            })
                                            .child(ui.text("edit_apply")),
                                    ),
                            ),
                    ),
                ),
        )
    }
}
