//! Detached save snapshots and bounded open operations for the single-writer session.

use crate::ptnd::{self, BlobStore, FileFingerprint, PackageLimits, PackagePayload};
use crate::{DocumentRevision, EngineError, Result};
use petunia_core::{ContentHash, Document, DocumentId};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Immutable input captured by the writer before dispatching a save job.
/// Blob clones share their allocations; later edits cannot change this snapshot.
#[derive(Debug, Clone)]
pub struct SaveSnapshot {
    pub document: Document,
    pub revision: DocumentRevision,
    pub blobs: BlobStore,
    pub preview: Option<Vec<u8>>,
    pub extensions: BTreeMap<String, Vec<u8>>,
}

/// How the destination is allowed to change. Overwriting is always explicit.
#[derive(Debug, Clone)]
pub enum SavePolicy {
    CreateNew,
    IfUnchanged(FileFingerprint),
    Overwrite,
}

/// Completion acknowledges the revision actually saved, even if editing continued.
#[derive(Debug, Clone)]
pub struct SaveCompletion {
    pub document_id: DocumentId,
    pub revision: DocumentRevision,
    pub path: PathBuf,
    pub fingerprint: FileFingerprint,
}

/// Opened content retains previews and opaque extension payloads across saves.
#[derive(Debug, Clone)]
pub struct OpenedDocument {
    pub package: ptnd::LoadedPackage,
    pub path: PathBuf,
    pub fingerprint: FileFingerprint,
}

fn read_bounded(path: &Path, max_bytes: u64) -> Result<(Vec<u8>, FileFingerprint)> {
    let file = std::fs::File::open(path)
        .map_err(|error| EngineError::Execution(format!("open {path:?}: {error}")))?;
    let metadata = file
        .metadata()
        .map_err(|error| EngineError::Execution(format!("stat {path:?}: {error}")))?;
    if !metadata.is_file() || metadata.len() > max_bytes {
        return Err(EngineError::Limit(format!(
            "file {path:?} exceeds package input limit"
        )));
    }
    let mut bytes = Vec::new();
    file.take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| EngineError::Execution(format!("read {path:?}: {error}")))?;
    if bytes.len() as u64 > max_bytes {
        return Err(EngineError::Limit(
            "file grew beyond package input limit".into(),
        ));
    }
    let fingerprint = FileFingerprint {
        content_hash: ContentHash::new(&bytes),
        size_bytes: bytes.len() as u64,
        modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
    };
    Ok((bytes, fingerprint))
}

/// Decode a bounded native file before replacing anything in the live session.
pub fn open_document(path: &Path, limits: &PackageLimits) -> Result<OpenedDocument> {
    let (bytes, fingerprint) = read_bounded(path, limits.max_bytes)?;
    let package = ptnd::load_package(&bytes, limits)?;
    Ok(OpenedDocument {
        package,
        path: path.to_path_buf(),
        fingerprint,
    })
}

/// Validate and encode first, then lock, check for external changes and atomically save.
/// The lock coordinates cooperating writers; an unrelated writer must honor it too.
pub fn save_snapshot(
    snapshot: &SaveSnapshot,
    path: &Path,
    policy: &SavePolicy,
    limits: &PackageLimits,
) -> Result<SaveCompletion> {
    let blobs = snapshot.blobs.to_map();
    let bytes = ptnd::save_package(
        &PackagePayload {
            document: &snapshot.document,
            blobs: &blobs,
            preview: snapshot.preview.as_deref(),
            extensions: &snapshot.extensions,
        },
        limits,
    )?;
    let _lock = ptnd::CooperativeFileLock::acquire(path)?;
    match policy {
        SavePolicy::CreateNew
            if path
                .try_exists()
                .map_err(|error| EngineError::Execution(format!("check {path:?}: {error}")))? =>
        {
            return Err(EngineError::Conflict(format!(
                "destination {path:?} already exists"
            )));
        }
        SavePolicy::IfUnchanged(expected) => {
            let (_, current) = read_bounded(path, limits.max_bytes)
                .map_err(|error| EngineError::Conflict(format!("destination {path:?}: {error}")))?;
            if current.content_hash != expected.content_hash {
                return Err(EngineError::Conflict(format!(
                    "destination {path:?} changed externally"
                )));
            }
        }
        _ => {}
    }
    ptnd::save_atomic(path, &bytes)?;
    let metadata = std::fs::metadata(path)
        .map_err(|error| EngineError::Execution(format!("stat saved {path:?}: {error}")))?;
    Ok(SaveCompletion {
        document_id: snapshot.document.id,
        revision: snapshot.revision,
        path: path.to_path_buf(),
        fingerprint: FileFingerprint {
            content_hash: ContentHash::new(&bytes),
            size_bytes: bytes.len() as u64,
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        },
    })
}
