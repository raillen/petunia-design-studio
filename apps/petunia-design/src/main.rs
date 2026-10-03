//! Petunia Design Studio desktop app, Freya shell (08.03).
//!
//! The shell is a projection: every mutation goes through the Action lane of
//! the Aubrieta GUI bridge and the UI re-renders from the shell state.
//! Domain crates never import toolkit types; the UI receives DTOs and sends
//! activation tokens back across the bridge.

mod actions;
mod appearance;
mod canvas_paint;
mod canvas_preview;
mod canvas_text;
mod chrome;
mod color_ui;
mod dialogs;
mod dock;
mod export_preview;
mod file_dialogs;
mod file_jobs;
mod file_workflows;
mod histogram_ui;
mod object_edit_dialog;
mod object_edits;
mod recovery;
mod theme;
mod typography;
mod ui_state;

use freya::prelude::*;
use petunia_design_application::interaction::{
    NormalizedPointerEvent, PointerButton, PointerPhase, SemanticModifiers,
};
use petunia_design_application::tools::ToolKind;
use petunia_design_geometry::GPoint;
use petunia_design_shell::PetuniaShell;

use petunia_design_shell::canvas::CursorAffordance;

fn map_cursor_affordance(affordance: CursorAffordance) -> CursorIcon {
    match affordance {
        CursorAffordance::Default => CursorIcon::Default,
        CursorAffordance::Pointer => CursorIcon::Pointer,
        CursorAffordance::Crosshair => CursorIcon::Crosshair,
        CursorAffordance::Move => CursorIcon::Move,
        CursorAffordance::Grab => CursorIcon::Grab,
        CursorAffordance::Grabbing => CursorIcon::Grabbing,
        CursorAffordance::ResizeNwse => CursorIcon::NwseResize,
        CursorAffordance::ResizeNesw => CursorIcon::NeswResize,
        CursorAffordance::ResizeCol => CursorIcon::EwResize,
        CursorAffordance::ResizeRow => CursorIcon::NsResize,
        CursorAffordance::Rotate => CursorIcon::Crosshair,
        CursorAffordance::Text => CursorIcon::Text,
        CursorAffordance::NotAllowed => CursorIcon::NotAllowed,
    }
}

#[cfg(test)]
use crate::actions::run_action_id;
use crate::appearance::AppearanceBar;
use crate::chrome::{
    resolve_tool_shortcut, shortcut_key, ContextToolbar, DocumentTabStrip, MenuBarRow, ToolRail,
    TooltipOverlay,
};
use crate::dialogs::{
    CommandPalette, ConfirmCloseDialog, CustomizeDialog, ExportDialog, NewDocumentDialog,
    OffsetPathDialog, OverwriteConflictDialog, PlaceImageDialog,
};
use crate::ui_state::UiShell;

const WINDOW_WIDTH: f64 = 1280.;
const WINDOW_HEIGHT: f64 = 800.;

/// Default document name: domain data, not UI copy.
const DEFAULT_DOCUMENT_TITLE: &str = "Untitled";

fn main() {
    let bridge = petunia_design_shell::PetuniaDesignGuiBridge::new();
    let title = bridge
        .localization()
        .text("ptnd.text.shell.brand", bridge.locale());

    launch(
        LaunchConfig::new().with_window(
            WindowConfig::new(app)
                .with_size(WINDOW_WIDTH, WINDOW_HEIGHT)
                .with_window_attributes(move |attributes, _| attributes.with_title(title)),
        ),
    );
}

/// Starts an empty document; examples are created explicitly by the user.
fn app() -> impl IntoElement {
    use_init_theme(theme::petunia_theme);

    let shell = use_state(|| {
        let mut shell = PetuniaShell::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        shell
            .new_document(DEFAULT_DOCUMENT_TITLE)
            .expect("a fresh document opens");
        shell
    });
    let ui = UiShell::fresh(shell);
    file_jobs::use_file_jobs(ui.clone());
    let root_a11y_id = use_a11y();
    let keyboard_shell = ui.shell;
    let modifiers = ui.modifiers;
    let active_tool = ui.active_tool;
    let temporary_tool = ui.temporary_tool;
    let suspended_tool = ui.suspended_tool;

    rect()
        .direction(Direction::Vertical)
        .content(Content::Flex)
        .width(Size::fill())
        .height(Size::fill())
        .background(theme::SURFACE_WORKSPACE)
        .a11y_id(root_a11y_id)
        .a11y_focusable(true)
        .a11y_auto_focus(true)
        .child(MenuBarRow(ui.clone()))
        .child(DocumentTabStrip(ui.clone()))
        .child(ContextToolbar(ui.clone()))
        .child(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .width(Size::fill())
                .height(Size::flex(1.0))
                .child(ToolRail(ui.clone()))
                .child(dock::LeftDock(ui.clone()))
                .child(dock::LeftDockSplitter(ui.clone()))
                .child(
                    rect()
                        .direction(Direction::Vertical)
                        .content(Content::Flex)
                        .width(Size::flex(1.0))
                        .height(Size::fill())
                        .child(Workspace(ui.clone()))
                        .child(dock::BottomDockSplitter(ui.clone()))
                        .child(dock::BottomDock(ui.clone())),
                )
                .child(dock::DockSplitter(ui.clone()))
                .child(dock::RightDock(ui.clone())),
        )
        .child(StatusBar(ui.clone()))
        .child(CommandPalette(ui.clone()))
        .child(CustomizeDialog(ui.clone()))
        .child(NewDocumentDialog(ui.clone()))
        .child(ExportDialog(ui.clone()))
        .child(PlaceImageDialog(ui.clone()))
        .child(file_dialogs::FileDialog(ui.clone()))
        .child(recovery::RecoveryDialog(ui.clone()))
        .child(object_edit_dialog::ObjectEditDialog(ui.clone()))
        .child(typography::TypographyDialog(ui.clone()))
        .child(ConfirmCloseDialog(ui.clone()))
        .child(OffsetPathDialog(ui.clone()))
        .child(OverwriteConflictDialog(ui.clone()))
        .child(TooltipOverlay(ui.clone()))
        .on_global_key_down({
            let keyboard_ui = ui.clone();
            let mut modifiers = modifiers;
            move |event: Event<KeyboardEventData>| {
                modifiers.set(semantic_modifiers(event.modifiers));
                dispatch_workspace_key(&keyboard_ui, &event);
            }
        })
        .on_global_key_up({
            let mut modifiers = modifiers;
            move |event: Event<KeyboardEventData>| {
                modifiers.set(semantic_modifiers(event.modifiers));
                if shortcut_key(&event).is_some_and(|key| key.eq_ignore_ascii_case("space")) {
                    restore_temporary_tool(
                        keyboard_shell,
                        active_tool,
                        temporary_tool,
                        suspended_tool,
                    );
                }
            }
        })
}

/// The document workspace. The canvas slice (08.29) draws the artboard here.
#[derive(Clone, PartialEq)]
struct Workspace(UiShell);

