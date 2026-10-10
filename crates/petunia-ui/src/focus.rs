//! Keyboard focus model: predictable order, visible focus and traps.
//!
//! Implements the accessibility half of ADR-0012 D9 and the focus
//! rules of D8: every region is reachable by keyboard alone, the order
//! is the visual order (left to right, top to bottom), Escape inside
//! a bar returns focus to the canvas, and a dialog keeps focus inside
//! itself until it closes.

use serde::{Deserialize, Serialize};

/// Which docked region a focus entry lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusZone {
    /// Tool strip on the left.
    Toolbar,
    /// Context bar under the toolbar.
    ContextBar,
    /// Layer and page tree, right side by default.
    Tree,
    /// The canvas itself.
    Canvas,
    /// Docked panels, including their tabs.
    Panel,
}

/// Why a focus entry cannot take focus right now. The rule of ADR-
/// 0012 D2 is that a disabled control shows the reason, not just a
/// grey state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisabledReason(pub String);

impl DisabledReason {
    /// Build a reason from any displayable text.
    #[must_use]
    pub fn new(reason: impl Into<String>) -> Self {
        Self(reason.into())
    }

    /// The text the UI shows next to the disabled control.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.0
    }
}

/// One focusable control in the tab order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusEntry {
    /// Stable identifier the UI maps back to its widget.
    pub id: String,
    /// Region the entry belongs to.
    pub zone: FocusZone,
    /// Visual position; the order sorts by `y`, then by `x`.
    pub x: i32,
    /// Visual position in the same units the layout uses.
    pub y: i32,
    /// Set when the control exists but cannot take focus.
    pub disabled: Option<DisabledReason>,
}

impl FocusEntry {
    /// Build an enabled entry at a layout position.
    #[must_use]
    pub fn new(id: impl Into<String>, zone: FocusZone, x: i32, y: i32) -> Self {
        Self {
            id: id.into(),
            zone,
            x,
            y,
            disabled: None,
        }
    }

    /// Mark the entry disabled with a visible reason.
    #[must_use]
    pub fn disabled(mut self, reason: impl Into<String>) -> Self {
        self.disabled = Some(DisabledReason::new(reason));
        self
    }

    /// Whether the entry can take focus.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.disabled.is_none()
    }
}

/// What an Escape press produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusOutcome {
    /// A dialog closed and focus returned to its opener.
    ClosedDialog { restored: String },
    /// Focus came back to the canvas from another region.
    ReturnedToCanvas { entry: String },
    /// Nothing to unwind; the press is free for other handlers.
    Unhandled,
}

/// An open modal dialog: its own focusable controls, in tab order.
#[derive(Debug, Clone)]
struct DialogTrap {
    entries: Vec<FocusEntry>,
    index: usize,
}

/// Keyboard focus manager for the whole window.
///
/// The order is computed from the entries' visual position, so the
/// keyboard order never disagrees with what the user sees. A dialog
/// traps the order while it is open; closing it restores focus to the
/// entry that opened it.
#[derive(Debug, Clone, Default)]
pub struct FocusManager {
    entries: Vec<FocusEntry>,
    current: Option<usize>,
    /// Open dialog, if any: its controls shadow the window order.
    trap: Option<DialogTrap>,
    /// Window entry that opened the active dialog, for focus restore.
    opener: Option<String>,
}

impl FocusManager {
    /// A manager with no entries.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the focusable set. Entries are sorted by visual
    /// position: top to bottom, then left to right.
    pub fn set_entries(&mut self, entries: Vec<FocusEntry>) {
        self.entries = entries;
        self.entries
            .sort_by(|left, right| left.y.cmp(&right.y).then(left.x.cmp(&right.x)));
        // A vanished entry cannot keep focus.
        if let Some(current) = self.current {
            if !self
                .entries
                .get(current)
                .is_some_and(|entry| entry.is_enabled())
            {
                self.current = None;
            }
        }
    }

    /// All window entries, in tab order.
    #[must_use]
    pub fn entries(&self) -> &[FocusEntry] {
        &self.entries
    }

    /// Entries of the open dialog, in their own tab order.
    #[must_use]
    pub fn dialog_entries(&self) -> &[FocusEntry] {
        self.trap
            .as_ref()
            .map_or(&[], |trap| trap.entries.as_slice())
    }

    /// Entry that currently holds focus. A disabled entry never holds
    /// focus, so `None` means nothing on screen is focused.
    #[must_use]
    pub fn current(&self) -> Option<&FocusEntry> {
        let entry = match &self.trap {
            Some(trap) => trap.entries.get(trap.index),
            None => self.current.and_then(|index| self.entries.get(index)),
        };
        entry.filter(|entry| entry.is_enabled())
    }

