//! Menu and command-palette presentation models (08.2, 15.G).
//!
//! The shell owns exactly one thing the application layer must not: turning
//! `ptnd.text.*` TextIds into user-visible strings. Structure (families,
//! groups, order, payloads, persona scoping, availability) comes from
//! [`petunia_design_application::menus`]; this module only resolves it for a
//! locale and flattens it into a DTO a UI toolkit can render.
//!
//! Nothing here invents items, reorders families, adds a group or decides
//! availability: if a menu row disappears, it disappeared in the registry.
//! Block reasons are TextIds too, so a blocked item is as translatable as a
//! label — that is why the catalog has a `ptnd.text.blocked.*` section.

use petunia_design_application::menus::{
    self, ActionContext, MenuFamilyModel, MenuItemModel, MenuNodeModel,
};
use petunia_design_resources::i18n::{Locale, LocalizationService};
use serde::{Deserialize, Serialize};

/// One menu entry, ready to render.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MenuItemPresentation {
    /// Canonical `ActionId` to dispatch.
    pub action_id: String,
    /// Opaque token the UI echoes back to activate this exact item. Two items
    /// may share an action id and differ only here.
    pub action_token: String,
    /// Localized label.
    pub label: String,
    /// Localized shortcut hint (empty when unbound).
    pub shortcut: String,
    /// Whether the item can be activated now.
    pub enabled: bool,
    /// Localized reason shown when `enabled` is false.
    pub disabled_reason: String,
}

/// One submenu, ready to render.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MenuGroupPresentation {
    /// Stable group id (`ptnd.menu.<family>.<group>`).
    pub id: String,
    /// Localized group title.
    pub label: String,
    /// Items in display order.
    pub items: Vec<MenuItemPresentation>,
}

/// One family node: an item or a submenu.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MenuNodePresentation {
    /// An activatable item.
    Item(MenuItemPresentation),
    /// A submenu holding items.
    Group(MenuGroupPresentation),
}

/// One menu bar family, ready to render.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MenuFamilyPresentation {
    /// Stable family id (`ptnd.menu.<family>`).
    pub id: String,
    /// Localized family title.
    pub label: String,
    /// Nodes in display order.
    pub nodes: Vec<MenuNodePresentation>,
}

impl MenuFamilyPresentation {
    /// Every item in the family, flattened in display order.
    pub fn items(&self) -> impl Iterator<Item = &MenuItemPresentation> {
        self.nodes.iter().flat_map(|node| match node {
            MenuNodePresentation::Item(item) => std::slice::from_ref(item),
            MenuNodePresentation::Group(group) => group.items.as_slice(),
        })
    }
}

/// The whole menu bar for one locale and context.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MenuBarPresentationModel {
    /// Families in display order.
    pub families: Vec<MenuFamilyPresentation>,
}

impl MenuBarPresentationModel {
    /// Total number of items across families, groups included.
    #[must_use]
    pub fn item_count(&self) -> usize {
        self.families
            .iter()
            .map(|family| family.items().count())
            .sum()
    }

    /// Number of actionable items.
    #[must_use]
    pub fn enabled_count(&self) -> usize {
        self.families
            .iter()
            .flat_map(MenuFamilyPresentation::items)
            .filter(|item| item.enabled)
            .count()
    }

    /// Looks up one item by its activation token.
    #[must_use]
    pub fn item_for_token(&self, token: &str) -> Option<&MenuItemPresentation> {
        self.families
            .iter()
            .flat_map(MenuFamilyPresentation::items)
            .find(|item| item.action_token == token)
    }
}

/// Resolves one item model against the localization service.
#[must_use]
pub fn present_item(
    item: &MenuItemModel,
    service: &LocalizationService,
    locale: &Locale,
) -> MenuItemPresentation {
    MenuItemPresentation {
        action_id: item.action_id.clone(),
        action_token: item.action_token.clone(),
        label: service.text(&item.label, locale),
        shortcut: item.shortcut.clone().unwrap_or_default(),
        enabled: item.enabled,
        disabled_reason: item
            .disabled_reason_id
            .as_deref()
            .map_or_else(String::new, |reason| service.text(reason, locale)),
    }
}

