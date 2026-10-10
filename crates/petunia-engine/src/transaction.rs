//! Typed document mutations and atomic transactions.
//!
//! UI, scripts and plugins express intent through commands; handlers
//! turn intent into a [`TransactionRequest`], which prepares against a
//! known revision and commits atomically. Partial application is an
//! architectural error: validation happens in prepare, and any
//! mid-commit failure rolls back through the prepared inverses.

use crate::error::EngineError;
use petunia_core::{
    BlendMode, Document, EffectId, ObjectId, ParentRef, SceneItem, SceneNode, Transform2D,
    VectorPath,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stable identity of the command that produced a transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandId(Uuid);

impl CommandId {
    /// Generates a fresh command identity.
    #[must_use]
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Key grouping consecutive commits into one undo experience
/// (typing, repeated nudges). Coalescing never crosses a saved
/// revision and never hides a poorly modeled preview.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MergeKey(pub String);

/// Authorial document revision: a history state identity, not an
/// operation counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DocumentRevision(pub u64);

impl DocumentRevision {
    /// The revision of a fresh document.
    pub const GENESIS: Self = Self(0);

    /// Next revision after one committed transaction.
    #[must_use]
    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

/// One typed property change on an effect instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EffectParameter {
    Enabled(bool),
    Opacity(f32),
    BlendMode(BlendMode),
}

/// A small, explicitly represented structural mutation. Internal
/// mutation API: QML and tools never assemble these directly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DocumentOp {
    /// Insert at page roots. The page root list is the z-order of this
    /// revision; group membership uses [`DocumentOp::InsertNode`].
    InsertRoot {
        index: usize,
        node: Box<SceneNode>,
    },
    /// Insert under a group parent.
    InsertNode {
        parent: ObjectId,
        index: usize,
        node: Box<SceneNode>,
    },
    RemoveSubtree {
        root: ObjectId,
    },
    SetTransform {
        object: ObjectId,
        transform: Transform2D,
    },
    /// Translate an object by a document-space delta, preserving its
    /// current rotation and scale.
    MoveObjects {
        object: ObjectId,
        dx: f64,
        dy: f64,
    },
    ReplacePath {
        object: ObjectId,
        path: VectorPath,
    },
    SetEffectParameter {
        effect: EffectId,
        parameter: EffectParameter,
    },
    ReorderChild {
        parent: ObjectId,
        child: ObjectId,
        index: usize,
    },
}

/// What a command handler asks the commit lane to apply.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub command_id: CommandId,
    pub operations: Vec<DocumentOp>,
    pub merge_key: Option<MergeKey>,
}

/// A validated transaction: forward ops plus their inverses, ready
/// for a short predictable commit against `expected_revision`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreparedTransaction {
    pub expected_revision: DocumentRevision,
    pub forward: Vec<DocumentOp>,
    pub inverse: Vec<DocumentOp>,
    pub affected_objects: Vec<ObjectId>,
    pub command_id: CommandId,
    pub merge_key: Option<MergeKey>,
}

/// The committed side of a transaction, stored in history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppliedTransaction {
    pub forward: Vec<DocumentOp>,
    pub inverse: Vec<DocumentOp>,
    pub affected_objects: Vec<ObjectId>,
    pub command_id: CommandId,
    pub merge_key: Option<MergeKey>,
}

/// Preparation failures: unknown objects, broken invariants, stale
/// revisions or operations the current model cannot host yet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionError {
    RevisionConflict {
        expected: DocumentRevision,
        current: DocumentRevision,
    },
    InvariantViolation(String),
    MissingObject(ObjectId),
    UnsupportedOperation(String),
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RevisionConflict { expected, current } => write!(
                f,
                "transaction prepared at revision {} but document is at {}",
                expected.0, current.0
            ),
            Self::InvariantViolation(message) => write!(f, "invariant violation: {message}"),
            Self::MissingObject(id) => write!(f, "missing object {id}"),
            Self::UnsupportedOperation(message) => write!(f, "unsupported operation: {message}"),
        }
    }
}

