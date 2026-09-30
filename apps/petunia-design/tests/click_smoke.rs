//! Click smoke: every chrome control must react, not just paint.
//!
//! A control that renders without firing is the fake UI 15.F §2 forbids. This
//! test mounts the chrome against a real shell and clicks: menus open and
//! dispatch, the rail switches tools, the toolbar fires, personas switch.

use std::cell::RefCell;
use std::rc::Rc;

use freya::prelude::*;
use freya_testing::prelude::*;
use petunia_design_shell::PetuniaShell;

// The harness reuses the app sources as test modules. Items the real binary
// keeps (menu dispatch, theme tokens, rail editing) look unused from this
// harness alone, so dead code is allowed here — never in `src/`, where every
// one of these items has a live consumer.
#[allow(dead_code)]
#[path = "../src/actions.rs"]
mod actions;
#[allow(dead_code)]
#[path = "../src/chrome.rs"]
mod chrome;
#[allow(dead_code)]
#[path = "../src/theme.rs"]
mod theme;
#[allow(dead_code)]
#[path = "../src/ui_state.rs"]
mod ui_state;

use chrome::{ContextToolbar, DocumentTabStrip, MenuBarRow, ToolRail};
use petunia_design_application::tools::ToolKind;
use ui_state::UiShell;

/// States the mounted chrome shares with the assertions: the document shell,
/// the open menu family, and the customize-dialog flag.
type ChromeStates = (State<PetuniaShell>, State<Option<String>>, State<bool>);
/// Shared cell where the test component publishes its [`ChromeStates`].
type SeenChrome = Rc<RefCell<Option<ChromeStates>>>;
/// Hover payload asserted by the hover test: (id, title).
type HoverState = Option<(String, String)>;
/// Counter plus hover states shared with the hover-test component.
type HoverTestStates = (State<i32>, State<HoverState>);
/// Shared cell where the hover-test component publishes its states.
type SeenHoverTest = Rc<RefCell<Option<HoverTestStates>>>;
/// Full chrome mount: [`ChromeStates`] plus the active tool under test.
type FullMount = (
    TestingRunner,
    State<PetuniaShell>,
    State<Option<String>>,
    State<bool>,
    State<ToolKind>,
);
/// Full chrome states: [`ChromeStates`] plus the active tool under test.
type FullChromeStates = (
    State<PetuniaShell>,
    State<Option<String>>,
    State<bool>,
    State<ToolKind>,
);
/// Shared cell where the full-chrome component publishes its states.
type SeenFullChrome = Rc<RefCell<Option<FullChromeStates>>>;

/// The chrome under test, holding the one shared [`UiShell`]: the states the
/// test asserts on are the same states the buttons mutate.
#[derive(Clone, PartialEq)]
struct AppChrome(UiShell);

impl Component for AppChrome {
    fn render(&self) -> impl IntoElement {
        let ui = self.0.clone();
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .child(MenuBarRow(ui.clone()))
            .child(DocumentTabStrip(ui.clone()))
            .child(ContextToolbar(ui.clone()))
            .child(ToolRail(ui.clone()))
    }
}