/// Resolves one family node.
fn present_node(
    node: &MenuNodeModel,
    service: &LocalizationService,
    locale: &Locale,
) -> MenuNodePresentation {
    match node {
        MenuNodeModel::Item(item) => {
            MenuNodePresentation::Item(present_item(item, service, locale))
        }
        MenuNodeModel::Group(group) => MenuNodePresentation::Group(MenuGroupPresentation {
            id: group.id.clone(),
            label: service.text(&group.label, locale),
            items: group
                .items
                .iter()
                .map(|item| present_item(item, service, locale))
                .collect(),
        }),
    }
}

/// Resolves a whole menu bar.
#[must_use]
pub fn present_menu_bar(
    families: &[MenuFamilyModel],
    service: &LocalizationService,
    locale: &Locale,
) -> MenuBarPresentationModel {
    MenuBarPresentationModel {
        families: families
            .iter()
            .map(|family| MenuFamilyPresentation {
                id: family.id.clone(),
                label: service.text(&family.label, locale),
                nodes: family
                    .nodes
                    .iter()
                    .map(|node| present_node(node, service, locale))
                    .collect(),
            })
            .collect(),
    }
}

/// One persona as the switcher renders it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonaPresentation {
    /// Registered persona id (`ptnd.persona.*`).
    pub id: String,
    /// Localized persona name.
    pub label: String,
    /// Localized one-line description of what the mode is for.
    pub hint: String,
}

/// Localization TextId for the hint shown when a persona becomes active.
///
/// Closed by design, like the availability table: a persona declared in the
/// registry without a hint here simply renders no hint, and
/// `every_persona_has_a_hint_in_both_locales` fails if the catalog entry is
/// missing. Returning `None` rather than building an id from the persona's
/// name keeps the shell from inventing TextIds.
#[must_use]
pub fn persona_hint_text_id(persona: &str) -> Option<&'static str> {
    match persona {
        petunia_design_application::surfaces::PERSONA_VECTOR => {
            Some("ptnd.text.persona.vector.hint")
        }
        petunia_design_application::surfaces::PERSONA_PHOTO => Some("ptnd.text.persona.photo.hint"),
        _ => None,
    }
}

/// Resolves one registered persona against the localization service.
#[must_use]
pub fn present_persona(
    entry: &petunia_design_application::surfaces::SurfaceEntry,
    service: &LocalizationService,
    locale: &Locale,
) -> PersonaPresentation {
    PersonaPresentation {
        id: entry.id.to_string(),
        label: service.text(entry.label, locale),
        hint: persona_hint_text_id(entry.id)
            .map_or_else(String::new, |id| service.text(id, locale)),
    }
}

/// Resolves every registered persona, in switcher order.
#[must_use]
pub fn present_personas(
    service: &LocalizationService,
    locale: &Locale,
) -> Vec<PersonaPresentation> {
    petunia_design_application::surfaces::personas()
        .iter()
        .map(|entry| present_persona(entry, service, locale))
        .collect()
}

/// What kind of control a shell cluster entry renders as.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShellControlKind {
    /// A square icon button.
    Icon,
    /// A readout pill, such as the zoom percentage.
    Readout,
    /// A separator. First class, so a divider is a placeable item the user can
    /// move like any other rather than a gap the UI decided to draw.
    Divider,
}

/// One entry of the centred shell control cluster (08.2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellControlPresentation {
    /// Stable `SurfaceId` of the control. The UI echoes it back to activate.
    pub id: String,
    /// What the control renders as.
    pub kind: ShellControlKind,
    /// Localized tooltip, already composed with the registry shortcut.
    pub tooltip: String,
    /// Whether the control can act right now.
    pub enabled: bool,
}

