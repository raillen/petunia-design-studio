//! Modal file destination adapter. Native pickers run asynchronously and only
//! fill the path field; publication still requires the dialog's explicit action.
use crate::file_workflows::FilePrompt;
use crate::theme;
use crate::ui_state::UiShell;
use freya::prelude::*;
use std::path::PathBuf;

#[derive(Clone, PartialEq)]
pub struct FileDialog(pub UiShell);
impl Component for FileDialog {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let mut path = use_state(String::new);
        let mut selected = use_state(|| None::<(String, PathBuf)>);
        let mut replace = use_state(|| None::<PathBuf>);
        let mut picker_busy = use_state(|| false);
        let mut generation = use_state(|| 0u64);
        let mut prompt = ui.file_prompt;
        let mut error = ui.file_error;
        let shell = ui.shell;
        use_side_effect(move || {
            let request = prompt.read().clone();
            let next_generation = generation.peek().wrapping_add(1);
            generation.set(next_generation);
            picker_busy.set(false);
            selected.set(None);
            replace.set(None);
            error.set(None);
            path.set(match request {
                Some(FilePrompt::Save { target, .. }) => shell
                    .peek()
                    .bridge
                    .sessions()
                    .iter()
                    .find(|s| s.identity() == target)
                    .map(|s| {
                        s.path().map_or_else(
                            || {
                                petunia_design_io::with_native_extension(std::path::Path::new(
                                    s.title(),
                                ))
                                .to_string_lossy()
                                .into_owned()
                            },
                            |p| p.to_string_lossy().into_owned(),
                        )
                    })
                    .unwrap_or_default(),
                _ => String::new(),
            });
        });
        let Some(request) = prompt.read().clone() else {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        };
        let saving = matches!(request, FilePrompt::Save { .. });
        let busy = *picker_busy.read() || ui.file_job.read().is_some();
        let title = ui.text(if matches!(request, FilePrompt::Recover) {
            "recover_title"
        } else if matches!(
            request,
            FilePrompt::Profile {
                purpose: crate::file_workflows::ProfilePurpose::Press,
                ..
            }
        ) {
            "icc_press"
        } else if matches!(request, FilePrompt::Profile { .. }) {
            "icc_monitor"
        } else if saving {
            "save"
        } else {
            "open"
        });
        let current_path = path.read().clone();
        let target = selected
            .peek()
            .as_ref()
            .filter(|(display, _)| display == &current_path)
            .map_or_else(|| PathBuf::from(current_path.trim()), |(_, p)| p.clone());
        let target = if saving {
            petunia_design_io::with_native_extension(&target)
        } else {
            target
        };
        let confirmed = replace.read().as_ref() == Some(&target);
        let error_text = error.read().clone();
        let replacement = replace.read().is_some() && confirmed;
        let failure = ui.text("failed");
        let empty = ui.text("empty_path");
        let browse_ui = ui.clone();
        let submit_ui = ui.clone();
        let mut dismiss_prompt = prompt;
        let mut dismiss_error = error;
        rect().child(
            Popup::new()
                .width(Size::px(560.))
                .max_width(Size::window_percent(96.))
                .on_close_request(move |_| {
                    if busy {
                        return;
                    }
                    dismiss_prompt.set(None);
                    dismiss_error.set(None);
                })
                .child(PopupTitle::new(title.clone()))
                .child(
                    PopupContent::new().child(
                        rect()
                            .direction(Direction::Vertical)
                            .width(Size::fill())
                            .spacing(theme::SPACE_2)
                            .child(
                                label()
                                    .text(ui.text("path_hint"))
                                    .color(theme::TEXT_SECONDARY),
                            )
                            .child(
                                Input::new(path)
                                    .width(Size::fill())
                                    .placeholder(ui.text("destination")),
                            )
                            .maybe_child((saving && !current_path.trim().is_empty()).then(|| {
                                label()
                                    .text(target.display().to_string())
                                    .color(theme::TEXT_SECONDARY)
                            }))
                            .child(
                                Button::new()
                                    .enabled(!busy)
                                    .on_press(move |_| {
                                        picker_busy.set(true);
                                        let ticket = *generation.peek();
                                        let expected = request.clone();
                                        let caption = title.clone();
                                        let suggestion = path.peek().clone();
                                        let browse_ui = browse_ui.clone();
                                        spawn(async move {
                                            let mut dialog =
                                                rfd::AsyncFileDialog::new().set_title(caption);
                                            if matches!(expected, FilePrompt::Profile { .. }) {
                                                dialog = dialog.add_filter(
                                                    "ICC",
                                                    &["icc", "icm", "ICC", "ICM"],
                                                );
                                            } else {
                                                dialog = dialog.add_filter(
                                                    "Petunia",
                                                    &["PTND", "ptnd", "aubrieta", "aubri"],
                                                );
                                                if !saving
                                                    && !matches!(expected, FilePrompt::Recover)
                                                {
                                                    dialog = dialog.add_filter("SVG", &["svg"]);
                                                }
                                            }
                                            if saving {
                                                if let Some(name) =
                                                    std::path::Path::new(&suggestion).file_name()
                                                {
                                                    dialog = dialog
                                                        .set_file_name(name.to_string_lossy());
                                                }
                                            }
                                            if matches!(expected, FilePrompt::Recover) {
                                                if let Some(directory) =
                                                    crate::recovery::default_directory()
                                                {
                                                    dialog = dialog.set_directory(directory);
                                                }
                                            }
                                            let result = if saving {
                                                dialog.save_file().await
                                            } else {
                                                dialog.pick_file().await
                                            };
                                            if *generation.peek() != ticket
                                                || browse_ui.file_prompt.peek().as_ref()
                                                    != Some(&expected)
                                            {
                                                return;
                                            }
                                            picker_busy.set(false);
                                            if let Some(file) = result {
                                                let chosen = file.path().to_path_buf();
                                                let display = chosen.to_string_lossy().into_owned();
                                                path.set(display.clone());
                                                selected.set(Some((display, chosen)));
                                                replace.set(None);
                                                error.set(None);
                                            }
                                        });
                                    })
                                    .child(ui.text("browse")),
                            )
                            .child(
                                label()
                                    .text(ui.text("picker_hint"))
                                    .color(theme::TEXT_SECONDARY),
                            )
                            .children(
                                error_text
                                    .into_iter()
                                    .map(|reason| label().text(reason).color(theme::TEXT_ERROR)),
                            )
                            .maybe_child(replacement.then(|| {
                                label()
                                    .text(format!(
                                        "{}\n{}",
                                        ui.text("overwrite_reason"),
                                        target.display()
                                    ))
                                    .color(theme::TEXT_PRIMARY)
                            }))
                            .child(
                                rect()
                                    .direction(Direction::Horizontal)
                                    .width(Size::fill())
                                    .main_align(Alignment::End)
                                    .spacing(theme::SPACE_1)
                                    .child(
                                        Button::new()
                                            .enabled(!busy)
                                            .on_press(move |_| {
                                                prompt.set(None);
                                                error.set(None);
                                            })
                                            .child(ui.text("cancel")),
                                    )
                                    .child(
                                        Button::new()
                                            .enabled(!busy)
                                            .on_press(move |_| {
                                                if current_path.trim().is_empty() {
                                                    error.set(Some(empty.clone()));
                                                    return;
                                                }
                                                if saving && target.exists() && !confirmed {
                                                    replace.set(Some(target.clone()));
                                                    return;
                                                }
                                                let Some(request) =
                                                    submit_ui.file_prompt.peek().clone()
                                                else {
                                                    return;
                                                };
                                                let result = crate::file_jobs::prompt(
                                                    &submit_ui, &request, &target,
                                                );
                                                match result {
                                                    Ok(()) => {
                                                        error.set(None);
                                                        replace.set(None);
                                                    }
                                                    Err(reason) => error
                                                        .set(Some(format!("{failure}: {reason}"))),
                                                }
                                            })
                                            .child(ui.text(if confirmed {
                                                "overwrite"
                                            } else if saving {
                                                "save"
                                            } else {
                                                "open"
                                            })),
                                    ),
                            ),
                    ),
                ),
        )
    }
}