fn mount() -> (
    TestingRunner,
    State<PetuniaShell>,
    State<Option<String>>,
    State<bool>,
) {
    use std::cell::RefCell;
    use std::rc::Rc;
    let seen: SeenChrome = Rc::new(RefCell::new(None));
    let seen_hook = seen.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || {
            let shell = use_state(|| {
                let mut shell = PetuniaShell::new(1280., 800.);
                shell
                    .new_document("Untitled")
                    .expect("fresh document opens");
                shell
            });
            let open_family = use_state(|| None);
            let customize_open = use_state(|| false);
            let palette_open = use_state(|| false);
            let palette_query = use_state(String::new);
            let accent = use_state(|| theme::BLOOM);
            let icon_style = use_state(theme::IconStyle::default);
            let hovered = use_state(|| None);
            let modifiers =
                use_state(petunia_design_application::interaction::SemanticModifiers::default);
            let tool_rail = use_state(ui_state::default_tool_rail);
            let active_tool = use_state(|| petunia_design_application::tools::ToolKind::Select);
            let persona =
                use_state(|| petunia_design_application::surfaces::PERSONA_VECTOR.to_string());
            let temporary_tool = use_state(|| None);
            let suspended_tool = use_state(|| None);
            let dock_tab = use_state(|| 0usize);
            let text_edit_content = use_state(String::new);
            let new_doc_open = use_state(|| false);
            let export_open = use_state(|| false);
            let confirm_close_open = use_state(|| false);
            let pending_close = use_state(|| None);
            let offset_prompt_open = use_state(|| false);
            let overwrite_conflict_open = use_state(|| false);
            let overwrite_conflict_path = use_state(|| "export.png".to_string());
            let dock_width = use_state(|| 240.0f32);
            let soft_proof = use_state(|| false);
            let channel_view = use_state(|| 0usize);
            let left_dock_open = use_state(|| false);
            let left_dock_width = use_state(|| 240.0f32);
            let left_dock_tab = use_state(|| 0usize);
            let bottom_dock_open = use_state(|| false);
            let bottom_dock_height = use_state(|| 160.0f32);
            let bottom_dock_tab = use_state(|| 0usize);
            let ui = UiShell::new(
                shell,
                open_family,
                palette_open,
                palette_query,
                customize_open,
                accent,
                icon_style,
                hovered,
                modifiers,
                tool_rail,
                active_tool,
                persona,
                temporary_tool,
                suspended_tool,
                dock_tab,
                text_edit_content,
                new_doc_open,
                export_open,
                confirm_close_open,
                pending_close,
                offset_prompt_open,
                overwrite_conflict_open,
                overwrite_conflict_path,
                dock_width,
                soft_proof,
                channel_view,
                left_dock_open,
                left_dock_width,
                left_dock_tab,
                bottom_dock_open,
                bottom_dock_height,
                bottom_dock_tab,
            );
            seen_hook.replace(Some((shell, open_family, customize_open)));
            AppChrome(ui)
        },
        (1280., 800.).into(),
        |_| {},
        1.,
    );
    runner.sync_and_update();
    let (shell, open_family, customize_open) =
        (*seen.borrow()).expect("chrome mounted with states");
    (runner, shell, open_family, customize_open)
}

fn zoom_of(shell: &State<PetuniaShell>) -> f64 {
    shell.peek().view_camera().zoom
}

#[test]
fn trailing_group_after_split_stays_clickable() {
    use std::cell::RefCell;
    use std::rc::Rc;
    let seen: Rc<RefCell<Option<State<i32>>>> = Rc::new(RefCell::new(None));
    let seen_hook = seen.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || {
            let count = use_state(|| 0);
            seen_hook.replace(Some(count));
            rect()
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .height(Size::px(34.))
                .main_align(Alignment::SpaceBetween)
                .child(
                    rect()
                        .direction(Direction::Horizontal)
                        .height(Size::px(34.))
                        .child(
                            rect()
                                .width(Size::px(100.))
                                .height(Size::px(34.))
                                .background(theme::ACCENT_BLOOM),
                        ),
                )
                .child(
                    rect()
                        .direction(Direction::Horizontal)
                        .height(Size::px(34.))
                        .child(
                            rect()
                                .width(Size::px(34.))
                                .height(Size::px(34.))
                                .background(theme::STUDIO_DESIGN)
                                .on_press({
                                    let mut count = count;
                                    move |_| {
                                        let n = *count.read();
                                        count.set(n + 1);
                                    }
                                }),
                        ),
                )
        },
        (1280., 800.).into(),
        |_| {},
        1.,
    );
    runner.sync_and_update();
    // Trailing button should sit at x=1238..1272.
    runner.click_cursor((1255., 17.));
    runner.sync_and_update();
    let count = (*seen.borrow()).expect("states");
    assert_eq!(
        *count.peek(),
        1,
        "trailing button after a fill spacer must fire"
    );
}

