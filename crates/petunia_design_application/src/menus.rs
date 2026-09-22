//! Registry-driven menu model (15.G, 08.2, 12.10).
//!
//! The menu bar is **derived**, never hand-written. Every item names a
//! [`SurfaceEntry`] already tracked by the surface registry — an `Action` row
//! or a `Tool` row — and the action it dispatches is the registry's own
//! binding. That is what makes "the wired actions are reachable" a
//! machine-checked claim instead of a UI screenshot: a wired action that no
//! family lists fails `every_wired_action_is_reachable_from_the_menu`.
//!
//! What the model deliberately does *not* do:
//! - invent action ids (a menu item may not create reachability the Action
//!   lane does not have);
//! - hide a blocked capability (a registry `Disabled` row surfaces as a
//!   disabled item carrying its reason, per 15.F §2 "no fake UI");
//! - spell a user-visible string (labels *and* block reasons are `ptnd.text.*`
//!   TextIds, so the catalog is the only place text lives);
//! - decide what is selected (that is the session's job — see
//!   [`MenuItemModel::action_token`]).
//!
//! Two levels, no more: a family holds nodes, and a node is either an item or
//! a group. That is enough for the menus the product actually has, and it keeps
//! the DTO a UI toolkit renders flat enough to be honest about.
//!
//! Parameterized actions (`object.align`, `object.boolean`, …) carry their
//! variant in the item payload, exactly like a real menu carrying a
//! parameter. The session fills in `surface`/`ids` from the live selection
//! when the payload does not name explicit targets, so the same item works
//! from a menu click, a shortcut, the command palette, MCP or a plugin.

use crate::surfaces::{
    SurfaceEntry, SurfaceKind, SurfaceStatus, PERSONA_PHOTO, PERSONA_VECTOR, SURFACES,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Every persona the shell can be in. Used as the default when a caller has not
/// switched yet, and by tests that need to prove reachability across modes.
pub const ALL_PERSONAS: &[&str] = &[PERSONA_VECTOR, PERSONA_PHOTO];

/// One menu entry: a registry surface plus the payload that specializes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuItem {
    /// Stable `SurfaceId` of the `Action` or `Tool` row this item reaches.
    pub surface: &'static str,
    /// Localization `TextId`. Equals the registry label for unparameterized
    /// items; a variant carries its own, more specific TextId.
    pub label: &'static str,
    /// JSON payload literal. `"null"` when the action takes no parameters.
    pub payload: &'static str,
}

/// A collapsible submenu inside a family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuGroup {
    /// Stable group id, `ptnd.menu.<family>.<group>`.
    pub id: &'static str,
    /// Localization `TextId` for the group title.
    pub label: &'static str,
    /// Items in display order.
    pub items: &'static [MenuItem],
}

/// One entry of a family: either an item or a submenu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuNode {
    /// An activatable item.
    Item(MenuItem),
    /// A submenu holding items.
    Group(MenuGroup),
}

/// One top-level menu bar family and its ordered nodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuFamily {
    /// Stable family id, `ptnd.menu.<family>`.
    pub id: &'static str,
    /// Localization `TextId` for the family title.
    pub label: &'static str,
    /// Personas this family belongs to. Empty means every persona sees it.
    /// This is what makes "the Vector menu is contextual to the vector persona"
    /// a registry fact the tests can check.
    pub personas: &'static [&'static str],
    /// Nodes in display order.
    pub nodes: &'static [MenuNode],
}

impl MenuFamily {
    /// Every item in the family, flattened in display order.
    pub fn items(&self) -> impl Iterator<Item = &MenuItem> {
        self.nodes.iter().flat_map(|node| match node {
            MenuNode::Item(item) => std::slice::from_ref(item),
            MenuNode::Group(group) => group.items,
        })
    }

    /// Every group in the family, in display order.
    pub fn groups(&self) -> impl Iterator<Item = &MenuGroup> {
        self.nodes.iter().filter_map(|node| match node {
            MenuNode::Group(group) => Some(group),
            MenuNode::Item(_) => None,
        })
    }

    /// Whether this family is part of the given persona's menu bar.
    #[must_use]
    pub fn visible_in(&self, persona: &str) -> bool {
        self.personas.is_empty() || self.personas.contains(&persona)
    }
}

/// Declares an unparameterized item: registry id, registry label, `null` payload.
const fn item(surface: &'static str, label: &'static str) -> MenuItem {
    MenuItem {
        surface,
        label,
        payload: "null",
    }
}

/// Declares a parameterized item: registry id, variant label, exact payload.
const fn variant(surface: &'static str, label: &'static str, payload: &'static str) -> MenuItem {
    MenuItem {
        surface,
        label,
        payload,
    }
}

/// Declares a submenu: group id, group label, items.
const fn group(id: &'static str, label: &'static str, items: &'static [MenuItem]) -> MenuNode {
    MenuNode::Group(MenuGroup { id, label, items })
}

/// Declares a plain item node.
const fn node(item: MenuItem) -> MenuNode {
    MenuNode::Item(item)
}

/// Align submenu: the six alignment modes of one parameterized action.
const ALIGN_ITEMS: &[MenuItem] = &[
    variant(
        "ptnd.action.object.align",
        "ptnd.text.object.align.left",
        r#"{"mode":"left"}"#,
    ),
    variant(
        "ptnd.action.object.align",
        "ptnd.text.object.align.center",
        r#"{"mode":"center"}"#,
    ),
    variant(
        "ptnd.action.object.align",
        "ptnd.text.object.align.right",
        r#"{"mode":"right"}"#,
    ),
    variant(
        "ptnd.action.object.align",
        "ptnd.text.object.align.top",
        r#"{"mode":"top"}"#,
    ),
    variant(
        "ptnd.action.object.align",
        "ptnd.text.object.align.middle",
        r#"{"mode":"middle"}"#,
    ),
    variant(
        "ptnd.action.object.align",
        "ptnd.text.object.align.bottom",
        r#"{"mode":"bottom"}"#,
    ),
];

