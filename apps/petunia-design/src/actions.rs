//! Action-lane dispatch for the Freya shell (08.2, 15.G).
//!
//! Every activation travels the same lane: resolve the token to its payload,
//! check the registry availability rule, then dispatch. A blocked capability
//! is never dispatched, even if a stale token asks for it.

use freya::prelude::WritableUtils;
use petunia_design_application::menus::{self, ActionContext};
use petunia_design_application::tools::ToolKind;
use petunia_design_application::{ActionId, ActionRequest};
use petunia_design_shell::PetuniaShell;

/// User-typed values stay in the UI; `run_action_token` never dispatches a
/// bare menu token for them (payload `null` would only error silently), and
/// reports the action id so the caller opens the prompt dialog instead. The
/// dialog dispatches with the real typed value.
fn needs_typed_value(action_id: &str, payload: &serde_json::Value) -> bool {
    action_id == "ptnd.action.object.offset_path"
        && payload
            .get("distance")
            .or_else(|| payload.get("delta"))
            .and_then(serde_json::Value::as_f64)
            .is_none_or(|value| !value.is_finite())
}

/// User-chosen destinations stay in the UI; `run_action_token` only dispatches
/// actions that need no destination, and reports the action id either way.
fn needs_destination(action_id: &str) -> bool {
    matches!(
        action_id,
        "ptnd.action.file.new"
            | "ptnd.action.file.open"
            | "ptnd.action.file.recover"
            | "ptnd.action.file.save"
            | "ptnd.action.file.save_as"
            | "ptnd.action.file.export"
            | "ptnd.action.file.place"
            | "ptnd.action.file.close"
            | "ptnd.action.file.quit"
    )
}

/// Runs one menu/palette/toolbar token through the Action lane.
///
/// Returns the resolved action id, or `None` for an unknown token.
pub fn run_action_token(shell: &mut PetuniaShell, token: &str) -> Option<String> {
    run_action_token_result(shell, token).ok().flatten()
}
fn run_action_token_result(
    shell: &mut PetuniaShell,
    token: &str,
) -> Result<Option<String>, petunia_design_foundation::PetuniaError> {
    let Some((item, payload)) = menus::item_for_token(token) else {
        return Ok(None);
    };
    let action_id = menus::dispatch_action_id(item)
        .unwrap_or(item.surface)
        .to_string();
    let ctx: ActionContext = shell.bridge.action_context();
    if let Some(tool) = ToolKind::from_action_id(&action_id) {
        let enabled = menus::menu_bar(&ctx)
            .iter()
            .flat_map(|family| family.items())
            .any(|candidate| candidate.action_token == token && candidate.enabled);
        if enabled {
            shell.set_active_tool(tool);
        }
        return Ok(Some(action_id));
    }
    if action_id == ActionId::EDIT_PREFERENCES {
        return Ok(Some(action_id));
    }
    if needs_destination(&action_id) {
        return Ok(Some(action_id));
    }
    if needs_typed_value(&action_id, &payload) {
        return Ok(Some(action_id));
    }
    if !menus::availability(&action_id, &ctx).enabled {
        return Ok(Some(action_id));
    }
    shell
        .bridge
        .dispatch_action(ActionRequest::new(ActionId::new(&action_id), payload))?;
    Ok(Some(action_id))
}

#[cfg(test)]
pub fn run_action_id(shell: &mut PetuniaShell, action_id: &str) -> Option<String> {
    menus::MENU_BAR
        .iter()
        .flat_map(|family| family.items())
        .find(|item| menus::dispatch_action_id(item) == Some(action_id))
        .and_then(|item| run_action_token(shell, &menus::item_token(item)))
}

/// Desktop activations share the same routes for menus and keyboard shortcuts.
pub fn run_ui_token(ui: &crate::ui_state::UiShell, token: &str) -> Option<String> {
    let (item, _) = menus::item_for_token(token)?;
    let action = menus::dispatch_action_id(item)
        .unwrap_or(item.surface)
        .to_owned();
    if !menus::availability(&action, &ui.shell.peek().bridge.action_context()).enabled {
        return Some(action);
    }
    if crate::file_jobs::route_clipboard(ui, &action)
        || crate::file_workflows::route_file_action(ui, &action)
    {
        return Some(action);
    }
    match run_action_token_result(&mut ui.shell.clone().write(), token) {
        Ok(result) => result,
        Err(error) => {
            ui.file_error.clone().set(Some(error.to_string()));
            Some(action)
        }
    }
}
pub fn run_ui_id(ui: &crate::ui_state::UiShell, action_id: &str) -> Option<String> {
    menus::MENU_BAR
        .iter()
        .flat_map(|family| family.items())
        .find(|item| menus::dispatch_action_id(item) == Some(action_id))
        .and_then(|item| run_ui_token(ui, &menus::item_token(item)))
}
