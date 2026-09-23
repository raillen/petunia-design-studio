//! Shell chrome: menu bar, personas, tab strip, shell cluster, toolbar, rail.
//!
//! Every label, order, tooltip and availability arrives from the surface
//! registry through the bridge. The UI paints lists it is handed; tooltips on
//! blocked entries carry the registry's own disabled reason (15.F §2).

use freya::prelude::*;
use petunia_design_application::menus::ActionContext;
use petunia_design_application::tools::ToolKind;
use petunia_design_shell::context_toolbar::{self, ToolbarEntryKind};
use petunia_design_shell::menu::{
    MenuFamilyPresentation, MenuGroupPresentation, MenuItemPresentation, MenuNodePresentation,
    ShellControlKind, ShellControlPresentation,
};
use petunia_design_shell::PetuniaShell;

use crate::actions::run_action_token;
use crate::theme;
use crate::ui_state::UiShell;

/// One tool button of the rail: icon, registry id, target tool.
struct RailTool {
    icon: theme::AppIcon,
    tool: ToolKind,
    active_names: &'static [&'static str],
}

const RAIL_VECTOR: &[RailTool] = &[
    RailTool {
        icon: theme::ICON_POINTER,
        tool: ToolKind::Select,
        active_names: &["Select"],
    },
    RailTool {
        icon: theme::ICON_POINTER,
        tool: ToolKind::Node,
        active_names: &["Node"],
    },
    RailTool {
        icon: theme::ICON_PEN,
        tool: ToolKind::Pen,
        active_names: &["Pen"],
    },
    RailTool {
        icon: theme::ICON_PENCIL,
        tool: ToolKind::Pencil,
        active_names: &["Pencil"],
    },
    RailTool {
        icon: theme::ICON_SQUARE,
        tool: ToolKind::Rectangle,
        active_names: &["Rectangle"],
    },
    RailTool {
        icon: theme::ICON_CIRCLE,
        tool: ToolKind::Ellipse,
        active_names: &["Ellipse"],
    },
    RailTool {
        icon: theme::ICON_HEXAGON,
        tool: ToolKind::Polygon,
        active_names: &["Polygon"],
    },
    RailTool {
        icon: theme::ICON_STAR,
        tool: ToolKind::Star,
        active_names: &["Star"],
    },
    RailTool {
        icon: theme::ICON_TEXT,
        tool: ToolKind::ArtisticText,
        active_names: &["ArtisticText", "FrameText"],
    },
    RailTool {
        icon: theme::ICON_POINTER,
        tool: ToolKind::Zoom,
        active_names: &["Zoom"],
    },
    RailTool {
        icon: theme::ICON_POINTER,
        tool: ToolKind::Hand,
        active_names: &["Hand", "Artboard"],
    },
];

/// Menu bar + persona row, driven by `query_menu_bar` and `personas`.
#[derive(Clone, PartialEq)]
pub struct MenuBarRow(pub UiShell);

impl Component for MenuBarRow {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let bar = ui.shell.peek().bridge.query_menu_bar();
        let personas = ui.shell.peek().bridge.personas();
        let active_persona = ui.shell.peek().bridge.persona();

        rect()
            .direction(Direction::Horizontal)
            .width(Size::fill())
            .height(Size::px(theme::PERSONA_ROW_HEIGHT))
            .background(theme::SURFACE_CHROME)
            .padding(Gaps::new(0., theme::SPACE_2, 0., theme::SPACE_2))
            .cross_align(Alignment::Center)
            .main_align(Alignment::SpaceBetween)
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .spacing(theme::SPACE_1)
                    .cross_align(Alignment::Center)
                    .child(brand_slot())
                    .children(
                        bar.families
                            .iter()
                            .enumerate()
                            .map(|(index, family)| family_button(ui.clone(), index, family)),
                    ),
            )
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .spacing(theme::SPACE_1)
                    .cross_align(Alignment::Center)
                    .child(shell_cluster(ui.clone()))
                    .children(personas.iter().map(|persona| {
                        persona_button(ui.clone(), persona, persona.id == active_persona)
                    })),
            )
    }
}

/// Centre cluster: undo, redo, zoom controls from `query_shell_controls`.
#[derive(Clone, PartialEq)]
struct ShellCluster(pub UiShell);

