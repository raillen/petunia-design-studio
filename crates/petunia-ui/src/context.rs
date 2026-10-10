//! Session state: edit contexts and object/node selection.
//!
//! Both structures are pure Session State. Nothing here is persisted
//! in PTND, and selecting, focusing or pushing a context never moves
//! `DocumentRevision` — only a committed Transaction does.

use petunia_core::ObjectId;
use serde::{Deserialize, Serialize};

/// An edit context on the stack.
///
/// `Scene` is the root. Group/Vector/Text/Shape/Symbol contexts are
/// pushed on top; exiting pops exactly one level, preserving object
/// selection in Session State.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditContext {
    /// Root scene navigation.
    Scene,
    /// Inside a group, editing its children.
    Group { group: ObjectId },
    /// Vector editing over one or more eligible paths.
    Vector {
        targets: Vec<ObjectId>,
        operation: VectorOperation,
    },
    /// Shape parameters without converting to paths.
    Shape { target: ObjectId },
    /// Text editing; consumes typed keys before global commands.
    Text { target: ObjectId },
    /// Symbol instance overrides; does not edit the definition.
    Symbol { target: ObjectId },
}

/// Operations available inside Vector Edit. `Node` is the default.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum VectorOperation {
    /// Node placement and drag; the default operation.
    #[default]
    Node,
    Bend,
    Pen,
    Cut,
    Smooth,
    Width,
}

impl VectorOperation {
    /// Stable identifier for ActionIds, labels and tests.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Node => "node",
            Self::Bend => "bend",
            Self::Pen => "pen",
            Self::Cut => "cut",
            Self::Smooth => "smooth",
            Self::Width => "width",
        }
    }
}

/// Why a context request was refused. The UI surfaces the reason
/// instead of silently doing nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextRejection {
    /// Target does not exist in the document.
    Missing(ObjectId),
    /// Target is hidden, locked or otherwise not editable.
    NotEligible(ObjectId),
    /// Multi-path request contained an ineligible object.
    MixedSelection {
        eligible: Vec<ObjectId>,
        excluded: Vec<ObjectId>,
    },
    /// No path target at all.
    NoTarget,
}

impl ContextRejection {
    /// Human-readable reason for a disabled control with cause.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::Missing(id) => format!("objeto {id} não existe mais"),
            Self::NotEligible(id) => format!("objeto {id} está travado ou oculto"),
            Self::MixedSelection { excluded, .. } => format!(
                "seleção mista: {} objeto(s) não são paths editáveis",
                excluded.len()
            ),
            Self::NoTarget => "nenhum path selecionado".to_string(),
        }
    }
}

/// The context stack, always rooted at `Scene`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ContextStack {
    stack: Vec<EditContext>,
}

/// Which item kinds can open their own context on double-click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Path,
    Group,
    Text,
    Shape,
    Symbol,
    Other,
}

impl ContextStack {
    /// A fresh stack rooted at `Scene`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            stack: vec![EditContext::Scene],
        }
    }

    /// Current (topmost) context.
    #[must_use]
    pub fn current(&self) -> &EditContext {
        self.stack.last().unwrap_or(&EditContext::Scene)
    }

    /// Depth of the stack; 1 means Scene only.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Breadcrumb projection for the UI.
    #[must_use]
    pub fn breadcrumb(&self) -> Vec<&EditContext> {
        self.stack.iter().collect()
    }

    /// Push a context on top.
    pub fn push(&mut self, context: EditContext) {
        self.stack.push(context);
    }

    /// Pop one level, never below `Scene`.
    pub fn pop(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    /// Escape ordering for an idle stack: cancels inner operations
    /// first, then unwinds one level at a time.
    pub fn on_escape(&mut self) -> EscapeOutcome {
        if let EditContext::Vector { operation, .. } = self.current() {
            if *operation != VectorOperation::Node {
                let mut next = self.current().clone();
                if let EditContext::Vector { operation, .. } = &mut next {
                    *operation = VectorOperation::Node;
                }
                self.stack.pop();
                self.stack.push(next);
                return EscapeOutcome::ReturnedToNode;
            }
        }
        if self.depth() > 1 {
            self.pop();
            EscapeOutcome::PoppedContext
        } else {
            EscapeOutcome::AtRoot
        }
    }

    /// Whether the stack is editing vectors right now.
    #[must_use]
    pub fn in_vector(&self) -> bool {
        matches!(self.current(), EditContext::Vector { .. })
    }

    /// Whether the stack is inside a group.
    #[must_use]
    pub fn in_group(&self) -> bool {
        matches!(self.current(), EditContext::Group { .. })
    }

    /// Active vector operation, when in Vector Edit.
    #[must_use]
    pub fn vector_operation(&self) -> Option<VectorOperation> {
        match self.current() {
            EditContext::Vector { operation, .. } => Some(*operation),
            _ => None,
        }
    }

    /// Switch the active vector operation, preserving targets.
    pub fn set_vector_operation(&mut self, operation: VectorOperation) -> bool {
        match self.current_mut() {
            EditContext::Vector {
                operation: current, ..
            } => {
                *current = operation;
                true
            }
            _ => false,
        }
    }

    fn current_mut(&mut self) -> &mut EditContext {
        self.stack.last_mut().expect("stack is never empty")
    }
}

