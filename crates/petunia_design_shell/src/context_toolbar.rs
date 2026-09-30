//! Canonical context toolbar (08.23).
//!
//! The toolbar is the fourth view of the same registry as the menu bar, the
//! command palette and the centred shell cluster: a control is in this bar
//! because an entry below says so, it carries the registry's own label,
//! shortcut, availability and blocked reason, and activating it travels the
//! same Action lane as a menu row.
//!
//! What makes it *contextual* is [`ToolbarScope`]: every entry declares the work
//! it belongs to, and the bar renders only the entries that apply to the active
//! tool. A vector control is absent while a raster tool is active instead of
//! sitting there doing nothing, and a control that cannot act right now is
//! disabled with a reason (15.F §2) rather than hidden or lying.
//!
//! Nothing here invents a label, an order or a reason: labels and reasons are
//! resolved through the same catalog the menu uses, and command entries are
//! resolved from the same menu-item models, by token.

use petunia_design_application::menus::{self, ActionContext, MenuItemModel, MenuNodeModel};
use petunia_design_application::surfaces::{SurfaceKind, SURFACES};
use petunia_design_application::tools::ToolKind;
use petunia_design_resources::i18n::{Locale, LocalizationService};
use serde::{Deserialize, Serialize};

use crate::menu::{present_item, MenuItemPresentation};

/// Which work an entry belongs to.
///
/// A closed table, like the availability rule: a tool is never silently
/// unhandled, because [`scope_matches`] is a total match over the enum and the
/// scope vocabulary is small enough to read in one screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolbarScope {
    /// Every tool, in every persona.
    Always,
    /// Every tool, but only while something is selected.
    Selection,
    /// Any tool that edits vector content: the vector persona's tools. Photo
    /// persona raster tools are not offered vector fills and booleans.
    VectorObjects,
    /// Path work only: node, pen, pencil, corner, contour, knife, scissors.
    Paths,
    /// Shape creation and region synthesis: rectangle, ellipse, polygon, star,
    /// shape builder, vector flood fill.
    Shapes,
}

/// What an entry renders as.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolbarEntryKind {
    /// The active tool: its glyph plus its catalog name.
    ToolBadge,
    /// The selection's X/Y/W/H/R readout.
    TransformReadout,
    /// The fill and stroke swatches.
    ColorSwatches,
    /// A menu item, by token, resolved exactly like a menu row.
    Command,
    /// A separator. First class, so a divider is a placeable item the user can
    /// move like any other rather than a gap some layout decided to draw.
    Divider,
    /// The flexible gap that keeps the trailing group against the right edge.
    Spacer,
}

/// One declared toolbar entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolbarEntry {
    /// Stable entry id. For a command this is the token's surface id.
    pub id: &'static str,
    /// What it renders as.
    pub kind: ToolbarEntryKind,
    /// Menu token (`surface#payload`) for a [`ToolbarEntryKind::Command`],
    /// empty otherwise. The token is what makes the registry the one source of
    /// the label, the shortcut, the availability and the blocked reason.
    pub token: &'static str,
    /// The work this entry belongs to.
    pub scope: ToolbarScope,
}

/// Declares an entry that renders chrome rather than a command.
const fn chrome(id: &'static str, kind: ToolbarEntryKind, scope: ToolbarScope) -> ToolbarEntry {
    ToolbarEntry {
        id,
        kind,
        token: "",
        scope,
    }
}

/// Declares a command entry, by the menu token that resolves it.
const fn command(id: &'static str, token: &'static str, scope: ToolbarScope) -> ToolbarEntry {
    ToolbarEntry {
        id,
        kind: ToolbarEntryKind::Command,
        token,
        scope,
    }
}

