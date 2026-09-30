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
use petunia_design_application::interaction::SemanticModifiers;
use petunia_design_application::surfaces::{PERSONA_PHOTO, PERSONA_VECTOR};
use petunia_design_application::tools::ToolKind;
use petunia_design_shell::PetuniaShell;

use crate::theme::{AccentColor, IconStyle, BLOOM};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RailColumns {
    One,
    Two,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolGroupConfig {
    pub id: String,
    pub label_id: &'static str,
    pub tools: Vec<ToolKind>,
    pub visible: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolRailState {
    pub columns: RailColumns,
    pub vector_groups: Vec<ToolGroupConfig>,
    pub photo_groups: Vec<ToolGroupConfig>,
    pub active_by_group: Vec<(String, ToolKind)>,
}

impl ToolRailState {
    #[must_use]
    pub fn groups(&self, photo: bool) -> &[ToolGroupConfig] {
        if photo {
            &self.photo_groups
        } else {
            &self.vector_groups
        }
    }

    pub fn groups_mut(&mut self, photo: bool) -> &mut Vec<ToolGroupConfig> {
        if photo {
            &mut self.photo_groups
        } else {
            &mut self.vector_groups
        }
    }

    #[must_use]
    pub fn active_for_group(&self, group: &ToolGroupConfig) -> Option<ToolKind> {
        self.active_by_group
            .iter()
            .find(|(id, _)| id == &group.id)
            .map(|(_, tool)| *tool)
            .or_else(|| group.tools.first().copied())
    }

    pub fn remember_active(&mut self, group_id: &str, tool: ToolKind) {
        if let Some((_, active)) = self
            .active_by_group
            .iter_mut()
            .find(|(id, _)| id == group_id)
        {
            *active = tool;
        } else {
            self.active_by_group.push((group_id.to_string(), tool));
        }
    }

    pub fn remember_tool(&mut self, photo: bool, tool: ToolKind) {
        if let Some(group) = self
            .groups(photo)
            .iter()
            .find(|group| group.tools.contains(&tool))
        {
            let group_id = group.id.clone();
            self.remember_active(&group_id, tool);
        }
    }

    pub fn move_group(&mut self, photo: bool, index: usize, delta: isize) {
        let groups = self.groups_mut(photo);
        let target = index as isize + delta;
        if target < 0 || target >= groups.len() as isize {
            return;
        }
        groups.swap(index, target as usize);
    }

    pub fn move_tool(&mut self, photo: bool, group_index: usize, tool_index: usize, delta: isize) {
        let groups = self.groups_mut(photo);
        let Some(group) = groups.get_mut(group_index) else {
            return;
        };
        let target = tool_index as isize + delta;
        if target < 0 || target >= group.tools.len() as isize {
            return;
        }
        group.tools.swap(tool_index, target as usize);
    }

    pub fn move_tool_to_group(
        &mut self,
        photo: bool,
        from_group: usize,
        tool_index: usize,
        to_group: usize,
    ) -> bool {
        if from_group == to_group {
            return false;
        }
        let groups = self.groups_mut(photo);
        if from_group >= groups.len()
            || to_group >= groups.len()
            || groups[from_group].tools.len() <= 1
            || tool_index >= groups[from_group].tools.len()
        {
            return false;
        }
        let tool = groups[from_group].tools.remove(tool_index);
        groups[to_group].tools.push(tool);
        true
    }

    pub fn toggle_group(&mut self, photo: bool, index: usize) {
        if let Some(group) = self.groups_mut(photo).get_mut(index) {
            group.visible = !group.visible;
        }
    }

    pub fn toggle_tool(&mut self, photo: bool, group_index: usize, tool: ToolKind) {
        let groups = self.groups_mut(photo);
        let Some(group) = groups.get_mut(group_index) else {
            return;
        };
        if let Some(index) = group.tools.iter().position(|candidate| *candidate == tool) {
            if group.tools.len() > 1 {
                group.tools.remove(index);
            }
        } else {
            group.tools.push(tool);
        }
    }

    pub fn merge_with_previous(&mut self, photo: bool, index: usize) {
        if index == 0 {
            return;
        }
        let groups = self.groups_mut(photo);
        let Some(current) = groups.get(index).cloned() else {
            return;
        };
        groups.remove(index);
        if let Some(previous) = groups.get_mut(index - 1) {
            previous.tools.extend(current.tools);
        }
    }
}

#[must_use]
pub fn default_tool_rail() -> ToolRailState {
    let group = |id: &str, label_id: &'static str, tools: &[ToolKind]| ToolGroupConfig {
        id: id.to_string(),
        label_id,
        tools: tools.to_vec(),
        visible: true,
    };
    ToolRailState {
        columns: RailColumns::One,
        vector_groups: vec![
            group(
                "selection",
                "ptnd.text.tool_group.selection",
                &[ToolKind::Select, ToolKind::Node, ToolKind::Perspective],
            ),
            group("pen", "ptnd.text.tool_group.pen", &[ToolKind::Pen]),
            group("pencil", "ptnd.text.tool_group.pencil", &[ToolKind::Pencil]),
            group(
                "shapes",
                "ptnd.text.tool_group.shapes",
                &[
                    ToolKind::Rectangle,
                    ToolKind::Ellipse,
                    ToolKind::Polygon,
                    ToolKind::Star,
                    ToolKind::VectorFloodFill,
                ],
            ),
            group(
                "text",
                "ptnd.text.tool_group.text",
                &[ToolKind::ArtisticText, ToolKind::FrameText],
            ),
            group(
                "modify",
                "ptnd.text.tool_group.modify",
                &[ToolKind::ShapeBuilder, ToolKind::Corner, ToolKind::Contour],
            ),
            group(
                "cut",
                "ptnd.text.tool_group.cut",
                &[ToolKind::Knife, ToolKind::Scissors],
            ),
            group(
                "gradient-transparency",
                "ptnd.text.tool_group.gradient_transparency",
                &[ToolKind::Gradient, ToolKind::Transparency],
            ),
            group(
                "pick",
                "ptnd.text.tool_group.pick",
                &[ToolKind::ColorPicker, ToolKind::StylePicker],
            ),
            group(
                "artboard",
                "ptnd.text.tool_group.artboard",
                &[ToolKind::Artboard],
            ),
            group(
                "measure",
                "ptnd.text.tool_group.measure",
                &[ToolKind::Measure],
            ),
            group("zoom", "ptnd.text.tool_group.zoom", &[ToolKind::Zoom]),
            group("hand", "ptnd.text.tool_group.hand", &[ToolKind::Hand]),
        ],
        photo_groups: vec![
            group(
                "photo-selection",
                "ptnd.text.tool_group.photo_selection",
                &[
                    ToolKind::MarqueeRect,
                    ToolKind::MarqueeEllipse,
                    ToolKind::Lasso,
                    ToolKind::SelectionBrush,
                    ToolKind::FloodSelect,
                ],
            ),
            group(
                "photo-content",
                "ptnd.text.tool_group.photo_content",
                &[ToolKind::PhotoGradient],
            ),
            group("photo-crop", "ptnd.text.tool_group.crop", &[ToolKind::Crop]),
            group("shared", "ptnd.text.tool_group.shared", &[ToolKind::Select]),
            group("photo-zoom", "ptnd.text.tool_group.zoom", &[ToolKind::Zoom]),
            group("photo-hand", "ptnd.text.tool_group.hand", &[ToolKind::Hand]),
        ],
        active_by_group: Vec::new(),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HoverTarget {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub shortcut: String,
    pub x: f64,
    pub y: f64,
}

/// Shared shell state plus the overlay state only the screen keeps.
///
/// `Clone` shares the same underlying states; `PartialEq` compares by
/// identity so the diffing pass treats one shared instance as equal.
#[derive(Clone)]
pub struct UiShell {
    pub shell: State<PetuniaShell>,
    pub open_family: State<Option<String>>,
    pub palette_open: State<bool>,
    pub palette_query: State<String>,
    pub customize_open: State<bool>,
    pub accent: State<AccentColor>,
    pub icon_style: State<IconStyle>,
    /// Hovered control: `(id, text)` shown in the status bar while hovered.
    /// Tooltips live here instead of in `TooltipContainer`, whose `Attached`
    /// wrapper takes the button out of the layout flow and displaces it.
    pub hovered: State<Option<HoverTarget>>,
    pub modifiers: State<SemanticModifiers>,
    pub tool_rail: State<ToolRailState>,
    pub active_tool: State<ToolKind>,
    pub persona: State<String>,
    pub temporary_tool: State<Option<ToolKind>>,
    pub suspended_tool: State<Option<ToolKind>>,
    pub dock_tab: State<usize>,
    pub text_edit_content: State<String>,
    pub new_doc_open: State<bool>,
    pub export_open: State<bool>,
    pub confirm_close_open: State<bool>,
    /// Tab the close confirmation applies to. `None` means "quit": the
    /// confirmation is about every open document, not one tab.
    pub pending_close: State<Option<usize>>,
    pub offset_prompt_open: State<bool>,
    pub overwrite_conflict_open: State<bool>,
    pub overwrite_conflict_path: State<String>,
    pub dock_width: State<f32>,
    pub soft_proof: State<bool>,
    pub channel_view: State<usize>,
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
            && self.modifiers == other.modifiers
            && self.tool_rail == other.tool_rail
            && self.active_tool == other.active_tool
            && self.persona == other.persona
            && self.temporary_tool == other.temporary_tool
            && self.suspended_tool == other.suspended_tool
            && self.dock_tab == other.dock_tab
            && self.text_edit_content == other.text_edit_content
            && self.new_doc_open == other.new_doc_open
            && self.export_open == other.export_open
            && self.confirm_close_open == other.confirm_close_open
            && self.pending_close == other.pending_close
            && self.offset_prompt_open == other.offset_prompt_open
            && self.overwrite_conflict_open == other.overwrite_conflict_open
            && self.overwrite_conflict_path == other.overwrite_conflict_path
            && self.dock_width == other.dock_width
            && self.soft_proof == other.soft_proof
            && self.channel_view == other.channel_view
    }
}

impl UiShell {
    #[must_use]
    // One `State` handle per shell concern, fanned out once from the component
    // scope; grouping them would only move the arity into a struct literal at
    // each construction site, so the arity is allowed here.
    // (clippy::too_many_arguments: Freya state-fan-out boundary)
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        shell: State<PetuniaShell>,
        open_family: State<Option<String>>,
        palette_open: State<bool>,
        palette_query: State<String>,
        customize_open: State<bool>,
        accent: State<AccentColor>,
        icon_style: State<IconStyle>,
        hovered: State<Option<HoverTarget>>,
        modifiers: State<SemanticModifiers>,
        tool_rail: State<ToolRailState>,
        active_tool: State<ToolKind>,
        persona: State<String>,
        temporary_tool: State<Option<ToolKind>>,
        suspended_tool: State<Option<ToolKind>>,
        dock_tab: State<usize>,
        text_edit_content: State<String>,
        new_doc_open: State<bool>,
        export_open: State<bool>,
        confirm_close_open: State<bool>,
        pending_close: State<Option<usize>>,
        offset_prompt_open: State<bool>,
        overwrite_conflict_open: State<bool>,
        overwrite_conflict_path: State<String>,
        dock_width: State<f32>,
        soft_proof: State<bool>,
        channel_view: State<usize>,
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
            modifiers,
            tool_rail,
            active_tool,
            persona,
            temporary_tool,
            suspended_tool,
            dock_tab,
            text_edit_content,
            new_doc_open,
            export_open,
            confirm_close_open,
            pending_close,
            offset_prompt_open,
            overwrite_conflict_open,
            overwrite_conflict_path,
            dock_width,
            soft_proof,
            channel_view,
        }
    }

    pub fn activate_tool(&self, tool: ToolKind) {
        let mut active_tool = self.active_tool;
        let mut temporary_tool = self.temporary_tool;
        let mut suspended_tool = self.suspended_tool;
        let mut shell = self.shell;
        temporary_tool.set(None);
        suspended_tool.set(None);
        active_tool.set(tool);
        shell.write().set_active_tool(tool);
    }

    pub fn set_persona(&self, persona: String) {
        let mut persona_state = self.persona;
        let mut active_tool = self.active_tool;
        let mut shell = self.shell;
        let photo = persona == PERSONA_PHOTO;
        let current_tool = *active_tool.read();
        let next_tool = if self
            .tool_rail
            .peek()
            .groups(photo)
            .iter()
            .any(|group| group.tools.contains(&current_tool))
        {
            current_tool
        } else {
            ToolKind::Select
        };
        persona_state.set(persona.clone());
        active_tool.set(next_tool);
        let mut shell_state = shell.write();
        let _ = shell_state.bridge.set_persona(&persona);
        shell_state.set_active_tool(next_tool);
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
        let modifiers = use_state(SemanticModifiers::default);
        let tool_rail = use_state(default_tool_rail);
        let active_tool = use_state(|| ToolKind::Select);
        let persona = use_state(|| PERSONA_VECTOR.to_string());
        let temporary_tool = use_state(|| None);
        let suspended_tool = use_state(|| None);
        let dock_tab = use_state(|| 0usize);
        let text_edit_content = use_state(String::new);
        let new_doc_open = use_state(|| false);
        let export_open = use_state(|| false);
        let confirm_close_open = use_state(|| false);
        let pending_close = use_state(|| None);
        let offset_prompt_open = use_state(|| false);
        let overwrite_conflict_open = use_state(|| false);
        let overwrite_conflict_path = use_state(|| "export.png".to_string());
        let dock_width = use_state(|| 320.0f32);
        let soft_proof = use_state(|| false);
        let channel_view = use_state(|| 0usize);
        Self::new(
            shell,
            open_family,
            palette_open,
            palette_query,
            customize_open,
            accent,
            icon_style,
            hovered,
            modifiers,
            tool_rail,
            active_tool,
            persona,
            temporary_tool,
            suspended_tool,
            dock_tab,
            text_edit_content,
            new_doc_open,
            export_open,
            confirm_close_open,
            pending_close,
            offset_prompt_open,
            overwrite_conflict_open,
            overwrite_conflict_path,
            dock_width,
            soft_proof,
            channel_view,
        )
    }
}
