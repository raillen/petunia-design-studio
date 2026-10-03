//! File workflow ownership in the desktop adapter. Prompts never create or close
//! a document. Stable session identities keep a save tied to the requested tab.
use freya::prelude::WritableUtils;
use petunia_design_application::session::SessionIdentity;
#[cfg(test)]
use petunia_design_application::{ActionId, ActionRequest};
use petunia_design_foundation::PetuniaError;
use petunia_design_shell::PetuniaShell;
#[cfg(test)]
use std::path::{Path, PathBuf};

use crate::ui_state::UiShell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveIntent {
    Stay,
    CloseOne,
    CloseAll,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfilePurpose {
    Press,
    Monitor,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilePrompt {
    Profile {
        target: SessionIdentity,
        revision: u64,
        surface: petunia_design_foundation::SurfaceId,
        purpose: ProfilePurpose,
        object: Option<petunia_design_foundation::ObjectId>,
    },
    Open,
    Recover,
    Save {
        target: SessionIdentity,
        intent: SaveIntent,
    },
}

#[cfg(test)]
fn path_payload(path: &Path) -> Result<serde_json::Value, PetuniaError> {
    let path = path
        .to_str()
        .ok_or_else(|| PetuniaError::invalid_input("This file workflow requires a UTF-8 path"))?;
    Ok(serde_json::json!({"path":path}))
}

fn missing_target() -> PetuniaError {
    PetuniaError::invalid_input("The document requested by this save is no longer open")
}
fn target_index(shell: &PetuniaShell, target: SessionIdentity) -> Result<usize, PetuniaError> {
    shell
        .bridge
        .sessions()
        .iter()
        .position(|s| s.identity() == target)
        .ok_or_else(missing_target)
}

/// Save a specified tab through the action lane; restore the user's active tab
/// even when serialization or the destination fails.
#[cfg(test)]
pub fn save_target(
    shell: &mut PetuniaShell,
    target: SessionIdentity,
    path: Option<&Path>,
) -> Result<PathBuf, PetuniaError> {
    let payload = path
        .map(path_payload)
        .transpose()?
        .unwrap_or(serde_json::Value::Null);
    let previous = shell.bridge.session().map(|s| s.identity());
    let index = target_index(shell, target)?;
    shell.bridge.switch_session(index)?;
    let result = shell
        .bridge
        .dispatch_action(ActionRequest::new(
            ActionId::new(if path.is_some() {
                "ptnd.action.file.save_as"
            } else {
                "ptnd.action.file.save"
            }),
            payload,
        ))
        .and_then(|_| {
            shell
                .bridge
                .session()
                .and_then(|s| s.path())
                .map(Path::to_path_buf)
                .ok_or_else(missing_target)
        });
    if let Some(previous) = previous {
        if let Ok(index) = target_index(shell, previous) {
            shell.bridge.switch_session(index)?;
        }
    }
    result
}

/// Save every dirty tab before closing any. Cancellation or failure leaves all
/// tabs open; documents already saved remain saved.
#[cfg(test)]
pub fn save_before_close(
    shell: &mut PetuniaShell,
    only: Option<SessionIdentity>,
) -> Result<Option<FilePrompt>, PetuniaError> {
    let targets = shell
        .bridge
        .sessions()
        .iter()
        .filter(|s| only.is_none_or(|id| id == s.identity()) && s.is_dirty())
        .map(|s| (s.identity(), s.path().is_some()))
        .collect::<Vec<_>>();
    for (target, has_path) in targets {
        if !has_path {
            return Ok(Some(FilePrompt::Save {
                target,
                intent: if only.is_some() {
                    SaveIntent::CloseOne
                } else {
                    SaveIntent::CloseAll
                },
            }));
        }
        save_target(shell, target, None)?;
    }
    if let Some(target) = only {
        let index = target_index(shell, target)?;
        if !shell.bridge.close_session_at(index, false)? {
            return Err(missing_target());
        }
    } else if !shell.bridge.close_all_sessions(false)? {
        return Err(missing_target());
    }
    Ok(None)
}

#[cfg(test)]
pub fn complete_prompt(
    shell: &mut PetuniaShell,
    prompt: &FilePrompt,
    path: &Path,
) -> Result<Option<FilePrompt>, PetuniaError> {
    match prompt {
        FilePrompt::Profile { .. } => Err(PetuniaError::invalid_input(
            "ICC admission is an asynchronous file workflow",
        )),
        FilePrompt::Recover => {
            shell.bridge.dispatch_action(ActionRequest::new(
                ActionId::new("ptnd.action.file.recover"),
                path_payload(path)?,
            ))?;
            Ok(None)
        }
        FilePrompt::Open => {
            shell.bridge.dispatch_action(ActionRequest::new(
                ActionId::new("ptnd.action.file.open"),
                path_payload(path)?,
            ))?;
            Ok(None)
        }
        FilePrompt::Save { target, intent } => {
            save_target(shell, *target, Some(path))?;
            match intent {
                SaveIntent::Stay => Ok(None),
                SaveIntent::CloseOne => save_before_close(shell, Some(*target)),
                SaveIntent::CloseAll => save_before_close(shell, None),
            }
        }
    }
}

/// Request a native ICC picker scoped to the current document/revision.
pub fn request_profile(ui: &UiShell, purpose: ProfilePurpose) {
    let shell = ui.shell.peek();
    if let Some(session) = shell.bridge.session() {
        if let Some(surface) = session.active_surface() {
            let object = if purpose == ProfilePurpose::Press
                && session.selection.selected_ids.len() == 1
            {
                session.selection.selected_ids.first().copied().filter(|id| {
                    matches!(
                        session.document().find_object(*id).and_then(|o| o.shape.as_ref()),
                        Some(petunia_design_document::ShapeKind::Raster { layer }) if layer.is_cmyk()
                    )
                })
            } else {
                None
            };
            ui.file_prompt.clone().set(Some(FilePrompt::Profile {
                target: session.identity(),
                revision: session.current_revision(),
                surface,
                purpose,
                object,
            }));
        }
    }
}
pub fn toggle_proof(ui: &UiShell) {
    if *ui.soft_proof.peek() {
        ui.soft_proof.clone().set(false);
        return;
    }
    let native_ink = ui.shell.peek().bridge.session().is_some_and(|session| {
        session
            .active_surface()
            .and_then(|id| session.document().surface(id).ok())
            .is_some_and(petunia_design_application::preview::has_native_ink)
    });
    if native_ink {
        ui.file_error
            .clone()
            .set(Some(ui.text("native_proof_unavailable")));
        return;
    }
    let has_press = ui.shell.peek().bridge.session().is_some_and(|session| {
        session
            .active_surface()
            .and_then(|id| session.document().surface(id).ok())
            .is_some_and(|surface| surface.cmyk_profile.is_some())
    });
    if !has_press {
        request_profile(ui, ProfilePurpose::Press);
        return;
    }
    if ui.monitor_profile.peek().is_none() {
        request_profile(ui, ProfilePurpose::Monitor);
        return;
    }
    ui.soft_proof.clone().set(true);
}

/// New configuration is validated before creating a tab. A domain rejection
/// rolls back the new tab and restores the previously active session.
pub fn create_configured_document(
    shell: &mut PetuniaShell,
    name: &str,
    dimensions: [f64; 2],
    bleed: f64,
    margin: f64,
) -> Result<(), PetuniaError> {
    if name.trim().is_empty()
        || dimensions.iter().any(|v| !v.is_finite() || *v <= 0.)
        || !bleed.is_finite()
        || bleed < 0.
        || !margin.is_finite()
        || margin < 0.
    {
        return Err(PetuniaError::invalid_input(
            "Document name and dimensions must be valid; bleed and margins must be nonnegative",
        ));
    }
    let previous = shell.bridge.session().map(|s| s.identity());
    shell.new_document(name.trim())?;
    let index = shell
        .bridge
        .active_session_index()
        .ok_or_else(missing_target)?;
    let surface = shell.bridge.active_surface().ok_or_else(missing_target)?;
    let result = shell.bridge.submit_all(
        "Configure new document",
        vec![
            petunia_design_application::Command::SetSurfaceGeometry {
                surface,
                origin: [0., 0.],
                dimensions,
            },
            petunia_design_application::Command::SetSurfaceBleed {
                surface,
                bleed: petunia_design_document::Bleed::uniform(bleed),
            },
            petunia_design_application::Command::SetSurfaceMargins {
                surface,
                margins: petunia_design_document::Margins::uniform(margin),
            },
        ],
    );
    if let Err(reason) = result {
        shell.bridge.close_session_at(index, true)?;
        if let Some(previous) = previous {
            if let Ok(index) = target_index(shell, previous) {
                shell.bridge.switch_session(index)?;
            }
        }
        return Err(reason);
    }
    Ok(())
}

/// Capture the tab's identity at the initiating click, before showing a modal.
pub fn request_close(ui: &UiShell, index: usize) {
    let mut shell = ui.shell;
    let failure = ui.text("failed");
    let target = shell
        .peek()
        .bridge
        .sessions()
        .get(index)
        .map(|s| (s.identity(), s.is_dirty()));
    let Some((identity, dirty)) = target else {
        return;
    };
    if dirty {
        let mut target = ui.close_target;
        target.set(Some(identity));
        let mut pending = ui.pending_close;
        pending.set(Some(index));
        let mut open = ui.confirm_close_open;
        open.set(true);
    } else if let Err(reason) = discard_target(&mut shell.write(), Some(identity)) {
        let mut error = ui.file_error;
        error.set(Some(format!("{failure}: {reason}")));
    }
}
pub fn discard_target(
    shell: &mut PetuniaShell,
    target: Option<SessionIdentity>,
) -> Result<bool, PetuniaError> {
    if let Some(target) = target {
        let index = target_index(shell, target)?;
        shell.bridge.close_session_at(index, true)
    } else {
        shell.bridge.close_all_sessions(true)
    }
}

/// Returns true for file actions owned by the adapter, so menu, palette and
/// keyboard use the same lifecycle without dispatching a bare action twice.
pub fn route_file_action(ui: &UiShell, id: &str) -> bool {
    let mut shell = ui.shell;
    let mut prompt = ui.file_prompt;
    let mut error = ui.file_error;
    let failure_label = ui.text("failed");
    let mut error_feedback = error;
    let mut set_error = |reason: PetuniaError| {
        error_feedback.set(Some(format!("{}: {reason}", failure_label)));
    };
    match id {
        "ptnd.action.file.new" => {
            let mut open = ui.new_doc_open;
            open.set(true);
        }
        "ptnd.action.file.recover" => {
            error.set(None);
            prompt.set(Some(FilePrompt::Recover));
        }
        "ptnd.action.file.open" => {
            error.set(None);
            prompt.set(Some(FilePrompt::Open));
        }
        "ptnd.action.file.save" | "ptnd.action.file.save_as" => {
            let target = shell
                .peek()
                .bridge
                .session()
                .map(|s| (s.identity(), s.path().is_some()));
            if let Some((target, has_path)) = target {
                error.set(None);
                if id.ends_with("save_as") || !has_path {
                    prompt.set(Some(FilePrompt::Save {
                        target,
                        intent: SaveIntent::Stay,
                    }));
                } else {
                    if let Err(reason) = crate::file_jobs::save(ui, target, None, SaveIntent::Stay)
                    {
                        set_error(reason);
                    }
                }
            }
        }
        "ptnd.action.file.export" => {
            let mut open = ui.export_open;
            open.set(true);
        }
        "ptnd.action.file.place" => {
            let mut open = ui.place_image_open;
            open.set(true);
        }
        "ptnd.action.file.close" => {
            let index = shell.peek().bridge.active_session_index();
            if let Some(index) = index {
                request_close(ui, index);
            }
        }
        "ptnd.action.file.quit" => {
            let mut state = shell.write();
            if state.bridge.any_session_dirty() {
                let mut target = ui.pending_close;
                target.set(None);
                let mut identity = ui.close_target;
                identity.set(None);
                let mut open = ui.confirm_close_open;
                open.set(true);
            } else if let Err(reason) = state.bridge.close_all_sessions(false) {
                set_error(reason);
            }
        }
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_design_application::Command;
    fn dirty(shell: &mut PetuniaShell) {
        let surface = shell.bridge.active_surface().unwrap();
        shell
            .bridge
            .submit_all(
                "Edit",
                vec![Command::SetSurfaceGeometry {
                    surface,
                    origin: [0., 0.],
                    dimensions: [640., 480.],
                }],
            )
            .unwrap();
    }
    fn temp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("petunia-ui-{}-{name}", std::process::id()))
    }
    #[cfg(unix)]
    #[test]
    fn non_utf8_native_paths_fail_without_panicking_or_changing_the_session() {
        use std::os::unix::ffi::OsStringExt;
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("Original").unwrap();
        let target = shell.bridge.session().unwrap().identity();
        let path = PathBuf::from(std::ffi::OsString::from_vec(vec![
            0xff, b'.', b'P', b'T', b'N', b'D',
        ]));
        assert!(save_target(&mut shell, target, Some(&path)).is_err());
        assert!(complete_prompt(&mut shell, &FilePrompt::Open, &path).is_err());
        assert_eq!(shell.bridge.session().unwrap().identity(), target);
        assert!(shell.bridge.session().unwrap().path().is_none());
    }
    #[test]
    fn invalid_new_configuration_never_creates_a_tab() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("Original").unwrap();
        for dimensions in [[0., 600.], [f64::NAN, 600.], [800., -1.]] {
            assert!(create_configured_document(&mut shell, "New", dimensions, 0., 0.).is_err());
        }
        assert!(create_configured_document(&mut shell, "  ", [800., 600.], 0., 0.).is_err());
        assert_eq!(shell.bridge.sessions().len(), 1);
        assert_eq!(shell.bridge.session().unwrap().title(), "Original");
    }
    #[test]
    fn save_targets_inactive_tab_and_restores_active_tab() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("First").unwrap();
        dirty(&mut shell);
        let first = shell.bridge.session().unwrap().identity();
        shell.new_document("Second").unwrap();
        dirty(&mut shell);
        let second = shell.bridge.session().unwrap().identity();
        let path = temp("inactive.PTND");
        let saved = save_target(&mut shell, first, Some(&path)).unwrap();
        assert_eq!(saved, path);
        assert_eq!(shell.bridge.session().unwrap().identity(), second);
        assert!(!shell.bridge.sessions()[0].is_dirty());
        assert!(shell.bridge.sessions()[1].is_dirty());
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn failed_save_preserves_tabs_dirty_state_and_active_tab() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("First").unwrap();
        dirty(&mut shell);
        let first = shell.bridge.session().unwrap().identity();
        shell.new_document("Second").unwrap();
        let second = shell.bridge.session().unwrap().identity();
        let path = temp("missing-parent").join("doc.PTND");
        assert!(save_target(&mut shell, first, Some(&path)).is_err());
        assert_eq!(shell.bridge.session().unwrap().identity(), second);
        assert!(shell.bridge.sessions()[0].is_dirty());
        assert_eq!(shell.bridge.sessions().len(), 2);
    }
    #[test]
    fn stale_prompt_never_saves_replacement_tab() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("First").unwrap();
        let first = shell.bridge.session().unwrap().identity();
        shell.bridge.close_session_at(0, true).unwrap();
        shell.new_document("Replacement").unwrap();
        dirty(&mut shell);
        let path = temp("stale.PTND");
        assert!(save_target(&mut shell, first, Some(&path)).is_err());
        assert!(!path.exists());
        assert!(shell.bridge.is_dirty());
    }
    #[test]
    fn save_all_waits_for_each_destination_before_closing_any_tab() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("First").unwrap();
        dirty(&mut shell);
        shell.new_document("Second").unwrap();
        dirty(&mut shell);
        let prompt = save_before_close(&mut shell, None).unwrap().unwrap();
        assert_eq!(shell.bridge.sessions().len(), 2);
        let first_path = temp("all-first.PTND");
        let next = complete_prompt(&mut shell, &prompt, &first_path)
            .unwrap()
            .unwrap();
        assert_eq!(shell.bridge.sessions().len(), 2);
        let second_path = temp("all-second.PTND");
        assert!(complete_prompt(&mut shell, &next, &second_path)
            .unwrap()
            .is_none());
        assert!(shell.bridge.sessions().is_empty());
        std::fs::remove_file(first_path).unwrap();
        std::fs::remove_file(second_path).unwrap();
    }
}
