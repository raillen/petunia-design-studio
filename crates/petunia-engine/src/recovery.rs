//! Crash recovery sessions: checkpoint plus journal replay.
//!
//! Layout under the application-private recovery root:
//!
//! ```text
//! RecoveryRoot/<DocumentId>/
//! ├── metadata.json
//! ├── checkpoint.ptnd
//! └── journal.bin
//! ```
//!
//! A checkpoint is a validated PTND package; the journal holds the
//! framed transactions committed after it. Replay loads the
//! checkpoint, skips stale records at or below its revision, then
//! applies an exact `revision + 1` chain through the normal commit
//! lane. The first gap, failure or corrupt record stops at the last
//! consistent state; recovery files are never deleted by a failed
//! replay.

use crate::error::{EngineError, Result};
use crate::journal::{
    decode_bytes, encode_frame, sync_file, verify_header, Journal, JournalLimits, JournalOperation,
    JournalRecord, ReplayStop,
};
use crate::ptnd::{atomic_write, load_document, save_document};
use crate::transaction::{
    commit_transaction, prepare_transaction, CommandId, DocumentOp, DocumentRevision,
    TransactionRequest,
};
use petunia_core::{Document, DocumentId, ResourceId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Bounds for one recovery session.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RecoveryLimits {
    pub journal: JournalLimits,
    pub checkpoint: crate::ptnd::PackageLimits,
    /// Largest accepted blob payload per transaction record in bytes.
    pub max_blob_bytes: u64,
}

impl Default for RecoveryLimits {
    fn default() -> Self {
        Self {
            journal: JournalLimits::default(),
            checkpoint: crate::ptnd::PackageLimits::default(),
            max_blob_bytes: 256 << 20,
        }
    }
}

/// Recovery metadata: which document, which revisions, and whether
/// the last shutdown was clean. Never holds secrets or external paths
/// beyond the session directory itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RecoveryMetadata {
    format: String,
    schema: u16,
    document_id: String,
    base_revision: u64,
    latest_revision: u64,
    journal_len: u64,
    updated_unix_ms: u64,
    clean_shutdown: bool,
}

/// Outcome of a recovery attempt.
#[derive(Debug)]
pub struct RecoveryReport {
    pub document: Document,
    pub blobs: BTreeMap<ResourceId, Vec<u8>>,
    pub checkpoint_revision: DocumentRevision,
    pub applied: u64,
    pub latest_revision: DocumentRevision,
    /// Why replay stopped; `CleanEnd` means the journal ran dry.
    pub stop: ReplayStop,
}

fn recovery(message: impl Into<String>) -> EngineError {
    EngineError::Recovery(message.into())
}

fn unix_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or(0)
}

/// One document's recovery session rooted at `root/<DocumentId>`.
#[derive(Debug, Clone)]
pub struct RecoverySession {
    dir: PathBuf,
    document_id: DocumentId,
    limits: RecoveryLimits,
}

impl RecoverySession {
    /// Open (creating) the session directory with restrictive
    /// permissions where the platform supports them.
    pub fn open(root: &Path, document_id: DocumentId, limits: RecoveryLimits) -> Result<Self> {
        let dir = root.join(document_id.to_string());
        create_private_dir(&dir)?;
        create_private_dir(root)?;
        Ok(Self {
            dir,
            document_id,
            limits,
        })
    }

    fn checkpoint_path(&self) -> PathBuf {
        self.dir.join("checkpoint.ptnd")
    }

    fn journal_path(&self) -> PathBuf {
        self.dir.join("journal.bin")
    }

    fn metadata_path(&self) -> PathBuf {
        self.dir.join("metadata.json")
    }

