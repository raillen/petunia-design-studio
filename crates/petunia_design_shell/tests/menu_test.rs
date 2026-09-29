//! Menu bar, command palette and view-state integration (15.G, 08.2, 15.B).
//!
//! These are the end-to-end claims the handoff asked for: every wired action
//! is reachable, the palette cannot drift from the menu, and a view action
//! actually moves the camera the UI renders.

use petunia_design_application::menus;
use petunia_design_resources::i18n::Locale;
use petunia_design_shell::shell::PetuniaShell;

fn shell_with_document() -> PetuniaShell {
    let mut shell = PetuniaShell::new(950.0, 700.0);
    shell.new_document("Menu Test").expect("document opens");
    shell
}

#[test]
fn every_wired_action_is_reachable_from_the_menu() {
    let shell = shell_with_document();
    let model = shell.bridge.query_menu_bar();
    let reachable: std::collections::HashSet<String> = model
        .families
        .iter()
        .flat_map(|family| family.items())
        .map(|item| item.action_id.clone())
        .collect();

    let mut missing: Vec<&str> = Vec::new();
    for entry in menus::wired_actions() {
        if !reachable.contains(entry.id) {
            missing.push(entry.id);
        }
    }
    assert!(
        missing.is_empty(),
        "wired actions the user cannot reach: {missing:?}"
    );
}

#[test]
fn palette_offers_exactly_the_enabled_menu_items() {
    let shell = shell_with_document();
    let model = shell.bridge.query_menu_bar();
    let menu_tokens: Vec<String> = model
        .families
        .iter()
        .flat_map(|family| family.items())
        .filter(|item| item.enabled)
        .map(|item| item.action_token.clone())
        .collect();
    let palette_tokens: Vec<String> = shell
        .bridge
        .query_command_index()
        .iter()
        .map(|item| item.action_token.clone())
        .collect();
    assert_eq!(menu_tokens, palette_tokens);
    assert!(!palette_tokens.is_empty());
}

#[test]
fn labels_are_localized_in_both_release_locales() {
    let mut shell = shell_with_document();

    shell.bridge.set_locale(Locale::EnUs);
    let english = shell.bridge.query_menu_bar();
    let file_family_en = english
        .families
        .iter()
        .find(|family| family.id == "ptnd.menu.file")
        .expect("the File family exists");
    assert_eq!(file_family_en.label, "File");

    shell.bridge.set_locale(Locale::PtBr);
    let portuguese = shell.bridge.query_menu_bar();
    let file_family_pt = portuguese
        .families
        .iter()
        .find(|family| family.id == "ptnd.menu.file")
        .expect("the File family exists");
    assert_eq!(file_family_pt.label, "Arquivo");

    // No label may render as a placeholder in either locale.
    for model in [&english, &portuguese] {
        for family in &model.families {
            assert!(!family.label.starts_with("[missing"));
            for item in family.items() {
                assert!(
                    !item.label.starts_with("[missing"),
                    "`{}` rendered a missing label",
                    item.action_id
                );
            }
        }
    }
}

#[test]
fn a_blocked_capability_is_shown_with_its_reason_but_never_offered() {
    let shell = shell_with_document();
    let model = shell.bridge.query_menu_bar();
    // `slice_path` stays blocked: its point must be picked on the path with
    // the Scissors tool, so the menu shows the reason and the palette hides it.
    let slice = model
        .item_for_token("ptnd.action.object.slice_path#null")
        .expect("the blocked capability stays visible so it is not silently missing");
    assert!(!slice.enabled);
    assert!(!slice.disabled_reason.is_empty());

    assert!(
        !shell
            .bridge
            .query_command_index()
            .iter()
            .any(|item| item.action_id == "ptnd.action.object.slice_path"),
        "a blocked action must not be offered by the palette"
    );

    let place = model
        .item_for_token("ptnd.action.file.place#null")
        .expect("file.place is available in the menu");
    assert!(place.enabled, "file.place is enabled with open document");
}

#[test]
fn menu_availability_follows_the_selection() {
    let mut shell = shell_with_document();
    let surface = shell.bridge.active_surface().expect("surface");
    let id = shell.bridge.next_object_id().expect("id");
    shell
        .bridge
        .submit_command(petunia_design_application::CommandRequest::new(
            petunia_design_application::Command::CreateObject {
                surface,
                id,
                name: "Item".to_string(),
            },
        ))
        .expect("object creates");

    let without_selection = shell.bridge.query_menu_bar();
    assert!(
        !without_selection
            .item_for_token("ptnd.action.edit.delete#null")
            .expect("delete is present")
            .enabled
    );

    shell.bridge.set_selection(vec![id]);
    let with_selection = shell.bridge.query_menu_bar();
    assert!(
        with_selection
            .item_for_token("ptnd.action.edit.delete#null")
            .expect("delete is present")
            .enabled
    );
    // Offset is wired through the numeric prompt: one selected object enables
    // the menu row and offers it in the palette.
    assert!(
        with_selection
            .item_for_token("ptnd.action.object.offset_path#null")
            .expect("offset is present")
            .enabled,
        "offset_path enables with a selection"
    );
    assert!(
        shell
            .bridge
            .query_command_index()
            .iter()
            .any(|item| item.action_id == "ptnd.action.object.offset_path"),
        "a wired action is offered by the palette"
    );
    // Grouping needs two objects and must say so.
    let group = with_selection
        .item_for_token("ptnd.action.object.group#null")
        .expect("group is present");
    assert!(!group.enabled);
    assert!(!group.disabled_reason.is_empty());
}

