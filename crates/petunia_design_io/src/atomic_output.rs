//! Same-directory durable publication with one cooperating writer per target.
//! A stable sidecar inode stays in place: unlinking a lock file after unlock
//! would let another process lock a different inode for the same destination.
use petunia_design_foundation::PetuniaError;
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub struct OutputLease {
    _file: File,
    destination: PathBuf,
    parent: PathBuf,
}
impl OutputLease {
    pub fn acquire(path: &Path) -> Result<Self, PetuniaError> {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."))
            .canonicalize()
            .map_err(|e| PetuniaError::io(format!("resolve output directory: {e}")))?;
        let name = path
            .file_name()
            .ok_or_else(|| PetuniaError::invalid_input("output requires a filename"))?;
        let destination = parent.join(name);
        match std::fs::symlink_metadata(&destination) {
            Ok(meta) if !meta.file_type().is_file() => {
                return Err(PetuniaError::invalid_input(
                    "output target must be a regular file, not a symlink or directory",
                ))
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(PetuniaError::io(e.to_string())),
        }
        let hash = Sha256::digest(name.as_encoded_bytes());
        let key = hash
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        let file = options
            .open(parent.join(format!(".petunia-output-{key}.lock")))
            .map_err(|e| PetuniaError::io(format!("open output lock: {e}")))?;
        if !file
            .metadata()
            .map_err(|e| PetuniaError::io(e.to_string()))?
            .is_file()
        {
            return Err(PetuniaError::invalid_input(
                "output lock must be a regular file",
            ));
        }
        file.try_lock().map_err(|e| {
            PetuniaError::io(format!(
                "output is already being written or cannot be locked: {e}"
            ))
        })?;
        Ok(Self {
            _file: file,
            destination,
            parent,
        })
    }
    pub fn temporary(&self) -> Result<tempfile::NamedTempFile, PetuniaError> {
        tempfile::Builder::new()
            .prefix(".petunia-output-")
            .tempfile_in(&self.parent)
            .map_err(|e| PetuniaError::io(format!("create output temporary file: {e}")))
    }
    pub fn publish(
        &self,
        temp: tempfile::NamedTempFile,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<(), PetuniaError> {
        temp.as_file()
            .sync_all()
            .map_err(|e| PetuniaError::io(format!("sync output: {e}")))?;
        if cancelled() {
            return Err(PetuniaError::invalid_input("output publication cancelled"));
        }
        temp.persist(&self.destination)
            .map_err(|e| PetuniaError::io(format!("publish output atomically: {e}")))?;
        #[cfg(unix)]
        File::open(&self.parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|e| PetuniaError::io(format!("sync output directory: {e}")))?;
        Ok(())
    }
}
pub fn write_atomic(
    path: &Path,
    bytes: &[u8],
    cancelled: &dyn Fn() -> bool,
) -> Result<(), PetuniaError> {
    let lease = OutputLease::acquire(path)?;
    let mut temp = lease.temporary()?;
    for chunk in bytes.chunks(64 * 1024) {
        if cancelled() {
            return Err(PetuniaError::invalid_input("output write cancelled"));
        }
        temp.write_all(chunk)
            .map_err(|e| PetuniaError::io(format!("write output: {e}")))?;
    }
    lease.publish(temp, cancelled)
}
