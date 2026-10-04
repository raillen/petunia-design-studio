//! Shared controls for the desktop Studio (ADR-012).
//! Keyboard activation, focus, labels and selected/disabled states are owned here.
use crate::{
    chrome::{app_icon_sized, with_tooltip},
    theme,
    ui_state::UiShell,
};
use freya::prelude::*;

#[derive(Clone, PartialEq)]
pub struct StudioButton {
    ui: UiShell,
    title: String,
    caption: Option<String>,
    icon: Option<theme::AppIcon>,
    swatch: Option<Color>,
    width: Size,
    height: f32,
    selected: bool,
    enabled: bool,
    tab: bool,
    on_press: Option<EventHandler<Event<PressEventData>>>,
}
impl StudioButton {
    pub fn new(ui: &UiShell, title: impl Into<String>) -> Self {
        Self {
            ui: ui.clone(),
            title: title.into(),
            caption: None,
            icon: None,
            swatch: None,
            width: Size::Inner,
            height: theme::CONTROL_HEIGHT,
            selected: false,
            enabled: true,
            tab: false,
            on_press: None,
        }
    }
    pub fn text(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }
    pub fn icon(mut self, icon: theme::AppIcon) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn swatch(mut self, color: Color) -> Self {
        self.swatch = Some(color);
        self
    }
    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn tab(mut self) -> Self {
        self.tab = true;
        self
    }
    pub fn on_press(mut self, handler: impl Into<EventHandler<Event<PressEventData>>>) -> Self {
        self.on_press = Some(handler.into());
        self
    }
}
impl Component for StudioButton {
    fn render(&self) -> impl IntoElement {
        let focus_id = use_a11y();
        let focus = use_focus(focus_id);
        let mut hovering = use_state(|| false);
        let selected = self.selected;
        let enabled = self.enabled;
        let color = if enabled {
            theme::TEXT_PRIMARY
        } else {
            theme::TEXT_DISABLED
        };
        let keyboard_focus = focus() == Focus::Keyboard;
        let border_color = if keyboard_focus {
            self.ui.accent.read().value
        } else if selected {
            theme::BORDER_ACTIVE
        } else {
            Color::TRANSPARENT
        };
        let handler = self.on_press.clone();
        let body = rect()
            .width(self.width.clone()).height(Size::px(self.height))
            .direction(Direction::Horizontal).content(Content::Flex).cross_align(Alignment::Center).main_align(Alignment::Center)
            .spacing(theme::SPACE_1).padding(Gaps::new_symmetric(0., theme::SPACE_2))
            .corner_radius(theme::CONTROL_RADIUS)
            .background(if selected { theme::SURFACE_SELECTED }
                else if enabled && hovering() { theme::SURFACE_HOVER } else { Color::TRANSPARENT })
            .border(Border::new().fill(border_color).width(if keyboard_focus {2.} else {1.})
                .alignment(BorderAlignment::Inner))
            .a11y_id(focus_id).a11y_focusable(enabled)
            .a11y_role(if self.tab { AccessibilityRole::Tab } else { AccessibilityRole::Button })
            .a11y_alt(self.title.clone())
            .a11y_builder(|node| {
                if self.tab { node.set_selected(selected); }
                if !enabled { node.set_disabled(); }
            })
            .cursor(if enabled { CursorIcon::Pointer } else { CursorIcon::NotAllowed })
            .on_pointer_enter(move |_| hovering.set(true))
            .on_pointer_leave(move |_| hovering.set(false))
            .on_all_press(move |event: Event<PressEventData>| {
                event.stop_propagation();
                let primary = !matches!(event.data(), PressEventData::Mouse(m) if m.button != Some(MouseButton::Left));
                if enabled && primary {
                    if matches!(event.data(),PressEventData::Keyboard(_)) {focus_id.request_focus();}
                    if let Some(handler) = &handler {handler.call(event);}
                }
            })
            .maybe_child(self.icon.map(|icon| app_icon_sized(icon, *self.ui.icon_style.read(), color, theme::ICON_INLINE)))
            .maybe_child(self.swatch.map(|color| rect().width(Size::px(18.)).height(Size::px(18.))
                .background(color).border(Border::new().fill(theme::TEXT_SECONDARY).width(1.).alignment(BorderAlignment::Inner))))
            .maybe_child(self.caption.as_ref().map(|caption| label().text(caption.clone()).font_size(theme::BODY_SIZE)
                .color(color).max_lines(1).text_overflow(TextOverflow::Ellipsis)));
        with_tooltip(
            body,
            &self.ui,
            format!("studio:{}", self.title),
            self.title.clone(),
            String::new(),
            String::new(),
        )
    }
}

/// Bounded responsive chrome; requests are retained so resizing restores the user's widths.
pub fn studio_width(requested: f32, window_width: f32, reserved: f32) -> f32 {
    let maximum = (window_width - reserved - 240.).clamp(240., 480.);
    requested.clamp(240., maximum)
}

pub fn left_studio_width(requested: f32, window_width: f32, tools: f32, right_open: bool) -> f32 {
    let maximum =
        (window_width - tools - 240. - if right_open { 248. } else { 4. }).clamp(160., 480.);
    requested.clamp(160., maximum)
}

pub fn bottom_studio_height(requested: f32, window_height: f32) -> f32 {
    let maximum = (window_height - 160. - 240. - 4.).clamp(80., 420.);
    requested.clamp(80., maximum)
}