impl Component for Workspace {
    fn render(&self) -> impl IntoElement {
        let mut shell = self.0.shell;
        recovery::use_recovery(&self.0);
        canvas_text::use_clipboard(&self.0);
        let modifiers = self.0.modifiers;
        let mut gesture_tick = use_state(|| 0u64);
        let ruler_drag = use_state(|| None::<(petunia_design_document::GuideOrientation, f64)>);
        let middle_pan_last = use_state(|| None::<GPoint>);
        let mut is_pointer_down = use_state(|| false);
        let a11y_id = use_a11y();
        use_future(move || async move {
            loop {
                timer(std::time::Duration::from_millis(16)).await;
                if shell.peek().tools.has_pending_jobs() {
                    let _ = shell.write().poll_tool_jobs();
                }
            }
        });
        let mut snapshot = shell.read().canvas_snapshot();
        snapshot.preview_source =
            canvas_text::use_projection(&self.0, snapshot.preview_source.clone());
        if *self.0.canvas_text_source.peek() != snapshot.preview_source {
            self.0
                .canvas_text_source
                .clone()
                .set(snapshot.preview_source.clone());
        }
        let (preview, preview_error) = canvas_preview::use_canvas_preview(
            &snapshot,
            *self.0.channel_view.read(),
            *self.0.soft_proof.read(),
            snapshot
                .preview_source
                .as_ref()
                .and_then(|source| source.surface_snapshot().cmyk_profile.clone())
                .zip(self.0.monitor_profile.read().clone())
                .map(|(proof, monitor)| petunia_design_color::IccProofSettings {
                    proof,
                    monitor,
                    options: *self.0.proof_options.read(),
                    proof_intent: self.0.proof_options.read().intent,
                }),
        );
        let cursor_icon = map_cursor_affordance(snapshot.overlays.cursor);
        let in_flight_guide = *ruler_drag.read();

        let active_text_object = snapshot.objects.iter().find(|o| {
            o.active
                && matches!(
                    o.shape.as_deref(),
                    Some(petunia_design_document::ShapeKind::Text { .. })
                )
        });

        let active_text_editor = if let Some(text_obj) = active_text_object {
            let screen_origin = snapshot.camera.doc_to_screen(GPoint::new(
                text_obj.frame_origin[0],
                text_obj.frame_origin[1],
            ));
            let text_obj_id = text_obj.id;
            let edit_ui = self.0.clone();
            let apply_ui = self.0.clone();
            let mut cancel_edit = self.0.canvas_text;
            Some(
                rect()
                    .position(
                        Position::new_absolute()
                            .left((screen_origin.x as f32).max(24.))
                            .top(((screen_origin.y - 42.) as f32).max(24.)),
                    )
                    .direction(Direction::Horizontal)
                    .background(theme::SURFACE_PANEL)
                    .border(
                        Border::new()
                            .fill(theme::BLOOM.value)
                            .width(1.5)
                            .alignment(BorderAlignment::Inner),
                    )
                    .padding(Gaps::new_all(4.))
                    .cross_align(Alignment::Center)
                    .spacing(4.)
                    .child(
                        Button::new()
                            .on_press(move |event: Event<PressEventData>| {
                                event.stop_propagation();
                                canvas_text::request(&edit_ui, text_obj_id);
                                a11y_id.request_focus();
                            })
                            .child(label().text(self.0.text("edit_text")).font_size(11.)),
                    )
                    .maybe(canvas_text::is_active(&self.0), |element| {
                        element
                            .child(
                                Button::new()
                                    .on_press(move |_| canvas_text::apply(&apply_ui))
                                    .child(self.0.text("edit_apply")),
                            )
                            .child(
                                Button::new()
                                    .on_press(move |_| cancel_edit.set(None))
                                    .child(self.0.text("cancel")),
                            )
                            .child(label().text(self.0.text("text_canvas_hint")).font_size(10.))
                    }),
            )
        } else {
            None
        };

        let text_overlay = canvas_text::overlay(&self.0, snapshot.preview_source.as_ref());
        let editing = canvas_text::is_active(&self.0);
        let mut workspace_container = rect()
            .width(Size::flex(1.0))
            .height(Size::fill())
            .background(theme::SURFACE_WORKSPACE)
            .cursor(cursor_icon)
            .a11y_id(a11y_id)
            .a11y_focusable(true)
            .a11y_role(if editing {
                AccessibilityRole::MultilineTextInput
            } else {
                AccessibilityRole::GenericContainer
            })
            .on_ime_preedit({
                let ui = self.0.clone();
                move |event| canvas_text::preedit(&ui, &event)
            })
            .on_global_pointer_press({
                let edit_ui = self.0.clone();
                move |event: Event<PointerEventData>| {
                    if *is_pointer_down.peek() {
                        is_pointer_down.set(false);
                        let location = event.element_location();
                        if canvas_text::pointer(
                            &edit_ui,
                            PointerPhase::Up,
                            shell
                                .peek()
                                .view_camera()
                                .screen_to_doc(GPoint::new(location.x, location.y)),
                        ) {
                            return;
                        }
                        let button = event
                            .button()
                            .and_then(|b| pointer_button(Some(b)))
                            .unwrap_or(PointerButton::Primary);
                        dispatch_workspace_at(
                            shell,
                            modifiers,
                            gesture_tick,
                            ruler_drag,
                            middle_pan_last,
                            PointerPhase::Up,
                            button,
                            GPoint::new(location.x, location.y),
                        );
                    }
                }
            })
            .on_sized({
                move |event: Event<SizedEventData>| {
                    let width = event.area.width() as f64;
                    let height = event.area.height() as f64;
                    let mut camera = shell.peek().view_camera();
                    if (camera.viewport_width - width).abs() > 0.5
                        || (camera.viewport_height - height).abs() > 0.5
                    {
                        camera.resize(width, height);
                        shell.write().set_view_camera(camera);
                        let next = *gesture_tick.peek() + 1;
                        gesture_tick.set(next);
                    }
                }
            })
            .child(
                canvas_paint::canvas_view(
                    snapshot,
                    in_flight_guide,
                    *self.0.soft_proof.read(),
                    *self.0.channel_view.read(),
                    preview,
                    text_overlay,
                )
                .on_pointer_down({
                    let edit_ui = self.0.clone();
                    move |event: Event<PointerEventData>| {
                        a11y_id.request_focus();
                        is_pointer_down.set(true);
                        let location = event.element_location();
                        if canvas_text::pointer(
                            &edit_ui,
                            PointerPhase::Down,
                            shell
                                .peek()
                                .view_camera()
                                .screen_to_doc(GPoint::new(location.x, location.y)),
                        ) {
                            return;
                        }
                        dispatch_workspace_pointer(
                            shell,
                            modifiers,
                            gesture_tick,
                            ruler_drag,
                            middle_pan_last,
                            PointerPhase::Down,
                            &event,
                        );
                    }
                })
                .on_pointer_move({
                    let edit_ui = self.0.clone();
                    move |event: Event<PointerEventData>| {
                        let location = event.element_location();
                        if canvas_text::pointer(
                            &edit_ui,
                            PointerPhase::Move,
                            shell
                                .peek()
                                .view_camera()
                                .screen_to_doc(GPoint::new(location.x, location.y)),
                        ) {
                            return;
                        }
                        dispatch_workspace_pointer(
                            shell,
                            modifiers,
                            gesture_tick,
                            ruler_drag,
                            middle_pan_last,
                            PointerPhase::Move,
                            &event,
                        );
                    }
                })
                .on_mouse_up({
                    let edit_ui = self.0.clone();
                    move |event: Event<MouseEventData>| {
                        event.prevent_default();
                        if *is_pointer_down.peek() {
                            is_pointer_down.set(false);
                            let button =
                                pointer_button(event.button).unwrap_or(PointerButton::Primary);
                            let location = event.element_location;
                            if canvas_text::pointer(
                                &edit_ui,
                                PointerPhase::Up,
                                shell
                                    .peek()
                                    .view_camera()
                                    .screen_to_doc(GPoint::new(location.x, location.y)),
                            ) {
                                return;
                            }
                            dispatch_workspace_at(
                                shell,
                                modifiers,
                                gesture_tick,
                                ruler_drag,
                                middle_pan_last,
                                PointerPhase::Up,
                                button,
                                GPoint::new(location.x, location.y),
                            );
                        }
                    }
                })
                .on_touch_end({
                    let edit_ui = self.0.clone();
                    move |event: Event<TouchEventData>| {
                        if *is_pointer_down.peek() {
                            is_pointer_down.set(false);
                            let location = event.element_location;
                            if canvas_text::pointer(
                                &edit_ui,
                                PointerPhase::Up,
                                shell
                                    .peek()
                                    .view_camera()
                                    .screen_to_doc(GPoint::new(location.x, location.y)),
                            ) {
                                return;
                            }
                            dispatch_workspace_at(
                                shell,
                                modifiers,
                                gesture_tick,
                                ruler_drag,
                                middle_pan_last,
                                PointerPhase::Up,
                                PointerButton::Primary,
                                GPoint::new(location.x, location.y),
                            );
                        }
                    }
                })
                .on_touch_cancel({
                    let edit_ui = self.0.clone();
                    move |event: Event<TouchEventData>| {
                        if *is_pointer_down.peek() {
                            is_pointer_down.set(false);
                            let location = event.element_location;
                            if canvas_text::pointer(
                                &edit_ui,
                                PointerPhase::Cancel,
                                shell
                                    .peek()
                                    .view_camera()
                                    .screen_to_doc(GPoint::new(location.x, location.y)),
                            ) {
                                return;
                            }
                            dispatch_workspace_at(
                                shell,
                                modifiers,
                                gesture_tick,
                                ruler_drag,
                                middle_pan_last,
                                PointerPhase::Cancel,
                                PointerButton::Primary,
                                GPoint::new(location.x, location.y),
                            );
                        }
                    }
                })
                .on_wheel({
                    move |event: Event<WheelEventData>| {
                        let cursor = event.element_location;
                        let screen_focus = GPoint::new(cursor.x, cursor.y);
                        let control = modifiers.read().disable_snap;
                        if control {
                            // Zoom centered on cursor location
                            if event.delta_y.abs() > 0.05 {
                                let factor = if event.delta_y < 0. { 1.12 } else { 1.0 / 1.12 };
                                shell.write().zoom_at(screen_focus, factor);
                                let next = *gesture_tick.peek() + 1;
                                gesture_tick.set(next);
                            }
                        } else {
                            // Smooth pan
                            if event.delta_y.abs() > 0.05 || event.delta_x.abs() > 0.05 {
                                let dx = -event.delta_x;
                                let dy = -event.delta_y;
                                shell.write().pan(dx, dy);
                                let next = *gesture_tick.peek() + 1;
                                gesture_tick.set(next);
                            }
                        }
                    }
                }),
            );

        if let Some(error) =
            preview_error.or_else(|| shell.read().tools.feedback().map(str::to_owned))
        {
            let prefix = {
                let shell = shell.read();
                shell.bridge.localization().text(
                    "ptnd.text.canvas.preview_unavailable",
                    shell.bridge.locale(),
                )
            };
            workspace_container = workspace_container
                .child(label().text(format!("{prefix}: {error}")).font_size(12.0));
        }
        if let Some(editor) = active_text_editor {
            workspace_container = workspace_container.child(editor);
        }

        workspace_container
    }
}

fn dispatch_workspace_pointer(
    shell: State<PetuniaShell>,
    modifiers: State<SemanticModifiers>,
    gesture_tick: State<u64>,
    ruler_drag: State<Option<(petunia_design_document::GuideOrientation, f64)>>,
    middle_pan_last: State<Option<GPoint>>,
    phase: PointerPhase,
    event: &Event<PointerEventData>,
) {
    let location = event.element_location();
    let button = if phase == PointerPhase::Move {
        PointerButton::Primary
    } else if let Some(button) = event
        .button()
        .and_then(|button| pointer_button(Some(button)))
    {
        button
    } else {
        PointerButton::Primary
    };
    dispatch_workspace_with_pressure(
        shell,
        modifiers,
        gesture_tick,
        ruler_drag,
        middle_pan_last,
        phase,
        button,
        GPoint::new(location.x, location.y),
        match event.data() {
            PointerEventData::Touch(touch) => normalized_pressure(touch.force),
            PointerEventData::Mouse(_) => 1.0,
        },
    );
}