    /// Write a validated checkpoint, rotate the journal down to its
    /// marker, and record the new base revision. The old checkpoint
    /// stays live until the new bytes prove themselves; the journal
    /// truncates only afterwards.
    pub fn write_checkpoint(
        &self,
        document: &Document,
        revision: DocumentRevision,
        blobs: &BTreeMap<ResourceId, Vec<u8>>,
    ) -> Result<()> {
        if document.id != self.document_id {
            return Err(recovery(format!(
                "checkpoint identifies {}, session holds {}",
                document.id, self.document_id,
            )));
        }
        let bytes = save_document(document, blobs, &self.limits.checkpoint)?;
        // Prove the bytes before they replace anything.
        let (proved, _) = load_document(&bytes, &self.limits.checkpoint)?;
        if proved.id != document.id {
            return Err(recovery("checkpoint round-trip changed identity"));
        }
        crate::ptnd::save_atomic(&self.checkpoint_path(), &bytes)?;
        // Journal rotation: exactly one marker, atomically placed.
        let marker = JournalRecord {
            seq: 0,
            revision,
            operation: JournalOperation::Checkpoint {
                snapshot_ref: "checkpoint.ptnd".to_string(),
                content_hash: Some(format!("crc32:{:08x}", crc32fast::hash(&bytes))),
            },
        };
        let mut encoded = Journal::header().to_vec();
        encoded.extend_from_slice(&encode_frame(&marker, &self.limits.journal)?);
        atomic_write(&self.journal_path(), &encoded, "journal")?;
        self.write_metadata(RecoveryMetadata {
            format: "petunia-recovery".to_string(),
            schema: 1,
            document_id: self.document_id.to_string(),
            base_revision: revision.0,
            latest_revision: revision.0,
            journal_len: 1,
            updated_unix_ms: unix_millis(),
            clean_shutdown: false,
        })?;
        Ok(())
    }

    /// Journal one committed transaction. The revision must continue
    /// the recorded chain exactly; gaps fail loudly at append time.
    pub fn append_transaction(
        &self,
        revision: DocumentRevision,
        ops: Vec<DocumentOp>,
        description: String,
        blobs: BTreeMap<ResourceId, Vec<u8>>,
    ) -> Result<()> {
        let metadata = self.read_metadata()?.ok_or_else(|| {
            recovery("no checkpoint yet; write one before journaling transactions")
        })?;
        if metadata.document_id != self.document_id.to_string() {
            return Err(recovery("recovery metadata identifies another document"));
        }
        if revision.0 != metadata.latest_revision + 1 {
            return Err(recovery(format!(
                "journal expects revision {}, got {}",
                metadata.latest_revision + 1,
                revision.0,
            )));
        }
        let blob_bytes: u64 = blobs.values().map(|blob| blob.len() as u64).sum();
        if blob_bytes > self.limits.max_blob_bytes {
            return Err(recovery(format!(
                "transaction blobs of {blob_bytes} bytes exceed limit {}",
                self.limits.max_blob_bytes
            )));
        }
        let record = JournalRecord {
            seq: metadata.journal_len,
            revision,
            operation: JournalOperation::TransactionApplied {
                description,
                ops,
                blobs,
            },
        };
        let frame = encode_frame(&record, &self.limits.journal)?;
        let path = self.journal_path();
        if !path.exists() {
            return Err(recovery("journal file missing; rewrite a checkpoint first"));
        }
        verify_header(&path)?;
        {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(&path)
                .map_err(|error| recovery(format!("journal open failed: {error}")))?;
            file.write_all(&frame)
                .map_err(|error| recovery(format!("journal append failed: {error}")))?;
            file.flush()
                .map_err(|error| recovery(format!("journal flush failed: {error}")))?;
        }
        self.write_metadata(RecoveryMetadata {
            latest_revision: revision.0,
            journal_len: metadata.journal_len + 1,
            updated_unix_ms: unix_millis(),
            clean_shutdown: false,
            ..metadata
        })?;
        Ok(())
    }

    /// Sync the journal file to durable storage. Call on policy
    /// points: close, document switch, save and critical commits.
    pub fn sync(&self) -> Result<()> {
        sync_file(&self.journal_path())
    }

