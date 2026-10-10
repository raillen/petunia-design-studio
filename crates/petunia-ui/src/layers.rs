//! Layers panel: the semantic tree as a keyboard-first list.
//!
//! The panel mirrors the scene hierarchy (pages, then roots) as rows
//! with stable identities and screen-reader labels. Focus moves with
//! arrows, `Left`/`Right` collapse and expand, `Enter` selects and
//! `Space` toggles visibility through a real transaction. The panel
//! never mutates the document itself: it reports intents the session
//! applies.

use petunia_core::{ObjectId, PageId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// What a row represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerKind {
    Page,
    Group,
    Path,
    Shape,
    Text,
    Image,
    Pixel,
    Trace,
    Generated,
    Symbol,
    Other,
}

/// One visible-or-collapsed row in the panel order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerRow {
    /// Scene object, or `None` for page rows.
    pub id: Option<ObjectId>,
    /// Page this row belongs to.
    pub page: PageId,
    pub name: String,
    pub kind: LayerKind,
    /// Nesting depth: pages sit at zero.
    pub depth: usize,
    pub visible: bool,
    pub locked: bool,
    pub child_count: usize,
    /// One-based position among siblings, for announcements.
    pub position: usize,
    /// Sibling count, for announcements.
    pub siblings: usize,
    /// Whether the row is selected in the session.
    pub selected: bool,
}

impl LayerRow {
    /// Screen-reader label: what it is, its name, its state and its
    /// position. One sentence, no visual-only information.
    #[must_use]
    pub fn label(&self) -> String {
        let kind = match self.kind {
            LayerKind::Page => "Página",
            LayerKind::Group => "Grupo",
            LayerKind::Path => "Path",
            LayerKind::Shape => "Shape",
            LayerKind::Text => "Texto",
            LayerKind::Image => "Imagem",
            LayerKind::Pixel => "Pixel layer",
            LayerKind::Trace => "Trace",
            LayerKind::Generated => "Vetor gerado",
            LayerKind::Symbol => "Símbolo",
            LayerKind::Other => "Objeto",
        };
        let mut label = format!("{kind} {}", self.name);
        if self.child_count == 1 {
            label.push_str(", 1 item");
        } else if self.child_count > 1 {
            label.push_str(&format!(", {} itens", self.child_count));
        }
        if self.locked {
            label.push_str(", travado");
        }
        if !self.visible {
            label.push_str(", oculto");
        }
        if self.selected {
            label.push_str(", selecionado");
        }
        label.push_str(&format!(", {} de {}", self.position, self.siblings));
        label
    }
}

/// Keyboard intent on the panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerKey {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    /// Select the focused row.
    Enter,
    /// Toggle visibility of the focused object row.
    Space,
}

/// The panel state: rows in panel order, collapsed groups, focus and
/// the mirrored selection.
#[derive(Debug, Clone, Default)]
pub struct LayersPanel {
    rows: Vec<LayerRow>,
    collapsed: HashSet<ObjectId>,
    focus: Option<usize>,
    revision: u64,
}

impl LayersPanel {
    /// Empty panel.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// All rows, in panel order.
    #[must_use]
    pub fn rows(&self) -> &[LayerRow] {
        &self.rows
    }

    /// Rows reachable by keyboard: collapsed subtrees stay hidden.
    #[must_use]
    pub fn visible_rows(&self) -> Vec<&LayerRow> {
        let mut out = Vec::new();
        let mut skip_depth: Option<usize> = None;
        for row in &self.rows {
            if let Some(depth) = skip_depth {
                if row.depth > depth {
                    continue;
                }
                skip_depth = None;
            }
            if let Some(id) = row.id {
                if self.collapsed.contains(&id) {
                    skip_depth = Some(row.depth);
                }
            }
            out.push(row);
        }
        out
    }

    /// Focused row, if any.
    #[must_use]
    pub fn focused(&self) -> Option<&LayerRow> {
        self.focus
            .and_then(|index| self.visible_rows().get(index).copied())
    }

    /// Identifier under focus, for the session to act on.
    #[must_use]
    pub fn focused_id(&self) -> Option<ObjectId> {
        self.focused()?.id
    }

    /// Whether a group is collapsed.
    #[must_use]
    pub fn is_collapsed(&self, id: ObjectId) -> bool {
        self.collapsed.contains(&id)
    }

