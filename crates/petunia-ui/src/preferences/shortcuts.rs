use super::{PreferenceError, PreferenceResult, UiPreferences};
use crate::shortcuts::{default_bindings, ActionId, KeyCombo, ShortcutTable};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShortcutBinding {
    pub action: ActionId,
    pub combo: KeyCombo,
}

impl UiPreferences {
    /// Update one override only after conflict detection against the complete
    /// effective set. A rejected edit leaves all preferences unchanged.
    pub fn set_shortcut_binding(
        &mut self,
        action: ActionId,
        combo: KeyCombo,
    ) -> PreferenceResult<()> {
        self.validate()?;
        let mut staged = self.clone();
        staged
            .shortcut_bindings
            .retain(|binding| binding.action != action);
        staged
            .shortcut_bindings
            .push(ShortcutBinding { action, combo });
        staged.validate()?;
        *self = staged;
        Ok(())
    }

    pub fn reset_shortcut_binding(&mut self, action: ActionId) -> PreferenceResult<()> {
        self.validate()?;
        let mut staged = self.clone();
        staged
            .shortcut_bindings
            .retain(|binding| binding.action != action);
        staged.validate()?;
        *self = staged;
        Ok(())
    }

    pub fn reset_shortcut_bindings(&mut self) -> PreferenceResult<()> {
        self.validate()?;
        self.shortcut_bindings.clear();
        Ok(())
    }

    /// Build the runtime projection. Clear overridden defaults first so valid
    /// exchanges of two shortcuts do not create an intermediate conflict.
    pub fn shortcut_table(&self) -> PreferenceResult<ShortcutTable> {
        self.validate_shortcut_bindings()?;
        let mut table = ShortcutTable::defaults();
        for binding in &self.shortcut_bindings {
            table.clear_binding(binding.action);
        }
        for binding in &self.shortcut_bindings {
            table
                .rebind(binding.action, binding.combo.clone())
                .map_err(|error| PreferenceError::Invalid(error.message()))?;
        }
        Ok(table)
    }

    pub(super) fn validate_shortcut_bindings(&self) -> PreferenceResult<()> {
        let mut effective: HashMap<ActionId, KeyCombo> = default_bindings().into_iter().collect();
        if self.shortcut_bindings.len() > effective.len() {
            return Err(PreferenceError::Invalid(
                "Too many shortcut overrides".into(),
            ));
        }
        let mut actions = HashSet::new();
        for binding in &self.shortcut_bindings {
            if !actions.insert(binding.action) || !effective.contains_key(&binding.action) {
                return Err(PreferenceError::Invalid(
                    "Unknown or repeated shortcut action".into(),
                ));
            }
            validate_combo(&binding.combo)?;
            effective.insert(binding.action, binding.combo.clone());
        }
        let mut combinations = HashMap::new();
        for (action, combo) in effective {
            // Normalize case for conflict checks: Qt treats letter key variants
            // as the same key; Shift is represented separately.
            let identity = (
                combo.key.to_ascii_lowercase(),
                combo.ctrl,
                combo.shift,
                combo.alt,
            );
            if let Some(previous) = combinations.insert(identity, action) {
                return Err(PreferenceError::Invalid(format!(
                    "Shortcut {} conflicts between {} and {}",
                    combo.display(),
                    previous.as_str(),
                    action.as_str()
                )));
            }
        }
        Ok(())
    }
}

fn validate_combo(combo: &KeyCombo) -> PreferenceResult<()> {
    let key = combo.key.as_str();
    if key.trim().is_empty() || key.len() > 64 || key.chars().any(char::is_control) {
        return Err(PreferenceError::Invalid(
            "Shortcut key must contain 1–64 printable characters".into(),
        ));
    }
    if matches!(
        key.to_ascii_lowercase().as_str(),
        "alt" | "altgr" | "ctrl" | "control" | "shift" | "meta" | "super" | "cmd" | "command"
    ) {
        return Err(PreferenceError::Invalid(
            "Essential actions cannot use modifier-only shortcuts".into(),
        ));
    }
    Ok(())
}