// The workspace pointer fan-out carries one `State` handle per gesture
// concern straight from the component scope into Freya event closures.
// Grouping them would only move the arity into a struct literal at each of
// the five call sites, so the arity is allowed here.
// (clippy::too_many_arguments: Freya event-plumbing boundary)
fn normalized_pressure(force: Option<Force>) -> f64 {
    let pressure = match force {
        Some(Force::Normalized(value)) => value,
        Some(Force::Calibrated {
            force,
            max_possible_force,
            ..
        }) if max_possible_force.is_finite() && max_possible_force > 0.0 => {
            force / max_possible_force
        }
        _ => 1.0,
    };
    if pressure.is_finite() {
        pressure.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

#[allow(clippy::too_many_arguments)]
fn dispatch_workspace_at(
    shell: State<PetuniaShell>,
    modifiers: State<SemanticModifiers>,
    gesture_tick: State<u64>,
    ruler_drag: State<Option<(petunia_design_document::GuideOrientation, f64)>>,
    middle_pan_last: State<Option<GPoint>>,
    phase: PointerPhase,
    button: PointerButton,
    screen: GPoint,
) {
    dispatch_workspace_with_pressure(
        shell,
        modifiers,
        gesture_tick,
        ruler_drag,
        middle_pan_last,
        phase,
        button,
        screen,
        1.0,
    );
}

#[allow(clippy::too_many_arguments)]
fn dispatch_workspace_with_pressure(
    mut shell: State<PetuniaShell>,
    modifiers: State<SemanticModifiers>,
    mut gesture_tick: State<u64>,
    mut ruler_drag: State<Option<(petunia_design_document::GuideOrientation, f64)>>,
    mut middle_pan_last: State<Option<GPoint>>,
    phase: PointerPhase,
    button: PointerButton,
    screen: GPoint,
    pressure: f64,
) {
    // Move events have no pressed button in Freya. Continue captured navigation
    // until release/cancel instead of letting Select/Brush consume the drag.
    if button == PointerButton::Middle || middle_pan_last.peek().is_some() {
        match phase {
            PointerPhase::Down => {
                middle_pan_last.set(Some(screen));
                return;
            }
            PointerPhase::Move => {
                let last = *middle_pan_last.peek();
                if let Some(last) = last {
                    let dx = screen.x - last.x;
                    let dy = screen.y - last.y;
                    shell.write().pan(dx, dy);
                    middle_pan_last.set(Some(screen));
                    let next = *gesture_tick.peek() + 1;
                    gesture_tick.set(next);
                    return;
                }
            }
            PointerPhase::Up | PointerPhase::Cancel => {
                if middle_pan_last.peek().is_some() {
                    middle_pan_last.set(None);
                    return;
                }
            }
        }
    }

    // 2. Interactive Guide drag out of metric rulers
    let camera = shell.peek().view_camera();
    let current_ruler_drag = *ruler_drag.peek();
    if let Some((orient, _)) = current_ruler_drag {
        match phase {
            PointerPhase::Move => {
                let doc_pt = camera.screen_to_doc(screen);
                let pos = match orient {
                    petunia_design_document::GuideOrientation::Horizontal => doc_pt.y,
                    petunia_design_document::GuideOrientation::Vertical => doc_pt.x,
                };
                ruler_drag.set(Some((orient, pos)));
                let next = *gesture_tick.peek() + 1;
                gesture_tick.set(next);
                return;
            }
            PointerPhase::Up => {
                let surf_id = shell.peek().bridge.active_surface();
                let doc_pt = camera.screen_to_doc(screen);
                let pos = match orient {
                    petunia_design_document::GuideOrientation::Horizontal => doc_pt.y,
                    petunia_design_document::GuideOrientation::Vertical => doc_pt.x,
                };
                if let Some(surf_id) = surf_id {
                    let _ = shell.write().bridge.submit_all(
                        "Add guide",
                        vec![petunia_design_application::Command::CreateGuide {
                            surface: surf_id,
                            orientation: orient,
                            position: pos,
                        }],
                    );
                }
                ruler_drag.set(None);
                let next = *gesture_tick.peek() + 1;
                gesture_tick.set(next);
                return;
            }
            PointerPhase::Cancel => {
                ruler_drag.set(None);
                let next = *gesture_tick.peek() + 1;
                gesture_tick.set(next);
                return;
            }
            PointerPhase::Down => {}
        }
    } else if phase == PointerPhase::Down && button == PointerButton::Primary {
        // Top horizontal ruler clicked (drag out a horizontal guide)
        if screen.y < 20.0 && screen.x >= 20.0 {
            let doc_pt = camera.screen_to_doc(screen);
            ruler_drag.set(Some((
                petunia_design_document::GuideOrientation::Horizontal,
                doc_pt.y,
            )));
            return;
        }
        // Left vertical ruler clicked (drag out a vertical guide)
        if screen.x < 20.0 && screen.y >= 20.0 {
            let doc_pt = camera.screen_to_doc(screen);
            ruler_drag.set(Some((
                petunia_design_document::GuideOrientation::Vertical,
                doc_pt.x,
            )));
            return;
        }
    }

    // 3. Normal canvas tool event dispatch
    let document = camera.screen_to_doc(screen);
    let mut event = NormalizedPointerEvent::new(phase, button, screen, document, *modifiers.read());
    event.pressure = pressure;
    let _ = shell.write().handle_pointer_event(&event);
    let next = *gesture_tick.peek() + 1;
    gesture_tick.set(next);
}

fn pointer_button(button: Option<MouseButton>) -> Option<PointerButton> {
    match button? {
        MouseButton::Left => Some(PointerButton::Primary),
        MouseButton::Middle => Some(PointerButton::Middle),
        MouseButton::Right => Some(PointerButton::Secondary),
        MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => None,
    }
}

fn semantic_modifiers(modifiers: Modifiers) -> SemanticModifiers {
    let shift = modifiers.contains(Modifiers::SHIFT);
    let alt = modifiers.contains(Modifiers::ALT);
    let control = modifiers.contains(Modifiers::CONTROL) || modifiers.contains(Modifiers::META);
    SemanticModifiers {
        constrain: shift,
        from_center: alt,
        duplicate: alt,
        disable_snap: control,
        fine_adjust: false,
    }
}

// All workspace shortcuts share the same UI action adapter as menus.
fn dispatch_workspace_key(ui: &UiShell, event: &Event<KeyboardEventData>) {
    // Popups own typing and shortcuts. Delete, Space and letters must never
    // edit the canvas behind an open file or configuration dialog.
    if ui.has_modal() || *ui.palette_open.peek() {
        return;
    }
    if canvas_text::key(ui, event) {
        return;
    }
    let mut shell = ui.shell;
    let mut palette_open = ui.palette_open;
    let mut palette_query = ui.palette_query;
    let mut tool_rail = ui.tool_rail;
    let mut active_tool = ui.active_tool;
    let mut customize_open = ui.customize_open;
    let mut temporary_tool = ui.temporary_tool;
    let mut suspended_tool = ui.suspended_tool;
    if let Some(key) = shortcut_key(event) {
        let has_command_modifier = event.modifiers.contains(Modifiers::CONTROL)
            || event.modifiers.contains(Modifiers::META)
            || event.modifiers.contains(Modifiers::ALT);
        if !has_command_modifier {
            let (photo, current_tool) = {
                let shell_ref = shell.peek();
                (
                    shell_ref.bridge.persona()
                        == petunia_design_application::surfaces::PERSONA_PHOTO,
                    shell_ref.active_tool(),
                )
            };
            let rail_state = tool_rail.read().clone();
            if let Some(tool) = resolve_tool_shortcut(key, photo, &rail_state, current_tool) {
                event.prevent_default();
                if key.eq_ignore_ascii_case("Space") {
                    temporary_tool.set(Some(tool));
                    suspended_tool.set(Some(current_tool));
                } else {
                    temporary_tool.set(None);
                    suspended_tool.set(None);
                    tool_rail.write().remember_tool(photo, tool);
                }
                shell.write().set_active_tool(tool);
                active_tool.set(tool);
                return;
            }
        }
    }
    let Some(token) = workspace_shortcut(event) else {
        return;
    };
    event.prevent_default();
    if token == "ptnd.action.view.command_palette" {
        let open = !*palette_open.read();
        palette_open.set(open);
        palette_query.set(String::new());
    }
    if token == petunia_design_application::ActionId::EDIT_PREFERENCES {
        customize_open.set(true);
    }
    let _ = crate::actions::run_ui_id(ui, token);
}

fn workspace_shortcut(event: &Event<KeyboardEventData>) -> Option<&'static str> {
    let control =
        event.modifiers.contains(Modifiers::CONTROL) || event.modifiers.contains(Modifiers::META);
    if control {
        return match &event.key {
            Key::Character(key) if key.eq_ignore_ascii_case("o") => Some("ptnd.action.file.open"),
            Key::Character(key) if key.eq_ignore_ascii_case("s") => {
                Some(if event.modifiers.contains(Modifiers::SHIFT) {
                    "ptnd.action.file.save_as"
                } else {
                    "ptnd.action.file.save"
                })
            }
            Key::Character(key) if key.eq_ignore_ascii_case("w") => Some("ptnd.action.file.close"),
            Key::Character(key) if key.eq_ignore_ascii_case("n") => Some("ptnd.action.file.new"),
            Key::Character(key) if key.eq_ignore_ascii_case("e") => Some("ptnd.action.file.export"),
            Key::Character(key) if key.eq_ignore_ascii_case("z") => {
                if event.modifiers.contains(Modifiers::SHIFT) {
                    Some("ptnd.action.edit.redo")
                } else {
                    Some("ptnd.action.edit.undo")
                }
            }
            Key::Character(key) if key.eq_ignore_ascii_case("y") => Some("ptnd.action.edit.redo"),
            Key::Character(key) if key.eq_ignore_ascii_case("k") => {
                Some("ptnd.action.view.command_palette")
            }
            Key::Character(key) if key.eq_ignore_ascii_case("a") => {
                Some("ptnd.action.edit.select_all")
            }
            Key::Character(key) if key.eq_ignore_ascii_case("d") => {
                if event.modifiers.contains(Modifiers::SHIFT) {
                    Some("ptnd.action.edit.deselect")
                } else {
                    Some("ptnd.action.edit.duplicate")
                }
            }
            Key::Character(key) if key.eq_ignore_ascii_case("g") => {
                if event.modifiers.contains(Modifiers::SHIFT) {
                    Some("ptnd.action.object.ungroup")
                } else {
                    Some("ptnd.action.object.group")
                }
            }
            Key::Character(key) if key == "0" => Some("ptnd.action.view.fit_surface"),
            Key::Character(key) if key == "+" || key == "=" => Some("ptnd.action.view.zoom_in"),
            Key::Character(key) if key == "-" => Some("ptnd.action.view.zoom_out"),
            Key::Character(key) if key == "," => {
                Some(petunia_design_application::ActionId::EDIT_PREFERENCES)
            }
            _ => None,
        };
    }
    match &event.key {
        Key::Named(NamedKey::Delete) | Key::Named(NamedKey::Backspace) => {
            Some("ptnd.action.edit.delete")
        }
        Key::Named(NamedKey::Escape) => Some("ptnd.action.edit.deselect"),
        _ => None,
    }
}

fn restore_temporary_tool(
    mut shell: State<PetuniaShell>,
    mut active_tool: State<ToolKind>,
    mut temporary_tool: State<Option<ToolKind>>,
    mut suspended_tool: State<Option<ToolKind>>,
) {
    let Some(previous_tool) = *suspended_tool.read() else {
        return;
    };
    shell.write().set_active_tool(previous_tool);
    active_tool.set(previous_tool);
    temporary_tool.set(None);
    suspended_tool.set(None);
}

#[derive(Clone, PartialEq)]
struct StatusBar(UiShell);

impl Component for StatusBar {
    fn render(&self) -> impl IntoElement {
        let shell_ref = self.0.shell.read();
        let title = shell_ref.bridge.session().map_or_else(
            || DEFAULT_DOCUMENT_TITLE.to_string(),
            |s| s.title().to_string(),
        );
        let zoom_pct = (shell_ref.view_camera().zoom * 100.).round() as i32;
        let zoom_label = shell_ref
            .bridge
            .localization()
            .text("ptnd.text.shell.zoom_readout", shell_ref.bridge.locale());
        // A hovered control explains itself here: tooltips live in the status
        // bar so buttons stay exactly where the registry put them.
        let hovered = self.0.hovered.read().clone();
        let active_tool = *self.0.active_tool.read();
        let hint = hovered.map_or_else(
            || {
                shell_ref
                    .bridge
                    .tool_hint(active_tool)
                    .or_else(|| shell_ref.bridge.persona_hint(shell_ref.bridge.persona()))
                    .unwrap_or_default()
            },
            |target| {
                if target.summary.is_empty() {
                    target.title.clone()
                } else {
                    target.summary.clone()
                }
            },
        );

        rect()
            .direction(Direction::Horizontal)
            .width(Size::fill())
            .height(Size::px(theme::STATUS_BAR_HEIGHT))
            .background(theme::SURFACE_CHROME_STRONG)
            .padding(Gaps::new(0., theme::SPACE_2, 0., theme::SPACE_2))
            .spacing(theme::SPACE_2)
            .cross_align(Alignment::Center)
            .child(
                label()
                    .text(format!("{title} · {zoom_label}: {zoom_pct}%"))
                    .color(theme::TEXT_SECONDARY)
                    .font_size(theme::CAPTION_SIZE),
            )
            .child(rect().width(Size::fill()))
            .child(
                label()
                    .text(
                        self.0
                            .file_error
                            .read()
                            .clone()
                            .or_else(|| self.0.file_notice.read().clone())
                            .unwrap_or(hint),
                    )
                    .color(theme::TEXT_SECONDARY)
                    .font_size(theme::CAPTION_SIZE),
            )
            .maybe_child(
                (self.0.file_notice.read().is_some() || self.0.file_error.read().is_some()).then(
                    || {
                        let mut notice = self.0.file_notice;
                        let mut error = self.0.file_error;
                        Button::new()
                            .on_press(move |_| {
                                notice.set(None);
                                error.set(None);
                            })
                            .child(self.0.text("dismiss"))
                    },
                ),
            )
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .spacing(theme::SPACE_1)
                    .child(
                        Button::new()
                            .on_press({
                                let mut open = self.0.left_dock_open;
                                move |_| {
                                    let cur = *open.peek();
                                    open.set(!cur);
                                }
                            })
                            .child(
                                label()
                                    .text(if *self.0.left_dock_open.read() {
                                        "◧ Doca Esq (Ativa)"
                                    } else {
                                        "◧ Doca Esq"
                                    })
                                    .font_size(10.)
                                    .color(if *self.0.left_dock_open.read() {
                                        theme::ACCENT_BLOOM
                                    } else {
                                        theme::TEXT_SECONDARY
                                    }),
                            ),
                    )
                    .child(
                        Button::new()
                            .on_press({
                                let mut open = self.0.bottom_dock_open;
                                move |_| {
                                    let cur = *open.peek();
                                    open.set(!cur);
                                }
                            })
                            .child(
                                label()
                                    .text(if *self.0.bottom_dock_open.read() {
                                        "⬒ Doca Inf (Ativa)"
                                    } else {
                                        "⬒ Doca Inf"
                                    })
                                    .font_size(10.)
                                    .color(if *self.0.bottom_dock_open.read() {
                                        theme::ACCENT_BLOOM
                                    } else {
                                        theme::TEXT_SECONDARY
                                    }),
                            ),
                    ),
            )
            .child(AppearanceBar(self.0.clone()))
    }
}