/// One declared shell control: identity, rendering kind and the action it runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShellControl {
    /// Stable `SurfaceId` of the control.
    pub id: &'static str,
    /// What it renders as.
    pub kind: ShellControlKind,
    /// Action id it dispatches. Empty for a readout or a divider.
    pub action: &'static str,
}

/// Declares a control that runs an action.
const fn control(id: &'static str, action: &'static str) -> ShellControl {
    ShellControl {
        id,
        kind: ShellControlKind::Icon,
        action,
    }
}

/// The centred cluster of the menu bar row, in canonical order (08.2).
///
/// Declared here rather than in the UI so order, tooltip and availability all
/// have one source. Undo/redo and view zoom belong to the shell, not to the
/// document tab strip, which keeps only snapping on its far right.
pub const SHELL_CONTROLS: &[ShellControl] = &[
    control("ptnd.surface.shell.undo", "ptnd.action.edit.undo"),
    control("ptnd.surface.shell.redo", "ptnd.action.edit.redo"),
    ShellControl {
        id: "ptnd.surface.shell.divider",
        kind: ShellControlKind::Divider,
        action: "",
    },
    control("ptnd.surface.shell.zoom_out", "ptnd.action.view.zoom_out"),
    ShellControl {
        id: "ptnd.surface.shell.zoom_readout",
        kind: ShellControlKind::Readout,
        action: "",
    },
    control("ptnd.surface.shell.zoom_in", "ptnd.action.view.zoom_in"),
    control("ptnd.surface.shell.fit", "ptnd.action.view.fit_surface"),
];

/// The action a shell control runs, when it runs one.
#[must_use]
pub fn shell_control_action(id: &str) -> Option<&'static str> {
    SHELL_CONTROLS
        .iter()
        .find(|entry| entry.id == id && !entry.action.is_empty())
        .map(|entry| entry.action)
}

/// Composes a tooltip from a registry label and its bound shortcut.
///
/// The two halves both come from the registry, so the tooltip cannot drift
/// from the menu; only the `label (shortcut)` shape lives here.
#[must_use]
fn compose_tooltip(label: &str, shortcut: Option<&str>) -> String {
    match shortcut {
        Some(key) => format!("{label} ({key})"),
        None => label.to_string(),
    }
}

/// The localized label of a registry surface, for a control that renders text.
#[must_use]
pub fn surface_label(service: &LocalizationService, locale: &Locale, id: &str) -> Option<String> {
    petunia_design_application::surfaces::surface(id).map(|entry| service.text(entry.label, locale))
}

/// Resolves the centred shell control cluster for the current context.
#[must_use]
pub fn present_shell_controls(
    service: &LocalizationService,
    locale: &Locale,
    ctx: &ActionContext,
) -> Vec<ShellControlPresentation> {
    SHELL_CONTROLS
        .iter()
        .map(|entry| {
            let resolved = petunia_design_application::surfaces::surface(entry.id);
            let label_id = resolved.map(|surface| surface.label);
            let shortcut = resolved.and_then(|surface| surface.shortcut);
            let tooltip = match (entry.kind, label_id) {
                (ShellControlKind::Icon, Some(label_id)) => {
                    compose_tooltip(&service.text(label_id, locale), shortcut)
                }
                // The readout is a control: clicking it opens the zoom levels,
                // so its registry label is its tooltip. A divider is not a
                // control the user activates.
                (ShellControlKind::Readout, Some(label_id)) => service.text(label_id, locale),
                _ => String::new(),
            };
            let enabled = match entry.kind {
                // A divider is not a control.
                ShellControlKind::Divider => false,
                // The readout always opens its list of levels; each level is
                // judged on its own by the popup, which reads the registry.
                ShellControlKind::Readout => true,
                ShellControlKind::Icon => {
                    !entry.action.is_empty()
                        && petunia_design_application::menus::availability(entry.action, ctx)
                            .enabled
                }
            };
            ShellControlPresentation {
                id: entry.id.to_string(),
                kind: entry.kind,
                tooltip,
                enabled,
            }
        })
        .collect()
}