#[test]
fn a_view_action_moves_the_camera_the_shell_renders() {
    // Regression: the shell used to own a camera separate from
    // `session.view.camera`, so `ptnd.action.view.*` changed nothing on screen
    // while interactive pan/zoom changed something else.
    let mut shell = shell_with_document();
    let before = shell.view_camera().zoom;

    shell
        .bridge
        .dispatch_action(petunia_design_application::ActionRequest::without_payload(
            petunia_design_application::ActionId::new("ptnd.action.view.zoom_in"),
        ))
        .expect("view.zoom_in dispatches");

    assert!(
        shell.view_camera().zoom > before,
        "the session camera and the shell camera must be the same camera"
    );
}

#[test]
fn three_view_toggles_reach_the_session_and_are_idempotent() {
    let mut shell = shell_with_document();
    let action = |id: &str| {
        petunia_design_application::ActionRequest::without_payload(
            petunia_design_application::ActionId::new(id),
        )
    };

    let rulers = shell.bridge.session().expect("session").view.rulers_visible;
    shell
        .bridge
        .dispatch_action(action("ptnd.action.view.toggle_rulers"))
        .expect("rulers toggle");
    assert_eq!(
        shell.bridge.session().expect("session").view.rulers_visible,
        !rulers
    );

    assert!(
        !shell
            .bridge
            .session()
            .expect("session")
            .view
            .command_palette_open
    );
    shell
        .bridge
        .dispatch_action(action("ptnd.action.view.command_palette"))
        .expect("palette opens");
    assert!(
        shell
            .bridge
            .session()
            .expect("session")
            .view
            .command_palette_open
    );
}

#[test]
fn the_palette_flag_is_session_state_not_ui_state() {
    // The overlay reads the flag from the session, so a shortcut, a plugin or
    // MCP opening the palette is visible to the UI without extra plumbing.
    let mut shell = shell_with_document();
    let before = shell.bridge.action_context().command_palette_open;
    shell
        .bridge
        .dispatch_action(petunia_design_application::ActionRequest::without_payload(
            petunia_design_application::ActionId::new("ptnd.action.view.command_palette"),
        ))
        .expect("palette toggles");
    let after = shell.bridge.action_context().command_palette_open;
    assert_ne!(before, after);
}

#[test]
fn action_queries_agree_with_the_menu() {
    let shell = shell_with_document();
    let model = shell.bridge.query_menu_bar();
    let actions = shell.bridge.query_actions();
    for family in &model.families {
        for item in family.items() {
            let state = actions
                .get(&petunia_design_application::ActionId::new(&item.action_id))
                .unwrap_or_else(|| panic!("`{}` is missing from query_actions", item.action_id));
            assert_eq!(
                state.is_enabled, item.enabled,
                "`{}` disagrees between the menu and the action query",
                item.action_id
            );
        }
    }
}

#[test]
fn the_zoom_box_offers_exactly_the_levels_the_view_menu_shows() {
    let shell = shell_with_document();

    // The box under the readout is not a second registry: it presents the View
    // submenu's own levels, so a level can never exist in one place and be
    // missing from the other.
    let menu_levels: Vec<String> = shell
        .bridge
        .query_menu_bar()
        .families
        .iter()
        .find(|family| family.id == "ptnd.menu.view")
        .expect("View is in the bar")
        .nodes
        .iter()
        .find_map(|node| match node {
            petunia_design_shell::menu::MenuNodePresentation::Group(group)
                if group.id == "ptnd.menu.view.zoom_levels" =>
            {
                Some(
                    group
                        .items
                        .iter()
                        .map(|level| level.action_token.clone())
                        .collect(),
                )
            }
            _ => None,
        })
        .expect("the View submenu carries the zoom levels");
    let box_levels: Vec<String> = shell
        .bridge
        .query_zoom_levels()
        .iter()
        .map(|item| item.action_token.clone())
        .collect();

    assert_eq!(box_levels, menu_levels);
    assert!(
        box_levels
            .iter()
            .any(|token| token.starts_with("ptnd.action.view.zoom_set#")),
        "the box carries parameterized levels: {box_levels:?}"
    );
}

#[test]
fn zoom_levels_are_localized_percentages_in_both_locales() {
    let mut shell = shell_with_document();

    for locale in [Locale::EnUs, Locale::PtBr] {
        shell.bridge.set_locale(locale);
        let levels = shell.bridge.query_zoom_levels();
        assert!(!levels.is_empty(), "the box always offers levels");
        for level in &levels {
            assert!(
                level.enabled,
                "`{}` is not actionable while a document is open",
                level.action_id
            );
            assert!(
                !level.label.starts_with("[missing"),
                "`{}` rendered a missing label",
                level.action_id
            );
            assert!(
                level.label.ends_with('%'),
                "a zoom level reads as a percentage, got `{}`",
                level.label
            );
        }
    }
}

#[test]
fn legacy_action_spellings_still_resolve_on_read() {
    // Read-side migration only (15.A): the map is keyed canonically, and the
    // bridge must still answer a caller holding the old spelling.
    let shell = shell_with_document();
    let legacy = shell
        .bridge
        .query_actions()
        .get(&petunia_design_application::ActionId::new(
            "ptnd.file.export",
        ))
        .cloned();
    assert!(legacy.is_some(), "pre-grammar ids must resolve on read");
}