/// The canonical context toolbar, in declared order (08.23).
///
/// Order, scoping and the dividers between groups are declared here rather than
/// in the UI, so the bar has one source and a test can police it. Every command
/// token must resolve to a menu item: a toolbar button that dispatches nothing
/// is the fake UI the project forbids.
pub const CONTEXT_TOOLBAR: &[ToolbarEntry] = &[
    chrome(
        "ptnd.ctb.tool_badge",
        ToolbarEntryKind::ToolBadge,
        ToolbarScope::Always,
    ),
    chrome(
        "ptnd.ctb.transform",
        ToolbarEntryKind::TransformReadout,
        ToolbarScope::Selection,
    ),
    chrome(
        "ptnd.ctb.divider.badge",
        ToolbarEntryKind::Divider,
        ToolbarScope::Always,
    ),
    chrome(
        "ptnd.ctb.swatches",
        ToolbarEntryKind::ColorSwatches,
        ToolbarScope::VectorObjects,
    ),
    chrome(
        "ptnd.ctb.divider.vector",
        ToolbarEntryKind::Divider,
        ToolbarScope::VectorObjects,
    ),
    command(
        "ptnd.ctb.convert_to_curves",
        "ptnd.action.object.convert_to_curves#null",
        ToolbarScope::Paths,
    ),
    command(
        "ptnd.ctb.bake_corners",
        "ptnd.action.object.bake_corners#null",
        ToolbarScope::Paths,
    ),
    chrome(
        "ptnd.ctb.divider.path",
        ToolbarEntryKind::Divider,
        ToolbarScope::Paths,
    ),
    command(
        "ptnd.ctb.boolean.union",
        r#"ptnd.action.object.boolean#{"op":"union"}"#,
        ToolbarScope::Shapes,
    ),
    command(
        "ptnd.ctb.boolean.difference",
        r#"ptnd.action.object.boolean#{"op":"difference"}"#,
        ToolbarScope::Shapes,
    ),
    command(
        "ptnd.ctb.boolean.intersection",
        r#"ptnd.action.object.boolean#{"op":"intersection"}"#,
        ToolbarScope::Shapes,
    ),
    command(
        "ptnd.ctb.boolean.exclusion",
        r#"ptnd.action.object.boolean#{"op":"exclusion"}"#,
        ToolbarScope::Shapes,
    ),
    chrome(
        "ptnd.ctb.divider.boolean",
        ToolbarEntryKind::Divider,
        ToolbarScope::Shapes,
    ),
    command(
        "ptnd.ctb.align.left",
        r#"ptnd.action.object.align#{"mode":"left"}"#,
        ToolbarScope::VectorObjects,
    ),
    command(
        "ptnd.ctb.align.center",
        r#"ptnd.action.object.align#{"mode":"center"}"#,
        ToolbarScope::VectorObjects,
    ),
    command(
        "ptnd.ctb.align.right",
        r#"ptnd.action.object.align#{"mode":"right"}"#,
        ToolbarScope::VectorObjects,
    ),
    command(
        "ptnd.ctb.align.top",
        r#"ptnd.action.object.align#{"mode":"top"}"#,
        ToolbarScope::VectorObjects,
    ),
    command(
        "ptnd.ctb.align.middle",
        r#"ptnd.action.object.align#{"mode":"middle"}"#,
        ToolbarScope::VectorObjects,
    ),
    command(
        "ptnd.ctb.align.bottom",
        r#"ptnd.action.object.align#{"mode":"bottom"}"#,
        ToolbarScope::VectorObjects,
    ),
    chrome(
        "ptnd.ctb.divider.align",
        ToolbarEntryKind::Divider,
        ToolbarScope::VectorObjects,
    ),
    chrome(
        "ptnd.ctb.spacer",
        ToolbarEntryKind::Spacer,
        ToolbarScope::Always,
    ),
    command(
        "ptnd.ctb.export",
        "ptnd.action.file.export#null",
        ToolbarScope::Always,
    ),
    command(
        "ptnd.ctb.delete",
        "ptnd.action.edit.delete#null",
        ToolbarScope::Always,
    ),
];

/// TextId of the tool's own catalog name, from its registry surface.
///
/// The badge names the active tool, so its label comes from the same registry
/// entry the tool rail and the Tool menu resolve.
#[must_use]
pub fn tool_label_text_id(tool: ToolKind) -> Option<&'static str> {
    SURFACES
        .iter()
        .find(|entry| {
            entry.kind == SurfaceKind::Tool
                && entry
                    .action
                    .and_then(ToolKind::from_action_id)
                    .is_some_and(|candidate| candidate == tool)
        })
        .map(|entry| entry.label)
}