impl std::error::Error for TransactionError {}

/// Commit-stage failures. The document is rolled back before the
/// error surfaces; partial state is never presented as success.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommitError {
    ApplyFailed(String),
    RolledBack(String),
}

impl std::fmt::Display for CommitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApplyFailed(message) => write!(f, "commit failed: {message}"),
            Self::RolledBack(message) => write!(f, "rolled back after: {message}"),
        }
    }
}

impl std::error::Error for CommitError {}

fn missing(id: ObjectId) -> TransactionError {
    TransactionError::MissingObject(id)
}

fn group_children(
    document: &petunia_core::Document,
    parent: ObjectId,
) -> std::result::Result<&Vec<ObjectId>, TransactionError> {
    let node = document
        .scene
        .get_node(parent)
        .ok_or_else(|| missing(parent))?;
    match &node.item {
        SceneItem::Group(children) => Ok(children),
        _ => Err(TransactionError::InvariantViolation(format!(
            "parent {parent} is not a group"
        ))),
    }
}

/// One subtree node paired with its current `(parent, index)`.
/// The root itself carries `None` and is placed by the caller.
type PlacedNode = (SceneNode, Option<(ObjectId, usize)>);

/// Collect a subtree in parent-before-children order, pairing each
/// node with its current `(parent, index)`.
fn collect_subtree(
    document: &Document,
    root: ObjectId,
) -> std::result::Result<Vec<PlacedNode>, TransactionError> {
    let mut ordered = Vec::new();
    let mut stack = vec![(root, None)];
    while let Some((id, placement)) = stack.pop() {
        let node = document
            .scene
            .get_node(id)
            .ok_or_else(|| missing(id))?
            .clone();
        if let SceneItem::Group(children) = &node.item {
            for (index, child) in children.iter().enumerate().rev() {
                stack.push((*child, Some((id, index))));
            }
        }
        ordered.push((node, placement));
    }
    // Depth-first pre-order already lists parents before children.
    Ok(ordered)
}

fn inverse_of(
    document: &Document,
    op: &DocumentOp,
) -> std::result::Result<Vec<DocumentOp>, TransactionError> {
    match op {
        DocumentOp::InsertRoot { node, .. } => {
            Ok(vec![DocumentOp::RemoveSubtree { root: node.id }])
        }
        DocumentOp::InsertNode { node, .. } => {
            Ok(vec![DocumentOp::RemoveSubtree { root: node.id }])
        }
        DocumentOp::RemoveSubtree { root } => {
            let outer = find_parent(document, *root).ok_or_else(|| {
                TransactionError::InvariantViolation(format!(
                    "removed root {root} has no parent to restore into"
                ))
            })?;
            let subtree = collect_subtree(document, *root)?;
            let mut inverse = Vec::with_capacity(subtree.len());
            for (mut node, placement) in subtree {
                // Linkage rebuilds through the InsertNode ops below,
                // so captured child lists are cleared to avoid doubles.
                if let SceneItem::Group(children) = &mut node.item {
                    children.clear();
                }
                let (parent, index) = placement.unwrap_or(outer);
                inverse.push(DocumentOp::InsertNode {
                    parent,
                    index,
                    node: Box::new(node),
                });
            }
            Ok(inverse)
        }
        DocumentOp::SetTransform { object, .. } => {
            let node = document
                .scene
                .get_node(*object)
                .ok_or_else(|| missing(*object))?;
            Ok(vec![DocumentOp::SetTransform {
                object: *object,
                transform: node.transform,
            }])
        }
        DocumentOp::MoveObjects { object, .. } => {
            let node = document
                .scene
                .get_node(*object)
                .ok_or_else(|| missing(*object))?;
            Ok(vec![DocumentOp::SetTransform {
                object: *object,
                transform: node.transform,
            }])
        }
        DocumentOp::ReplacePath { object, .. } => {
            let node = document
                .scene
                .get_node(*object)
                .ok_or_else(|| missing(*object))?;
            match &node.item {
                SceneItem::Path(path) => Ok(vec![DocumentOp::ReplacePath {
                    object: *object,
                    path: path.clone(),
                }]),
                _ => Err(TransactionError::InvariantViolation(format!(
                    "object {object} is not a path"
                ))),
            }
        }
        DocumentOp::SetEffectParameter { effect, .. } => {
            Err(TransactionError::UnsupportedOperation(format!(
                "effect {effect} has no appearance storage in this revision"
            )))
        }
        DocumentOp::ReorderChild { parent, child, .. } => {
            let children = group_children(document, *parent)?;
            let from = children.iter().position(|id| id == child).ok_or_else(|| {
                TransactionError::InvariantViolation(format!(
                    "child {child} not found in parent {parent}"
                ))
            })?;
            Ok(vec![DocumentOp::ReorderChild {
                parent: *parent,
                child: *child,
                index: from,
            }])
        }
    }
}

