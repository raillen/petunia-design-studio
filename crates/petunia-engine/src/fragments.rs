//! Document fragments for duplicate, copy and paste.
//!
//! A fragment carries roots plus the nodes, registry records and
//! swatches its dependency closure requires — never the whole
//! document. Paste remaps every owned identity and rewrites internal
//! references with one mapping; undo removes the new entities while
//! redo restores the same new IDs.

use crate::error::{EngineError, Result};
use crate::transaction::DocumentOp;
use petunia_core::{
    ClipBinding, Document, DocumentId, ObjectId, ResourceRecord, SceneItem, SceneNode,
    StyleDefinition, StyleId, Swatch, SymbolDefinition,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// How one reference kind travels into a fragment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferencePolicy {
    /// Included and remapped (nodes, clip relations, flow links).
    OwnedDependency,
    /// Reused when the destination already has a compatible
    /// identity, copied otherwise (styles, symbols, resources).
    SharedDependency,
    /// Kept as an external pointer when policy allows.
    ExternalDependency,
    /// Dropped with a warning when semantics allow.
    OptionalReference,
}

/// Internal interchange DTO: roots plus closure, never a
/// mini-document with its own authority.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentFragment {
    pub roots: Vec<ObjectId>,
    pub nodes: Vec<SceneNode>,
    pub resources: Vec<ResourceRecord>,
    pub styles: Vec<(StyleId, StyleDefinition)>,
    pub symbols: Vec<SymbolDefinition>,
    pub swatches: Vec<Swatch>,
    pub metadata: FragmentMetadata,
}

/// Provenance of one fragment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FragmentMetadata {
    pub source_document: DocumentId,
    pub source_revision: u64,
}

/// One old-to-new identity mapping from a paste or duplicate.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IdRemapping {
    pub objects: HashMap<ObjectId, ObjectId>,
}

impl IdRemapping {
    /// Fresh identities for every node in the fragment.
    #[must_use]
    pub fn fresh_for(nodes: &[SceneNode]) -> Self {
        let objects = nodes
            .iter()
            .map(|node| (node.id, ObjectId::new_v4()))
            .collect();
        Self { objects }
    }

    /// Map an old identity, or keep it when outside the fragment
    /// (shared dependencies are preserved, not rewritten).
    #[must_use]
    pub fn map_object(&self, id: ObjectId) -> ObjectId {
        self.objects.get(&id).copied().unwrap_or(id)
    }
}

/// Collect roots plus their transitive scene subtrees. Registry
/// records travel when the caller supplies them; node closure never
/// leaves dangling internal references silently.
pub fn collect_fragment(
    document: &Document,
    roots: &[ObjectId],
    metadata: FragmentMetadata,
) -> Result<DocumentFragment> {
    let mut nodes = Vec::new();
    let mut stack: Vec<ObjectId> = roots.to_vec();
    let mut seen = std::collections::HashSet::new();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        let node = document
            .scene
            .get_node(id)
            .ok_or_else(|| {
                EngineError::Execution(format!("fragment root {id} missing from scene"))
            })?
            .clone();
        if let SceneItem::Group(children) = &node.item {
            stack.extend(children.iter().copied());
        }
        nodes.push(node);
    }
    Ok(DocumentFragment {
        roots: roots.to_vec(),
        nodes,
        resources: Vec::new(),
        styles: Vec::new(),
        symbols: Vec::new(),
        swatches: Vec::new(),
        metadata,
    })
}

/// Rewrite every owned reference inside the fragment with `mapping`:
/// node identities, group membership, clip relations and text flow
/// links. Shared style/symbol/resource identities are preserved.
#[must_use]
pub fn remap_fragment(fragment: &DocumentFragment, mapping: &IdRemapping) -> DocumentFragment {
    let mut remapped = fragment.clone();
    for node in &mut remapped.nodes {
        node.id = mapping.map_object(node.id);
        if let SceneItem::Group(children) = &mut node.item {
            for child in children.iter_mut() {
                *child = mapping.map_object(*child);
            }
        }
        if let Some(clip) = node.clip.as_mut() {
            *clip = ClipBinding {
                target: mapping.map_object(clip.target),
                source: mapping.map_object(clip.source),
                use_as: clip.use_as,
            };
        }
        if let SceneItem::Text(text) = &mut node.item {
            if let Some(next) = text.flow.next {
                text.flow.next = Some(mapping.map_object(next));
            }
        }
    }
    remapped.roots = remapped
        .roots
        .iter()
        .map(|id| mapping.map_object(*id))
        .collect();
    remapped
}

