//! Ephemeral UI state for the Petunia Freya shell.
//!
//! The domain session owns documents, tools and history. The values below own
//! what the screen is doing right now: which family popup is open, what the
//! palette query says, whether the toolbar dialog shows.
//!
//! Every field is created with `use_state` inside a component `render`, never
//! detached from a scope, so every `read` subscribes and every `set`
//! re-renders. A state detached from a scope subscribes nothing and its
//! handlers stay inert, which is exactly the not-clickable shell this module
//! exists to prevent.

use freya::prelude::*;
use petunia_design_shell::PetuniaShell;

use crate::theme::{AccentColor, IconStyle, BLOOM};

/// Shared shell state plus the overlay state only the screen keeps.
///
/// `Clone` shares the same underlying states; `PartialEq` compares by
/// identity so the diffing pass treats one shared instance as equal.
#[derive(Clone)]
pub struct UiShell {
    pub shell: State<PetuniaShell>,
    pub open_family: State<Option<usize>>,
    pub palette_open: State<bool>,
    pub palette_query: State<String>,
    pub customize_open: State<bool>,
    pub accent: State<AccentColor>,
    pub icon_style: State<IconStyle>,
    /// Hovered control: `(id, text)` shown in the status bar while hovered.
    /// Tooltips live here instead of in `TooltipContainer`, whose `Attached`
    /// wrapper takes the button out of the layout flow and displaces it.
    pub hovered: State<Option<(String, String)>>,
}

impl PartialEq for UiShell {
    fn eq(&self, other: &Self) -> bool {
        self.shell == other.shell
            && self.open_family == other.open_family
            && self.palette_open == other.palette_open
            && self.palette_query == other.palette_query
            && self.customize_open == other.customize_open
            && self.accent == other.accent
            && self.icon_style == other.icon_style
            && self.hovered == other.hovered
    }
}

impl UiShell {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        shell: State<PetuniaShell>,
        open_family: State<Option<usize>>,
        palette_open: State<bool>,
        palette_query: State<String>,
        customize_open: State<bool>,
        accent: State<AccentColor>,
        icon_style: State<IconStyle>,
        hovered: State<Option<(String, String)>>,
    ) -> Self {
        Self {
            shell,
            open_family,
            palette_open,
            palette_query,
            customize_open,
            accent,
            icon_style,
            hovered,
        }
    }

    /// Builds the shell state from one fresh scope-owned document shell.
    #[must_use]
    pub fn fresh(shell: State<PetuniaShell>) -> Self {
        let open_family = use_state(|| None);
        let palette_open = use_state(|| false);
        let palette_query = use_state(String::new);
        let customize_open = use_state(|| false);
        let accent = use_state(|| BLOOM);
        let icon_style = use_state(IconStyle::default);
        let hovered = use_state(|| None);
        Self::new(
            shell,
            open_family,
            palette_open,
            palette_query,
            customize_open,
            accent,
            icon_style,
            hovered,
        )
    }
}