    /// Identifier of the focused entry, for the UI to draw the ring.
    #[must_use]
    pub fn current_id(&self) -> Option<&str> {
        self.current().map(|entry| entry.id.as_str())
    }

    /// Whether some entry holds focus.
    #[must_use]
    pub fn has_focus(&self) -> bool {
        self.current().is_some()
    }

    /// Whether a dialog currently traps the order.
    #[must_use]
    pub fn is_trapped(&self) -> bool {
        self.trap.is_some()
    }

    /// Move focus to an entry by identifier; disabled entries refuse,
    /// and while a dialog is open only its own entries are reachable.
    pub fn focus(&mut self, id: &str) -> bool {
        if let Some(trap) = &mut self.trap {
            let Some(index) = trap.entries.iter().position(|entry| entry.id == id) else {
                return false;
            };
            if !trap.entries[index].is_enabled() {
                return false;
            }
            trap.index = index;
            return true;
        }
        let Some(index) = self.entries.iter().position(|entry| entry.id == id) else {
            return false;
        };
        if !self.entries[index].is_enabled() {
            return false;
        }
        self.current = Some(index);
        true
    }

    /// Tab: next enabled entry, wrapping at the ends.
    pub fn focus_next(&mut self) -> Option<&FocusEntry> {
        self.step(1)
    }

    /// Shift+Tab: previous enabled entry, wrapping at the ends.
    pub fn focus_previous(&mut self) -> Option<&FocusEntry> {
        self.step(-1)
    }

    /// Cycle to the next region (F6), keeping a predictable order.
    ///
    /// Ignored while a dialog is open: the trap owns the order.
    pub fn focus_next_zone(&mut self) -> Option<&FocusEntry> {
        if self.trap.is_some() {
            return self.current();
        }
        let Some(current) = self.current() else {
            return self.first_enabled();
        };
        let order = zone_order(current.zone);
        for candidate in order.iter().skip(1) {
            if let Some(entry) = self
                .entries
                .iter()
                .find(|entry| entry.zone == *candidate && entry.is_enabled())
            {
                let id = entry.id.clone();
                self.focus(&id);
                return self.current();
            }
        }
        self.current()
    }

    /// Escape. A dialog closes and restores focus to its opener; any
    /// other region returns focus to the canvas.
    pub fn on_escape(&mut self) -> FocusOutcome {
        if let Some(trap) = self.trap.take() {
            let restored = self
                .opener
                .take()
                .filter(|id| self.focus(id))
                .or_else(|| {
                    trap.entries
                        .iter()
                        .find(|entry| entry.is_enabled())
                        .map(|entry| entry.id.clone())
                })
                .filter(|id| self.focus(id));
            return match restored {
                Some(id) => FocusOutcome::ClosedDialog { restored: id },
                None => {
                    self.current = None;
                    FocusOutcome::Unhandled
                }
            };
        }
        // Already on the canvas: the press belongs to the context
        // chain, not to the focus layer (D8).
        let already_on_canvas = self
            .current()
            .is_some_and(|entry| entry.zone == FocusZone::Canvas);
        if already_on_canvas {
            return FocusOutcome::Unhandled;
        }
        let Some(canvas) = self
            .entries
            .iter()
            .find(|entry| entry.zone == FocusZone::Canvas && entry.is_enabled())
        else {
            return FocusOutcome::Unhandled;
        };
        let entry = canvas.id.clone();
        self.focus(&entry);
        FocusOutcome::ReturnedToCanvas { entry }
    }

    /// Open a modal dialog: focus moves inside it and stays there.
    ///
    /// `entries` are the dialog's own focusable controls, in the order
    /// they appear. The window entry holding focus now is remembered
    /// and gets it back when the dialog closes.
    pub fn open_dialog(&mut self, entries: Vec<FocusEntry>) -> bool {
        if entries.is_empty() {
            return false;
        }
        self.opener = self.current_id().map(str::to_string);
        let mut trap = DialogTrap { entries, index: 0 };
        // A dialog with nothing focusable still traps: the user must
        // not wander into the window behind it. Focus simply shows
        // nothing until the dialog closes.
        trap.index = trap
            .entries
            .iter()
            .position(|entry| entry.is_enabled())
            .unwrap_or(0);
        self.trap = Some(trap);
        true
    }

    /// Whether an identifier belongs to the open dialog.
    #[must_use]
    pub fn is_in_dialog(&self, id: &str) -> bool {
        self.trap
            .as_ref()
            .is_some_and(|trap| trap.entries.iter().any(|entry| entry.id == id))
    }