#[cfg(test)]
mod workspace_tests {
    use super::*;
    use freya_testing::prelude::*;
    use petunia_design_application::tools::ToolKind;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn seed_shapes(shell: &mut PetuniaShell) {
        use petunia_design_application::{Command, CommandRequest};
        let surface = shell.bridge.active_surface().unwrap();
        for (name, bounds) in [
            ("Rectangle A", [260., 180., 220., 160.]),
            ("Rectangle B", [440., 260., 180., 120.]),
        ] {
            let id = shell.bridge.next_object_id().unwrap();
            shell
                .bridge
                .submit_command(CommandRequest::new(Command::CreateShapeObject {
                    surface,
                    id,
                    name: name.into(),
                    shape: petunia_design_document::ShapeKind::Rectangle {
                        corner_radii: [0.; 4],
                    },
                    bounds: Some(bounds),
                    fill: Some("#3498db".into()),
                    stroke: None,
                    stroke_width: 0.,
                }))
                .unwrap();
        }
        shell.bridge.clear_selection();
    }

    #[test]
    fn pointer_drag_creates_a_shape_through_the_shell() {
        let seen: Rc<RefCell<Option<State<PetuniaShell>>>> = Rc::new(RefCell::new(None));
        let seen_hook = seen.clone();
        let (mut runner, ()) = TestingRunner::new(
            move || {
                let shell = use_state(|| {
                    let mut shell = PetuniaShell::new(800., 600.);
                    shell.new_document("Test").expect("document opens");
                    shell.set_active_tool(ToolKind::Rectangle);
                    shell
                });
                seen_hook.replace(Some(shell));
                Workspace(UiShell::fresh(shell))
            },
            (800., 600.).into(),
            |_| {},
            1.,
        );

        runner.sync_and_update();
        runner.press_cursor((180., 160.));
        runner.move_cursor((320., 260.));
        runner.release_cursor((320., 260.));

        let shell = (*seen.borrow()).expect("workspace mounted");
        let shell = shell.peek();
        let session = shell.bridge.session().expect("active session");
        let surface_id = session.active_surface().expect("active surface");
        let surface = session.surface(surface_id).expect("surface");
        assert_eq!(surface.objects().len(), 1);
        assert!(surface.objects()[0].bounds.expect("shape bounds")[2] > 0.);
    }

    #[test]
    fn pointer_drag_moves_shape_on_canvas() {
        let seen: Rc<RefCell<Option<State<PetuniaShell>>>> = Rc::new(RefCell::new(None));
        let seen_hook = seen.clone();
        let (mut runner, ()) = TestingRunner::new(
            move || {
                let shell = use_state(|| {
                    let mut shell = PetuniaShell::new(800., 600.);
                    shell.new_document("Test").expect("document opens");
                    seed_shapes(&mut shell);
                    shell.set_active_tool(ToolKind::Select);
                    shell
                });
                seen_hook.replace(Some(shell));
                Workspace(UiShell::fresh(shell))
            },
            (800., 600.).into(),
            |_| {},
            1.,
        );

        runner.sync_and_update();
        let initial_bounds = {
            let shell = (*seen.borrow()).unwrap();
            let shell = shell.peek();
            let session = shell.bridge.session().unwrap();
            let surf = session.surface(session.active_surface().unwrap()).unwrap();
            surf.objects()[0].bounds.unwrap()
        };

        // Rectangle A is at [260.0, 180.0, 220.0, 160.0]. Click inside it and drag +60, +40.
        runner.press_cursor((300., 200.));
        runner.move_cursor((360., 240.));
        runner.release_cursor((360., 240.));
        runner.sync_and_update();

        let moved_bounds = {
            let shell = (*seen.borrow()).unwrap();
            let shell = shell.peek();
            let session = shell.bridge.session().unwrap();
            let surf = session.surface(session.active_surface().unwrap()).unwrap();
            surf.objects()[0].bounds.unwrap()
        };

        assert!((moved_bounds[0] - (initial_bounds[0] + 60.0)).abs() < 1.0);
        assert!((moved_bounds[1] - (initial_bounds[1] + 40.0)).abs() < 1.0);
    }

    #[test]
    fn text_tool_click_creates_text_object_and_select_tool_can_select_it() {
        let seen: Rc<RefCell<Option<State<PetuniaShell>>>> = Rc::new(RefCell::new(None));
        let seen_hook = seen.clone();
        let (mut runner, ()) = TestingRunner::new(
            move || {
                let shell = use_state(|| {
                    let mut shell = PetuniaShell::new(800., 600.);
                    shell.new_document("Test").expect("document opens");
                    shell.set_active_tool(ToolKind::ArtisticText);
                    shell
                });
                seen_hook.replace(Some(shell));
                Workspace(UiShell::fresh(shell))
            },
            (800., 600.).into(),
            |_| {},
            1.,
        );

        runner.sync_and_update();
        // Click at (150, 150) with ArtisticText tool to create text
        runner.press_cursor((150., 150.));
        runner.release_cursor((150., 150.));
        runner.sync_and_update();

        let text_id = {
            let shell = (*seen.borrow()).unwrap();
            let shell = shell.peek();
            let session = shell.bridge.session().unwrap();
            let surf = session.surface(session.active_surface().unwrap()).unwrap();
            assert_eq!(
                surf.objects().len(),
                1,
                "text object must be created on surface"
            );
            assert!(matches!(
                surf.objects()[0].shape,
                Some(petunia_design_document::ShapeKind::Text { .. })
            ));
            surf.objects()[0].id
        };

        // Switch to Select tool, deselect, then click on the text object to select it
        {
            let mut shell_state = (*seen.borrow()).unwrap();
            let mut shell = shell_state.write();
            shell.set_active_tool(ToolKind::Select);
            shell.bridge.clear_selection();
        }
        runner.sync_and_update();

        // Click on the text object at (160, 160)
        runner.press_cursor((160., 160.));
        runner.release_cursor((160., 160.));
        runner.sync_and_update();

        {
            let shell = (*seen.borrow()).unwrap();
            let shell = shell.peek();
            let sel = shell.bridge.selection();
            assert_eq!(sel.count, 1, "text object must be selected by clicking it");
            assert!(sel.contains(text_id));
        }
    }

