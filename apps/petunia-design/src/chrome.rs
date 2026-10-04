//! Shell chrome: menu bar, personas, tab strip, shell cluster, toolbar, rail.
//!
//! Every label, order, tooltip and availability arrives from the surface
//! registry through the bridge. The UI paints lists it is handed; tooltips on
//! blocked entries carry the registry's own disabled reason (15.F §2).

use freya::prelude::*;
use petunia_design_application::menus::ActionContext;
use petunia_design_application::tools::ToolKind;
use petunia_design_application::ActionId;
use petunia_design_shell::context_toolbar::{self, ToolbarEntryKind};
use petunia_design_shell::menu::{
    MenuFamilyPresentation, MenuGroupPresentation, MenuItemPresentation, MenuNodePresentation,
    ShellControlKind, ShellControlPresentation,
};
use petunia_design_shell::PetuniaShell;

use crate::actions::run_action_token;
use crate::studio_widgets::StudioButton;
use crate::theme;
use crate::ui_state::{HoverTarget, RailColumns, ToolGroupConfig, ToolRailState, UiShell};

#[derive(Clone, Copy)]
struct ToolMeta {
    icon: theme::AppIcon,
    label_id: &'static str,
    summary_id: &'static str,
}

fn tool_meta(tool: ToolKind) -> ToolMeta {
    match tool {
        ToolKind::Select => ToolMeta {
            icon: theme::ICON_POINTER,
            label_id: "ptnd.text.tool.select",
            summary_id: "ptnd.text.tool.select.summary",
        },
        ToolKind::Node => ToolMeta {
            icon: theme::ICON_NODE,
            label_id: "ptnd.text.tool.node",
            summary_id: "ptnd.text.tool.node.summary",
        },
        ToolKind::PointTransform => ToolMeta {
            icon: theme::ICON_MOVE,
            label_id: "ptnd.text.tool.point_transform",
            summary_id: "ptnd.text.tool.point_transform.summary",
        },
        ToolKind::Pen => ToolMeta {
            icon: theme::ICON_PEN,
            label_id: "ptnd.text.tool.pen",
            summary_id: "ptnd.text.tool.pen.summary",
        },
        ToolKind::Pencil => ToolMeta {
            icon: theme::ICON_PENCIL,
            label_id: "ptnd.text.tool.pencil",
            summary_id: "ptnd.text.tool.pencil.summary",
        },
        ToolKind::Corner => ToolMeta {
            icon: theme::ICON_CORNER,
            label_id: "ptnd.text.tool.corner",
            summary_id: "ptnd.text.tool.corner.summary",
        },
        ToolKind::Contour => ToolMeta {
            icon: theme::ICON_CONTOUR,
            label_id: "ptnd.text.tool.contour",
            summary_id: "ptnd.text.tool.contour.summary",
        },
        ToolKind::Perspective => ToolMeta {
            icon: theme::ICON_PERSPECTIVE,
            label_id: "ptnd.text.tool.perspective",
            summary_id: "ptnd.text.tool.perspective.summary",
        },
        ToolKind::Knife => ToolMeta {
            icon: theme::ICON_KNIFE,
            label_id: "ptnd.text.tool.knife",
            summary_id: "ptnd.text.tool.knife.summary",
        },
        ToolKind::Scissors => ToolMeta {
            icon: theme::ICON_SCISSORS,
            label_id: "ptnd.text.tool.scissors",
            summary_id: "ptnd.text.tool.scissors.summary",
        },
        ToolKind::Rectangle => ToolMeta {
            icon: theme::ICON_SQUARE,
            label_id: "ptnd.text.tool.rectangle",
            summary_id: "ptnd.text.tool.rectangle.summary",
        },
        ToolKind::Ellipse => ToolMeta {
            icon: theme::ICON_CIRCLE,
            label_id: "ptnd.text.tool.ellipse",
            summary_id: "ptnd.text.tool.ellipse.summary",
        },
        ToolKind::Polygon => ToolMeta {
            icon: theme::ICON_HEXAGON,
            label_id: "ptnd.text.tool.polygon",
            summary_id: "ptnd.text.tool.polygon.summary",
        },
        ToolKind::Star => ToolMeta {
            icon: theme::ICON_STAR,
            label_id: "ptnd.text.tool.star",
            summary_id: "ptnd.text.tool.star.summary",
        },
        ToolKind::ShapeBuilder => ToolMeta {
            icon: theme::ICON_BOOLEAN,
            label_id: "ptnd.text.tool.shape_builder",
            summary_id: "ptnd.text.tool.shape_builder.summary",
        },
        ToolKind::VectorFloodFill => ToolMeta {
            icon: theme::ICON_FILL,
            label_id: "ptnd.text.tool.vector_flood_fill",
            summary_id: "ptnd.text.tool.vector_flood_fill.summary",
        },
        ToolKind::ArtisticText => ToolMeta {
            icon: theme::ICON_TYPE,
            label_id: "ptnd.text.tool.artistic_text",
            summary_id: "ptnd.text.tool.artistic_text.summary",
        },
        ToolKind::FrameText => ToolMeta {
            icon: theme::ICON_TEXT,
            label_id: "ptnd.text.tool.frame_text",
            summary_id: "ptnd.text.tool.frame_text.summary",
        },
        ToolKind::Gradient => ToolMeta {
            icon: theme::ICON_GRADIENT,
            label_id: "ptnd.text.tool.gradient",
            summary_id: "ptnd.text.tool.gradient.summary",
        },
        ToolKind::Transparency => ToolMeta {
            icon: theme::ICON_TRANSPARENCY,
            label_id: "ptnd.text.tool.transparency",
            summary_id: "ptnd.text.tool.transparency.summary",
        },
        ToolKind::ColorPicker => ToolMeta {
            icon: theme::ICON_COLOR_PICKER,
            label_id: "ptnd.text.tool.eyedropper",
            summary_id: "ptnd.text.tool.eyedropper.summary",
        },
        ToolKind::StylePicker => ToolMeta {
            icon: theme::ICON_ATTRIBUTE_PICKER,
            label_id: "ptnd.text.tool.style_picker",
            summary_id: "ptnd.text.tool.style_picker.summary",
        },
        ToolKind::Artboard => ToolMeta {
            icon: theme::ICON_ARTBOARD,
            label_id: "ptnd.text.tool.surface",
            summary_id: "ptnd.text.tool.surface.summary",
        },
        ToolKind::Measure => ToolMeta {
            icon: theme::ICON_MEASURE,
            label_id: "ptnd.text.tool.measure",
            summary_id: "ptnd.text.tool.measure.summary",
        },
        ToolKind::Zoom => ToolMeta {
            icon: theme::ICON_ZOOM_IN,
            label_id: "ptnd.text.tool.zoom",
            summary_id: "ptnd.text.tool.zoom.summary",
        },
        ToolKind::Hand => ToolMeta {
            icon: theme::ICON_HAND,
            label_id: "ptnd.text.tool.hand",
            summary_id: "ptnd.text.tool.hand.summary",
        },
        ToolKind::MarqueeRect => ToolMeta {
            icon: theme::ICON_MARQUEE_RECT,
            label_id: "ptnd.text.tool.marquee_rect",
            summary_id: "ptnd.text.tool.marquee_rect.summary",
        },
        ToolKind::MarqueeEllipse => ToolMeta {
            icon: theme::ICON_MARQUEE_ELLIPSE,
            label_id: "ptnd.text.tool.marquee_ellipse",
            summary_id: "ptnd.text.tool.marquee_ellipse.summary",
        },
        ToolKind::Lasso => ToolMeta {
            icon: theme::ICON_LASSO,
            label_id: "ptnd.text.tool.lasso",
            summary_id: "ptnd.text.tool.lasso.summary",
        },
        ToolKind::SelectionBrush => ToolMeta {
            icon: theme::ICON_SELECTION_BRUSH,
            label_id: "ptnd.text.tool.selection_brush",
            summary_id: "ptnd.text.tool.selection_brush.summary",
        },
        ToolKind::FloodSelect => ToolMeta {
            icon: theme::ICON_WAND,
            label_id: "ptnd.text.tool.flood_select",
            summary_id: "ptnd.text.tool.flood_select.summary",
        },
        ToolKind::PixelPaintBrush => ToolMeta {
            icon: theme::ICON_PAINT,
            label_id: "ptnd.text.tool.brush",
            summary_id: "ptnd.text.tool.brush.summary",
        },
        ToolKind::PixelFill => ToolMeta {
            icon: theme::ICON_FILL,
            label_id: "ptnd.text.tool.pixel_fill",
            summary_id: "ptnd.text.tool.pixel_fill.summary",
        },
        ToolKind::PixelEraser => ToolMeta {
            icon: theme::ICON_ERASER,
            label_id: "ptnd.text.tool.eraser",
            summary_id: "ptnd.text.tool.eraser.summary",
        },
        ToolKind::PhotoGradient => ToolMeta {
            icon: theme::ICON_GRADIENT,
            label_id: "ptnd.text.tool.gradient",
            summary_id: "ptnd.text.tool.gradient.summary",
        },
        ToolKind::Crop => ToolMeta {
            icon: theme::ICON_CROP_PHOTO,
            label_id: "ptnd.text.tool.crop",
            summary_id: "ptnd.text.tool.crop.summary",
        },
    }
}

