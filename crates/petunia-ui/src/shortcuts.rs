//! Shortcut table, rebinding and conflict detection.
//!
//! Implements ADR-0012 D4: a primary set is defined as *data*, every
//! action is rebindable, conflicts are detected before they are
//! applied, and no essential action depends on a single modifier
//! (Alt-only is forbidden by the accessibility gate).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One key combination. `key` is a normalized identifier such as
/// `"z"`, `"f1"`, `"["` or `"space"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyCombo {
    pub key: String,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl KeyCombo {
    /// A bare key without modifiers.
    #[must_use]
    pub fn key(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            ctrl: false,
            shift: false,
            alt: false,
        }
    }

    /// Add Ctrl/Cmd.
    #[must_use]
    pub fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    /// Add Shift.
    #[must_use]
    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    /// Stable textual form for labels and conflict messages.
    #[must_use]
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.shift {
            parts.push("Shift".to_string());
        }
        if self.alt {
            parts.push("Alt".to_string());
        }
        parts.push(self.key.clone());
        parts.join("+")
    }

    /// True when the combo is only a bare modifier key, which the
    /// accessibility gate forbids as an essential action.
    #[must_use]
    pub fn is_modifier_only(&self) -> bool {
        self.alt && !self.ctrl && !self.shift && is_modifier_key(&self.key)
    }
}

fn is_modifier_key(key: &str) -> bool {
    matches!(key, "alt" | "altgr" | "ctrl")
}

/// Actions reachable by shortcut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionId {
    SelectTool,
    NodeTool,
    PenTool,
    BrushTool,
    EraserTool,
    Undo,
    Redo,
    SelectAll,
    Group,
    Ungroup,
    BringForward,
    SendBackward,
    Confirm,
    Cancel,
    PanTool,
}

impl ActionId {
    /// Stable identifier for storage and logs.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SelectTool => "tool.select",
            Self::NodeTool => "tool.node",
            Self::PenTool => "tool.pen",
            Self::BrushTool => "tool.brush",
            Self::EraserTool => "tool.eraser",
            Self::Undo => "edit.undo",
            Self::Redo => "edit.redo",
            Self::SelectAll => "edit.select_all",
            Self::Group => "edit.group",
            Self::Ungroup => "edit.ungroup",
            Self::BringForward => "edit.bring_forward",
            Self::SendBackward => "edit.send_backward",
            Self::Confirm => "edit.confirm",
            Self::Cancel => "edit.cancel",
            Self::PanTool => "view.pan",
        }
    }
}

/// The primary set from ADR-0012 D4.
#[must_use]
pub fn default_bindings() -> Vec<(ActionId, KeyCombo)> {
    use ActionId::*;
    vec![
        (SelectTool, KeyCombo::key("v")),
        (NodeTool, KeyCombo::key("n")),
        (PenTool, KeyCombo::key("p")),
        (BrushTool, KeyCombo::key("b")),
        (EraserTool, KeyCombo::key("e")),
        (Undo, KeyCombo::key("z").with_ctrl()),
        (Redo, KeyCombo::key("z").with_ctrl().with_shift()),
        (SelectAll, KeyCombo::key("a").with_ctrl()),
        (Group, KeyCombo::key("g").with_ctrl()),
        (Ungroup, KeyCombo::key("g").with_ctrl().with_shift()),
        (BringForward, KeyCombo::key("]")),
        (SendBackward, KeyCombo::key("[")),
        (Confirm, KeyCombo::key("enter")),
        (Cancel, KeyCombo::key("escape")),
        (PanTool, KeyCombo::key("space")),
    ]
}

/// Remap failure. The table is unchanged when any error is returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindError {
    /// Combo already assigned to another action.
    Conflict { combo: String, action: String },
    /// Modifier-only combos are refused.
    ModifierOnly { combo: String },
}

impl BindError {
    /// Human-readable reason.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::Conflict { combo, action } => {
                format!("{combo} já está atribuído a {action}")
            }
            Self::ModifierOnly { combo } => {
                format!("{combo} não pode ser atalho único de uma ação essencial")
            }
        }
    }
}

/// Shortcut table: action → combo map with lookup in both directions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutTable {
    bindings: HashMap<ActionId, KeyCombo>,
}

impl ShortcutTable {
    /// The primary set from ADR-0012 D4.
    #[must_use]
    pub fn defaults() -> Self {
        Self {
            bindings: default_bindings().into_iter().collect(),
        }
    }