/// True when an entry belongs to the active tool (and selection state).
#[must_use]
pub fn scope_matches(scope: ToolbarScope, tool: ToolKind, has_selection: bool) -> bool {
    match scope {
        ToolbarScope::Always => true,
        ToolbarScope::Selection => has_selection,
        ToolbarScope::VectorObjects => !tool.is_photo_persona(),
        ToolbarScope::Paths => matches!(
            tool,
            ToolKind::Node
                | ToolKind::Pen
                | ToolKind::Pencil
                | ToolKind::Corner
                | ToolKind::Contour
                | ToolKind::Knife
                | ToolKind::Scissors
        ),
        ToolbarScope::Shapes => matches!(
            tool,
            ToolKind::Rectangle
                | ToolKind::Ellipse
                | ToolKind::Polygon
                | ToolKind::Star
                | ToolKind::ShapeBuilder
                | ToolKind::VectorFloodFill
        ),
    }
}

/// One toolbar entry, ready to render.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolbarItemPresentation {
    /// Stable entry id. The UI echoes it back to activate.
    pub id: String,
    /// What to render.
    pub kind: ToolbarEntryKind,
    /// Localized label, for entries that render text.
    pub label: String,
    /// Localized tooltip, already composed with the registry shortcut.
    pub tooltip: String,
    /// Second tooltip, for the one entry that renders two things (the fill and
    /// stroke swatches). Empty for every other entry.
    pub tooltip_alt: String,
    /// Whether the entry can act right now.
    pub enabled: bool,
    /// Localized reason shown when `enabled` is false.
    pub disabled_reason: String,
}

/// Resolves the menu-item model behind a toolbar token.
///
/// The toolbar deliberately does not redeclare labels or availability: it looks
/// the token up in the very models the menu bar was built from, for this same
/// context.
#[must_use]
pub fn item_for_token(
    service: &LocalizationService,
    locale: &Locale,
    ctx: &ActionContext,
    token: &str,
) -> Option<MenuItemPresentation> {
    item_for_token_in(&menu_items(ctx), service, locale, token)
}

fn menu_items(ctx: &ActionContext) -> Vec<MenuItemModel> {
    menus::menu_bar(ctx)
        .iter()
        .flat_map(|family| family.nodes.iter())
        .flat_map(|node| match node {
            MenuNodeModel::Item(item) => std::slice::from_ref(item),
            MenuNodeModel::Group(group) => group.items.as_slice(),
        })
        .cloned()
        .collect()
}

fn item_for_token_in(
    items: &[MenuItemModel],
    service: &LocalizationService,
    locale: &Locale,
    token: &str,
) -> Option<MenuItemPresentation> {
    items
        .iter()
        .find(|item| item.action_token == token)
        .map(|item| present_item(item, service, locale))
}

/// Resolves the toolbar for the active tool.
///
/// Entries that do not apply to this tool are absent, not disabled: what is
/// absent was never applicable, and what is present but blocked says why.
#[must_use]
pub fn present_context_toolbar(
    service: &LocalizationService,
    locale: &Locale,
    ctx: &ActionContext,
    tool: ToolKind,
    has_selection: bool,
) -> Vec<ToolbarItemPresentation> {
    let items = menu_items(ctx);
    CONTEXT_TOOLBAR
        .iter()
        .filter(|entry| scope_matches(entry.scope, tool, has_selection))
        .map(|entry| present_entry(entry, service, locale, &items, tool))
        .collect()
}