fn shell_cluster(ui: UiShell) -> ShellCluster {
    ShellCluster(ui)
}

impl Component for ShellCluster {
    fn render(&self) -> impl IntoElement {
        let controls = self.0.shell.peek().bridge.query_shell_controls();
        rect()
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(theme::SPACE_1)
            .children(
                controls
                    .iter()
                    .map(|control| shell_control(self.0.clone(), control)),
            )
    }
}

/// Document tab strip: the open document tab plus the snap toggle.
#[derive(Clone, PartialEq)]
pub struct DocumentTabStrip(pub UiShell);

impl Component for DocumentTabStrip {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let shell_ref = ui.shell.peek();
        let title = shell_ref
            .bridge
            .session()
            .map_or_else(|| "Untitled".to_string(), |s| s.title().to_string());
        let snap_label = shell_ref
            .bridge
            .surface_label("ptnd.surface.tabs.snapping")
            .unwrap_or_default();
        let snapping = shell_ref
            .bridge
            .session()
            .is_some_and(|s| s.view.snapping_enabled);

        rect()
            .direction(Direction::Horizontal)
            .width(Size::fill())
            .height(Size::px(theme::TAB_STRIP_HEIGHT))
            .background(theme::SURFACE_CHROME_STRONG)
            .padding(Gaps::new(0., theme::SPACE_2, 0., theme::SPACE_2))
            .spacing(theme::SPACE_2)
            .cross_align(Alignment::Center)
            .child(
                label()
                    .text(title)
                    .color(theme::TEXT_PRIMARY)
                    .font_size(theme::CAPTION_SIZE),
            )
            .child(snap_toggle(ui.clone(), snap_label, snapping))
    }
}

/// Left tool rail: vector persona tools, availability from the registry.
#[derive(Clone, PartialEq)]
pub struct ToolRail(pub UiShell);

impl Component for ToolRail {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let active_tool = format!("{:?}", ui.shell.peek().active_tool());
        rect()
            .direction(Direction::Vertical)
            .width(Size::px(theme::TOOL_RAIL_WIDTH))
            .height(Size::fill())
            .background(theme::SURFACE_CHROME)
            .padding(Gaps::new_all(theme::SPACE_1))
            .spacing(theme::TOOL_GAP)
            .cross_align(Alignment::Center)
            .children(
                RAIL_VECTOR
                    .iter()
                    .map(|tool| rail_button(ui.clone(), tool, &active_tool)),
            )
    }
}

/// Canonical context toolbar (08.23), scoped by the shell to the active tool.
#[derive(Clone, PartialEq)]
pub struct ContextToolbar(pub UiShell);

impl Component for ContextToolbar {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let shell_ref = ui.shell.peek();
        let tool = shell_ref.active_tool();
        let has_selection = shell_ref.bridge.action_context().selection_count > 0;
        let entries = shell_ref.bridge.query_context_toolbar(tool, has_selection);

        // A `fill` spacer breaks hit-testing for every sibling after it, so
        // the bar never renders one: entries are split at spacers into groups
        // and the row spreads the groups with `SpaceBetween`. One spacer puts
        // the trailing group on the right edge; more spacers spread evenly.
        let mut groups: Vec<Vec<Element>> = vec![Vec::new()];
        for entry in &entries {
            if entry.kind == ToolbarEntryKind::Spacer {
                groups.push(Vec::new());
            } else {
                groups
                    .last_mut()
                    .expect("groups never empty")
                    .push(toolbar_entry(ui.clone(), entry).into_element());
            }
        }
        groups
            .last_mut()
            .expect("groups never empty")
            .push(customize_button(ui.clone()).into_element());
        rect()
            .direction(Direction::Horizontal)
            .width(Size::fill())
            .height(Size::px(theme::TOOLBAR_HEIGHT))
            .background(theme::SURFACE_PANEL)
            .padding(Gaps::new(0., theme::SPACE_2, 0., theme::SPACE_2))
            .main_align(Alignment::SpaceBetween)
            .cross_align(Alignment::Center)
            .children(
                groups
                    .into_iter()
                    .filter(|group| !group.is_empty())
                    .map(|group| {
                        rect()
                            .direction(Direction::Horizontal)
                            .spacing(theme::SPACE_1)
                            .cross_align(Alignment::Center)
                            .children(group)
                    }),
            )
    }
}

