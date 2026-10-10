//! Append-only recovery journal with framed, checksummed records.
//!
//! Layout on disk:
//!
//! ```text
//! magic "PRJ1" + schema u16 + flags u16
//! record*
//!
//! record := payload_len u32 | type u8 | revision u64 | seq u64
//!           | payload | crc32 u32 over (type | revision | seq | payload)
//! ```
//!
//! Replay never fails the whole file on a damaged tail: it returns
//! the confirmed prefix plus the exact stop reason. Files are only
//! ever appended or atomically replaced; recovery never deletes.

use crate::error::{EngineError, Result};
use crate::transaction::{DocumentOp, DocumentRevision};
use petunia_core::ResourceId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;

/// Journal file magic.
pub const JOURNAL_MAGIC: &[u8; 4] = b"PRJ1";

/// Journal framing schema understood by this build.
pub const JOURNAL_SCHEMA: u16 = 1;

/// Record carrying a clean checkpoint marker.
const RECORD_CHECKPOINT: u8 = 1;

/// Record carrying one committed transaction.
const RECORD_TRANSACTION: u8 = 2;

/// Bounds for journal encoding, replay and durable appends.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct JournalLimits {
    /// Largest accepted single record frame in bytes.
    pub max_record_bytes: u64,
    /// Largest accepted record count per file.
    pub max_records: usize,
    /// Largest accepted journal file in bytes.
    pub max_file_bytes: u64,
}

impl Default for JournalLimits {
    fn default() -> Self {
        Self {
            max_record_bytes: 64 << 20,
            max_records: 100_000,
            max_file_bytes: 1 << 30,
        }
    }
}

/// Journaled operations. Transactions carry their forward operations
/// plus any blobs they introduced, so replay needs nothing else.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JournalOperation {
    /// A clean checkpoint exists at this revision.
    Checkpoint {
        snapshot_ref: String,
        content_hash: Option<String>,
    },
    /// A transaction committed at this revision.
    TransactionApplied {
        description: String,
        ops: Vec<DocumentOp>,
        #[serde(deserialize_with = "petunia_core::serialization::deserialize_unique_btree_map")]
        blobs: BTreeMap<ResourceId, Vec<u8>>,
    },
}

/// One journal record: sequence identity, authorial revision and the
/// operation itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalRecord {
    pub seq: u64,
    pub revision: DocumentRevision,
    pub operation: JournalOperation,
}

/// Why replay stopped. `CleanEnd` means every record parsed.
#[derive(Debug, Clone, PartialEq)]
pub enum ReplayStop {
    CleanEnd,
    /// Final record is cut short: prefix before it is confirmed.
    TruncatedTail {
        at_seq: u64,
    },
    /// A record fails its checksum: prefix before it is confirmed.
    CorruptRecord {
        at_seq: u64,
    },
    /// Sequence numbers skip: prefix before it is confirmed.
    SequenceBreak {
        at_seq: u64,
        expected: u64,
        found: u64,
    },
    /// A revision gap the chain cannot cross: prefix before it is
    /// confirmed.
    Gap {
        at_seq: u64,
        expected: u64,
        found: u64,
    },
    /// A record that cannot be prepared or applied: the document
    /// stays at the last consistent state, files preserved.
    ApplyFailed {
        at_seq: u64,
        reason: String,
    },
    /// Revisions step backwards: prefix before it is confirmed.
    RevisionBreak {
        at_seq: u64,
    },
}

/// Confirmed prefix plus the stop reason.
#[derive(Debug, Clone, PartialEq)]
pub struct JournalReplay {
    pub records: Vec<JournalRecord>,
    pub stop: ReplayStop,
}

fn recovery(message: impl Into<String>) -> EngineError {
    EngineError::Recovery(message.into())
}

/// In-memory journal. Sequence numbers are assigned on append and
/// never reused; revisions never step backwards.
#[derive(Debug, Default)]
pub struct Journal {
    entries: Vec<JournalRecord>,
}