/// Menu bar + persona row, driven by `query_menu_bar` and `personas`.
#[derive(Clone, PartialEq)]
pub struct MenuBarRow(pub UiShell);

impl Component for MenuBarRow {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let bar = ui.shell.read().bridge.query_menu_bar();
        let mut preferences = ui.customize_open;
        rect()
            .direction(Direction::Horizontal)
            .content(Content::Flex)
            .width(Size::fill())
            .height(Size::px(theme::MENU_ROW_HEIGHT))
            .background(theme::SURFACE_CHROME_STRONG)
            .padding(Gaps::new_symmetric(0., theme::SPACE_2))
            .cross_align(Alignment::Center)
            .child(brand_slot())
            .child(
                ScrollView::new()
                    .direction(Direction::Horizontal)
                    .show_scrollbar(false)
                    .width(Size::flex(1.))
                    .height(Size::fill())
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .content(Content::Flex)
                            .height(Size::fill())
                            .cross_align(Alignment::Center)
                            .children(bar.families.into_iter().map(|family| FamilyButton {
                                ui: ui.clone(),
                                family,
                            })),
                    ),
            )
            .child(
                StudioButton::new(ui, ui.studio_text("preferences"))
                    .icon(theme::ICON_SETTINGS)
                    .height(theme::MENU_ROW_HEIGHT)
                    .width(Size::px(32.))
                    .on_press(move |_| preferences.set(true)),
            )
    }
}

/// Main toolbar: studios at the leading edge, document/view commands at the trailing edge.
#[derive(Clone, PartialEq)]
pub struct StudioToolbar(pub UiShell);
impl Component for StudioToolbar {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let personas = ui.shell.read().bridge.personas();
        let active = ui.persona.read().clone();
        let mut open = ui.right_studio_open;
        let visible = *open.read();
        let mut export = ui.export_open;
        let shell = ui.shell.read();
        let session = shell.bridge.session();
        let snapping = session.is_some_and(|s| s.view.snapping_enabled);
        let rulers = session.is_some_and(|s| s.view.rulers_visible);
        let has_document = session.is_some();
        drop(shell);
        let mut ruler_shell = ui.shell;
        rect()
            .direction(Direction::Horizontal)
            .content(Content::Flex)
            .width(Size::fill())
            .height(Size::px(theme::PERSONA_ROW_HEIGHT))
            .background(theme::SURFACE_CHROME)
            .padding(Gaps::new_symmetric(0., theme::SPACE_2))
            .cross_align(Alignment::Center)
            .main_align(Alignment::SpaceBetween)
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .spacing(theme::SPACE_1)
                    .cross_align(Alignment::Center)
                    .children(
                        personas.iter().map(|persona| {
                            persona_button(ui.clone(), persona, persona.id == active)
                        }),
                    ),
            )
            .child(
                ScrollView::new()
                    .direction(Direction::Horizontal)
                    .show_scrollbar(false)
                    .width(Size::flex(1.))
                    .height(Size::fill())
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .content(Content::Flex)
                            .height(Size::fill())
                            .cross_align(Alignment::Center)
                            .spacing(theme::SPACE_1)
                            .child(shell_cluster(ui.clone()))
                            .child(snap_toggle(
                                ui.clone(),
                                ui.studio_text("snapping"),
                                snapping,
                            ))
                            .child(
                                StudioButton::new(ui, ui.studio_text("rulers"))
                                    .icon(theme::ICON_MEASURE)
                                    .selected(rulers)
                                    .enabled(has_document)
                                    .on_press(move |_| {
                                        let _ = run_action_token(
                                            &mut ruler_shell.write(),
                                            "ptnd.action.view.toggle_rulers",
                                        );
                                    }),
                            ),
                    ),
            )
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .spacing(theme::SPACE_1)
                    .child(
                        StudioButton::new(
                            ui,
                            ui.studio_text(if visible {
                                "hide_studio"
                            } else {
                                "show_studio"
                            }),
                        )
                        .icon(theme::ICON_LAYERS)
                        .selected(visible)
                        .width(Size::px(32.))
                        .on_press(move |_| open.set(!visible)),
                    )
                    .child(
                        Button::new()
                            .enabled(has_document)
                            .on_press(move |_| export.set(true))
                            .child(ui.text("export")),
                    ),
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
            .content(Content::Flex)
            .cross_align(Alignment::Center)
            .spacing(theme::SPACE_1)
            .children(
                controls
                    .iter()
                    .map(|control| shell_control(self.0.clone(), control)),
            )
    }
}

/// Document tab strip: open document tab, new tab (+), and view toggles.
#[derive(Clone, PartialEq)]
pub struct DocumentTabStrip(pub UiShell);

impl Component for DocumentTabStrip {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let shell = ui.shell.read();
        let active = shell.bridge.active_session_index();
        let tabs = shell
            .bridge
            .sessions()
            .iter()
            .enumerate()
            .map(|(index, s)| {
                (
                    index,
                    s.title().to_owned(),
                    s.is_dirty(),
                    active == Some(index),
                )
            })
            .collect::<Vec<_>>();
        drop(shell);
        let mut open = ui.new_doc_open;
        rect()
            .direction(Direction::Horizontal)
            .content(Content::Flex)
            .width(Size::fill())
            .height(Size::px(theme::TAB_STRIP_HEIGHT))
            .background(theme::SURFACE_CHROME_STRONG)
            .child(
                ScrollView::new()
                    .direction(Direction::Horizontal)
                    .show_scrollbar(false)
                    .height(Size::fill())
                    .width(Size::flex(1.))
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .content(Content::Flex)
                            .height(Size::fill())
                            .spacing(2.)
                            .cross_align(Alignment::Center)
                            .children(tabs.into_iter().map(|(index, title, dirty, selected)| {
                                let mut switch = ui.shell;
                                let close = ui.clone();
                                let mut hovered = ui.hovered;
                                rect()
                                    .direction(Direction::Horizontal)
                                    .content(Content::Flex)
                                    .height(Size::fill())
                                    .width(Size::px(196.))
                                    .cross_align(Alignment::Center)
                                    .background(if selected {
                                        theme::SURFACE_PANEL
                                    } else {
                                        Color::TRANSPARENT
                                    })
                                    .child(
                                        StudioButton::new(ui, title.clone())
                                            .text(title)
                                            .tab()
                                            .selected(selected)
                                            .width(Size::flex(1.))
                                            .on_press(move |_| {
                                                if switch
                                                    .write()
                                                    .bridge
                                                    .switch_session(index)
                                                    .is_ok()
                                                {
                                                    hovered.set(None);
                                                }
                                            }),
                                    )
                                    .maybe_child(dirty.then(|| {
                                        rect()
                                            .width(Size::px(5.))
                                            .height(Size::px(5.))
                                            .corner_radius(3.)
                                            .background(theme::STATE_WARNING)
                                    }))
                                    .child(
                                        StudioButton::new(ui, ui.studio_text("close_tab"))
                                            .icon(theme::ICON_CLOSE)
                                            .width(Size::px(28.))
                                            .on_press(move |_| {
                                                crate::file_workflows::request_close(&close, index)
                                            }),
                                    )
                            })),
                    ),
            )
            .child(
                StudioButton::new(ui, ui.studio_text("new_tab"))
                    .icon(theme::ICON_PLUS)
                    .width(Size::px(32.))
                    .on_press(move |_| open.set(true)),
            )
    }
}

/// Left tool rail: grouped, configurable tools for the active persona.
#[derive(Clone, PartialEq)]
pub struct ToolRail(pub UiShell);

impl Component for ToolRail {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let rail = ui.tool_rail.read().clone();
        let photo = *ui.persona.read() == petunia_design_application::surfaces::PERSONA_PHOTO;
        let groups = rail
            .groups(photo)
            .iter()
            .filter(|g| g.visible && !g.tools.is_empty())
            .cloned()
            .collect::<Vec<_>>();
        let open = use_state(|| None::<String>);
        let columns = if rail.columns == RailColumns::Two {
            2
        } else {
            1
        };
        rect()
            .width(Size::px(theme::TOOL_RAIL_WIDTH * columns as f32))
            .height(Size::fill())
            .background(theme::SURFACE_CHROME)
            .child(
                ScrollView::new()
                    .width(Size::fill())
                    .height(Size::fill())
                    .show_scrollbar(false)
                    .child(
                        rect()
                            .direction(Direction::Vertical)
                            .width(Size::fill())
                            .padding(Gaps::new_symmetric(theme::SPACE_1, 2.))
                            .spacing(theme::TOOL_GAP)
                            .children(groups.chunks(columns).map(|row| {
                                rect()
                                    .direction(Direction::Horizontal)
                                    .content(Content::Flex)
                                    .spacing(2.)
                                    .children(row.iter().map(|group| ToolGroupButton {
                                        ui: ui.clone(),
                                        group: group.clone(),
                                        open_group: open,
                                        open: open.read().as_deref() == Some(group.id.as_str()),
                                    }))
                            })),
                    ),
            )
    }
}