fn brand_slot() -> impl IntoElement {
    rect()
        .width(Size::px(theme::BRAND_MARK_SIZE))
        .height(Size::px(theme::BRAND_MARK_SIZE))
        .background(theme::SURFACE_CHROME_STRONG)
}

fn family_button(ui: UiShell, index: usize, family: &MenuFamilyPresentation) -> impl IntoElement {
    let mut open_family = ui.open_family;
    let is_open = *open_family.read() == Some(index);
    rect()
        .padding(Gaps::new(
            theme::SPACE_1,
            theme::SPACE_2,
            theme::SPACE_1,
            theme::SPACE_2,
        ))
        .center()
        .background(if is_open {
            theme::SURFACE_PANEL
        } else {
            Color::TRANSPARENT
        })
        .on_press(move |_| {
            open_family.set(if is_open { None } else { Some(index) });
        })
        .child(
            label()
                .text(family.label.clone())
                .color(if is_open {
                    theme::TEXT_PRIMARY
                } else {
                    theme::TEXT_SECONDARY
                })
                .font_size(theme::BODY_SIZE),
        )
}

/// The open family's popup: items and groups straight from the presentation.
#[derive(Clone, PartialEq)]
pub struct FamilyPopup(pub UiShell);

impl Component for FamilyPopup {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let families = ui.shell.peek().bridge.query_menu_bar().families;
        let Some(index) = *ui.open_family.read() else {
            // Kept in the layout flow so the row below never shifts when the
            // popup opens.
            return rect().width(Size::px(0.)).height(Size::px(0.));
        };
        let Some(family) = families.get(index) else {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        };
        rect()
            .width(Size::fill())
            .height(Size::px(0.))
            .layer(Layer::Overlay)
            .child(
                Menu::new()
                    .on_close({
                        let mut open_family = ui.open_family;
                        move |_| open_family.set(None)
                    })
                    .children(family.nodes.iter().map(|node| menu_node(ui.clone(), node))),
            )
    }
}

fn menu_node(ui: UiShell, node: &MenuNodePresentation) -> impl IntoElement {
    match node {
        MenuNodePresentation::Item(item) => menu_item(ui, item).into_element(),
        MenuNodePresentation::Group(group) => family_submenu(ui, group).into_element(),
    }
}

fn menu_item(ui: UiShell, item: &MenuItemPresentation) -> impl IntoElement {
    let token = item.action_token.clone();
    let row_label = if item.shortcut.is_empty() {
        item.label.clone()
    } else {
        format!("{}  {}", item.label, item.shortcut)
    };
    let enabled = item.enabled;
    let tooltip_text = if enabled {
        item.label.clone()
    } else {
        item.disabled_reason.clone()
    };
    let mut shell = ui.shell;
    let mut open_family = ui.open_family;
    let row = MenuItem::new()
        .on_press(move |_| {
            if !enabled {
                return;
            }
            run_action_token(&mut shell.write(), &token);
            open_family.set(None);
        })
        .child(
            label()
                .text(row_label)
                .color(if enabled {
                    theme::TEXT_PRIMARY
                } else {
                    theme::TEXT_TERTIARY
                })
                .font_size(theme::BODY_SIZE),
        );
    // A blocked row carries its reason as a second line: the reason is
    // registry copy, never invented here, and it stays in the layout flow.
    if enabled || tooltip_text.is_empty() {
        row.into_element()
    } else {
        rect()
            .direction(Direction::Vertical)
            .child(row)
            .child(
                label()
                    .text(tooltip_text)
                    .color(theme::TEXT_TERTIARY)
                    .font_size(theme::CAPTION_SIZE),
            )
            .into_element()
    }
}

fn family_submenu(ui: UiShell, group: &MenuGroupPresentation) -> impl IntoElement {
    SubMenu::new()
        .label(
            label()
                .text(group.label.clone())
                .color(theme::TEXT_PRIMARY)
                .font_size(theme::BODY_SIZE),
        )
        .children(group.items.iter().map(|item| menu_item(ui.clone(), item)))
}