/// Distribute submenu.
const DISTRIBUTE_ITEMS: &[MenuItem] = &[
    variant(
        "ptnd.action.object.distribute",
        "ptnd.text.object.distribute.horizontal",
        r#"{"axis":"horizontal"}"#,
    ),
    variant(
        "ptnd.action.object.distribute",
        "ptnd.text.object.distribute.vertical",
        r#"{"axis":"vertical"}"#,
    ),
];

/// Boolean submenu. It lives in the Vector family only: combining paths is
/// vector work, and the same item must not appear twice or its token — which
/// carries the item's payload — would stop identifying one slot.
const BOOLEAN_ITEMS: &[MenuItem] = &[
    variant(
        "ptnd.action.object.boolean",
        "ptnd.text.object.boolean.union",
        r#"{"op":"union"}"#,
    ),
    variant(
        "ptnd.action.object.boolean",
        "ptnd.text.object.boolean.difference",
        r#"{"op":"difference"}"#,
    ),
    variant(
        "ptnd.action.object.boolean",
        "ptnd.text.object.boolean.intersection",
        r#"{"op":"intersection"}"#,
    ),
    variant(
        "ptnd.action.object.boolean",
        "ptnd.text.object.boolean.exclusion",
        r#"{"op":"exclusion"}"#,
    ),
];

/// Zoom levels shared by `View → Zoom Levels` and by the shell's zoom box
/// (08.2). One parameterized action carries the level, exactly the way the
/// alignment variants carry their mode, so the box and the submenu read the
/// same items: the box cannot offer a level the menu does not have.
const ZOOM_LEVELS: &[MenuItem] = &[
    variant(
        "ptnd.action.view.zoom_set",
        "ptnd.text.view.zoom_25",
        r#"{"zoom":0.25}"#,
    ),
    variant(
        "ptnd.action.view.zoom_set",
        "ptnd.text.view.zoom_50",
        r#"{"zoom":0.5}"#,
    ),
    // 100% keeps the pre-existing "actual size" action rather than restating it
    // as a level: one 100% in the registry, reachable from the same group.
    item("ptnd.action.view.zoom_100", "ptnd.text.view.zoom_100"),
    variant(
        "ptnd.action.view.zoom_set",
        "ptnd.text.view.zoom_200",
        r#"{"zoom":2.0}"#,
    ),
    variant(
        "ptnd.action.view.zoom_set",
        "ptnd.text.view.zoom_400",
        r#"{"zoom":4.0}"#,
    ),
    variant(
        "ptnd.action.view.zoom_set",
        "ptnd.text.view.zoom_800",
        r#"{"zoom":8.0}"#,
    ),
];

