//! Reusable structure: symbol definitions, instances and overrides.
//!
//! A symbol reuses structure under a stable identity while instances
//! reference it without copying the subtree. Nesting is allowed;
//! dependency cycles are rejected. Overrides are typed and never
//! structural in v0.1; independent structure uses Detach Symbol.

use crate::error::{CoreError, Result};
use crate::id::{ObjectId, ResourceId, SymbolId};
use crate::paint::ColorSource;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

/// One reusable authorial structure: its own node subtree plus the
/// roots addressing it. Internal object IDs stay global to the
/// document; evaluation derives instance subtrees without copying
/// them into the scene.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolDefinition {
    pub id: SymbolId,
    pub name: String,
    pub roots: Vec<ObjectId>,
    /// Authorial subtree. Roots address entries here; every other
    /// node parents an object inside this same map, never a page.
    pub nodes: BTreeMap<ObjectId, crate::scene::SceneNode>,
}

impl SymbolDefinition {
    /// Names must be non-empty; roots must be unique entries of
    /// `nodes`; every non-root node must parent an object inside the
    /// same map, since definitions are page-less.
    pub fn new(
        name: impl Into<String>,
        roots: Vec<ObjectId>,
        nodes: BTreeMap<ObjectId, crate::scene::SceneNode>,
    ) -> Result<Self> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(CoreError::InvariantViolation(
                "symbol definition needs a name".to_string(),
            ));
        }
        let unique: HashSet<ObjectId> = roots.iter().copied().collect();
        if unique.len() != roots.len() {
            return Err(CoreError::InvariantViolation(
                "symbol roots must not repeat".to_string(),
            ));
        }
        for root in &roots {
            if !nodes.contains_key(root) {
                return Err(CoreError::InvariantViolation(format!(
                    "symbol root {root} is not in the definition subtree"
                )));
            }
        }
        for (id, node) in &nodes {
            match node.parent {
                crate::scene::ParentRef::Page(_) => {
                    if !roots.contains(id) {
                        return Err(CoreError::InvariantViolation(format!(
                            "symbol node {id} parents a page instead of the definition"
                        )));
                    }
                }
                crate::scene::ParentRef::Object(host) => {
                    if !nodes.contains_key(&host) {
                        return Err(CoreError::InvariantViolation(format!(
                            "symbol node {id} parents {host} outside the definition"
                        )));
                    }
                }
            }
        }
        Ok(Self {
            id: SymbolId::new_v4(),
            name,
            roots,
            nodes,
        })
    }

    /// Definitions referenced by nested instances inside this
    /// subtree, for dependency-cycle validation.
    #[must_use]
    pub fn nested_definitions(&self) -> Vec<SymbolId> {
        let mut out = Vec::new();
        for node in self.nodes.values() {
            if let crate::scene::SceneItem::SymbolInstance(instance) = &node.item {
                if !out.contains(&instance.definition) {
                    out.push(instance.definition);
                }
            }
        }
        out
    }
}

/// Typed non-structural override payloads: text content, paint,
/// visibility and resource swaps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SymbolOverrideValue {
    TextContent(String),
    Paint(ColorSource),
    Visibility(bool),
    Resource(ResourceId),
}

/// One override targets a stable object inside the definition by ID,
/// never by textual property path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolOverride {
    pub target: ObjectId,
    pub value: SymbolOverrideValue,
}

/// A live reference to a definition plus its overrides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolInstance {
    pub definition: SymbolId,
    pub overrides: Vec<SymbolOverride>,
}

/// Registry of definitions by identity.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SymbolRegistry {
    #[serde(deserialize_with = "crate::serialization::deserialize_unique_btree_map")]
    definitions: BTreeMap<SymbolId, SymbolDefinition>,
}

impl SymbolRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            definitions: BTreeMap::new(),
        }
    }

    /// Insert or replace a definition.
    pub fn insert(&mut self, definition: SymbolDefinition) {
        self.definitions.insert(definition.id, definition);
    }

    /// Look up a definition by identity.
    #[must_use]
    pub fn get(&self, id: SymbolId) -> Option<&SymbolDefinition> {
        self.definitions.get(&id)
    }

    /// Iterate definitions in deterministic order; traversal only.
    pub fn iter(&self) -> impl Iterator<Item = (SymbolId, &SymbolDefinition)> {
        self.definitions
            .iter()
            .map(|(id, definition)| (*id, definition))
    }

    /// Number of tracked definitions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// True when no definition is tracked.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}