fn persona_button(
    ui: UiShell,
    persona: &petunia_design_shell::menu::PersonaPresentation,
    active: bool,
) -> impl IntoElement {
    let id = persona.id.clone();
    let mut shell = ui.shell;
    let accent = *ui.accent.read();
    with_hover(
        rect()
            .padding(Gaps::new(
                theme::SPACE_1,
                theme::SPACE_2,
                theme::SPACE_1,
                theme::SPACE_2,
            ))
            .center()
            .background(if active {
                accent.value
            } else {
                Color::TRANSPARENT
            })
            .on_press(move |_| {
                let _ = shell.write().bridge.set_persona(&id);
            })
            .child(
                label()
                    .text(persona.label.clone())
                    .color(if active {
                        theme::TEXT_PRIMARY
                    } else {
                        theme::TEXT_TERTIARY
                    })
                    .font_size(theme::CAPTION_SIZE),
            ),
        &ui,
        hover_id("persona", &persona.id),
        persona.hint.clone(),
    )
}

fn snap_toggle(ui: UiShell, snap_label: String, snapping: bool) -> impl IntoElement {
    let mut shell = ui.shell;
    let hint = snap_label.clone();
    let state_text_id = if snapping {
        "ptnd.text.shell.snap_on"
    } else {
        "ptnd.text.shell.snap_off"
    };
    let state_label = ui
        .shell
        .peek()
        .bridge
        .localization()
        .text(state_text_id, ui.shell.peek().bridge.locale());
    with_hover(
        rect()
            .direction(Direction::Horizontal)
            .spacing(theme::SPACE_1)
            .cross_align(Alignment::Center)
            .on_press(move |_| {
                use petunia_design_application::{ActionId, ActionRequest};
                let _ = shell
                    .write()
                    .bridge
                    .dispatch_action(ActionRequest::without_payload(ActionId::new(
                        "ptnd.action.view.toggle_snapping",
                    )));
            })
            .child(
                label()
                    .text(snap_label)
                    .color(theme::TEXT_SECONDARY)
                    .font_size(theme::CAPTION_SIZE),
            )
            .child(
                label()
                    .text(state_label)
                    .color(if snapping {
                        theme::ACCENT_BLOOM
                    } else {
                        theme::TEXT_TERTIARY
                    })
                    .font_size(theme::CAPTION_SIZE),
            ),
        &ui,
        hover_id("tabstrip", "snapping"),
        hint,
    )
}

fn shell_control(ui: UiShell, control: &ShellControlPresentation) -> impl IntoElement {
    let body = match control.kind {
        ShellControlKind::Divider => rect()
            .width(Size::px(1.))
            .height(Size::px(theme::ICON_INLINE))
            .background(theme::BORDER_SUBTLE),
        ShellControlKind::Readout => {
            let shell_ref = ui.shell.peek();
            let zoom_pct = (shell_ref.view_camera().zoom * 100.).round() as i32;
            rect()
                .padding(Gaps::new(0., theme::SPACE_2, 0., theme::SPACE_2))
                .center()
                .child(
                    label()
                        .text(format!("{zoom_pct}%"))
                        .color(theme::TEXT_SECONDARY)
                        .font_size(theme::CAPTION_SIZE),
                )
        }
        ShellControlKind::Icon => {
            let icon = control_icon(&control.id);
            let enabled = control.enabled;
            let id = control.id.clone();
            let mut shell = ui.shell;
            with_hover(
                rect()
                    .width(Size::px(theme::TOOL_BUTTON))
                    .height(Size::px(theme::TOOL_BUTTON))
                    .center()
                    .on_press(move |_| {
                        if enabled {
                            shell_control_action(&mut shell, &id);
                        }
                    })
                    .child(app_icon(icon, *ui.icon_style.read(), theme::TEXT_TERTIARY)),
                &ui,
                hover_id("shell", &control.id),
                control.tooltip.clone(),
            )
        }
    };
    body.into_element()
}