#[derive(Clone, PartialEq)]
struct ToolGroupButton {
    ui: UiShell,
    group: ToolGroupConfig,
    open_group: State<Option<String>>,
    open: bool,
}

impl Component for ToolGroupButton {
    fn render_key(&self) -> DiffKey {
        DiffKey::from(&self.group.id)
    }
    fn render(&self) -> impl IntoElement {
        let primary_id = use_a11y();
        let primary_focus = use_focus(primary_id);
        let variant_id = use_a11y();
        let ui = self.ui.clone();
        let rail_state = ui.tool_rail.read().clone();
        let active_tool = *ui.active_tool.read();
        let selected_tool = if self.group.tools.contains(&active_tool) {
            active_tool
        } else {
            rail_state
                .active_for_group(&self.group)
                .or_else(|| self.group.tools.first().copied())
                .unwrap_or(ToolKind::Select)
        };
        let meta = tool_meta(selected_tool);
        let blocked_reason = tool_disabled_reason(&ui, selected_tool);
        let selectable = blocked_reason.is_empty();
        let title = localized_text(&ui, meta.label_id);
        let summary = if selectable {
            localized_text(&ui, meta.summary_id)
        } else {
            blocked_reason.clone()
        };
        let shortcut = tool_shortcut(selected_tool).to_string();
        let ui_for_press = ui.clone();
        let mut rail = ui.tool_rail;
        let group_id = self.group.id.clone();
        let tool = selected_tool;
        let primary = with_tooltip(
            rect().a11y_id(primary_id).a11y_focusable(selectable).a11y_role(AccessibilityRole::Button).a11y_builder(|node|{node.set_selected(active_tool==selected_tool);if !selectable {node.set_disabled();}})
                .border(Border::new().fill(if primary_focus()==Focus::Keyboard {ui.accent.read().value}else {Color::TRANSPARENT}).width(2.).alignment(BorderAlignment::Inner))
                .width(Size::px(28.))
                .height(Size::px(theme::TOOL_BUTTON))
                .center()
                .background(if active_tool == selected_tool {
                    ui.accent.read().value
                } else {
                    Color::TRANSPARENT
                })
                .on_all_press(move |event:Event<PressEventData>| {
                    event.stop_propagation();
                    if matches!(event.data(),PressEventData::Mouse(mouse) if mouse.button!=Some(MouseButton::Left)){return;}
                    if !selectable {
                        return;
                    }
                    ui_for_press.activate_tool(tool);
                    rail.write().remember_active(&group_id, tool);
                })
                .child(app_icon_sized(
                    meta.icon,
                    *ui.icon_style.read(),
                    if active_tool == selected_tool {
                        theme::TEXT_ON_ACCENT
                    } else if selectable {
                        theme::TEXT_TERTIARY
                    } else {
                        theme::TEXT_DISABLED
                    },
                    theme::ICON_TOOL_RAIL,
                )),
            &ui,
            format!("tool:{}", selected_tool.action_id()),
            title,
            summary,
            shortcut,
        );
        let mut toggle_group = self.open_group;
        let toggle_id = self.group.id.clone();
        let variant = (self.group.tools.len() > 1).then(|| {
            with_tooltip(
                rect().a11y_id(variant_id).a11y_focusable(true).a11y_role(AccessibilityRole::Button)
                    .width(Size::px(24.))
                    .height(Size::px(theme::TOOL_BUTTON))
                    .center()
                    .on_all_press(move |event:Event<PressEventData>| {
                    event.stop_propagation();
                    if matches!(event.data(),PressEventData::Mouse(mouse) if mouse.button!=Some(MouseButton::Left)){return;}
                        let next = if toggle_group
                            .read()
                            .as_deref()
                            .is_some_and(|id| id == toggle_id)
                        {
                            None
                        } else {
                            Some(toggle_id.clone())
                        };
                        toggle_group.set(next);
                    })
                    .child(app_icon(
                        theme::ICON_CHEVRON_DOWN,
                        *ui.icon_style.read(),
                        theme::TEXT_TERTIARY,
                    )),
                &ui,
                format!("tool-group-more:{}", self.group.id),
                localized_text(&ui, "ptnd.text.shell.more_tools"),
                String::new(),
                String::new(),
            )
        });
        let menu = self.open.then(|| {
            let mut close_group = self.open_group;
            let mut close_hovered = ui.hovered;
            Menu::new()
                .on_close(move |_| {
                    close_group.set(None);
                    close_hovered.set(None);
                })
                .children(
                    self.group
                        .tools
                        .iter()
                        .map(|tool| tool_menu_item(ui.clone(), *tool, &self.group.id, close_group)),
                )
        });
        rect()
            .direction(Direction::Vertical)
            .cross_align(Alignment::Start)
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .child(primary)
                    .maybe_child(variant),
            )
            .maybe_child(menu.map(|menu| {
                Portal::new(format!("tool-group:{}", self.group.id))
                    .width(Size::px(0.))
                    .height(Size::px(0.))
                    .child(menu)
            }))
    }
}

fn tool_menu_item(
    ui: UiShell,
    tool: ToolKind,
    group_id: &str,
    open_group: State<Option<String>>,
) -> impl IntoElement {
    let meta = tool_meta(tool);
    let title = localized_text(&ui, meta.label_id);
    let ui_for_press = ui.clone();
    let mut rail = ui.tool_rail;
    let active = *ui.active_tool.read() == tool;
    let icon_style = *ui.icon_style.read();
    let mut open_group = open_group;
    let mut enter_hovered = ui.hovered;
    let hover_key = format!("tool:{}", tool.action_id());
    let blocked_reason = tool_disabled_reason(&ui, tool);
    let selectable = blocked_reason.is_empty();
    let summary = if selectable {
        tool_summary(&ui, tool)
    } else {
        blocked_reason.clone()
    };
    let shortcut = tool_shortcut(tool).to_string();
    let hover_shortcut = shortcut.clone();
    let hover_title = title.clone();
    let group_id = group_id.to_string();
    MenuItem::new()
        .on_pointer_enter(move |event: Event<PointerEventData>| {
            let point = event.global_location();
            enter_hovered.set(Some(HoverTarget {
                id: hover_key.clone(),
                title: hover_title.clone(),
                summary: summary.clone(),
                shortcut: hover_shortcut.clone(),
                x: point.x,
                y: point.y,
            }));
        })
        .on_press(move |_| {
            if !selectable {
                return;
            }
            ui_for_press.activate_tool(tool);
            rail.write().remember_active(&group_id, tool);
            open_group.set(None);
        })
        .child(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .spacing(theme::SPACE_1)
                .child(app_icon(
                    meta.icon,
                    icon_style,
                    if active {
                        theme::TEXT_PRIMARY
                    } else if selectable {
                        theme::TEXT_TERTIARY
                    } else {
                        theme::TEXT_DISABLED
                    },
                ))
                .child(label().text(title).font_size(theme::BODY_SIZE))
                .maybe_child(
                    active.then(|| app_icon(theme::ICON_CHECK, icon_style, theme::TEXT_PRIMARY)),
                )
                .maybe_child((!shortcut.is_empty()).then(|| {
                    label()
                        .text(shortcut.clone())
                        .color(theme::TEXT_TERTIARY)
                        .font_size(theme::CAPTION_SIZE)
                })),
        )
}

pub fn resolve_tool_shortcut(
    key: &str,
    photo: bool,
    rail_state: &ToolRailState,
    current_tool: ToolKind,
) -> Option<ToolKind> {
    let groups = rail_state.groups(photo);
    let matching_groups: Vec<_> = groups
        .iter()
        .filter(|group| {
            group
                .tools
                .iter()
                .any(|tool| tool_shortcut(*tool).eq_ignore_ascii_case(key))
        })
        .collect();
    let group = matching_groups
        .iter()
        .copied()
        .find(|group| group.tools.contains(&current_tool))
        .or_else(|| matching_groups.first().copied())?;
    let matching_tools: Vec<_> = group
        .tools
        .iter()
        .copied()
        .filter(|tool| tool_shortcut(*tool).eq_ignore_ascii_case(key))
        .collect();
    if matching_tools.len() <= 1 {
        return matching_tools.first().copied();
    }
    let index = matching_tools
        .iter()
        .position(|tool| *tool == current_tool)
        .map_or(0, |index| (index + 1) % matching_tools.len());
    matching_tools.get(index).copied()
}

pub fn shortcut_key(event: &Event<KeyboardEventData>) -> Option<&str> {
    match &event.key {
        Key::Character(key) if key == " " => Some("Space"),
        Key::Character(key) => Some(key.as_str()),
        _ => None,
    }
}

