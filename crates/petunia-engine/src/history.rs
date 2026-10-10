//! Linear history over committed transactions.
//!
//! Each entry links two document revisions; undo walks backward,
//! redo walks forward, and a fresh edit after undo discards the old
//! redo branch. History is session runtime state with its own memory
//! budget: pruning drops the oldest unreachable prefix and never the
//! current document.

use crate::error::{EngineError, Result};
use crate::transaction::{
    apply_ops_atomic, commit_transaction, prepare_transaction, AppliedTransaction, DocumentOp,
    DocumentRevision, MergeKey, PreparedTransaction, TransactionError, TransactionRequest,
};
use petunia_core::Document;
use serde::{Deserialize, Serialize};

/// Stable identity of a history entry for panels and localization.
/// The UI translates the identity; no localized text persists here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoryDescription {
    MoveObjects,
    SetVisibility,
    EditObjects,
    DeleteObjects,
    SetFill,
    ConvertToCurves,
    InsertObjects,
    SetTransform,
}

/// One link between two document revisions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub transaction: AppliedTransaction,
    pub before_revision: DocumentRevision,
    pub after_revision: DocumentRevision,
    pub merge_key: Option<MergeKey>,
    pub description: HistoryDescription,
}

impl HistoryEntry {
    /// Conservative payload estimate including both geometry directions.
    #[must_use]
    pub fn estimated_bytes(&self) -> u64 {
        estimate_transaction(
            &self.transaction.forward,
            &self.transaction.inverse,
            self.transaction.affected_objects.len(),
        )
    }
}

/// Count serialized nested payload without allocating another geometry buffer.
/// The multiplier and inline operation sizes account for in-memory containers
/// and spare allocation capacity; this is a retention estimate, not an allocator.
fn estimate_transaction(forward: &[DocumentOp], inverse: &[DocumentOp], affected: usize) -> u64 {
    struct Counter(u64);
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self.0.saturating_add(bytes.len() as u64);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter(0);
    if serde_json::to_writer(&mut counter, &(forward, inverse)).is_err() {
        return u64::MAX;
    }
    counter
        .0
        .saturating_mul(2)
        .saturating_add(
            ((forward.len() + inverse.len()) * std::mem::size_of::<DocumentOp>()) as u64,
        )
        .saturating_add((affected * std::mem::size_of::<petunia_core::ObjectId>()) as u64)
        .saturating_add(128)
}

/// Linear undo history with revision tracking and memory budget.
#[derive(Debug)]
pub struct History {
    entries: Vec<HistoryEntry>,
    /// Number of applied entries; `entries[position..]` is redo.
    position: usize,
    current_revision: DocumentRevision,
    saved_revision: DocumentRevision,
    highest_revision: DocumentRevision,
    soft_budget_bytes: u64,
    hard_budget_bytes: u64,
}

impl History {
    /// Start tracking at `GENESIS` with explicit budgets. Pruning may
    /// begin past the soft budget; a single entry past the hard
    /// budget is refused before commit.
    #[must_use]
    pub fn new(soft_budget_bytes: u64, hard_budget_bytes: u64) -> Self {
        Self {
            entries: Vec::new(),
            position: 0,
            current_revision: DocumentRevision::GENESIS,
            saved_revision: DocumentRevision::GENESIS,
            highest_revision: DocumentRevision::GENESIS,
            soft_budget_bytes: soft_budget_bytes.min(hard_budget_bytes),
            hard_budget_bytes,
        }
    }

    /// Current authorial revision.
    #[must_use]
    pub fn current_revision(&self) -> DocumentRevision {
        self.current_revision
    }