    fn step(&mut self, direction: i32) -> Option<&FocusEntry> {
        // Walk starts at the neighbour of the current entry, so one
        // press always moves focus somewhere else.
        let (count, start) = {
            let entries: &[FocusEntry] = match &self.trap {
                Some(trap) => trap.entries.as_slice(),
                None => self.entries.as_slice(),
            };
            let count = entries.len();
            if count == 0 {
                return None;
            }
            let start = match self
                .current_id()
                .and_then(|id| entries.iter().position(|entry| entry.id == id))
            {
                Some(index) => (index as i32 + direction).rem_euclid(count as i32),
                None if direction >= 0 => 0,
                None => count as i32 - 1,
            };
            (count, start)
        };
        for step in 0..count {
            let index = (start + direction * step as i32).rem_euclid(count as i32) as usize;
            let enabled = match &self.trap {
                Some(trap) => trap
                    .entries
                    .get(index)
                    .is_some_and(|entry| entry.is_enabled()),
                None => self
                    .entries
                    .get(index)
                    .is_some_and(|entry| entry.is_enabled()),
            };
            if !enabled {
                continue;
            }
            match &mut self.trap {
                Some(trap) => trap.index = index,
                None => self.current = Some(index),
            }
            return self.current();
        }
        None
    }

    fn first_enabled(&self) -> Option<&FocusEntry> {
        self.entries.iter().find(|entry| entry.is_enabled())
    }
}

