use super::{
    PreferenceError, PreferenceRecovery, PreferenceResult, UiPreferences, MAX_PREFERENCE_BYTES,
};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

impl UiPreferences {
    pub fn from_json(json: &str) -> PreferenceResult<Self> {
        if json.len() > MAX_PREFERENCE_BYTES {
            return Err(PreferenceError::TooLarge);
        }
        let preferences: Self = serde_json::from_str(json)?;
        preferences.validate()?;
        Ok(preferences)
    }

    pub fn to_json(&self) -> PreferenceResult<String> {
        self.validate()?;
        let json = serde_json::to_string_pretty(self)?;
        if json.len() > MAX_PREFERENCE_BYTES {
            return Err(PreferenceError::TooLarge);
        }
        Ok(json)
    }

    pub fn load(path: impl AsRef<Path>) -> PreferenceResult<Self> {
        let path = path.as_ref();
        require_regular_file(path)?;
        let file = File::open(path)?;
        if file.metadata()?.len() > MAX_PREFERENCE_BYTES as u64 {
            return Err(PreferenceError::TooLarge);
        }
        let mut bytes = Vec::new();
        // The read limit also handles a file that grows after the metadata check.
        file.take(MAX_PREFERENCE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_PREFERENCE_BYTES {
            return Err(PreferenceError::TooLarge);
        }
        let preferences: Self = serde_json::from_slice(&bytes)?;
        preferences.validate()?;
        Ok(preferences)
    }

    pub fn load_recovering(path: impl AsRef<Path>) -> PreferenceRecovery {
        match Self::load(path) {
            Ok(preferences) => PreferenceRecovery {
                preferences,
                diagnostic: None,
            },
            Err(PreferenceError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                PreferenceRecovery {
                    preferences: Self::default(),
                    diagnostic: None,
                }
            }
            Err(error) => PreferenceRecovery {
                preferences: Self::default(),
                diagnostic: Some(error.to_string()),
            },
        }
    }

    /// Validates and writes a bounded UTF-8 file, replacing the destination only
    /// after a complete synced write to an exclusively created sibling file.
    /// Errors before the rename leave the original untouched. On platforms that
    /// cannot atomically replace an existing file, the save fails safely.
    pub fn save(&self, path: impl AsRef<Path>) -> PreferenceResult<()> {
        let json = self.to_json()?;
        let path = path.as_ref();
        match fs::symlink_metadata(path) {
            Ok(_) => require_regular_file(path)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(error.into()),
        }
        let (temporary_path, mut temporary_file) = create_sibling_file(path)?;
        let result = (|| {
            temporary_file.write_all(json.as_bytes())?;
            temporary_file.write_all(b"\n")?;
            temporary_file.sync_all()?;
            drop(temporary_file);
            fs::rename(&temporary_path, path)?;
            Ok(())
        })();
        if result.is_err() {
            // Cleanup cannot replace the primary error or touch the source file.
            let _ = fs::remove_file(&temporary_path);
        }
        result
    }
}

fn require_regular_file(path: &Path) -> PreferenceResult<()> {
    if !fs::symlink_metadata(path)?.file_type().is_file() {
        return Err(PreferenceError::Invalid(
            "Preference path must be a regular file, not a link or directory".into(),
        ));
    }
    Ok(())
}

fn create_sibling_file(path: &Path) -> PreferenceResult<(PathBuf, File)> {
    let filename = path
        .file_name()
        .ok_or_else(|| PreferenceError::Invalid("Preference path has no filename".into()))?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| PreferenceError::Invalid("System clock is before the Unix epoch".into()))?
        .as_nanos();
    for attempt in 0..16 {
        let mut temporary_name = filename.to_os_string();
        temporary_name.push(format!(".{}.{}.{}.tmp", std::process::id(), stamp, attempt));
        let temporary_path = parent.join(temporary_name);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&temporary_path) {
            Ok(file) => return Ok((temporary_path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(PreferenceError::Invalid(
        "Could not reserve a temporary preference file".into(),
    ))
}
