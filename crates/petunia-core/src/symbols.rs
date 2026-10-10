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

/// One reusable authorial structure. Internal object IDs stay global
/// to the document; evaluation derives instance subtrees.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolDefinition {
    pub id: SymbolId,
    pub name: String,
    pub roots: Vec<ObjectId>,
}

impl SymbolDefinition {
    /// Names must be non-empty; roots must hold no duplicates.
    pub fn new(name: impl Into<String>, roots: Vec<ObjectId>) -> Result<Self> {
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
        Ok(Self {
            id: SymbolId::new_v4(),
            name,
            roots,
        })
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

    #[test]
    fn definition_rejects_blank_names_and_duplicate_roots() {
        let root = ObjectId::new_v4();
        assert!(SymbolDefinition::new("Button", vec![root]).is_ok());
        assert!(SymbolDefinition::new("  ", vec![root]).is_err());
        assert!(SymbolDefinition::new("Button", vec![root, root]).is_err());
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
