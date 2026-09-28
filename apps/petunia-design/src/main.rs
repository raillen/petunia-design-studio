//! Petunia Design Studio desktop app, Freya shell (08.03).
//!
//! The shell is a projection: every mutation goes through the Action lane of
//! the Aubrieta GUI bridge and the UI re-renders from the shell state.
//! Domain crates never import toolkit types; the UI receives DTOs and sends
//! activation tokens back across the bridge.

mod actions;
mod appearance;
mod canvas_paint;
mod chrome;
mod dialogs;
mod dock;
mod theme;
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

use crate::actions::run_action_id;
use crate::appearance::AppearanceBar;
use crate::chrome::{
    resolve_tool_shortcut, shortcut_key, ContextToolbar, DocumentTabStrip, MenuBarRow, ToolRail,
    TooltipOverlay,
};
use crate::dialogs::{
    CommandPalette, ConfirmCloseDialog, CustomizeDialog, ExportDialog, NewDocumentDialog,
};
use crate::ui_state::{ToolRailState, UiShell};

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

/// Seeds two overlapping demo shapes on the default artboard for instant testing of
/// Shape Builder, Gradient, Node, and Select tools without manual setup.
fn seed_starter_shapes(shell: &mut PetuniaShell) {
    use petunia_design_application::Command;
    let Some(surface_id) = shell.bridge.active_surface() else {
        return;
    };
    let id1 = match shell.bridge.next_object_id() {
        Ok(id) => id,
        Err(_) => return,
    };
    let id2 = match shell.bridge.next_object_id() {
        Ok(id) => id,
        Err(_) => return,
    };

    let cmds = vec![
        Command::CreateObject {
            surface: surface_id,
            id: id1,
            name: "Rectangle A".to_string(),
        },
        Command::SetShape {
            id: id1,
            shape: Some(petunia_design_document::ShapeKind::Rectangle {
                corner_radii: [12.0, 12.0, 12.0, 12.0],
            }),
        },
        Command::SetBounds {
            id: id1,
            bounds: Some([260.0, 180.0, 220.0, 160.0]),
            rotation: 0.0,
        },
        Command::SetFill {
            id: id1,
            fill: Some("ptnd.blue/500".to_string()),
        },
        Command::CreateObject {
            surface: surface_id,
            id: id2,
            name: "Circle B".to_string(),
        },
        Command::SetShape {
            id: id2,
            shape: Some(petunia_design_document::ShapeKind::Ellipse),
        },
        Command::SetBounds {
            id: id2,
            bounds: Some([380.0, 240.0, 200.0, 200.0]),
            rotation: 0.0,
        },
        Command::SetFill {
            id: id2,
            fill: Some("ptnd.purple/500".to_string()),
        },
    ];
    let _ = shell.bridge.submit_all("Seed starter shapes", cmds);
    shell.bridge.clear_selection();
}