impl Journal {
    /// Empty journal.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append one operation at `revision`, assigning the next sequence
    /// number. Revisions must not step backwards.
    pub fn append(
        &mut self,
        revision: DocumentRevision,
        operation: JournalOperation,
    ) -> Result<()> {
        if let Some(last) = self.entries.last() {
            if revision.0 < last.revision.0 {
                return Err(recovery(format!(
                    "journal revision steps back from {} to {}",
                    last.revision.0, revision.0,
                )));
            }
        }
        self.entries.push(JournalRecord {
            seq: self.entries.len() as u64,
            revision,
            operation,
        });
        Ok(())
    }

    /// All entries in sequence order.
    #[must_use]
    pub fn entries(&self) -> &[JournalRecord] {
        &self.entries
    }

    /// Latest checkpoint marker, if any.
    #[must_use]
    pub fn last_checkpoint(&self) -> Option<(DocumentRevision, &str)> {
        self.entries
            .iter()
            .rev()
            .find_map(|entry| match &entry.operation {
                JournalOperation::Checkpoint { snapshot_ref, .. } => {
                    Some((entry.revision, snapshot_ref.as_str()))
                }
                _ => None,
            })
    }

    /// Encode the whole journal: header plus every frame.
    pub fn encode(&self, limits: &JournalLimits) -> Result<Vec<u8>> {
        if self.entries.len() > limits.max_records {
            return Err(recovery(format!(
                "journal of {} records exceeds limit {}",
                self.entries.len(),
                limits.max_records
            )));
        }
        let mut out = Vec::new();
        out.extend_from_slice(JOURNAL_MAGIC);
        out.extend_from_slice(&JOURNAL_SCHEMA.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        for entry in &self.entries {
            out.extend_from_slice(&encode_record(entry, limits)?);
            if out.len() as u64 > limits.max_file_bytes {
                return Err(recovery(format!(
                    "journal exceeds file limit {}",
                    limits.max_file_bytes
                )));
            }
        }
        Ok(out)
    }

    /// Append records at and after `since_seq` to the file, creating
    /// it with a header when missing. Flushes without syncing; call
    /// [`sync_file`] on policy points. Returns the new entry count.
    pub fn flush_new(
        &self,
        path: &std::path::Path,
        since_seq: u64,
        limits: &JournalLimits,
    ) -> Result<u64> {
        if since_seq > self.entries.len() as u64 {
            return Err(recovery(format!(
                "flush starts past the journal end: {since_seq}"
            )));
        }
        let exists = path.exists();
        if exists {
            verify_header(path)?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|error| recovery(format!("journal open failed: {error}")))?;
        if !exists {
            file.write_all(&Self::header())
                .map_err(|error| recovery(format!("journal header write failed: {error}")))?;
        }
        for entry in &self.entries[since_seq as usize..] {
            let frame = encode_record(entry, limits)?;
            file.write_all(&frame)
                .map_err(|error| recovery(format!("journal append failed: {error}")))?;
        }
        file.flush()
            .map_err(|error| recovery(format!("journal flush failed: {error}")))?;
        Ok(self.entries.len() as u64)
    }

    pub(crate) fn header() -> [u8; 8] {
        let mut header = [0u8; 8];
        header[..4].copy_from_slice(JOURNAL_MAGIC);
        header[4..6].copy_from_slice(&JOURNAL_SCHEMA.to_le_bytes());
        header
    }
}

/// Header every journal file starts with.
pub fn verify_header(path: &std::path::Path) -> Result<()> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)
        .map_err(|error| recovery(format!("journal open failed: {error}")))?;
    let mut header = [0u8; 8];
    file.read_exact(&mut header)
        .map_err(|error| recovery(format!("journal header unreadable: {error}")))?;
    verify_header_bytes(&header)
}

fn verify_header_bytes(header: &[u8]) -> Result<()> {
    if header.len() < 8 || &header[..4] != JOURNAL_MAGIC {
        return Err(recovery("not a petunia recovery journal"));
    }
    let schema = u16::from_le_bytes([header[4], header[5]]);
    if schema != JOURNAL_SCHEMA {
        return Err(recovery(format!(
            "journal schema {schema} is not replayable by this build"
        )));
    }
    Ok(())
}

