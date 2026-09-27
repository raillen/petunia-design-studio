//! Package reader/writer with extension policy and atomic saves.

use std::fs::File;
use std::io::{Read, Write as _};
use std::path::{Path, PathBuf};

use petunia_design_document::Document;
use petunia_design_foundation::{PetuniaError, NATIVE_SCHEMA_VERSION};
use serde::{Deserialize, Serialize};

/// Internal media type identifying the package regardless of suffix (15.A).
pub const MEDIA_TYPE: &str = "application/vnd.petunia-design-studio.project+zip";

/// Schema namespace recorded by the manifest (15.A).
pub const SCHEMA_NAMESPACE: &str = "ptnd";

/// Canonical native suffix, case-insensitively matched on read.
pub const NATIVE_SUFFIX: &str = "ptnd";

/// Human-facing extension used in titles, pickers and messages.
pub const NATIVE_EXTENSION_DISPLAY: &str = ".PTND";

/// The suffix without its leading dot, for `Path::with_extension`. Kept as its
/// own constant so the display form and the on-disk form cannot drift.
pub const NATIVE_EXTENSION: &str = "PTND";

/// Diagnostics/filenames namespace.
const NATIVE_FILE_LABEL: &str = "name.PTND";

const MANIFEST_PATH: &str = "manifest.json";
const DOCUMENT_PATH: &str = "document/document.json";

/// Legacy suffixes, accepted by the migration reader only (15.A).
const LEGACY_SUFFIXES: &[&str] = &["aubrieta", "aubri"];

/// Legacy package media type, accepted on read so old projects can migrate.
const LEGACY_MEDIA_TYPE: &str = "application/vnd.aubrieta.design";

/// Which package format was opened (15.A migration reporting).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageFormat {
    /// Current native format.
    Ptnd,
    /// Legacy project; the caller MUST Save As before writing back.
    Legacy,
}

impl PackageFormat {
    /// True when the opened package requires an explicit Save As.
    #[must_use]
    pub fn requires_save_as(self) -> bool {
        matches!(self, Self::Legacy)
    }
}

/// A successfully opened package plus its originating format.
#[derive(Clone, Debug, PartialEq)]
pub struct OpenedPackage {
    /// Decoded canonical document.
    pub document: Document,
    /// Format the bytes came from.
    pub format: PackageFormat,
}

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
            writer: format!("petunia-design/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

/// Returns `path` with the canonical `.PTND` extension, never duplicating a
/// suffix the path already carries (15.A forbids `name.PTND.PTND`).
#[must_use]
pub fn with_native_extension(path: &Path) -> PathBuf {
    let lower = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if lower == NATIVE_SUFFIX {
        return path.to_path_buf();
    }
    if LEGACY_SUFFIXES.contains(&lower.as_str()) {
        // Save As upgrades the suffix in place (`old.aubrieta` -> `old.PTND`)
        // instead of passing the legacy path through, which the writable-suffix
        // policy would refuse.
        return path.with_extension(NATIVE_EXTENSION);
    }
    let mut name = path
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_default();
    name.push(NATIVE_EXTENSION_DISPLAY);
    path.with_file_name(name)
}

/// True when the path already points at a native `.PTND` file.
#[must_use]
pub fn has_native_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case(NATIVE_SUFFIX))
}

/// Saves a document atomically: temp file in the same directory + rename.
/// Only the current native suffix is writable: legacy paths must go through
/// Save As so the original file is never overwritten (15.A).
pub fn save_package(document: &Document, path: &Path) -> Result<(), PetuniaError> {
    check_writable_suffix(path)?;
    let temp_path = temp_sibling(path);
    write_package(document, &temp_path).inspect_err(|_| {
        let _ = std::fs::remove_file(&temp_path);
    })?;
    std::fs::rename(&temp_path, path)
        .map_err(|e| PetuniaError::io(format!("atomic rename {}: {e}", path.display())))?;
    Ok(())
}