/// What an Escape did, so the UI can decide what to announce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeOutcome {
    /// A sub-operation returned to `Node`, sub-selection kept.
    ReturnedToNode,
    /// One context level was popped.
    PoppedContext,
    /// Already at Scene root; nothing to unwind.
    AtRoot,
}

/// Sub-selection inside Vector Edit, addressed by stable IDs only.
///
/// No indices: reorder must never invalidate the selection.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubSelection {
    nodes: Vec<NodeId>,
    segments: Vec<SegmentId>,
    handles: Vec<HandleId>,
}

/// A node inside a specific contour of a specific path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId {
    pub object: ObjectId,
    pub contour: u32,
    pub node: u32,
}

/// A segment between two nodes of a contour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SegmentId {
    pub object: ObjectId,
    pub contour: u32,
    pub from: u32,
    pub to: u32,
}

/// Which handle of a node, in local path space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HandleRef {
    In,
    Out,
}

/// Handle target in the semantic tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HandleId {
    pub object: ObjectId,
    pub contour: u32,
    pub node: u32,
    pub handle: HandleRef,
}

impl SubSelection {
    /// Empty sub-selection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle a node. Returns true when now selected.
    pub fn toggle_node(&mut self, id: NodeId) -> bool {
        toggle_in(&mut self.nodes, id)
    }

    /// Toggle a segment.
    pub fn toggle_segment(&mut self, id: SegmentId) -> bool {
        toggle_in(&mut self.segments, id)
    }

    /// Toggle a handle.
    pub fn toggle_handle(&mut self, id: HandleId) -> bool {
        toggle_in(&mut self.handles, id)
    }

    /// Replace the sub-selection by one node.
    pub fn select_node(&mut self, id: NodeId) {
        self.nodes.clear();
        self.segments.clear();
        self.handles.clear();
        self.nodes.push(id);
    }
    /// Replace the sub-selection by one segment.
    pub fn select_segment(&mut self, id: SegmentId) {
        self.nodes.clear();
        self.segments.clear();
        self.handles.clear();
        self.segments.push(id);
    }

    /// Whether anything is selected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.segments.is_empty() && self.handles.is_empty()
    }

    /// Selected nodes, in stable-ID order.
    #[must_use]
    pub fn nodes(&self) -> &[NodeId] {
        &self.nodes
    }

    /// Total selected targets across kinds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len() + self.segments.len() + self.handles.len()
    }

    /// Clear every kind.
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.segments.clear();
        self.handles.clear();
    }

    /// Drop IDs that no longer belong to `targets`.
    pub fn retain_targets(&mut self, targets: &[ObjectId]) {
        self.nodes.retain(|id| targets.contains(&id.object));
        self.segments.retain(|id| targets.contains(&id.object));
        self.handles.retain(|id| targets.contains(&id.object));
    }
}

fn toggle_in<T: PartialEq>(items: &mut Vec<T>, id: T) -> bool {
    if let Some(position) = items.iter().position(|item| *item == id) {
        items.remove(position);
        false
    } else {
        items.push(id);
        true
    }
}

/// Object selection and optional sub-selection: Session State.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SelectionState {
    objects: Vec<ObjectId>,
    pub sub: SubSelection,
}

impl SelectionState {
    /// Empty selection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Ordered object IDs, topmost first.
    #[must_use]
    pub fn objects(&self) -> &[ObjectId] {
        &self.objects
    }

    /// Replace the object selection, dropping stale sub-selection.
    pub fn set_objects(&mut self, objects: Vec<ObjectId>) {
        self.objects = objects;
        self.sub.clear();
    }

    /// Shift+click semantics: toggle one object.
    pub fn toggle_object(&mut self, id: ObjectId) {
        toggle_in(&mut self.objects, id);
    }

    /// Add objects (marquee Add / Ctrl+A results).
    pub fn add_objects(&mut self, ids: impl IntoIterator<Item = ObjectId>) {
        for id in ids {
            if !self.objects.contains(&id) {
                self.objects.push(id);
            }
        }
    }

    /// Clear everything.
    pub fn clear(&mut self) {
        self.objects.clear();
        self.sub.clear();
    }

