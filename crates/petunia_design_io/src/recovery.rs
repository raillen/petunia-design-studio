//! Durable recovery copies use the same native codec/atomic-save boundary.
//! The original project path is informational; this store never writes it.
use crate::open_package;
use petunia_design_document::Document;
use petunia_design_foundation::PetuniaError;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_ENTRIES: usize = 256;
const MAX_METADATA: u64 = 64 * 1024;
const MAX_STORE_BYTES: u64 = 1024 * 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryKey {
    path: PathBuf,
}
impl RecoveryKey {
    pub fn path(&self) -> &Path {
        &self.path
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RecoveryMetadata {
    pub title: String,
    pub original_path: Option<PathBuf>,
    pub revision: u64,
    pub captured_unix_seconds: u64,
}
impl RecoveryMetadata {
    pub fn new(title: String, original_path: Option<PathBuf>, revision: u64) -> Self {
        Self {
            title,
            original_path,
            revision,
            captured_unix_seconds: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |duration| duration.as_secs()),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryEntry {
    pub key: RecoveryKey,
    pub metadata: RecoveryMetadata,
}
#[derive(Clone, Debug)]
pub struct RecoveryStore {
    directory: PathBuf,
}
impl RecoveryStore {
    pub fn new(directory: PathBuf) -> Result<Self, PetuniaError> {
        std::fs::create_dir_all(&directory)
            .map_err(|e| PetuniaError::io(format!("create recovery directory: {e}")))?;
        let directory = directory
            .canonicalize()
            .map_err(|e| PetuniaError::io(format!("resolve recovery directory: {e}")))?;
        Ok(Self { directory })
    }
    pub fn directory(&self) -> &Path {
        &self.directory
    }
    fn owns(&self, key: &RecoveryKey) -> bool {
        key.path.parent() == Some(self.directory.as_path())
            && key
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("recovery-") && name.ends_with(".PTND"))
    }
    fn entries(&self) -> Result<Vec<PathBuf>, PetuniaError> {
        let mut result = Vec::new();
        for (index, entry) in std::fs::read_dir(&self.directory)
            .map_err(|e| PetuniaError::io(e.to_string()))?
            .enumerate()
        {
            if index >= 4096 {
                return Err(PetuniaError::invalid_input(
                    "recovery directory entry budget exceeded",
                ));
            }
            let entry = entry.map_err(|e| PetuniaError::io(e.to_string()))?;
            let path = entry.path();
            if self.owns(&RecoveryKey { path: path.clone() }) {
                let file_type = entry
                    .file_type()
                    .map_err(|e| PetuniaError::io(e.to_string()))?;
                if file_type.is_symlink() {
                    return Err(PetuniaError::invalid_input(
                        "recovery entries must not be symlinks",
                    ));
                }
                if !file_type.is_file() {
                    continue;
                }
                result.push(path);
            }
        }
        Ok(result)
    }
    fn allocate(&self) -> Result<RecoveryKey, PetuniaError> {
        if self.entries()?.len() >= MAX_ENTRIES {
            return Err(PetuniaError::invalid_input(
                "recovery snapshot count budget exceeded",
            ));
        }
        let temporary = tempfile::Builder::new()
            .prefix("recovery-")
            .suffix(".PTND")
            .tempfile_in(&self.directory)
            .map_err(|e| PetuniaError::io(format!("reserve recovery copy: {e}")))?;
        let (file, path) = temporary
            .keep()
            .map_err(|e| PetuniaError::io(e.to_string()))?;
        drop(file);
        Ok(RecoveryKey { path })
    }
    /// Returns a durable copy key, reusing only a key this store owns. Failed
    /// replacement retains the preceding valid snapshot and original file.
    pub fn write(
        &self,
        document: &Document,
        metadata: &RecoveryMetadata,
        previous: Option<&RecoveryKey>,
    ) -> Result<RecoveryKey, PetuniaError> {
        let _store_lease =
            crate::atomic_output::OutputLease::acquire(&self.directory.join(".recovery-store"))?;
        if metadata.title.len() > 4096
            || metadata
                .original_path
                .as_ref()
                .is_some_and(|path| path.as_os_str().len() > 16 * 1024)
        {
            return Err(PetuniaError::invalid_input(
                "recovery metadata budget exceeded",
            ));
        }
        let encoded = serde_json::to_vec(metadata).map_err(|e| PetuniaError::io(e.to_string()))?;
        if encoded.len() as u64 > MAX_METADATA {
            return Err(PetuniaError::invalid_input(
                "recovery metadata byte budget exceeded",
            ));
        }
        let mut stored = 0u64;
        for path in self.entries()? {
            if previous.is_some_and(|key| key.path == path) {
                continue;
            }
            stored = stored.saturating_add(
                std::fs::metadata(&path)
                    .map_err(|e| PetuniaError::io(e.to_string()))?
                    .len(),
            );
        }
        if previous.is_some_and(|key| !self.owns(key)) {
            return Err(PetuniaError::invalid_input("foreign recovery key"));
        }
        // The complete candidate is synced before replacing a previous backup.
        // Exact compressed size enforces the disk quota without rejecting small
        // documents merely because a maximal package could be much larger.
        let candidate = tempfile::Builder::new()
            .prefix(".recovery-candidate-")
            .suffix(".PTND")
            .tempfile_in(&self.directory)
            .map_err(|e| PetuniaError::io(e.to_string()))?;
        crate::package::save_recovery_package(document, candidate.path(), metadata)?;
        let size = std::fs::metadata(candidate.path())
            .map_err(|e| PetuniaError::io(e.to_string()))?
            .len();
        if stored.saturating_add(size) > MAX_STORE_BYTES {
            return Err(PetuniaError::invalid_input("recovery disk budget exceeded"));
        }
        let key = match previous {
            Some(key) => key.clone(),
            None => self.allocate()?,
        };
        candidate
            .persist(&key.path)
            .map_err(|e| PetuniaError::io(format!("publish recovery package: {e}")))?;
        #[cfg(unix)]
        File::open(&self.directory)
            .and_then(|file| file.sync_all())
            .map_err(|e| PetuniaError::io(e.to_string()))?;
        Ok(key)
    }
    /// Listing reads small metadata only. The selected native package is fully
    /// validated during restore; corruption never silently repairs artwork.
    pub fn list(&self) -> Result<Vec<RecoveryEntry>, PetuniaError> {
        let mut entries = Vec::new();
        for path in self.entries()? {
            if entries.len() >= MAX_ENTRIES {
                return Err(PetuniaError::invalid_input(
                    "recovery snapshot count budget exceeded",
                ));
            }
            if std::fs::metadata(&path)
                .map_err(|e| PetuniaError::io(e.to_string()))?
                .len()
                == 0
            {
                continue;
            }
            let fallback = RecoveryMetadata {
                title: "Recovered document".into(),
                original_path: None,
                revision: 0,
                captured_unix_seconds: 0,
            };
            let metadata = crate::package::recovery_metadata(&path)
                .ok()
                .flatten()
                .filter(|meta| meta.title.len() <= 4096)
                .unwrap_or(fallback);
            entries.push(RecoveryEntry {
                key: RecoveryKey { path },
                metadata,
            });
        }
        entries.sort_by(|a, b| {
            b.metadata
                .captured_unix_seconds
                .cmp(&a.metadata.captured_unix_seconds)
                .then_with(|| a.key.path.cmp(&b.key.path))
        });
        Ok(entries)
    }
    pub fn restore(&self, key: &RecoveryKey) -> Result<Document, PetuniaError> {
        if !self.owns(key) {
            return Err(PetuniaError::invalid_input("foreign recovery key"));
        }
        Ok(open_package(&key.path)?.document)
    }
    /// Only the application's own resolved/saved recovery copy may be purged.
    pub fn remove(&self, key: &RecoveryKey) -> Result<(), PetuniaError> {
        if !self.owns(key) {
            return Err(PetuniaError::invalid_input("foreign recovery key"));
        }
        for path in [&key.path, &key.path.with_extension("json")] {
            match std::fs::remove_file(path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(PetuniaError::io(error.to_string())),
            }
        }
        #[cfg(unix)]
        File::open(&self.directory)
            .and_then(|file| file.sync_all())
            .map_err(|e| PetuniaError::io(e.to_string()))?;
        Ok(())
    }
}