pub fn tool_label(ui: &UiShell, tool: ToolKind) -> String {
    localized_text(ui, tool_meta(tool).label_id)
}

pub fn tool_summary(ui: &UiShell, tool: ToolKind) -> String {
    localized_text(ui, tool_meta(tool).summary_id)
}

pub fn tool_shortcut(tool: ToolKind) -> &'static str {
    petunia_design_application::surfaces::SURFACES
        .iter()
        .find(|entry| {
            entry.kind == petunia_design_application::surfaces::SurfaceKind::Tool
                && entry.action == Some(tool.action_id())
                && entry.status == petunia_design_application::surfaces::SurfaceStatus::Wired
        })
        .and_then(|entry| entry.shortcut)
        .unwrap_or("")
}

/// Registry row for a tool, so the rail can refuse what the registry blocks.
fn tool_surface(
    tool: ToolKind,
) -> Option<&'static petunia_design_application::surfaces::SurfaceEntry> {
    petunia_design_application::surfaces::SURFACES
        .iter()
        .find(|entry| {
            entry.kind == petunia_design_application::surfaces::SurfaceKind::Tool
                && entry.action == Some(tool.action_id())
        })
}

/// Registry reason id for a blocked tool, or `None` when it is wired.
///
/// Kept toolkit-free so the availability rule can be tested without mounting
/// the rail: the registry decides, and the rail only renders the answer.
#[must_use]
pub fn tool_disabled_reason_id(tool: ToolKind) -> Option<&'static str> {
    match tool_surface(tool).map(|entry| entry.status) {
        Some(petunia_design_application::surfaces::SurfaceStatus::Disabled(reason)) => Some(reason),
        Some(petunia_design_application::surfaces::SurfaceStatus::Absent) => {
            Some("ptnd.text.blocked.not_implemented")
        }
        _ => None,
    }
}

/// Localized disabled reason for a tool, or empty when it is wired.
///
/// The registry is the single source of truth: a tool whose row is `Disabled`
/// or `Absent` must not be activatable from the rail, or the user gets a silent
/// no-op instead of a stated limitation.
pub fn tool_disabled_reason(ui: &UiShell, tool: ToolKind) -> String {
    match tool_disabled_reason_id(tool) {
        Some(reason) => localized_text(ui, reason),
        None => String::new(),
    }
}

fn localized_text(ui: &UiShell, text_id: &str) -> String {
    let shell = ui.shell.read();
    let bridge = &shell.bridge;
    bridge.localization().text(text_id, bridge.locale())
}

/// Canonical context toolbar (08.23), scoped by the shell to the active tool.
#[derive(Clone, PartialEq)]
pub struct ContextToolbar(pub UiShell);

impl Component for ContextToolbar {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let shell_ref = ui.shell.peek();
        let tool = *ui.active_tool.read();
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
        if let Some(tool_controls) = tool_quick_controls(ui, tool) {
            if groups.len() > 1 {
                groups[1].push(tool_controls);
            } else {
                groups[0].push(tool_controls);
            }
        }
        groups
            .last_mut()
            .expect("groups never empty")
            .push(customize_button(ui.clone()).into_element());
        rect()
            .width(Size::fill())
            .height(Size::px(theme::TOOLBAR_HEIGHT))
            .background(theme::SURFACE_PANEL)
            .child(
                ScrollView::new()
                    .direction(Direction::Horizontal)
                    .show_scrollbar(false)
                    .width(Size::fill())
                    .height(Size::fill())
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .content(Content::Flex)
                            .height(Size::fill())
                            .padding(Gaps::new_symmetric(0., theme::SPACE_2))
                            .spacing(theme::SPACE_3)
                            .cross_align(Alignment::Center)
                            .children(groups.into_iter().filter(|group| !group.is_empty()).map(
                                |group| {
                                    rect()
                                        .direction(Direction::Horizontal)
                                        .content(Content::Flex)
                                        .spacing(theme::SPACE_1)
                                        .cross_align(Alignment::Center)
                                        .children(group)
                                },
                            )),
                    ),
            )
    }
}