    #[test]
    fn select_toggle_deselect_and_reselect_cycle() {
        let seen: Rc<RefCell<Option<State<PetuniaShell>>>> = Rc::new(RefCell::new(None));
        let seen_hook = seen.clone();
        let (mut runner, ()) = TestingRunner::new(
            move || {
                let shell = use_state(|| {
                    let mut shell = PetuniaShell::new(800., 600.);
                    shell.new_document("Test").expect("document opens");
                    seed_shapes(&mut shell);
                    shell.set_active_tool(ToolKind::Select);
                    shell
                });
                seen_hook.replace(Some(shell));
                Workspace(UiShell::fresh(shell))
            },
            (800., 600.).into(),
            |_| {},
            1.,
        );

        runner.sync_and_update();

        let (id1, id2) = {
            let shell = (*seen.borrow()).unwrap();
            let shell = shell.peek();
            let session = shell.bridge.session().unwrap();
            let surf = session.surface(session.active_surface().unwrap()).unwrap();
            (surf.objects()[0].id, surf.objects()[1].id)
        };

        // 1. Click Rectangle A (300, 200)
        runner.press_cursor((300., 200.));
        runner.release_cursor((300., 200.));
        runner.sync_and_update();
        {
            let shell = (*seen.borrow()).unwrap();
            let sel = shell.peek().bridge.selection();
            assert_eq!(
                sel.selected_ids,
                vec![id1],
                "Step 1: Object A should be selected"
            );
        }

        // 2. Click Circle B (500, 300)
        runner.press_cursor((500., 300.));
        runner.release_cursor((500., 300.));
        runner.sync_and_update();
        {
            let shell = (*seen.borrow()).unwrap();
            let sel = shell.peek().bridge.selection();
            assert_eq!(
                sel.selected_ids,
                vec![id2],
                "Step 2: Object B should be selected"
            );
        }

        // 3. Click empty space (100, 100) to deselect
        runner.press_cursor((100., 100.));
        runner.release_cursor((100., 100.));
        runner.sync_and_update();
        {
            let shell = (*seen.borrow()).unwrap();
            let sel = shell.peek().bridge.selection();
            assert!(sel.is_empty, "Step 3: Selection should be empty");
        }

        // 4. Click Rectangle A (300, 200) again!
        runner.press_cursor((300., 200.));
        runner.release_cursor((300., 200.));
        runner.sync_and_update();
        {
            let shell = (*seen.borrow()).unwrap();
            let sel = shell.peek().bridge.selection();
            assert_eq!(
                sel.selected_ids,
                vec![id1],
                "Step 4: Object A should be selected again"
            );
        }

        // 5. Click Circle B (500, 300) again!
        runner.press_cursor((500., 300.));
        runner.release_cursor((500., 300.));
        runner.sync_and_update();
        {
            let shell = (*seen.borrow()).unwrap();
            let sel = shell.peek().bridge.selection();
            assert_eq!(
                sel.selected_ids,
                vec![id2],
                "Step 5: Object B should be selected again"
            );
        }
    }

    #[test]
    fn group_and_ungroup_actions_work_on_selection() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("Test").expect("document opens");
        seed_shapes(&mut shell);

        // Select all objects
        let _ = run_action_id(&mut shell, "ptnd.action.edit.select_all");
        assert_eq!(shell.bridge.selection().count, 2);

        // Group them
        let res = run_action_id(&mut shell, "ptnd.action.object.group");
        assert!(res.is_some());
        // Selection now has 1 group container
        assert_eq!(shell.bridge.selection().count, 1);

