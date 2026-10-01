//! GUI owns nonblocking recovery polling; the application worker owns all I/O.
use crate::ui_state::UiShell;
use freya::prelude::*;
use petunia_design_application::recovery::RecoveryController;
use petunia_design_jobs::JobFailure;
use std::{cell::RefCell, path::PathBuf, rc::Rc, time::Duration};

pub fn default_directory() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("PETUNIA_RECOVERY_DIR")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
    {
        return Some(path);
    }
    // GUI fixtures must never create/clean the developer's personal backups.
    if cfg!(test) {
        return None;
    }
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
                .map(|path| path.join(".local/state"))
        })
        .map(|path| path.join("petunia-design/recovery"))
}
pub fn use_recovery(ui: &UiShell) {
    let owner = use_hook(|| {
        Rc::new(RefCell::new(
            default_directory().map(RecoveryController::new),
        ))
    });
    let startup_ui = ui.clone();
    let shell = ui.shell;
    let mut error = ui.file_error;
    use_future(move || {
        let owner = owner.clone();
        let startup_ui = startup_ui.clone();
        async move {
            if let Some(directory) = default_directory() {
                if let Err(failure) = crate::file_jobs::list_recovery(&startup_ui, directory) {
                    error.set(Some(failure.to_string()));
                }
            }
            loop {
                timer(Duration::from_millis(500)).await;
                let failure = {
                    let mut owner = owner.borrow_mut();
                    match owner.as_mut() {
                        Some(Ok(controller)) => {
                            let shell = shell.peek();
                            controller.tick(&shell.bridge.sessions()).err()
                        }
                        Some(Err(failure)) => Some(failure.clone()),
                        None => None,
                    }
                };
                if let Some(failure) = failure {
                    if failure != JobFailure::QueueFull {
                        let message = format!("Recovery: {failure}");
                        if error.peek().as_ref() != Some(&message) {
                            error.set(Some(message));
                        }
                    }
                }
            }
        }
    });
}

#[derive(Clone, PartialEq)]
pub struct RecoveryDialog(pub UiShell);
impl Component for RecoveryDialog {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        if !*ui.recovery_open.read() {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let mut open = ui.recovery_open;
        let busy = ui.file_job.read().is_some();
        let entries = ui.recovery_entries.read().clone();
        rect().child(
            Popup::new()
                .width(Size::px(600.))
                .max_width(Size::window_percent(96.))
                .on_close_request(move |_| {
                    if !busy {
                        open.set(false);
                    }
                })
                .child(PopupTitle::new(ui.text("recover_title")))
                .child(
                    PopupContent::new().child(
                        rect()
                            .direction(Direction::Vertical)
                            .spacing(crate::theme::SPACE_2)
                            .child(label().text(ui.text("recovery_available")))
                            .child(
                                ScrollView::new().height(Size::px(260.)).child(
                                    rect()
                                        .direction(Direction::Vertical)
                                        .spacing(crate::theme::SPACE_1)
                                        .children(entries.into_iter().map(|entry| {
                                            let restore_ui = ui.clone();
                                            let path = entry.key.path().to_path_buf();
                                            let source = entry
                                                .metadata
                                                .original_path
                                                .as_ref()
                                                .map_or_else(String::new, |path| {
                                                    path.display().to_string()
                                                });
                                            Button::new()
                                                .enabled(!busy)
                                                .on_press(move |_| {
                                                    if let Err(error) = crate::file_jobs::prompt(
                                                        &restore_ui,
                                                        &crate::file_workflows::FilePrompt::Recover,
                                                        &path,
                                                    ) {
                                                        restore_ui
                                                            .file_error
                                                            .clone()
                                                            .set(Some(error.to_string()));
                                                    }
                                                })
                                                .child(format!(
                                                    "{} · r{} · {}\n{}",
                                                    entry.metadata.title,
                                                    entry.metadata.revision,
                                                    entry.metadata.captured_unix_seconds,
                                                    source
                                                ))
                                        })),
                                ),
                            )
                            .children(
                                ui.file_error.read().clone().into_iter().map(|error| {
                                    label().text(error).color(crate::theme::TEXT_ERROR)
                                }),
                            )
                            .child(
                                Button::new()
                                    .enabled(!busy)
                                    .on_press(move |_| open.set(false))
                                    .child(ui.text("recovery_continue")),
                            ),
                    ),
                ),
        )
    }
}