/// True when `(definition, nested definitions)` edges contain a
/// dependency cycle. Nested symbols compose; `A → B → A` is rejected
/// before any transaction commits it.
#[must_use]
pub fn has_symbol_cycle(edges: &[(SymbolId, Vec<SymbolId>)]) -> bool {
    let graph: BTreeMap<SymbolId, &[SymbolId]> = edges
        .iter()
        .map(|(id, nested)| (*id, nested.as_slice()))
        .collect();
    for (root, _) in edges {
        let mut stack = vec![*root];
        let mut seen = HashSet::new();
        while let Some(current) = stack.pop() {
            if current == *root && seen.contains(&current) {
                return true;
            }
            if !seen.insert(current) {
                continue;
            }
            if let Some(nested) = graph.get(&current) {
                stack.extend(nested.iter().copied());
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(id: ObjectId) -> crate::scene::SceneNode {
        crate::scene::SceneNode {
            id,
            name: "leaf".to_string(),
            parent: crate::scene::ParentRef::Object(ObjectId::new_v4()),
            visible: true,
            locked: false,
            transform: crate::Transform2D::IDENTITY,
            opacity: 1.0,
            clip: None,
            mask: None,
            geometry_effects: Default::default(),
            post_effects: Default::default(),
            item: crate::scene::SceneItem::Group(Vec::new()),
        }
    }

    #[test]
    fn definition_rejects_blank_names_and_duplicate_roots() {
        let root = ObjectId::new_v4();
        let mut nodes = BTreeMap::new();
        nodes.insert(root, leaf(root));
        // The leaf parents an outside object: fix it to a page root.
        nodes.get_mut(&root).expect("leaf").parent =
            crate::scene::ParentRef::Page(crate::PageId::new_v4());
        assert!(SymbolDefinition::new("Button", vec![root], nodes.clone()).is_ok());
        assert!(SymbolDefinition::new("  ", vec![root], nodes.clone()).is_err());
        assert!(SymbolDefinition::new("Button", vec![root, root], nodes.clone()).is_err());
        assert!(SymbolDefinition::new("Button", vec![ObjectId::new_v4()], nodes).is_err());
    }

    #[test]
    fn definition_lists_nested_definitions() {
        use crate::scene::SceneItem;
        use crate::symbols::SymbolInstance;
        let root = ObjectId::new_v4();
        let mut nodes = BTreeMap::new();
        let inner = SymbolId::new_v4();
        nodes.insert(
            root,
            crate::scene::SceneNode {
                id: root,
                name: "host".to_string(),
                parent: crate::scene::ParentRef::Page(crate::PageId::new_v4()),
                visible: true,
                locked: false,
                transform: crate::Transform2D::IDENTITY,
                opacity: 1.0,
                clip: None,
                mask: None,
                geometry_effects: Default::default(),
                post_effects: Default::default(),
                item: SceneItem::SymbolInstance(SymbolInstance {
                    definition: inner,
                    overrides: Vec::new(),
                }),
            },
        );
        let definition =
            SymbolDefinition::new("Button", vec![root], nodes).expect("valid definition");
        assert_eq!(definition.nested_definitions(), vec![inner]);
    }

    #[test]
    fn cycle_detection_blocks_circular_nesting() {
        let a = SymbolId::new_v4();
        let b = SymbolId::new_v4();
        let c = SymbolId::new_v4();
        assert!(!has_symbol_cycle(&[
            (a, vec![b]),
            (b, vec![c]),
            (c, vec![])
        ]));
        assert!(has_symbol_cycle(&[(a, vec![b]), (b, vec![a])]));
        assert!(has_symbol_cycle(&[(a, vec![a])]));
    }

    #[test]
    fn instance_serialization_round_trip() {
        let instance = SymbolInstance {
            definition: SymbolId::new_v4(),
            overrides: vec![SymbolOverride {
                target: ObjectId::new_v4(),
                value: SymbolOverrideValue::Visibility(false),
            }],
        };
        let json = serde_json::to_string(&instance).expect("serializable");
        let back: SymbolInstance = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(back, instance);
    }
}