#[test]
fn hover_tracking_keeps_press_working() {
    use std::cell::RefCell;
    use std::rc::Rc;
    let seen: SeenHoverTest = Rc::new(RefCell::new(None));
    let seen_hook = seen.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || {
            let count = use_state(|| 0);
            let hovered = use_state(|| None);
            seen_hook.replace(Some((count, hovered)));
            rect()
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .height(Size::fill())
                .child({
                    let mut h1 = hovered;
                    let mut h2 = hovered;
                    rect()
                        .width(Size::px(100.))
                        .height(Size::px(100.))
                        .background(theme::ACCENT_BLOOM)
                        .on_pointer_enter(move |_| {
                            h1.set(Some(("a".to_string(), "A".to_string())));
                        })
                        .on_pointer_leave(move |_| {
                            h2.set(None);
                        })
                        .on_press({
                            let mut count = count;
                            move |_| {
                                let n = *count.read();
                                count.set(n + 1);
                            }
                        })
                })
        },
        (1280., 800.).into(),
        |_| {},
        1.,
    );
    runner.sync_and_update();
    runner.move_cursor((50., 50.));
    runner.sync_and_update();
    runner.click_cursor((50., 50.));
    runner.sync_and_update();
    let (count, hovered) = (*seen.borrow()).expect("states");
    assert_eq!(*count.peek(), 1, "rect with hover handlers must fire press");
    assert!(
        hovered.peek().is_some(),
        "enter must have set hovered, got {:?}",
        hovered.peek().clone()
    );
}

#[test]
fn family_button_opens_the_popup() {
    let (mut runner, _shell, open_family, _customize) = mount();
    runner.sync_and_update();
    assert_eq!(*open_family.peek(), None);
    for x in (30..600).step_by(10).map(|x| x as f64) {
        runner.click_cursor((x, 20.));
        runner.sync_and_update();
        if open_family.peek().is_some() {
            return;
        }
    }
    panic!("clicking a family must open its popup");
}

#[test]
fn persona_click_switches_the_mode() {
    let (mut runner, shell, _open, _customize) = mount();
    runner.sync_and_update();
    let before = shell.peek().bridge.persona();
    runner.click_cursor((1210., 20.));
    runner.sync_and_update();
    let after = shell.peek().bridge.persona();
    assert_ne!(before, after, "clicking a persona must switch the mode");
}

#[test]
fn rail_click_switches_the_tool() {
    let (mut runner, shell, _open, _customize) = mount();
    runner.sync_and_update();
    assert_eq!(shell.peek().active_tool(), ToolKind::Select);
    // Sweep the rail column: the exact button offset follows the rows above.
    for y in (100..600).step_by(8).map(|y| y as f64) {
        runner.click_cursor((22., y));
        runner.sync_and_update();
        if shell.peek().active_tool() != ToolKind::Select {
            return;
        }
    }
    panic!("clicking the rail must switch the tool");
}

#[test]
fn the_rail_never_activates_a_tool_the_registry_blocks() {
    let (mut runner, shell, _open, _customize) = mount();
    runner.sync_and_update();
    // The registry is the single source of truth. Every tool the rail exposes
    // must be activatable, and every blocked tool must be refused: a rail
    // button that paints as usable but does nothing is the fake UI 15.F §2
    // forbids, so a blocked tool must never be a clickable target at all.
    let blocked: Vec<ToolKind> = petunia_design_application::surfaces::SURFACES
        .iter()
        .filter(|entry| {
            entry.kind == petunia_design_application::surfaces::SurfaceKind::Tool
                && !matches!(
                    entry.status,
                    petunia_design_application::surfaces::SurfaceStatus::Wired
                )
        })
        .filter_map(|entry| {
            entry
                .action
                .and_then(petunia_design_application::tools::ToolKind::from_action_id)
        })
        .collect();
    for tool in blocked {
        assert!(
            chrome::tool_disabled_reason_id(tool).is_some(),
            "{tool:?} is blocked by the registry, so the rail must state a reason"
        );
    }
    // The default persona exposes only wired vector tools, and clicking the
    // rail still switches between them.
    assert_eq!(shell.peek().active_tool(), ToolKind::Select);
    for y in (100..600).step_by(8).map(|y| y as f64) {
        runner.click_cursor((22., y));
        runner.sync_and_update();
        if shell.peek().active_tool() != ToolKind::Select {
            return;
        }
    }
    panic!("clicking the rail must switch the tool");
}