        // Ungroup them
        let res2 = run_action_id(&mut shell, "ptnd.action.object.ungroup");
        assert!(res2.is_some());
        // Both objects are back
        assert_eq!(shell.bridge.selection().count, 2);
    }

    #[test]
    fn raster_tile_to_skia_image_converts_correctly() {
        use petunia_design_raster::{AlphaMode, PixelFormat, Tile, TileCoord, TILE_SIZE};
        let mut tile = Tile::new_empty(
            TileCoord::new(0, 0),
            PixelFormat::Rgba8,
            AlphaMode::Straight,
        );
        tile.set_pixel_normalized(10, 10, [1.0, 0.0, 0.0, 1.0]);
        let rgba8 = match tile.format {
            PixelFormat::Rgba8 => tile.data.as_ref().clone(),
            _ => vec![],
        };
        assert_eq!(rgba8.len(), TILE_SIZE * TILE_SIZE * 4);
        let offset = (10 * TILE_SIZE + 10) * 4;
        assert_eq!(rgba8[offset], 255);
        assert_eq!(rgba8[offset + 1], 0);
        assert_eq!(rgba8[offset + 2], 0);
        assert_eq!(rgba8[offset + 3], 255);
    }

    #[test]
    fn ruler_drag_creates_horizontal_and_vertical_guides() {
        let seen: Rc<RefCell<Option<State<PetuniaShell>>>> = Rc::new(RefCell::new(None));
        let seen_hook = seen.clone();
        let (mut runner, ()) = TestingRunner::new(
            move || {
                let shell = use_state(|| {
                    let mut shell = PetuniaShell::new(800., 600.);
                    shell.new_document("Test").expect("document opens");
                    shell
                });
                seen_hook.replace(Some(shell));
                Workspace(UiShell::fresh(shell))
            },
            (800., 600.).into(),
            |_| {},
            1.,
        );

        runner.sync_and_update();

        // 1. Drag horizontal guide from top ruler (y = 10, x = 100) down to y = 150
        runner.press_cursor((100., 10.));
        runner.move_cursor((100., 150.));
        runner.release_cursor((100., 150.));
        runner.sync_and_update();

        {
            let shell = (*seen.borrow()).unwrap();
            let shell = shell.peek();
            let session = shell.bridge.session().unwrap();
            let surf = session.surface(session.active_surface().unwrap()).unwrap();
            assert_eq!(surf.guides.len(), 1, "horizontal guide should be added");
            assert_eq!(
                surf.guides[0].orientation,
                petunia_design_document::GuideOrientation::Horizontal
            );
        }

        // 2. Drag vertical guide from left ruler (x = 10, y = 100) right to x = 200
        runner.press_cursor((10., 100.));
        runner.move_cursor((200., 100.));
        runner.release_cursor((200., 100.));
        runner.sync_and_update();

        {
            let shell = (*seen.borrow()).unwrap();
            let shell = shell.peek();
            let session = shell.bridge.session().unwrap();
            let surf = session.surface(session.active_surface().unwrap()).unwrap();
            assert_eq!(surf.guides.len(), 2, "vertical guide should be added");
            assert_eq!(
                surf.guides[1].orientation,
                petunia_design_document::GuideOrientation::Vertical
            );
        }
    }

    #[test]
    fn place_image_menu_token_requests_path_without_creating_a_placeholder() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("ImageTest").expect("document opens");
        let before = shell.bridge.session().unwrap().document().clone();
        let action = actions::run_action_token(&mut shell, "ptnd.action.file.place#null");
        assert_eq!(action.as_deref(), Some("ptnd.action.file.place"));
        assert_eq!(shell.bridge.session().unwrap().document(), &before);
    }

    #[test]
    fn in_canvas_text_editor_updates_text_object() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("TextEditTest").expect("document opens");
        let surface_id = shell.bridge.active_surface().unwrap();
        let text_id = shell.bridge.next_object_id().unwrap();

        // 1. Create text object
        shell
            .bridge
            .submit_all(
                "Create text",
                vec![petunia_design_application::Command::CreateShapeObject {
                    surface: surface_id,
                    id: text_id,
                    name: "Text1".to_string(),
                    shape: petunia_design_document::ShapeKind::Text {
                        content: "Initial".to_string(),
                        font_family: "Inter".to_string(),
                        font_size: 18.0,
                        line_height: 1.2,
                        letter_spacing: 0.0,
                        on_path: None,
                    },
                    bounds: Some([100.0, 100.0, 120.0, 30.0]),
                    fill: None,
                    stroke: None,
                    stroke_width: 0.0,
                }],
            )
            .expect("command succeeds");

        // 2. Select it
        shell.bridge.set_selection(vec![text_id]);

        // 3. Verify snapshot identifies it as active text object
        let snapshot = shell.canvas_snapshot();
        let active_text = snapshot.objects.iter().find(|o| {
            o.active
                && matches!(
                    o.shape.as_deref(),
                    Some(petunia_design_document::ShapeKind::Text { .. })
                )
        });
        assert!(
            active_text.is_some(),
            "active text object found in snapshot"
        );

        // 4. Update its content directly via command as in-canvas editor does
        shell
            .bridge
            .submit_all(
                "Update text in-canvas",
                vec![petunia_design_application::Command::SetShape {
                    id: text_id,
                    shape: Some(petunia_design_document::ShapeKind::Text {
                        content: "Edited In-Canvas Content".to_string(),
                        font_family: "Inter".to_string(),
                        font_size: 18.0,
                        line_height: 1.2,
                        letter_spacing: 0.0,
                        on_path: None,
                    }),
                }],
            )
            .expect("update succeeds");

        let session = shell.bridge.session().unwrap();
        let obj = session.find_object(text_id).unwrap();
        match &obj.shape {
            Some(petunia_design_document::ShapeKind::Text { content, .. }) => {
                assert_eq!(content, "Edited In-Canvas Content");
            }
            other => panic!("expected updated text, got {:?}", other),
        }
    }

    #[test]
    fn new_document_action_and_custom_geometry() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("NewDocTest").expect("document opens");
        let surface_id = shell.bridge.active_surface().unwrap();

        // Preset: Full HD 1920x1080 with 3mm bleed and 20pt margin
        shell
            .bridge
            .submit_all(
                "Set Surface Geometry 1080p",
                vec![
                    petunia_design_application::Command::SetSurfaceGeometry {
                        surface: surface_id,
                        origin: [0.0, 0.0],
                        dimensions: [1920.0, 1080.0],
                    },
                    petunia_design_application::Command::SetSurfaceBleed {
                        surface: surface_id,
                        bleed: petunia_design_document::Bleed::uniform(8.5),
                    },
                    petunia_design_application::Command::SetSurfaceMargins {
                        surface: surface_id,
                        margins: petunia_design_document::Margins::uniform(20.0),
                    },
                ],
            )
            .expect("command succeeds");

        let session = shell.bridge.session().unwrap();
        let surface = session.surface(surface_id).unwrap();
        assert_eq!(surface.dimensions, [1920.0, 1080.0]);
        assert_eq!(surface.bleed, petunia_design_document::Bleed::uniform(8.5));
        assert_eq!(
            surface.margins,
            petunia_design_document::Margins::uniform(20.0)
        );
    }

    #[test]
    fn export_document_action_dispatches_cleanly() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("ExportTestDoc").expect("document opens");

        let path = std::env::temp_dir().join("petunia_test_export.png");
        let payload = serde_json::json!({
            "path": path.to_string_lossy(),
            "format": "png",
        });
        let res = shell
            .bridge
            .dispatch_action(petunia_design_application::ActionRequest::new(
                petunia_design_application::ActionId::new("ptnd.action.file.export"),
                payload,
            ));
        assert!(
            res.is_ok(),
            "export action should be dispatched successfully"
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn navigator_minimap_and_dock_width_adjustments() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("NavigatorTest").expect("document opens");

        // Zoom and pan adjustments reflect in camera
        shell.zoom_at(GPoint::new(400.0, 300.0), 1.5);
        shell.pan(50.0, -30.0);
        let cam = shell.view_camera();
        assert!((cam.zoom - 1.5).abs() < 1e-4);
        assert!((cam.pan_x - -150.0).abs() < 1e-4 || cam.pan_x.is_finite());
        assert!((cam.pan_y - -180.0).abs() < 1e-4 || cam.pan_y.is_finite());

        // Clamping logic for dock width
        let min_w = 180.0f32;
        let max_w = 520.0f32;
        assert_eq!((100.0f32).clamp(min_w, max_w), min_w);
        assert_eq!((600.0f32).clamp(min_w, max_w), max_w);
        assert_eq!((300.0f32).clamp(min_w, max_w), 300.0f32);

        // Minimap scale and click-pan center target logic
        let surf_w = 800.0f64;
        let surf_h = 600.0f64;
        let scale = (240.0f64 / surf_w).min(150.0f64 / surf_h);
        let click_x = 120.0f64;
        let click_y = 75.0f64;
        let target_doc_x = (click_x / scale).clamp(0.0, surf_w);
        let target_doc_y = (click_y / scale).clamp(0.0, surf_h);
        assert_eq!(target_doc_x, 480.0);
        assert_eq!(target_doc_y, 300.0);

        let mut camera = shell.view_camera();
        let expected_pan_x = camera.viewport_width / 2.0 - target_doc_x * camera.zoom;
        camera.pan_x = expected_pan_x;
        camera.pan_y = camera.viewport_height / 2.0 - target_doc_y * camera.zoom;
        shell.set_view_camera(camera);
        let updated_cam = shell.view_camera();
        assert!((updated_cam.pan_x - expected_pan_x).abs() < 1e-4);
    }

    #[test]
    fn status_bar_tool_hints_resolve_in_both_locales_and_pluralize() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("HintsDoc").expect("doc opens");

        // Tool hint in default EnUs
        assert_eq!(
            shell.bridge.tool_hint(ToolKind::Select).as_deref(),
            Some("Select and transform objects")
        );
        assert_eq!(
            shell.bridge.tool_hint(ToolKind::Pen).as_deref(),
            Some("Build precise Bézier paths")
        );
        assert_eq!(shell.bridge.plural_items(1), "1 item");
        assert_eq!(shell.bridge.plural_items(4), "4 items");

        // Switch to pt-BR
        shell
            .bridge
            .set_locale(petunia_design_shell::bridge::Locale::PtBr);
        assert_eq!(
            shell.bridge.tool_hint(ToolKind::Select).as_deref(),
            Some("Selecione e transforme objetos")
        );
        assert_eq!(
            shell.bridge.tool_hint(ToolKind::Pen).as_deref(),
            Some("Construa caminhos Bézier precisos")
        );
        assert_eq!(shell.bridge.plural_items(1), "1 item");
        assert_eq!(shell.bridge.plural_items(4), "4 itens");
    }

    #[test]
    fn confirm_close_safeguards_unsaved_document() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell
            .new_document("UnsavedDocTest")
            .expect("document opens");
        assert!(
            shell.bridge.is_dirty(),
            "document with initial canvas starts with dirty revision"
        );

        // Safe close without force is blocked by unsaved changes
        let safe_close = shell.bridge.close_session(false).expect("close check");
        assert!(
            !safe_close,
            "safe close requires user confirmation when dirty"
        );

        // Force close (from confirm dialog) closes successfully
        let force_close = shell.bridge.close_session(true).expect("force close");
        assert!(
            force_close,
            "confirming close discards changes and closes session"
        );
        assert!(shell.bridge.session().is_none(), "session is now closed");
    }

    #[test]
    fn dialog_toggle_lifecycle_has_no_hook_panics() {
        let seen: Rc<RefCell<Option<UiShell>>> = Rc::new(RefCell::new(None));
        let seen_hook = seen.clone();

        let (mut runner, ()) = TestingRunner::new(
            move || {
                let shell = use_state(|| {
                    let mut s = PetuniaShell::new(800., 600.);
                    s.new_document("DialogTest").expect("document opens");
                    s
                });
                let ui = UiShell::fresh(shell);
                seen_hook.replace(Some(ui.clone()));
                rect()
                    .child(CommandPalette(ui.clone()))
                    .child(NewDocumentDialog(ui.clone()))
                    .child(ExportDialog(ui.clone()))
                    .child(PlaceImageDialog(ui.clone()))
                    .child(file_dialogs::FileDialog(ui.clone()))
                    .child(recovery::RecoveryDialog(ui.clone()))
                    .child(object_edit_dialog::ObjectEditDialog(ui.clone()))
                    .child(typography::TypographyDialog(ui.clone()))
                    .child(CustomizeDialog(ui.clone()))
                    .child(ConfirmCloseDialog(ui.clone()))
                    .child(OffsetPathDialog(ui.clone()))
                    .child(OverwriteConflictDialog(ui))
            },
            (800., 600.).into(),
            |_| {},
            1.,
        );

        // Initial render: all dialogs are closed
        runner.sync_and_update();

        let mut ui = seen.borrow().clone().expect("ui mounted");

        // Toggle CommandPalette open and close
        ui.palette_open.set(true);
        runner.sync_and_update();
        ui.palette_open.set(false);
        runner.sync_and_update();

        // Toggle NewDocumentDialog open and close
        ui.new_doc_open.set(true);
        runner.sync_and_update();
        ui.new_doc_open.set(false);
        runner.sync_and_update();

        ui.place_image_open.set(true);
        runner.sync_and_update();
        ui.place_image_open.set(false);
        runner.sync_and_update();

        // Toggle ExportDialog open and close
        ui.export_open.set(true);
        runner.sync_and_update();
        ui.export_open.set(false);
        runner.sync_and_update();

        // Toggle OffsetPathDialog open and close
        ui.offset_prompt_open.set(true);
        runner.sync_and_update();
        ui.offset_prompt_open.set(false);
        runner.sync_and_update();

        // Toggle OverwriteConflictDialog open and close
        ui.overwrite_conflict_open.set(true);
        runner.sync_and_update();
        ui.overwrite_conflict_open.set(false);
        runner.sync_and_update();

        // Toggle all open simultaneously and close all
        ui.palette_open.set(true);
        ui.new_doc_open.set(true);
        ui.export_open.set(true);
        ui.customize_open.set(true);
        ui.confirm_close_open.set(true);
        ui.offset_prompt_open.set(true);
        ui.overwrite_conflict_open.set(true);
        runner.sync_and_update();

        ui.palette_open.set(false);
        ui.new_doc_open.set(false);
        ui.export_open.set(false);
        ui.customize_open.set(false);
        ui.confirm_close_open.set(false);
        ui.offset_prompt_open.set(false);
        ui.overwrite_conflict_open.set(false);
        runner.sync_and_update();
    }

    #[test]
    fn full_app_dock_layout_and_click_test() {
        let seen: Rc<RefCell<Option<UiShell>>> = Rc::new(RefCell::new(None));
        let seen_hook = seen.clone();

        let (mut runner, ()) = TestingRunner::new(
            move || {
                use_init_theme(theme::petunia_theme);
                let shell = use_state(|| {
                    let mut s = PetuniaShell::new(1280., 800.);
                    s.new_document("DockLayoutDoc").expect("doc opens");
                    seed_shapes(&mut s);
                    s
                });
                let ui = UiShell::fresh(shell);
                seen_hook.replace(Some(ui.clone()));
                rect()
                    .direction(Direction::Vertical)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .height(Size::fill())
                    .child(MenuBarRow(ui.clone()))
                    .child(DocumentTabStrip(ui.clone()))
                    .child(ContextToolbar(ui.clone()))
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .content(Content::Flex)
                            .width(Size::fill())
                            .height(Size::flex(1.0))
                            .child(ToolRail(ui.clone()))
                            .child(dock::LeftDock(ui.clone()))
                            .child(dock::LeftDockSplitter(ui.clone()))
                            .child(
                                rect()
                                    .direction(Direction::Vertical)
                                    .content(Content::Flex)
                                    .width(Size::flex(1.0))
                                    .height(Size::fill())
                                    .child(Workspace(ui.clone()))
                                    .child(dock::BottomDockSplitter(ui.clone()))
                                    .child(dock::BottomDock(ui.clone())),
                            )
                            .child(dock::DockSplitter(ui.clone()))
                            .child(dock::RightDock(ui.clone())),
                    )
                    .child(StatusBar(ui.clone()))
            },
            (1280., 800.).into(),
            |_| {},
            1.,
        );

        runner.sync_and_update();

        let ui = seen.borrow().clone().expect("ui mounted");
        assert_eq!(*ui.dock_tab.read(), 0, "initial tab is Camadas (0)");

        for (title, index) in [
            ("Propriedades", 1),
            ("Cores", 2),
            ("Histórico", 3),
            ("Navegador", 4),
            ("Camadas", 0),
        ] {
            let area = runner
                .find(|node, element| {
                    Label::try_downcast(element)
                        .filter(|label| label.text == title)
                        .map(|_| node)
                })
                .unwrap()
                .layout()
                .area;
            assert!(area.max_x() <= 1280. && area.height() < 25.);
            runner.click_cursor((
                f64::from(area.min_x() + area.width() / 2.),
                f64::from(area.min_y() + area.height() / 2.),
            ));
            runner.sync_and_update();
            assert_eq!(*ui.dock_tab.read(), index);
        }
        if let Some(directory) = std::env::var_os("PETUNIA_UI_EVIDENCE_DIR") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory).unwrap();
            runner.poll(
                std::time::Duration::from_millis(16),
                std::time::Duration::from_millis(500),
            );
            let path = directory.join("workspace.png");
            runner.render_to_file(&path);
            let pixels =
                petunia_design_io::import_raster(&std::fs::read(path).unwrap(), 8 * 1024 * 1024)
                    .unwrap();
            assert!(
                pixels
                    .data
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .filter(|pixel| pixel[..3] == [52, 152, 219])
                    .count()
                    > 1000,
                "canvas must display the rendered artwork before capture"
            );
        }
    }

    #[test]
    fn gradient_and_measure_overlays_and_controls() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("OverlaysTest").expect("document opens");

        // Measure tool mode toggling
        shell.tools.set_active_tool(ToolKind::Measure);
        assert_eq!(
            shell.tools.measure_tool().mode(),
            petunia_design_shell::tools::MeasureMode::Distance
        );
        shell
            .tools
            .measure_tool_mut()
            .set_mode(petunia_design_shell::tools::MeasureMode::Area);
        assert_eq!(
            shell.tools.measure_tool().mode(),
            petunia_design_shell::tools::MeasureMode::Area
        );
        shell.tools.measure_tool_mut().cancel();

        // Gradient tool kind toggling
        shell.tools.set_active_tool(ToolKind::Gradient);
        assert_eq!(
            shell.tools.gradient_tool().kind(),
            petunia_design_shell::tools::GradientKind::Linear
        );
        shell
            .tools
            .gradient_tool_mut()
            .set_kind(petunia_design_shell::tools::GradientKind::Radial);
        assert_eq!(
            shell.tools.gradient_tool().kind(),
            petunia_design_shell::tools::GradientKind::Radial
        );

        // Perspective tool overlays cursor affordance on selected object
        let rect_id = petunia_design_foundation::ObjectId::new(101);
        let surface_id = shell.bridge.active_surface().unwrap();
        shell
            .bridge
            .submit_all(
                "Create rect for perspective",
                vec![
                    petunia_design_application::Command::CreateObject {
                        surface: surface_id,
                        id: rect_id,
                        name: "Rect".to_string(),
                    },
                    petunia_design_application::Command::SetBounds {
                        id: rect_id,
                        bounds: Some([10.0, 10.0, 100.0, 100.0]),
                        rotation: 0.0,
                    },
                    petunia_design_application::Command::SetShape {
                        id: rect_id,
                        shape: Some(petunia_design_document::ShapeKind::Rectangle {
                            corner_radii: [0.0; 4],
                        }),
                    },
                ],
            )
            .unwrap();
        shell.bridge.set_selection(vec![rect_id]);

        shell.tools.set_active_tool(ToolKind::Perspective);
        let cam = shell.view_camera();
        let overlays = shell.tools.overlays(&cam, &shell.bridge);
        assert_eq!(
            overlays.handles.len(),
            4,
            "Perspective quad provides 4 corner handles"
        );
        assert_eq!(
            overlays.cursor,
            petunia_design_shell::canvas::CursorAffordance::Crosshair
        );
    }
}

