//! Append-only recovery journal.
//!
//! The journal records checkpoints and applied revisions as JSON
//! lines; recovery replays from the last checkpoint. It never
//! decides alone what the recovery store may delete: lifecycle
//! stays independent from history pruning.

use crate::error::{EngineError, Result};
use crate::transaction::DocumentRevision;
use serde::{Deserialize, Serialize};

/// One journal line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub seq: u64,
    pub revision: DocumentRevision,
    pub operation: JournalOperation,
}

/// Journaled operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JournalOperation {
    /// A clean snapshot exists at this revision.
    SaveCheckpoint { snapshot_ref: String },
    /// A transaction committed at this revision.
    TransactionApplied { description: String },
    /// An autosave snapshot for crash recovery.
    Autosave { snapshot_ref: String },
}

/// In-memory journal with file append and replay. Sequence numbers
/// are assigned on append and never reused.
#[derive(Debug, Default)]
pub struct Journal {
    entries: Vec<JournalEntry>,
}

impl Journal {
    /// Empty journal.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Append one operation, assigning the next sequence number.
    pub fn append(&mut self, revision: DocumentRevision, operation: JournalOperation) {
        let seq = self.entries.len() as u64;
        self.entries.push(JournalEntry {
            seq,
            revision,
            operation,
        });
    }

    /// All entries in sequence order.
    #[must_use]
    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }

    /// Latest checkpoint snapshot reference, if any.
    #[must_use]
    pub fn last_checkpoint(&self) -> Option<&str> {
        self.entries
            .iter()
            .rev()
            .find_map(|entry| match &entry.operation {
                JournalOperation::SaveCheckpoint { snapshot_ref } => Some(snapshot_ref.as_str()),
                _ => None,
            })
    }

    /// Persist all entries as JSON lines, replacing any previous file.
    pub fn persist(&self, path: &std::path::Path) -> Result<()> {
        let mut text = String::new();
        for entry in &self.entries {
            let line = serde_json::to_string(entry).map_err(|error| {
                EngineError::Execution(format!("journal encode failed: {error}"))
            })?;
            text.push_str(&line);
            text.push('\n');
        }
        std::fs::write(path, text)
            .map_err(|error| EngineError::Execution(format!("journal write failed: {error}")))
    }

    /// Replay a journal file, validating sequence order.
    pub fn replay(path: &std::path::Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| EngineError::Execution(format!("journal read failed: {error}")))?;
        let mut journal = Self::new();
        for (index, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let entry: JournalEntry = serde_json::from_str(line).map_err(|error| {
                EngineError::Execution(format!("journal line {} corrupt: {error}", index + 1))
            })?;
            if entry.seq != journal.entries.len() as u64 {
                return Err(EngineError::Execution(format!(
                    "journal sequence break at line {}",
                    index + 1
                )));
            }
            journal.entries.push(entry);
        }
        Ok(journal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("petunia-journal-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir.join(name)
    }

    #[test]
    fn journal_round_trip_and_checkpoint_lookup() {
        let mut journal = Journal::new();
        journal.append(
            DocumentRevision(3),
            JournalOperation::SaveCheckpoint {
                snapshot_ref: "snap-3.ptnd".to_string(),
            },
        );
        journal.append(
            DocumentRevision(4),
            JournalOperation::TransactionApplied {
                description: "MoveObjects".to_string(),
            },
        );
        assert_eq!(
            journal.last_checkpoint(),
            Some("snap-3.ptnd"),
            "{journal:?}"
        );
        let path = scratch("journal.jsonl");
        journal.persist(&path).expect("persists");
        let back = Journal::replay(&path).expect("replays");
        assert_eq!(back.entries(), journal.entries());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn broken_sequence_is_rejected() {
        let path = scratch("broken.jsonl");
        let good = serde_json::to_string(&JournalEntry {
            seq: 0,
            revision: DocumentRevision(1),
            operation: JournalOperation::TransactionApplied {
                description: "MoveObjects".to_string(),
            },
        })
        .expect("encodes");
        let skipped = serde_json::to_string(&JournalEntry {
            seq: 5,
            revision: DocumentRevision(2),
            operation: JournalOperation::TransactionApplied {
                description: "MoveObjects".to_string(),
            },
        })
        .expect("encodes");
        std::fs::write(&path, format!("{good}\n{skipped}\n")).expect("writes");
        assert!(Journal::replay(&path).is_err());
        let _ = std::fs::remove_file(&path);
    }
}