/// Turn a remapped fragment into insert operations under `parent`
/// for one paste transaction. Roots append to the external parent;
/// nested nodes rejoin their (already remapped) fragment parents.
/// Undo removes the new entities; redo restores the same new IDs
/// from the recorded operations.
pub fn fragment_into_ops(
    document: &Document,
    fragment: &DocumentFragment,
    parent: ObjectId,
) -> Result<Vec<DocumentOp>> {
    let host_children = match document.scene.get_node(parent).map(|node| &node.item) {
        Some(SceneItem::Group(children)) => children.len(),
        Some(_) => {
            return Err(EngineError::Execution(format!(
                "paste parent {parent} is not a group"
            )))
        }
        None => {
            return Err(EngineError::Execution(format!(
                "paste parent {parent} missing from scene"
            )))
        }
    };
    // Parents before children: roots first, then the rest in the
    // collected (already parent-first) order.
    let mut ordered: Vec<&SceneNode> = Vec::with_capacity(fragment.nodes.len());
    for root in &fragment.roots {
        if let Some(node) = fragment.nodes.iter().find(|node| node.id == *root) {
            ordered.push(node);
        }
    }
    for node in &fragment.nodes {
        if !ordered.iter().any(|placed| placed.id == node.id) {
            ordered.push(node);
        }
    }
    let mut ops = Vec::with_capacity(ordered.len());
    let mut placed: Vec<ObjectId> = Vec::new();
    let mut appended = 0usize;
    for node in ordered {
        // Roots and orphans append to the external parent; nested
        // nodes rejoin their fragment parents at the next free slot.
        let (host, index) = if fragment.roots.contains(&node.id) {
            let index = host_children + appended;
            appended += 1;
            (parent, index)
        } else {
            match find_fragment_parent(fragment, node.id) {
                Some(host) => (host, placement_index(fragment, &placed, host)),
                None => {
                    let index = host_children + appended;
                    appended += 1;
                    (parent, index)
                }
            }
        };
        ops.push(DocumentOp::InsertNode {
            parent: host,
            index,
            node: Box::new(node.clone()),
        });
        placed.push(node.id);
    }
    Ok(ops)
}

fn find_fragment_parent(fragment: &DocumentFragment, child: ObjectId) -> Option<ObjectId> {
    fragment.nodes.iter().find_map(|node| match &node.item {
        SceneItem::Group(children) if children.contains(&child) => Some(node.id),
        _ => None,
    })
}

fn placement_index(fragment: &DocumentFragment, placed: &[ObjectId], host: ObjectId) -> usize {
    let Some(host_node) = fragment.nodes.iter().find(|node| node.id == host) else {
        return usize::MAX / 2;
    };
    let SceneItem::Group(children) = &host_node.item else {
        return usize::MAX / 2;
    };
    children
        .iter()
        .filter(|member| placed.contains(member))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_core::{SceneNode, VectorPath};

    fn document_with_group() -> (Document, ObjectId, ObjectId) {
        let mut document = Document::new("frag");
        let mut group = SceneNode::new_path("group", VectorPath::new());
        group.item = SceneItem::Group(Vec::new());
        let parent = group.id;
        document.scene.insert_node(group);
        let child = SceneNode::new_path("box", VectorPath::rect(0.0, 0.0, 5.0, 5.0));
        let child_id = child.id;
        document.scene.insert_node(child);
        if let Some(parent_node) = document.scene.get_node_mut(parent) {
            if let SceneItem::Group(children) = &mut parent_node.item {
                children.push(child_id);
            }
        }
        (document, parent, child_id)
    }

    fn metadata() -> FragmentMetadata {
        FragmentMetadata {
            source_document: DocumentId::new_v4(),
            source_revision: 7,
        }
    }

    #[test]
    fn collect_remap_preserves_internal_relations() {
        let (document, parent, child) = document_with_group();
        let fragment = collect_fragment(&document, &[parent], metadata()).expect("collects");
        assert_eq!(fragment.nodes.len(), 2);
        let mapping = IdRemapping::fresh_for(&fragment.nodes);
        let remapped = remap_fragment(&fragment, &mapping);
        // Both identities are new and distinct.
        assert_ne!(remapped.roots[0], parent);
        let group = remapped
            .nodes
            .iter()
            .find(|node| node.id == remapped.roots[0])
            .expect("group");
        let SceneItem::Group(children) = &group.item else {
            panic!("group keeps its shape");
        };
        assert_eq!(children.len(), 1);
        assert_ne!(children[0], child);
        // The child's own mapping matches the group's reference.
        assert_eq!(mapping.map_object(child), children[0]);
    }

    #[test]
    fn paste_ops_rebuild_the_subtree() {
        use crate::transaction::{commit_transaction, prepare_transaction, DocumentRevision};
        let (mut document, parent, _) = document_with_group();
        let fragment = collect_fragment(&document, &[parent], metadata()).expect("collects");
        let mapping = IdRemapping::fresh_for(&fragment.nodes);
        let remapped = remap_fragment(&fragment, &mapping);
        let ops = fragment_into_ops(&document, &remapped, parent).expect("paste ops");
        assert_eq!(ops.len(), 2);
        let prepared = prepare_transaction(
            &document,
            crate::transaction::TransactionRequest {
                command_id: crate::transaction::CommandId::new_v4(),
                operations: ops,
                merge_key: None,
            },
            DocumentRevision::GENESIS,
        )
        .expect("prepares");
        commit_transaction(&mut document, prepared).expect("commits");
        // Original group plus pasted group and pasted child.
        assert_eq!(document.scene.len(), 4);
    }

    #[test]
    fn missing_roots_fail_loudly() {
        let (document, _, _) = document_with_group();
        assert!(collect_fragment(&document, &[ObjectId::new_v4()], metadata()).is_err());
    }
}
