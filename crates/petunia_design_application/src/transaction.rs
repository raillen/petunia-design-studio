//! Interactive transaction: preview on a working copy, commit atomically,
//! cancel with exactness (F-01, 09.3).
//!
//! Tools must not `submit_command` on every pointer move (which flooded undo
//! with N entries). Instead: `begin` clones the document, `update` stages
//! commands on the working copy for overlay preview, `commit` pushes one
//! combined `ChangeSet` to `History`, `cancel` discards without touching the
//! real document or history.

use petunia_design_document::{ChangeSet, Document};
use petunia_design_foundation::PetuniaError;

use crate::commands::{self, CommandRequest};
use crate::history::History;

/// One interactive gesture staged off-document.
#[derive(Debug)]
pub struct Transaction {
    /// Working copy receiving staged commands.
    working: Document,
    /// Combined staged changes in application order.
    staged: ChangeSet,
    /// Human label for history panels (e.g. `"Move"`, `"Resize"`).
    label: String,
}

impl Transaction {
    /// Begins a transaction from the live document snapshot.
    #[must_use]
    pub fn begin(document: &Document, label: impl Into<String>) -> Self {
        Self {
            working: document.clone(),
            staged: ChangeSet::empty(),
            label: label.into(),
        }
    }

    /// Stages one command on the working copy, accumulating its changes.
    /// Returns the command's incremental `ChangeSet` for overlay preview.
    pub fn update(&mut self, request: &CommandRequest) -> Result<ChangeSet, PetuniaError> {
        let delta = commands::execute(&mut self.working, request)?;
        self.staged.extend(delta.clone());
        Ok(delta)
    }

    /// Read-only view of the working copy for overlay preview.
    #[must_use]
    pub fn preview(&self) -> &Document {
        &self.working
    }

    /// Combined staged changes so far.
    #[must_use]
    pub fn staged(&self) -> &ChangeSet {
        &self.staged
    }

    /// Human label for this gesture.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// True when nothing was staged (commit would be a NoOp).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.staged.is_empty()
    }

    /// Commits atomically: replaces the live document with the working copy
    /// and records one combined entry. Empty transactions are a NoOp and
    /// leave history untouched.
    pub fn commit(self, document: &mut Document, history: &mut History) {
        if self.staged.is_empty() {
            return;
        }
        *document = self.working;
        history.record(self.staged);
    }

    /// Cancels: discards the working copy. The live document and history are
    /// untouched, so cancel is exactly the pre-gesture state.
    pub fn cancel(self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{Command, CommandRequest};
    use petunia_design_foundation::IdGenerator;

    fn test_doc() -> (Document, IdGenerator, petunia_design_foundation::SurfaceId) {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let surface = gen.next_surface();
        {
            let mut mutator = petunia_design_document::DocumentMutator::new(&mut doc);
            mutator.add_surface(surface, "Page").unwrap();
        }
        (doc, gen, surface)
    }

    #[test]
    fn preview_does_not_touch_live_document_until_commit() {
        let (mut doc, mut gen, surface) = test_doc();
        let obj = gen.next_object();
        let mut tx = Transaction::begin(&doc, "Create");
        tx.update(&CommandRequest::new(Command::CreateObject {
            surface,
            id: obj,
            name: "Box".to_string(),
        }))
        .unwrap();
        // Live doc untouched during preview.
        assert!(doc.find_object(obj).is_none());
        assert!(tx.preview().find_object(obj).is_some());
        let mut history = History::new(100);
        tx.commit(&mut doc, &mut history);
        assert!(doc.find_object(obj).is_some());
        assert_eq!(history.undo_len(), 1);
    }

    #[test]
    fn cancel_leaves_document_and_history_untouched() {
        let (mut doc, mut gen, surface) = test_doc();
        let obj = gen.next_object();
        let mut tx = Transaction::begin(&doc, "Create");
        tx.update(&CommandRequest::new(Command::CreateObject {
            surface,
            id: obj,
            name: "Box".to_string(),
        }))
        .unwrap();
        let history = History::new(100);
        tx.cancel();
        assert!(doc.find_object(obj).is_none());
        assert_eq!(history.undo_len(), 0);
        let _ = &mut doc;
    }

    #[test]
    fn empty_commit_is_noop() {
        let (mut doc, _gen, _surface) = test_doc();
        let tx = Transaction::begin(&doc, "Empty");
        let mut history = History::new(100);
        tx.commit(&mut doc, &mut history);
        assert_eq!(history.undo_len(), 0);
    }
}