/// Sync a journal file to durable storage.
pub fn sync_file(path: &std::path::Path) -> Result<()> {
    std::fs::File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|error| recovery(format!("journal sync failed: {error}")))
}

/// Encode one framed record for durable appends. Prefer
/// [`Journal::flush_new`] when the whole in-memory journal is handy.
pub fn encode_frame(record: &JournalRecord, limits: &JournalLimits) -> Result<Vec<u8>> {
    encode_record(record, limits)
}

/// Encode one framed record.
fn encode_record(record: &JournalRecord, limits: &JournalLimits) -> Result<Vec<u8>> {
    let payload = serde_json::to_vec(&record.operation)
        .map_err(|error| recovery(format!("journal encode failed: {error}")))?;
    let frame_len = 1 + 8 + 8 + payload.len() + 4;
    if frame_len as u64 > limits.max_record_bytes {
        return Err(recovery(format!(
            "journal record at seq {} of {} bytes exceeds limit {}",
            record.seq, frame_len, limits.max_record_bytes
        )));
    }
    let kind = match &record.operation {
        JournalOperation::Checkpoint { .. } => RECORD_CHECKPOINT,
        JournalOperation::TransactionApplied { .. } => RECORD_TRANSACTION,
    };
    let mut frame = Vec::with_capacity(4 + frame_len);
    frame.extend_from_slice(&(frame_len as u32).to_le_bytes());
    frame.push(kind);
    frame.extend_from_slice(&record.revision.0.to_le_bytes());
    frame.extend_from_slice(&record.seq.to_le_bytes());
    frame.extend_from_slice(&payload);
    let mut digest = crc32fast::Hasher::new();
    digest.update(&frame[4..]);
    frame.extend_from_slice(&digest.finalize().to_le_bytes());
    Ok(frame)
}

/// Decode bytes into the confirmed prefix plus the stop reason.
/// A bad header fails the whole file; anything later only truncates
/// the confirmed prefix.
pub fn decode_bytes(bytes: &[u8], limits: &JournalLimits) -> Result<JournalReplay> {
    verify_header_bytes(bytes.get(..8).unwrap_or(&[]))?;
    let mut records = Vec::new();
    let mut offset = 8usize;
    let mut last_revision: Option<u64> = None;
    loop {
        let seq = records.len() as u64;
        if offset == bytes.len() {
            return Ok(JournalReplay {
                records,
                stop: ReplayStop::CleanEnd,
            });
        }
        if bytes.len() - offset < 4 {
            return Ok(JournalReplay {
                records,
                stop: ReplayStop::TruncatedTail { at_seq: seq },
            });
        }
        let frame_len = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| recovery(format!("journal frame at seq {seq} is unreadable")))?,
        ) as usize;
        if frame_len as u64 > limits.max_record_bytes {
            return Ok(JournalReplay {
                records,
                stop: ReplayStop::CorruptRecord { at_seq: seq },
            });
        }
        if frame_len < 1 + 8 + 8 + 4 {
            return Ok(JournalReplay {
                records,
                stop: ReplayStop::CorruptRecord { at_seq: seq },
            });
        }
        if bytes.len() - offset - 4 < frame_len {
            return Ok(JournalReplay {
                records,
                stop: ReplayStop::TruncatedTail { at_seq: seq },
            });
        }
        let frame = &bytes[offset + 4..offset + 4 + frame_len];
        let (body, checksum) = frame.split_at(frame.len() - 4);
        let expected = u32::from_le_bytes(
            checksum
                .try_into()
                .map_err(|_| recovery(format!("journal frame at seq {seq} is unreadable")))?,
        );
        let mut digest = crc32fast::Hasher::new();
        digest.update(body);
        if digest.finalize() != expected {
            return Ok(JournalReplay {
                records,
                stop: ReplayStop::CorruptRecord { at_seq: seq },
            });
        }
        let kind = body[0];
        let revision = u64::from_le_bytes(
            body[1..9]
                .try_into()
                .map_err(|_| recovery(format!("journal frame at seq {seq} is unreadable")))?,
        );
        let found_seq = u64::from_le_bytes(
            body[9..17]
                .try_into()
                .map_err(|_| recovery(format!("journal frame at seq {seq} is unreadable")))?,
        );
        if found_seq != seq {
            return Ok(JournalReplay {
                records,
                stop: ReplayStop::SequenceBreak {
                    at_seq: seq,
                    expected: seq,
                    found: found_seq,
                },
            });
        }
        if !matches!(kind, RECORD_CHECKPOINT | RECORD_TRANSACTION) {
            return Ok(JournalReplay {
                records,
                stop: ReplayStop::CorruptRecord { at_seq: seq },
            });
        }
        if let Some(last) = last_revision {
            if revision < last {
                return Ok(JournalReplay {
                    records,
                    stop: ReplayStop::RevisionBreak { at_seq: seq },
                });
            }
        }
        let payload = &body[17..];
        // A well-framed record with an undecodable payload stops the
        // prefix: residual bytes are never interpreted.
        let operation: JournalOperation = match serde_json::from_slice(payload) {
            Ok(operation) => operation,
            Err(_) => {
                return Ok(JournalReplay {
                    records,
                    stop: ReplayStop::CorruptRecord { at_seq: seq },
                });
            }
        };
        let kind_matches = matches!(
            (kind, &operation),
            (RECORD_CHECKPOINT, JournalOperation::Checkpoint { .. })
                | (
                    RECORD_TRANSACTION,
                    JournalOperation::TransactionApplied { .. }
                )
        );
        if !kind_matches {
            return Ok(JournalReplay {
                records,
                stop: ReplayStop::CorruptRecord { at_seq: seq },
            });
        }
        last_revision = Some(revision);
        records.push(JournalRecord {
            seq,
            revision: DocumentRevision(revision),
            operation,
        });
        offset += 4 + frame_len;
    }
}