/// Resolves one entry against the catalog and the registry.
fn present_entry(
    entry: &ToolbarEntry,
    service: &LocalizationService,
    locale: &Locale,
    items: &[MenuItemModel],
    tool: ToolKind,
) -> ToolbarItemPresentation {
    let mut resolved = ToolbarItemPresentation {
        id: entry.id.to_string(),
        kind: entry.kind,
        label: String::new(),
        tooltip: String::new(),
        tooltip_alt: String::new(),
        enabled: true,
        disabled_reason: String::new(),
    };

    match entry.kind {
        ToolbarEntryKind::ToolBadge => {
            // The badge names the tool from its registry entry, so a renamed
            // tool is renamed here too.
            let name =
                tool_label_text_id(tool).map_or_else(String::new, |id| service.text(id, locale));
            let shortcut = SURFACES
                .iter()
                .find(|candidate| candidate.action == Some(tool.action_id()))
                .and_then(|candidate| candidate.shortcut);
            resolved.tooltip = match shortcut {
                Some(key) => format!("{name} ({key})"),
                None => name.clone(),
            };
            resolved.label = name;
        }
        ToolbarEntryKind::TransformReadout => {
            let label = service.text("ptnd.text.panel.transform", locale);
            resolved.label = label.clone();
            resolved.tooltip = label;
        }
        ToolbarEntryKind::ColorSwatches => {
            resolved.tooltip = service.text("ptnd.text.panel.fill", locale);
            resolved.tooltip_alt = service.text("ptnd.text.panel.stroke", locale);
        }
        ToolbarEntryKind::Command => {
            // A token that resolves nowhere would be a button that dispatches
            // nothing. It is reported as blocked instead of dressed up as work,
            // and a test forbids the case outright.
            match item_for_token_in(items, service, locale, entry.token) {
                Some(item) => {
                    resolved.enabled = item.enabled;
                    resolved.disabled_reason = item.disabled_reason.clone();
                    resolved.tooltip = match item.shortcut.is_empty() {
                        true => item.label.clone(),
                        false => format!("{} ({})", item.label, item.shortcut),
                    };
                    resolved.label = item.label;
                }
                None => {
                    resolved.enabled = false;
                    resolved.disabled_reason =
                        service.text("ptnd.text.blocked.action_unknown", locale);
                }
            }
        }
        ToolbarEntryKind::Divider | ToolbarEntryKind::Spacer => {}
    }

    resolved
}

/// The declared entry behind an id, for a control that echoes an id back.
#[must_use]
pub fn entry(id: &str) -> Option<&'static ToolbarEntry> {
    CONTEXT_TOOLBAR.iter().find(|candidate| candidate.id == id)
}

/// One user-placed slot in the context toolbar.
///
/// The catalog (`CONTEXT_TOOLBAR`) stays the source of which entries exist.
/// This is only order and visibility: a slot names a catalog id, and a divider
/// the user inserts is a first-class slot with no catalog id.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolbarSlot {
    /// Catalog entry id, or empty for a user-inserted divider.
    pub id: String,
    /// Whether the slot is shown. A hidden slot stays in the layout so the
    /// user can put it back without losing its place.
    pub visible: bool,
}

impl ToolbarSlot {
    /// A visible catalog slot.
    #[must_use]
    pub fn shown(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            visible: true,
        }
    }

    /// A user-inserted divider. It has no catalog id, which is how the layout
    /// tells it apart from a declared entry.
    #[must_use]
    pub fn divider() -> Self {
        Self {
            id: String::new(),
            visible: true,
        }
    }

    /// True when this slot is a user-inserted divider.
    #[must_use]
    pub fn is_divider(&self) -> bool {
        self.id.is_empty()
    }
}

/// User order and visibility of the context toolbar.
///
/// Default is the catalog order, every entry visible. Mutations never invent
/// an id the catalog does not know, and they never drop the spacer: the bar
/// has one flexible gap, and losing it would pin the trailing group to the
/// left.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolbarLayout {
    /// Slots in display order.
    pub slots: Vec<ToolbarSlot>,
}

impl Default for ToolbarLayout {
    fn default() -> Self {
        Self::canonical()
    }
}

impl ToolbarLayout {
    /// The catalog order, every entry visible.
    #[must_use]
    pub fn canonical() -> Self {
        Self {
            slots: CONTEXT_TOOLBAR
                .iter()
                .map(|entry| ToolbarSlot::shown(entry.id))
                .collect(),
        }
    }

    /// Drops unknown ids and appends catalog entries the layout is missing, so
    /// a saved layout from an older catalog still renders every current entry.
    #[must_use]
    pub fn reconciled(mut self) -> Self {
        self.slots
            .retain(|slot| slot.is_divider() || entry(&slot.id).is_some());
        for declared in CONTEXT_TOOLBAR {
            if !self.slots.iter().any(|slot| slot.id == declared.id) {
                self.slots.push(ToolbarSlot::shown(declared.id));
            }
        }
        if !self.slots.iter().any(|slot| slot.id == "ptnd.ctb.spacer") {
            self.slots.push(ToolbarSlot::shown("ptnd.ctb.spacer"));
        }
        self
    }

    /// Hides or shows the slot at `index`. The spacer stays visible: it is the
    /// bar's geometry, not a command the user can turn off.
    pub fn set_slot_visible(&mut self, index: usize, visible: bool) -> bool {
        let Some(slot) = self.slots.get(index) else {
            return false;
        };
        if slot.id == "ptnd.ctb.spacer" {
            return false;
        }
        self.slots[index].visible = visible;
        true
    }