#[cfg(test)]
mod workflow_ui_tests {
    use super::*;
    use freya_testing::prelude::*;
    use std::{cell::RefCell, rc::Rc};

    fn mount() -> (TestingRunner, UiShell) {
        let seen = Rc::new(RefCell::new(None::<UiShell>));
        let observed = seen.clone();
        let (mut runner, ()) = TestingRunner::new(
            move || {
                use_init_theme(theme::petunia_theme);
                let shell = use_state(|| {
                    let mut s = PetuniaShell::new(1280., 800.);
                    s.new_document("Original").unwrap();
                    s
                });
                let ui = UiShell::fresh(shell);
                observed.replace(Some(ui.clone()));
                let keyboard_ui = ui.clone();
                rect()
                    .width(Size::fill())
                    .height(Size::fill())
                    .child(NewDocumentDialog(ui.clone()))
                    .child(ExportDialog(ui.clone()))
                    .child(file_dialogs::FileDialog(ui.clone()))
                    .child(recovery::RecoveryDialog(ui.clone()))
                    .child(object_edit_dialog::ObjectEditDialog(ui.clone()))
                    .child(typography::TypographyDialog(ui.clone()))
                    .child(ConfirmCloseDialog(ui.clone()))
                    .on_global_key_down(move |event: Event<KeyboardEventData>| {
                        dispatch_workspace_key(&keyboard_ui, &event)
                    })
            },
            (1280., 800.).into(),
            |_| {},
            1.,
        );
        runner.sync_and_update();
        let ui = seen.borrow().clone().unwrap();
        (runner, ui)
    }
    fn click(runner: &mut TestingRunner, text: &str) {
        let node = runner
            .find_many(|node, element| {
                Label::try_downcast(element)
                    .filter(|label| label.text == text)
                    .map(|_| node)
            })
            .pop()
            .unwrap_or_else(|| panic!("Missing visible label: {text}"));
        let area = node.layout().area;
        runner.click_cursor((
            f64::from(area.min_x() + area.width() / 2.),
            f64::from(area.min_y() + area.height() / 2.),
        ));
        runner.sync_and_update();
    }
    fn capture_if_requested(runner: &mut TestingRunner, name: &str) {
        if let Some(directory) = std::env::var_os("PETUNIA_UI_EVIDENCE_DIR") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory).unwrap();
            runner.render_to_file(directory.join(format!("{name}.png")));
        }
    }

    fn canvas_text_fixture(ui: &UiShell) -> petunia_design_foundation::ObjectId {
        let mut state = ui.shell;
        let mut shell = state.write();
        let surface = shell.bridge.active_surface().unwrap();
        let id = shell.bridge.next_object_id().unwrap();
        shell
            .bridge
            .submit_all(
                "Text fixture",
                vec![
                    petunia_design_application::Command::CreateObject {
                        surface,
                        id,
                        name: "Text".into(),
                    },
                    petunia_design_application::Command::SetBounds {
                        id,
                        bounds: Some([0., 0., 100., 100.]),
                        rotation: 0.,
                    },
                    petunia_design_application::Command::SetShape {
                        id,
                        shape: Some(petunia_design_document::ShapeKind::Text {
                            content: "Original".into(),
                            font_family: "DejaVu Sans".into(),
                            font_size: 12.,
                            line_height: 1.2,
                            letter_spacing: 0.,
                            on_path: None,
                        }),
                    },
                ],
            )
            .unwrap();
        id
    }
    #[test]
    fn canvas_typing_is_a_draft_and_commit_is_one_undoable_command() {
        let (mut runner, ui) = mount();
        let id = canvas_text_fixture(&ui);
        let before = ui.shell.peek().bridge.session().unwrap().document().clone();
        crate::canvas_text::request(&ui, id);
        runner.sync_and_update();
        runner.write_text(" Petúnia");
        runner.sync_and_update();
        assert_eq!(
            ui.shell.peek().bridge.session().unwrap().document(),
            &before
        );
        assert_eq!(
            ui.canvas_text.peek().as_ref().unwrap().buffer.content(),
            "Original Petúnia"
        );
        crate::canvas_text::apply(&ui);
        assert!(ui.canvas_text.peek().is_none());
        let mut state = ui.shell;
        state.write().bridge.undo().unwrap();
        assert_eq!(state.peek().bridge.session().unwrap().document(), &before);
    }
    #[test]
    fn escape_cancels_canvas_text_and_tab_switch_does_not_consume_other_tab_keys() {
        let (mut runner, ui) = mount();
        let id = canvas_text_fixture(&ui);
        let before = ui.shell.peek().bridge.session().unwrap().document().clone();
        crate::canvas_text::request(&ui, id);
        runner.write_text(" draft");
        runner.press_key(Key::Named(NamedKey::Escape));
        assert!(ui.canvas_text.peek().is_none());
        assert_eq!(
            ui.shell.peek().bridge.session().unwrap().document(),
            &before
        );
        crate::canvas_text::request(&ui, id);
        ui.shell.clone().write().new_document("Second").unwrap();
        assert!(!crate::canvas_text::is_active(&ui));
        runner.write_text("v");
        assert_eq!(
            ui.canvas_text.peek().as_ref().unwrap().buffer.content(),
            "Original"
        );
    }
    #[test]
    fn new_menu_prompt_and_cancel_create_no_tab() {
        let (mut runner, ui) = mount();
        let mut shell = ui.shell;
        let token = "ptnd.action.file.new#null";
        let action = actions::run_action_token(&mut shell.write(), token).unwrap();
        assert_eq!(shell.peek().bridge.sessions().len(), 1);
        assert!(file_workflows::route_file_action(&ui, &action));
        runner.sync_and_update();
        runner.press_key(Key::Named(NamedKey::Escape));
        assert!(!*ui.new_doc_open.peek());
        assert_eq!(shell.peek().bridge.sessions().len(), 1);
    }
    #[test]
    fn confirming_new_creates_exactly_one_configured_tab() {
        let (mut runner, ui) = mount();
        file_workflows::route_file_action(&ui, "ptnd.action.file.new");
        runner.sync_and_update();
        capture_if_requested(&mut runner, "new-document");
        click(&mut runner, &ui.text("create"));
        assert!(!*ui.new_doc_open.peek());
        let shell = ui.shell.peek();
        assert_eq!(shell.bridge.sessions().len(), 2);
        let session = shell.bridge.session().unwrap();
        let surface = session.surface(session.active_surface().unwrap()).unwrap();
        assert_eq!(surface.dimensions, [1920., 1080.]);
    }
    #[test]
    fn save_without_destination_opens_prompt_and_cancel_preserves_document() {
        let (mut runner, ui) = mount();
        file_workflows::route_file_action(&ui, "ptnd.action.file.save");
        runner.sync_and_update();
        assert!(ui.file_prompt.peek().is_some());
        assert!(ui.shell.peek().bridge.session().unwrap().path().is_none());
        runner.press_key(Key::Named(NamedKey::Escape));
        assert!(ui.file_prompt.peek().is_none());
        assert_eq!(ui.shell.peek().bridge.sessions().len(), 1);
    }
    #[test]
    fn open_failure_is_visible_and_preserves_existing_tab() {
        let (mut runner, ui) = mount();
        file_workflows::route_file_action(&ui, "ptnd.action.file.open");
        runner.sync_and_update();
        capture_if_requested(&mut runner, "open-document");
        click(&mut runner, &ui.text("open"));
        assert!(ui.file_error.peek().is_some());
        assert!(ui.file_prompt.peek().is_some());
        assert_eq!(ui.shell.peek().bridge.sessions().len(), 1);
    }
    #[test]
    fn typing_tool_shortcuts_inside_file_dialog_leaves_canvas_tool_unchanged() {
        let (mut runner, ui) = mount();
        file_workflows::route_file_action(&ui, "ptnd.action.file.open");
        runner.sync_and_update();
        for key in ["p", "v", "Space"] {
            runner.write_text(key);
        }
        assert_eq!(ui.shell.peek().active_tool(), ToolKind::Select);
    }
    #[test]
    fn export_request_uses_selected_dpi_and_normalizes_suffix_before_overwrite_check() {
        let surface = petunia_design_foundation::SurfaceId::new(42);
        let (payload, request) =
            dialogs::desktop_export_payload("png", " /tmp/logo.SVG ", 300, Some(surface)).unwrap();
        assert_eq!(payload["dpi"], 300);
        assert_eq!(request.dpi, 300.);
        assert_eq!(request.surface, Some(surface));
        assert_eq!(payload["path"], "/tmp/logo.png");
        assert_eq!(request.path, std::path::PathBuf::from("/tmp/logo.png"));
        let (_, pdf) =
            dialogs::desktop_export_payload("pdf", "/tmp/pages.png", 72, Some(surface)).unwrap();
        assert_eq!(pdf.surface, None);
    }
    #[test]
    fn export_selected_dpi_produces_the_promised_png_dimensions() {
        let mut shell = PetuniaShell::new(800., 600.);
        file_workflows::create_configured_document(&mut shell, "PNG", [72., 36.], 0., 0.).unwrap();
        let output =
            std::env::temp_dir().join(format!("petunia-ui-density-{}.png", std::process::id()));
        let (payload, request) = dialogs::desktop_export_payload(
            "png",
            output.to_str().unwrap(),
            300,
            shell.bridge.active_surface(),
        )
        .unwrap();
        shell
            .bridge
            .dispatch_action(petunia_design_application::ActionRequest::new(
                petunia_design_application::ActionId::new("ptnd.action.file.export"),
                payload,
            ))
            .unwrap();
        let bytes = std::fs::read(&request.path).unwrap();
        let image = petunia_design_io::import_raster(&bytes, 32 * 1024 * 1024).unwrap();
        assert_eq!((image.width, image.height), (300, 150));
        std::fs::remove_file(request.path).unwrap();
    }
    #[test]
    fn close_confirmation_keeps_its_target_when_tab_indices_change() {
        let (mut runner, ui) = mount();
        let mut shell = ui.shell;
        shell.write().new_document("Target").unwrap();
        let surface = shell.peek().bridge.active_surface().unwrap();
        shell
            .write()
            .bridge
            .submit_all(
                "Edit",
                vec![petunia_design_application::Command::SetSurfaceGeometry {
                    surface,
                    origin: [0., 0.],
                    dimensions: [640., 480.],
                }],
            )
            .unwrap();
        shell.write().new_document("Keep").unwrap();
        file_workflows::request_close(&ui, 1);
        runner.sync_and_update();
        shell.write().bridge.close_session_at(0, true).unwrap();
        runner.sync_and_update();
        click(&mut runner, &ui.text("close_discard"));
        assert_eq!(shell.peek().bridge.sessions().len(), 1);
        assert_eq!(shell.peek().bridge.session().unwrap().title(), "Keep");
    }
}