pub fn panel_header(title: impl Into<String>) -> Rect {
    rect()
        .height(Size::px(theme::PANEL_HEADER_HEIGHT))
        .width(Size::fill())
        .background(theme::SURFACE_CHROME)
        .padding(Gaps::new_symmetric(0., theme::SPACE_2))
        .direction(Direction::Horizontal)
        .content(Content::Flex)
        .cross_align(Alignment::Center)
        .child(
            label()
                .text(title.into())
                .font_size(theme::BODY_SIZE)
                .color(theme::TEXT_PRIMARY)
                .max_lines(1)
                .text_overflow(TextOverflow::Ellipsis),
        )
}

/// Freya treats render callbacks as equal. A view-local epoch republishes
/// captured camera/overlay inputs on each render without changing document IDs.
pub fn canvas_render_epoch() -> u64 {
    let epoch = use_hook(|| std::rc::Rc::new(std::cell::Cell::new(0u64)));
    let next = epoch.get().wrapping_add(1);
    epoch.set(next);
    next
}

#[derive(Clone, Copy, PartialEq)]
pub enum DockEdge {
    Left,
    Right,
    Bottom,
}

#[derive(Clone, PartialEq)]
pub struct StudioSplitter {
    ui: UiShell,
    edge: DockEdge,
}
impl StudioSplitter {
    pub fn new(ui: &UiShell, edge: DockEdge) -> Self {
        Self {
            ui: ui.clone(),
            edge,
        }
    }
}
impl Component for StudioSplitter {
    fn render(&self) -> impl IntoElement {
        let mut dragging = use_state(|| false);
        let mut start = use_state(|| 0f64);
        let mut original = use_state(|| 0f32);
        let focus_id = use_a11y();
        let focus = use_focus(focus_id);
        let ui = &self.ui;
        let edge = self.edge;
        let mut requested = match edge {
            DockEdge::Left => ui.left_dock_width,
            DockEdge::Right => ui.dock_width,
            DockEdge::Bottom => ui.bottom_dock_height,
        };
        let open = match edge {
            DockEdge::Left => *ui.left_dock_open.read(),
            DockEdge::Right => *ui.right_studio_open.read(),
            DockEdge::Bottom => *ui.bottom_dock_open.read(),
        };
        let active = *dragging.read();
        if !open {
            return rect().width(Size::px(0.)).height(Size::px(0.));
        }
        let horizontal = edge == DockEdge::Bottom;
        let clamp = move |value: f32| {
            if horizontal {
                value.clamp(80., 420.)
            } else if edge == DockEdge::Left {
                value.clamp(160., 480.)
            } else {
                value.clamp(240., 480.)
            }
        };
        rect()
            .width(if horizontal {
                Size::fill()
            } else {
                Size::px(4.)
            })
            .height(if horizontal {
                Size::px(4.)
            } else {
                Size::fill()
            })
            .background(if active || focus() == Focus::Keyboard {
                ui.accent.read().value
            } else {
                theme::BORDER_SUBTLE
            })
            .cursor(if horizontal {
                CursorIcon::NsResize
            } else {
                CursorIcon::EwResize
            })
            .a11y_id(focus_id)
            .a11y_focusable(true)
            .a11y_role(AccessibilityRole::Splitter)
            .a11y_alt(ui.studio_text(match edge {
                DockEdge::Left => "resize_assets",
                DockEdge::Right => "resize_studio",
                DockEdge::Bottom => "resize_bottom",
            }))
            .on_pointer_down(move |event: Event<PointerEventData>| {
                if !event.is_primary() {
                    return;
                }
                event.stop_propagation();
                event.prevent_default();
                focus_id.request_focus();
                start.set(if horizontal {
                    event.global_location().y
                } else {
                    event.global_location().x
                });
                original.set(*requested.peek());
                dragging.set(true);
            })
            .on_capture_global_pointer_move(move |event: Event<PointerEventData>| {
                if !*dragging.peek() {
                    return;
                }
                event.stop_propagation();
                event.prevent_default();
                let position = if horizontal {
                    event.global_location().y
                } else {
                    event.global_location().x
                };
                let delta = (position - *start.peek()) as f32
                    * if edge == DockEdge::Left { 1. } else { -1. };
                requested.set(clamp(*original.peek() + delta));
            })
            .on_capture_global_pointer_press(move |event: Event<PointerEventData>| {
                if *dragging.peek() {
                    event.stop_propagation();
                    event.prevent_default();
                    dragging.set(false);
                }
            })
            .on_key_down(move |event: Event<KeyboardEventData>| {
                let step = if event.modifiers.contains(Modifiers::SHIFT) {
                    32.
                } else {
                    8.
                };
                let delta = match event.key {
                    Key::Named(NamedKey::ArrowLeft) if !horizontal => {
                        if edge == DockEdge::Right {
                            step
                        } else {
                            -step
                        }
                    }
                    Key::Named(NamedKey::ArrowRight) if !horizontal => {
                        if edge == DockEdge::Right {
                            -step
                        } else {
                            step
                        }
                    }
                    Key::Named(NamedKey::ArrowUp) if horizontal => step,
                    Key::Named(NamedKey::ArrowDown) if horizontal => -step,
                    Key::Named(NamedKey::Escape) if *dragging.peek() => {
                        event.stop_propagation();
                        event.prevent_default();
                        requested.set(*original.peek());
                        dragging.set(false);
                        return;
                    }
                    _ => return,
                };
                event.stop_propagation();
                event.prevent_default();
                let value = *requested.peek() + delta;
                requested.set(clamp(value));
            })
    }
}