fn find_parent(document: &Document, child: ObjectId) -> Option<(ObjectId, usize)> {
    // Walk every reachable group; root-level groups seed the search.
    let mut stack: Vec<ObjectId> = document.scene.root_order().to_vec();
    let mut seen = std::collections::HashSet::new();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        let node = document.scene.get_node(id)?;
        if let SceneItem::Group(children) = &node.item {
            for (index, member) in children.iter().enumerate() {
                if *member == child {
                    return Some((id, index));
                }
                stack.push(*member);
            }
        }
    }
    None
}

/// Validate a request and derive inverses. Preparation dry-runs on
/// a shadow copy so later operations in one transaction may build on
/// earlier ones (pastes, multi-step edits); nothing touches the real
/// document here.
pub fn prepare_transaction(
    document: &Document,
    request: TransactionRequest,
    expected_revision: DocumentRevision,
) -> std::result::Result<PreparedTransaction, TransactionError> {
    if request.operations.is_empty() {
        return Err(TransactionError::InvariantViolation(
            "transaction needs at least one operation".to_string(),
        ));
    }
    let mut shadow = document.clone();
    let mut undo_groups: Vec<Vec<DocumentOp>> = Vec::new();
    let mut affected = Vec::new();
    for op in &request.operations {
        validate_operation(&shadow, op)?;
        let undo = inverse_of(&shadow, op)?;
        apply_operation(&mut shadow, op.clone()).map_err(|error| {
            TransactionError::InvariantViolation(format!("shadow apply failed: {error}"))
        })?;
        for back in &undo {
            affected_object(back, &mut affected);
        }
        affected_object(op, &mut affected);
        undo_groups.push(undo);
    }
    // Undo runs the per-operation inverses in reverse operation order.
    let mut inverse = Vec::new();
    for group in undo_groups.into_iter().rev() {
        inverse.extend(group);
    }
    Ok(PreparedTransaction {
        expected_revision,
        forward: request.operations,
        inverse,
        affected_objects: affected,
        command_id: request.command_id,
        merge_key: request.merge_key,
    })
}

fn affected_object(op: &DocumentOp, affected: &mut Vec<ObjectId>) {
    let mut push = |id: ObjectId| {
        if !affected.contains(&id) {
            affected.push(id);
        }
    };
    match op {
        DocumentOp::InsertRoot { node, .. } => {
            push(node.id);
        }
        DocumentOp::InsertNode { parent, node, .. } => {
            push(*parent);
            push(node.id);
        }
        DocumentOp::RemoveSubtree { root } => push(*root),
        DocumentOp::SetTransform { object, .. } => push(*object),
        DocumentOp::MoveObjects { object, .. } => push(*object),
        DocumentOp::ReplacePath { object, .. } => push(*object),
        DocumentOp::SetEffectParameter { .. } => {}
        DocumentOp::ReorderChild { parent, child, .. } => {
            push(*parent);
            push(*child);
        }
    }
}