/// Canonical menu bar (08.2, 15.G). Order here is the order on screen.
pub const MENU_BAR: &[MenuFamily] = &[
    MenuFamily {
        id: "ptnd.menu.file",
        label: "ptnd.text.menu.file",
        personas: &[],
        nodes: &[
            node(item("ptnd.action.file.new", "ptnd.text.file.new")),
            node(item("ptnd.action.file.open", "ptnd.text.file.open")),
            node(item("ptnd.action.file.save", "ptnd.text.file.save")),
            node(item("ptnd.action.file.save_as", "ptnd.text.file.save_as")),
            node(item("ptnd.action.file.export", "ptnd.text.file.export")),
            // Blocked, and shown as blocked rather than omitted silently.
            node(item("ptnd.action.file.place", "ptnd.text.file.place")),
        ],
    },
    MenuFamily {
        id: "ptnd.menu.edit",
        label: "ptnd.text.menu.edit",
        personas: &[],
        nodes: &[
            node(item("ptnd.action.edit.undo", "ptnd.text.edit.undo")),
            node(item("ptnd.action.edit.redo", "ptnd.text.edit.redo")),
            node(item(
                "ptnd.action.edit.duplicate",
                "ptnd.text.edit.duplicate",
            )),
            node(item("ptnd.action.edit.delete", "ptnd.text.edit.delete")),
        ],
    },
    MenuFamily {
        id: "ptnd.menu.select",
        label: "ptnd.text.menu.select",
        personas: &[],
        nodes: &[
            node(item("ptnd.action.edit.select_all", "ptnd.text.select.all")),
            node(item("ptnd.action.edit.deselect", "ptnd.text.select.none")),
        ],
    },
    MenuFamily {
        id: "ptnd.menu.object",
        label: "ptnd.text.menu.object",
        personas: &[],
        nodes: &[
            node(item("ptnd.action.object.group", "ptnd.text.object.group")),
            node(item(
                "ptnd.action.object.ungroup",
                "ptnd.text.object.ungroup",
            )),
            group(
                "ptnd.menu.object.align",
                "ptnd.text.object.align",
                ALIGN_ITEMS,
            ),
            group(
                "ptnd.menu.object.distribute",
                "ptnd.text.object.distribute",
                DISTRIBUTE_ITEMS,
            ),
            node(item("ptnd.action.object.lock", "ptnd.text.object.lock")),
            node(item("ptnd.action.object.hide", "ptnd.text.object.hide")),
        ],
    },
    MenuFamily {
        id: "ptnd.menu.layer",
        label: "ptnd.text.menu.layer",
        personas: &[],
        nodes: &[
            group(
                "ptnd.menu.layer.arrange",
                "ptnd.text.layer.arrange",
                &[
                    item(
                        "ptnd.action.object.arrange.front",
                        "ptnd.text.object.arrange.front",
                    ),
                    item(
                        "ptnd.action.object.arrange.back",
                        "ptnd.text.object.arrange.back",
                    ),
                ],
            ),
            group(
                "ptnd.menu.layer.clip_mask",
                "ptnd.text.object.clip_mask",
                &[
                    item(
                        "ptnd.action.object.clip_mask.create",
                        "ptnd.text.object.clip_mask",
                    ),
                    item(
                        "ptnd.action.object.clip_mask.release",
                        "ptnd.text.object.clip_mask_release",
                    ),
                ],
            ),
        ],
    },
    // Contextual to the vector persona: path editing and shape creation are
    // vector work, so they do not appear while the photo persona is active.
    MenuFamily {
        id: "ptnd.menu.vector",
        label: "ptnd.text.menu.vector",
        personas: &[PERSONA_VECTOR],
        nodes: &[
            node(item(
                "ptnd.action.object.convert_to_curves",
                "ptnd.text.object.convert_to_curves",
            )),
            node(item(
                "ptnd.action.object.bake_corners",
                "ptnd.text.object.bake_corners",
            )),
            group(
                "ptnd.menu.vector.boolean",
                "ptnd.text.object.boolean",
                BOOLEAN_ITEMS,
            ),
            // `object.offset_path` and `object.slice_path` are deliberately
            // disabled rather than absent: both need a distance/point the user
            // must choose, and inventing a default here would be a fake command.
            group(
                "ptnd.menu.vector.path",
                "ptnd.text.panel.stroke",
                &[
                    item(
                        "ptnd.action.object.offset_path",
                        "ptnd.text.object.offset_path",
                    ),
                    item(
                        "ptnd.action.object.slice_path",
                        "ptnd.text.object.slice_path",
                    ),
                ],
            ),
            group(
                "ptnd.menu.vector.shapes",
                "ptnd.text.tool.shape_builder",
                &[
                    item("ptnd.tool.vector.rectangle", "ptnd.text.tool.rectangle"),
                    item("ptnd.tool.vector.ellipse", "ptnd.text.tool.ellipse"),
                    item("ptnd.tool.vector.polygon", "ptnd.text.tool.polygon"),
                    item("ptnd.tool.vector.star", "ptnd.text.tool.star"),
                ],
            ),
            group(
                "ptnd.menu.vector.draw",
                "ptnd.text.tool.pen",
                &[
                    item("ptnd.tool.vector.pen", "ptnd.text.tool.pen"),
                    item("ptnd.tool.vector.pencil", "ptnd.text.tool.pencil"),
                    item("ptnd.tool.vector.node", "ptnd.text.tool.node"),
                    item("ptnd.tool.vector.corner", "ptnd.text.tool.corner"),
                    item("ptnd.tool.vector.knife", "ptnd.text.tool.knife"),
                    item("ptnd.tool.vector.contour", "ptnd.text.tool.contour"),
                ],
            ),
        ],
    },
    // Contextual to the photo persona.
    MenuFamily {
        id: "ptnd.menu.image",
        label: "ptnd.text.menu.image",
        personas: &[PERSONA_PHOTO],
        nodes: &[
            // No image *command* is offered here, because none exists yet:
            // `file.place` stays in File where users look for it, and inventing
            // placeholder commands would be the fake UI 15.F §2 forbids. What
            // the mode can really do today is pick an image tool.
            group(
                "ptnd.menu.image.tools",
                "ptnd.text.panel.transform",
                &[
                    item("ptnd.tool.photo.crop", "ptnd.text.tool.crop"),
                    item("ptnd.tool.photo.gradient", "ptnd.text.tool.gradient"),
                    item("ptnd.tool.photo.eyedropper", "ptnd.text.tool.eyedropper"),
                    item("ptnd.tool.photo.brush", "ptnd.text.tool.brush"),
                    item("ptnd.tool.photo.eraser", "ptnd.text.tool.eraser"),
                ],
            ),
        ],
    },
    MenuFamily {
        id: "ptnd.menu.view",
        label: "ptnd.text.menu.view",
        personas: &[],
        nodes: &[
            node(item("ptnd.action.view.zoom_in", "ptnd.text.view.zoom_in")),
            node(item("ptnd.action.view.zoom_out", "ptnd.text.view.zoom_out")),
            group(
                "ptnd.menu.view.zoom_levels",
                "ptnd.text.view.zoom_levels",
                ZOOM_LEVELS,
            ),
            node(item(
                "ptnd.action.view.fit_surface",
                "ptnd.text.view.fit_surface",
            )),
            node(item(
                "ptnd.action.view.toggle_rulers",
                "ptnd.text.view.rulers",
            )),
            node(item(
                "ptnd.action.view.toggle_snapping",
                "ptnd.text.view.snapping",
            )),
            node(item(
                "ptnd.action.view.command_palette",
                "ptnd.text.view.command_palette",
            )),
        ],
    },
];

