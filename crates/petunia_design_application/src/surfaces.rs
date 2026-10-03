//! Canonical surface registry seed (15.G, 15.C).
//!
//! Machine-readable list of public interaction surfaces so "the interface
//! works" cannot collapse into a vague claim. Functional Atlas scope still
//! decides promotion; this registry records identity, binding and status.
//!
//! Grammar (15.G):
//! - actions: `ptnd.action.<domain>.<verb>`
//! - tools:   `ptnd.tool.<persona>.<tool>`
//! - panels:  `ptnd.panel.<name>`
//! - dialogs: `ptnd.dialog.<name>` / `ptnd.window.<name>`
//! - controls: `ptnd.surface.<area>.<control>`
//! - menus:   `ptnd.menu.<family>.<item>`

/// What kind of surface an entry describes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SurfaceKind {
    /// Global shell affordance (chrome, docks, canvas host).
    Shell,
    /// Menu bar entry.
    Menu,
    /// Semantic action reachable from menus/shortcuts/palette.
    Action,
    /// Interactive canvas tool.
    Tool,
    /// Dockable panel.
    Panel,
    /// Modal dialog or secondary window.
    Dialog,
    /// Working mode the shell is in. Decides which menus and tools are
    /// reachable, so "this menu is contextual to the persona" is a registry
    /// fact rather than a UI condition.
    Persona,
}

impl SurfaceKind {
    /// Required namespace segment for this kind.
    #[must_use]
    pub fn namespace(self) -> &'static str {
        match self {
            Self::Shell => "surface",
            Self::Menu => "menu",
            Self::Action => "action",
            Self::Tool => "tool",
            Self::Panel => "panel",
            Self::Dialog => "dialog",
            Self::Persona => "persona",
        }
    }
}

/// Delivery scope of the surface (12.8 taxonomy).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SurfaceScope {
    /// Must ship in V1.
    V1Required,
    /// Required for a later milestone.
    MilestoneRequired,
    /// Candidate beyond V1; UX may exist, engine may not.
    PostV1Candidate,
    /// Explicitly out of scope.
    OutOfScope,
}

/// Wiring status. `Disabled` always carries a `ptnd.text.*` TextId naming why
/// (15.F §2), never an English literal: the reason is user-visible, so it must
/// be translatable exactly like a label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceStatus {
    /// Bound to a real semantic action/tool/property and exercised.
    Wired,
    /// Rendered but intentionally inert, with a reason.
    Disabled(&'static str),
    /// Not implemented yet; must not be presented as functional.
    Absent,
}

/// One row of the surface manifest (15.G "Every manifest row MUST include").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceEntry {
    /// Stable SurfaceId.
    pub id: &'static str,
    /// Surface kind, which decides the required namespace segment.
    pub kind: SurfaceKind,
    /// Delivery scope.
    pub scope: SurfaceScope,
    /// Current wiring status.
    pub status: SurfaceStatus,
    /// Localization TextId for the label.
    pub label: &'static str,
    /// Semantic ActionId bound to this surface, when it has one.
    pub action: Option<&'static str>,
    /// Keyboard shortcut in UI form (e.g. `Ctrl+K`), when bound.
    pub shortcut: Option<&'static str>,
}