/// Tracks hover for the status-bar hint without disturbing layout.
///
/// `TooltipContainer` positions through `Attached`, which takes the button out
/// of the layout flow; hover state keeps every pixel where the registry put
/// it and still explains blocked controls (15.F §2).
fn with_hover(el: Rect, ui: &UiShell, id: String, text: String) -> Rect {
    let mut enter_hovered = ui.hovered;
    let mut leave_hovered = ui.hovered;
    let leave_id = id.clone();
    el.on_pointer_enter(move |_| {
        enter_hovered.set(Some((id.clone(), text.clone())));
    })
    .on_pointer_leave(move |_| {
        if leave_hovered
            .read()
            .as_ref()
            .is_some_and(|(hovered_id, _)| hovered_id == &leave_id)
        {
            leave_hovered.set(None);
        }
    })
}

fn hover_id(kind: &str, id: &str) -> String {
    format!("{kind}:{id}")
}

fn shell_control_action(shell: &mut State<PetuniaShell>, id: &str) {
    use petunia_design_application::{ActionId, ActionRequest};
    use petunia_design_shell::menu::shell_control_action as control_action;
    let Some(action_id) = control_action(id) else {
        return;
    };
    let ctx: ActionContext = shell.peek().bridge.action_context();
    if !petunia_design_application::menus::availability(action_id, &ctx).enabled {
        return;
    }
    let _ = shell
        .write()
        .bridge
        .dispatch_action(ActionRequest::without_payload(ActionId::new(action_id)));
}

fn control_icon(id: &str) -> theme::AppIcon {
    match id {
        "ptnd.surface.shell.undo" => theme::ICON_UNDO,
        "ptnd.surface.shell.redo" => theme::ICON_REDO,
        "ptnd.surface.shell.zoom_out" => theme::ICON_ZOOM_OUT,
        "ptnd.surface.shell.zoom_in" => theme::ICON_ZOOM_IN,
        "ptnd.surface.shell.fit" => theme::ICON_FIT,
        _ => theme::ICON_INFO,
    }
}

fn rail_button(ui: UiShell, tool: &RailTool, active_tool: &str) -> impl IntoElement {
    let active = tool.active_names.contains(&active_tool);
    let tool_kind = tool.tool;
    let mut shell = ui.shell;
    let accent = *ui.accent.read();
    // The name comes from the tool's own registry surface, like the badge.
    let tool_name = context_toolbar::tool_label_text_id(tool_kind).map_or_else(String::new, |id| {
        ui.shell
            .peek()
            .bridge
            .localization()
            .text(id, ui.shell.peek().bridge.locale())
    });
    with_hover(
        rect()
            .width(Size::px(theme::TOOL_BUTTON))
            .height(Size::px(theme::TOOL_BUTTON))
            .center()
            .background(if active {
                accent.value
            } else {
                Color::TRANSPARENT
            })
            .on_press(move |_| {
                shell.write().set_active_tool(tool_kind);
            })
            .child(app_icon(
                tool.icon,
                *ui.icon_style.read(),
                if active {
                    theme::TEXT_PRIMARY
                } else {
                    theme::TEXT_TERTIARY
                },
            )),
        &ui,
        hover_id("rail", tool_kind.action_id()),
        tool_name,
    )
}