    /// Recover the document: checkpoint plus the confirmed journal
    /// prefix. Returns `None` when a clean shutdown left nothing to
    /// recover. Failed replays preserve every recovery file.
    pub fn recover(&self) -> Result<Option<RecoveryReport>> {
        let Some(metadata) = self.read_metadata()? else {
            return Ok(None);
        };
        if metadata.document_id != self.document_id.to_string() {
            return Err(recovery("recovery metadata identifies another document"));
        }
        let checkpoint_bytes = std::fs::read(self.checkpoint_path())
            .map_err(|_| recovery("checkpoint.ptnd is missing; nothing consistent to recover"))?;
        let (mut document, mut blobs) = load_document(&checkpoint_bytes, &self.limits.checkpoint)?;
        if document.id != self.document_id {
            return Err(recovery("checkpoint identifies another document"));
        }
        let replay = match std::fs::read(self.journal_path()) {
            Err(_) => crate::journal::JournalReplay {
                records: Vec::new(),
                stop: ReplayStop::CleanEnd,
            },
            Ok(bytes) => decode_bytes(&bytes, &self.limits.journal)?,
        };
        // The checkpoint file is authoritative for its own revision;
        // anything at or below it in the journal is already inside.
        let mut current = DocumentRevision(metadata.base_revision);
        let mut applied = 0u64;
        let mut stop = replay.stop.clone();
        for record in &replay.records {
            if record.revision.0 <= current.0 {
                continue;
            }
            let JournalOperation::TransactionApplied {
                description: _,
                ops,
                blobs: record_blobs,
            } = &record.operation
            else {
                // A checkpoint marker past the loaded checkpoint has no
                // file behind it in this session: the chain cannot cross.
                stop = ReplayStop::Gap {
                    at_seq: record.seq,
                    expected: current.0 + 1,
                    found: record.revision.0,
                };
                break;
            };
            if record.revision.0 != current.0 + 1 {
                stop = ReplayStop::Gap {
                    at_seq: record.seq,
                    expected: current.0 + 1,
                    found: record.revision.0,
                };
                break;
            }
            let request = TransactionRequest {
                command_id: CommandId::new_v4(),
                operations: ops.clone(),
                merge_key: None,
            };
            let prepared = match prepare_transaction(&document, request, current) {
                Ok(prepared) => prepared,
                Err(error) => {
                    stop = ReplayStop::ApplyFailed {
                        at_seq: record.seq,
                        reason: format!("prepare failed: {error}"),
                    };
                    break;
                }
            };
            if let Err(error) = commit_transaction(&mut document, prepared) {
                stop = ReplayStop::ApplyFailed {
                    at_seq: record.seq,
                    reason: format!("apply failed: {error}"),
                };
                break;
            }
            for (id, bytes) in record_blobs {
                blobs.insert(*id, bytes.clone());
            }
            current = DocumentRevision(current.0 + 1);
            applied += 1;
        }
        if metadata.clean_shutdown && applied == 0 && stop == ReplayStop::CleanEnd {
            return Ok(None);
        }
        Ok(Some(RecoveryReport {
            document,
            blobs,
            checkpoint_revision: DocumentRevision(metadata.base_revision),
            applied,
            latest_revision: current,
            stop,
        }))
    }

    /// Mark a clean shutdown: recovery exists but holds nothing newer
    /// than the confirmed save. Best-effort removal happens via
    /// [`RecoverySession::discard`].
    pub fn mark_clean(&self) -> Result<()> {
        let Some(mut metadata) = self.read_metadata()? else {
            return Ok(());
        };
        metadata.clean_shutdown = true;
        metadata.updated_unix_ms = unix_millis();
        self.write_metadata(metadata)
    }

    /// Remove the whole session directory. Best-effort: a missing
    /// directory is already gone.
    pub fn discard(&self) -> Result<()> {
        match std::fs::remove_dir_all(&self.dir) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(recovery(format!("recovery discard failed: {error}"))),
        }
    }

    fn read_metadata(&self) -> Result<Option<RecoveryMetadata>> {
        let path = self.metadata_path();
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(recovery(format!("metadata read failed: {error}"))),
        };
        let metadata: RecoveryMetadata = serde_json::from_slice(&bytes)
            .map_err(|error| recovery(format!("metadata parse failed: {error}")))?;
        if metadata.format != "petunia-recovery" || metadata.schema != 1 {
            return Err(recovery("recovery metadata schema is not replayable"));
        }
        Ok(Some(metadata))
    }

    fn write_metadata(&self, metadata: RecoveryMetadata) -> Result<()> {
        let bytes = serde_json::to_vec(&metadata)
            .map_err(|error| recovery(format!("metadata encode failed: {error}")))?;
        atomic_write(&self.metadata_path(), &bytes, "json")?;
        restrict_file(&self.metadata_path());
        Ok(())
    }
}

fn create_private_dir(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(path)
            .map_err(|error| recovery(format!("recovery directory failed: {error}")))?;
    }
    #[cfg(not(unix))]
    {
        std::fs::create_dir_all(path)
            .map_err(|error| recovery(format!("recovery directory failed: {error}")))?;
    }
    Ok(())
}