fn tool_quick_controls(ui: &UiShell, tool: ToolKind) -> Option<Element> {
    let mut shell = ui.shell;
    match tool {
        ToolKind::Rectangle => Some(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .spacing(theme::SPACE_1)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text(ui.studio_text("corners"))
                        .color(theme::TEXT_SECONDARY)
                        .font_size(theme::CAPTION_SIZE),
                )
                .child(quick_action_btn(ui,ui.studio_text("bake_corners"), move |_| {
                    let _ = run_action_token(&mut shell.write(), "ptnd.action.object.bake_corners");
                }))
                .child(quick_action_btn(ui,ui.studio_text("curves"), move |_| {
                    let _ = run_action_token(&mut shell.write(), "ptnd.action.object.convert_to_curves");
                }))
                .into_element(),
        ),
        ToolKind::Corner => Some(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .spacing(theme::SPACE_1)
                .cross_align(Alignment::Center)
                .child(quick_action_btn(ui,ui.studio_text("bake_corners"), move |_| {
                    let _ = run_action_token(&mut shell.write(), "ptnd.action.object.bake_corners");
                }))
                .child(quick_action_btn(ui,ui.studio_text("curves"), move |_| {
                    let _ = run_action_token(&mut shell.write(), "ptnd.action.object.convert_to_curves");
                }))
                .into_element(),
        ),
        ToolKind::Node => Some(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .spacing(theme::SPACE_1)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text(ui.studio_text("node_options"))
                        .color(theme::TEXT_SECONDARY)
                        .font_size(theme::CAPTION_SIZE),
                )
                .child(quick_action_btn(ui,ui.studio_text("cusp"), move |_| {
                    let _ = shell.write().convert_selected_nodes(petunia_design_shell::tools::NodeType::Cusp);
                }))
                .child(quick_action_btn(ui,ui.studio_text("smooth"), move |_| {
                    let _ = shell.write().convert_selected_nodes(petunia_design_shell::tools::NodeType::Smooth);
                }))
                .child(quick_action_btn(ui,ui.studio_text("symmetric"), move |_| {
                    let _ = shell.write().convert_selected_nodes(petunia_design_shell::tools::NodeType::Symmetric);
                }))
                .child(quick_action_btn(ui,ui.studio_text("delete"), move |_| {
                    let _ = shell.write().delete_selected_nodes();
                }))
                .child(quick_action_btn(ui,ui.studio_text("curves"), move |_| {
                    let _ = run_action_token(&mut shell.write(), "ptnd.action.object.convert_to_curves");
                }))
                .into_element(),
        ),
        ToolKind::Pen => {
            let mode = shell.peek().tools.pen_tool().mode();
            Some(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .spacing(theme::SPACE_1)
                    .cross_align(Alignment::Center)
                    .child(
                        label()
                            .text(ui.studio_text("pen_options"))
                            .color(theme::TEXT_SECONDARY)
                            .font_size(theme::CAPTION_SIZE),
                    )
                    .child(quick_action_btn(ui,
                        format!("{}{}",ui.studio_text("bezier"),if mode == petunia_design_shell::tools::PenMode::Bezier {" ✓"} else {""}),
                        move |_| {
                            shell.write().tools.pen_tool_mut().set_mode(petunia_design_shell::tools::PenMode::Bezier);
                        },
                    ))
                    .child(quick_action_btn(ui,
                        if mode == petunia_design_shell::tools::PenMode::Polygon { format!("{} ✓",ui.studio_text("polygon")) } else { ui.studio_text("polygon") },
                        move |_| {
                            shell.write().tools.pen_tool_mut().set_mode(petunia_design_shell::tools::PenMode::Polygon);
                        },
                    ))
                    .child(quick_action_btn(ui,
                        if mode == petunia_design_shell::tools::PenMode::Line { format!("{} ✓",ui.studio_text("line")) } else { ui.studio_text("line") },
                        move |_| {
                            shell.write().tools.pen_tool_mut().set_mode(petunia_design_shell::tools::PenMode::Line);
                        },
                    ))
                    .child(quick_action_btn(ui,ui.studio_text("finish"), move |_| {
                        let _ = shell.write().finish_open_path();
                    }))
                    .child(quick_action_btn(ui,ui.studio_text("curves"), move |_| {
                        let _ = run_action_token(&mut shell.write(), "ptnd.action.object.convert_to_curves");
                    }))
                    .into_element(),
            )
        }
        ToolKind::Pencil => {
            let fidelity = shell.peek().tools.pencil_tool().fidelity();
            Some(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .spacing(theme::SPACE_1)
                    .cross_align(Alignment::Center)
                    .child(
                        label()
                            .text(ui.studio_text("pencil_options"))
                            .color(theme::TEXT_SECONDARY)
                            .font_size(theme::CAPTION_SIZE),
                    )
                    .child(quick_action_btn(ui,
                        format!("{}{}",ui.studio_text("precise"),if fidelity == petunia_design_shell::tools::PencilFidelity::Precise {" ✓"} else {""}),
                        move |_| {
                            shell.write().tools.pencil_tool_mut().set_fidelity(petunia_design_shell::tools::PencilFidelity::Precise);
                        },
                    ))
                    .child(quick_action_btn(ui,
                        format!("{}{}",ui.studio_text("balanced"),if fidelity == petunia_design_shell::tools::PencilFidelity::Balanced {" ✓"} else {""}),
                        move |_| {
                            shell.write().tools.pencil_tool_mut().set_fidelity(petunia_design_shell::tools::PencilFidelity::Balanced);
                        },
                    ))
                    .child(quick_action_btn(ui,
                        format!("{}{}",ui.studio_text("smooth"),if fidelity == petunia_design_shell::tools::PencilFidelity::Smooth {" ✓"} else {""}),
                        move |_| {
                            shell.write().tools.pencil_tool_mut().set_fidelity(petunia_design_shell::tools::PencilFidelity::Smooth);
                        },
                    ))
                    .child(quick_action_btn(ui,ui.studio_text("curves"), move |_| {
                        let _ = run_action_token(&mut shell.write(), "ptnd.action.object.convert_to_curves");
                    }))
                    .into_element(),
            )
        }
        ToolKind::PixelPaintBrush | ToolKind::PixelEraser => {
            let is_brush = tool == ToolKind::PixelPaintBrush;
            let settings = if is_brush {
                shell.peek().tools.photo_brush_tool().brush_settings()
            } else {
                shell.peek().tools.photo_eraser_tool().brush_settings()
            };
            let radius = settings.radius;
            let hardness = (settings.hardness * 100.0).round() as u32;
            let opacity = (settings.opacity * 100.0).round() as u32;

            Some(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .spacing(theme::SPACE_1)
                    .cross_align(Alignment::Center)
                    .child(
                        label()
                            .text(ui.studio_text(if is_brush {"brush"} else {"eraser"}))
                            .color(theme::TEXT_SECONDARY)
                            .font_size(theme::CAPTION_SIZE),
                    )
                    .child(
                        label()
                            .text(format!("{:.0}px", radius))
                            .color(theme::TEXT_PRIMARY)
                            .font_size(theme::CAPTION_SIZE),
                    )
                    .child(quick_action_btn(ui,ui.studio_text("radius_less"), move |_| {
                        let mut sh = shell.write();
                        let target = if is_brush {
                            sh.tools.photo_brush_tool_mut()
                        } else {
                            sh.tools.photo_eraser_tool_mut()
                        };
                        let mut s = target.brush_settings();
                        s.radius = (s.radius - 4.0).clamp(2.0, 256.0);
                        target.set_brush_settings(s);
                    }))
                    .child(quick_action_btn(ui,ui.studio_text("radius_more"), move |_| {
                        let mut sh = shell.write();
                        let target = if is_brush {
                            sh.tools.photo_brush_tool_mut()
                        } else {
                            sh.tools.photo_eraser_tool_mut()
                        };
                        let mut s = target.brush_settings();
                        s.radius = (s.radius + 4.0).clamp(2.0, 256.0);
                        target.set_brush_settings(s);
                    }))
                    .child(quick_action_btn(ui,format!("{} {}%",ui.studio_text("hardness"), hardness), move |_| {
                        let mut sh = shell.write();
                        let target = if is_brush {
                            sh.tools.photo_brush_tool_mut()
                        } else {
                            sh.tools.photo_eraser_tool_mut()
                        };
                        let mut s = target.brush_settings();
                        s.hardness = if s.hardness < 0.3 { 0.5 } else if s.hardness < 0.8 { 1.0 } else { 0.0 };
                        target.set_brush_settings(s);
                    }))
                    .child(quick_action_btn(ui,format!("{} {}%",ui.studio_text("opacity"), opacity), move |_| {
                        let mut sh = shell.write();
                        let target = if is_brush {
                            sh.tools.photo_brush_tool_mut()
                        } else {
                            sh.tools.photo_eraser_tool_mut()
                        };
                        let mut s = target.brush_settings();
                        s.opacity = if s.opacity < 0.4 { 0.5 } else if s.opacity < 0.8 { 1.0 } else { 0.25 };
                        target.set_brush_settings(s);
                    }))
                    .into_element(),
            )
        }
        ToolKind::Select => Some(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .spacing(theme::SPACE_1)
                .cross_align(Alignment::Center)
                .child(quick_action_btn(ui,ui.studio_text("union"), move |_| {
                    let _ = run_action_token(&mut shell.write(), "ptnd.action.object.boolean#union");
                }))
                .child(quick_action_btn(ui,ui.studio_text("subtract"), move |_| {
                    let _ = run_action_token(&mut shell.write(), "ptnd.action.object.boolean#subtract");
                }))
                .child(quick_action_btn(ui,ui.studio_text("intersection"), move |_| {
                    let _ = run_action_token(&mut shell.write(), "ptnd.action.object.boolean#intersect");
                }))
                .into_element(),
        ),
        ToolKind::Star => Some(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .spacing(theme::SPACE_1)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text(ui.studio_text("star_shape"))
                        .color(theme::TEXT_SECONDARY)
                        .font_size(theme::CAPTION_SIZE),
                )
                .child(quick_action_btn(ui,ui.studio_text("points_less"), move |_| {
                    let maybe_change = {
                        let sh = shell.peek();
                        sh.bridge.selection().selected_ids.first().copied().and_then(|id| {
                            let obj = sh.bridge.session().and_then(|s| s.find_object(id));
                            if let Some(obj) = obj {
                                if let Some(petunia_design_document::ShapeKind::Star { points, inner_ratio }) = obj.shape {
                                    return Some((id, points, inner_ratio));
                                }
                            }
                            None
                        })
                    };
                    if let Some((id, points, inner_ratio)) = maybe_change {
                        let new_points = (points as i32 - 1).clamp(3, 36) as u32;
                        let _ = shell.write().bridge.submit_all(
                            "Change star points",
                            vec![petunia_design_application::Command::SetShape {
                                id,
                                shape: Some(petunia_design_document::ShapeKind::Star { points: new_points, inner_ratio }),
                            }],
                        );
                    }
                }))
                .child(quick_action_btn(ui,ui.studio_text("points_more"), move |_| {
                    let maybe_change = {
                        let sh = shell.peek();
                        sh.bridge.selection().selected_ids.first().copied().and_then(|id| {
                            let obj = sh.bridge.session().and_then(|s| s.find_object(id));
                            if let Some(obj) = obj {
                                if let Some(petunia_design_document::ShapeKind::Star { points, inner_ratio }) = obj.shape {
                                    return Some((id, points, inner_ratio));
                                }
                            }
                            None
                        })
                    };
                    if let Some((id, points, inner_ratio)) = maybe_change {
                        let new_points = (points as i32 + 1).clamp(3, 36) as u32;
                        let _ = shell.write().bridge.submit_all(
                            "Change star points",
                            vec![petunia_design_application::Command::SetShape {
                                id,
                                shape: Some(petunia_design_document::ShapeKind::Star { points: new_points, inner_ratio }),
                            }],
                        );
                    }
                }))
                .child(quick_action_btn(ui,ui.studio_text("curves"), move |_| {
                    let _ = run_action_token(&mut shell.write(), "ptnd.action.object.convert_to_curves");
                }))
                .into_element(),
        ),
        ToolKind::Polygon => Some(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .spacing(theme::SPACE_1)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text(ui.studio_text("polygon_shape"))
                        .color(theme::TEXT_SECONDARY)
                        .font_size(theme::CAPTION_SIZE),
                )
                .child(quick_action_btn(ui,ui.studio_text("sides_less"), move |_| {
                    let maybe_change = {
                        let sh = shell.peek();
                        sh.bridge.selection().selected_ids.first().copied().and_then(|id| {
                            let obj = sh.bridge.session().and_then(|s| s.find_object(id));
                            if let Some(obj) = obj {
                                if let Some(petunia_design_document::ShapeKind::Polygon { sides }) = obj.shape {
                                    return Some((id, sides));
                                }
                            }
                            None
                        })
                    };
                    if let Some((id, sides)) = maybe_change {
                        let new_sides = (sides as i32 - 1).clamp(3, 36) as u32;
                        let _ = shell.write().bridge.submit_all(
                            "Change polygon sides",
                            vec![petunia_design_application::Command::SetShape {
                                id,
                                shape: Some(petunia_design_document::ShapeKind::Polygon { sides: new_sides }),
                            }],
                        );
                    }
                }))
                .child(quick_action_btn(ui,ui.studio_text("sides_more"), move |_| {
                    let maybe_change = {
                        let sh = shell.peek();
                        sh.bridge.selection().selected_ids.first().copied().and_then(|id| {
                            let obj = sh.bridge.session().and_then(|s| s.find_object(id));
                            if let Some(obj) = obj {
                                if let Some(petunia_design_document::ShapeKind::Polygon { sides }) = obj.shape {
                                    return Some((id, sides));
                                }
                            }
                            None
                        })
                    };
                    if let Some((id, sides)) = maybe_change {
                        let new_sides = (sides as i32 + 1).clamp(3, 36) as u32;
                        let _ = shell.write().bridge.submit_all(
                            "Change polygon sides",
                            vec![petunia_design_application::Command::SetShape {
                                id,
                                shape: Some(petunia_design_document::ShapeKind::Polygon { sides: new_sides }),
                            }],
                        );
                    }
                }))
                .child(quick_action_btn(ui,ui.studio_text("curves"), move |_| {
                    let _ = run_action_token(&mut shell.write(), "ptnd.action.object.convert_to_curves");
                }))
                .into_element(),
        ),
        ToolKind::Contour => Some(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .spacing(theme::SPACE_1)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text(ui.studio_text("add_contour"))
                        .color(theme::TEXT_SECONDARY)
                        .font_size(theme::CAPTION_SIZE),
                )
                .child(quick_action_btn(ui,"-2pt", move |_| {
                    let maybe_target = {
                        let sh = shell.peek();
                        sh.bridge.selection().selected_ids.first().copied().map(|id| {
                            let dist = sh
                                .bridge
                                .session()
                                .and_then(|s| s.find_object(id))
                                .and_then(|o| {
                                    o.modifiers.iter().find_map(|m| {
                                        if let petunia_design_document::ModifierKind::ContourOffset { distance, .. } = m.kind {
                                            Some(distance)
                                        } else {
                                            None
                                        }
                                    })
                                })
                                .unwrap_or(0.0);
                            (id, dist)
                        })
                    };
                    if let Some((id, current_dist)) = maybe_target {
                        let _ = shell.write().bridge.submit_all(
                            "Adjust contour",
                            vec![petunia_design_application::Command::OffsetPath { id, delta: current_dist - 2.0 }],
                        );
                    }
                }))
                .child(quick_action_btn(ui,"+2pt", move |_| {
                    let maybe_target = {
                        let sh = shell.peek();
                        sh.bridge.selection().selected_ids.first().copied().map(|id| {
                            let dist = sh
                                .bridge
                                .session()
                                .and_then(|s| s.find_object(id))
                                .and_then(|o| {
                                    o.modifiers.iter().find_map(|m| {
                                        if let petunia_design_document::ModifierKind::ContourOffset { distance, .. } = m.kind {
                                            Some(distance)
                                        } else {
                                            None
                                        }
                                    })
                                })
                                .unwrap_or(0.0);
                            (id, dist)
                        })
                    };
                    if let Some((id, current_dist)) = maybe_target {
                        let _ = shell.write().bridge.submit_all(
                            "Adjust contour",
                            vec![petunia_design_application::Command::OffsetPath { id, delta: current_dist + 2.0 }],
                        );
                    }
                }))
                .child(quick_action_btn(ui,ui.studio_text("bake_contour"), move |_| {
                    let target_id = shell.peek().bridge.selection().selected_ids.first().copied();
                    if let Some(id) = target_id {
                        let _ = shell.write().bridge.submit_all(
                            "Bake contour",
                            vec![petunia_design_application::Command::BakeContour { id }],
                        );
                    }
                }))
                .into_element(),
        ),
        ToolKind::Perspective => Some(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .spacing(theme::SPACE_1)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text(ui.studio_text("perspective"))
                        .color(theme::TEXT_SECONDARY)
                        .font_size(theme::CAPTION_SIZE),
                )
                .child(quick_action_btn(ui,ui.studio_text("bake_geometry"), move |_| {
                    let target_id = shell.peek().bridge.selection().selected_ids.first().copied();
                    if let Some(id) = target_id {
                        let _ = shell.write().bridge.submit_all(
                            "Bake perspective",
                            vec![petunia_design_application::Command::BakeGeometry { id }],
                        );
                    }
                }))
                .into_element(),
        ),
        ToolKind::ArtisticText | ToolKind::FrameText => Some(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .spacing(theme::SPACE_1)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text(ui.studio_text("text_options"))
                        .color(theme::TEXT_SECONDARY)
                        .font_size(theme::CAPTION_SIZE),
                )
                .child(quick_action_btn(ui,ui.studio_text("curves"), move |_| {
                    let _ = run_action_token(&mut shell.write(), "ptnd.action.object.convert_to_curves");
                }))
                .into_element(),
        ),
        ToolKind::MarqueeRect | ToolKind::MarqueeEllipse | ToolKind::Lasso => Some(
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .spacing(theme::SPACE_1)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text(ui.studio_text("pixel_selection"))
                        .color(theme::TEXT_SECONDARY)
                        .font_size(theme::CAPTION_SIZE),
                )
                .child(quick_action_btn(ui,ui.studio_text("deselect"), move |_| {
                    let _ = run_action_token(&mut shell.write(), "ptnd.action.select.deselect");
                }))
                .into_element(),
        ),
        ToolKind::Measure => {
            let mode = shell.peek().tools.measure_tool().mode();
            Some(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .spacing(theme::SPACE_1)
                    .cross_align(Alignment::Center)
                    .child(
                        label()
                            .text(ui.studio_text("measure"))
                            .color(theme::TEXT_SECONDARY)
                            .font_size(theme::CAPTION_SIZE),
                    )
                    .child(quick_action_btn(ui,
                        format!("{}{}",ui.studio_text("distance"),if mode == petunia_design_shell::tools::MeasureMode::Distance {" ✓"} else {""}),
                        move |_| {
                            shell.write().tools.measure_tool_mut().set_mode(petunia_design_shell::tools::MeasureMode::Distance);
                        },
                    ))
                    .child(quick_action_btn(ui,
                        format!("{}{}",ui.studio_text("area"),if mode == petunia_design_shell::tools::MeasureMode::Area {" ✓"} else {""}),
                        move |_| {
                            shell.write().tools.measure_tool_mut().set_mode(petunia_design_shell::tools::MeasureMode::Area);
                        },
                    ))
                    .child(quick_action_btn(ui,ui.studio_text("delete_selection"), move |_| {
                        shell.write().tools.measure_tool_mut().cancel();
                    }))
                    .into_element(),
            )
        }
        ToolKind::Gradient | ToolKind::Transparency => {
            let kind = shell.peek().tools.gradient_tool().kind();
            Some(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .spacing(theme::SPACE_1)
                    .cross_align(Alignment::Center)
                    .child(
                        label()
                            .text(ui.studio_text("gradient"))
                            .color(theme::TEXT_SECONDARY)
                            .font_size(theme::CAPTION_SIZE),
                    )
                    .child(quick_action_btn(ui,
                        format!("{}{}",ui.studio_text("linear"),if kind == petunia_design_shell::tools::GradientKind::Linear {" ✓"} else {""}),
                        move |_| {
                            shell.write().tools.gradient_tool_mut().set_kind(petunia_design_shell::tools::GradientKind::Linear);
                        },
                    ))
                    .child(quick_action_btn(ui,
                        format!("{}{}",ui.studio_text("radial"),if kind == petunia_design_shell::tools::GradientKind::Radial {" ✓"} else {""}),
                        move |_| {
                            shell.write().tools.gradient_tool_mut().set_kind(petunia_design_shell::tools::GradientKind::Radial);
                        },
                    ))
                    .into_element(),
            )
        }
        ToolKind::StylePicker => {
            let filter = shell.peek().tools.style_picker_tool().filter();
            Some(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .spacing(theme::SPACE_1)
                    .cross_align(Alignment::Center)
                    .child(
                        label()
                            .text(ui.studio_text("copy_style"))
                            .color(theme::TEXT_SECONDARY)
                            .font_size(theme::CAPTION_SIZE),
                    )
                    .child(quick_action_btn(ui,
                        format!("{}{}",ui.studio_text("fill"),if filter.fill {" ✓"} else {""}),
                        move |_| {
                            let mut sh = shell.write();
                            let cur = sh.tools.style_picker_tool().filter();
                            sh.tools.style_picker_tool_mut().set_filter(petunia_design_shell::tools::StyleFilter {
                                fill: !cur.fill,
                                ..cur
                            });
                        },
                    ))
                    .child(quick_action_btn(ui,
                        if filter.stroke { format!("{} ✓",ui.studio_text("stroke")) } else { ui.studio_text("stroke") },
                        move |_| {
                            let mut sh = shell.write();
                            let cur = sh.tools.style_picker_tool().filter();
                            sh.tools.style_picker_tool_mut().set_filter(petunia_design_shell::tools::StyleFilter {
                                stroke: !cur.stroke,
                                ..cur
                            });
                        },
                    ))
                    .child(quick_action_btn(ui,
                        format!("{}{}",ui.studio_text("effects"),if filter.effects {" ✓"} else {""}),
                        move |_| {
                            let mut sh = shell.write();
                            let cur = sh.tools.style_picker_tool().filter();
                            sh.tools.style_picker_tool_mut().set_filter(petunia_design_shell::tools::StyleFilter {
                                effects: !cur.effects,
                                ..cur
                            });
                        },
                    ))
                    .child(quick_action_btn(ui,
                        format!("{}{}",ui.studio_text("typography"),if filter.typography {" ✓"} else {""}),
                        move |_| {
                            let mut sh = shell.write();
                            let cur = sh.tools.style_picker_tool().filter();
                            sh.tools.style_picker_tool_mut().set_filter(petunia_design_shell::tools::StyleFilter {
                                typography: !cur.typography,
                                ..cur
                            });
                        },
                    ))
                    .into_element(),
            )
        }
        ToolKind::ShapeBuilder => {
            let op = shell.peek().tools.shape_builder_tool().op();
            Some(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .spacing(theme::SPACE_1)
                    .cross_align(Alignment::Center)
                    .child(
                        label()
                            .text(ui.studio_text("shape_builder"))
                            .color(theme::TEXT_SECONDARY)
                            .font_size(theme::CAPTION_SIZE),
                    )
                    .child(quick_action_btn(ui,
                        format!("{}{}",ui.studio_text("add"),if op == petunia_design_shell::tools::BuilderOp::Add {" ✓"} else {""}),
                        move |_| {
                            shell.write().tools.shape_builder_tool_mut().set_op(petunia_design_shell::tools::BuilderOp::Add);
                        },
                    ))
                    .child(quick_action_btn(ui,
                        if op == petunia_design_shell::tools::BuilderOp::Subtract { format!("{} ✓",ui.studio_text("subtract")) } else { ui.studio_text("subtract") },
                        move |_| {
                            shell.write().tools.shape_builder_tool_mut().set_op(petunia_design_shell::tools::BuilderOp::Subtract);
                        },
                    ))
                    .into_element(),
            )
        }
        ToolKind::VectorFloodFill => {
            let current_token = shell.peek().tools.smart_fill_tool().fill_token().to_string();
            Some(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .spacing(theme::SPACE_1)
                    .cross_align(Alignment::Center)
                    .child(
                        label()
                            .text(ui.studio_text("interactive_fill"))
                            .color(theme::TEXT_SECONDARY)
                            .font_size(theme::CAPTION_SIZE),
                    )
                    .child(quick_action_btn(ui,
                        format!("{}{}",ui.studio_text("blue"),if current_token == "ptnd.blue/500" {" ✓"} else {""}),
                        move |_| {
                            shell.write().tools.smart_fill_tool_mut().set_fill_token("ptnd.blue/500");
                        },
                    ))
                    .child(quick_action_btn(ui,
                        format!("{}{}",ui.studio_text("red"),if current_token == "ptnd.red/500" {" ✓"} else {""}),
                        move |_| {
                            shell.write().tools.smart_fill_tool_mut().set_fill_token("ptnd.red/500");
                        },
                    ))
                    .child(quick_action_btn(ui,
                        format!("{}{}",ui.studio_text("green"),if current_token == "ptnd.green/500" {" ✓"} else {""}),
                        move |_| {
                            shell.write().tools.smart_fill_tool_mut().set_fill_token("ptnd.green/500");
                        },
                    ))
                    .child(quick_action_btn(ui,
                        format!("{}{}",ui.studio_text("yellow"),if current_token == "ptnd.yellow/500" {" ✓"} else {""}),
                        move |_| {
                            shell.write().tools.smart_fill_tool_mut().set_fill_token("ptnd.yellow/500");
                        },
                    ))
                    .into_element(),
            )
        }
        _ => None,
    }
}