/// Session facts that decide whether an item can run right now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionContext {
    /// A document is open in the active session.
    pub has_document: bool,
    /// Number of selected objects.
    pub selection_count: usize,
    /// The history can undo.
    pub can_undo: bool,
    /// The history can redo.
    pub can_redo: bool,
    /// The document has unsaved modifications.
    pub is_dirty: bool,
    /// A command palette overlay is currently open.
    pub command_palette_open: bool,
    /// Persona the shell is in. Decides which families are reachable.
    pub persona: &'static str,
}

impl Default for ActionContext {
    fn default() -> Self {
        Self {
            has_document: false,
            selection_count: 0,
            can_undo: false,
            can_redo: false,
            is_dirty: false,
            command_palette_open: false,
            persona: PERSONA_VECTOR,
        }
    }
}

/// Whether an action can run, with the user-visible reason when it cannot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Availability {
    /// True when dispatching the action would do something.
    pub enabled: bool,
    /// `ptnd.text.*` TextId of the reason. Always `Some` when `enabled` is
    /// false (15.F §2). A TextId rather than a sentence: the reason is shown
    /// to the user, so it is translatable like any other string.
    pub reason: Option<&'static str>,
}

impl Availability {
    /// The action can run.
    pub const ENABLED: Self = Self {
        enabled: true,
        reason: None,
    };

    /// The action cannot run, with the stated reason TextId.
    #[must_use]
    pub const fn blocked(reason: &'static str) -> Self {
        Self {
            enabled: false,
            reason: Some(reason),
        }
    }
}

/// One resolved menu item, ready for presentation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MenuItemModel {
    /// Stable `SurfaceId` from the registry.
    pub surface_id: String,
    /// Canonical `ActionId` to dispatch. For a `Tool` row this is the action
    /// the tool binds, not the row id.
    pub action_id: String,
    /// Opaque token the UI echoes back to activate this exact item
    /// (`<surface_id>#<payload json>`). Two items may share an action id and
    /// differ only here, so the token is the item's identity on the wire.
    pub action_token: String,
    /// Localization `TextId` for the label.
    pub label: String,
    /// Shortcut in UI form, when the registry binds one.
    pub shortcut: Option<String>,
    /// Whether the item is actionable in the current context.
    pub enabled: bool,
    /// `ptnd.text.*` TextId of the reason when `enabled` is false.
    pub disabled_reason_id: Option<String>,
}

/// One resolved submenu, ready for presentation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MenuGroupModel {
    /// Stable group id.
    pub id: String,
    /// Localization `TextId` for the group title.
    pub label: String,
    /// Items in display order.
    pub items: Vec<MenuItemModel>,
}

/// One resolved family node: an item or a submenu.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MenuNodeModel {
    /// An activatable item.
    Item(MenuItemModel),
    /// A submenu holding items.
    Group(MenuGroupModel),
}

/// One resolved menu family, ready for presentation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MenuFamilyModel {
    /// Stable family id (`ptnd.menu.<family>`).
    pub id: String,
    /// Localization `TextId` for the family title.
    pub label: String,
    /// Nodes in display order.
    pub nodes: Vec<MenuNodeModel>,
}

impl MenuFamilyModel {
    /// Every item in the family, flattened in display order.
    pub fn items(&self) -> impl Iterator<Item = &MenuItemModel> {
        self.nodes.iter().flat_map(|node| match node {
            MenuNodeModel::Item(item) => std::slice::from_ref(item),
            MenuNodeModel::Group(group) => group.items.as_slice(),
        })
    }
}

/// Resolves a `surface#payload` token back to its item.
///
/// The UI never composes payloads; it echoes a token it was given. Unknown
/// tokens are rejected rather than guessed at.
#[must_use]
pub fn item_for_token(token: &str) -> Option<(&'static MenuItem, Value)> {
    let (surface_id, payload) = token.split_once('#')?;
    let payload: Value = serde_json::from_str(payload).ok()?;
    for family in MENU_BAR {
        for candidate in family.items() {
            if candidate.surface == surface_id && payload_literal(candidate) == payload {
                return Some((candidate, payload));
            }
        }
    }
    None
}

/// Parsed payload of an item. Returns `Null` when the literal is malformed,
/// which `menu_payloads_are_valid_json` rules out at test time.
#[must_use]
pub fn payload_literal(item: &MenuItem) -> Value {
    serde_json::from_str(item.payload).unwrap_or(Value::Null)
}

/// Builds the opaque token identifying one item on the UI wire.
#[must_use]
pub fn item_token(item: &MenuItem) -> String {
    format!("{}#{}", item.surface, item.payload)
}

/// Looks up a registry row by id.
#[must_use]
pub fn surface_entry(id: &str) -> Option<&'static SurfaceEntry> {
    SURFACES.iter().find(|entry| entry.id == id)
}

/// The action a menu item dispatches.
///
/// An `Action` row is its own action id; a `Tool` row dispatches the action it
/// binds. Every other kind is not menu-reachable.
#[must_use]
pub fn dispatch_action_id(item: &MenuItem) -> Option<&'static str> {
    let entry = surface_entry(item.surface)?;
    match entry.kind {
        SurfaceKind::Action => Some(entry.id),
        SurfaceKind::Tool => entry.action,
        _ => None,
    }
}