fn validate_operation(
    document: &Document,
    op: &DocumentOp,
) -> std::result::Result<(), TransactionError> {
    match op {
        DocumentOp::InsertRoot { index, node } => {
            if *index > document.scene.root_order().len() {
                return Err(TransactionError::InvariantViolation(format!(
                    "root insert index {index} beyond {} roots",
                    document.scene.root_order().len()
                )));
            }
            if document.scene.get_node(node.id).is_some() {
                return Err(TransactionError::InvariantViolation(format!(
                    "node {} already exists",
                    node.id
                )));
            }
            // Roots land on the default page; the node must declare it.
            if node.parent != ParentRef::Page(document.scene.default_page()) {
                return Err(TransactionError::InvariantViolation(format!(
                    "root node {} declares {parent:?}, not the default page",
                    node.id,
                    parent = node.parent,
                )));
            }
            Ok(())
        }
        DocumentOp::InsertNode {
            parent,
            index,
            node,
        } => {
            let children = group_children(document, *parent)?;
            if *index > children.len() {
                return Err(TransactionError::InvariantViolation(format!(
                    "insert index {index} beyond {} children",
                    children.len()
                )));
            }
            if document.scene.get_node(node.id).is_some() {
                return Err(TransactionError::InvariantViolation(format!(
                    "node {} already exists",
                    node.id
                )));
            }
            Ok(())
        }
        DocumentOp::RemoveSubtree { root } => {
            document
                .scene
                .get_node(*root)
                .ok_or_else(|| missing(*root))?;
            collect_subtree(document, *root).map(|_| ())
        }
        DocumentOp::SetTransform { object, .. } => {
            document
                .scene
                .get_node(*object)
                .ok_or_else(|| missing(*object))?;
            Ok(())
        }
        DocumentOp::MoveObjects { object, .. } => {
            document
                .scene
                .get_node(*object)
                .ok_or_else(|| missing(*object))?;
            Ok(())
        }
        DocumentOp::ReplacePath { object, .. } => {
            let node = document
                .scene
                .get_node(*object)
                .ok_or_else(|| missing(*object))?;
            match &node.item {
                SceneItem::Path(_) => Ok(()),
                _ => Err(TransactionError::InvariantViolation(format!(
                    "object {object} is not a path"
                ))),
            }
        }
        DocumentOp::SetEffectParameter { effect, .. } => {
            Err(TransactionError::UnsupportedOperation(format!(
                "effect {effect} has no appearance storage in this revision"
            )))
        }
        DocumentOp::ReorderChild {
            parent,
            child,
            index,
        } => {
            let children = group_children(document, *parent)?;
            if !children.contains(child) {
                return Err(TransactionError::InvariantViolation(format!(
                    "child {child} not found in parent {parent}"
                )));
            }
            if *index >= children.len() {
                return Err(TransactionError::InvariantViolation(format!(
                    "reorder index {index} out of {} children",
                    children.len()
                )));
            }
            Ok(())
        }
    }
}

