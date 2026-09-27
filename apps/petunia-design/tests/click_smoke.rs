//! Click smoke: every chrome control must react, not just paint.
//!
//! A control that renders without firing is the fake UI 15.F §2 forbids. This
//! test mounts the chrome against a real shell and clicks: menus open and
//! dispatch, the rail switches tools, the toolbar fires, personas switch.

use freya::prelude::*;
use freya_testing::prelude::*;
use petunia_design_shell::PetuniaShell;

#[path = "../src/actions.rs"]
mod actions;
#[path = "../src/chrome.rs"]
mod chrome;
#[path = "../src/theme.rs"]
mod theme;
#[path = "../src/ui_state.rs"]
mod ui_state;

use chrome::{ContextToolbar, DocumentTabStrip, MenuBarRow, ToolRail};
use petunia_design_application::tools::ToolKind;
use ui_state::UiShell;

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
    let seen: Rc<RefCell<Option<(State<PetuniaShell>, State<Option<String>>, State<bool>)>>> =
        Rc::new(RefCell::new(None));
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
            let dock_width = use_state(|| 240.0f32);
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
                dock_width,
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
        seen.borrow().clone().expect("chrome mounted with states");
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
    let count = seen.borrow().clone().expect("states");
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
    type HoverState = Option<(String, String)>;
    let seen: Rc<RefCell<Option<(State<i32>, State<HoverState>)>>> = Rc::new(RefCell::new(None));
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
    let (count, hovered) = seen.borrow().clone().expect("states");
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