/// Availability of one action id in the given context.
///
/// Closed by design: an action without a rule is reported blocked, and
/// `every_menu_action_has_an_availability_rule` fails the build if one is
/// added to the registry without a rule here.
#[must_use]
pub fn availability(action_id: &str, ctx: &ActionContext) -> Availability {
    let some_selection = || ctx.selection_count >= 1;
    match action_id {
        // File: export and save need a document; save only when it is dirty.
        "ptnd.action.file.new" | "ptnd.action.file.open" => Availability::ENABLED,
        "ptnd.action.file.save" => {
            if !ctx.has_document {
                Availability::blocked("ptnd.text.blocked.no_document")
            } else if !ctx.is_dirty {
                Availability::blocked("ptnd.text.blocked.no_unsaved_changes")
            } else {
                Availability::ENABLED
            }
        }
        "ptnd.action.file.save_as" | "ptnd.action.file.export" => {
            if ctx.has_document {
                Availability::ENABLED
            } else {
                Availability::blocked("ptnd.text.blocked.no_document")
            }
        }
        "ptnd.action.file.place" => Availability::blocked("ptnd.text.blocked.place_image"),
        // Edit.
        "ptnd.action.edit.undo" => {
            if ctx.can_undo {
                Availability::ENABLED
            } else {
                Availability::blocked("ptnd.text.blocked.nothing_to_undo")
            }
        }
        "ptnd.action.edit.redo" => {
            if ctx.can_redo {
                Availability::ENABLED
            } else {
                Availability::blocked("ptnd.text.blocked.nothing_to_redo")
            }
        }
        "ptnd.action.edit.duplicate" | "ptnd.action.edit.delete" => {
            if some_selection() {
                Availability::ENABLED
            } else {
                Availability::blocked("ptnd.text.blocked.select_object")
            }
        }
        "ptnd.action.edit.select_all" => {
            if ctx.has_document {
                Availability::ENABLED
            } else {
                Availability::blocked("ptnd.text.blocked.no_document")
            }
        }
        "ptnd.action.edit.deselect" => {
            if some_selection() {
                Availability::ENABLED
            } else {
                Availability::blocked("ptnd.text.blocked.nothing_selected")
            }
        }
        // Object: group, boolean and align need two or more objects.
        "ptnd.action.object.group"
        | "ptnd.action.object.boolean"
        | "ptnd.action.object.align"
        | "ptnd.action.object.clip_mask.create" => {
            if ctx.selection_count >= 2 {
                Availability::ENABLED
            } else {
                Availability::blocked("ptnd.text.blocked.select_two")
            }
        }
        "ptnd.action.object.distribute" => {
            if ctx.selection_count >= 3 {
                Availability::ENABLED
            } else {
                Availability::blocked("ptnd.text.blocked.distribute_three")
            }
        }
        "ptnd.action.object.ungroup"
        | "ptnd.action.object.arrange.front"
        | "ptnd.action.object.arrange.back"
        | "ptnd.action.object.lock"
        | "ptnd.action.object.hide"
        | "ptnd.action.object.convert_to_curves"
        | "ptnd.action.object.bake_corners"
        | "ptnd.action.object.offset_path"
        | "ptnd.action.object.slice_path"
        | "ptnd.action.object.clip_mask.release" => {
            if some_selection() {
                Availability::ENABLED
            } else {
                Availability::blocked("ptnd.text.blocked.select_object")
            }
        }
        // View: pure view state, always available. Fit needs a surface.
        "ptnd.action.view.zoom_in"
        | "ptnd.action.view.zoom_out"
        | "ptnd.action.view.zoom_100"
        | "ptnd.action.view.zoom_set"
        | "ptnd.action.view.toggle_rulers"
        | "ptnd.action.view.toggle_snapping"
        | "ptnd.action.view.command_palette" => Availability::ENABLED,
        "ptnd.action.view.fit_surface" => {
            if ctx.has_document {
                Availability::ENABLED
            } else {
                Availability::blocked("ptnd.text.blocked.no_surface_to_fit")
            }
        }
        // Tools are modes, not commands: picking one never needs a selection.
        id if id.starts_with("ptnd.tool.") => Availability::ENABLED,
        _ => Availability::blocked("ptnd.text.blocked.no_rule"),
    }
}

/// Resolves one item into its presentation model.
#[must_use]
pub fn resolve_item(item: &MenuItem, ctx: &ActionContext) -> MenuItemModel {
    let entry = surface_entry(item.surface);
    let shortcut = entry.and_then(|e| e.shortcut).map(str::to_string);
    let action_id = dispatch_action_id(item).unwrap_or(item.surface);
    // A blocked registry row wins over the context: the capability itself is
    // missing, so no session state can enable it.
    let (enabled, reason) = match entry.map(|e| e.status) {
        Some(SurfaceStatus::Disabled(reason)) => (false, Some(reason)),
        Some(SurfaceStatus::Absent) => (false, Some("ptnd.text.blocked.not_implemented")),
        _ => {
            let availability = availability(action_id, ctx);
            (availability.enabled, availability.reason)
        }
    };
    MenuItemModel {
        surface_id: item.surface.to_string(),
        action_id: action_id.to_string(),
        action_token: item_token(item),
        label: item.label.to_string(),
        shortcut,
        enabled,
        disabled_reason_id: reason.map(str::to_string),
    }
}

/// Builds a family model for the given context.
fn resolve_family(family: &MenuFamily, ctx: &ActionContext) -> MenuFamilyModel {
    MenuFamilyModel {
        id: family.id.to_string(),
        label: family.label.to_string(),
        nodes: family
            .nodes
            .iter()
            .map(|node| match node {
                MenuNode::Item(item) => MenuNodeModel::Item(resolve_item(item, ctx)),
                MenuNode::Group(group) => MenuNodeModel::Group(MenuGroupModel {
                    id: group.id.to_string(),
                    label: group.label.to_string(),
                    items: group
                        .items
                        .iter()
                        .map(|item| resolve_item(item, ctx))
                        .collect(),
                }),
            })
            .collect(),
    }
}