fn toolbar_entry(
    ui: UiShell,
    entry: &petunia_design_shell::context_toolbar::ToolbarItemPresentation,
) -> impl IntoElement {
    let tooltip_text = if entry.enabled {
        entry.tooltip.clone()
    } else {
        entry.disabled_reason.clone()
    };
    let body = match entry.kind {
        ToolbarEntryKind::Spacer => rect()
            .width(Size::fill())
            .height(Size::px(theme::ICON_TOOLBAR)),
        ToolbarEntryKind::Divider => rect()
            .width(Size::px(1.))
            .height(Size::px(theme::ICON_INLINE))
            .background(theme::BORDER_SUBTLE),
        ToolbarEntryKind::ToolBadge => {
            let icon = tool_badge_icon(&entry.id);
            rect()
                .direction(Direction::Horizontal)
                .spacing(theme::SPACE_1)
                .cross_align(Alignment::Center)
                .child(app_icon(icon, *ui.icon_style.read(), theme::TEXT_PRIMARY))
                .child(
                    label()
                        .text(entry.label.clone())
                        .color(theme::TEXT_PRIMARY)
                        .font_size(theme::CAPTION_SIZE),
                )
        }
        ToolbarEntryKind::TransformReadout => rect()
            .padding(Gaps::new(0., theme::SPACE_2, 0., theme::SPACE_2))
            .center()
            .child(
                label()
                    .text(entry.label.clone())
                    .color(theme::TEXT_SECONDARY)
                    .font_size(theme::CAPTION_SIZE),
            ),
        ToolbarEntryKind::ColorSwatches => rect()
            .direction(Direction::Horizontal)
            .spacing(theme::SPACE_1)
            .cross_align(Alignment::Center)
            .child(
                rect()
                    .width(Size::px(18.))
                    .height(Size::px(18.))
                    .background(theme::ACCENT_BLOOM),
            )
            .child(
                rect()
                    .width(Size::px(18.))
                    .height(Size::px(18.))
                    .background(Color::TRANSPARENT),
            ),
        ToolbarEntryKind::Command => {
            let icon = toolbar_command_icon(&entry.id);
            let enabled = entry.enabled;
            let id = entry.id.clone();
            let mut shell = ui.shell;
            let accent = *ui.accent.read();
            with_hover(
                rect()
                    .width(Size::px(theme::TOOL_BUTTON))
                    .height(Size::px(theme::TOOL_BUTTON))
                    .center()
                    .on_press(move |_| {
                        if enabled {
                            activate_toolbar_entry(&mut shell, &id);
                        }
                    })
                    .child(app_icon(
                        icon,
                        *ui.icon_style.read(),
                        if enabled {
                            accent.value
                        } else {
                            theme::TEXT_TERTIARY
                        },
                    )),
                &ui,
                hover_id("toolbar", &entry.id),
                tooltip_text,
            )
        }
    };
    body.into_element()
}

fn activate_toolbar_entry(shell: &mut State<PetuniaShell>, id: &str) {
    let Some(entry) = context_toolbar::entry(id) else {
        return;
    };
    if entry.kind != ToolbarEntryKind::Command {
        return;
    }
    let token = entry.token.to_string();
    run_action_token(&mut shell.write(), &token);
}

fn tool_badge_icon(_id: &str) -> theme::AppIcon {
    theme::ICON_POINTER
}

fn toolbar_command_icon(id: &str) -> theme::AppIcon {
    if id.contains("align") {
        theme::ICON_POINTER
    } else if id.contains("boolean") {
        theme::ICON_SQUARE
    } else if id.contains("export") {
        theme::ICON_PLUS
    } else if id.contains("delete") {
        theme::ICON_TRASH
    } else {
        theme::ICON_STAR
    }
}

fn customize_button(ui: UiShell) -> impl IntoElement {
    let mut customize_open = ui.customize_open;
    let catalog_label = ui
        .shell
        .peek()
        .bridge
        .localization()
        .text("ptnd.text.shell.customize", ui.shell.peek().bridge.locale());
    with_hover(
        rect()
            .width(Size::px(theme::TOOL_BUTTON))
            .height(Size::px(theme::TOOL_BUTTON))
            .center()
            .on_press(move |_| {
                let open = *customize_open.read();
                customize_open.set(!open);
            })
            .child(app_icon(
                theme::ICON_CUSTOMIZE,
                *ui.icon_style.read(),
                theme::TEXT_TERTIARY,
            )),
        &ui,
        hover_id("toolbar", "customize"),
        catalog_label,
    )
}

/// Renders one Tabler glyph recolored to the icon style in use.
///
/// Outline glyphs use `currentColor`, so they are recolored through `color`;
/// filled glyphs are solid and recolored through `fill`.
pub fn app_icon(icon: theme::AppIcon, style: theme::IconStyle, color: Color) -> impl IntoElement {
    let bytes = icon.bytes(style);
    match style {
        theme::IconStyle::Outline => SvgViewer::new(bytes)
            .width(Size::px(theme::ICON_TOOLBAR))
            .height(Size::px(theme::ICON_TOOLBAR))
            .color(color),
        theme::IconStyle::Filled => SvgViewer::new(bytes)
            .width(Size::px(theme::ICON_TOOLBAR))
            .height(Size::px(theme::ICON_TOOLBAR))
            .fill(color),
    }
}
