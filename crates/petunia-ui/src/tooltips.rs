//! Tooltips as a secondary path, never the only one (ADR-0012 D5).
//!
//! A tooltip shows the action name plus the shortcut it has *now*,
//! read from the shortcut table instead of a literal. The same text is
//! available in the command palette and the shortcut editor, which are
//! the canonical paths, so nothing here is required to discover a
//! shortcut.

use crate::shortcuts::{ActionId, ShortcutTable};

/// Milliseconds of hover before a tooltip appears.
pub const HOVER_DELAY_MS: u64 = 400;

/// Tooltip text for one action, with the shortcut it has right now.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tooltip {
    /// Human name of the action.
    pub name: String,
    /// The combo currently bound, already formatted for display.
    pub shortcut: Option<String>,
}

impl Tooltip {
    /// One-line form: name, then the shortcut when there is one.
    #[must_use]
    pub fn display(&self) -> String {
        match &self.shortcut {
            Some(shortcut) => format!("{} ({})", self.name, shortcut),
            None => self.name.clone(),
        }
    }
}

/// Build the tooltip for an action from the live table.
///
/// Returns `None` when the action has no name, which cannot happen for
/// the built-in set but keeps the contract honest for extensions.
#[must_use]
pub fn tooltip_for(action: ActionId, table: &ShortcutTable) -> Option<Tooltip> {
    let name = action_name(action)?;
    Some(Tooltip {
        name,
        shortcut: table.combo_for(action).map(|combo| combo.display()),
    })
}

/// Same text the command palette and the shortcut editor show, so the
/// tooltip can never be the only place a shortcut is written.
#[must_use]
pub fn palette_label_for(action: ActionId, table: &ShortcutTable) -> Option<String> {
    tooltip_for(action, table).map(|tooltip| tooltip.display())
}

/// Whether focus alone should reveal the tooltip without waiting for
/// the hover delay. Keyboard users never wait (D5).
#[must_use]
pub const fn reveals_on_focus() -> bool {
    true
}

/// Whether a tooltip may be shown after hovering for the given time.
#[must_use]
pub const fn hover_reveals_after(elapsed_ms: u64) -> bool {
    elapsed_ms >= HOVER_DELAY_MS
}

/// Display name of an action, in the app's language.
#[must_use]
pub fn action_name(action: ActionId) -> Option<String> {
    let name = match action {
        ActionId::SelectTool => "Selecionar",
        ActionId::NodeTool => "Editar nodes",
        ActionId::PenTool => "Caneta",
        ActionId::BrushTool => "Pincel",
        ActionId::EraserTool => "Borracha",
        ActionId::Undo => "Desfazer",
        ActionId::Redo => "Refazer",
        ActionId::SelectAll => "Selecionar tudo",
        ActionId::Group => "Agrupar",
        ActionId::Ungroup => "Desagrupar",
        ActionId::BringForward => "Trazer para frente",
        ActionId::SendBackward => "Enviar para trás",
        ActionId::Confirm => "Confirmar",
        ActionId::Cancel => "Cancelar",
        ActionId::PanTool => "Mover canvas",
        ActionId::NudgeLeft => "Mover 1 px à esquerda",
        ActionId::NudgeRight => "Mover 1 px à direita",
        ActionId::NudgeUp => "Mover 1 px para cima",
        ActionId::NudgeDown => "Mover 1 px para baixo",
        ActionId::CommandPalette => "Paleta de comandos",
        ActionId::ManagePersonas => "Gerenciar personas",
        ActionId::Preferences => "Preferências",
        ActionId::ShortcutEditor => "Editor de atalhos",
        ActionId::FocusNext => "Próxima região do workspace",
        ActionId::FocusPrevious => "Região anterior do workspace",
    };
    Some(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shortcuts::KeyCombo;

    #[test]
    fn tooltip_reads_the_current_shortcut_not_a_literal() {
        let table = ShortcutTable::defaults();
        let tooltip = tooltip_for(ActionId::Undo, &table).expect("named action");
        assert_eq!(tooltip.name, "Desfazer");
        assert_eq!(tooltip.shortcut.as_deref(), Some("Ctrl+z"));
        assert_eq!(tooltip.display(), "Desfazer (Ctrl+z)");

        // Rebind the action and the tooltip follows the table.
        let mut table = table;
        assert!(table
            .rebind(ActionId::Undo, KeyCombo::key("y").with_ctrl())
            .is_ok());
        let tooltip = tooltip_for(ActionId::Undo, &table).expect("named action");
        assert_eq!(tooltip.shortcut.as_deref(), Some("Ctrl+y"));
    }

    #[test]
    fn an_unbound_action_shows_only_its_name() {
        let mut table = ShortcutTable::defaults();
        table.clear_binding(ActionId::PenTool);
        let tooltip = tooltip_for(ActionId::PenTool, &table).expect("named action");
        assert_eq!(tooltip.shortcut, None);
        assert_eq!(tooltip.display(), "Caneta");
    }

    #[test]
    fn the_same_text_is_available_in_the_palette() {
        let table = ShortcutTable::defaults();
        let from_tooltip = tooltip_for(ActionId::Group, &table)
            .expect("named")
            .display();
        let from_palette = palette_label_for(ActionId::Group, &table).expect("named");
        assert_eq!(from_tooltip, from_palette);
    }

    #[test]
    fn focus_reveals_immediately_but_hover_waits_400ms() {
        assert!(reveals_on_focus());
        assert!(!hover_reveals_after(399));
        assert!(hover_reveals_after(400));
        assert!(hover_reveals_after(HOVER_DELAY_MS));
    }

    #[test]
    fn workspace_navigation_labels_follow_rebound_shortcuts() {
        let mut table = ShortcutTable::defaults();
        table
            .rebind(ActionId::FocusNext, KeyCombo::key("f7"))
            .unwrap();
        let tooltip = tooltip_for(ActionId::FocusNext, &table).unwrap();
        assert_eq!(tooltip.name, "Próxima região do workspace");
        assert_eq!(tooltip.shortcut.as_deref(), Some("f7"));
        assert_eq!(
            palette_label_for(ActionId::FocusNext, &table),
            Some(tooltip.display())
        );
        for action in [
            ActionId::CommandPalette,
            ActionId::ManagePersonas,
            ActionId::Preferences,
            ActionId::ShortcutEditor,
            ActionId::FocusPrevious,
        ] {
            assert!(tooltip_for(action, &table).is_some());
        }
    }
}