/// Builds the whole menu bar for the current context.
///
/// Families declared for another persona are absent, not greyed out: they do
/// not apply to this mode at all.
#[must_use]
pub fn menu_bar(ctx: &ActionContext) -> Vec<MenuFamilyModel> {
    MENU_BAR
        .iter()
        .filter(|family| family.visible_in(ctx.persona))
        .map(|family| resolve_family(family, ctx))
        .collect()
}

/// Resolves the zoom levels the shell's zoom box offers, in display order.
///
/// The popup under the readout is not a second registry: it presents the same
/// `ZOOM_LEVELS` items the View submenu does, with the same availability rule.
#[must_use]
pub fn zoom_levels(ctx: &ActionContext) -> Vec<MenuItemModel> {
    ZOOM_LEVELS
        .iter()
        .map(|item| resolve_item(item, ctx))
        .collect()
}

/// Builds the menu bar a persona sees, with the context's own selection state.
#[must_use]
pub fn menu_bar_for(persona: &'static str, ctx: &ActionContext) -> Vec<MenuFamilyModel> {
    menu_bar(&ActionContext { persona, ..*ctx })
}

/// Flattens every **enabled** menu item, in menu order.
///
/// This is the command palette's source (08.2): the palette may only offer
/// what the menu already offers, so the two can never drift apart.
#[must_use]
pub fn command_index(ctx: &ActionContext) -> Vec<MenuItemModel> {
    menu_bar(ctx)
        .iter()
        .flat_map(|family| family.items())
        .filter(|model| model.enabled)
        .cloned()
        .collect()
}

/// Every action id reachable from the menu bar, across **all** personas.
///
/// Reachability is a union over personas: the Vector menu is only on screen in
/// one mode, but the action is still reachable by the user who is in it.
#[must_use]
pub fn reachable_actions() -> Vec<&'static str> {
    let mut ids: Vec<&'static str> = MENU_BAR
        .iter()
        .flat_map(|family| family.items())
        .filter_map(dispatch_action_id)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

/// Action ids reachable in one persona, in menu order.
#[must_use]
pub fn reachable_actions_in(persona: &str) -> Vec<&'static str> {
    MENU_BAR
        .iter()
        .filter(|family| family.visible_in(persona))
        .flat_map(|family| family.items())
        .filter_map(dispatch_action_id)
        .collect()
}