/// Opens and validates a package: suffix, media type, schema version.
/// Legacy packages are decoded through the same schema and reported as
/// [`PackageFormat::Legacy`] so the caller can force Save As (15.A).
pub fn open_package(path: &Path) -> Result<OpenedPackage, PetuniaError> {
    let suffix_format = check_readable_suffix(path)?;
    let file =
        File::open(path).map_err(|e| PetuniaError::io(format!("open {}: {e}", path.display())))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| PetuniaError::io(format!("read zip {}: {e}", path.display())))?;

    let mut manifest_text = String::new();
    archive
        .by_name(MANIFEST_PATH)
        .map_err(|_| {
            PetuniaError::invalid_input(format!("{} lacks {MANIFEST_PATH}", path.display()))
        })?
        .read_to_string(&mut manifest_text)
        .map_err(|e| PetuniaError::io(format!("read manifest: {e}")))?;
    let manifest: PackageManifest = serde_json::from_str(&manifest_text)
        .map_err(|e| PetuniaError::io(format!("parse manifest: {e}")))?;
    let manifest_format = match manifest.media_type.as_str() {
        MEDIA_TYPE => PackageFormat::Ptnd,
        LEGACY_MEDIA_TYPE => PackageFormat::Legacy,
        other => {
            return Err(PetuniaError::invalid_input(format!(
                "wrong media type `{other}` in {}",
                path.display()
            )))
        }
    };
    if manifest.schema_version != NATIVE_SCHEMA_VERSION {
        return Err(PetuniaError::invalid_input(format!(
            "unsupported package schema {}, expected {}",
            manifest.schema_version, NATIVE_SCHEMA_VERSION
        )));
    }

    let mut document_text = String::new();
    archive
        .by_name(DOCUMENT_PATH)
        .map_err(|_| {
            PetuniaError::invalid_input(format!("{} lacks {DOCUMENT_PATH}", path.display()))
        })?
        .read_to_string(&mut document_text)
        .map_err(|e| PetuniaError::io(format!("read document: {e}")))?;
    let document = Document::from_json(&document_text)?;
    // A legacy suffix alone marks the package legacy even when the manifest
    // already carries the current media type.
    let format = if suffix_format == PackageFormat::Legacy {
        PackageFormat::Legacy
    } else {
        manifest_format
    };
    Ok(OpenedPackage { document, format })
}

fn write_package(document: &Document, path: &Path) -> Result<(), PetuniaError> {
    let file = File::create(path)
        .map_err(|e| PetuniaError::io(format!("create {}: {e}", path.display())))?;
    let mut zip = zip::ZipWriter::new(file);
    let manifest_options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip.start_file(MANIFEST_PATH, manifest_options)
        .map_err(|e| PetuniaError::io(format!("zip manifest: {e}")))?;
    let manifest_json = serde_json::to_string_pretty(&PackageManifest::current())
        .map_err(|e| PetuniaError::io(format!("serialize manifest: {e}")))?;
    zip.write_all(manifest_json.as_bytes())
        .map_err(|e| PetuniaError::io(format!("write manifest: {e}")))?;

    let document_options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file(DOCUMENT_PATH, document_options)
        .map_err(|e| PetuniaError::io(format!("zip document: {e}")))?;
    let document_json = document.to_json()?;
    zip.write_all(document_json.as_bytes())
        .map_err(|e| PetuniaError::io(format!("write document: {e}")))?;
    zip.finish()
        .map_err(|e| PetuniaError::io(format!("finish {}: {e}", path.display())))?;
    Ok(())
}

/// Writable formats: the current native suffix only (15.A forbids writing
/// new projects with legacy identifiers/suffixes).
fn check_writable_suffix(path: &Path) -> Result<(), PetuniaError> {
    let suffix = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if suffix == NATIVE_SUFFIX {
        return Ok(());
    }
    if LEGACY_SUFFIXES.contains(&suffix.as_str()) {
        return Err(PetuniaError::invalid_input(format!(
            "`.{suffix}` is a legacy project suffix; use Save As with `{NATIVE_EXTENSION_DISPLAY}`"
        )));
    }
    Err(PetuniaError::invalid_input(format!(
        "expected `{NATIVE_EXTENSION_DISPLAY}` (e.g. `{NATIVE_FILE_LABEL}`), got `{}`",
        path.display()
    )))
}

