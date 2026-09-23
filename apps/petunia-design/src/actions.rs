//! Action-lane dispatch for the Freya shell (08.2, 15.G).
//!
//! Every activation travels the same lane: resolve the token to its payload,
//! check the registry availability rule, then dispatch. A blocked capability
//! is never dispatched, even if a stale token asks for it.

use petunia_design_application::menus::{self, ActionContext};
use petunia_design_application::{ActionId, ActionRequest};
use petunia_design_shell::PetuniaShell;

/// User-chosen destinations stay in the UI; `run_action_token` only dispatches
/// actions that need no destination, and reports the action id either way.
fn needs_destination(action_id: &str) -> bool {
    matches!(
        action_id,
        "ptnd.action.file.open" | "ptnd.action.file.save_as" | "ptnd.action.file.export"
    )
}

/// Runs one menu/palette/toolbar token through the Action lane.
///
/// Returns the resolved action id, or `None` for an unknown token.
pub fn run_action_token(shell: &mut PetuniaShell, token: &str) -> Option<String> {
    let (item, payload) = menus::item_for_token(token)?;
    let action_id = item.surface.to_string();
    if needs_destination(&action_id) {
        return Some(action_id);
    }
    let ctx: ActionContext = shell.bridge.action_context();
    if !menus::availability(&action_id, &ctx).enabled {
        return Some(action_id);
    }
    let _ = shell
        .bridge
        .dispatch_action(ActionRequest::new(ActionId::new(&action_id), payload));
    Some(action_id)
}