    /// Rebind an action, refusing conflicts and modifier-only combos.
    pub fn rebind(
        &mut self,
        action: ActionId,
        combo: KeyCombo,
    ) -> std::result::Result<(), BindError> {
        if combo.is_modifier_only() {
            return Err(BindError::ModifierOnly {
                combo: combo.display(),
            });
        }
        if let Some(existing) = self.action_for(&combo) {
            if existing != action {
                return Err(BindError::Conflict {
                    combo: combo.display(),
                    action: existing.as_str().to_string(),
                });
            }
        }
        self.bindings.insert(action, combo);
        Ok(())
    }

    /// Action bound to a combo, if any.
    #[must_use]
    pub fn action_for(&self, combo: &KeyCombo) -> Option<ActionId> {
        self.bindings
            .iter()
            .find(|(_, bound)| *bound == combo)
            .map(|(action, _)| *action)
    }

    /// Combo bound to an action, if any.
    #[must_use]
    pub fn combo_for(&self, action: ActionId) -> Option<&KeyCombo> {
        self.bindings.get(&action)
    }

    /// Restore one action to its primary binding.
    pub fn reset_action(&mut self, action: ActionId) -> bool {
        if let Some((_, combo)) = default_bindings()
            .into_iter()
            .find(|(candidate, _)| *candidate == action)
        {
            self.bindings.insert(action, combo);
            true
        } else {
            false
        }
    }

    /// Restore every action to the primary set.
    pub fn reset_all(&mut self) {
        self.bindings = default_bindings().into_iter().collect();
    }

    /// Number of bound actions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Whether nothing is bound.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// All actions in declaration order, for the editor UI.
    #[must_use]
    pub fn entries(&self) -> Vec<(ActionId, KeyCombo)> {
        default_bindings()
            .into_iter()
            .filter_map(|(action, default_combo)| {
                self.bindings
                    .get(&action)
                    .map(|combo| (action, combo.clone()))
                    .or(Some((action, default_combo)))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_cover_the_primary_set() {
        let table = ShortcutTable::defaults();
        assert_eq!(table.len(), 15);
        assert_eq!(
            table
                .combo_for(ActionId::Undo)
                .map(KeyCombo::display)
                .as_deref(),
            Some("Ctrl+z")
        );
        assert_eq!(
            table.action_for(&KeyCombo::key("v")),
            Some(ActionId::SelectTool)
        );
    }

    #[test]
    fn rebind_detects_conflicts_and_keeps_the_table() {
        let mut table = ShortcutTable::defaults();
        let error = table.rebind(ActionId::Undo, KeyCombo::key("z").with_ctrl());
        assert!(
            error.is_ok(),
            "binding a combo to the same action is a no-op"
        );

        let taken = table.rebind(ActionId::SelectAll, KeyCombo::key("v"));
        assert!(
            matches!(taken, Err(BindError::Conflict { .. })),
            "{taken:?}"
        );
        // The table is untouched after a rejected rebind.
        assert_eq!(
            table.action_for(&KeyCombo::key("v")),
            Some(ActionId::SelectTool)
        );
    }

    #[test]
    fn modifier_only_combos_are_refused() {
        let mut table = ShortcutTable::defaults();
        let error = table.rebind(
            ActionId::BringForward,
            KeyCombo {
                key: "alt".into(),
                ctrl: false,
                shift: false,
                alt: true,
            },
        );
        assert!(matches!(error, Err(BindError::ModifierOnly { .. })));
        assert!(error.unwrap_err().message().contains("não pode"));
    }

    #[test]
    fn reset_restores_defaults() {
        let mut table = ShortcutTable::defaults();
        assert!(table
            .rebind(ActionId::Undo, KeyCombo::key("y").with_ctrl())
            .is_ok());
        assert_eq!(
            table.action_for(&KeyCombo::key("y").with_ctrl()),
            Some(ActionId::Undo)
        );
        assert!(table.reset_action(ActionId::Undo));
        assert_eq!(table.action_for(&KeyCombo::key("y").with_ctrl()), None);
        assert_eq!(
            table.action_for(&KeyCombo::key("z").with_ctrl()),
            Some(ActionId::Undo)
        );
    }

    #[test]
    fn display_matches_platform_independent_order() {
        let combo = KeyCombo::key("z").with_ctrl().with_shift();
        assert_eq!(combo.display(), "Ctrl+Shift+z");
    }
}