/// Wired `Action` rows in the registry. These are the actions the product
/// claims a user can run today.
#[must_use]
pub fn wired_actions() -> Vec<&'static SurfaceEntry> {
    SURFACES
        .iter()
        .filter(|entry| entry.kind == SurfaceKind::Action && entry.status == SurfaceStatus::Wired)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surfaces::LIVE_ACTIONS;
    use std::collections::{HashMap, HashSet};

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

    fn every_item() -> impl Iterator<Item = &'static MenuItem> {
        MENU_BAR.iter().flat_map(MenuFamily::items)
    }

    #[test]
    fn every_wired_action_is_reachable_from_the_menu() {
        // The handoff's core claim: the actions exist and the user reaches
        // none. This is the test that makes the opposite true, and keeps it
        // true when the registry grows.
        let reachable: HashSet<&str> = reachable_actions().into_iter().collect();
        let mut missing = Vec::new();
        for entry in wired_actions() {
            if !reachable.contains(entry.id) {
                missing.push(entry.id);
            }
        }
        assert!(
            missing.is_empty(),
            "wired actions with no menu item: {missing:?}"
        );
    }

    #[test]
    fn every_menu_item_names_a_registry_action_or_tool_row() {
        for item in every_item() {
            let entry = surface_entry(item.surface)
                .unwrap_or_else(|| panic!("menu item `{}` is not a surface", item.surface));
            assert!(
                matches!(entry.kind, SurfaceKind::Action | SurfaceKind::Tool),
                "menu item `{}` must name an Action or Tool row, got {:?}",
                item.surface,
                entry.kind
            );
            assert!(
                dispatch_action_id(item).is_some(),
                "menu item `{}` dispatches nothing",
                item.surface
            );
        }
    }

    #[test]
    fn unparameterized_items_keep_the_registry_label() {
        // A variant may relabel, but a plain item must not silently disagree
        // with the registry about what the action is called.
        for item in every_item() {
            if item.payload != "null" {
                continue;
            }
            let entry = surface_entry(item.surface).expect("checked above");
            // `file.place` is listed by the Image menu as well as File; both
            // read the registry label, so this stays a single source of truth.
            assert_eq!(
                item.label, entry.label,
                "plain menu item `{}` relabels the registry surface",
                item.surface
            );
        }
    }

    #[test]
    fn labels_and_group_ids_are_namespaced() {
        for family in MENU_BAR {
            assert!(
                family.id.starts_with("ptnd.menu."),
                "family `{}` must follow the 15.G menu grammar",
                family.id
            );
            assert!(family.label.starts_with("ptnd.text."));
            assert!(!family.nodes.is_empty(), "family `{}` is empty", family.id);
            for node in family.nodes {
                match node {
                    MenuNode::Item(item) => assert!(
                        item.label.starts_with("ptnd.text."),
                        "menu item `{}` has non-namespaced label `{}`",
                        item.surface,
                        item.label
                    ),
                    MenuNode::Group(group) => {
                        assert!(
                            group.id.starts_with(&format!("{}.", family.id)),
                            "group `{}` is not scoped to family `{}`",
                            group.id,
                            family.id
                        );
                        assert!(
                            group.label.starts_with("ptnd.text."),
                            "group `{}` has non-namespaced label `{}`",
                            group.id,
                            group.label
                        );
                        assert!(!group.items.is_empty(), "group `{}` has no items", group.id);
                    }
                }
            }
        }
    }

    #[test]
    fn family_and_group_ids_are_unique() {
        let mut families = HashSet::new();
        let mut groups = HashSet::new();
        for family in MENU_BAR {
            assert!(
                families.insert(family.id),
                "duplicate family `{}`",
                family.id
            );
            for group in family.groups() {
                assert!(groups.insert(group.id), "duplicate group `{}`", group.id);
            }
        }
    }

    #[test]
    fn menu_payloads_are_valid_json() {
        for item in every_item() {
            let parsed: Result<Value, _> = serde_json::from_str(item.payload);
            assert!(
                parsed.is_ok(),
                "menu item `{}` has malformed payload `{}`",
                item.surface,
                item.payload
            );
        }
    }

    #[test]
    fn action_tokens_are_unique_and_round_trip() {
        // Two items may share an action id (align, boolean, distribute), so
        // the token — not the action id — is the item's wire identity.
        let mut seen = HashSet::new();
        for item in every_item() {
            let token = item_token(item);
            assert!(seen.insert(token.clone()), "duplicate item token `{token}`");
            let (resolved, payload) =
                item_for_token(&token).unwrap_or_else(|| panic!("token `{token}` lost"));
            assert_eq!(resolved.surface, item.surface);
            assert_eq!(payload, payload_literal(item));
        }
        assert!(item_for_token("ptnd.action.edit.undo#null#extra").is_none());
        assert!(item_for_token("ptnd.action.does.not.exist#null").is_none());
    }

    #[test]
    fn every_menu_action_has_an_availability_rule() {
        // The match in `availability` is the closed-world rule table; a
        // fallback hit means a new wired action arrived without a rule.
        for item in every_item() {
            let entry = surface_entry(item.surface).expect("checked above");
            if matches!(
                entry.status,
                SurfaceStatus::Disabled(_) | SurfaceStatus::Absent
            ) {
                continue;
            }
            let Some(action_id) = dispatch_action_id(item) else {
                continue;
            };
            let availability = availability(action_id, &context());
            assert_ne!(
                availability.reason,
                Some("ptnd.text.blocked.no_rule"),
                "action `{action_id}` has no availability rule"
            );
        }
    }

    #[test]
    fn every_blocked_reason_is_a_text_id() {
        // A blocked item is shown to the user with its reason, so an English
        // literal here would be a string the catalog can never reach. The
        // rule table itself is checked, not just the resolved items, because
        // an unreached arm would still be a latent literal.
        let bare = ActionContext::default();
        for item in every_item() {
            let Some(action_id) = dispatch_action_id(item) else {
                continue;
            };
            let rule = availability(action_id, &bare);
            if let Some(reason) = rule.reason {
                assert!(
                    reason.starts_with("ptnd.text."),
                    "`{action_id}` blocks with the literal `{reason}`"
                );
                assert_ne!(
                    reason, "ptnd.text.blocked.no_rule",
                    "`{action_id}` has no availability rule"
                );
            }
            let model = resolve_item(item, &bare);
            if let Some(reason) = model.disabled_reason_id.as_deref() {
                assert!(
                    reason.starts_with("ptnd.text."),
                    "item `{}` blocks with the literal `{reason}`",
                    item.surface
                );
            }
        }
    }

    #[test]
    fn blocked_registry_rows_are_disabled_with_their_reason() {
        let bar = menu_bar(&context());
        let place = bar
            .iter()
            .flat_map(MenuFamilyModel::items)
            .find(|item| item.surface_id == "ptnd.action.file.place")
            .expect("place is declared in the File family");
        assert!(!place.enabled);
        assert_eq!(
            place.disabled_reason_id.as_deref(),
            Some("ptnd.text.blocked.place_image")
        );
    }

    #[test]
    fn context_decides_availability() {
        let empty = ActionContext::default();
        assert!(!availability("ptnd.action.edit.delete", &empty).enabled);
        assert_eq!(
            availability("ptnd.action.edit.delete", &empty).reason,
            Some("ptnd.text.blocked.select_object")
        );
        assert!(availability("ptnd.action.object.group", &context()).enabled);

        let one = ActionContext {
            has_document: true,
            selection_count: 1,
            ..ActionContext::default()
        };
        assert!(availability("ptnd.action.edit.delete", &one).enabled);
        assert!(!availability("ptnd.action.object.group", &one).enabled);
        assert!(availability("ptnd.action.object.group", &context()).enabled);

        let clean = ActionContext {
            has_document: true,
            is_dirty: false,
            ..ActionContext::default()
        };
        let save = availability("ptnd.action.file.save", &clean);
        assert!(!save.enabled);
        assert_eq!(save.reason, Some("ptnd.text.blocked.no_unsaved_changes"));
        assert!(availability("ptnd.action.file.save", &context()).enabled);
    }

    #[test]
    fn tools_are_never_blocked_by_selection_state() {
        // A tool is a mode, so picking one from a menu must not require a
        // selection the user may not have yet.
        let empty = ActionContext::default();
        for item in every_item() {
            let entry = surface_entry(item.surface).expect("checked above");
            if entry.kind != SurfaceKind::Tool {
                continue;
            }
            let model = resolve_item(item, &empty);
            if entry.status == SurfaceStatus::Wired {
                assert!(
                    model.enabled,
                    "tool `{}` is wired but blocked",
                    item.surface
                );
            }
        }
    }

    #[test]
    fn command_palette_offers_exactly_the_enabled_menu_items() {
        // Palette and menu must not drift: a different set means one of them
        // is lying about what the product can do.
        let ctx = context();
        let enabled: Vec<String> = menu_bar(&ctx)
            .iter()
            .flat_map(MenuFamilyModel::items)
            .filter(|item| item.enabled)
            .map(|item| item.action_token.clone())
            .collect();
        let offered: Vec<String> = command_index(&ctx)
            .iter()
            .map(|item| item.action_token.clone())
            .collect();
        assert_eq!(enabled, offered);

        // A bare context offers no stateful action at all.
        let idle = command_index(&ActionContext {
            has_document: true,
            ..ActionContext::default()
        });
        assert!(idle
            .iter()
            .all(|item| !item.action_id.starts_with("ptnd.action.edit.delete")));
    }

    #[test]
    fn every_reachable_action_resolves_in_the_live_inventory() {
        // A `Tool` row dispatches the action it binds, not its own row id, so
        // the reached id is checked against the live inventory directly rather
        // than through a registry lookup.
        let live: HashSet<&str> = LIVE_ACTIONS.iter().copied().collect();
        for item in every_item() {
            let entry = surface_entry(item.surface).expect("checked above");
            if entry.status != SurfaceStatus::Wired {
                continue;
            }
            let action_id = dispatch_action_id(item).expect("checked above");
            assert!(
                live.contains(action_id),
                "menu exposes `{}` as wired but `{action_id}` is not a live action",
                item.surface
            );
        }
    }

    #[test]
    fn slots_cover_every_wired_action_once_per_payload() {
        // Guards against a duplicated item: one registry row normally owns one
        // menu slot, except for parameterized actions, which own one per
        // variant and must then carry distinct payloads.
        let mut slots: HashMap<&str, Vec<&str>> = HashMap::new();
        for item in every_item() {
            slots.entry(item.surface).or_default().push(item.payload);
        }
        for (surface, payloads) in slots {
            let unique: HashSet<&str> = payloads.iter().copied().collect();
            assert_eq!(
                unique.len(),
                payloads.len(),
                "`{surface}` appears twice with the same payload"
            );
            if payloads.len() > 1 {
                assert!(
                    payloads.iter().all(|payload| *payload != "null"),
                    "`{surface}` is parameterized, so every variant needs a payload"
                );
            }
        }
    }

    #[test]
    fn persona_scoped_families_are_hidden_outside_their_persona() {
        // "The Vector menu is contextual to the vector persona" must be a
        // property of the model, not a UI condition.
        let vector = menu_bar(&ActionContext {
            persona: PERSONA_VECTOR,
            ..context()
        });
        let photo = menu_bar(&ActionContext {
            persona: PERSONA_PHOTO,
            ..context()
        });

        let vector_ids: Vec<&str> = vector.iter().map(|f| f.id.as_str()).collect();
        let photo_ids: Vec<&str> = photo.iter().map(|f| f.id.as_str()).collect();
        assert!(vector_ids.contains(&"ptnd.menu.vector"));
        assert!(!vector_ids.contains(&"ptnd.menu.image"));
        assert!(photo_ids.contains(&"ptnd.menu.image"));
        assert!(!photo_ids.contains(&"ptnd.menu.vector"));

        // Persona-neutral families are visible in both.
        for shared in ["ptnd.menu.file", "ptnd.menu.object", "ptnd.menu.view"] {
            assert!(vector_ids.contains(&shared), "{shared} missing in vector");
            assert!(photo_ids.contains(&shared), "{shared} missing in photo");
        }
    }

    #[test]
    fn every_family_is_reachable_in_at_least_one_persona() {
        for family in MENU_BAR {
            assert!(
                ALL_PERSONAS
                    .iter()
                    .any(|persona| family.visible_in(persona)),
                "family `{}` is scoped to no known persona",
                family.id
            );
        }
    }

    #[test]
    fn persona_scoped_actions_are_reachable_somewhere() {
        // The union is what the product must satisfy: an action may live in
        // one mode only, but it must live in one.
        // The ids here are the ones actually dispatched, which for a tool row
        // is the action the tool binds.
        let reachable: HashSet<&str> = reachable_actions().into_iter().collect();
        for id in [
            "ptnd.action.object.convert_to_curves",
            "ptnd.tool.photo.crop",
            "ptnd.tool.pen",
        ] {
            assert!(reachable.contains(id), "`{id}` is reachable nowhere");
        }
        assert!(reachable_actions_in(PERSONA_VECTOR).contains(&"ptnd.tool.pen"));
        assert!(!reachable_actions_in(PERSONA_PHOTO).contains(&"ptnd.tool.pen"));
        assert!(reachable_actions_in(PERSONA_PHOTO).contains(&"ptnd.tool.photo.crop"));
    }

    #[test]
    fn dispatch_ids_follow_the_registry_binding() {
        // A Tool row dispatches the action it binds, never its own row id:
        // otherwise the menu would send an id the Action lane cannot resolve.
        for item in every_item() {
            let entry = surface_entry(item.surface).expect("checked above");
            let dispatched = dispatch_action_id(item).expect("checked above");
            match entry.kind {
                SurfaceKind::Action => assert_eq!(dispatched, entry.id),
                SurfaceKind::Tool => assert_eq!(Some(dispatched), entry.action),
                other => panic!("unexpected kind {other:?}"),
            }
        }
    }
}