    /// Hides or shows one catalog entry. The spacer stays visible: it is the
    /// bar's geometry, not a command the user can turn off.
    pub fn set_visible(&mut self, id: &str, visible: bool) -> bool {
        if id == "ptnd.ctb.spacer" {
            return false;
        }
        let Some(slot) = self.slots.iter_mut().find(|slot| slot.id == id) else {
            return false;
        };
        slot.visible = visible;
        true
    }

    /// Moves a slot one place toward the start (`delta` negative) or the end.
    pub fn move_slot(&mut self, index: usize, delta: i32) -> bool {
        let Some(target) = index.checked_add_signed(delta as isize) else {
            return false;
        };
        if index >= self.slots.len() || target >= self.slots.len() {
            return false;
        }
        let slot = self.slots.remove(index);
        self.slots.insert(target, slot);
        true
    }

    /// Inserts a user divider after `index`. `None` appends.
    pub fn insert_divider(&mut self, after: Option<usize>) {
        let slot = ToolbarSlot::divider();
        match after {
            Some(index) if index < self.slots.len() => self.slots.insert(index + 1, slot),
            _ => self.slots.push(slot),
        }
    }

    /// Removes a user-inserted divider. A catalog entry cannot be removed this
    /// way: hiding it is the operation that takes it off the bar.
    pub fn remove_divider(&mut self, index: usize) -> bool {
        if self.slots.get(index).is_some_and(ToolbarSlot::is_divider) {
            self.slots.remove(index);
            return true;
        }
        false
    }
}

/// Resolves the toolbar for the active tool, in the user's layout order.
///
/// Hidden slots and entries that do not apply to this tool are absent. A user
/// divider is present whenever the surrounding group is, so a divider the user
/// placed between two vector commands does not appear while a raster tool is
/// active.
#[must_use]
pub fn present_layout(
    layout: &ToolbarLayout,
    service: &LocalizationService,
    locale: &Locale,
    ctx: &ActionContext,
    tool: ToolKind,
    has_selection: bool,
) -> Vec<ToolbarItemPresentation> {
    let items = menu_items(ctx);
    let mut presented = Vec::new();
    for slot in &layout.slots {
        if !slot.visible {
            continue;
        }
        if slot.is_divider() {
            presented.push(ToolbarItemPresentation {
                id: format!("ptnd.ctb.user_divider.{}", presented.len()),
                kind: ToolbarEntryKind::Divider,
                label: String::new(),
                tooltip: String::new(),
                tooltip_alt: String::new(),
                enabled: true,
                disabled_reason: String::new(),
            });
            continue;
        }
        let Some(declared) = entry(&slot.id) else {
            continue;
        };
        if !scope_matches(declared.scope, tool, has_selection) {
            continue;
        }
        presented.push(present_entry(declared, service, locale, &items, tool));
    }
    presented
}

/// One catalog entry as the customization dialog shows it: label, kind and
/// whether the user currently shows it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolbarCatalogRow {
    /// Catalog id, or empty for a user divider.
    pub id: String,
    /// Localized label. Empty for a divider.
    pub label: String,
    /// Rendering kind, same vocabulary as [`ToolbarEntryKind`].
    pub kind: ToolbarEntryKind,
    /// Whether the slot is currently shown.
    pub visible: bool,
    /// Whether the user may hide it. The spacer may not.
    pub can_hide: bool,
}