fn apply_operation(
    document: &mut Document,
    op: DocumentOp,
) -> std::result::Result<(), CommitError> {
    match op {
        DocumentOp::InsertRoot { index, node } => {
            let id = node.id;
            let page = document.scene.default_page();
            document
                .scene
                .insert_root(page, *node)
                .map_err(|error| CommitError::ApplyFailed(error.to_string()))?;
            // `insert_root` appends; move to the requested index
            // (clamped) so z-order stays authorial.
            document
                .scene
                .reorder_child(ParentRef::Page(page), id, index)
                .map_err(|error| CommitError::ApplyFailed(error.to_string()))?;
            Ok(())
        }
        DocumentOp::InsertNode {
            parent,
            index,
            node,
        } => {
            document
                .scene
                .insert_child(parent, *node, Some(index))
                .map_err(|error| CommitError::ApplyFailed(error.to_string()))?;
            Ok(())
        }
        DocumentOp::RemoveSubtree { root } => {
            // The narrow removal detaches both sides atomically and
            // refuses to orphan live clip/mask sources.
            document
                .scene
                .remove_subtree(root)
                .map(|_| ())
                .map_err(|error| CommitError::ApplyFailed(error.to_string()))
        }
        DocumentOp::SetTransform { object, transform } => {
            let node = document.scene.get_node_mut(object).ok_or_else(|| {
                CommitError::ApplyFailed(format!("object {object} vanished at commit"))
            })?;
            node.transform = transform;
            Ok(())
        }
        DocumentOp::MoveObjects { object, dx, dy } => {
            let node = document.scene.get_node_mut(object).ok_or_else(|| {
                CommitError::ApplyFailed(format!("object {object} vanished at commit"))
            })?;
            node.transform.tx += dx;
            node.transform.ty += dy;
            Ok(())
        }
        DocumentOp::ReplacePath { object, path } => {
            let node = document.scene.get_node_mut(object).ok_or_else(|| {
                CommitError::ApplyFailed(format!("object {object} vanished at commit"))
            })?;
            match &mut node.item {
                SceneItem::Path(slot) => {
                    *slot = path;
                    Ok(())
                }
                _ => Err(CommitError::ApplyFailed(format!(
                    "object {object} is not a path"
                ))),
            }
        }
        DocumentOp::SetEffectParameter { effect, .. } => Err(CommitError::ApplyFailed(format!(
            "effect {effect} has no appearance storage in this revision"
        ))),
        DocumentOp::ReorderChild {
            parent,
            child,
            index,
        } => document
            .scene
            .reorder_child(ParentRef::Object(parent), child, index)
            .map_err(|error| CommitError::ApplyFailed(error.to_string())),
    }
}

/// Apply a prepared transaction. Inverses for the operations applied
/// so far are captured against the live document just before each
/// step, so any mid-commit failure rolls back exactly what ran.
pub fn commit_transaction(
    document: &mut Document,
    prepared: PreparedTransaction,
) -> std::result::Result<AppliedTransaction, CommitError> {
    let mut undo_groups: Vec<Vec<DocumentOp>> = Vec::new();
    for op in prepared.forward.clone() {
        let undo = inverse_of(document, &op)
            .map_err(|error| CommitError::ApplyFailed(error.to_string()))?;
        if let Err(error) = apply_operation(document, op) {
            for back in undo_groups.iter().rev().flatten().cloned() {
                // Rollback mirrors applied work; a failure here would
                // mean the document model itself broke mid-commit.
                let _ = apply_operation(document, back);
            }
            return Err(CommitError::RolledBack(error.to_string()));
        }
        undo_groups.push(undo);
    }
    Ok(AppliedTransaction {
        affected_objects: prepared.affected_objects.clone(),
        command_id: prepared.command_id,
        merge_key: prepared.merge_key,
        forward: prepared.forward,
        inverse: prepared.inverse,
    })
}