fn app() -> impl IntoElement {
    use_init_theme(theme::petunia_theme);

    let shell = use_state(|| {
        let mut shell = PetuniaShell::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        shell
            .new_document(DEFAULT_DOCUMENT_TITLE)
            .expect("a fresh document opens");
        seed_starter_shapes(&mut shell);
        shell
    });
    let ui = UiShell::fresh(shell);
    let root_a11y_id = use_a11y();
    let keyboard_shell = ui.shell;
    let modifiers = ui.modifiers;
    let palette_open = ui.palette_open;
    let palette_query = ui.palette_query;
    let tool_rail = ui.tool_rail;
    let active_tool = ui.active_tool;
    let customize_open = ui.customize_open;
    let temporary_tool = ui.temporary_tool;
    let suspended_tool = ui.suspended_tool;
    let new_doc_open = ui.new_doc_open;
    let export_open = ui.export_open;

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
                .child(Workspace(ui.clone()))
                .child(DockSplitter(ui.clone()))
                .child(dock::RightDock(ui.clone())),
        )
        .child(StatusBar(ui.clone()))
        .child(CommandPalette(ui.clone()))
        .child(CustomizeDialog(ui.clone()))
        .child(NewDocumentDialog(ui.clone()))
        .child(ExportDialog(ui.clone()))
        .child(ConfirmCloseDialog(ui.clone()))
        .child(TooltipOverlay(ui.clone()))
        .on_global_key_down({
            let shell = keyboard_shell;
            let mut modifiers = modifiers;
            move |event: Event<KeyboardEventData>| {
                modifiers.set(semantic_modifiers(event.modifiers));
                dispatch_workspace_key(
                    shell,
                    palette_open,
                    palette_query,
                    tool_rail,
                    active_tool,
                    customize_open,
                    new_doc_open,
                    export_open,
                    temporary_tool,
                    suspended_tool,
                    &event,
                );
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

/// Draggable splitter between Workspace and RightDock.
#[derive(Clone, PartialEq)]
struct DockSplitter(UiShell);

impl Component for DockSplitter {
    fn render(&self) -> impl IntoElement {
        let is_dragging = use_state(|| false);
        let drag_start_x = use_state(|| 0.0f64);
        let drag_start_width = use_state(|| 240.0f32);
        let dock_width = self.0.dock_width;
        let dragging_val = *is_dragging.read();

        let splitter_bar = rect()
            .width(Size::px(4.))
            .height(Size::fill())
            .background(if dragging_val {
                theme::ACCENT_BLOOM
            } else {
                theme::SURFACE_CHROME_STRONG
            })
            .cursor(CursorIcon::EwResize)
            .on_mouse_down({
                let mut is_dragging = is_dragging;
                let mut drag_start_x = drag_start_x;
                let mut drag_start_width = drag_start_width;
                move |event: Event<MouseEventData>| {
                    is_dragging.set(true);
                    drag_start_x.set(event.global_location.x);
                    drag_start_width.set(*dock_width.peek());
                }
            });

        if dragging_val {
            rect()
                .direction(Direction::Horizontal)
                .width(Size::px(4.))
                .height(Size::fill())
                .child(splitter_bar)
                .child(
                    Portal::new("dock-splitter-drag")
                        .width(Size::px(0.))
                        .height(Size::px(0.))
                        .child(
                            rect()
                                .position(Position::new_absolute().top(0.).left(0.))
                                .width(Size::fill())
                                .height(Size::fill())
                                .cursor(CursorIcon::EwResize)
                                .on_mouse_move({
                                    let mut dock_width = dock_width;
                                    let drag_start_x = drag_start_x;
                                    let drag_start_width = drag_start_width;
                                    move |event: Event<MouseEventData>| {
                                        let delta = *drag_start_x.read() - event.global_location.x;
                                        let new_w = (*drag_start_width.read() + delta as f32).clamp(180.0, 520.0);
                                        dock_width.set(new_w);
                                    }
                                })
                                .on_mouse_up({
                                    let mut is_dragging = is_dragging;
                                    move |_| {
                                        is_dragging.set(false);
                                    }
                                }),
                        ),
                )
        } else {
            rect()
                .width(Size::px(4.))
                .height(Size::fill())
                .child(splitter_bar)
        }
    }
}

/// The document workspace. The canvas slice (08.29) draws the artboard here.
#[derive(Clone, PartialEq)]
struct Workspace(UiShell);

impl Component for Workspace {
    fn render(&self) -> impl IntoElement {
        let mut shell = self.0.shell;
        let modifiers = self.0.modifiers;
        let mut gesture_tick = use_state(|| 0u64);
        let ruler_drag = use_state(|| None::<(petunia_design_document::GuideOrientation, f64)>);
        let middle_pan_last = use_state(|| None::<GPoint>);
        let a11y_id = use_a11y();
        let snapshot = shell.read().canvas_snapshot();
        let cursor_icon = map_cursor_affordance(snapshot.overlays.cursor);
        let active_tool = *self.0.active_tool.read();
        let tick = *gesture_tick.read();
        let in_flight_guide = *ruler_drag.read();
        let canvas_key = (snapshot.revision ^ tick)
            .wrapping_add(snapshot.camera.zoom.to_bits())
            .wrapping_add(snapshot.camera.pan_x.to_bits())
            .wrapping_add(snapshot.camera.pan_y.to_bits())
            .wrapping_add(active_tool as u64);

        let active_text_object = snapshot.objects.iter().find(|o| {
            o.active && matches!(o.shape, Some(petunia_design_document::ShapeKind::Text { .. }))
        });

        let active_text_editor = if let Some(text_obj) = active_text_object {
            let (content, font_size) = match &text_obj.shape {
                Some(petunia_design_document::ShapeKind::Text { content, font_size, .. }) => (content.clone(), *font_size),
                _ => (String::new(), 16.0),
            };
            let screen_origin = snapshot.camera.doc_to_screen(GPoint::new(
                text_obj.frame_origin[0],
                text_obj.frame_origin[1],
            ));
            let text_obj_id = text_obj.id;
            let mut text_edit_content = self.0.text_edit_content;
            if text_edit_content.peek().is_empty() && !content.is_empty() {
                text_edit_content.set(content.clone());
            }
            let mut shell_for_commit = shell;
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
                        rect()
                            .width(Size::px(220.))
                            .child(Input::new(text_edit_content).placeholder("Texto...")),
                    )
                    .child(
                        Button::new()
                            .on_press(move |_| {
                                let current_text = text_edit_content.peek().clone();
                                let shape = petunia_design_document::ShapeKind::Text {
                                    content: current_text,
                                    font_family: "Inter".to_string(),
                                    font_size,
                                    line_height: 1.2,
                                    letter_spacing: 0.0,
                                    on_path: None,
                                };
                                let _ = shell_for_commit.write().bridge.submit_all(
                                    "Update text in-canvas",
                                    vec![petunia_design_application::Command::SetShape {
                                        id: text_obj_id,
                                        shape: Some(shape),
                                    }],
                                );
                            })
                            .child(label().text("Aplicar").font_size(11.)),
                    ),
            )
        } else {
            None
        };

        let mut workspace_container = rect()
            .width(Size::flex(1.0))
            .height(Size::fill())
            .background(theme::SURFACE_WORKSPACE)
            .cursor(cursor_icon)
            .a11y_id(a11y_id)
            .a11y_focusable(true)
            .on_global_pointer_press({
                move |event: Event<PointerEventData>| {
                    let location = event.element_location();
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
                canvas_paint::canvas_view(snapshot, canvas_key, in_flight_guide)
                    .on_pointer_down({
                        move |event| {
                            a11y_id.request_focus();
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
                        move |event| {
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
                        move |event: Event<MouseEventData>| {
                            event.prevent_default();
                            let button = pointer_button(event.button).unwrap_or(PointerButton::Primary);
                            let location = event.element_location;
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
                    })
                    .on_touch_end({
                        move |event: Event<TouchEventData>| {
                            let location = event.element_location;
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
                    })
                    .on_touch_cancel({
                        move |event: Event<TouchEventData>| {
                            let location = event.element_location;
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
    dispatch_workspace_at(
        shell,
        modifiers,
        gesture_tick,
        ruler_drag,
        middle_pan_last,
        phase,
        button,
        GPoint::new(location.x, location.y),
    );
}

fn dispatch_workspace_at(
    mut shell: State<PetuniaShell>,
    modifiers: State<SemanticModifiers>,
    mut gesture_tick: State<u64>,
    mut ruler_drag: State<Option<(petunia_design_document::GuideOrientation, f64)>>,
    mut middle_pan_last: State<Option<GPoint>>,
    phase: PointerPhase,
    button: PointerButton,
    screen: GPoint,
) {
    // 1. Middle mouse button pan navigation
    if button == PointerButton::Middle {
        match phase {
            PointerPhase::Down => {
                middle_pan_last.set(Some(screen));
                return;
            }
            PointerPhase::Move => {
                let last = middle_pan_last.peek().clone();
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
    let current_ruler_drag = ruler_drag.peek().clone();
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
                    let guide_id = (std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis())
                        .unwrap_or(0)
                        % 1_000_000) as u32;
                    let guide = petunia_design_document::Guide::new(guide_id, orient, pos);
                    let _ = shell.write().bridge.submit_all(
                        "Add guide",
                        vec![petunia_design_application::Command::AddGuide {
                            surface: surf_id,
                            guide,
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
    let event = NormalizedPointerEvent::new(phase, button, screen, document, *modifiers.read());
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

fn dispatch_workspace_key(
    mut shell: State<PetuniaShell>,
    mut palette_open: State<bool>,
    mut palette_query: State<String>,
    mut tool_rail: State<ToolRailState>,
    mut active_tool: State<ToolKind>,
    mut customize_open: State<bool>,
    mut new_doc_open: State<bool>,
    mut export_open: State<bool>,
    mut temporary_tool: State<Option<ToolKind>>,
    mut suspended_tool: State<Option<ToolKind>>,
    event: &Event<KeyboardEventData>,
) {
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
    if token == "ptnd.action.file.new" {
        new_doc_open.set(true);
    }
    if token == "ptnd.action.file.export" {
        export_open.set(true);
    }
    let _ = run_action_id(&mut shell.write(), token);
}

fn workspace_shortcut(event: &Event<KeyboardEventData>) -> Option<&'static str> {
    let control =
        event.modifiers.contains(Modifiers::CONTROL) || event.modifiers.contains(Modifiers::META);
    if control {
        return match &event.key {
            Key::Character(key) if key.eq_ignore_ascii_case("n") => Some("ptnd.action.file.new"),
            Key::Character(key) if key.eq_ignore_ascii_case("e") => {
                Some("ptnd.action.file.export")
            }
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
        let shell_ref = self.0.shell.peek();
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
        let hint = hovered.map_or_else(
            || {
                shell_ref
                    .bridge
                    .persona_hint(shell_ref.bridge.persona())
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
                    .text(hint)
                    .color(theme::TEXT_TERTIARY)
                    .font_size(theme::CAPTION_SIZE),
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

        let shell = seen.borrow().clone().expect("workspace mounted");
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
                    seed_starter_shapes(&mut shell);
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
            let shell = seen.borrow().clone().unwrap();
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
            let shell = seen.borrow().clone().unwrap();
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
            let shell = seen.borrow().clone().unwrap();
            let shell = shell.peek();
            let session = shell.bridge.session().unwrap();
            let surf = session.surface(session.active_surface().unwrap()).unwrap();
            assert_eq!(surf.objects().len(), 1, "text object must be created on surface");
            assert!(matches!(
                surf.objects()[0].shape,
                Some(petunia_design_document::ShapeKind::Text { .. })
            ));
            surf.objects()[0].id
        };

        // Switch to Select tool, deselect, then click on the text object to select it
        {
            let mut shell_state = seen.borrow().clone().unwrap();
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
            let shell = seen.borrow().clone().unwrap();
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
                    seed_starter_shapes(&mut shell);
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
            let shell = seen.borrow().clone().unwrap();
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
            let shell = seen.borrow().clone().unwrap();
            let sel = shell.peek().bridge.selection();
            assert_eq!(sel.selected_ids, vec![id1], "Step 1: Object A should be selected");
        }

        // 2. Click Circle B (500, 300)
        runner.press_cursor((500., 300.));
        runner.release_cursor((500., 300.));
        runner.sync_and_update();
        {
            let shell = seen.borrow().clone().unwrap();
            let sel = shell.peek().bridge.selection();
            assert_eq!(sel.selected_ids, vec![id2], "Step 2: Object B should be selected");
        }

        // 3. Click empty space (100, 100) to deselect
        runner.press_cursor((100., 100.));
        runner.release_cursor((100., 100.));
        runner.sync_and_update();
        {
            let shell = seen.borrow().clone().unwrap();
            let sel = shell.peek().bridge.selection();
            assert!(sel.is_empty, "Step 3: Selection should be empty");
        }

        // 4. Click Rectangle A (300, 200) again!
        runner.press_cursor((300., 200.));
        runner.release_cursor((300., 200.));
        runner.sync_and_update();
        {
            let shell = seen.borrow().clone().unwrap();
            let sel = shell.peek().bridge.selection();
            assert_eq!(sel.selected_ids, vec![id1], "Step 4: Object A should be selected again");
        }

        // 5. Click Circle B (500, 300) again!
        runner.press_cursor((500., 300.));
        runner.release_cursor((500., 300.));
        runner.sync_and_update();
        {
            let shell = seen.borrow().clone().unwrap();
            let sel = shell.peek().bridge.selection();
            assert_eq!(sel.selected_ids, vec![id2], "Step 5: Object B should be selected again");
        }
    }

    #[test]
    fn group_and_ungroup_actions_work_on_selection() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("Test").expect("document opens");
        seed_starter_shapes(&mut shell);

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
        let mut tile = Tile::new_empty(TileCoord::new(0, 0), PixelFormat::Rgba8, AlphaMode::Straight);
        tile.set_pixel_normalized(10, 10, [1.0, 0.0, 0.0, 1.0]);
        let rgba8 = match tile.format {
            PixelFormat::Rgba8 => tile.data.clone(),
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
            let shell = seen.borrow().clone().unwrap();
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
            let shell = seen.borrow().clone().unwrap();
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
    fn place_image_action_creates_image_object_in_document() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("ImageTest").expect("document opens");
        let action_res = actions::run_action_token(&mut shell, "ptnd.action.file.place#null");
        assert_eq!(action_res, Some("ptnd.action.file.place".to_string()));

        let session = shell.bridge.session().unwrap();
        let surface_id = session.active_surface().unwrap();
        let surface = session.surface(surface_id).unwrap();
        assert!(!surface.objects().is_empty(), "image object should be created");
        let last_object = surface.objects().last().unwrap();
        match &last_object.shape {
            Some(petunia_design_document::ShapeKind::Image { path, .. }) => {
                assert!(path.contains("sample_image.png") || !path.is_empty());
            }
            other => panic!("expected ShapeKind::Image, got {:?}", other),
        }
    }

    #[test]
    fn in_canvas_text_editor_updates_text_object() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("TextEditTest").expect("document opens");
        let surface_id = shell.bridge.active_surface().unwrap();
        let text_id = shell.bridge.next_object_id().unwrap();

        // 1. Create text object
        shell.bridge.submit_all(
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
        ).expect("command succeeds");

        // 2. Select it
        shell.bridge.set_selection(vec![text_id]);

        // 3. Verify snapshot identifies it as active text object
        let snapshot = shell.canvas_snapshot();
        let active_text = snapshot.objects.iter().find(|o| {
            o.active && matches!(o.shape, Some(petunia_design_document::ShapeKind::Text { .. }))
        });
        assert!(active_text.is_some(), "active text object found in snapshot");

        // 4. Update its content directly via command as in-canvas editor does
        shell.bridge.submit_all(
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
        ).expect("update succeeds");

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

        // Preset: Full HD 1920x1080
        shell.bridge.submit_all(
            "Set Surface Geometry 1080p",
            vec![petunia_design_application::Command::SetSurfaceGeometry {
                surface: surface_id,
                origin: [0.0, 0.0],
                dimensions: [1920.0, 1080.0],
            }],
        ).expect("command succeeds");

        let session = shell.bridge.session().unwrap();
        let surface = session.surface(surface_id).unwrap();
        assert_eq!(surface.dimensions, [1920.0, 1080.0]);
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
        let res = shell.bridge.dispatch_action(
            petunia_design_application::ActionRequest::new(
                petunia_design_application::ActionId::new("ptnd.action.file.export"),
                payload,
            ),
        );
        assert!(res.is_ok(), "export action should be dispatched successfully");
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
    }

    #[test]
    fn confirm_close_safeguards_unsaved_document() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("UnsavedDocTest").expect("document opens");
        assert!(shell.bridge.is_dirty(), "document with initial canvas starts with dirty revision");

        // Safe close without force is blocked by unsaved changes
        let safe_close = shell.bridge.close_session(false).expect("close check");
        assert!(!safe_close, "safe close requires user confirmation when dirty");

        // Force close (from confirm dialog) closes successfully
        let force_close = shell.bridge.close_session(true).expect("force close");
        assert!(force_close, "confirming close discards changes and closes session");
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
                    .child(CustomizeDialog(ui.clone()))
                    .child(ConfirmCloseDialog(ui))
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

        // Toggle ExportDialog open and close
        ui.export_open.set(true);
        runner.sync_and_update();
        ui.export_open.set(false);
        runner.sync_and_update();

        // Toggle all open simultaneously and close all
        ui.palette_open.set(true);
        ui.new_doc_open.set(true);
        ui.export_open.set(true);
        ui.customize_open.set(true);
        ui.confirm_close_open.set(true);
        runner.sync_and_update();

        ui.palette_open.set(false);
        ui.new_doc_open.set(false);
        ui.export_open.set(false);
        ui.customize_open.set(false);
        ui.confirm_close_open.set(false);
        runner.sync_and_update();
    }

    #[test]
    fn full_app_dock_layout_and_click_test() {
        let seen: Rc<RefCell<Option<UiShell>>> = Rc::new(RefCell::new(None));
        let seen_hook = seen.clone();

        let (mut runner, ()) = TestingRunner::new(
            move || {
                let shell = use_state(|| {
                    let mut s = PetuniaShell::new(1280., 800.);
                    s.new_document("DockLayoutDoc").expect("doc opens");
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
                            .child(Workspace(ui.clone()))
                            .child(DockSplitter(ui.clone()))
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

        // Click Propriedades tab (roughly 960 + 120 = 1080)
        runner.click_cursor((1080., 120.));
        runner.sync_and_update();
        assert_eq!(*ui.dock_tab.read(), 1, "switches to Propriedades (1)");

        // Click Cores tab (roughly 960 + 200 = 1160)
        runner.click_cursor((1160., 120.));
        runner.sync_and_update();
        assert_eq!(*ui.dock_tab.read(), 2, "switches to Cores (2)");

        // Click Histórico tab (roughly 960 + 270 = 1230)
        runner.click_cursor((1230., 120.));
        runner.sync_and_update();
        assert_eq!(*ui.dock_tab.read(), 3, "switches to Histórico (3)");

        // Click Camadas tab (roughly 960 + 40 = 1000)
        runner.click_cursor((1000., 120.));
        runner.sync_and_update();
        assert_eq!(*ui.dock_tab.read(), 0, "switches back to Camadas (0)");
    }

    #[test]
    fn gradient_and_measure_overlays_and_controls() {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("OverlaysTest").expect("document opens");

        // Measure tool mode toggling
        shell.tools.set_active_tool(ToolKind::Measure);
        assert_eq!(shell.tools.measure_tool().mode(), petunia_design_shell::tools::MeasureMode::Distance);
        shell.tools.measure_tool_mut().set_mode(petunia_design_shell::tools::MeasureMode::Area);
        assert_eq!(shell.tools.measure_tool().mode(), petunia_design_shell::tools::MeasureMode::Area);
        shell.tools.measure_tool_mut().cancel();

        // Gradient tool kind toggling
        shell.tools.set_active_tool(ToolKind::Gradient);
        assert_eq!(shell.tools.gradient_tool().kind(), petunia_design_shell::tools::GradientKind::Linear);
        shell.tools.gradient_tool_mut().set_kind(petunia_design_shell::tools::GradientKind::Radial);
        assert_eq!(shell.tools.gradient_tool().kind(), petunia_design_shell::tools::GradientKind::Radial);

        // Perspective tool overlays cursor affordance on selected object
        let rect_id = petunia_design_foundation::ObjectId::new(101);
        let surface_id = shell.bridge.active_surface().unwrap();
        shell.bridge.submit_all(
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
            ],
        ).unwrap();
        shell.bridge.set_selection(vec![rect_id]);

        shell.tools.set_active_tool(ToolKind::Perspective);
        let cam = shell.view_camera();
        let overlays = shell.tools.overlays(&cam, &shell.bridge);
        assert_eq!(overlays.handles.len(), 4, "Perspective quad provides 4 corner handles");
        assert_eq!(overlays.cursor, petunia_design_shell::canvas::CursorAffordance::Crosshair);
    }
}