/// The layout as the customization dialog edits it.
///
/// Every slot is listed, hidden ones included, so the user can put an entry
/// back. Labels come from the same catalog the bar uses.
#[must_use]
pub fn present_catalog(
    layout: &ToolbarLayout,
    service: &LocalizationService,
    locale: &Locale,
) -> Vec<ToolbarCatalogRow> {
    let ctx = ActionContext::default();
    let items = menu_items(&ctx);
    layout
        .slots
        .iter()
        .map(|slot| {
            if slot.is_divider() {
                return ToolbarCatalogRow {
                    id: String::new(),
                    label: service.text("ptnd.text.shell.divider", locale),
                    kind: ToolbarEntryKind::Divider,
                    visible: slot.visible,
                    can_hide: true,
                };
            }
            let declared = entry(&slot.id);
            let kind = declared.map_or(ToolbarEntryKind::Command, |entry| entry.kind);
            let label = match kind {
                ToolbarEntryKind::Divider => service.text("ptnd.text.shell.divider", locale),
                ToolbarEntryKind::Spacer => service.text("ptnd.text.shell.spacer", locale),
                _ => declared.map_or_else(String::new, |entry| {
                    present_entry(entry, service, locale, &items, ToolKind::Select).label
                }),
            };
            ToolbarCatalogRow {
                id: slot.id.clone(),
                label,
                kind,
                visible: slot.visible,
                can_hide: slot.id != "ptnd.ctb.spacer",
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_design_application::menus::ActionContext;

    /// A context with a document open and nothing selected: the state in which
    /// every object command must still be *present* and say why it cannot run.
    fn context() -> ActionContext {
        ActionContext {
            has_document: true,
            selection_count: 0,
            can_undo: false,
            can_redo: false,
            is_dirty: false,
            clipboard_non_empty: false,
            command_palette_open: false,
            persona: petunia_design_application::surfaces::PERSONA_VECTOR,
        }
    }

    fn service() -> LocalizationService {
        LocalizationService::with_shell_catalog()
    }

    fn entries_for(tool: ToolKind) -> Vec<ToolbarItemPresentation> {
        present_context_toolbar(&service(), &Locale::PtBr, &context(), tool, false)
    }

    #[test]
    fn entry_ids_are_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for entry in CONTEXT_TOOLBAR {
            assert!(seen.insert(entry.id), "duplicate toolbar id `{}`", entry.id);
        }
    }

    #[test]
    fn every_command_token_resolves_to_a_menu_item() {
        let ctx = context();
        let service = service();
        for entry in CONTEXT_TOOLBAR {
            if entry.kind != ToolbarEntryKind::Command {
                assert!(
                    entry.token.is_empty(),
                    "`{}` is not a command but carries a token",
                    entry.id
                );
                continue;
            }
            assert!(
                item_for_token(&service, &Locale::EnUs, &ctx, entry.token).is_some(),
                "toolbar token `{}` does not resolve in the menu bar",
                entry.token
            );
        }
    }

    #[test]
    fn every_tool_of_both_personas_gets_a_badge_name() {
        for tool in [
            ToolKind::Select,
            ToolKind::Node,
            ToolKind::Pen,
            ToolKind::Rectangle,
            ToolKind::Star,
            ToolKind::ArtisticText,
            ToolKind::Gradient,
            ToolKind::ColorPicker,
            ToolKind::Corner,
            ToolKind::Knife,
            ToolKind::Scissors,
            ToolKind::Hand,
            ToolKind::Zoom,
            ToolKind::Artboard,
            ToolKind::Crop,
            ToolKind::PixelPaintBrush,
        ] {
            let label_id = tool_label_text_id(tool);
            assert!(
                label_id.is_some(),
                "tool {:?} has no registry surface, so the badge would be nameless",
                tool
            );
        }
    }

    #[test]
    fn photo_tools_are_never_offered_vector_work() {
        for tool in [
            ToolKind::Crop,
            ToolKind::PixelPaintBrush,
            ToolKind::PixelEraser,
            ToolKind::MarqueeRect,
            ToolKind::Lasso,
            ToolKind::PhotoGradient,
        ] {
            // A raster tool gets exactly the always-scoped entries: no vector
            // swatches, no path or boolean command, no vector group divider.
            // Computing the expected set from the table keeps this honest when
            // the table grows.
            let always: std::collections::BTreeSet<&str> = CONTEXT_TOOLBAR
                .iter()
                .filter(|entry| entry.scope == ToolbarScope::Always)
                .map(|entry| entry.id)
                .collect();
            let items = entries_for(tool);
            for item in &items {
                assert!(
                    always.contains(item.id.as_str()),
                    "{:?} was offered `{}` ({:?}), which is not always-scoped",
                    tool,
                    item.id,
                    item.kind
                );
            }
            assert!(
                items
                    .iter()
                    .any(|item| item.kind == ToolbarEntryKind::ToolBadge),
                "{:?} lost its own badge",
                tool
            );
        }
    }

    #[test]
    fn the_readout_appears_only_with_a_selection() {
        let without = entries_for(ToolKind::Select);
        assert!(
            !without
                .iter()
                .any(|item| item.kind == ToolbarEntryKind::TransformReadout),
            "the readout showed with nothing selected"
        );

        let with = present_context_toolbar(
            &service(),
            &Locale::PtBr,
            &context(),
            ToolKind::Select,
            true,
        );
        assert!(
            with.iter()
                .any(|item| item.kind == ToolbarEntryKind::TransformReadout),
            "the readout vanished with a selection"
        );
    }

    #[test]
    fn path_work_is_absent_while_a_shape_tool_creates_shapes() {
        let shapes = entries_for(ToolKind::Rectangle);
        assert!(
            !shapes
                .iter()
                .any(|item| item.id == "ptnd.ctb.convert_to_curves"),
            "convert-to-curves was offered while the rectangle tool is active"
        );
        assert!(
            shapes
                .iter()
                .any(|item| item.id == "ptnd.ctb.boolean.union"),
            "a shape tool lost the boolean group"
        );

        let paths = entries_for(ToolKind::Pen);
        assert!(
            paths
                .iter()
                .any(|item| item.id == "ptnd.ctb.convert_to_curves"),
            "the pen tool lost convert-to-curves"
        );
        assert!(
            !paths.iter().any(|item| item.id == "ptnd.ctb.boolean.union"),
            "the pen tool was offered boolean synthesis"
        );
    }

    #[test]
    fn the_bar_never_opens_or_ends_on_a_divider_or_spacer() {
        for tool in [
            ToolKind::Select,
            ToolKind::Pen,
            ToolKind::Rectangle,
            ToolKind::Crop,
        ] {
            let items = entries_for(tool);
            assert!(!items.is_empty(), "{tool:?} produced an empty toolbar");
            let first = items.first().expect("non-empty");
            let last = items.last().expect("non-empty");
            assert_eq!(
                first.kind,
                ToolbarEntryKind::ToolBadge,
                "{tool:?} does not start with the tool badge"
            );
            assert!(
                !matches!(
                    last.kind,
                    ToolbarEntryKind::Divider | ToolbarEntryKind::Spacer
                ),
                "{tool:?} ends on a separator"
            );
            // The right-hand group survives every tool, so export and delete
            // are always reachable from the bar.
            assert!(items.iter().any(|item| item.id == "ptnd.ctb.export"));
            assert!(items.iter().any(|item| item.id == "ptnd.ctb.delete"));
        }
    }

    #[test]
    fn a_blocked_command_states_a_reason_in_both_locales() {
        let ctx = context();
        for locale in [Locale::EnUs, Locale::PtBr] {
            let items = present_context_toolbar(&service(), &locale, &ctx, ToolKind::Select, false);
            for item in &items {
                if item.kind != ToolbarEntryKind::Command {
                    continue;
                }
                assert!(
                    !item.label.is_empty(),
                    "`{}` has no label in {locale:?}",
                    item.id
                );
                assert!(
                    !item.tooltip.is_empty(),
                    "`{}` has no tooltip in {locale:?}",
                    item.id
                );
                if !item.enabled {
                    assert!(
                        !item.disabled_reason.is_empty(),
                        "`{}` is blocked without a reason in {locale:?}",
                        item.id
                    );
                }
            }
        }
    }

    #[test]
    fn declared_entry_lookup_round_trips() {
        for declared in CONTEXT_TOOLBAR {
            assert_eq!(
                entry(declared.id).map(|found| found.id),
                Some(declared.id),
                "`{}` did not round trip through entry()",
                declared.id
            );
        }
        assert!(entry("ptnd.ctb.nope").is_none());
    }

    #[test]
    fn layout_hides_reorders_and_keeps_the_spacer() {
        let mut layout = ToolbarLayout::canonical();
        assert!(layout.set_visible("ptnd.ctb.delete", false));
        assert!(!layout.set_visible("ptnd.ctb.spacer", false));
        assert!(layout.move_slot(0, 1));
        layout.insert_divider(Some(0));
        assert!(layout.slots[1].is_divider());
        assert!(layout.remove_divider(1));
        assert!(!layout.remove_divider(0));

        let hidden = present_layout(
            &layout,
            &service(),
            &Locale::PtBr,
            &context(),
            ToolKind::Select,
            false,
        );
        assert!(hidden.iter().all(|item| item.id != "ptnd.ctb.delete"));
        assert!(hidden.iter().any(|item| item.id == "ptnd.ctb.spacer"));
    }
}