fn quick_action_btn(
    ui: &UiShell,
    title: impl Into<String>,
    on_press: impl FnMut(Event<PressEventData>) + 'static,
) -> Element {
    let title = title.into();
    StudioButton::new(ui, title.clone())
        .text(title)
        .on_press(on_press)
        .into_element()
}

fn brand_slot() -> impl IntoElement {
    rect()
        .width(Size::px(theme::BRAND_MARK_SIZE))
        .height(Size::px(theme::BRAND_MARK_SIZE))
        .background(theme::SURFACE_CHROME_STRONG)
        .child(label().text("p").font_size(18.).color(theme::ACCENT_BLOOM))
}

#[derive(Clone, PartialEq)]
struct FamilyButton {
    ui: UiShell,
    family: MenuFamilyPresentation,
}

impl Component for FamilyButton {
    fn render(&self) -> impl IntoElement {
        let ui = self.ui.clone();
        let family_id = self.family.id.clone();
        let is_open = ui.open_family.read().as_deref() == Some(family_id.as_str());
        let mut open_family = ui.open_family;
        let button = StudioButton::new(&ui, self.family.label.clone())
            .text(self.family.label.clone())
            .selected(is_open)
            .height(theme::MENU_ROW_HEIGHT)
            .on_press(move |_| {
                open_family.set(if is_open {
                    None
                } else {
                    Some(family_id.clone())
                })
            });
        let menu = is_open.then(|| {
            let mut close_state = ui.open_family;
            let mut close_hovered = ui.hovered;
            Menu::new()
                .on_close(move |_| {
                    close_state.set(None);
                    close_hovered.set(None);
                })
                .children(
                    self.family
                        .nodes
                        .iter()
                        .map(|node| menu_node(ui.clone(), node)),
                )
        });
        rect()
            .direction(Direction::Vertical)
            .cross_align(Alignment::Start)
            .child(button)
            .maybe_child(menu.map(|menu| {
                Portal::new(self.family.id.clone())
                    .width(Size::px(0.))
                    .height(Size::px(0.))
                    .child(menu)
            }))
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
    let mut open_family = ui.open_family;
    let mut active_tool = ui.active_tool;
    let mut tool_rail = ui.tool_rail;
    let mut customize_open = ui.customize_open;
    let mut offset_prompt_open = ui.offset_prompt_open;
    let photo = *ui.persona.read() == petunia_design_application::surfaces::PERSONA_PHOTO;

    let summary = action_summary(&ui, &item.action_id);
    let hover_key = hover_id("menu", &item.action_id);
    let mut enter_hovered = ui.hovered;
    let title = item.label.clone();
    let shortcut = item.shortcut.clone();
    let row = MenuItem::new()
        .on_pointer_enter(move |event: Event<PointerEventData>| {
            let point = event.global_location();
            enter_hovered.set(Some(HoverTarget {
                id: hover_key.clone(),
                title: title.clone(),
                summary: summary.clone(),
                shortcut: shortcut.clone(),
                x: point.x,
                y: point.y,
            }));
        })
        .on_press(move |_| {
            if !enabled {
                return;
            }
            let action_id = crate::actions::run_ui_token(&ui, &token);
            if let Some(action_id) = action_id {
                if action_id == ActionId::EDIT_PREFERENCES {
                    customize_open.set(true);
                }

                if action_id == "ptnd.action.object.offset_path" {
                    offset_prompt_open.set(true);
                }
                if let Some(tool) = ToolKind::from_action_id(&action_id) {
                    active_tool.set(tool);
                    tool_rail.write().remember_tool(photo, tool);
                }
            }
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
    let target = ui.clone();
    let photo = persona.id == petunia_design_application::surfaces::PERSONA_PHOTO;
    StudioButton::new(
        &ui,
        ui.studio_text(if photo {
            "pixel_studio"
        } else {
            "vector_studio"
        }),
    )
    .text(ui.studio_text(if photo { "pixel" } else { "vector" }))
    .icon(if photo {
        theme::ICON_PHOTO
    } else {
        theme::ICON_NODE
    })
    .selected(active)
    .on_press(move |_| target.set_persona(id.clone()))
}

fn snap_toggle(ui: UiShell, snap_label: String, snapping: bool) -> impl IntoElement {
    let mut shell = ui.shell;
    StudioButton::new(&ui, snap_label)
        .icon(theme::ICON_ATTRIBUTE_PICKER)
        .selected(snapping)
        .on_press(move |_| {
            let _ = run_action_token(&mut shell.write(), "ptnd.action.view.toggle_snapping");
        })
}

fn shell_control(ui: UiShell, control: &ShellControlPresentation) -> impl IntoElement {
    match control.kind {
        ShellControlKind::Divider => rect()
            .width(Size::px(1.))
            .height(Size::px(theme::ICON_INLINE))
            .background(theme::BORDER_SUBTLE)
            .into_element(),
        ShellControlKind::Readout => {
            let zoom = (ui.shell.read().view_camera().zoom * 100.).round() as i32;
            label()
                .text(format!("{zoom}%"))
                .width(Size::px(48.))
                .max_lines(1)
                .font_size(theme::BODY_SIZE)
                .color(theme::TEXT_SECONDARY)
                .into_element()
        }
        ShellControlKind::Icon => {
            let enabled = control.enabled;
            let id = control.id.clone();
            let mut shell = ui.shell;
            StudioButton::new(&ui, control.tooltip.clone())
                .icon(control_icon(&control.id))
                .width(Size::px(28.))
                .enabled(enabled)
                .on_press(move |_| shell_control_action(&mut shell, &id))
                .into_element()
        }
    }
}

pub(crate) fn with_tooltip(
    el: Rect,
    ui: &UiShell,
    id: String,
    title: String,
    summary: String,
    shortcut: String,
) -> Rect {
    let mut enter_hovered = ui.hovered;
    let mut leave_hovered = ui.hovered;
    let leave_id = id.clone();
    el.a11y_alt(title.clone())
        .on_pointer_enter(move |event: Event<PointerEventData>| {
            let point = event.global_location();
            enter_hovered.set(Some(HoverTarget {
                id: id.clone(),
                title: title.clone(),
                summary: summary.clone(),
                shortcut: shortcut.clone(),
                x: point.x,
                y: point.y,
            }));
        })
        .on_pointer_leave(move |_| {
            if leave_hovered
                .read()
                .as_ref()
                .is_some_and(|hovered| hovered.id == leave_id)
            {
                leave_hovered.set(None);
            }
        })
}

#[derive(Clone, PartialEq)]
pub struct TooltipOverlay(pub UiShell);

impl Component for TooltipOverlay {
    fn render(&self) -> impl IntoElement {
        let Some(target) = self.0.hovered.read().clone() else {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        };
        let root_size = *Platform::get().root_size.read();
        let width = 260.;
        let height = if target.summary.is_empty() { 34. } else { 56. };
        let left = (target.x + 12.).min((root_size.width as f64 - width).max(8.));
        let top = (target.y + 12.).min((root_size.height as f64 - height).max(8.));
        let title = if target.shortcut.is_empty() {
            target.title
        } else {
            format!("{} ({})", target.title, target.shortcut)
        };
        let mut lines = vec![label()
            .text(title)
            .color(theme::TEXT_PRIMARY)
            .font_size(theme::BODY_SIZE)
            .into_element()];
        if !target.summary.is_empty() {
            lines.push(
                label()
                    .text(target.summary)
                    .color(theme::TEXT_SECONDARY)
                    .font_size(theme::CAPTION_SIZE)
                    .into_element(),
            );
        }
        rect()
            .position(Position::new_global().left(left as f32).top(top as f32))
            .layer(Layer::Overlay)
            .interactive(Interactive::No)
            .width(Size::px(width as f32))
            .background(theme::SURFACE_PANEL)
            .border(
                Border::new()
                    .fill(theme::BORDER_SUBTLE)
                    .width(1.)
                    .alignment(BorderAlignment::Inner),
            )
            .corner_radius(6.)
            .padding(Gaps::new(6., 8., 6., 8.))
            .child(rect().direction(Direction::Vertical).children(lines))
    }
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

fn toolbar_summary(ui: &UiShell, id: &str) -> String {
    let text_id = if id.contains("delete") {
        "ptnd.text.summary.delete"
    } else if id.contains("export") {
        "ptnd.text.summary.export"
    } else if id.contains("align") || id.contains("boolean") {
        "ptnd.text.summary.object_action"
    } else {
        "ptnd.text.summary.toolbar_action"
    };
    localized_text(ui, text_id)
}

fn action_summary(ui: &UiShell, action_id: &str) -> String {
    let text_id = if action_id.starts_with("ptnd.action.file") {
        "ptnd.text.summary.file_action"
    } else if action_id.starts_with("ptnd.action.view") {
        "ptnd.text.summary.view_action"
    } else if action_id.starts_with("ptnd.action.edit") {
        "ptnd.text.summary.edit_action"
    } else if action_id.starts_with("ptnd.action.object") {
        "ptnd.text.summary.object_action"
    } else if action_id.starts_with("ptnd.tool") {
        "ptnd.text.summary.tool_action"
    } else {
        "ptnd.text.summary.command"
    };
    localized_text(ui, text_id)
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
            let icon = tool_badge_icon(&ui);
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
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
        ToolbarEntryKind::ColorSwatches => {
            let state = ui.shell.read();
            let selected = state.bridge.selection().selected_ids;
            let object = selected
                .first()
                .and_then(|id| state.bridge.session()?.find_object(*id));
            let brush = object.is_some_and(|o| {
                matches!(
                    o.shape,
                    Some(petunia_design_document::ShapeKind::Raster { .. })
                )
            }) || (selected.is_empty()
                && *ui.persona.read() == petunia_design_application::surfaces::PERSONA_PHOTO);
            let brush_color = crate::studio::rgb_token(
                state.tools.photo_brush_tool().brush_settings().color[..3]
                    .try_into()
                    .unwrap_or([0.; 3]),
            );
            let colors = [
                object.and_then(|o| o.fill.clone()),
                object.and_then(|o| o.stroke.clone()),
            ];
            drop(state);
            rect()
                .direction(Direction::Horizontal)
                .content(Content::Flex)
                .spacing(theme::SPACE_1)
                .children(colors.into_iter().enumerate().map(|(index, token)| {
                    let fill = index == 0;
                    let caption = ui.studio_text(if fill { "fill" } else { "stroke" });
                    let token = if brush {
                        Some(brush_color.clone())
                    } else {
                        token
                    };
                    let text = token.as_deref().unwrap_or("—");
                    let title = format!("{caption}: {text}");
                    let color = token
                        .as_deref()
                        .map(petunia_design_document::resolve_color_to_rgb)
                        .map(|rgb| {
                            Color::from_rgb(
                                (rgb[0] * 255.).round() as u8,
                                (rgb[1] * 255.).round() as u8,
                                (rgb[2] * 255.).round() as u8,
                            )
                        })
                        .unwrap_or(Color::TRANSPARENT);
                    let mut target = ui.studio_fill_target;
                    let mut tab = ui.dock_tab;
                    let mut visible = ui.right_studio_open;
                    crate::studio_widgets::StudioButton::new(&ui, title)
                        .swatch(color)
                        .width(Size::px(34.))
                        .on_press(move |_| {
                            target.set(fill);
                            tab.set(2);
                            visible.set(true);
                        })
                }))
        }
        ToolbarEntryKind::Command => {
            let icon = toolbar_command_icon(&entry.id);
            let id = entry.id.clone();
            let mut shell = ui.shell;
            rect().child(
                crate::studio_widgets::StudioButton::new(&ui, tooltip_text.clone())
                    .icon(icon)
                    .enabled(entry.enabled)
                    .width(Size::px(theme::TOOL_BUTTON))
                    .on_press(move |_| activate_toolbar_entry(&mut shell, &id)),
            )
        }
    };
    with_tooltip(
        body,
        &ui,
        hover_id("toolbar", &entry.id),
        tooltip_text,
        toolbar_summary(&ui, &entry.id),
        String::new(),
    )
    .into_element()
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

fn tool_badge_icon(ui: &UiShell) -> theme::AppIcon {
    tool_meta(*ui.active_tool.read()).icon
}

fn toolbar_command_icon(id: &str) -> theme::AppIcon {
    if id.contains("align") || id.contains("distribute") {
        theme::ICON_MOVE
    } else if id.contains("boolean") || id.contains("shape") {
        theme::ICON_SPARKLES
    } else if id.contains("export") {
        theme::ICON_PHOTO
    } else if id.contains("delete") {
        theme::ICON_TRASH
    } else if id.contains("copy") || id.contains("duplicate") {
        theme::ICON_COPY
    } else if id.contains("lock") {
        theme::ICON_LOCK
    } else if id.contains("eye") {
        theme::ICON_EYE
    } else if id.contains("undo") {
        theme::ICON_UNDO
    } else if id.contains("redo") {
        theme::ICON_REDO
    } else {
        theme::ICON_SETTINGS
    }
}

fn customize_button(ui: UiShell) -> impl IntoElement {
    let mut customize_open = ui.customize_open;
    let name = localized_text(&ui, "ptnd.text.shell.customize");
    StudioButton::new(&ui, name)
        .icon(theme::ICON_CUSTOMIZE)
        .width(Size::px(theme::TOOL_BUTTON))
        .on_press(move |_| {
            let open = *customize_open.peek();
            customize_open.set(!open);
        })
}

/// Renders one Tabler glyph recolored to the icon style in use.
///
/// Outline glyphs use `currentColor`, so they are recolored through `color`;
/// filled glyphs are solid and recolored through `fill`.
pub fn app_icon(icon: theme::AppIcon, style: theme::IconStyle, color: Color) -> impl IntoElement {
    app_icon_sized(icon, style, color, theme::ICON_TOOLBAR)
}

pub fn app_icon_sized(
    icon: theme::AppIcon,
    style: theme::IconStyle,
    color: Color,
    size: f32,
) -> impl IntoElement {
    let bytes = icon.bytes(style);
    if style == theme::IconStyle::Filled && icon.filled.is_some() {
        SvgViewer::new(bytes)
            .width(Size::px(size))
            .height(Size::px(size))
            .fill(color)
    } else {
        SvgViewer::new(bytes)
            .width(Size::px(size))
            .height(Size::px(size))
            .color(color)
    }
}
