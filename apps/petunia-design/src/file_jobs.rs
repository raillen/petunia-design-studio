//! Desktop file I/O and cold codec work run outside the UI thread. One accepted
//! operation owns a bounded worker slot; publication keeps its captured tab.
use crate::{
    file_workflows::{FilePrompt, ProfilePurpose, SaveIntent},
    ui_state::UiShell,
};
use freya::prelude::*;
use petunia_design_application::{
    export_service::{export_document_cancellable, ExportRequest},
    session::SessionIdentity,
};
use petunia_design_document::Document;
use petunia_design_foundation::{ObjectId, PetuniaError, SurfaceId};
use petunia_design_jobs::{JobExecutor, JobFailure, JobManager};
use std::{
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

#[derive(Clone)]
pub struct Request {
    token: u64,
    operation: Operation,
}
impl PartialEq for Request {
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
    }
}
#[derive(Clone)]
struct ClipboardTarget {
    session: SessionIdentity,
    revision: u64,
    surface: SurfaceId,
}
#[derive(Clone)]
enum Operation {
    Profile {
        target: ClipboardTarget,
        path: PathBuf,
        purpose: ProfilePurpose,
    },
    ListRecovery {
        directory: PathBuf,
    },
    ClipboardWrite {
        target: ClipboardTarget,
        frame: [f64; 4],
        objects: Vec<petunia_design_document::DocumentObject>,
        cut: Option<Vec<ObjectId>>,
    },
    ClipboardRead {
        target: ClipboardTarget,
    },
    Open {
        path: PathBuf,
        recovery: bool,
    },
    Save {
        target: SessionIdentity,
        revision: u64,
        state: u64,
        document: Document,
        path: PathBuf,
        intent: SaveIntent,
    },
    Place {
        target: SessionIdentity,
        revision: u64,
        surface: SurfaceId,
        path: PathBuf,
    },
    Export {
        document: Document,
        request: ExportRequest,
    },
}
enum Outcome {
    Profile {
        target: ClipboardTarget,
        path: PathBuf,
        purpose: ProfilePurpose,
        profile: petunia_design_color::IccProfile,
    },
    RecoveryList {
        entries: Vec<petunia_design_io::recovery::RecoveryEntry>,
    },
    Clipboard {
        target: ClipboardTarget,
        objects: Vec<petunia_design_document::DocumentObject>,
        origin: [f64; 2],
        cut: Option<Vec<ObjectId>>,
        paste: bool,
    },
    Open {
        path: PathBuf,
        opened: petunia_design_io::OpenedPackage,
        recovery: bool,
    },
    Svg {
        path: PathBuf,
        document: Document,
    },
    Save {
        target: SessionIdentity,
        revision: u64,
        state: u64,
        path: PathBuf,
        intent: SaveIntent,
    },
    Place {
        target: SessionIdentity,
        revision: u64,
        surface: SurfaceId,
        path: PathBuf,
        source: Arc<petunia_design_raster::EncodedImage>,
        size: [u32; 2],
    },
    Export {
        path: PathBuf,
    },
}
fn invalid(reason: &str) -> PetuniaError {
    PetuniaError::invalid_input(reason)
}
fn enqueue(ui: &UiShell, operation: Operation) -> Result<(), PetuniaError> {
    if ui.file_job.peek().is_some() {
        return Err(invalid("a file operation is already running"));
    }
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let token = NEXT
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
        .map_err(|_| invalid("file operation identity exhausted"))?;
    ui.file_error.clone().set(None);
    ui.file_notice.clone().set(Some(ui.text("file_working")));
    ui.file_job.clone().set(Some(Request { token, operation }));
    Ok(())
}
pub fn list_recovery(ui: &UiShell, directory: PathBuf) -> Result<(), PetuniaError> {
    enqueue(ui, Operation::ListRecovery { directory })
}
pub fn prompt(ui: &UiShell, prompt: &FilePrompt, path: &Path) -> Result<(), PetuniaError> {
    if path.as_os_str().is_empty() {
        return Err(invalid("choose a file path"));
    }
    match prompt {
        FilePrompt::Profile {
            target,
            revision,
            surface,
            purpose,
        } => enqueue(
            ui,
            Operation::Profile {
                target: ClipboardTarget {
                    session: *target,
                    revision: *revision,
                    surface: *surface,
                },
                path: path.to_path_buf(),
                purpose: *purpose,
            },
        ),
        FilePrompt::Open => enqueue(
            ui,
            Operation::Open {
                path: path.to_path_buf(),
                recovery: false,
            },
        ),
        FilePrompt::Recover => enqueue(
            ui,
            Operation::Open {
                path: path.to_path_buf(),
                recovery: true,
            },
        ),
        FilePrompt::Save { target, intent } => save(ui, *target, Some(path), *intent),
    }
}
pub fn save(
    ui: &UiShell,
    target: SessionIdentity,
    path: Option<&Path>,
    intent: SaveIntent,
) -> Result<(), PetuniaError> {
    let shell = ui.shell.peek();
    let sessions = shell.bridge.sessions();
    let session = sessions
        .into_iter()
        .find(|s| s.identity() == target)
        .ok_or_else(|| invalid("save target is no longer open"))?;
    let path = path
        .or_else(|| session.path())
        .ok_or_else(|| invalid("save requires a destination"))?;
    let operation = Operation::Save {
        target,
        revision: session.current_revision(),
        state: session.history().state_id(),
        document: session.document().clone(),
        path: petunia_design_io::with_native_extension(path),
        intent,
    };
    drop(shell);
    enqueue(ui, operation)
}
pub fn place(ui: &UiShell, path: &Path) -> Result<(), PetuniaError> {
    if path.as_os_str().is_empty() {
        return Err(invalid("choose an image path"));
    }
    let shell = ui.shell.peek();
    let session = shell
        .bridge
        .session()
        .ok_or_else(|| invalid("no image target document"))?;
    let operation = Operation::Place {
        target: session.identity(),
        revision: session.current_revision(),
        surface: session
            .active_surface()
            .ok_or_else(|| invalid("no image target surface"))?,
        path: path.to_path_buf(),
    };
    drop(shell);
    enqueue(ui, operation)
}
pub fn cancel_export(ui: &UiShell) {
    if ui
        .file_job
        .peek()
        .as_ref()
        .is_some_and(|request| matches!(request.operation, Operation::Export { .. }))
    {
        // Replacing the subscribed request drops its JobHandle and cancels the
        // worker. The destination changes only at the atomic publication point.
        ui.file_job.clone().set(None);
        ui.file_notice
            .clone()
            .set(Some(ui.text("export_cancelled")));
    }
}
pub fn export(ui: &UiShell, request: ExportRequest) -> Result<(), PetuniaError> {
    let document = ui
        .shell
        .peek()
        .bridge
        .session()
        .ok_or_else(|| invalid("no export document"))?
        .document()
        .clone();
    enqueue(ui, Operation::Export { document, request })
}
/// Capture an Action request on the UI thread; native ownership and codecs run
/// on the worker, then stable tab/revision guards publish the Command result.
pub fn route_clipboard(ui: &UiShell, action: &str) -> bool {
    let paste = action == "ptnd.action.edit.paste";
    let cut = action == "ptnd.action.edit.cut";
    if !paste && !cut && action != "ptnd.action.edit.copy" {
        return false;
    }
    let result = (|| {
        let shell = ui.shell.peek();
        let session = shell
            .bridge
            .session()
            .ok_or_else(|| invalid("no clipboard target document"))?;
        let surface = session
            .active_surface()
            .ok_or_else(|| invalid("no clipboard target surface"))?;
        let target = ClipboardTarget {
            session: session.identity(),
            revision: session.current_revision(),
            surface,
        };
        let operation = if paste {
            Operation::ClipboardRead { target }
        } else {
            let objects = session.capture_clipboard_fragment()?;
            if objects.is_empty() {
                return Err(invalid("select objects to copy"));
            }
            let board = session.document().surface(surface)?;
            Operation::ClipboardWrite {
                target,
                frame: [
                    board.origin[0],
                    board.origin[1],
                    board.dimensions[0],
                    board.dimensions[1],
                ],
                objects,
                cut: cut.then(|| session.selection.selected_ids.clone()),
            }
        };
        drop(shell);
        enqueue(ui, operation)
    })();
    if let Err(error) = result {
        ui.file_error.clone().set(Some(error.to_string()));
    }
    true
}
fn read_clipboard(
    target: ClipboardTarget,
    context: &petunia_design_jobs::JobContext,
) -> Result<Outcome, JobFailure> {
    use petunia_design_platform::native_clipboard::{LinuxClipboardBackend, FRAGMENT_MIME};
    let backend = LinuxClipboardBackend::detect().map_err(|e| JobFailure::Failed(e.to_string()))?;
    let cancelled = || context.cancellation().is_cancelled();
    let types = backend
        .available_types(&cancelled)
        .map_err(|e| JobFailure::Failed(e.to_string()))?;
    let available = |mime: &str| types.iter().any(|s| s == mime);
    // Once a format is advertised, malformed/unsupported input is surfaced.
    // Do not silently flatten a vector fragment to another offered format.
    let document = if available(FRAGMENT_MIME) {
        let bytes = backend
            .read(FRAGMENT_MIME, &cancelled)
            .map_err(|e| JobFailure::Failed(e.to_string()))?;
        petunia_design_io::decode_clipboard_fragment(&bytes).map_err(failed)?
    } else if available("image/svg+xml") {
        let bytes = backend
            .read("image/svg+xml", &cancelled)
            .map_err(|e| JobFailure::Failed(e.to_string()))?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| JobFailure::Failed("clipboard SVG is not UTF-8".into()))?;
        petunia_design_io::import_svg(text).map_err(failed)?
    } else {
        let (shape, name, bounds) = if available("image/png") {
            let bytes = backend
                .read("image/png", &cancelled)
                .map_err(|e| JobFailure::Failed(e.to_string()))?;
            let source = Arc::new(
                petunia_design_raster::EncodedImage::new(bytes)
                    .map_err(|e| JobFailure::Failed(e.to_string()))?,
            );
            let image = petunia_design_raster::ImageCache::shared()
                .prepare(&source, &cancelled)
                .map_err(|e| JobFailure::Failed(e.to_string()))?;
            let bounds = [0., 0., f64::from(image.width()), f64::from(image.height())];
            (
                petunia_design_document::ShapeKind::Image {
                    path: "Clipboard.png".into(),
                    data: Some(source),
                },
                "Clipboard image",
                bounds,
            )
        } else {
            let mime = [
                backend.text_mime(),
                "text/plain;charset=utf-8",
                "text/plain",
                "UTF8_STRING",
            ]
            .into_iter()
            .find(|mime| available(mime))
            .ok_or_else(|| {
                JobFailure::Failed(
                    "clipboard contains no supported object, SVG, PNG or UTF-8 text".into(),
                )
            })?;
            let bytes = backend
                .read(mime, &cancelled)
                .map_err(|e| JobFailure::Failed(e.to_string()))?;
            if bytes.len() > 64 * 1024 {
                return Err(JobFailure::Failed("clipboard text exceeds 64 KiB".into()));
            }
            let content = String::from_utf8(bytes)
                .map_err(|_| JobFailure::Failed("clipboard text is not UTF-8".into()))?;
            if content.is_empty() {
                return Err(JobFailure::Failed("clipboard text is empty".into()));
            }
            (
                petunia_design_document::ShapeKind::Text {
                    content,
                    font_family: "sans-serif".into(),
                    font_size: 24.,
                    line_height: 1.3,
                    letter_spacing: 0.,
                    on_path: None,
                },
                "Clipboard text",
                [0., 0., 240., 48.],
            )
        };
        let is_text = matches!(shape, petunia_design_document::ShapeKind::Text { .. });
        let mut object = petunia_design_document::DocumentObject::new(ObjectId::new(1), name);
        object.shape = Some(shape);
        object.bounds = Some(bounds);
        if is_text {
            object.fill = Some("#000000".into());
            object.text_style.flow = petunia_design_document::TextFlow::Artistic;
        }
        let mut document = Document::new();
        {
            let mut mutator = petunia_design_document::DocumentMutator::new(&mut document);
            mutator
                .add_surface(SurfaceId::new(1), "Clipboard")
                .map_err(failed)?;
            mutator
                .set_surface_geometry(SurfaceId::new(1), [0., 0.], [bounds[2], bounds[3]])
                .map_err(failed)?;
            mutator
                .add_objects_bulk(SurfaceId::new(1), vec![object])
                .map_err(failed)?;
        }
        document
    };
    let surface = document
        .surfaces()
        .first()
        .ok_or_else(|| JobFailure::Failed("clipboard has no surface".into()))?;
    if document.surfaces().len() != 1 {
        return Err(JobFailure::Failed(
            "clipboard must contain one surface".into(),
        ));
    }
    Ok(Outcome::Clipboard {
        target,
        objects: surface.objects().to_vec(),
        origin: surface.origin,
        cut: None,
        paste: true,
    })
}
/// Save all required tabs before closing any. A modified snapshot completion
/// keeps every tab open rather than silently discarding the intervening edits.
pub fn save_before_close(ui: &UiShell, only: Option<SessionIdentity>) -> Result<(), PetuniaError> {
    let target = ui
        .shell
        .peek()
        .bridge
        .sessions()
        .into_iter()
        .filter(|s| only.is_none_or(|id| s.identity() == id))
        .find(|s| s.is_dirty())
        .map(|s| (s.identity(), s.path().map(Path::to_path_buf)));
    if let Some((target, path)) = target {
        let intent = if only.is_some() {
            SaveIntent::CloseOne
        } else {
            SaveIntent::CloseAll
        };
        if let Some(path) = path {
            save(ui, target, Some(&path), intent)?;
        } else {
            ui.file_prompt
                .clone()
                .set(Some(FilePrompt::Save { target, intent }));
        }
    } else {
        crate::file_workflows::discard_target(&mut ui.shell.clone().write(), only)?;
    }
    Ok(())
}
fn run(
    operation: Operation,
    context: &petunia_design_jobs::JobContext,
) -> Result<Outcome, JobFailure> {
    context.check_cancelled()?;
    let outcome = match operation {
        Operation::Profile {
            target,
            path,
            purpose,
        } => {
            let profile = petunia_design_color::IccProfile::read(&path).map_err(failed)?;
            let expected = match purpose {
                ProfilePurpose::Press => petunia_design_color::IccColorSpace::Cmyk,
                ProfilePurpose::Monitor => petunia_design_color::IccColorSpace::Rgb,
            };
            if profile.color_space() != expected
                || (purpose == ProfilePurpose::Press && !profile.is_press_profile())
                || (purpose == ProfilePurpose::Monitor && !profile.is_monitor_profile())
            {
                return Err(JobFailure::Failed(
                    "ICC profile channel space does not match its purpose".into(),
                ));
            }
            Outcome::Profile {
                target,
                path,
                purpose,
                profile,
            }
        }
        Operation::ListRecovery { directory } => Outcome::RecoveryList {
            entries: petunia_design_io::recovery::RecoveryStore::new(directory)
                .and_then(|store| store.list())
                .map_err(failed)?,
        },
        Operation::ClipboardWrite {
            target,
            frame,
            objects,
            cut,
        } => {
            let bytes = petunia_design_io::encode_clipboard_fragment(objects.clone(), frame)
                .map_err(failed)?;
            use petunia_design_platform::native_clipboard::{LinuxClipboardBackend, FRAGMENT_MIME};
            let backend =
                LinuxClipboardBackend::detect().map_err(|e| JobFailure::Failed(e.to_string()))?;
            backend
                .write(FRAGMENT_MIME, bytes, &|| {
                    context.cancellation().is_cancelled()
                })
                .map_err(|e| JobFailure::Failed(e.to_string()))?;
            Outcome::Clipboard {
                target,
                objects,
                origin: [frame[0], frame[1]],
                cut,
                paste: false,
            }
        }
        Operation::ClipboardRead { target } => read_clipboard(target, context)?,
        Operation::Open { path, recovery } => {
            if !recovery
                && path
                    .extension()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s.eq_ignore_ascii_case("svg"))
            {
                let document = petunia_design_io::read_svg(&path).map_err(failed)?;
                Outcome::Svg { path, document }
            } else {
                let opened = petunia_design_io::open_package(&path).map_err(failed)?;
                if recovery && opened.recovery.is_none() {
                    return Err(JobFailure::Failed(
                        "selected file is not a recovery snapshot".into(),
                    ));
                }
                Outcome::Open {
                    path,
                    opened,
                    recovery,
                }
            }
        }
        Operation::Save {
            target,
            revision,
            state,
            document,
            path,
            intent,
        } => {
            petunia_design_io::save_package(&document, &path).map_err(failed)?;
            Outcome::Save {
                target,
                revision,
                state,
                path,
                intent,
            }
        }
        Operation::Place {
            target,
            revision,
            surface,
            path,
        } => {
            let source = Arc::new(petunia_design_io::read_encoded_image(&path).map_err(failed)?);
            let image = petunia_design_raster::ImageCache::shared()
                .prepare(&source, &|| context.cancellation().is_cancelled())
                .map_err(|e| JobFailure::Failed(e.to_string()))?;
            Outcome::Place {
                target,
                revision,
                surface,
                path,
                source,
                size: [image.width(), image.height()],
            }
        }
        Operation::Export { document, request } => {
            let outcome = export_document_cancellable(&document, &request, context.cancellation())
                .map_err(failed)?;
            Outcome::Export { path: outcome.path }
        }
    };
    context.check_cancelled()?;
    Ok(outcome)
}
fn failed(error: PetuniaError) -> JobFailure {
    JobFailure::Failed(error.to_string())
}
fn publish(ui: &UiShell, outcome: Outcome) -> Result<PathBuf, PetuniaError> {
    match outcome {
        Outcome::Profile {
            target,
            path,
            purpose,
            profile,
        } => {
            match purpose {
                ProfilePurpose::Press => {
                    ui.shell
                        .clone()
                        .write()
                        .bridge
                        .assign_prepared_cmyk_profile(
                            target.session,
                            target.revision,
                            target.surface,
                            profile,
                        )?;
                }
                ProfilePurpose::Monitor => {
                    ui.monitor_profile.clone().set(Some(profile));
                }
            }
            ui.file_prompt.clone().set(None);
            Ok(path)
        }
        Outcome::RecoveryList { entries } => {
            let visible = !entries.is_empty();
            ui.recovery_entries.clone().set(entries);
            ui.recovery_open.clone().set(visible);
            ui.file_notice.clone().set(None);
            Ok(PathBuf::new())
        }
        Outcome::Clipboard {
            target,
            objects,
            origin,
            cut,
            paste,
        } => {
            ui.shell
                .clone()
                .write()
                .bridge
                .complete_clipboard_fragment(
                    target.session,
                    target.revision,
                    target.surface,
                    objects,
                    cut,
                    paste,
                    origin,
                )?;
            ui.file_notice.clone().set(Some(ui.text("completed")));
            Ok(PathBuf::new())
        }
        Outcome::Open {
            path,
            opened,
            recovery,
        } => {
            ui.shell
                .clone()
                .write()
                .bridge
                .attach_opened_package(&path, opened, recovery)?;
            ui.file_prompt.clone().set(None);
            if recovery {
                ui.recovery_open.clone().set(false);
            }
            Ok(path)
        }
        Outcome::Svg { path, document } => {
            ui.shell.clone().write().bridge.attach_imported_document(
                path.file_stem()
                    .map_or_else(|| "SVG".into(), |n| n.to_string_lossy().into_owned()),
                document,
            )?;
            ui.file_prompt.clone().set(None);
            Ok(path)
        }
        Outcome::Save {
            target,
            revision,
            state,
            path,
            intent,
        } => {
            let clean = ui.shell.clone().write().bridge.acknowledge_saved_snapshot(
                target,
                path.clone(),
                revision,
                state,
            )?;
            ui.file_prompt.clone().set(None);
            if intent != SaveIntent::Stay {
                if !clean {
                    ui.file_notice.clone().set(Some(ui.text("file_new_edits")));
                    return Ok(path);
                }
                save_before_close(
                    ui,
                    if intent == SaveIntent::CloseOne {
                        Some(target)
                    } else {
                        None
                    },
                )?;
            }
            Ok(path)
        }
        Outcome::Place {
            target,
            revision,
            surface,
            path,
            source,
            size,
        } => {
            ui.shell.clone().write().bridge.place_prepared_image(
                target,
                revision,
                surface,
                path.clone(),
                source,
                size,
            )?;
            ui.place_image_open.clone().set(false);
            Ok(path)
        }
        Outcome::Export { path } => {
            ui.export_open.clone().set(false);
            Ok(path)
        }
    }
}
pub fn use_file_jobs(ui: UiShell) {
    let owner = use_hook(|| Rc::new(JobExecutor::new(1, 1, JobManager::new())));
    let requests = ui.file_job;
    use_future(move || {
        let request = requests.read().clone();
        let owner = owner.clone();
        let ui = ui.clone();
        async move {
            let Some(request) = request else { return };
            let executor = match owner.as_ref() {
                Ok(executor) => executor,
                Err(error) => {
                    ui.file_error.clone().set(Some(error.to_string()));
                    ui.file_job.clone().set(None);
                    return;
                }
            };
            let handle = match executor.submit("desktop-file", 0, move |context| {
                run(request.operation, context)
            }) {
                Ok(handle) => handle,
                Err(error) => {
                    ui.file_error.clone().set(Some(error.to_string()));
                    ui.file_job.clone().set(None);
                    return;
                }
            };
            loop {
                match handle.try_result(0) {
                    Ok(None) => timer(Duration::from_millis(16)).await,
                    result => {
                        ui.file_job.clone().set(None);
                        let result = result
                            .map_err(|e| PetuniaError::io(e.to_string()))
                            .and_then(|outcome| {
                                publish(&ui, outcome.expect("complete worker outcome"))
                            });
                        match result {
                            Ok(path) => {
                                if ui.file_job.peek().is_none()
                                    && ui.file_notice.peek().as_ref()
                                        == Some(&ui.text("file_working"))
                                {
                                    ui.file_notice.clone().set(Some(format!(
                                        "{}: {}",
                                        ui.text("completed"),
                                        path.display()
                                    )))
                                }
                            }
                            Err(error) => {
                                ui.file_notice.clone().set(None);
                                ui.file_error
                                    .clone()
                                    .set(Some(format!("{}: {error}", ui.text("failed"))))
                            }
                        }
                        break;
                    }
                }
            }
        }
    });
}