/// Regions in F6 order: the docked regions first, canvas last, so a
/// single press after finishing a panel lands back on the canvas.
fn zone_order(from: FocusZone) -> [FocusZone; 5] {
    use FocusZone::{Canvas, ContextBar, Panel, Toolbar, Tree};
    match from {
        Toolbar => [Toolbar, ContextBar, Tree, Panel, Canvas],
        ContextBar => [ContextBar, Tree, Panel, Canvas, Toolbar],
        Tree => [Tree, Panel, Canvas, Toolbar, ContextBar],
        Panel => [Panel, Canvas, Toolbar, ContextBar, Tree],
        Canvas => [Canvas, Toolbar, ContextBar, Tree, Panel],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window() -> Vec<FocusEntry> {
        vec![
            FocusEntry::new("tool.select", FocusZone::Toolbar, 0, 40),
            FocusEntry::new("tool.node", FocusZone::Toolbar, 0, 80),
            FocusEntry::new("bar.op", FocusZone::ContextBar, 100, 0),
            FocusEntry::new("tree.root", FocusZone::Tree, 900, 40),
            FocusEntry::new("panel.props", FocusZone::Panel, 900, 300),
            FocusEntry::new("canvas", FocusZone::Canvas, 400, 200),
        ]
    }

    #[test]
    fn tab_order_is_the_visual_order() {
        let mut focus = FocusManager::new();
        focus.set_entries(window());
        assert_eq!(
            focus
                .entries()
                .iter()
                .map(|entry| entry.id.as_str())
                .collect::<Vec<_>>(),
            // y=0 barra, y=40 toolbar e arvore (x decide), y=80 node,
            // y=200 canvas, y=300 painel.
            vec![
                "bar.op",
                "tool.select",
                "tree.root",
                "tool.node",
                "canvas",
                "panel.props"
            ]
        );
    }

    #[test]
    fn tab_wraps_at_both_ends() {
        let mut focus = FocusManager::new();
        focus.set_entries(window());
        assert_eq!(
            focus.focus_next().map(|entry| entry.id.as_str()),
            Some("bar.op")
        );
        assert_eq!(
            focus.focus_next().map(|entry| entry.id.as_str()),
            Some("tool.select")
        );
        assert_eq!(
            focus.focus_previous().map(|entry| entry.id.as_str()),
            Some("bar.op")
        );
        assert_eq!(
            focus.focus_next().map(|entry| entry.id.as_str()),
            Some("tool.select")
        );
        let last = focus
            .entries()
            .last()
            .map(|entry| entry.id.clone())
            .expect("entries");
        focus.focus(&last);
        assert_eq!(
            focus.focus_next().map(|entry| entry.id.as_str()),
            Some("bar.op")
        );
    }

    #[test]
    fn disabled_entries_are_skipped_and_explain_themselves() {
        let mut focus = FocusManager::new();
        let mut entries = window();
        for entry in &mut entries {
            if entry.id == "bar.op" {
                entry.disabled = Some(DisabledReason::new("nenhuma seleção"));
            }
        }
        focus.set_entries(entries);
        focus.focus("canvas");
        assert_eq!(
            focus.focus_next().map(|entry| entry.id.as_str()),
            Some("panel.props")
        );
        focus.focus_previous();
        focus.focus_previous();
        // A disabled entry is skipped in both directions, so the walk
        // never lands on it.
        assert!(focus.current().is_some_and(|entry| entry.is_enabled()));
        assert!(!focus.focus("bar.op"), "a disabled entry refuses focus");
        // The reason is carried, not just the grey state (D2).
        assert_eq!(
            focus
                .entries()
                .iter()
                .find(|entry| entry.id == "bar.op")
                .and_then(|entry| entry.disabled.clone())
                .map(|reason| reason.message().to_string()),
            Some("nenhuma seleção".to_string())
        );
    }

    #[test]
    fn zone_cycle_reaches_the_canvas() {
        let mut focus = FocusManager::new();
        focus.set_entries(window());
        focus.focus("tool.select");
        assert_eq!(
            focus.focus_next_zone().map(|entry| entry.zone),
            Some(FocusZone::ContextBar)
        );
        assert_eq!(
            focus.focus_next_zone().map(|entry| entry.zone),
            Some(FocusZone::Tree)
        );
        assert_eq!(
            focus.focus_next_zone().map(|entry| entry.zone),
            Some(FocusZone::Panel)
        );
        assert_eq!(
            focus.focus_next_zone().map(|entry| entry.zone),
            Some(FocusZone::Canvas)
        );
    }

    #[test]
    fn escape_on_the_canvas_is_left_for_the_context_chain() {
        let mut focus = FocusManager::new();
        focus.set_entries(window());
        focus.focus("canvas");
        assert_eq!(focus.on_escape(), FocusOutcome::Unhandled);
    }

    #[test]
    fn escape_from_a_bar_returns_to_the_canvas() {
        let mut focus = FocusManager::new();
        focus.set_entries(window());
        focus.focus("bar.op");
        assert_eq!(
            focus.on_escape(),
            FocusOutcome::ReturnedToCanvas {
                entry: "canvas".to_string()
            }
        );
        assert_eq!(focus.current_id(), Some("canvas"));
    }

    #[test]
    fn dialog_traps_the_order_and_restores_the_opener() {
        let mut focus = FocusManager::new();
        focus.set_entries(window());
        focus.focus("panel.props");
        let dialog = vec![
            FocusEntry::new("dlg.field", FocusZone::Panel, 10, 10),
            FocusEntry::new("dlg.cancel", FocusZone::Panel, 10, 40),
        ];
        assert!(focus.open_dialog(dialog));
        assert!(focus.is_trapped());
        assert_eq!(focus.current_id(), Some("dlg.field"));
        assert_eq!(
            focus.focus_next().map(|entry| entry.id.as_str()),
            Some("dlg.cancel")
        );
        assert_eq!(
            focus.focus_next().map(|entry| entry.id.as_str()),
            Some("dlg.field")
        );
        // Nothing outside the dialog is reachable while trapped.
        assert!(!focus.focus("canvas"));
        assert_eq!(
            focus.on_escape(),
            FocusOutcome::ClosedDialog {
                restored: "panel.props".to_string()
            }
        );
        assert!(!focus.is_trapped());
        assert_eq!(focus.current_id(), Some("panel.props"));
    }

    #[test]
    fn dialog_without_focusable_entries_still_traps() {
        let mut focus = FocusManager::new();
        focus.set_entries(window());
        focus.focus("canvas");
        let dialog =
            vec![FocusEntry::new("dlg.text", FocusZone::Panel, 0, 0).disabled("somente leitura")];
        assert!(focus.open_dialog(dialog));
        assert!(
            focus.is_trapped(),
            "the user must not wander behind a dialog"
        );
        assert!(!focus.focus("canvas"));
        assert!(focus.current().is_none());
        // Escape still closes it and returns focus where it was.
        assert_eq!(
            focus.on_escape(),
            FocusOutcome::ClosedDialog {
                restored: "canvas".to_string()
            }
        );
    }

    #[test]
    fn relayout_keeps_a_valid_selection() {
        let mut focus = FocusManager::new();
        focus.set_entries(window());
        focus.focus("canvas");
        focus.set_entries(vec![FocusEntry::new("only", FocusZone::Canvas, 0, 0)]);
        assert!(
            focus.current().is_none(),
            "a vanished entry cannot keep focus"
        );
        assert_eq!(
            focus.focus_next().map(|entry| entry.id.as_str()),
            Some("only")
        );
    }

    #[test]
    fn empty_manager_stays_quiet() {
        let mut focus = FocusManager::new();
        assert!(focus.focus_next().is_none());
        assert!(focus.current().is_none());
        assert_eq!(focus.on_escape(), FocusOutcome::Unhandled);
        assert!(!focus.open_dialog(Vec::new()));
    }
}