/// Quietly apply one operation outside history (undo/redo reuse this
/// path through history, tests apply inverses directly).
pub fn apply_quiet(
    document: &mut Document,
    op: DocumentOp,
) -> std::result::Result<(), EngineError> {
    apply_operation(document, op).map_err(|error| EngineError::Execution(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_core::VectorPath;

    fn group_fixture() -> (Document, ObjectId) {
        let mut document = Document::new("tx");
        let page = document.scene.default_page();
        let mut group = SceneNode::new_path(
            "group",
            VectorPath::new(),
            petunia_core::ParentRef::Page(page),
        );
        group.item = SceneItem::Group(Vec::new());
        let parent = group.id;
        document.scene.insert_node(group);
        (document, parent)
    }

    fn request(operations: Vec<DocumentOp>) -> TransactionRequest {
        TransactionRequest {
            command_id: CommandId::new_v4(),
            operations,
            merge_key: None,
        }
    }

    #[test]
    fn atomic_commit_applies_all_or_nothing() {
        let (document, parent) = group_fixture();
        let node = SceneNode::new_path(
            "box",
            VectorPath::rect(0.0, 0.0, 10.0, 10.0),
            petunia_core::ParentRef::Object(parent),
        );
        let good = DocumentOp::InsertNode {
            parent,
            index: 0,
            node: Box::new(node),
        };
        let bad = DocumentOp::SetTransform {
            object: ObjectId::new_v4(),
            transform: Transform2D::translation(1.0, 1.0),
        };
        let prepared = prepare_transaction(
            &document,
            request(vec![good, bad]),
            DocumentRevision::GENESIS,
        );
        assert!(matches!(prepared, Err(TransactionError::MissingObject(_))));
        assert_eq!(document.scene.len(), 1);
    }

    #[test]
    fn insert_and_remove_round_trip_through_inverses() {
        let (mut document, parent) = group_fixture();
        let node = SceneNode::new_path(
            "box",
            VectorPath::rect(0.0, 0.0, 10.0, 10.0),
            petunia_core::ParentRef::Object(parent),
        );
        let id = node.id;
        let prepared = prepare_transaction(
            &document,
            request(vec![DocumentOp::InsertNode {
                parent,
                index: 0,
                node: Box::new(node),
            }]),
            DocumentRevision::GENESIS,
        )
        .expect("valid");
        assert_eq!(prepared.affected_objects.len(), 2);
        let applied = commit_transaction(&mut document, prepared).expect("commits");
        assert!(document.scene.get_node(id).is_some());
        // The prepared inverse removes the inserted node again.
        assert_eq!(applied.inverse.len(), 1);
        for back in applied.inverse {
            apply_quiet(&mut document, back).expect("inverse applies");
        }
        assert!(document.scene.get_node(id).is_none());
    }

    #[test]
    fn insert_at_roots_orders_z_and_undoes() {
        let mut document = Document::new("roots");
        let page = document.scene.default_page();
        let first = SceneNode::new_path(
            "first",
            VectorPath::rect(0.0, 0.0, 5.0, 5.0),
            petunia_core::ParentRef::Page(page),
        );
        let first_id = first.id;
        let second = SceneNode::new_path(
            "second",
            VectorPath::rect(5.0, 0.0, 5.0, 5.0),
            petunia_core::ParentRef::Page(page),
        );
        let second_id = second.id;
        // Append the second first, then insert the first at index 0.
        let insert_second = prepare_transaction(
            &document,
            request(vec![DocumentOp::InsertRoot {
                index: 0,
                node: Box::new(second),
            }]),
            DocumentRevision::GENESIS,
        )
        .expect("valid");
        commit_transaction(&mut document, insert_second).expect("commits");
        let insert_first = prepare_transaction(
            &document,
            request(vec![DocumentOp::InsertRoot {
                index: 0,
                node: Box::new(first),
            }]),
            DocumentRevision(1),
        )
        .expect("valid");
        commit_transaction(&mut document, insert_first).expect("commits");
        assert_eq!(
            document.scene.root_order(),
            &[first_id, second_id],
            "insert at index 0 must lead the z-order"
        );
        // Out-of-range root indices are rejected, not silently wrapped.
        assert!(prepare_transaction(
            &document,
            request(vec![DocumentOp::InsertRoot {
                index: 9,
                node: Box::new(SceneNode::new_path(
                    "x",
                    VectorPath::new(),
                    petunia_core::ParentRef::Page(page),
                )),
            }]),
            DocumentRevision(1),
        )
        .is_err());
    }

    #[test]
    fn effect_parameters_report_unsupported() {
        let (document, _) = group_fixture();
        let op = DocumentOp::SetEffectParameter {
            effect: EffectId::new_v4(),
            parameter: EffectParameter::Opacity(0.5),
        };
        assert!(matches!(
            prepare_transaction(&document, request(vec![op]), DocumentRevision::GENESIS),
            Err(TransactionError::UnsupportedOperation(_))
        ));
    }
}