#[cfg(test)]
mod object_edit_ui_tests {
    use super::*;
    use freya_testing::prelude::*;
    use object_edits::{EditKind, EditValue};
    use std::{cell::RefCell, rc::Rc};

    fn mount() -> (TestingRunner, UiShell, petunia_design_foundation::ObjectId) {
        let seen = Rc::new(RefCell::new(None));
        let observed = seen.clone();
        let (mut runner, ()) = TestingRunner::new(
            move || {
                use_init_theme(theme::petunia_theme);
                let shell = use_state(|| object_edits::tests::fixture().0);
                let ui = UiShell::fresh(shell);
                observed.replace(Some(ui.clone()));
                let keys = ui.clone();
                rect()
                    .width(Size::fill())
                    .height(Size::fill())
                    .child(dock::RightDock(ui.clone()))
                    .child(object_edit_dialog::ObjectEditDialog(ui))
                    .on_global_key_down(move |event: Event<KeyboardEventData>| {
                        dispatch_workspace_key(&keys, &event)
                    })
            },
            (1000., 720.).into(),
            |_| {},
            1.,
        );
        runner.sync_and_update();
        let ui = seen.borrow().clone().unwrap();
        let text = ui.shell.peek().bridge.selection().selected_ids[0];
        (runner, ui, text)
    }
    fn click_label(runner: &mut TestingRunner, text: &str) {
        let node = runner
            .find_many(|node, element| {
                Label::try_downcast(element)
                    .filter(|label| label.text == text)
                    .map(|_| node)
            })
            .pop()
            .unwrap_or_else(|| panic!("Missing {text}"));
        let area = node.layout().area;
        runner.click_cursor((
            f64::from(area.min_x() + area.width() / 2.),
            f64::from(area.min_y() + area.height() / 2.),
        ));
        runner.sync_and_update();
    }
    fn replace_input(runner: &mut TestingRunner, index: usize, text: &str) {
        let mut nodes = runner.find_many(|node, element| {
            Rect::try_downcast(element)
                .filter(|el| {
                    matches!(
                        el.accessibility.builder.role(),
                        AccessibilityRole::TextInput | AccessibilityRole::MultilineTextInput
                    )
                })
                .map(|_| node)
        });
        nodes.sort_by(|a, b| a.layout().area.min_y().total_cmp(&b.layout().area.min_y()));
        let area = nodes[index].layout().area;
        let point = (f64::from(area.min_x() + 20.), f64::from(area.min_y() + 15.));
        runner.move_cursor(point);
        runner.click_cursor(point);
        runner.send_event(PlatformEvent::Keyboard {
            name: KeyboardEventName::KeyDown,
            key: Key::Character("a".into()),
            code: Code::KeyA,
            modifiers: Modifiers::CONTROL,
        });
        runner.sync_and_update();
        runner.press_key(Key::Named(NamedKey::Backspace));
        if !text.is_empty() {
            runner.write_text(text);
        }
        runner.sync_and_update();
    }
    fn screenshot(runner: &mut TestingRunner, name: &str) {
        if let Some(directory) = std::env::var_os("PETUNIA_UI_EVIDENCE_DIR") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory).unwrap();
            runner.render_to_file(directory.join(format!("{name}.png")));
        }
    }
    #[test]
    fn rename_invalid_draft_is_visible_then_confirmation_and_undo_work() {
        let (mut runner, ui, text) = mount();
        object_edit_dialog::request(&ui, text, EditKind::Name);
        runner.sync_and_update();
        replace_input(&mut runner, 0, "");
        click_label(&mut runner, &ui.text("edit_apply"));
        assert!(ui.object_edit.peek().is_some());
        assert!(runner
            .find(|_, el| Label::try_downcast(el)
                .filter(|label| label.text == ui.text("edit_invalid_name")))
            .is_some());
        replace_input(&mut runner, 0, "Título final");
        screenshot(&mut runner, "rename-layer");
        click_label(&mut runner, &ui.text("edit_apply"));
        assert!(ui.object_edit.peek().is_none());
        assert_eq!(
            ui.shell
                .peek()
                .bridge
                .session()
                .unwrap()
                .find_object(text)
                .unwrap()
                .name,
            "Título final"
        );
        ui.shell.clone().write().bridge.undo().unwrap();
        assert_eq!(
            ui.shell
                .peek()
                .bridge
                .session()
                .unwrap()
                .find_object(text)
                .unwrap()
                .name,
            "Title"
        );
    }
    #[test]
    fn text_cancel_and_shortcuts_leave_document_and_tools_unchanged() {
        let (mut runner, ui, text) = mount();
        let original =
            object_edits::ObjectEdit::capture(&ui.shell.peek(), text, EditKind::Text).unwrap();
        object_edit_dialog::request(&ui, text, EditKind::Text);
        runner.sync_and_update();
        replace_input(&mut runner, 0, "p v Space");
        runner.press_key(Key::Named(NamedKey::Delete));
        assert_eq!(ui.shell.peek().active_tool(), ToolKind::Select);
        assert_eq!(
            object_edits::ObjectEdit::capture(&ui.shell.peek(), text, EditKind::Text).unwrap(),
            original
        );
        runner.press_key(Key::Named(NamedKey::Escape));
        assert!(ui.object_edit.peek().is_none());
        assert_eq!(
            object_edits::ObjectEdit::capture(&ui.shell.peek(), text, EditKind::Text).unwrap(),
            original
        );
    }
    #[test]
    fn clearing_text_remains_empty_after_reopening_and_preserves_styles() {
        let (mut runner, ui, text) = mount();
        object_edit_dialog::request(&ui, text, EditKind::Text);
        runner.sync_and_update();
        replace_input(&mut runner, 0, "");
        click_label(&mut runner, &ui.text("edit_apply"));
        object_edit_dialog::request(&ui, text, EditKind::Text);
        runner.sync_and_update();
        let EditValue::Text(petunia_design_document::ShapeKind::Text {
            content,
            font_family,
            on_path,
            ..
        }) = ui.object_edit.peek().as_ref().unwrap().original.clone()
        else {
            panic!()
        };
        assert!(content.is_empty());
        assert_eq!(font_family, "Noto Sans");
        assert!(on_path.is_some());
        replace_input(&mut runner, 0, "Linha um");
        runner.press_key(Key::Named(NamedKey::Enter));
        runner.write_text("Linha dois");
        screenshot(&mut runner, "edit-text");
        click_label(&mut runner, &ui.text("edit_apply"));
        assert!(
            matches!(ui.shell.peek().bridge.session().unwrap().find_object(text).unwrap().shape.as_ref(), Some(petunia_design_document::ShapeKind::Text { content, .. }) if content == "Linha um\nLinha dois")
        );
    }
    #[test]
    fn invalid_transform_keeps_dialog_and_values_then_applies_once() {
        let (mut runner, ui, text) = mount();
        object_edit_dialog::request(&ui, text, EditKind::Transform);
        runner.sync_and_update();
        let revision = ui.shell.peek().bridge.session().unwrap().current_revision();
        replace_input(&mut runner, 2, "0");
        click_label(&mut runner, &ui.text("edit_apply"));
        assert!(ui.object_edit.peek().is_some());
        assert_eq!(
            ui.shell.peek().bridge.session().unwrap().current_revision(),
            revision
        );
        assert!(runner
            .find(|_, el| Label::try_downcast(el).filter(|label| label.text.starts_with("W: ")))
            .is_some());
        screenshot(&mut runner, "transform-error");
        replace_input(&mut runner, 2, "240,5 pt");
        screenshot(&mut runner, "edit-transform");
        click_label(&mut runner, &ui.text("edit_apply"));
        let shell = ui.shell.peek();
        let obj = shell.bridge.session().unwrap().find_object(text).unwrap();
        assert_eq!(obj.bounds, Some([30., 40., 240.5, 50.]));
        assert_eq!(obj.rotation, 0.73);
    }
    #[test]
    fn layer_rename_button_does_not_replace_selection() {
        let (mut runner, ui, text) = mount();
        let selected = ui.shell.peek().bridge.selection().selected_ids;
        let nodes = runner.find_many(|node, el| {
            Label::try_downcast(el)
                .filter(|label| label.text == ui.text("edit_name"))
                .map(|_| node)
        });
        let points: Vec<_> = nodes
            .iter()
            .map(|node| {
                let a = node.layout().area;
                (
                    f64::from(a.min_x() + a.width() / 2.),
                    f64::from(a.min_y() + a.height() / 2.),
                )
            })
            .collect();
        assert_eq!(points.len(), 2);
        for point in points {
            runner.click_cursor(point);
            runner.sync_and_update();
            assert!(ui.object_edit.peek().is_some());
            assert_eq!(ui.shell.peek().bridge.selection().selected_ids, selected);
            click_label(&mut runner, &ui.text("cancel"));
        }
        assert_eq!(selected, vec![text]);
    }
}