    /// Rebuild from fresh rows (page order, depth-first), preserving
    /// focus by identity, the collapsed set and the revision stamp.
    /// Vanished focus falls back to the first visible row.
    pub fn rebuild(&mut self, rows: Vec<LayerRow>, selected: &[ObjectId], revision: u64) {
        let focused_id = self.focused_id();
        self.rows = rows;
        for row in &mut self.rows {
            row.selected = row.id.is_some_and(|id| selected.contains(&id));
        }
        self.collapsed
            .retain(|id| self.rows.iter().any(|row| row.id == Some(*id)));
        self.revision = revision;
        let visible = self.visible_rows();
        self.focus = focused_id
            .and_then(|id| visible.iter().position(|row| row.id == Some(id)))
            .or(if visible.is_empty() { None } else { Some(0) });
    }

    /// Revision the panel was built from.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Move keyboard focus; returns whether focus changed.
    pub fn key(&mut self, key: LayerKey) -> bool {
        let count = self.visible_rows().len();
        if count == 0 {
            self.focus = None;
            return false;
        }
        let current = self.focus.unwrap_or(0).min(count - 1);
        let next = match key {
            LayerKey::Up => current.checked_sub(1),
            LayerKey::Down => (current + 1 < count).then_some(current + 1),
            LayerKey::Home => Some(0),
            LayerKey::End => Some(count - 1),
            LayerKey::Left => {
                // Collapse a group, else climb to its parent row.
                let target = self
                    .visible_rows()
                    .get(current)
                    .map(|row| (row.id, row.child_count));
                match target {
                    Some((Some(id), children)) if children > 0 && !self.collapsed.contains(&id) => {
                        self.collapsed.insert(id);
                        return true;
                    }
                    _ => return self.focus_parent(current),
                }
            }
            LayerKey::Right => {
                // Expand a collapsed group, else dive to its first child.
                let target = self
                    .visible_rows()
                    .get(current)
                    .map(|row| (row.id, row.child_count));
                match target {
                    Some((Some(id), _)) if self.collapsed.remove(&id) => return true,
                    Some((_, children)) if children > 0 && current + 1 < count => {
                        self.focus = Some(current + 1);
                        return true;
                    }
                    _ => None,
                }
            }
            LayerKey::Enter | LayerKey::Space => return false,
        };
        match next {
            Some(index) => {
                let changed = Some(index) != self.focus;
                self.focus = Some(index);
                changed
            }
            None => false,
        }
    }

    /// Toggle the collapsed state of the focused group.
    pub fn toggle_collapsed(&mut self) -> bool {
        let Some(row) = self.focused() else {
            return false;
        };
        let Some(id) = row.id else {
            return false;
        };
        if row.child_count == 0 {
            return false;
        }
        if self.collapsed.contains(&id) {
            self.collapsed.remove(&id);
        } else {
            self.collapsed.insert(id);
        }
        // Focus stays on the row; ensure it is still visible.
        let visible = self.visible_rows();
        self.focus = visible.iter().position(|row| row.id == Some(id));
        true
    }

