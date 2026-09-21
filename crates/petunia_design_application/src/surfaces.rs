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

/// Wiring status. `Disabled` always carries a user-visible reason (15.F §2).
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
    // ── Shell ───────────────────────────────────────────────────────────────
    SurfaceEntry { id: "ptnd.surface.shell.brand", kind: SurfaceKind::Shell, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.shell.brand", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.surface.shell.persona_persona", kind: SurfaceKind::Shell, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.shell.persona", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.surface.shell.undo", kind: SurfaceKind::Shell, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.shell.undo", action: Some("ptnd.action.edit.undo"), shortcut: Some("Ctrl+Z") },
    SurfaceEntry { id: "ptnd.surface.shell.redo", kind: SurfaceKind::Shell, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.shell.redo", action: Some("ptnd.action.edit.redo"), shortcut: Some("Ctrl+Y") },
    SurfaceEntry { id: "ptnd.surface.shell.command_palette", kind: SurfaceKind::Shell, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.shell.palette", action: Some("ptnd.action.view.command_palette"), shortcut: Some("Ctrl+K") },
    SurfaceEntry { id: "ptnd.surface.shell.export", kind: SurfaceKind::Shell, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.shell.export", action: Some("ptnd.action.file.export"), shortcut: Some("Ctrl+E") },
    SurfaceEntry { id: "ptnd.surface.shell.overflow", kind: SurfaceKind::Shell, scope: SurfaceScope::V1Required, status: SurfaceStatus::Absent, label: "ptnd.text.shell.overflow", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.surface.tabs.document_strip", kind: SurfaceKind::Shell, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tabs.strip", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.surface.tabs.document_close", kind: SurfaceKind::Shell, scope: SurfaceScope::V1Required, status: SurfaceStatus::Absent, label: "ptnd.text.tabs.close", action: Some("ptnd.action.file.close"), shortcut: None },
    SurfaceEntry { id: "ptnd.surface.tabs.document_dirty", kind: SurfaceKind::Shell, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tabs.dirty", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.surface.context_toolbar", kind: SurfaceKind::Shell, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.shell.context_toolbar", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.surface.tool_rail", kind: SurfaceKind::Shell, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.shell.tool_rail", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.surface.status_bar", kind: SurfaceKind::Shell, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.shell.status_bar", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.surface.dock.left", kind: SurfaceKind::Shell, scope: SurfaceScope::MilestoneRequired, status: SurfaceStatus::Absent, label: "ptnd.text.dock.left", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.surface.dock.right", kind: SurfaceKind::Shell, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.dock.right", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.surface.dock.bottom", kind: SurfaceKind::Shell, scope: SurfaceScope::MilestoneRequired, status: SurfaceStatus::Absent, label: "ptnd.text.dock.bottom", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.surface.canvas.viewport", kind: SurfaceKind::Shell, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.canvas.viewport", action: None, shortcut: None },

    // ── File actions ────────────────────────────────────────────────────────
    SurfaceEntry { id: "ptnd.action.file.new", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.file.new", action: Some("ptnd.action.file.new"), shortcut: Some("Ctrl+N") },
    SurfaceEntry { id: "ptnd.action.file.open", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.file.open", action: Some("ptnd.action.file.open"), shortcut: Some("Ctrl+O") },
    SurfaceEntry { id: "ptnd.action.file.open_recent", kind: SurfaceKind::Action, scope: SurfaceScope::PostV1Candidate, status: SurfaceStatus::Absent, label: "ptnd.text.file.open_recent", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.action.file.close", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Absent, label: "ptnd.text.file.close", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.action.file.save", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.file.save", action: Some("ptnd.action.file.save"), shortcut: Some("Ctrl+S") },
    SurfaceEntry { id: "ptnd.action.file.save_as", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.file.save_as", action: Some("ptnd.action.file.save_as"), shortcut: Some("Ctrl+Shift+S") },
    SurfaceEntry { id: "ptnd.action.file.export", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.file.export", action: Some("ptnd.action.file.export"), shortcut: Some("Ctrl+E") },
    SurfaceEntry { id: "ptnd.action.file.quit", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Absent, label: "ptnd.text.file.quit", action: None, shortcut: None },

    // ── Edit actions ────────────────────────────────────────────────────────
    SurfaceEntry { id: "ptnd.action.edit.undo", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.edit.undo", action: Some("ptnd.action.edit.undo"), shortcut: Some("Ctrl+Z") },
    SurfaceEntry { id: "ptnd.action.edit.redo", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.edit.redo", action: Some("ptnd.action.edit.redo"), shortcut: Some("Ctrl+Y") },
    SurfaceEntry { id: "ptnd.action.edit.cut", kind: SurfaceKind::Action, scope: SurfaceScope::PostV1Candidate, status: SurfaceStatus::Absent, label: "ptnd.text.edit.cut", action: None, shortcut: Some("Ctrl+X") },
    SurfaceEntry { id: "ptnd.action.edit.copy", kind: SurfaceKind::Action, scope: SurfaceScope::PostV1Candidate, status: SurfaceStatus::Absent, label: "ptnd.text.edit.copy", action: None, shortcut: Some("Ctrl+C") },
    SurfaceEntry { id: "ptnd.action.edit.paste", kind: SurfaceKind::Action, scope: SurfaceScope::PostV1Candidate, status: SurfaceStatus::Absent, label: "ptnd.text.edit.paste", action: None, shortcut: Some("Ctrl+V") },
    SurfaceEntry { id: "ptnd.action.edit.duplicate", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.edit.duplicate", action: Some("ptnd.action.edit.duplicate"), shortcut: Some("Ctrl+D") },
    SurfaceEntry { id: "ptnd.action.edit.delete", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.edit.delete", action: Some("ptnd.action.edit.delete"), shortcut: Some("Delete") },
    SurfaceEntry { id: "ptnd.action.edit.preferences", kind: SurfaceKind::Action, scope: SurfaceScope::PostV1Candidate, status: SurfaceStatus::Absent, label: "ptnd.text.edit.preferences", action: None, shortcut: None },

    // ── Selection actions ───────────────────────────────────────────────────
    SurfaceEntry { id: "ptnd.action.select.all", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.select.all", action: Some("ptnd.action.select.all"), shortcut: Some("Ctrl+A") },
    SurfaceEntry { id: "ptnd.action.select.none", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.select.none", action: Some("ptnd.action.select.none"), shortcut: Some("Ctrl+Shift+A") },
    SurfaceEntry { id: "ptnd.action.select.invert", kind: SurfaceKind::Action, scope: SurfaceScope::PostV1Candidate, status: SurfaceStatus::Absent, label: "ptnd.text.select.invert", action: None, shortcut: None },

    // ── Object actions ──────────────────────────────────────────────────────
    SurfaceEntry { id: "ptnd.action.object.group", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.object.group", action: Some("ptnd.action.object.group"), shortcut: Some("Ctrl+G") },
    SurfaceEntry { id: "ptnd.action.object.ungroup", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.object.ungroup", action: Some("ptnd.action.object.ungroup"), shortcut: Some("Ctrl+Shift+G") },
    SurfaceEntry { id: "ptnd.action.object.arrange_front", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.object.arrange_front", action: Some("ptnd.action.object.arrange.front"), shortcut: None },
    SurfaceEntry { id: "ptnd.action.object.arrange_back", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.object.arrange_back", action: Some("ptnd.action.object.arrange.back"), shortcut: None },
    SurfaceEntry { id: "ptnd.action.object.align", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.object.align", action: Some("ptnd.action.object.align"), shortcut: None },
    SurfaceEntry { id: "ptnd.action.object.distribute", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.object.distribute", action: Some("ptnd.action.object.distribute"), shortcut: None },
    SurfaceEntry { id: "ptnd.action.object.boolean", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.object.boolean", action: Some("ptnd.action.object.boolean"), shortcut: None },
    SurfaceEntry { id: "ptnd.action.object.convert_to_curves", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.object.convert_to_curves", action: Some("ptnd.action.object.convert_to_curves"), shortcut: None },
    SurfaceEntry { id: "ptnd.action.object.bake_corners", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.object.bake_corners", action: Some("ptnd.action.object.bake_corners"), shortcut: None },
    SurfaceEntry { id: "ptnd.action.object.offset_path", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.object.offset_path", action: Some("ptnd.action.object.offset_path"), shortcut: None },
    SurfaceEntry { id: "ptnd.action.object.slice_path", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.object.slice_path", action: Some("ptnd.action.object.slice_path"), shortcut: None },
    SurfaceEntry { id: "ptnd.action.object.lock", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.object.lock", action: Some("ptnd.action.object.lock"), shortcut: None },
    SurfaceEntry { id: "ptnd.action.object.hide", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.object.hide", action: Some("ptnd.action.object.hide"), shortcut: None },

    // ── View actions ────────────────────────────────────────────────────────
    SurfaceEntry { id: "ptnd.action.view.zoom_in", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.view.zoom_in", action: Some("ptnd.action.view.zoom_in"), shortcut: Some("Ctrl++") },
    SurfaceEntry { id: "ptnd.action.view.zoom_out", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.view.zoom_out", action: Some("ptnd.action.view.zoom_out"), shortcut: Some("Ctrl+-") },
    SurfaceEntry { id: "ptnd.action.view.zoom_100", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.view.zoom_100", action: Some("ptnd.action.view.zoom_100"), shortcut: Some("Ctrl+1") },
    SurfaceEntry { id: "ptnd.action.view.fit_surface", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.view.fit_surface", action: Some("ptnd.action.view.fit_surface"), shortcut: Some("Ctrl+0") },
    SurfaceEntry { id: "ptnd.action.view.toggle_rulers", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.view.rulers", action: Some("ptnd.action.view.toggle_rulers"), shortcut: Some("Ctrl+R") },
    SurfaceEntry { id: "ptnd.action.view.toggle_snapping", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.view.snapping", action: Some("ptnd.action.view.toggle_snapping"), shortcut: None },
    SurfaceEntry { id: "ptnd.action.view.focus_canvas", kind: SurfaceKind::Action, scope: SurfaceScope::V1Required, status: SurfaceStatus::Absent, label: "ptnd.text.view.focus_canvas", action: None, shortcut: None },

    // ── Design tools ────────────────────────────────────────────────────────
    SurfaceEntry { id: "ptnd.tool.design.move", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.move", action: Some("ptnd.tool.select"), shortcut: Some("V") },
    SurfaceEntry { id: "ptnd.tool.design.node", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.node", action: Some("ptnd.tool.node"), shortcut: Some("A") },
    SurfaceEntry { id: "ptnd.tool.design.surface", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.surface", action: Some("ptnd.tool.artboard"), shortcut: Some("H") },
    SurfaceEntry { id: "ptnd.tool.design.pen", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.pen", action: Some("ptnd.tool.pen"), shortcut: Some("P") },
    SurfaceEntry { id: "ptnd.tool.design.pencil", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.pencil", action: Some("ptnd.tool.pencil"), shortcut: Some("N") },
    SurfaceEntry { id: "ptnd.tool.design.rectangle", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.rectangle", action: Some("ptnd.tool.rectangle"), shortcut: Some("M") },
    SurfaceEntry { id: "ptnd.tool.design.ellipse", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.ellipse", action: Some("ptnd.tool.ellipse"), shortcut: Some("E") },
    SurfaceEntry { id: "ptnd.tool.design.polygon", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.polygon", action: Some("ptnd.tool.polygon"), shortcut: Some("Y") },
    SurfaceEntry { id: "ptnd.tool.design.star", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.star", action: Some("ptnd.tool.star"), shortcut: Some("S") },
    SurfaceEntry { id: "ptnd.tool.design.line", kind: SurfaceKind::Tool, scope: SurfaceScope::PostV1Candidate, status: SurfaceStatus::Absent, label: "ptnd.text.tool.line", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.tool.design.artistic_text", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.artistic_text", action: Some("ptnd.tool.artistic_text"), shortcut: Some("T") },
    SurfaceEntry { id: "ptnd.tool.design.frame_text", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.frame_text", action: Some("ptnd.tool.frame_text"), shortcut: None },
    SurfaceEntry { id: "ptnd.tool.design.place_image", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.place_image", action: Some("ptnd.action.file.place"), shortcut: None },
    SurfaceEntry { id: "ptnd.tool.design.gradient", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.gradient", action: Some("ptnd.tool.gradient"), shortcut: Some("G") },
    SurfaceEntry { id: "ptnd.tool.design.transparency", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.transparency", action: Some("ptnd.tool.transparency"), shortcut: None },
    SurfaceEntry { id: "ptnd.tool.design.eyedropper", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.eyedropper", action: Some("ptnd.tool.color_picker"), shortcut: Some("I") },
    SurfaceEntry { id: "ptnd.tool.design.knife", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.knife", action: Some("ptnd.tool.knife"), shortcut: Some("K") },
    SurfaceEntry { id: "ptnd.tool.design.corner", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.corner", action: Some("ptnd.tool.corner"), shortcut: Some("C") },
    SurfaceEntry { id: "ptnd.tool.design.contour", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.contour", action: Some("ptnd.tool.contour"), shortcut: None },
    SurfaceEntry { id: "ptnd.tool.design.hand", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.hand", action: Some("ptnd.tool.pan"), shortcut: Some("Space") },
    SurfaceEntry { id: "ptnd.tool.design.zoom", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.zoom", action: Some("ptnd.tool.zoom"), shortcut: Some("Z") },
    SurfaceEntry { id: "ptnd.tool.design.shape_builder", kind: SurfaceKind::Tool, scope: SurfaceScope::PostV1Candidate, status: SurfaceStatus::Disabled("shape builder arrives after V1 (10.3 scope)"), label: "ptnd.text.tool.shape_builder", action: Some("ptnd.tool.shape_builder"), shortcut: Some("W") },
    SurfaceEntry { id: "ptnd.tool.design.vector_brush", kind: SurfaceKind::Tool, scope: SurfaceScope::PostV1Candidate, status: SurfaceStatus::Absent, label: "ptnd.text.tool.vector_brush", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.tool.design.point_transform", kind: SurfaceKind::Tool, scope: SurfaceScope::PostV1Candidate, status: SurfaceStatus::Disabled("folded into the Transform HUD (08.33)"), label: "ptnd.text.tool.point_transform", action: Some("ptnd.tool.point_transform"), shortcut: Some("F") },

    // ── Photo tools ─────────────────────────────────────────────────────────
    SurfaceEntry { id: "ptnd.tool.photo.move", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.move", action: Some("ptnd.tool.select"), shortcut: Some("V") },
    SurfaceEntry { id: "ptnd.tool.photo.brush", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Disabled("raster pixel layers are Post-V1 (10.9/10.10)"), label: "ptnd.text.tool.brush", action: Some("ptnd.tool.photo.brush"), shortcut: Some("B") },
    SurfaceEntry { id: "ptnd.tool.photo.eraser", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Disabled("raster pixel layers are Post-V1 (10.9/10.10)"), label: "ptnd.text.tool.eraser", action: Some("ptnd.tool.photo.eraser"), shortcut: None },
    SurfaceEntry { id: "ptnd.tool.photo.crop", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.crop", action: Some("ptnd.tool.photo.crop"), shortcut: None },
    SurfaceEntry { id: "ptnd.tool.photo.gradient", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.gradient", action: Some("ptnd.tool.photo.gradient"), shortcut: Some("G") },
    SurfaceEntry { id: "ptnd.tool.photo.eyedropper", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.eyedropper", action: Some("ptnd.tool.color_picker"), shortcut: Some("I") },
    SurfaceEntry { id: "ptnd.tool.photo.hand", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.hand", action: Some("ptnd.tool.pan"), shortcut: Some("Space") },
    SurfaceEntry { id: "ptnd.tool.photo.zoom", kind: SurfaceKind::Tool, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.tool.zoom", action: Some("ptnd.tool.zoom"), shortcut: Some("Z") },

    // ── Panels ──────────────────────────────────────────────────────────────
    SurfaceEntry { id: "ptnd.panel.layers", kind: SurfaceKind::Panel, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.panel.layers", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.panel.properties", kind: SurfaceKind::Panel, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.panel.properties", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.panel.color", kind: SurfaceKind::Panel, scope: SurfaceScope::V1Required, status: SurfaceStatus::Absent, label: "ptnd.text.panel.color", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.panel.swatches", kind: SurfaceKind::Panel, scope: SurfaceScope::V1Required, status: SurfaceStatus::Absent, label: "ptnd.text.panel.swatches", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.panel.history", kind: SurfaceKind::Panel, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.panel.history", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.panel.transform", kind: SurfaceKind::Panel, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.panel.transform", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.panel.align", kind: SurfaceKind::Panel, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.panel.align", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.panel.appearance", kind: SurfaceKind::Panel, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.panel.appearance", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.panel.stroke", kind: SurfaceKind::Panel, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.panel.stroke", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.panel.navigator", kind: SurfaceKind::Panel, scope: SurfaceScope::PostV1Candidate, status: SurfaceStatus::Absent, label: "ptnd.text.panel.navigator", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.panel.data_merge", kind: SurfaceKind::Panel, scope: SurfaceScope::PostV1Candidate, status: SurfaceStatus::Disabled("data merge UI is Post-V1; engine exists"), label: "ptnd.text.panel.data_merge", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.panel.assets", kind: SurfaceKind::Panel, scope: SurfaceScope::PostV1Candidate, status: SurfaceStatus::Absent, label: "ptnd.text.panel.assets", action: None, shortcut: None },

    // ── Dialogs / windows ───────────────────────────────────────────────────
    SurfaceEntry { id: "ptnd.dialog.new_document", kind: SurfaceKind::Dialog, scope: SurfaceScope::V1Required, status: SurfaceStatus::Absent, label: "ptnd.text.dialog.new_document", action: Some("ptnd.action.file.new"), shortcut: None },
    SurfaceEntry { id: "ptnd.dialog.export", kind: SurfaceKind::Dialog, scope: SurfaceScope::V1Required, status: SurfaceStatus::Wired, label: "ptnd.text.dialog.export", action: Some("ptnd.action.file.export"), shortcut: Some("Ctrl+E") },
    SurfaceEntry { id: "ptnd.window.home", kind: SurfaceKind::Dialog, scope: SurfaceScope::V1Required, status: SurfaceStatus::Absent, label: "ptnd.text.window.home", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.dialog.about", kind: SurfaceKind::Dialog, scope: SurfaceScope::V1Required, status: SurfaceStatus::Absent, label: "ptnd.text.dialog.about", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.dialog.overwrite_conflict", kind: SurfaceKind::Dialog, scope: SurfaceScope::V1Required, status: SurfaceStatus::Absent, label: "ptnd.text.dialog.overwrite", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.window.preferences", kind: SurfaceKind::Dialog, scope: SurfaceScope::PostV1Candidate, status: SurfaceStatus::Absent, label: "ptnd.text.window.preferences", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.window.plugin_manager", kind: SurfaceKind::Dialog, scope: SurfaceScope::PostV1Candidate, status: SurfaceStatus::Absent, label: "ptnd.text.window.plugins", action: None, shortcut: None },
    SurfaceEntry { id: "ptnd.panel.background_tasks", kind: SurfaceKind::Panel, scope: SurfaceScope::V1Required, status: SurfaceStatus::Absent, label: "ptnd.text.panel.jobs", action: None, shortcut: None },
];

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
            let is_dialog_window = entry.kind == SurfaceKind::Dialog
                && entry.id.starts_with("ptnd.window.");
            assert!(
                is_dialog_window || entry.id.starts_with(&expected),
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
                    reason.len() > 12,
                    "surface `{}` disabled reason too vague: `{reason}`",
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
    }
}