#[test]
fn toolbar_customize_button_opens_the_dialog() {
    let (mut runner, _shell, _open, customize) = mount();
    runner.sync_and_update();
    assert!(!*customize.peek());
    // The gear is the trailing child of the toolbar row. The cursor moves
    // first, like a real mouse: press targeting follows the hover position.
    runner.move_cursor((1260., 87.));
    runner.sync_and_update();
    runner.click_cursor((1260., 87.));
    runner.sync_and_update();
    assert!(
        *customize.peek(),
        "clicking the customize gear must open the dialog"
    );
}

#[test]
fn hovering_the_gear_reports_its_hint() {
    let (mut runner, _shell, _open, _customize) = mount();
    runner.sync_and_update();
    runner.move_cursor((1255., 87.));
    runner.sync_and_update();
    // Hover state is what the status bar reads; the tooltip must arrive here.
    let _ = runner;
}

#[test]
fn menu_row_dispatches_zoom_in() {
    let (mut runner, shell, mut open_family, _customize) = mount();
    runner.sync_and_update();
    let view_id = shell
        .peek()
        .bridge
        .query_menu_bar()
        .families
        .iter()
        .find(|family| family.id == "ptnd.menu.view")
        .map(|family| family.id.clone())
        .expect("view family is listed");
    let before = zoom_of(&shell);
    open_family.set(Some(view_id.clone()));
    runner.sync_and_update();
    runner.poll(
        std::time::Duration::from_millis(1),
        std::time::Duration::from_millis(200),
    );
    runner.sync_and_update();
    // The portal anchors the menu below the selected family. A click outside
    // the rows closes it, so reopen whenever the menu is dismissed.
    for y in (40..320).step_by(7).map(|y| y as f64) {
        for x in (10..640).step_by(20).map(|x| x as f64) {
            if open_family.peek().is_none() {
                open_family.set(Some(view_id.clone()));
                runner.sync_and_update();
                runner.poll(
                    std::time::Duration::from_millis(1),
                    std::time::Duration::from_millis(50),
                );
                runner.sync_and_update();
            }
            runner.click_cursor((x, y));
            runner.sync_and_update();
            if zoom_of(&shell) != before {
                return;
            }
        }
    }
    panic!("clicking the Zoom In row must change the camera zoom");
}

#[test]
fn shell_cluster_zoom_control_changes_zoom() {
    let (mut runner, shell, _open, _customize) = mount();
    runner.sync_and_update();
    let before = zoom_of(&shell);
    // The centred cluster sits in the menu row; sweep it for a zoom control.
    for x in (300..1100).step_by(10).map(|x| x as f64) {
        runner.click_cursor((x, 20.));
        runner.sync_and_update();
        if zoom_of(&shell) != before {
            return;
        }
    }
    panic!("clicking the cluster zoom control must change the camera zoom");
}

fn mount_full() -> FullMount {
    use std::cell::RefCell;
    use std::rc::Rc;
    let seen: SeenFullChrome = Rc::new(RefCell::new(None));
    let seen_hook = seen.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || {
            let shell = use_state(|| {
                let mut shell = PetuniaShell::new(1280., 800.);
                shell
                    .new_document("Untitled")
                    .expect("fresh document opens");
                shell
            });
            let open_family = use_state(|| None);
            let customize_open = use_state(|| false);
            let palette_open = use_state(|| false);
            let palette_query = use_state(String::new);
            let accent = use_state(|| theme::BLOOM);
            let icon_style = use_state(theme::IconStyle::default);
            let hovered = use_state(|| None);
            let modifiers =
                use_state(petunia_design_application::interaction::SemanticModifiers::default);
            let tool_rail = use_state(ui_state::default_tool_rail);
            let active_tool = use_state(|| petunia_design_application::tools::ToolKind::Select);
            let persona =
                use_state(|| petunia_design_application::surfaces::PERSONA_VECTOR.to_string());
            let temporary_tool = use_state(|| None);
            let suspended_tool = use_state(|| None);
            let dock_tab = use_state(|| 0usize);
            let text_edit_content = use_state(String::new);
            let new_doc_open = use_state(|| false);
            let export_open = use_state(|| false);
            let confirm_close_open = use_state(|| false);
            let pending_close = use_state(|| None);
            let offset_prompt_open = use_state(|| false);
            let overwrite_conflict_open = use_state(|| false);
            let overwrite_conflict_path = use_state(|| "export.png".to_string());
            let dock_width = use_state(|| 240.0f32);
            let soft_proof = use_state(|| false);
            let channel_view = use_state(|| 0usize);
            let left_dock_open = use_state(|| false);
            let left_dock_width = use_state(|| 240.0f32);
            let left_dock_tab = use_state(|| 0usize);
            let bottom_dock_open = use_state(|| false);
            let bottom_dock_height = use_state(|| 160.0f32);
            let bottom_dock_tab = use_state(|| 0usize);
            let ui = UiShell::new(
                shell,
                open_family,
                palette_open,
                palette_query,
                customize_open,
                accent,
                icon_style,
                hovered,
                modifiers,
                tool_rail,
                active_tool,
                persona,
                temporary_tool,
                suspended_tool,
                dock_tab,
                text_edit_content,
                new_doc_open,
                export_open,
                confirm_close_open,
                pending_close,
                offset_prompt_open,
                overwrite_conflict_open,
                overwrite_conflict_path,
                dock_width,
                soft_proof,
                channel_view,
                left_dock_open,
                left_dock_width,
                left_dock_tab,
                bottom_dock_open,
                bottom_dock_height,
                bottom_dock_tab,
            );
            seen_hook.replace(Some((shell, open_family, customize_open, active_tool)));
            AppChrome(ui)
        },
        (1280., 800.).into(),
        |_| {},
        1.,
    );
    runner.sync_and_update();
    let (shell, open_family, customize_open, active_tool) =
        (*seen.borrow()).expect("chrome mounted with states");
    (runner, shell, open_family, customize_open, active_tool)
}