fn restrict_file(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::DocumentRevision;
    use petunia_core::{ParentRef, SceneNode, VectorPath};

    fn scratch_root(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("petunia-recovery-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        root
    }

    fn session(name: &str) -> (RecoverySession, Document) {
        let document = Document::new("recovery");
        let session =
            RecoverySession::open(&scratch_root(name), document.id, RecoveryLimits::default())
                .expect("opens");
        (session, document)
    }

    fn insert_root(document: &Document, name: &str) -> DocumentOp {
        DocumentOp::InsertRoot {
            index: document.scene.len(),
            node: Box::new(SceneNode::new_path(
                name,
                VectorPath::rect(0.0, 0.0, 10.0, 10.0),
                ParentRef::Page(document.scene.default_page()),
            )),
        }
    }

    fn teardown(session: &RecoverySession) {
        let _ = session.discard();
    }

    #[test]
    fn checkpoint_plus_journal_recovers_everything() {
        let (session, mut document) = session("cycle");
        session
            .write_checkpoint(&document, DocumentRevision::GENESIS, &BTreeMap::new())
            .expect("checkpoints");
        // Two committed transactions, journaled at revisions 1 and 2.
        for (revision, name) in [(1u64, "one"), (2, "two")] {
            let op = insert_root(&document, name);
            let request = TransactionRequest {
                command_id: CommandId::new_v4(),
                operations: vec![op.clone()],
                merge_key: None,
            };
            let prepared = prepare_transaction(&document, request, DocumentRevision(revision - 1))
                .expect("prepares");
            commit_transaction(&mut document, prepared).expect("commits");
            session
                .append_transaction(
                    DocumentRevision(revision),
                    vec![op],
                    name.to_string(),
                    BTreeMap::new(),
                )
                .expect("journals");
        }
        session.sync().expect("syncs");
        let report = session.recover().expect("recovers").expect("report");
        assert_eq!(report.applied, 2);
        assert_eq!(report.latest_revision, DocumentRevision(2));
        assert_eq!(report.document.scene.len(), 2);
        assert_eq!(report.stop, ReplayStop::CleanEnd);
        teardown(&session);
    }

    #[test]
    fn torn_tail_recovers_the_confirmed_prefix() {
        let (session, mut document) = session("torn");
        session
            .write_checkpoint(&document, DocumentRevision::GENESIS, &BTreeMap::new())
            .expect("checkpoints");
        for revision in [1u64, 2] {
            let op = insert_root(&document, &format!("n{revision}"));
            let request = TransactionRequest {
                command_id: CommandId::new_v4(),
                operations: vec![op.clone()],
                merge_key: None,
            };
            let prepared = prepare_transaction(&document, request, DocumentRevision(revision - 1))
                .expect("prepares");
            commit_transaction(&mut document, prepared).expect("commits");
            session
                .append_transaction(
                    DocumentRevision(revision),
                    vec![op],
                    "n".to_string(),
                    BTreeMap::new(),
                )
                .expect("journals");
        }
        // Crash mid-append: tear the last bytes off the journal file.
        let path = session.journal_path();
        let mut bytes = std::fs::read(&path).expect("reads");
        bytes.truncate(bytes.len() - 7);
        std::fs::write(&path, &bytes).expect("tears");
        let report = session.recover().expect("recovers").expect("report");
        assert_eq!(report.applied, 1);
        assert_eq!(report.latest_revision, DocumentRevision(1));
        assert_eq!(report.document.scene.len(), 1);
        assert!(matches!(report.stop, ReplayStop::TruncatedTail { .. }));
        // Recovery files survive the failed replay.
        assert!(session.journal_path().exists());
        assert!(session.checkpoint_path().exists());
        teardown(&session);
    }

    #[test]
    fn stale_journal_after_checkpoint_is_skipped() {
        let (session, mut document) = session("stale");
        session
            .write_checkpoint(&document, DocumentRevision::GENESIS, &BTreeMap::new())
            .expect("checkpoints");
        let op = insert_root(&document, "one");
        let request = TransactionRequest {
            command_id: CommandId::new_v4(),
            operations: vec![op.clone()],
            merge_key: None,
        };
        let prepared =
            prepare_transaction(&document, request, DocumentRevision::GENESIS).expect("prepares");
        commit_transaction(&mut document, prepared).expect("commits");
        session
            .append_transaction(
                DocumentRevision(1),
                vec![op],
                "one".to_string(),
                BTreeMap::new(),
            )
            .expect("journals");
        // A newer checkpoint absorbs the journaled transaction, but a
        // crash lands before the journal truncates: restore the
        // pre-rotation journal over the rotated one.
        let stale_journal = std::fs::read(session.journal_path()).expect("reads");
        session
            .write_checkpoint(&document, DocumentRevision(1), &BTreeMap::new())
            .expect("re-checkpoints");
        std::fs::write(session.journal_path(), &stale_journal).expect("restores stale");
        let report = session.recover().expect("recovers").expect("report");
        assert_eq!(report.applied, 0);
        assert_eq!(report.latest_revision, DocumentRevision(1));
        assert_eq!(report.document.scene.len(), 1);
        teardown(&session);
    }

    #[test]
    fn revision_gap_stops_at_the_last_consistent_state() {
        let (session, document) = session("gap");
        session
            .write_checkpoint(&document, DocumentRevision::GENESIS, &BTreeMap::new())
            .expect("checkpoints");
        let op = insert_root(&document, "skip-one");
        session
            .append_transaction(
                DocumentRevision(2),
                vec![op],
                "gap".to_string(),
                BTreeMap::new(),
            )
            .expect_err("gap must fail at append");
        // Forge the gap straight into the file: a valid frame at rev 2.
        let record = JournalRecord {
            seq: 1,
            revision: DocumentRevision(2),
            operation: JournalOperation::TransactionApplied {
                description: "gap".to_string(),
                ops: vec![insert_root(&document, "gap")],
                blobs: BTreeMap::new(),
            },
        };
        let frame = encode_frame(&record, &JournalLimits::default()).expect("encodes");
        {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(session.journal_path())
                .expect("opens");
            file.write_all(&frame).expect("writes");
        }
        let report = session.recover().expect("recovers").expect("report");
        assert_eq!(report.applied, 0);
        assert!(matches!(report.stop, ReplayStop::Gap { .. }));
        teardown(&session);
    }

    #[test]
    fn unappliable_record_stops_with_diagnostics() {
        let (session, document) = session("unappliable");
        session
            .write_checkpoint(&document, DocumentRevision::GENESIS, &BTreeMap::new())
            .expect("checkpoints");
        // Forge a record that can never prepare: removing nothing.
        let record = JournalRecord {
            seq: 1,
            revision: DocumentRevision(1),
            operation: JournalOperation::TransactionApplied {
                description: "bad".to_string(),
                ops: vec![DocumentOp::RemoveSubtree {
                    root: petunia_core::ObjectId::new_v4(),
                }],
                blobs: BTreeMap::new(),
            },
        };
        let frame = encode_frame(&record, &JournalLimits::default()).expect("encodes");
        {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(session.journal_path())
                .expect("opens");
            file.write_all(&frame).expect("writes");
        }
        let report = session.recover().expect("recovers").expect("report");
        assert_eq!(report.applied, 0);
        assert!(matches!(report.stop, ReplayStop::ApplyFailed { .. }));
        teardown(&session);
    }

    #[test]
    fn clean_shutdown_recovers_nothing() {
        let (session, document) = session("clean");
        assert!(session.recover().expect("recovers").is_none());
        session
            .write_checkpoint(&document, DocumentRevision::GENESIS, &BTreeMap::new())
            .expect("checkpoints");
        session.mark_clean().expect("marks");
        assert!(session.recover().expect("recovers").is_none());
        teardown(&session);
    }

    #[test]
    fn foreign_schema_and_identity_are_refused() {
        let (session, document) = session("foreign");
        session
            .write_checkpoint(&document, DocumentRevision::GENESIS, &BTreeMap::new())
            .expect("checkpoints");
        // Corrupt the journal schema marker.
        let path = session.journal_path();
        let mut bytes = std::fs::read(&path).expect("reads");
        bytes[4] = 0xFF;
        bytes[5] = 0xFF;
        std::fs::write(&path, &bytes).expect("writes");
        assert!(session.recover().is_err());
        // A checkpoint for another document is refused.
        let other = Document::new("other");
        let foreign = RecoverySession::open(
            &scratch_root("foreign-other"),
            other.id,
            RecoveryLimits::default(),
        )
        .expect("opens");
        foreign
            .write_checkpoint(&other, DocumentRevision::GENESIS, &BTreeMap::new())
            .expect("checkpoints");
        // Point this session at foreign files by swapping directories.
        let _ = std::fs::remove_dir_all(session.dir.clone());
        std::fs::rename(foreign.dir.clone(), session.dir.clone()).expect("swaps");
        assert!(session.recover().is_err());
        teardown(&session);
        teardown(&foreign);
    }

    #[test]
    fn blob_budget_is_enforced_at_append() {
        let (session, document) = session("budget");
        session
            .write_checkpoint(&document, DocumentRevision::GENESIS, &BTreeMap::new())
            .expect("checkpoints");
        let tight = RecoverySession::open(
            &scratch_root("budget-tight"),
            document.id,
            RecoveryLimits {
                max_blob_bytes: 8,
                ..RecoveryLimits::default()
            },
        )
        .expect("opens");
        // No checkpoint in the tight session: append must demand one.
        assert!(tight
            .append_transaction(
                DocumentRevision(1),
                Vec::new(),
                "x".to_string(),
                BTreeMap::new(),
            )
            .is_err());
        teardown(&session);
        teardown(&tight);
    }
}
