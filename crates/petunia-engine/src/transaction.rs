//! Typed document mutations and atomic transactions.
//!
//! UI, scripts and plugins express intent through commands; handlers
//! turn intent into a [`TransactionRequest`], which prepares against a
//! known revision and commits atomically. Partial application is an
//! architectural error: validation happens in prepare, and any
//! mid-commit failure rolls back through the prepared inverses.

use crate::error::EngineError;
use petunia_core::{
    Appearance, BlendMode, Document, EffectId, GridDefinition, GridId, ObjectId, ParentRef,
    ResourceId, ResourceRecord, SceneItem, SceneNode, SpotColor, SpotColorId, StyleDefinition,
    StyleId, Swatch, SwatchId, SymbolDefinition, SymbolId, Transform2D, VectorPath,
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

/// Authorial registry changes participate in the same atomic transaction as
/// pasted nodes. `None` removes an entity; inverse operations capture its record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RegistryOp {
    Grid {
        id: GridId,
        value: Option<Box<GridDefinition>>,
    },
    Resource {
        id: ResourceId,
        value: Option<Box<ResourceRecord>>,
    },
    Style {
        id: StyleId,
        value: Option<Box<StyleDefinition>>,
    },
    Symbol {
        id: SymbolId,
        value: Option<Box<SymbolDefinition>>,
    },
    Swatch {
        id: SwatchId,
        value: Option<Box<Swatch>>,
    },
    Spot {
        id: SpotColorId,
        value: Option<Box<SpotColor>>,
    },
}

