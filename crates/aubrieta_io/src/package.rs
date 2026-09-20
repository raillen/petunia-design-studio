//! Package reader/writer with extension policy and atomic saves.

use std::fs::File;
use std::io::{Read, Write as _};
use std::path::{Path, PathBuf};

use aubrieta_document::Document;
use aubrieta_foundation::{AubrietaError, NATIVE_SCHEMA_VERSION};
use serde::{Deserialize, Serialize};

/// Internal media type identifying the package regardless of suffix.
pub const MEDIA_TYPE: &str = "application/vnd.aubrieta.design";

const MANIFEST_PATH: &str = "manifest.json";
const DOCUMENT_PATH: &str = "document/document.json";

/// Accepted native suffixes for the same package/schema.
const ACCEPTED_SUFFIXES: &[&str] = &["aubrieta", "aubri"];

/// Rejected suffixes (glossary `forbidden`): reported, never sniffed around.
const REJECTED_SUFFIXES: &[&str] = &["abrt", "pds", "petunia"];

/// Readable package manifest at the ZIP root.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManifest {
    /// Must equal [`MEDIA_TYPE`].
    pub media_type: String,
    /// Native schema version; must equal [`NATIVE_SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Workspace build that wrote the package (informational).
    pub writer: String,
}

impl PackageManifest {
    /// Current manifest for this build.
    #[must_use]
    pub fn current() -> Self {
        Self {
            media_type: MEDIA_TYPE.to_string(),
            schema_version: NATIVE_SCHEMA_VERSION,
            writer: format!("aubrieta-io/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

/// Saves a document atomically: temp file in the same directory + rename.
pub fn save_package(document: &Document, path: &Path) -> Result<(), AubrietaError> {
    check_suffix(path)?;
    let parent: &Path = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let temp_path = temp_sibling(path);
    write_package(document, &temp_path).inspect_err(|_| {
        let _ = std::fs::remove_file(&temp_path);
    })?;
    std::fs::rename(&temp_path, path)
        .map_err(|e| AubrietaError::io(format!("atomic rename {}: {e}", path.display())))?;
    let _ = parent;
    Ok(())
}

/// Opens and validates a package: suffix, media type, schema version.
pub fn open_package(path: &Path) -> Result<Document, AubrietaError> {
    check_suffix(path)?;
    let file =
        File::open(path).map_err(|e| AubrietaError::io(format!("open {}: {e}", path.display())))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AubrietaError::io(format!("read zip {}: {e}", path.display())))?;

    let mut manifest_text = String::new();
    archive
        .by_name(MANIFEST_PATH)
        .map_err(|_| {
            AubrietaError::invalid_input(format!("{} lacks {MANIFEST_PATH}", path.display()))
        })?
        .read_to_string(&mut manifest_text)
        .map_err(|e| AubrietaError::io(format!("read manifest: {e}")))?;
    let manifest: PackageManifest = serde_json::from_str(&manifest_text)
        .map_err(|e| AubrietaError::io(format!("parse manifest: {e}")))?;
    if manifest.media_type != MEDIA_TYPE {
        return Err(AubrietaError::invalid_input(format!(
            "wrong media type `{}` in {}",
            manifest.media_type,
            path.display()
        )));
    }
    if manifest.schema_version != NATIVE_SCHEMA_VERSION {
        return Err(AubrietaError::invalid_input(format!(
            "unsupported package schema {}, expected {}",
            manifest.schema_version, NATIVE_SCHEMA_VERSION
        )));
    }

    let mut document_text = String::new();
    archive
        .by_name(DOCUMENT_PATH)
        .map_err(|_| {
            AubrietaError::invalid_input(format!("{} lacks {DOCUMENT_PATH}", path.display()))
        })?
        .read_to_string(&mut document_text)
        .map_err(|e| AubrietaError::io(format!("read document: {e}")))?;
    Document::from_json(&document_text)
}

fn write_package(document: &Document, path: &Path) -> Result<(), AubrietaError> {
    let file = File::create(path)
        .map_err(|e| AubrietaError::io(format!("create {}: {e}", path.display())))?;
    let mut zip = zip::ZipWriter::new(file);
    let manifest_options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip.start_file(MANIFEST_PATH, manifest_options)
        .map_err(|e| AubrietaError::io(format!("zip manifest: {e}")))?;
    let manifest_json = serde_json::to_string_pretty(&PackageManifest::current())
        .map_err(|e| AubrietaError::io(format!("serialize manifest: {e}")))?;
    zip.write_all(manifest_json.as_bytes())
        .map_err(|e| AubrietaError::io(format!("write manifest: {e}")))?;

    let document_options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file(DOCUMENT_PATH, document_options)
        .map_err(|e| AubrietaError::io(format!("zip document: {e}")))?;
    let document_json = document.to_json()?;
    zip.write_all(document_json.as_bytes())
        .map_err(|e| AubrietaError::io(format!("write document: {e}")))?;
    zip.finish()
        .map_err(|e| AubrietaError::io(format!("finish {}: {e}", path.display())))?;
    Ok(())
}

fn check_suffix(path: &Path) -> Result<(), AubrietaError> {
    let suffix = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ACCEPTED_SUFFIXES.contains(&suffix.as_str()) {
        return Ok(());
    }
    if REJECTED_SUFFIXES.contains(&suffix.as_str()) {
        return Err(AubrietaError::invalid_input(format!(
            "`.{suffix}` is a rejected glossary term; use `.aubrieta` or `.aubri`"
        )));
    }
    Err(AubrietaError::invalid_input(format!(
        "expected `.aubrieta` or `.aubri`, got `{}`",
        path.display()
    )))
}

fn temp_sibling(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_default();
    name.push(".tmp");
    path.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aubrieta_document::{DocumentMutator, DocumentObject};
    use aubrieta_foundation::IdGenerator;

    fn sample_doc() -> Document {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let surface = gen.next_surface();
        let mut mutator = DocumentMutator::new(&mut doc);
        mutator.add_surface(surface, "Page").unwrap();
        mutator
            .add_object(surface, DocumentObject::new(gen.next_object(), "Rect"))
            .unwrap();
        doc
    }

    #[test]
    fn save_reopen_roundtrip_both_suffixes() {
        for suffix in ["aubrieta", "aubri"] {
            let dir = std::env::temp_dir().join("aubrieta-io-tests");
            std::fs::create_dir_all(&dir).unwrap();
            let path = dir.join(format!("roundtrip.{suffix}"));
            let _ = std::fs::remove_file(&path);
            let doc = sample_doc();
            save_package(&doc, &path).unwrap();
            // No temp residue after atomic save.
            assert!(!temp_sibling(&path).exists());
            let reopened = open_package(&path).unwrap();
            assert_eq!(reopened, doc);
            std::fs::remove_file(&path).unwrap();
        }
    }

    #[test]
    fn rejected_suffixes_are_explicit() {
        let dir = std::env::temp_dir().join("aubrieta-io-tests");
        let doc = sample_doc();
        for suffix in ["abrt", "pds"] {
            let err = save_package(&doc, &dir.join(format!("x.{suffix}"))).expect_err("rejected");
            assert!(err.to_string().contains("rejected"), "{err}");
        }
        let err = save_package(&doc, &dir.join("x.png")).expect_err("unknown suffix");
        assert!(err.to_string().contains("expected"), "{err}");
    }
}