/// Static registry. Grows as surfaces are wired; the reconciliation test
/// below keeps the grammar and duplicate rules honest.
pub const SURFACES: &[SurfaceEntry] = &[
    // ── Personas (15.G) ────────────────────────────────────────────────────
    // The working mode itself is a surface: menus and tools declare which
    // personas they belong to, and the shell switches between them by id.
    SurfaceEntry {
        id: "ptnd.persona.vector",
        kind: SurfaceKind::Persona,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.persona.vector",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.persona.photo",
        kind: SurfaceKind::Persona,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.persona.photo",
        action: None,
        shortcut: None,
    },
    // ── Shell ───────────────────────────────────────────────────────────────
    SurfaceEntry {
        id: "ptnd.surface.shell.brand",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.shell.brand",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.surface.menu_bar",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.menu.bar",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.surface.shell.persona_switcher",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.shell.persona",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.surface.shell.brand_mark",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.shell.brand_mark",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.surface.shell.undo",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.shell.undo",
        action: Some("ptnd.action.edit.undo"),
        shortcut: Some("Ctrl+Z"),
    },
    SurfaceEntry {
        id: "ptnd.surface.shell.redo",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.shell.redo",
        action: Some("ptnd.action.edit.redo"),
        shortcut: Some("Ctrl+Y"),
    },
    SurfaceEntry {
        id: "ptnd.surface.shell.divider",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.shell.divider",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.surface.shell.zoom_out",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.view.zoom_out",
        action: Some("ptnd.action.view.zoom_out"),
        shortcut: Some("Ctrl+-"),
    },
    SurfaceEntry {
        id: "ptnd.surface.shell.zoom_readout",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.shell.zoom_readout",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.surface.shell.zoom_in",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.view.zoom_in",
        action: Some("ptnd.action.view.zoom_in"),
        shortcut: Some("Ctrl++"),
    },
    SurfaceEntry {
        id: "ptnd.surface.shell.fit",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.view.fit_surface",
        action: Some("ptnd.action.view.fit_surface"),
        shortcut: Some("Ctrl+0"),
    },
    SurfaceEntry {
        id: "ptnd.surface.shell.command_palette",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.shell.palette",
        action: Some("ptnd.action.view.command_palette"),
        shortcut: Some("Ctrl+K"),
    },
    SurfaceEntry {
        id: "ptnd.surface.shell.export",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.shell.export",
        action: Some("ptnd.action.file.export"),
        shortcut: Some("Ctrl+E"),
    },
    SurfaceEntry {
        id: "ptnd.surface.shell.overflow",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.shell.overflow",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.surface.tabs.document_strip",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tabs.strip",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.surface.tabs.document_close",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tabs.close",
        action: Some("ptnd.action.file.close"),
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.surface.tabs.document_dirty",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tabs.dirty",
        action: None,
        shortcut: None,
    },
    // The one control that stays in the tab strip, pinned to its far right.
    SurfaceEntry {
        id: "ptnd.surface.tabs.snapping",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.view.snapping",
        action: Some("ptnd.action.view.toggle_snapping"),
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.surface.context_toolbar",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.shell.context_toolbar",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.surface.tool_rail",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.shell.tool_rail",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.surface.status_bar",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.shell.status_bar",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.surface.dock.left",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::MilestoneRequired,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.dock.left",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.surface.dock.right",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.dock.right",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.surface.dock.bottom",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::MilestoneRequired,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.dock.bottom",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.surface.canvas.viewport",
        kind: SurfaceKind::Shell,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.canvas.viewport",
        action: None,
        shortcut: None,
    },
    // ── File actions ────────────────────────────────────────────────────────
    SurfaceEntry {
        id: "ptnd.action.file.new",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.file.new",
        action: None,
        shortcut: Some("Ctrl+N"),
    },
    SurfaceEntry {
        id: "ptnd.action.file.recover",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.file.recover",
        action: Some("ptnd.action.file.recover"),
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.file.open",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.file.open",
        action: None,
        shortcut: Some("Ctrl+O"),
    },
    SurfaceEntry {
        id: "ptnd.action.file.open_recent",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::PostV1Candidate,
        status: SurfaceStatus::Absent,
        label: "ptnd.text.file.open_recent",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.file.close",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.file.close",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.file.save",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.file.save",
        action: None,
        shortcut: Some("Ctrl+S"),
    },
    SurfaceEntry {
        id: "ptnd.action.file.save_as",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.file.save_as",
        action: None,
        shortcut: Some("Ctrl+Shift+S"),
    },
    SurfaceEntry {
        id: "ptnd.action.file.export",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.file.export",
        action: None,
        shortcut: Some("Ctrl+E"),
    },
    SurfaceEntry {
        id: "ptnd.action.file.place",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.file.place",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.file.quit",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.file.quit",
        action: None,
        shortcut: None,
    },
    // ── Edit actions ────────────────────────────────────────────────────────
    SurfaceEntry {
        id: "ptnd.action.edit.undo",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.edit.undo",
        action: None,
        shortcut: Some("Ctrl+Z"),
    },
    SurfaceEntry {
        id: "ptnd.action.edit.redo",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.edit.redo",
        action: None,
        shortcut: Some("Ctrl+Y"),
    },
    SurfaceEntry {
        id: "ptnd.action.edit.cut",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::PostV1Candidate,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.edit.cut",
        action: None,
        shortcut: Some("Ctrl+X"),
    },
    SurfaceEntry {
        id: "ptnd.action.edit.copy",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::PostV1Candidate,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.edit.copy",
        action: None,
        shortcut: Some("Ctrl+C"),
    },
    SurfaceEntry {
        id: "ptnd.action.edit.paste",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::PostV1Candidate,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.edit.paste",
        action: None,
        shortcut: Some("Ctrl+V"),
    },
    SurfaceEntry {
        id: "ptnd.action.edit.duplicate",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.edit.duplicate",
        action: None,
        shortcut: Some("Ctrl+D"),
    },
    SurfaceEntry {
        id: "ptnd.action.edit.delete",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.edit.delete",
        action: None,
        shortcut: Some("Delete"),
    },
    SurfaceEntry {
        id: "ptnd.action.edit.preferences",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::PostV1Candidate,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.edit.preferences",
        action: None,
        shortcut: Some("Ctrl+,"),
    },
    // ── Selection actions ───────────────────────────────────────────────────
    SurfaceEntry {
        id: "ptnd.action.edit.select_all",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.select.all",
        action: None,
        shortcut: Some("Ctrl+A"),
    },
    SurfaceEntry {
        id: "ptnd.action.edit.deselect",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.select.none",
        action: None,
        shortcut: Some("Ctrl+Shift+A"),
    },
    SurfaceEntry {
        id: "ptnd.action.select.invert",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.select.invert",
        action: None,
        shortcut: None,
    },
    // ── Object actions ──────────────────────────────────────────────────────
    SurfaceEntry {
        id: "ptnd.action.object.group",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.object.group",
        action: None,
        shortcut: Some("Ctrl+G"),
    },
    SurfaceEntry {
        id: "ptnd.action.object.ungroup",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.object.ungroup",
        action: None,
        shortcut: Some("Ctrl+Shift+G"),
    },
    SurfaceEntry {
        id: "ptnd.action.object.arrange.front",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.object.arrange.front",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.object.arrange.back",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.object.arrange.back",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.object.align",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.object.align",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.object.distribute",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.object.distribute",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.object.boolean",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.object.boolean",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.object.convert_to_curves",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.object.convert_to_curves",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.object.bake_corners",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.object.bake_corners",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.object.offset_path",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.object.offset_path",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.object.slice_path",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Disabled("ptnd.text.blocked.slice_path"),
        label: "ptnd.text.object.slice_path",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.object.pixel_mask.create",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::MilestoneRequired,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.object.pixel_mask",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.object.clip_mask.create",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.object.clip_mask",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.object.clip_mask.release",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.object.clip_mask_release",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.object.lock",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.object.lock",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.object.hide",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.object.hide",
        action: None,
        shortcut: None,
    },
    // ── View actions ────────────────────────────────────────────────────────
    SurfaceEntry {
        id: "ptnd.action.view.zoom_in",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.view.zoom_in",
        action: None,
        shortcut: Some("Ctrl++"),
    },
    SurfaceEntry {
        id: "ptnd.action.view.zoom_out",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.view.zoom_out",
        action: None,
        shortcut: Some("Ctrl+-"),
    },
    SurfaceEntry {
        id: "ptnd.action.view.zoom_100",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.view.zoom_100",
        action: None,
        shortcut: Some("Ctrl+1"),
    },
    // One action carries every zoom level in its payload, the way the alignment
    // variants carry their mode: the zoom box and the View submenu offer the
    // same rows without a new command per preset.
    SurfaceEntry {
        id: "ptnd.action.view.zoom_set",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.view.zoom_levels",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.view.command_palette",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.view.command_palette",
        action: None,
        shortcut: Some("Ctrl+K"),
    },
    SurfaceEntry {
        id: "ptnd.action.view.fit_surface",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.view.fit_surface",
        action: None,
        shortcut: Some("Ctrl+0"),
    },
    SurfaceEntry {
        id: "ptnd.action.view.toggle_rulers",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.view.rulers",
        action: None,
        shortcut: Some("Ctrl+R"),
    },
    SurfaceEntry {
        id: "ptnd.action.view.toggle_snapping",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.view.snapping",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.action.view.focus_canvas",
        kind: SurfaceKind::Action,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Absent,
        label: "ptnd.text.view.focus_canvas",
        action: None,
        shortcut: None,
    },
    // ── Design tools ────────────────────────────────────────────────────────
    SurfaceEntry {
        id: "ptnd.tool.vector.perspective",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.perspective",
        action: Some("ptnd.tool.perspective"),
        shortcut: Some("Q"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.move",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.move",
        action: Some("ptnd.tool.select"),
        shortcut: Some("V"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.node",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.node",
        action: Some("ptnd.tool.node"),
        shortcut: Some("A"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.surface",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.surface",
        action: Some("ptnd.tool.artboard"),
        shortcut: Some("H"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.pen",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.pen",
        action: Some("ptnd.tool.pen"),
        shortcut: Some("P"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.pencil",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.pencil",
        action: Some("ptnd.tool.pencil"),
        shortcut: Some("N"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.rectangle",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.rectangle",
        action: Some("ptnd.tool.shape.rectangle"),
        shortcut: Some("M"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.ellipse",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.ellipse",
        action: Some("ptnd.tool.shape.ellipse"),
        shortcut: Some("E"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.polygon",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.polygon",
        action: Some("ptnd.tool.shape.polygon"),
        shortcut: Some("Y"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.star",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.star",
        action: Some("ptnd.tool.shape.star"),
        shortcut: Some("S"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.line",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::PostV1Candidate,
        status: SurfaceStatus::Absent,
        label: "ptnd.text.tool.line",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.artistic_text",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.artistic_text",
        action: Some("ptnd.tool.text.artistic"),
        shortcut: Some("T"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.frame_text",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.frame_text",
        action: Some("ptnd.tool.text.frame"),
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.place_image",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.place_image",
        action: Some("ptnd.action.file.place"),
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.gradient",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.gradient",
        action: Some("ptnd.tool.gradient"),
        shortcut: Some("G"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.transparency",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.transparency",
        action: Some("ptnd.tool.transparency"),
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.eyedropper",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.eyedropper",
        action: Some("ptnd.tool.color_picker"),
        shortcut: Some("I"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.style_picker",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.style_picker",
        action: Some("ptnd.tool.style_picker"),
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.knife",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.knife",
        action: Some("ptnd.tool.knife"),
        shortcut: Some("K"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.corner",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.corner",
        action: Some("ptnd.tool.corner"),
        shortcut: Some("C"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.contour",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.contour",
        action: Some("ptnd.tool.contour"),
        shortcut: Some("O"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.scissors",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.scissors",
        action: Some("ptnd.tool.scissors"),
        shortcut: Some("Shift+K"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.measure",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.measure",
        action: Some("ptnd.tool.measure"),
        shortcut: Some("R"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.hand",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.hand",
        action: Some("ptnd.tool.pan"),
        shortcut: Some("Space"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.zoom",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.zoom",
        action: Some("ptnd.tool.zoom"),
        shortcut: Some("Z"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.shape_builder",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.shape_builder",
        action: Some("ptnd.tool.shape_builder"),
        shortcut: Some("W"),
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.vector_brush",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::PostV1Candidate,
        status: SurfaceStatus::Absent,
        label: "ptnd.text.tool.vector_brush",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.vector_flood_fill",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.vector_flood_fill",
        action: Some("ptnd.tool.vector_flood_fill"),
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.tool.vector.point_transform",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::PostV1Candidate,
        status: SurfaceStatus::Disabled("ptnd.text.blocked.point_transform"),
        label: "ptnd.text.tool.point_transform",
        action: Some("ptnd.tool.point_transform"),
        shortcut: Some("F"),
    },
    // ── Photo tools ─────────────────────────────────────────────────────────
    SurfaceEntry {
        id: "ptnd.tool.photo.move",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.move",
        action: Some("ptnd.tool.select"),
        shortcut: Some("V"),
    },
    SurfaceEntry {
        id: "ptnd.tool.photo.marquee_rect",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.marquee_rect",
        action: Some("ptnd.tool.photo.marquee_rect"),
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.tool.photo.marquee_ellipse",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.marquee_ellipse",
        action: Some("ptnd.tool.photo.marquee_ellipse"),
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.tool.photo.lasso",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.lasso",
        action: Some("ptnd.tool.photo.lasso"),
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.tool.photo.selection_brush",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Disabled("ptnd.text.blocked.raster_post_v1"),
        label: "ptnd.text.tool.selection_brush",
        action: Some("ptnd.tool.photo.selection_brush"),
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.tool.photo.flood_select",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Disabled("ptnd.text.blocked.raster_post_v1"),
        label: "ptnd.text.tool.flood_select",
        action: Some("ptnd.tool.photo.flood_select"),
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.tool.photo.brush",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.brush",
        action: Some("ptnd.tool.photo.brush"),
        shortcut: Some("B"),
    },
    SurfaceEntry {
        id: "ptnd.tool.photo.fill",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.pixel_fill",
        action: Some("ptnd.tool.photo.fill"),
        shortcut: Some("Shift+G"),
    },
    SurfaceEntry {
        id: "ptnd.tool.photo.eraser",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.eraser",
        action: Some("ptnd.tool.photo.eraser"),
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.tool.photo.crop",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.crop",
        action: Some("ptnd.tool.photo.crop"),
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.tool.photo.gradient",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.gradient",
        action: Some("ptnd.tool.photo.gradient"),
        shortcut: Some("G"),
    },
    SurfaceEntry {
        id: "ptnd.tool.photo.eyedropper",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.eyedropper",
        action: Some("ptnd.tool.color_picker"),
        shortcut: Some("I"),
    },
    SurfaceEntry {
        id: "ptnd.tool.photo.hand",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.hand",
        action: Some("ptnd.tool.pan"),
        shortcut: Some("Space"),
    },
    SurfaceEntry {
        id: "ptnd.tool.photo.zoom",
        kind: SurfaceKind::Tool,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.tool.zoom",
        action: Some("ptnd.tool.zoom"),
        shortcut: Some("Z"),
    },
    // ── Panels ──────────────────────────────────────────────────────────────
    SurfaceEntry {
        id: "ptnd.panel.layers",
        kind: SurfaceKind::Panel,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.panel.layers",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.panel.properties",
        kind: SurfaceKind::Panel,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.panel.properties",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.panel.color",
        kind: SurfaceKind::Panel,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.panel.color",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.panel.swatches",
        kind: SurfaceKind::Panel,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.panel.swatches",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.panel.history",
        kind: SurfaceKind::Panel,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.panel.history",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.panel.transform",
        kind: SurfaceKind::Panel,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.panel.transform",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.panel.align",
        kind: SurfaceKind::Panel,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.panel.align",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.panel.appearance",
        kind: SurfaceKind::Panel,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.panel.appearance",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.panel.stroke",
        kind: SurfaceKind::Panel,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.panel.stroke",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.panel.navigator",
        kind: SurfaceKind::Panel,
        scope: SurfaceScope::PostV1Candidate,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.panel.navigator",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.panel.data_merge",
        kind: SurfaceKind::Panel,
        scope: SurfaceScope::PostV1Candidate,
        status: SurfaceStatus::Disabled("ptnd.text.blocked.data_merge_post_v1"),
        label: "ptnd.text.panel.data_merge",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.panel.assets",
        kind: SurfaceKind::Panel,
        scope: SurfaceScope::PostV1Candidate,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.panel.assets",
        action: None,
        shortcut: None,
    },
    // ── Dialogs / windows ───────────────────────────────────────────────────
    SurfaceEntry {
        id: "ptnd.dialog.new_document",
        kind: SurfaceKind::Dialog,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.dialog.new_document",
        action: Some("ptnd.action.file.new"),
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.dialog.export",
        kind: SurfaceKind::Dialog,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.dialog.export",
        action: Some("ptnd.action.file.export"),
        shortcut: Some("Ctrl+E"),
    },
    SurfaceEntry {
        id: "ptnd.window.home",
        kind: SurfaceKind::Dialog,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Absent,
        label: "ptnd.text.window.home",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.dialog.about",
        kind: SurfaceKind::Dialog,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Absent,
        label: "ptnd.text.dialog.about",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.dialog.overwrite_conflict",
        kind: SurfaceKind::Dialog,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.dialog.overwrite",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.window.preferences",
        kind: SurfaceKind::Dialog,
        scope: SurfaceScope::PostV1Candidate,
        status: SurfaceStatus::Absent,
        label: "ptnd.text.window.preferences",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.window.plugin_manager",
        kind: SurfaceKind::Dialog,
        scope: SurfaceScope::PostV1Candidate,
        status: SurfaceStatus::Absent,
        label: "ptnd.text.window.plugins",
        action: None,
        shortcut: None,
    },
    SurfaceEntry {
        id: "ptnd.panel.background_tasks",
        kind: SurfaceKind::Panel,
        scope: SurfaceScope::V1Required,
        status: SurfaceStatus::Wired,
        label: "ptnd.text.panel.jobs",
        action: None,
        shortcut: None,
    },
];

/// Every action identifier that currently resolves to real behavior.
/// Three lanes resolve actions today, all machine-extracted:
/// `DocumentSession::dispatch_action` arms in `session.rs`, the
/// `ToolKind::action_id` table in `tools.rs`, the host-level arms in
/// `gui_bridge.rs` (document lifecycle, which replaces the session), and the
/// Freya host UI action lane. Only a resolution path proves
/// behavior: an `ActionId` constant in `actions.rs` is a declaration, not an
/// implementation. A surface claiming `Wired` MUST bind one of these (15.C/15.G).
pub const LIVE_ACTIONS: &[&str] = &[
    "ptnd.action.edit.copy",
    "ptnd.action.edit.cut",
    "ptnd.action.edit.delete",
    "ptnd.action.edit.deselect",
    "ptnd.action.edit.duplicate",
    "ptnd.action.edit.paste",
    "ptnd.action.edit.preferences",
    "ptnd.action.edit.redo",
    "ptnd.action.edit.select_all",
    "ptnd.action.edit.undo",
    "ptnd.action.file.close",
    "ptnd.action.file.export",
    "ptnd.action.file.new",
    "ptnd.action.file.open",
    "ptnd.action.file.place",
    "ptnd.action.file.quit",
    "ptnd.action.file.recover",
    "ptnd.action.file.save",
    "ptnd.action.file.save_as",
    "ptnd.action.object.align",
    "ptnd.action.object.arrange.back",
    "ptnd.action.object.arrange.front",
    "ptnd.action.object.bake_corners",
    "ptnd.action.object.boolean",
    "ptnd.action.object.clip_mask.create",
    "ptnd.action.object.pixel_mask.create",
    "ptnd.action.object.clip_mask.release",
    "ptnd.action.object.convert_to_curves",
    "ptnd.action.object.distribute",
    "ptnd.action.object.group",
    "ptnd.action.object.hide",
    "ptnd.action.object.lock",
    "ptnd.action.object.offset_path",
    "ptnd.action.object.ungroup",
    "ptnd.action.select.invert",
    "ptnd.action.view.command_palette",
    "ptnd.action.view.fit_surface",
    "ptnd.action.view.toggle_rulers",
    "ptnd.action.view.toggle_snapping",
    "ptnd.action.view.zoom_100",
    "ptnd.action.view.zoom_in",
    "ptnd.action.view.zoom_out",
    "ptnd.action.view.zoom_set",
    "ptnd.tool.artboard",
    "ptnd.tool.color_picker",
    "ptnd.tool.contour",
    "ptnd.tool.corner",
    "ptnd.tool.gradient",
    "ptnd.tool.knife",
    "ptnd.tool.measure",
    "ptnd.tool.node",
    "ptnd.tool.pan",
    "ptnd.tool.pen",
    "ptnd.tool.pencil",
    "ptnd.tool.perspective",
    "ptnd.tool.photo.brush",
    "ptnd.tool.photo.fill",
    "ptnd.tool.photo.crop",
    "ptnd.tool.photo.eraser",
    "ptnd.tool.photo.flood_select",
    "ptnd.tool.photo.gradient",
    "ptnd.tool.photo.lasso",
    "ptnd.tool.photo.marquee_ellipse",
    "ptnd.tool.photo.marquee_rect",
    "ptnd.tool.photo.selection_brush",
    "ptnd.tool.point_transform",
    "ptnd.tool.scissors",
    "ptnd.tool.select",
    "ptnd.tool.shape.ellipse",
    "ptnd.tool.shape.polygon",
    "ptnd.tool.shape.rectangle",
    "ptnd.tool.shape.star",
    "ptnd.tool.shape_builder",
    "ptnd.tool.style_picker",
    "ptnd.tool.text.artistic",
    "ptnd.tool.text.frame",
    "ptnd.tool.transparency",
    "ptnd.tool.vector_flood_fill",
    "ptnd.tool.zoom",
];

/// Canonical persona id for the vector (illustration) working mode.
pub const PERSONA_VECTOR: &str = "ptnd.persona.vector";

/// Canonical persona id for the photo (raster) working mode.
pub const PERSONA_PHOTO: &str = "ptnd.persona.photo";

/// Every persona the shell may switch to, in switcher display order.
#[must_use]
pub fn personas() -> Vec<&'static SurfaceEntry> {
    SURFACES
        .iter()
        .filter(|entry| entry.kind == SurfaceKind::Persona)
        .collect()
}

/// Canonical action ids required by the 15.G grammar that do **not** resolve
/// yet. They are tracked here so the gap is machine-visible instead of being
/// hidden behind a registry claim. Moving an id from this list to
/// [`LIVE_ACTIONS`] is the definition of wiring it.
///
/// Every id here must also exist in [`SURFACES`] with a non-`Wired` status
/// and, when it is merely blocked rather than unimplemented, an explicit
/// [`SurfaceStatus::Disabled`] reason (reconciliation test below).
pub const DECLARED_NOT_LIVE: &[&str] = &["ptnd.action.object.slice_path"];

/// Looks up one surface by stable id.
#[must_use]
pub fn surface(id: &str) -> Option<&'static SurfaceEntry> {
    SURFACES.iter().find(|entry| entry.id == id)
}

/// Surfaces that are still not wired, with their reason when known.
#[must_use]
pub fn unwired() -> Vec<&'static SurfaceEntry> {
    SURFACES
        .iter()
        .filter(|entry| entry.status != SurfaceStatus::Wired)
        .collect()
}

/// Counts of surfaces by wiring status: `(wired, disabled, absent)`.
#[must_use]
pub fn status_counts() -> (usize, usize, usize) {
    let mut wired = 0;
    let mut disabled = 0;
    let mut absent = 0;
    for entry in SURFACES {
        match entry.status {
            SurfaceStatus::Wired => wired += 1,
            SurfaceStatus::Disabled(_) => disabled += 1,
            SurfaceStatus::Absent => absent += 1,
        }
    }
    (wired, disabled, absent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn identifiers_follow_the_15g_grammar() {
        for entry in SURFACES {
            assert!(
                entry.id.starts_with("ptnd."),
                "surface `{}` must be namespaced",
                entry.id
            );
            let expected = format!("ptnd.{}.", entry.kind.namespace());
            // 15.G allows dialogs and secondary windows to share one kind.
            let is_dialog_window =
                entry.kind == SurfaceKind::Dialog && entry.id.starts_with("ptnd.window.");
            // Action entries are the live action inventory itself, so their id
            // is the action id (canonical `ptnd.action.*` once migrated).
            let is_live_action =
                entry.kind == SurfaceKind::Action && LIVE_ACTIONS.contains(&entry.id);
            assert!(
                is_dialog_window || is_live_action || entry.id.starts_with(&expected),
                "surface `{}` must start with `{expected}` for its kind",
                entry.id
            );
        }
    }

    #[test]
    fn identifiers_are_unique() {
        let mut seen = HashSet::new();
        for entry in SURFACES {
            assert!(seen.insert(entry.id), "duplicate surface id `{}`", entry.id);
        }
    }

    #[test]
    fn bound_actions_are_namespaced_and_shortcuts_are_well_formed() {
        for entry in SURFACES {
            if let Some(action) = entry.action {
                assert!(
                    action.starts_with("ptnd."),
                    "surface `{}` binds non-namespaced action `{action}`",
                    entry.id
                );
            }
            if let Some(shortcut) = entry.shortcut {
                assert!(
                    !shortcut.is_empty() && !shortcut.starts_with(' '),
                    "surface `{}` has a malformed shortcut `{shortcut}`",
                    entry.id
                );
            }
            assert!(
                entry.label.starts_with("ptnd.text."),
                "surface `{}` must reference a ptnd.text.* TextId, got `{}`",
                entry.id,
                entry.label
            );
        }
    }

    #[test]
    fn disabled_surfaces_explain_themselves() {
        for entry in unwired() {
            if let SurfaceStatus::Disabled(reason) = entry.status {
                assert!(
                    reason.starts_with("ptnd.text."),
                    "surface `{}` must name a ptnd.text.* reason TextId, got `{reason}`",
                    entry.id
                );
            }
        }
    }

    #[test]
    fn shortcuts_do_not_conflict_within_the_registry() {
        // A conflict is the same shortcut inside one kind reaching *different*
        // actions. Design and Photo personas deliberately share shortcuts for
        // the same semantic action (15.G), and the shell mirrors menu actions.
        // Tools are scoped by persona (`ptnd.tool.<persona>.<tool>`): Design
        // and Photo may bind the same key to their own tool because only one
        // persona is active at a time.
        let mut owner: HashMap<(String, &str), &str> = HashMap::new();
        for entry in SURFACES {
            let (Some(shortcut), Some(action)) = (entry.shortcut, entry.action) else {
                continue;
            };
            let scope = if entry.kind == SurfaceKind::Tool {
                let mut segments = entry.id.split('.');
                let _ = segments.next();
                let _ = segments.next();
                let persona = segments.next().unwrap_or("unknown");
                format!("tool.{persona}")
            } else {
                entry.kind.namespace().to_string()
            };
            let key = (scope.clone(), shortcut);
            match owner.get(&key) {
                Some(existing) if *existing != action => panic!(
                    "shortcut `{shortcut}` reaches both `{existing}` and `{action}` in {scope}"
                ),
                Some(_) => {}
                None => {
                    owner.insert(key, action);
                }
            }
        }
    }

    #[test]
    fn wired_surfaces_bind_a_live_action() {
        // The registry cannot claim a control is wired unless the action it
        // binds actually resolves today (15.F §2 "no fake UI").
        let live: HashSet<&str> = LIVE_ACTIONS.iter().copied().collect();
        for entry in SURFACES {
            if entry.status != SurfaceStatus::Wired {
                continue;
            }
            // For `kind = Action` the id *is* the action, so it must itself
            // resolve. Other kinds are regions or controls: they are Wired
            // either by binding a live action or, with no action at all, by
            // being a rendered region (a toolbar strip has nothing to invoke).
            let bound = if entry.kind == SurfaceKind::Action {
                entry.id
            } else {
                let Some(action) = entry.action else {
                    continue;
                };
                action
            };
            assert!(
                live.contains(bound),
                "surface `{}` claims Wired but binds `{bound}`, which is not a live action",
                entry.id
            );
        }
    }

    #[test]
    fn live_action_inventory_is_namespaced_and_unique() {
        let mut seen = HashSet::new();
        for action in LIVE_ACTIONS {
            assert!(
                action.starts_with("ptnd."),
                "action `{action}` not namespaced"
            );
            assert!(seen.insert(*action), "duplicate live action `{action}`");
        }
    }

    #[test]
    fn declared_not_live_actions_are_tracked_and_not_claimed_live() {
        // A gap must be a row, not a comment: every declared-but-unwired
        // action exists in the registry, is not `Wired`, and is not in the
        // live inventory. This is what stops the two lists from drifting.
        let live: HashSet<&str> = LIVE_ACTIONS.iter().copied().collect();
        for id in DECLARED_NOT_LIVE {
            let entry = surface(id)
                .unwrap_or_else(|| panic!("declared-but-unwired `{id}` is missing from SURFACES"));
            assert!(
                entry.kind == SurfaceKind::Action,
                "declared-but-unwired `{id}` must be an Action surface"
            );
            assert!(
                !live.contains(id),
                "`{id}` is declared as unwired but listed in LIVE_ACTIONS"
            );
            assert!(
                entry.status != SurfaceStatus::Wired,
                "`{id}` is declared as unwired but the registry claims Wired"
            );
        }
    }

    #[test]
    fn blocked_actions_explain_themselves() {
        // Unimplemented is allowed; unexplained is not (15.F §2).
        for id in DECLARED_NOT_LIVE {
            let Some(entry) = surface(id) else { continue };
            if let SurfaceStatus::Disabled(reason) = entry.status {
                assert!(
                    reason.starts_with("ptnd.text."),
                    "`{id}` is disabled without a reason TextId: `{reason}`"
                );
            }
        }
    }

    #[test]
    fn registry_reports_real_coverage() {
        let (wired, disabled, absent) = status_counts();
        assert_eq!(wired + disabled + absent, SURFACES.len());
        // Milestone-1 floor: the registry must track the wired shell at
        // minimum, and every non-wired entry must be visible to the gate.
        assert!(wired >= 40, "expected the wired shell core, got {wired}");
        assert!(surface("ptnd.surface.shell.command_palette").is_some());
        assert!(
            surface("ptnd.panel.layers").is_some_and(|e| e.status == SurfaceStatus::Wired),
            "layers panel must be wired"
        );
        assert!(
            surface("ptnd.panel.color").is_some_and(|e| e.status == SurfaceStatus::Wired),
            "color panel must be wired"
        );
        assert!(
            surface("ptnd.panel.swatches").is_some_and(|e| e.status == SurfaceStatus::Wired),
            "swatches panel must be wired"
        );
        assert!(
            surface("ptnd.panel.background_tasks")
                .is_some_and(|e| e.status == SurfaceStatus::Wired),
            "background tasks panel must be wired"
        );
        assert!(
            surface("ptnd.tool.vector.place_image")
                .is_some_and(|e| e.status == SurfaceStatus::Wired
                    && e.action == Some("ptnd.action.file.place")),
            "place image tool must be wired to file.place"
        );
        assert!(
            surface("ptnd.dialog.overwrite_conflict")
                .is_some_and(|e| e.status == SurfaceStatus::Wired),
            "overwrite conflict dialog must be wired"
        );
        assert!(
            surface("ptnd.surface.dock.left").is_some_and(|e| e.status == SurfaceStatus::Wired),
            "left dock must be wired"
        );
        assert!(
            surface("ptnd.surface.dock.bottom").is_some_and(|e| e.status == SurfaceStatus::Wired),
            "bottom dock must be wired"
        );
        assert!(
            surface("ptnd.panel.navigator").is_some_and(|e| e.status == SurfaceStatus::Wired),
            "navigator panel must be wired"
        );
        assert!(
            surface("ptnd.panel.assets").is_some_and(|e| e.status == SurfaceStatus::Wired),
            "assets panel must be wired"
        );
    }

    #[test]
    fn personas_are_registered_unique_and_labelled() {
        let entries = personas();
        assert!(
            entries.len() >= 2,
            "the shell switches between at least vector and photo"
        );
        let mut seen = HashSet::new();
        for entry in &entries {
            assert!(seen.insert(entry.id), "duplicate persona `{}`", entry.id);
            assert!(
                entry.status == SurfaceStatus::Wired,
                "persona `{}` is offered by the switcher, so it must be wired",
                entry.id
            );
        }
        assert!(entries.iter().any(|e| e.id == PERSONA_VECTOR));
        assert!(entries.iter().any(|e| e.id == PERSONA_PHOTO));
    }

    #[test]
    fn every_blocked_reason_names_a_text_id() {
        // A blocked capability is shown to the user with its reason, so the
        // reason is a user-visible string and must be translatable. An English
        // literal here would be the one string the catalog cannot reach.
        for entry in SURFACES {
            if let SurfaceStatus::Disabled(reason) = entry.status {
                assert!(
                    reason.starts_with("ptnd.text."),
                    "surface `{}` blocks with the literal `{reason}`",
                    entry.id
                );
            }
        }
    }

    #[test]
    fn personas_own_no_menus_or_tools_themselves() {
        // A persona is a mode, not an affordance: it must not smuggle a
        // shortcut or an action of its own into the registry.
        for entry in personas() {
            assert!(
                entry.action.is_none(),
                "persona `{}` must not bind an action",
                entry.id
            );
            assert!(
                entry.shortcut.is_none(),
                "persona `{}` must not bind a shortcut",
                entry.id
            );
        }
    }
}