    /// True when authorial changes postdate the last save.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.current_revision != self.saved_revision
    }

    /// Record the current revision as saved. Undo back to it turns
    /// the document clean again without a separate dirty flag.
    pub fn save_checkpoint(&mut self) {
        self.saved_revision = self.current_revision;
    }

    /// Acknowledge the revision actually written by an asynchronous save.
    /// Editing may already have advanced the document; only that saved state
    /// becomes the checkpoint, so newer authorial changes remain dirty.
    pub fn mark_saved(&mut self, revision: DocumentRevision) -> Result<()> {
        if revision > self.highest_revision {
            return Err(EngineError::Execution(
                "saved revision was never allocated by this history".into(),
            ));
        }
        self.saved_revision = revision;
        Ok(())
    }

    /// Number of retained entries (undoable plus redoable).
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when nothing is retained.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.position > 0
    }

    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.position < self.entries.len()
    }

    /// Immutable retained entries and applied prefix, for a history panel.
    #[must_use]
    pub fn entries(&self) -> (&[HistoryEntry], usize) {
        (&self.entries, self.position)
    }

    /// Restore history entries during crash recovery so the user can
    /// undo replayed transactions.
    pub fn restore_entries(&mut self, entries: Vec<HistoryEntry>) {
        if let Some(last) = entries.last() {
            self.current_revision = last.after_revision;
        }
        for entry in &entries {
            self.highest_revision = self
                .highest_revision
                .max(entry.before_revision)
                .max(entry.after_revision);
        }
        self.entries.extend(entries);
        self.position = self.entries.len();
    }

    /// Commit a prepared transaction: check the revision, apply
    /// atomically, append (or coalesce) the entry, enforce the
    /// budget and advance the revision.
    pub fn commit(
        &mut self,
        document: &mut Document,
        prepared: PreparedTransaction,
        description: HistoryDescription,
    ) -> Result<DocumentRevision> {
        if prepared.expected_revision != self.current_revision {
            return Err(EngineError::Execution(
                TransactionError::RevisionConflict {
                    expected: prepared.expected_revision,
                    current: self.current_revision,
                }
                .to_string(),
            ));
        }
        // Public prepared DTOs can be stale or malformed; derive the actual
        // inverses against the current document before calculating retention.
        let prepared = prepare_transaction(
            document,
            TransactionRequest {
                command_id: prepared.command_id,
                operations: prepared.forward,
                merge_key: prepared.merge_key,
            },
            self.current_revision,
        )
        .map_err(|error| EngineError::Execution(error.to_string()))?;
        // Refuse oversized work before touching the document or the
        // redo branch: without safe undo there is no atomic commit.
        let cost = estimate_transaction(
            &prepared.forward,
            &prepared.inverse,
            prepared.affected_objects.len(),
        );
        if cost > self.hard_budget_bytes {
            return Err(EngineError::BudgetExceeded(format!(
                "entry needs {cost} bytes past hard budget {}",
                self.hard_budget_bytes
            )));
        }
        let after = DocumentRevision(self.highest_revision.0.checked_add(1).ok_or_else(|| {
            EngineError::Execution("document revision space exhausted".to_string())
        })?);
        let merge_key = prepared.merge_key.clone();
        let applied = commit_transaction(document, prepared)
            .map_err(|error| EngineError::Execution(error.to_string()))?;
        // Preserve redo until the document commit succeeds.
        self.entries.truncate(self.position);
        let before = self.current_revision;
        self.highest_revision = after;
        let entry = HistoryEntry {
            transaction: applied,
            before_revision: before,
            after_revision: after,
            merge_key: merge_key.clone(),
            description,
        };
        // Coalescing joins consecutive commits sharing a merge key,
        // but never rewrites across a saved revision.
        if let Some(key) = merge_key {
            if let Some(combined) = self.entries.pop_if(|last| {
                last.transaction.merge_key.as_ref() == Some(&key)
                    && last.after_revision != self.saved_revision
                    && last.description == description
                    && last.transaction.affected_objects == entry.transaction.affected_objects
                    && last.estimated_bytes().saturating_add(cost) <= self.hard_budget_bytes
            }) {
                let mut forward = combined.transaction.forward;
                forward.extend(entry.transaction.forward);
                let mut inverse = entry.transaction.inverse;
                inverse.extend(combined.transaction.inverse);
                let mut affected = combined.transaction.affected_objects;
                for id in entry.transaction.affected_objects {
                    if !affected.contains(&id) {
                        affected.push(id);
                    }
                }
                self.entries.push(HistoryEntry {
                    transaction: AppliedTransaction {
                        forward,
                        inverse,
                        affected_objects: affected,
                        command_id: entry.transaction.command_id,
                        merge_key: entry.transaction.merge_key,
                    },
                    before_revision: combined.before_revision,
                    after_revision: after,
                    merge_key: entry.merge_key,
                    description,
                });
                self.position = self.entries.len();
                self.current_revision = after;
                self.enforce_budget()?;
                return Ok(after);
            }
        }
        self.entries.push(entry);
        self.position = self.entries.len();
        self.current_revision = after;
        self.enforce_budget()?;
        Ok(after)
    }

    /// Drop the oldest unreachable prefix while the retained cost
    /// exceeds the soft budget. The current document never changes.
    fn enforce_budget(&mut self) -> Result<()> {
        let retained: u64 = self.entries.iter().map(HistoryEntry::estimated_bytes).sum();
        if retained <= self.soft_budget_bytes {
            return Ok(());
        }
        let mut freed = 0u64;
        let mut drop = 0usize;
        for entry in &self.entries {
            if retained - freed <= self.soft_budget_bytes {
                break;
            }
            // Never drop entries at or after the current position:
            // they are the reachable undo/redo path.
            if drop + 1 >= self.position {
                break;
            }
            freed += entry.estimated_bytes();
            drop += 1;
        }
        self.entries.drain(..drop);
        self.position -= drop;
        Ok(())
    }

    /// Undo the latest applied entry.
    pub fn undo(&mut self, document: &mut Document) -> Result<()> {
        let entry = self
            .entries
            .get(..self.position)
            .and_then(|window| window.last())
            .ok_or(EngineError::NothingToUndo)?;
        apply_ops_atomic(document, &entry.transaction.inverse)?;
        self.position -= 1;
        self.current_revision = entry.before_revision;
        Ok(())
    }

    /// Redo the next retained entry.
    pub fn redo(&mut self, document: &mut Document) -> Result<()> {
        let entry = self
            .entries
            .get(self.position)
            .cloned()
            .ok_or(EngineError::NothingToRedo)?;
        apply_ops_atomic(document, &entry.transaction.forward)?;
        self.position += 1;
        self.current_revision = entry.after_revision;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{prepare_transaction, CommandId, DocumentOp, TransactionRequest};
    use petunia_core::{SceneItem, SceneNode, VectorPath};

    const SOFT: u64 = 1 << 30;
    const HARD: u64 = 1 << 30;

    fn group_document() -> (Document, petunia_core::ObjectId) {
        let mut document = Document::new("history");
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

    fn insert_request(
        parent: petunia_core::ObjectId,
        merge_key: Option<&str>,
    ) -> TransactionRequest {
        TransactionRequest {
            command_id: CommandId::new_v4(),
            operations: vec![DocumentOp::InsertNode {
                parent,
                index: 0,
                node: Box::new(SceneNode::new_path(
                    "box",
                    VectorPath::rect(0.0, 0.0, 5.0, 5.0),
                    petunia_core::ParentRef::Object(parent),
                )),
            }],
            merge_key: merge_key.map(|key| MergeKey(key.to_string())),
        }
    }

    fn commit_insert(
        history: &mut History,
        document: &mut Document,
        parent: petunia_core::ObjectId,
        merge_key: Option<&str>,
    ) -> DocumentRevision {
        let revision = history.current_revision();
        let prepared = prepare_transaction(document, insert_request(parent, merge_key), revision)
            .expect("prepares");
        history
            .commit(document, prepared, HistoryDescription::InsertObjects)
            .expect("commits")
    }

    #[test]
    fn commit_undo_redo_moves_revisions_and_dirty() {
        let (mut document, parent) = group_document();
        let mut history = History::new(SOFT, HARD);
        assert!(!history.is_dirty());

        commit_insert(&mut history, &mut document, parent, None);
        assert_eq!(document.scene.len(), 2);
        assert!(history.is_dirty());
        assert_eq!(history.current_revision(), DocumentRevision(1));

        history.save_checkpoint();
        assert!(!history.is_dirty());

        history.undo(&mut document).expect("undoes");
        assert_eq!(document.scene.len(), 1);
        assert_eq!(history.current_revision(), DocumentRevision(0));
        assert!(history.is_dirty());

        history.redo(&mut document).expect("redoes");
        assert_eq!(document.scene.len(), 2);
        assert_eq!(history.current_revision(), DocumentRevision(1));
        assert!(!history.is_dirty());
    }

    #[test]
    fn stale_revision_conflicts_and_redo_truncates() {
        let (mut document, parent) = group_document();
        let mut history = History::new(SOFT, HARD);
        commit_insert(&mut history, &mut document, parent, None);
        history.undo(&mut document).expect("undoes");
        assert!(history.can_redo());

        // A request prepared against the undone revision conflicts.
        let stale =
            prepare_transaction(&document, insert_request(parent, None), DocumentRevision(1))
                .expect("prepares");
        assert!(history
            .commit(&mut document, stale, HistoryDescription::InsertObjects)
            .is_err());

        // A fresh edit discards the old redo branch.
        commit_insert(&mut history, &mut document, parent, None);
        assert!(!history.can_redo());
        assert_eq!(history.current_revision(), DocumentRevision(2));
    }

    #[test]
    fn coalescing_joins_keys_but_not_across_save() {
        let (mut document, parent) = group_document();
        let mut history = History::new(SOFT, HARD);
        for visible in [false, true] {
            let prepared = prepare_transaction(
                &document,
                TransactionRequest {
                    command_id: CommandId::new_v4(),
                    merge_key: Some(MergeKey("typing".into())),
                    operations: vec![DocumentOp::SetVisibility {
                        object: parent,
                        visible,
                    }],
                },
                history.current_revision(),
            )
            .expect("prepare");
            history
                .commit(&mut document, prepared, HistoryDescription::SetVisibility)
                .expect("commit");
        }
        assert_eq!(history.len(), 1);

        history.save_checkpoint();
        commit_insert(&mut history, &mut document, parent, Some("typing"));
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn budget_prunes_prefix_and_refuses_oversize() {
        let (mut document, parent) = group_document();
        // Soft budget holds roughly one entry: old prefixes prune.
        let mut history = History::new(700, HARD);
        commit_insert(&mut history, &mut document, parent, None);
        commit_insert(&mut history, &mut document, parent, None);
        commit_insert(&mut history, &mut document, parent, None);
        assert!(history.len() < 3);
        // The current document never changes through pruning.
        assert_eq!(document.scene.len(), 4);

        let mut tiny = History::new(SOFT, 100);
        let revision = tiny.current_revision();
        let prepared = prepare_transaction(&document, insert_request(parent, None), revision)
            .expect("prepares");
        assert!(matches!(
            tiny.commit(&mut document, prepared, HistoryDescription::InsertObjects),
            Err(EngineError::BudgetExceeded(_))
        ));
        // Refused work touches neither document nor history.
        assert_eq!(document.scene.len(), 4);
        assert!(tiny.is_empty());
    }
}