/// Read formats: native plus legacy migration suffixes.
fn check_readable_suffix(path: &Path) -> Result<PackageFormat, PetuniaError> {
    let suffix = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if suffix == NATIVE_SUFFIX {
        return Ok(PackageFormat::Ptnd);
    }
    if LEGACY_SUFFIXES.contains(&suffix.as_str()) {
        return Ok(PackageFormat::Legacy);
    }
    Err(PetuniaError::invalid_input(format!(
        "expected `{NATIVE_EXTENSION_DISPLAY}` or a legacy project suffix, got `{}`",
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
    #[test]
    fn the_display_and_on_disk_extensions_agree() {
        use super::{NATIVE_EXTENSION, NATIVE_EXTENSION_DISPLAY};
        assert_eq!(format!(".{NATIVE_EXTENSION}"), NATIVE_EXTENSION_DISPLAY);
        assert_eq!(NATIVE_EXTENSION.to_ascii_lowercase(), super::NATIVE_SUFFIX);
    }

    use super::*;
    use petunia_design_document::{DocumentMutator, DocumentObject};
    use petunia_design_foundation::IdGenerator;

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
    fn native_roundtrip_preserves_semantic_state() {
        let dir = std::env::temp_dir().join("petunia-design-io-tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("roundtrip.PTND");
        let _ = std::fs::remove_file(&path);
        let doc = sample_doc();
        save_package(&doc, &path).unwrap();
        // No temp residue after atomic save.
        assert!(!temp_sibling(&path).exists());
        let opened = open_package(&path).unwrap();
        assert_eq!(opened.format, PackageFormat::Ptnd);
        assert!(!opened.format.requires_save_as());
        assert_eq!(opened.document, doc);
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn lowercase_suffix_reads_as_the_same_format() {
        let dir = std::env::temp_dir().join("petunia-design-io-tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("lowercase.ptnd");
        let _ = std::fs::remove_file(&path);
        let doc = sample_doc();
        save_package(&doc, &path).unwrap();
        let opened = open_package(&path).unwrap();
        assert_eq!(opened.format, PackageFormat::Ptnd);
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn legacy_suffixes_are_read_only_and_report_migration() {
        let dir = std::env::temp_dir().join("petunia-design-io-tests");
        std::fs::create_dir_all(&dir).unwrap();
        let doc = sample_doc();
        // Build a legacy-suffixed package by writing native bytes under the
        // legacy name: the reader must classify it as legacy.
        let native = dir.join("legacy-source.PTND");
        save_package(&doc, &native).unwrap();
        let legacy = dir.join("old-project.aubrieta");
        let _ = std::fs::remove_file(&legacy);
        std::fs::copy(&native, &legacy).unwrap();

        let opened = open_package(&legacy).unwrap();
        assert_eq!(opened.format, PackageFormat::Legacy);
        assert!(opened.format.requires_save_as());
        assert_eq!(opened.document, doc);

        // Writing back to the legacy path must be refused (Save As required).
        let err = save_package(&doc, &legacy).expect_err("legacy write refused");
        assert!(err.to_string().contains("legacy"), "{err}");

        std::fs::remove_file(&native).unwrap();
        std::fs::remove_file(&legacy).unwrap();
    }

    #[test]
    fn extension_helper_never_duplicates_the_suffix() {
        assert_eq!(
            with_native_extension(Path::new("art/Untitled")).to_string_lossy(),
            "art/Untitled.PTND"
        );
        assert_eq!(
            with_native_extension(Path::new("art/Untitled.PTND")).to_string_lossy(),
            "art/Untitled.PTND"
        );
        assert_eq!(
            with_native_extension(Path::new("art/Untitled.ptnd")).to_string_lossy(),
            "art/Untitled.ptnd"
        );
        // A legacy suffix is *upgraded*: Save As writes `Old.PTND` and leaves
        // the original `Old.aubrieta` untouched on disk.
        assert_eq!(
            with_native_extension(Path::new("art/Old.aubrieta")).to_string_lossy(),
            "art/Old.PTND"
        );
        assert_eq!(
            with_native_extension(Path::new("art/Old.aubri")).to_string_lossy(),
            "art/Old.PTND"
        );
        assert_eq!(
            with_native_extension(Path::new("art/Old.AUBRIETA")).to_string_lossy(),
            "art/Old.PTND"
        );
        assert!(has_native_extension(Path::new("a.PTND")));
        assert!(has_native_extension(Path::new("a.ptnd")));
        assert!(!has_native_extension(Path::new("a.aubrieta")));
    }

    #[test]
    fn unknown_suffixes_are_rejected_on_write_and_read() {
        let dir = std::env::temp_dir().join("petunia-design-io-tests");
        let doc = sample_doc();
        for suffix in ["abrt", "pds", "png"] {
            let err = save_package(&doc, &dir.join(format!("x.{suffix}"))).expect_err("rejected");
            assert!(err.to_string().contains(".PTND"), "{err}");
            let err = open_package(&dir.join(format!("x.{suffix}"))).expect_err("rejected");
            assert!(err.to_string().contains(".PTND"), "{err}");
        }
    }
}