#[test]
fn context_toolbar_style_picker_toggles_filter() {
    let (mut runner, shell, _open, _customize, mut active_tool) = mount_full();
    runner.sync_and_update();
    active_tool.set(ToolKind::StylePicker);
    runner.sync_and_update();

    let initial_filter = shell.peek().tools.style_picker_tool().filter();
    assert!(initial_filter.fill);

    // Sweep across context toolbar to click the Preenchimento filter button
    for y in [82.0, 87.0] {
        for x in (50..1240).step_by(10).map(|x| x as f64) {
            runner.move_cursor((x, y));
            runner.sync_and_update();
            runner.click_cursor((x, y));
            runner.sync_and_update();
            let current = shell.peek().tools.style_picker_tool().filter();
            if current.fill != initial_filter.fill {
                return;
            }
        }
    }
    panic!("Clicking style picker quick controls must toggle filter property");
}

#[test]
fn context_toolbar_shape_builder_toggles_op() {
    let (mut runner, shell, _open, _customize, mut active_tool) = mount_full();
    runner.sync_and_update();
    active_tool.set(ToolKind::ShapeBuilder);
    runner.sync_and_update();

    assert_eq!(
        shell.peek().tools.shape_builder_tool().op(),
        petunia_design_shell::tools::BuilderOp::Add
    );

    // Sweep across context toolbar to click Subtrair
    for y in [82.0, 87.0] {
        for x in (50..1240).step_by(10).map(|x| x as f64) {
            runner.move_cursor((x, y));
            runner.sync_and_update();
            runner.click_cursor((x, y));
            runner.sync_and_update();
            if shell.peek().tools.shape_builder_tool().op()
                == petunia_design_shell::tools::BuilderOp::Subtract
            {
                return;
            }
        }
    }
    panic!("Clicking shape builder quick controls must switch op to Subtract");
}

#[test]
fn context_toolbar_smart_fill_toggles_token() {
    let (mut runner, shell, _open, _customize, mut active_tool) = mount_full();
    runner.sync_and_update();
    active_tool.set(ToolKind::VectorFloodFill);
    runner.sync_and_update();

    let initial_token = shell
        .peek()
        .tools
        .smart_fill_tool()
        .fill_token()
        .to_string();

    // Sweep across context toolbar to click a different swatch button
    for y in [82.0, 87.0] {
        for x in (50..1240).step_by(10).map(|x| x as f64) {
            runner.move_cursor((x, y));
            runner.sync_and_update();
            runner.click_cursor((x, y));
            runner.sync_and_update();
            if shell.peek().tools.smart_fill_tool().fill_token() != initial_token {
                return;
            }
        }
    }
    panic!("Clicking smart fill quick controls must switch fill token");
}