    fn focus_parent(&mut self, current: usize) -> bool {
        let visible = self.visible_rows();
        let Some(row) = visible.get(current) else {
            return false;
        };
        let depth = row.depth;
        if depth == 0 {
            return false;
        }
        for (index, candidate) in visible.iter().enumerate().take(current).rev() {
            if candidate.depth < depth {
                self.focus = Some(index);
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel() -> (LayersPanel, ObjectId, ObjectId, ObjectId) {
        let group = ObjectId::new_v4();
        let leaf = ObjectId::new_v4();
        let lone = ObjectId::new_v4();
        let page = PageId::new_v4();
        let mut panel = LayersPanel::new();
        panel.rebuild(
            vec![
                LayerRow {
                    id: None,
                    page,
                    name: "Page 1".to_string(),
                    kind: LayerKind::Page,
                    depth: 0,
                    visible: true,
                    locked: false,
                    child_count: 2,
                    position: 1,
                    siblings: 1,
                    selected: false,
                },
                LayerRow {
                    id: Some(group),
                    page,
                    name: "header".to_string(),
                    kind: LayerKind::Group,
                    depth: 1,
                    visible: true,
                    locked: false,
                    child_count: 1,
                    position: 1,
                    siblings: 2,
                    selected: false,
                },
                LayerRow {
                    id: Some(leaf),
                    page,
                    name: "box".to_string(),
                    kind: LayerKind::Path,
                    depth: 2,
                    visible: true,
                    locked: true,
                    child_count: 0,
                    position: 1,
                    siblings: 1,
                    selected: false,
                },
                LayerRow {
                    id: Some(lone),
                    page,
                    name: "solo".to_string(),
                    kind: LayerKind::Path,
                    depth: 1,
                    visible: true,
                    locked: false,
                    child_count: 0,
                    position: 2,
                    siblings: 2,
                    selected: false,
                },
            ],
            &[leaf],
            7,
        );
        (panel, group, leaf, lone)
    }

    #[test]
    fn labels_carry_state_and_position() {
        let (panel, _, _, _) = panel();
        assert_eq!(
            panel.rows()[2].label(),
            "Path box, travado, selecionado, 1 de 1"
        );
        assert_eq!(panel.rows()[1].label(), "Grupo header, 1 item, 1 de 2");
    }

    #[test]
    fn arrows_walk_visible_rows_and_stop_at_edges() {
        let (mut panel, _, _, _) = panel();
        assert_eq!(panel.focused().map(|row| row.name.as_str()), Some("Page 1"));
        assert!(panel.key(LayerKey::Down));
        assert_eq!(panel.focused().map(|row| row.name.as_str()), Some("header"));
        assert!(panel.key(LayerKey::Down));
        assert_eq!(panel.focused().map(|row| row.name.as_str()), Some("box"));
        assert!(panel.key(LayerKey::Down));
        assert_eq!(panel.focused().map(|row| row.name.as_str()), Some("solo"));
        assert!(!panel.key(LayerKey::Down));
        assert!(panel.key(LayerKey::Up));
        assert_eq!(panel.focused().map(|row| row.name.as_str()), Some("box"));
        assert!(panel.key(LayerKey::Home));
        assert_eq!(panel.focused().map(|row| row.name.as_str()), Some("Page 1"));
        assert!(panel.key(LayerKey::End));
        assert_eq!(panel.focused().map(|row| row.name.as_str()), Some("solo"));
    }

    #[test]
    fn left_right_collapse_expand_and_climb() {
        let (mut panel, _, _, _) = panel();
        panel.key(LayerKey::Down);
        assert_eq!(panel.focused().map(|row| row.name.as_str()), Some("header"));
        // Left collapses the group; its child hides.
        assert!(panel.key(LayerKey::Left));
        assert_eq!(panel.visible_rows().len(), 3);
        assert_eq!(panel.focused().map(|row| row.name.as_str()), Some("header"));
        // Right expands it again.
        assert!(panel.key(LayerKey::Right));
        assert_eq!(panel.visible_rows().len(), 4);
        // Into the child, then Left climbs back to the parent.
        assert!(panel.key(LayerKey::Down));
        assert_eq!(panel.focused().map(|row| row.name.as_str()), Some("box"));
        assert!(panel.key(LayerKey::Left));
        assert_eq!(panel.focused().map(|row| row.name.as_str()), Some("header"));
    }

    #[test]
    fn rebuild_preserves_focus_collapse_and_selection() {
        let (mut panel, group, _, _) = panel();
        panel.key(LayerKey::Down);
        assert_eq!(panel.focused().map(|row| row.name.as_str()), Some("header"));
        panel.key(LayerKey::Left);
        assert!(panel.is_collapsed(group));
        // Rebuild with the focused...
        let mut rows: Vec<LayerRow> = panel.rows().to_vec();
        rows.retain(|row| row.name != "box");
        panel.rebuild(rows, &[], 8);
        assert_eq!(panel.revision(), 8);
        assert!(panel.is_collapsed(group));
        // Focus survived on the header by identity.
        assert_eq!(panel.focused().map(|row| row.name.as_str()), Some("header"));
        // And when the focused row itself vanishes, focus falls back.
        let mut rows: Vec<LayerRow> = panel.rows().to_vec();
        rows.retain(|row| row.name != "header");
        panel.rebuild(rows, &[], 9);
        assert_eq!(panel.focused().map(|row| row.name.as_str()), Some("Page 1"));
    }
}