/// Decode a journal file into its confirmed prefix.
pub fn decode_file(path: &std::path::Path, limits: &JournalLimits) -> Result<JournalReplay> {
    let bytes =
        std::fs::read(path).map_err(|error| recovery(format!("journal read failed: {error}")))?;
    if bytes.len() as u64 > limits.max_file_bytes {
        return Err(recovery(format!(
            "journal file exceeds limit {}",
            limits.max_file_bytes
        )));
    }
    decode_bytes(&bytes, limits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::DocumentOp;
    use petunia_core::{SceneNode, VectorPath};

    fn transaction(revision: u64) -> (DocumentRevision, JournalOperation) {
        (
            DocumentRevision(revision),
            JournalOperation::TransactionApplied {
                description: "test".to_string(),
                ops: vec![DocumentOp::RemoveSubtree {
                    root: SceneNode::new_path(
                        "box",
                        VectorPath::new(),
                        petunia_core::ParentRef::Page(petunia_core::PageId::new_v4()),
                    )
                    .id,
                }],
                blobs: BTreeMap::new(),
            },
        )
    }

    fn checkpoint(revision: u64) -> (DocumentRevision, JournalOperation) {
        (
            DocumentRevision(revision),
            JournalOperation::Checkpoint {
                snapshot_ref: "checkpoint.ptnd".to_string(),
                content_hash: Some("crc32:00000000".to_string()),
            },
        )
    }

    fn journaled() -> Journal {
        let mut journal = Journal::new();
        let (revision, operation) = checkpoint(10);
        journal.append(revision, operation).expect("appends");
        let (revision, operation) = transaction(11);
        journal.append(revision, operation).expect("appends");
        let (revision, operation) = transaction(12);
        journal.append(revision, operation).expect("appends");
        journal
    }

    #[test]
    fn framing_round_trip_preserves_records() {
        let journal = journaled();
        let bytes = journal.encode(&JournalLimits::default()).expect("encodes");
        let replay = decode_bytes(&bytes, &JournalLimits::default()).expect("decodes");
        assert_eq!(replay.stop, ReplayStop::CleanEnd);
        assert_eq!(replay.records, journal.entries());
    }

    #[test]
    fn truncated_tail_keeps_the_confirmed_prefix() {
        let journal = journaled();
        let bytes = journal.encode(&JournalLimits::default()).expect("encodes");
        // Cut inside the last record.
        let cut = bytes.len() - 3;
        let replay = decode_bytes(&bytes[..cut], &JournalLimits::default()).expect("decodes");
        assert_eq!(replay.records.len(), 2);
        assert_eq!(replay.stop, ReplayStop::TruncatedTail { at_seq: 2 });
        // A single torn byte also truncates instead of failing.
        let mut torn = bytes.clone();
        torn.truncate(torn.len() - 1);
        let replay = decode_bytes(&torn, &JournalLimits::default()).expect("decodes");
        assert_eq!(replay.records.len(), 2);
    }

    #[test]
    fn corrupt_middle_record_stops_the_prefix() {
        let journal = journaled();
        let mut bytes = journal.encode(&JournalLimits::default()).expect("encodes");
        // Corrupt a payload byte of the second record (past header +
        // first record). Flipping bits breaks its checksum.
        let mut offset = 8usize;
        for _ in 0..1 {
            let frame_len =
                u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4 + frame_len;
        }
        bytes[offset + 10] ^= 0xFF;
        let replay = decode_bytes(&bytes, &JournalLimits::default()).expect("decodes");
        assert_eq!(replay.records.len(), 1);
        assert_eq!(replay.stop, ReplayStop::CorruptRecord { at_seq: 1 });
    }

    #[test]
    fn foreign_bytes_and_schemas_fail_loudly() {
        assert!(decode_bytes(b"not a journal", &JournalLimits::default()).is_err());
        let mut bytes = journaled()
            .encode(&JournalLimits::default())
            .expect("encodes");
        bytes[4] = 0xFF;
        bytes[5] = 0xFF;
        assert!(decode_bytes(&bytes, &JournalLimits::default()).is_err());
    }

    #[test]
    fn sequence_and_revision_breaks_stop_cleanly() {
        let journal = journaled();
        let mut bytes = journal.encode(&JournalLimits::default()).expect("encodes");
        // Rewrite the last record's sequence number with a valid CRC.
        let mut offset = 8usize;
        let mut starts = Vec::new();
        for _ in 0..3 {
            starts.push(offset);
            let frame_len =
                u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4 + frame_len;
        }
        // Corrupt seq of record 2 by patching payload seq bytes and CRC.
        let third = starts[2];
        // seq lives at frame+4+1+8..+8.
        let seq_at = third + 4 + 1 + 8;
        bytes[seq_at] = 0x2A;
        // Fix the CRC so the frame parses, exposing the break.
        let frame_len = u32::from_le_bytes(bytes[third..third + 4].try_into().unwrap()) as usize;
        let mut digest = crc32fast::Hasher::new();
        digest.update(&bytes[third + 4..third + 4 + frame_len - 4]);
        bytes[third + 4 + frame_len - 4..third + 4 + frame_len]
            .copy_from_slice(&digest.finalize().to_le_bytes());
        let replay = decode_bytes(&bytes, &JournalLimits::default()).expect("decodes");
        assert_eq!(
            replay.stop,
            ReplayStop::SequenceBreak {
                at_seq: 2,
                expected: 2,
                found: 42,
            }
        );
        assert_eq!(replay.records.len(), 2);
    }

    #[test]
    fn revisions_never_step_backwards() {
        let mut journal = Journal::new();
        let (revision, operation) = transaction(5);
        journal.append(revision, operation).expect("appends");
        let (revision, operation) = transaction(4);
        assert!(journal.append(revision, operation).is_err());
    }

    #[test]
    fn oversized_records_are_refused_before_allocation() {
        let mut journal = Journal::new();
        let (revision, _) = transaction(1);
        journal
            .append(
                revision,
                JournalOperation::TransactionApplied {
                    description: "big".to_string(),
                    ops: Vec::new(),
                    blobs: BTreeMap::from([(petunia_core::ResourceId::new_v4(), vec![0u8; 1024])]),
                },
            )
            .expect("appends");
        let tight = JournalLimits {
            max_record_bytes: 64,
            ..JournalLimits::default()
        };
        assert!(journal.encode(&tight).is_err());
    }
}