    /// Single selected object, when exactly one.
    #[must_use]
    pub fn single(&self) -> Option<ObjectId> {
        if self.objects.len() == 1 {
            Some(self.objects[0])
        } else {
            None
        }
    }

    /// Whether the object is selected.
    #[must_use]
    pub fn contains(&self, id: ObjectId) -> bool {
        self.objects.contains(&id)
    }

    /// Number of selected objects.
    #[must_use]
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    /// Whether no object is selected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object() -> ObjectId {
        ObjectId::new_v4()
    }

    #[test]
    fn stack_starts_at_scene_and_never_roots_out() {
        let mut stack = ContextStack::new();
        assert_eq!(stack.current(), &EditContext::Scene);
        assert_eq!(stack.depth(), 1);
        stack.pop();
        assert_eq!(stack.current(), &EditContext::Scene);
        assert!(stack.on_escape() == EscapeOutcome::AtRoot);
    }

    #[test]
    fn escape_unwinds_operation_before_context() {
        let mut stack = ContextStack::new();
        let path = object();
        stack.push(EditContext::Vector {
            targets: vec![path],
            operation: VectorOperation::Bend,
        });

        // First Escape returns to Node, keeping the same target.
        assert_eq!(stack.on_escape(), EscapeOutcome::ReturnedToNode);
        assert_eq!(stack.vector_operation(), Some(VectorOperation::Node));

        // Second Escape pops the context.
        assert_eq!(stack.on_escape(), EscapeOutcome::PoppedContext);
        assert!(!stack.in_vector());
    }

    #[test]
    fn escape_from_group_pops_exactly_one_level() {
        let mut stack = ContextStack::new();
        let outer = object();
        let inner = object();
        stack.push(EditContext::Group { group: outer });
        stack.push(EditContext::Group { group: inner });
        assert_eq!(stack.depth(), 3);
        stack.on_escape();
        assert_eq!(stack.depth(), 2);
        assert_eq!(stack.current(), &EditContext::Group { group: outer });
    }

    #[test]
    fn switching_operation_preserves_targets() {
        let mut stack = ContextStack::new();
        let path = object();
        stack.push(EditContext::Vector {
            targets: vec![path],
            operation: VectorOperation::Node,
        });
        assert!(stack.set_vector_operation(VectorOperation::Width));
        match stack.current() {
            EditContext::Vector { targets, .. } => assert_eq!(targets, &[path]),
            other => panic!("expected vector, got {other:?}"),
        }
        // Outside Vector Edit the switch is refused.
        stack.pop();
        assert!(!stack.set_vector_operation(VectorOperation::Bend));
    }

    #[test]
    fn sub_selection_toggles_and_clears() {
        let path = object();
        let node = NodeId {
            object: path,
            contour: 0,
            node: 3,
        };
        let other = NodeId {
            object: path,
            contour: 0,
            node: 4,
        };
        let mut sub = SubSelection::new();
        assert!(sub.toggle_node(node));
        assert!(sub.toggle_node(other));
        assert_eq!(sub.len(), 2);
        assert!(!sub.toggle_node(node));
        assert_eq!(sub.len(), 1);
        sub.select_node(node);
        assert_eq!(sub.len(), 1);
        sub.clear();
        assert!(sub.is_empty());
    }

    #[test]
    fn sub_selection_drops_ids_of_vanished_targets() {
        let path = object();
        let gone = object();
        let mut sub = SubSelection::new();
        sub.select_node(NodeId {
            object: path,
            contour: 0,
            node: 1,
        });
        sub.toggle_node(NodeId {
            object: gone,
            contour: 0,
            node: 2,
        });
        sub.retain_targets(&[path]);
        assert_eq!(sub.len(), 1);
    }

    #[test]
    fn object_selection_is_shift_toggle_and_ordered() {
        let mut selection = SelectionState::new();
        assert!(selection.is_empty());
        let a = object();
        let b = object();
        selection.toggle_object(a);
        selection.toggle_object(b);
        assert_eq!(selection.objects(), &[a, b]);
        selection.toggle_object(a);
        assert_eq!(selection.objects(), &[b]);
        assert_eq!(selection.single(), Some(b));

        // Replacing objects clears sub-selection.
        selection.sub.toggle_node(NodeId {
            object: b,
            contour: 0,
            node: 0,
        });
        assert!(!selection.sub.is_empty());
        selection.set_objects(vec![a]);
        assert!(selection.sub.is_empty());
    }

    #[test]
    fn context_rejections_explain_themselves() {
        let id = object();
        let mixed = ContextRejection::MixedSelection {
            eligible: vec![id],
            excluded: vec![ObjectId::new_v4(), ObjectId::new_v4()],
        };
        assert!(mixed.message().contains("2"), "{}", mixed.message());
        assert!(!ContextRejection::NotEligible(id).message().is_empty());
    }
}
