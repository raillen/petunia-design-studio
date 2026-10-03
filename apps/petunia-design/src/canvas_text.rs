//! Draft-based artwork text editor. Native IME events and grapheme editing use
//! the exact prepared scene for caret/selection, including ancestor transforms.
use crate::{
    object_edits::{EditKind, EditValue, ObjectEdit},
    ui_state::UiShell,
};
use freya::prelude::*;
use petunia_design_application::preview::{PreviewSource, PreviewSourceId};
use petunia_design_foundation::ObjectId;
use petunia_design_geometry::{GPath, GPoint};
use petunia_design_text::TextEditBuffer;
use std::{cell::RefCell, rc::Rc, sync::Arc};

#[derive(Clone, Debug, PartialEq)]
pub struct Draft {
    pub original: ObjectEdit,
    pub buffer: TextEditBuffer,
    dragging: bool,
}
#[derive(Clone, Default)]
pub struct Overlay {
    pub selection: Vec<GPath>,
    pub preedit: Vec<GPath>,
    pub caret: Option<[GPoint; 2]>,
}
#[derive(Clone, PartialEq)]
pub struct ClipboardRequest {
    token: u64,
    draft: Draft,
    operation: ClipboardOperation,
}
#[derive(Clone, PartialEq)]
enum ClipboardOperation {
    Copy(bool),
    Paste,
}
fn clipboard(ui: &UiShell, draft: &Draft, operation: ClipboardOperation) {
    if draft.buffer.has_preedit()
        || matches!(operation, ClipboardOperation::Copy(_))
            && draft.buffer.selection().range().is_empty()
    {
        return;
    }
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1);
    ui.text_clipboard.clone().set(Some(ClipboardRequest {
        token: NEXT.fetch_add(1, Ordering::Relaxed),
        draft: draft.clone(),
        operation,
    }));
}
pub fn use_clipboard(ui: &UiShell) {
    use petunia_design_jobs::{JobExecutor, JobFailure, JobManager};
    use petunia_design_platform::native_clipboard::LinuxClipboardBackend;
    let owner = use_hook(|| Rc::new(JobExecutor::new(1, 1, JobManager::new())));
    use_future({
        let owner = owner.clone();
        let ui = ui.clone();
        move || {
            let request = ui.text_clipboard.read().clone();
            let owner = owner.clone();
            let ui = ui.clone();
            async move {
                let Some(request) = request else { return };
                let worker_request = request.clone();
                let handle = match owner.as_ref() {
                    Ok(executor) => executor.submit("text-clipboard", 0, move |context| {
                        let backend = LinuxClipboardBackend::detect()
                            .map_err(|e| JobFailure::Failed(e.to_string()))?;
                        match worker_request.operation {
                            ClipboardOperation::Copy(_) => {
                                let range = worker_request.draft.buffer.selection().range();
                                let text = &worker_request.draft.buffer.content()[range];
                                backend
                                    .write(backend.text_mime(), text.as_bytes().to_vec(), &|| {
                                        context.cancellation().is_cancelled()
                                    })
                                    .map_err(|e| JobFailure::Failed(e.to_string()))?;
                                Ok(None)
                            }
                            ClipboardOperation::Paste => {
                                let bytes = backend
                                    .read(backend.text_mime(), &|| {
                                        context.cancellation().is_cancelled()
                                    })
                                    .map_err(|e| JobFailure::Failed(e.to_string()))?;
                                String::from_utf8(bytes).map(Some).map_err(|_| {
                                    JobFailure::Failed("clipboard text is not UTF-8".into())
                                })
                            }
                        }
                    }),
                    Err(error) => Err(error.clone()),
                };
                let result = match handle {
                    Ok(handle) => loop {
                        match handle.try_result(0) {
                            Ok(Some(value)) => break Ok(value),
                            Err(error) => break Err(error),
                            Ok(None) => timer(std::time::Duration::from_millis(16)).await,
                        }
                    },
                    Err(error) => Err(error),
                };
                if ui
                    .text_clipboard
                    .peek()
                    .as_ref()
                    .is_none_or(|current| current.token != request.token)
                {
                    return;
                }
                ui.text_clipboard.clone().set(None);
                match result {
                    Err(error) => ui.file_error.clone().set(Some(error.to_string())),
                    Ok(text) => {
                        let Some(mut draft) = ui.canvas_text.peek().clone().filter(|draft| {
                            draft.original == request.draft.original
                                && draft.buffer == request.draft.buffer
                        }) else {
                            return;
                        };
                        let result = match request.operation {
                            ClipboardOperation::Copy(true) => draft.buffer.insert(""),
                            ClipboardOperation::Paste => {
                                draft.buffer.insert(text.as_deref().unwrap_or(""))
                            }
                            ClipboardOperation::Copy(false) => Ok(false),
                        };
                        if let Err(error) = result {
                            ui.file_error.clone().set(Some(error.to_string()));
                        }
                        ui.canvas_text.clone().set(Some(draft));
                    }
                }
            }
        }
    });
}
pub fn request(ui: &UiShell, object: ObjectId) {
    match ObjectEdit::capture(&ui.shell.peek(), object, EditKind::Text) {
        Ok(original) => {
            let EditValue::Text(petunia_design_document::ShapeKind::Text { content, .. }) =
                &original.original
            else {
                return;
            };
            match TextEditBuffer::new(content.clone()) {
                Ok(mut buffer) => {
                    buffer.move_to(content.len(), false);
                    ui.canvas_text.clone().set(Some(Draft {
                        original,
                        buffer,
                        dragging: false,
                    }));
                }
                Err(error) => ui.file_error.clone().set(Some(error.to_string())),
            }
        }
        Err(reason) => ui
            .file_error
            .clone()
            .set(Some(crate::object_edit_dialog::failure_text(ui, reason))),
    }
}
pub fn apply(ui: &UiShell) {
    let Some(draft) = ui.canvas_text.peek().clone() else {
        return;
    };
    if draft.buffer.has_preedit() {
        return;
    }
    match draft.original.apply(
        &mut ui.shell.clone().write(),
        draft.buffer.content(),
        [""; 5],
    ) {
        Ok(_) => ui.canvas_text.clone().set(None),
        Err(reason) => ui
            .file_error
            .clone()
            .set(Some(crate::object_edit_dialog::failure_text(ui, reason))),
    }
}
type Projection = (PreviewSourceId, ObjectId, String, Arc<PreviewSource>);
pub fn use_projection(
    ui: &UiShell,
    source: Option<Arc<PreviewSource>>,
) -> Option<Arc<PreviewSource>> {
    let cache = use_hook(|| Rc::new(RefCell::new(None::<Projection>)));
    let draft = ui.canvas_text.read();
    let Some(draft) = draft.as_ref() else {
        cache.borrow_mut().take();
        return source;
    };
    if ui
        .shell
        .peek()
        .bridge
        .session()
        .is_none_or(|session| session.identity() != draft.original.session)
    {
        return source;
    }
    let source = source?;
    let content = draft.buffer.display_content();
    let id = draft.original.object;
    if let Some((_, _, _, preview)) =
        cache
            .borrow()
            .as_ref()
            .filter(|(base, target, previous, _)| {
                *base == source.id() && *target == id && *previous == content
            })
    {
        return Some(preview.clone());
    }
    match source.with_text_edit(id, content.clone()) {
        Ok(preview) => {
            *cache.borrow_mut() = Some((source.id(), id, content, preview.clone()));
            Some(preview)
        }
        Err(_) => Some(source),
    }
}
pub fn is_active(ui: &UiShell) -> bool {
    ui.canvas_text.read().as_ref().is_some_and(|draft| {
        ui.shell.read().bridge.session().is_some_and(|session| {
            session.identity() == draft.original.session
                && session
                    .active_surface()
                    .and_then(|id| session.document().surface(id).ok())
                    .is_some_and(|surface| {
                        surface
                            .objects()
                            .iter()
                            .any(|object| object.id == draft.original.object)
                    })
        })
    })
}
pub fn overlay(ui: &UiShell, source: Option<&Arc<PreviewSource>>) -> Overlay {
    if !is_active(ui) {
        return Overlay::default();
    }
    let draft = ui.canvas_text.peek();
    let Some(draft) = draft.as_ref() else {
        return Overlay::default();
    };
    let Some(scene) = source.and_then(|source| source.prepared_scene()) else {
        return Overlay::default();
    };
    let Some(node) = scene.node(draft.original.object) else {
        return Overlay::default();
    };
    let Some(text) = node.prepared_text() else {
        return Overlay::default();
    };
    let world = node.local_to_world();
    let selection = draft.buffer.display_selection();
    let paths = |range| {
        text.selection_rects(range)
            .into_iter()
            .map(|rect| GPath::rect(rect, 0., 0.).transformed(world))
            .collect()
    };
    let caret = text.caret_rect(selection.focus);
    Overlay {
        selection: paths(selection.range()),
        preedit: draft.buffer.preedit_range().map_or_else(Vec::new, paths),
        caret: Some([
            world.apply(GPoint::new(caret.x0, caret.y0)),
            world.apply(GPoint::new(caret.x1, caret.y1)),
        ]),
    }
}
pub fn pointer(
    ui: &UiShell,
    phase: petunia_design_application::PointerPhase,
    world: GPoint,
) -> bool {
    use petunia_design_application::PointerPhase;
    if !is_active(ui) {
        return false;
    }
    let Some(mut draft) = ui.canvas_text.peek().clone() else {
        return false;
    };
    let source = ui.canvas_text_source.peek().clone();
    let Some(scene) = source.and_then(|source| source.prepared_scene()) else {
        return true;
    };
    let Some(node) = scene.node(draft.original.object) else {
        return true;
    };
    let Some(text) = node.prepared_text() else {
        return true;
    };
    let Some(inverse) = node.local_to_world().inverse() else {
        return true;
    };
    let local = inverse.apply(world);
    if phase == PointerPhase::Down {
        draft
            .buffer
            .move_to(text.offset_at_point(local), ui.modifiers.peek().constrain);
        draft.dragging = true;
    } else if phase == PointerPhase::Move && draft.dragging {
        draft.buffer.move_to(text.offset_at_point(local), true);
    } else if matches!(phase, PointerPhase::Up | PointerPhase::Cancel) {
        draft.dragging = false;
    }
    ui.canvas_text.clone().set(Some(draft));
    true
}
pub fn preedit(ui: &UiShell, event: &Event<ImePreeditEventData>) {
    if !is_active(ui) {
        return;
    }
    let mut state = ui.canvas_text;
    let Some(mut draft) = state.peek().clone() else {
        return;
    };
    if let Err(reason) = draft.buffer.set_preedit(event.text.clone(), event.cursor) {
        ui.file_error.clone().set(Some(reason.to_string()));
    }
    state.set(Some(draft));
}
pub fn key(ui: &UiShell, event: &Event<KeyboardEventData>) -> bool {
    if !is_active(ui) {
        return false;
    }
    let Some(mut draft) = ui.canvas_text.peek().clone() else {
        return false;
    };
    let command = event
        .modifiers
        .intersects(Modifiers::CONTROL | Modifiers::META);
    let shift = event.modifiers.contains(Modifiers::SHIFT);
    use NamedKey::*;
    let mut result = Ok(false);
    match &event.key {
        Key::Named(Escape) => {
            if draft.buffer.has_preedit() {
                draft.buffer.cancel_preedit();
            } else {
                ui.canvas_text.clone().set(None);
                event.prevent_default();
                return true;
            }
        }
        Key::Named(Enter) if command => {
            apply(ui);
            event.prevent_default();
            return true;
        }
        Key::Named(Enter) => result = draft.buffer.insert("\n"),
        Key::Named(Tab) => result = draft.buffer.insert("\t"),
        Key::Named(Backspace) => result = draft.buffer.delete(true),
        Key::Named(Delete) => result = draft.buffer.delete(false),
        Key::Named(ArrowLeft) => draft.buffer.move_horizontal(false, shift),
        Key::Named(ArrowRight) => draft.buffer.move_horizontal(true, shift),
        Key::Named(Home | End) => {
            let at = draft.buffer.selection().focus;
            let content = draft.buffer.content();
            let next = if matches!(event.key, Key::Named(Home)) {
                if command {
                    0
                } else {
                    content[..at].rfind('\n').map_or(0, |i| i + 1)
                }
            } else if command {
                content.len()
            } else {
                content[at..].find('\n').map_or(content.len(), |i| at + i)
            };
            draft.buffer.move_to(next, shift);
        }
        Key::Named(ArrowUp | ArrowDown) => {
            if let Some(scene) = ui
                .canvas_text_source
                .peek()
                .as_ref()
                .and_then(|source| source.prepared_scene())
            {
                if let Some(text) = scene
                    .node(draft.original.object)
                    .and_then(|node| node.prepared_text())
                {
                    let caret = text.caret_rect(draft.buffer.selection().focus);
                    let direction = if matches!(event.key, Key::Named(ArrowUp)) {
                        -1.
                    } else {
                        1.
                    };
                    draft.buffer.move_to(
                        text.offset_at_point(GPoint::new(
                            caret.x0,
                            (caret.y0 + caret.y1) / 2. + direction * caret.height(),
                        )),
                        shift,
                    );
                }
            }
        }
        Key::Character(text) if command && text.eq_ignore_ascii_case("a") => {
            draft.buffer.select_all()
        }
        Key::Character(text) if command && text.eq_ignore_ascii_case("z") => {
            if shift {
                draft.buffer.redo();
            } else {
                draft.buffer.undo();
            }
        }
        Key::Character(text) if command && text.eq_ignore_ascii_case("y") => {
            draft.buffer.redo();
        }
        Key::Character(text) if command && text.eq_ignore_ascii_case("c") => {
            clipboard(ui, &draft, ClipboardOperation::Copy(false))
        }
        Key::Character(text) if command && text.eq_ignore_ascii_case("x") => {
            clipboard(ui, &draft, ClipboardOperation::Copy(true))
        }
        Key::Character(text) if command && text.eq_ignore_ascii_case("v") => {
            clipboard(ui, &draft, ClipboardOperation::Paste)
        }
        Key::Character(text) if !command && !event.modifiers.contains(Modifiers::ALT) => {
            result = draft.buffer.insert(text)
        }
        _ => {}
    }
    if let Err(error) = result {
        ui.file_error.clone().set(Some(error.to_string()));
    }
    ui.canvas_text.clone().set(Some(draft));
    event.prevent_default();
    true
}