/// A small, explicitly represented structural mutation. Internal
/// mutation API: QML and tools never assemble these directly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DocumentOp {
    Registry(RegistryOp),
    /// Swap an editable raster surface without copying its image bytes.
    SetPixelSurface {
        object: ObjectId,
        surface: petunia_core::PixelSurfaceRef,
    },
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
    /// Remove a selection and its internal binding dependencies atomically.
    RemoveObjects {
        roots: Vec<ObjectId>,
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
    /// Set node visibility. Visibility is Document State: it changes
    /// output, export and print, so it commits like any mutation.
    SetVisibility {
        object: ObjectId,
        visible: bool,
    },
    /// Change the authorial lock flag. Unlocking remains available on locked nodes.
    SetLocked {
        object: ObjectId,
        locked: bool,
    },
    /// Replace a path or shape's authorial paint stack atomically.
    SetAppearance {
        object: ObjectId,
        appearance: Appearance,
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
type PlacedNode = (SceneNode, Option<(ParentRef, usize)>);

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
                stack.push((*child, Some((ParentRef::Object(id), index))));
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
        DocumentOp::SetPixelSurface { object, .. } => {
            let node = document
                .scene
                .get_node(*object)
                .ok_or_else(|| missing(*object))?;
            let SceneItem::PixelLayer(layer) = node.item else {
                return Err(TransactionError::InvariantViolation(
                    "surface target is not a pixel layer".into(),
                ));
            };
            Ok(vec![DocumentOp::SetPixelSurface {
                object: *object,
                surface: layer.surface,
            }])
        }
        DocumentOp::Registry(change) => Ok(vec![DocumentOp::Registry(match change {
            RegistryOp::Grid { id, .. } => RegistryOp::Grid {
                id: *id,
                value: document.grids.get(*id).copied().map(Box::new),
            },
            RegistryOp::Resource { id, .. } => RegistryOp::Resource {
                id: *id,
                value: document.resources.get(*id).cloned().map(Box::new),
            },
            RegistryOp::Style { id, .. } => RegistryOp::Style {
                id: *id,
                value: document.styles.get(*id).cloned().map(Box::new),
            },
            RegistryOp::Symbol { id, .. } => RegistryOp::Symbol {
                id: *id,
                value: document.symbols.get(*id).cloned().map(Box::new),
            },
            RegistryOp::Swatch { id, .. } => RegistryOp::Swatch {
                id: *id,
                value: document.swatches.get(*id).cloned().map(Box::new),
            },
            RegistryOp::Spot { id, .. } => RegistryOp::Spot {
                id: *id,
                value: document.spots.get(*id).cloned().map(Box::new),
            },
        })]),
        DocumentOp::InsertRoot { node, .. } => {
            Ok(vec![DocumentOp::RemoveSubtree { root: node.id }])
        }
        DocumentOp::InsertNode { node, .. } => {
            Ok(vec![DocumentOp::RemoveSubtree { root: node.id }])
        }
        DocumentOp::RemoveObjects { roots } => {
            let selected: std::collections::HashSet<_> = roots.iter().copied().collect();
            let mut roots: Vec<_> = roots
                .iter()
                .copied()
                .filter(|root| {
                    !document
                        .scene
                        .ancestors(*root)
                        .iter()
                        .any(|ancestor| selected.contains(ancestor))
                })
                .collect();
            roots.sort_by_key(|root| {
                find_parent(document, *root)
                    .map(|(_, index)| index)
                    .unwrap_or(usize::MAX)
            });
            roots.dedup();
            let mut inverse = Vec::new();
            for root in roots {
                inverse.extend(inverse_of(document, &DocumentOp::RemoveSubtree { root })?);
            }
            Ok(inverse)
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
                node.parent = parent;
                inverse.push(match parent {
                    ParentRef::Page(_) => DocumentOp::InsertRoot {
                        index,
                        node: Box::new(node),
                    },
                    ParentRef::Object(parent) => DocumentOp::InsertNode {
                        parent,
                        index,
                        node: Box::new(node),
                    },
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
        DocumentOp::SetVisibility { object, .. } => {
            let node = document
                .scene
                .get_node(*object)
                .ok_or_else(|| missing(*object))?;
            Ok(vec![DocumentOp::SetVisibility {
                object: *object,
                visible: node.visible,
            }])
        }
        DocumentOp::SetLocked { object, .. } => {
            let node = document
                .scene
                .get_node(*object)
                .ok_or_else(|| missing(*object))?;
            Ok(vec![DocumentOp::SetLocked {
                object: *object,
                locked: node.locked,
            }])
        }
        DocumentOp::SetAppearance { object, .. } => {
            let node = document
                .scene
                .get_node(*object)
                .ok_or_else(|| missing(*object))?;
            let appearance = node.item_appearance().ok_or_else(|| {
                TransactionError::InvariantViolation(format!(
                    "object {object} has no editable appearance"
                ))
            })?;
            Ok(vec![DocumentOp::SetAppearance {
                object: *object,
                appearance: appearance.clone(),
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
                SceneItem::Path(item) => Ok(vec![DocumentOp::ReplacePath {
                    object: *object,
                    path: item.path.clone(),
                }]),
                _ => Err(TransactionError::InvariantViolation(format!(
                    "object {object} is not a path"
                ))),
            }
        }
        DocumentOp::SetEffectParameter { effect, parameter } => {
            let state = effect_state(document, *effect)?;
            let parameter = match parameter {
                EffectParameter::Enabled(_) => EffectParameter::Enabled(state.enabled),
                EffectParameter::Opacity(_) => EffectParameter::Opacity(state.opacity),
                EffectParameter::BlendMode(_) => EffectParameter::BlendMode(state.blend_mode),
            };
            Ok(vec![DocumentOp::SetEffectParameter {
                effect: *effect,
                parameter,
            }])
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

fn find_parent(document: &Document, child: ObjectId) -> Option<(ParentRef, usize)> {
    let parent = document.scene.parent_of(child)?;
    let children = match parent {
        ParentRef::Page(page) => document.scene.page_roots(page)?,
        ParentRef::Object(host) => document.scene.children_of(host)?,
    };
    children
        .iter()
        .position(|id| *id == child)
        .map(|index| (parent, index))
}

struct EffectState {
    owner: ObjectId,
    enabled: bool,
    opacity: f32,
    blend_mode: BlendMode,
}
fn effect_state(
    document: &Document,
    effect: EffectId,
) -> std::result::Result<EffectState, TransactionError> {
    let mut found = None;
    let mut stack: Vec<_> = document
        .scene
        .root_lists()
        .into_iter()
        .flatten()
        .copied()
        .collect();
    let mut seen = std::collections::HashSet::new();
    while let Some(owner) = stack.pop() {
        if !seen.insert(owner) {
            continue;
        }
        let node = document
            .scene
            .get_node(owner)
            .ok_or_else(|| missing(owner))?;
        if let Some(children) = node.item.children() {
            stack.extend(children.iter().copied());
        }
        for (id, enabled, opacity, blend_mode) in node
            .geometry_effects
            .items
            .iter()
            .map(|instance| {
                (
                    instance.id,
                    instance.enabled,
                    instance.opacity,
                    instance.blend_mode,
                )
            })
            .chain(node.post_effects.items.iter().map(|instance| {
                (
                    instance.id,
                    instance.enabled,
                    instance.opacity,
                    instance.blend_mode,
                )
            }))
        {
            if id != effect {
                continue;
            }
            if found.is_some() {
                return Err(TransactionError::InvariantViolation(
                    "effect identity is ambiguous".into(),
                ));
            }
            found = Some(EffectState {
                owner,
                enabled,
                opacity,
                blend_mode,
            });
        }
    }
    found.ok_or_else(|| {
        TransactionError::UnsupportedOperation(format!("effect {effect} is not in the scene"))
    })
}
fn set_effect_parameter<T>(
    instance: &mut petunia_core::EffectInstance<T>,
    parameter: EffectParameter,
) {
    match parameter {
        EffectParameter::Enabled(value) => instance.enabled = value,
        EffectParameter::Opacity(value) => instance.opacity = value,
        EffectParameter::BlendMode(value) => instance.blend_mode = value,
    }
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
        if let DocumentOp::SetEffectParameter { effect, .. } = op {
            let owner = effect_state(&shadow, *effect)?.owner;
            if !affected.contains(&owner) {
                affected.push(owner);
            }
        }
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
    shadow
        .validate()
        .map_err(|error| TransactionError::InvariantViolation(error.to_string()))?;
    // Undo runs the per-operation inverses in reverse operation order.
    let mut inverse = Vec::new();
    for group in undo_groups.into_iter().rev() {
        inverse.extend(group);
    }
    Ok(PreparedTransaction {
        expected_revision,
        forward: request.operations,
        inverse: combine_removals(inverse),
        affected_objects: affected,
        command_id: request.command_id,
        merge_key: request.merge_key,
    })
}

// Adjacent inverse insert removals form one selection: its owned clip/mask
// sources may not be removed independently while their pasted target is live.
fn combine_removals(operations: Vec<DocumentOp>) -> Vec<DocumentOp> {
    let mut combined = Vec::new();
    let mut roots = Vec::new();
    for operation in operations {
        if let DocumentOp::RemoveSubtree { root } = operation {
            roots.push(root);
        } else {
            if !roots.is_empty() {
                combined.push(DocumentOp::RemoveObjects {
                    roots: std::mem::take(&mut roots),
                });
            }
            combined.push(operation);
        }
    }
    if !roots.is_empty() {
        combined.push(DocumentOp::RemoveObjects { roots });
    }
    combined
}

fn affected_object(op: &DocumentOp, affected: &mut Vec<ObjectId>) {
    let mut push = |id: ObjectId| {
        if !affected.contains(&id) {
            affected.push(id);
        }
    };
    match op {
        DocumentOp::SetPixelSurface { object, .. } => push(*object),
        DocumentOp::Registry(_) => {}
        DocumentOp::InsertRoot { node, .. } => {
            push(node.id);
        }
        DocumentOp::InsertNode { parent, node, .. } => {
            push(*parent);
            push(node.id);
        }
        DocumentOp::RemoveObjects { roots } => {
            for root in roots {
                push(*root);
            }
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
        DocumentOp::SetVisibility { object, .. } => push(*object),
        DocumentOp::SetLocked { object, .. } => push(*object),
        DocumentOp::SetAppearance { object, .. } => push(*object),
    }
}

fn validate_operation(
    document: &Document,
    op: &DocumentOp,
) -> std::result::Result<(), TransactionError> {
    match op {
        DocumentOp::SetPixelSurface { object, .. } => {
            let node = document
                .scene
                .get_node(*object)
                .ok_or_else(|| missing(*object))?;
            if matches!(node.item, SceneItem::PixelLayer(_)) {
                Ok(())
            } else {
                Err(TransactionError::InvariantViolation(
                    "surface target is not a pixel layer".into(),
                ))
            }
        }
        DocumentOp::Registry(change) => {
            let matches = match change {
                RegistryOp::Grid { id, value } => {
                    value.as_ref().is_none_or(|record| record.id == *id)
                }
                RegistryOp::Resource { id, value } => {
                    value.as_ref().is_none_or(|record| record.id == *id)
                }
                RegistryOp::Symbol { id, value } => {
                    value.as_ref().is_none_or(|record| record.id == *id)
                }
                RegistryOp::Swatch { id, value } => {
                    value.as_ref().is_none_or(|record| record.id == *id)
                }
                RegistryOp::Spot { id, value } => {
                    value.as_ref().is_none_or(|record| record.id == *id)
                }
                RegistryOp::Style { .. } => true,
            };
            if matches {
                Ok(())
            } else {
                Err(TransactionError::InvariantViolation(
                    "registry key differs from record identity".into(),
                ))
            }
        }
        DocumentOp::InsertRoot { index, node } => {
            let ParentRef::Page(page) = node.parent else {
                return Err(TransactionError::InvariantViolation(
                    "root must declare a page parent".into(),
                ));
            };
            let roots = document
                .scene
                .page_roots(page)
                .ok_or_else(|| TransactionError::InvariantViolation("root page missing".into()))?;
            if *index > roots.len() {
                return Err(TransactionError::InvariantViolation(
                    "root insertion index out of range".into(),
                ));
            }
            if document.scene.get_node(node.id).is_some() {
                return Err(TransactionError::InvariantViolation(format!(
                    "node {} already exists",
                    node.id
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
        DocumentOp::RemoveObjects { roots } => {
            if roots.is_empty() {
                return Err(TransactionError::InvariantViolation(
                    "remove requires roots".into(),
                ));
            }
            for root in roots {
                collect_subtree(document, *root)?;
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
        DocumentOp::SetVisibility { object, .. } | DocumentOp::SetLocked { object, .. } => {
            document
                .scene
                .get_node(*object)
                .ok_or_else(|| missing(*object))?;
            Ok(())
        }
        DocumentOp::SetAppearance { object, appearance } => {
            let node = document
                .scene
                .get_node(*object)
                .ok_or_else(|| missing(*object))?;
            if node.item_appearance().is_none() {
                return Err(TransactionError::InvariantViolation(format!(
                    "object {object} has no editable appearance"
                )));
            }
            // Group locks protect their descendants as well. An earlier
            // SetLocked in this request can explicitly unlock before painting.
            let mut current = node;
            let mut remaining = document.scene.len();
            loop {
                remaining = remaining.checked_sub(1).ok_or_else(|| {
                    TransactionError::InvariantViolation(format!(
                        "cyclic ancestry on object {object}"
                    ))
                })?;
                if current.locked {
                    return Err(TransactionError::InvariantViolation(format!(
                        "object {object} is protected by locked object {}",
                        current.id
                    )));
                }
                match current.parent {
                    ParentRef::Page(_) => break,
                    ParentRef::Object(parent) => {
                        current = document
                            .scene
                            .get_node(parent)
                            .ok_or_else(|| missing(parent))?;
                    }
                }
            }
            appearance
                .validate()
                .map_err(|error| TransactionError::InvariantViolation(error.to_string()))
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
        DocumentOp::SetEffectParameter { effect, parameter } => {
            effect_state(document, *effect)?;
            if let EffectParameter::Opacity(opacity) = parameter {
                if !opacity.is_finite() || !(0.0..=1.0).contains(opacity) {
                    return Err(TransactionError::InvariantViolation(
                        "effect opacity out of range".into(),
                    ));
                }
            }
            Ok(())
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
        DocumentOp::SetPixelSurface { object, surface } => {
            let node = document
                .scene
                .get_node_mut(object)
                .ok_or_else(|| CommitError::ApplyFailed("surface target missing".into()))?;
            let SceneItem::PixelLayer(layer) = &mut node.item else {
                return Err(CommitError::ApplyFailed(
                    "surface target is not a pixel layer".into(),
                ));
            };
            layer.surface = surface;
            Ok(())
        }
        DocumentOp::Registry(change) => {
            match change {
                RegistryOp::Grid { id, value } => {
                    if let Some(value) = value {
                        document.grids.insert(*value);
                    } else {
                        document.grids.remove(id);
                    }
                }

                RegistryOp::Resource { id, value } => {
                    if let Some(value) = value {
                        document.resources.insert(*value);
                    } else {
                        document.resources.remove(id);
                    }
                }
                RegistryOp::Style { id, value } => {
                    if let Some(value) = value {
                        document.styles.insert(id, *value);
                    } else {
                        document.styles.remove(id);
                    }
                }
                RegistryOp::Symbol { id, value } => {
                    if let Some(value) = value {
                        document.symbols.insert(*value);
                    } else {
                        document.symbols.remove(id);
                    }
                }
                RegistryOp::Swatch { id, value } => {
                    if let Some(value) = value {
                        document.swatches.insert(*value);
                    } else {
                        document.swatches.remove(id);
                    }
                }
                RegistryOp::Spot { id, value } => {
                    if let Some(value) = value {
                        document.spots.insert(*value);
                    } else {
                        document.spots.remove(id);
                    }
                }
            }
            Ok(())
        }
        DocumentOp::InsertRoot { index, node } => {
            let id = node.id;
            let ParentRef::Page(page) = node.parent else {
                return Err(CommitError::ApplyFailed("root requires page parent".into()));
            };
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
        DocumentOp::RemoveObjects { roots } => document
            .scene
            .remove_subtrees(&roots)
            .map(|_| ())
            .map_err(|error| CommitError::ApplyFailed(error.to_string())),
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
        DocumentOp::SetVisibility { object, visible } => {
            let node = document.scene.get_node_mut(object).ok_or_else(|| {
                CommitError::ApplyFailed(format!("object {object} vanished at commit"))
            })?;
            node.visible = visible;
            Ok(())
        }
        DocumentOp::SetLocked { object, locked } => {
            let node = document.scene.get_node_mut(object).ok_or_else(|| {
                CommitError::ApplyFailed(format!("object {object} vanished at commit"))
            })?;
            node.locked = locked;
            Ok(())
        }
        DocumentOp::SetAppearance { object, appearance } => {
            let node = document.scene.get_node_mut(object).ok_or_else(|| {
                CommitError::ApplyFailed(format!("object {object} vanished at commit"))
            })?;
            match &mut node.item {
                SceneItem::Path(item) => item.appearance = appearance,
                SceneItem::Shape(item) => item.appearance = appearance,
                _ => {
                    return Err(CommitError::ApplyFailed(format!(
                        "object {object} has no editable appearance"
                    )));
                }
            }
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
                SceneItem::Path(object) => {
                    object.path = path;
                    Ok(())
                }
                _ => Err(CommitError::ApplyFailed(format!(
                    "object {object} is not a path"
                ))),
            }
        }
        DocumentOp::SetEffectParameter { effect, parameter } => {
            let owner = effect_state(document, effect)
                .map_err(|error| CommitError::ApplyFailed(error.to_string()))?
                .owner;
            let node = document
                .scene
                .get_node_mut(owner)
                .ok_or_else(|| CommitError::ApplyFailed("effect owner missing".into()))?;
            if let Some(instance) = node
                .geometry_effects
                .items
                .iter_mut()
                .find(|instance| instance.id == effect)
            {
                set_effect_parameter(instance, parameter);
            } else if let Some(instance) = node
                .post_effects
                .items
                .iter_mut()
                .find(|instance| instance.id == effect)
            {
                set_effect_parameter(instance, parameter);
            }
            Ok(())
        }
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
    // Revalidate against the live state, including inverses. The staging copy
    // exists only during commit; history stores narrow operations, not documents.
    let validated = prepare_transaction(
        document,
        TransactionRequest {
            command_id: prepared.command_id,
            operations: prepared.forward.clone(),
            merge_key: prepared.merge_key.clone(),
        },
        prepared.expected_revision,
    )
    .map_err(|error| CommitError::RolledBack(error.to_string()))?;
    apply_ops_atomic(document, &validated.forward)
        .map_err(|error| CommitError::RolledBack(error.to_string()))?;
    Ok(AppliedTransaction {
        affected_objects: prepared.affected_objects.clone(),
        command_id: prepared.command_id,
        merge_key: prepared.merge_key,
        forward: prepared.forward,
        inverse: validated.inverse,
    })
}

/// Apply a whole undo/redo sequence atomically, including reference validation.
pub(crate) fn apply_ops_atomic(
    document: &mut Document,
    operations: &[DocumentOp],
) -> std::result::Result<(), EngineError> {
    let mut staged = document.clone();
    for operation in operations {
        apply_operation(&mut staged, operation.clone())
            .map_err(|error| EngineError::Execution(error.to_string()))?;
    }
    staged.validate()?;
    *document = staged;
    Ok(())
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
    fn visibility_toggle_commits_and_undoes() {
        let (mut document, _) = group_fixture();
        let node = SceneNode::new_path(
            "box",
            VectorPath::rect(0.0, 0.0, 10.0, 10.0),
            petunia_core::ParentRef::Page(document.scene.default_page()),
        );
        let id = node.id;
        let inserted = prepare_transaction(
            &document,
            request(vec![DocumentOp::InsertRoot {
                index: 0,
                node: Box::new(node),
            }]),
            DocumentRevision::GENESIS,
        )
        .expect("valid");
        commit_transaction(&mut document, inserted).expect("commits");
        let revision = DocumentRevision(1);
        let prepared = prepare_transaction(
            &document,
            request(vec![DocumentOp::SetVisibility {
                object: id,
                visible: false,
            }]),
            revision,
        )
        .expect("valid");
        commit_transaction(&mut document, prepared).expect("commits");
        assert!(!document.scene.get_node(id).expect("node").visible);
        // Undo restores the captured flag, not a default.
        let back = prepare_transaction(
            &document,
            request(vec![DocumentOp::SetVisibility {
                object: id,
                visible: true,
            }]),
            DocumentRevision(2),
        )
        .expect("valid");
        commit_transaction(&mut document, back).expect("commits");
        assert!(document.scene.get_node(id).expect("node").visible);
        // Missing objects fail validation, not the commit.
        assert!(prepare_transaction(
            &document,
            request(vec![DocumentOp::SetVisibility {
                object: ObjectId::new_v4(),
                visible: false,
            }]),
            DocumentRevision(3),
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