/// Resolves the levels offered by the shell's zoom box (08.2).
///
/// The popup under the readout is not a second registry: it presents the very
/// items the View submenu carries, localized once and judged by the same
/// availability rule, so the box cannot offer a level the menu lacks.
#[must_use]
pub fn present_zoom_levels(
    service: &LocalizationService,
    locale: &Locale,
    ctx: &ActionContext,
) -> Vec<MenuItemPresentation> {
    petunia_design_application::menus::zoom_levels(ctx)
        .iter()
        .map(|item| present_item(item, service, locale))
        .collect()
}

/// Resolves the command palette index (enabled items only, menu order).
#[must_use]
pub fn present_command_index(
    service: &LocalizationService,
    locale: &Locale,
    ctx: &ActionContext,
) -> Vec<MenuItemPresentation> {
    menus::command_index(ctx)
        .iter()
        .map(|item| present_item(item, service, locale))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_design_application::surfaces::{
        SurfaceStatus, PERSONA_PHOTO, PERSONA_VECTOR, SURFACES,
    };
    use std::collections::HashSet;

    fn service() -> (LocalizationService, Locale) {
        (LocalizationService::with_shell_catalog(), Locale::PtBr)
    }

    fn context() -> ActionContext {
        ActionContext {
            has_document: true,
            selection_count: 3,
            can_undo: true,
            can_redo: true,
            is_dirty: true,
            command_palette_open: false,
            persona: PERSONA_VECTOR,
        }
    }

    #[test]
    fn presentation_never_contains_a_placeholder_label() {
        let (service, locale) = service();
        let model = present_menu_bar(&menus::menu_bar(&context()), &service, &locale);
        for family in &model.families {
            assert!(
                !family.label.starts_with("[missing"),
                "family `{}` has no catalog entry",
                family.id
            );
            assert!(!family.nodes.is_empty());
            for node in &family.nodes {
                match node {
                    MenuNodePresentation::Item(item) => {
                        assert!(
                            !item.label.starts_with("[missing"),
                            "item `{}` has no catalog entry",
                            item.action_id
                        );
                        assert!(
                            !item.label.trim().is_empty(),
                            "item `{}` rendered an empty label",
                            item.action_id
                        );
                    }
                    MenuNodePresentation::Group(group) => {
                        assert!(
                            !group.label.starts_with("[missing"),
                            "group `{}` has no catalog entry",
                            group.id
                        );
                        assert!(!group.items.is_empty());
                    }
                }
            }
        }
    }

    #[test]
    fn every_registry_text_id_resolves_in_both_release_locales() {
        // Coverage gate: the shell catalog must answer for every label the
        // registry references, or the UI would ship `[missing: …]`.
        let service = LocalizationService::with_shell_catalog();
        let mut checked = HashSet::new();
        for entry in SURFACES {
            if !checked.insert(entry.label) {
                continue;
            }
            for locale in [Locale::EnUs, Locale::PtBr] {
                let resolved = service.text(entry.label, &locale);
                assert!(
                    !resolved.starts_with("[missing"),
                    "`{}` is missing from the {} catalog",
                    entry.label,
                    locale.tag()
                );
            }
        }
    }

    #[test]
    fn every_registry_block_reason_resolves_in_both_release_locales() {
        // A block reason is shown to the user, so it is user-visible text and
        // must resolve like a label. This is the gate that keeps a raw English
        // sentence from creeping back into the registry.
        let service = LocalizationService::with_shell_catalog();
        let mut checked = HashSet::new();
        for entry in SURFACES {
            let SurfaceStatus::Disabled(reason) = entry.status else {
                continue;
            };
            if !checked.insert(reason) {
                continue;
            }
            for locale in [Locale::EnUs, Locale::PtBr] {
                let resolved = service.text(reason, &locale);
                assert!(
                    !resolved.starts_with("[missing"),
                    "`{}` (block reason of `{}`) is missing from the {} catalog",
                    reason,
                    entry.id,
                    locale.tag()
                );
            }
        }
    }

    #[test]
    fn every_blocked_item_renders_a_localized_reason() {
        // Both sources of a block — a registry `Disabled` row and a session
        // availability rule — must arrive as readable text in both locales.
        let service = LocalizationService::with_shell_catalog();
        for persona in [PERSONA_VECTOR, PERSONA_PHOTO] {
            let ctx = ActionContext {
                persona,
                ..ActionContext::default()
            };
            for locale in [Locale::EnUs, Locale::PtBr] {
                let model = present_menu_bar(&menus::menu_bar(&ctx), &service, &locale);
                for item in model
                    .families
                    .iter()
                    .flat_map(MenuFamilyPresentation::items)
                {
                    if item.enabled {
                        continue;
                    }
                    assert!(
                        !item.disabled_reason.is_empty(),
                        "`{}` is blocked with no reason in {}",
                        item.action_id,
                        locale.tag()
                    );
                    assert!(
                        !item.disabled_reason.starts_with("[missing"),
                        "`{}` blocks with an unresolved TextId in {}",
                        item.action_id,
                        locale.tag()
                    );
                }
            }
        }
    }

    #[test]
    fn every_menu_label_resolves_in_both_release_locales() {
        let service = LocalizationService::with_shell_catalog();
        for family in menus::MENU_BAR {
            for locale in [Locale::EnUs, Locale::PtBr] {
                assert!(!service.text(family.label, &locale).starts_with("[missing"));
                for node in family.nodes {
                    match node {
                        menus::MenuNode::Item(item) => assert!(
                            !service.text(item.label, &locale).starts_with("[missing"),
                            "`{}` is missing from the {} catalog",
                            item.label,
                            locale.tag()
                        ),
                        menus::MenuNode::Group(group) => {
                            assert!(
                                !service.text(group.label, &locale).starts_with("[missing"),
                                "`{}` is missing from the {} catalog",
                                group.label,
                                locale.tag()
                            );
                            for item in group.items {
                                assert!(
                                    !service.text(item.label, &locale).starts_with("[missing"),
                                    "`{}` is missing from the {} catalog",
                                    item.label,
                                    locale.tag()
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn palette_offers_exactly_the_enabled_menu_items() {
        let (service, locale) = service();
        let ctx = context();
        let bar = present_menu_bar(&menus::menu_bar(&ctx), &service, &locale);
        let menu_tokens: Vec<String> = bar
            .families
            .iter()
            .flat_map(MenuFamilyPresentation::items)
            .filter(|item| item.enabled)
            .map(|item| item.action_token.clone())
            .collect();
        let palette_tokens: Vec<String> = present_command_index(&service, &locale, &ctx)
            .iter()
            .map(|item| item.action_token.clone())
            .collect();
        assert_eq!(menu_tokens, palette_tokens);
    }

    #[test]
    fn disabled_items_carry_a_localized_surface_and_a_reason() {
        let (service, locale) = service();
        let model = present_menu_bar(&menus::menu_bar(&context()), &service, &locale);
        // `offset_path` is wired through the numeric prompt, so with a
        // selection it is enabled; `slice_path` stays disabled with a
        // localized reason pointing at the Scissors gesture.
        let offset = model
            .item_for_token("ptnd.action.object.offset_path#null")
            .expect("offset_path is present and wired");
        assert!(offset.enabled, "offset opens the numeric prompt");
        let slice = model
            .item_for_token("ptnd.action.object.slice_path#null")
            .expect("slice_path is present so the blocked capability is visible");
        assert!(!slice.enabled);
        assert!(!slice.disabled_reason.is_empty());
        assert!(!slice.disabled_reason.starts_with("ptnd.text."));
    }

    #[test]
    fn persona_families_reach_the_presentation_layer() {
        let (service, locale) = service();
        let vector = present_menu_bar(
            &menus::menu_bar(&ActionContext {
                persona: PERSONA_VECTOR,
                ..context()
            }),
            &service,
            &locale,
        );
        let photo = present_menu_bar(
            &menus::menu_bar(&ActionContext {
                persona: PERSONA_PHOTO,
                ..context()
            }),
            &service,
            &locale,
        );
        assert!(vector.families.iter().any(|f| f.id == "ptnd.menu.vector"));
        assert!(!vector.families.iter().any(|f| f.id == "ptnd.menu.image"));
        assert!(photo.families.iter().any(|f| f.id == "ptnd.menu.image"));
        assert!(!photo.families.iter().any(|f| f.id == "ptnd.menu.vector"));
    }

    #[test]
    fn groups_survive_presentation_with_their_items() {
        let (service, locale) = service();
        let model = present_menu_bar(&menus::menu_bar(&context()), &service, &locale);
        let object = model
            .families
            .iter()
            .find(|family| family.id == "ptnd.menu.object")
            .expect("object family is present");
        let align = object
            .nodes
            .iter()
            .find_map(|node| match node {
                MenuNodePresentation::Group(group) if group.id == "ptnd.menu.object.align" => {
                    Some(group)
                }
                _ => None,
            })
            .expect("align is a submenu");
        assert_eq!(align.items.len(), 6);
        assert!(align.label.len() > 2);
    }

    #[test]
    fn every_persona_has_a_hint_in_both_locales() {
        let service = LocalizationService::with_shell_catalog();
        let personas = present_personas(&service, &Locale::EnUs);
        assert!(
            personas.len() >= 2,
            "the switcher offers at least vector and photo"
        );
        for persona in &personas {
            assert!(!persona.label.is_empty());
            assert!(
                !persona.label.starts_with("[missing"),
                "persona `{}` has no catalog label",
                persona.id
            );
            for locale in [Locale::EnUs, Locale::PtBr] {
                let resolved = persona_hint_text_id(&persona.id)
                    .map(|id| service.text(id, &locale))
                    .unwrap_or_default();
                assert!(
                    !resolved.is_empty() && !resolved.starts_with("[missing"),
                    "persona `{}` has no hint in {}",
                    persona.id,
                    locale.tag()
                );
            }
        }
    }

    #[test]
    fn persona_presentation_follows_the_registry_and_the_locale() {
        let service = LocalizationService::with_shell_catalog();
        let english = present_personas(&service, &Locale::EnUs);
        let portuguese = present_personas(&service, &Locale::PtBr);
        assert_eq!(english.len(), portuguese.len());
        assert_eq!(english[0].id, PERSONA_VECTOR);
        assert_eq!(english[0].label, "Vector Persona");
        assert_eq!(portuguese[0].label, "Persona Vetorial");
        assert_eq!(english[1].id, PERSONA_PHOTO);
        assert_eq!(english[1].label, "Photo Persona");
        assert_eq!(portuguese[1].label, "Persona Foto");
    }

    #[test]
    fn token_lookup_returns_none_for_unknown_tokens() {
        let (service, locale) = service();
        let model = present_menu_bar(&menus::menu_bar(&context()), &service, &locale);
        assert!(model.item_for_token("ptnd.action.nope#null").is_none());
        assert!(model.enabled_count() > 0);
        // The presentation must carry exactly the items the model resolved for
        // this context — persona scoping included, so the photo-only Image
        // family is absent here.
        assert_eq!(
            model.item_count(),
            menus::menu_bar(&context())
                .iter()
                .map(|family| family.items().count())
                .sum::<usize>()
        );
    }
}
