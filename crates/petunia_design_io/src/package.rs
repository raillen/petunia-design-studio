//! Package reader/writer with extension policy and atomic saves.

use std::fs::{File, OpenOptions};
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
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
const MAX_DOCUMENT_BYTES: u64 = 256 * 1024 * 1024;

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
    pub recovery: Option<crate::recovery::RecoveryMetadata>,
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
    /// Schema 4 stores immutable image/tile bytes in verified binary entries.
    #[serde(default)]
    pub binary_resources: bool,
    /// Recovery metadata is committed atomically with its document bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery: Option<crate::recovery::RecoveryMetadata>,
}

impl PackageManifest {
    /// Current manifest for this build.
    #[must_use]
    pub fn current() -> Self {
        Self {
            media_type: MEDIA_TYPE.to_string(),
            schema_version: NATIVE_SCHEMA_VERSION,
            writer: format!("petunia-design/{}", env!("CARGO_PKG_VERSION")),
            binary_resources: true,
            recovery: None,
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
    save_with_manifest(document, path, &PackageManifest::current())
}

pub(crate) fn save_recovery_package(
    document: &Document,
    path: &Path,
    metadata: &crate::recovery::RecoveryMetadata,
) -> Result<(), PetuniaError> {
    let mut manifest = PackageManifest::current();
    manifest.recovery = Some(metadata.clone());
    save_with_manifest(document, path, &manifest)
}

fn save_with_manifest(
    document: &Document,
    path: &Path,
    manifest: &PackageManifest,
) -> Result<(), PetuniaError> {
    check_writable_suffix(path)?;
    document.validate()?;
    // Recovery candidates already have a private random pathname and are
    // published under the store lease. Ordinary saves need a stable target lock.
    if manifest.recovery.is_some() {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(|e| PetuniaError::io(e.to_string()))?;
        write_package(document, file, manifest)?
            .sync_all()
            .map_err(|e| PetuniaError::io(e.to_string()))?;
    } else {
        let lease = crate::atomic_output::OutputLease::acquire(path)?;
        let temp = lease.temporary()?;
        let file = temp
            .as_file()
            .try_clone()
            .map_err(|e| PetuniaError::io(e.to_string()))?;
        write_package(document, file, manifest)?;
        lease.publish(temp, &|| false)?;
    }
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
    if archive.len() > 10_000 {
        return Err(PetuniaError::invalid_input("package has too many entries"));
    }
    for name in [MANIFEST_PATH, DOCUMENT_PATH] {
        if archive.file_names().filter(|n| *n == name).count() != 1 {
            return Err(PetuniaError::invalid_input(format!(
                "package requires exactly one `{name}`"
            )));
        }
    }

    let manifest_text = read_bounded_entry(&mut archive, MANIFEST_PATH, MAX_MANIFEST_BYTES)?;
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
    if !(1..=NATIVE_SCHEMA_VERSION).contains(&manifest.schema_version) {
        return Err(PetuniaError::invalid_input(format!(
            "unsupported package schema {}, expected {}",
            manifest.schema_version, NATIVE_SCHEMA_VERSION
        )));
    }

    let document_text = read_bounded_entry(&mut archive, DOCUMENT_PATH, MAX_DOCUMENT_BYTES)?;
    #[derive(Deserialize)]
    struct SchemaHeader {
        schema_version: u32,
    }
    let payload_version = serde_json::from_str::<SchemaHeader>(&document_text)
        .map_err(|e| PetuniaError::invalid_input(format!("document JSON: {e}")))?
        .schema_version;
    if payload_version != manifest.schema_version {
        return Err(PetuniaError::invalid_input(
            "manifest and document schema versions disagree",
        ));
    }
    let mut document = Document::from_json(&document_text)?;
    if manifest.schema_version >= 4 {
        if !manifest.binary_resources {
            return Err(PetuniaError::invalid_input(
                "schema 4 requires binary resources",
            ));
        }
        crate::binary_resources::read(&mut document, &mut archive)?;
    } else if manifest.binary_resources {
        return Err(PetuniaError::invalid_input(
            "binary resources require schema 4",
        ));
    }
    // A legacy suffix alone marks the package legacy even when the manifest
    // already carries the current media type.
    let format = if suffix_format == PackageFormat::Legacy {
        PackageFormat::Legacy
    } else {
        manifest_format
    };
    Ok(OpenedPackage {
        document,
        format,
        recovery: manifest.recovery,
    })
}

/// Reads small manifest metadata only; restore still validates the full payload.
pub(crate) fn recovery_metadata(
    path: &Path,
) -> Result<Option<crate::recovery::RecoveryMetadata>, PetuniaError> {
    let file = File::open(path).map_err(|e| PetuniaError::io(e.to_string()))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| PetuniaError::invalid_input(e.to_string()))?;
    if archive.len() > 10_000
        || archive
            .file_names()
            .filter(|name| *name == MANIFEST_PATH)
            .count()
            != 1
    {
        return Err(PetuniaError::invalid_input(
            "invalid recovery package entries",
        ));
    }
    let text = read_bounded_entry(&mut archive, MANIFEST_PATH, MAX_MANIFEST_BYTES)?;
    let manifest: PackageManifest =
        serde_json::from_str(&text).map_err(|e| PetuniaError::invalid_input(e.to_string()))?;
    if manifest.media_type != MEDIA_TYPE || manifest.schema_version != NATIVE_SCHEMA_VERSION {
        return Err(PetuniaError::invalid_input("unsupported recovery package"));
    }
    Ok(manifest.recovery)
}

fn read_bounded_entry(
    archive: &mut zip::ZipArchive<File>,
    name: &str,
    limit: u64,
) -> Result<String, PetuniaError> {
    let entry = archive
        .by_name(name)
        .map_err(|e| PetuniaError::invalid_input(format!("package entry `{name}`: {e}")))?;
    if entry.size() > limit {
        return Err(PetuniaError::invalid_input(format!(
            "package entry `{name}` exceeds {limit} bytes"
        )));
    }
    let mut text = String::new();
    entry
        .take(limit + 1)
        .read_to_string(&mut text)
        .map_err(|e| PetuniaError::io(format!("read `{name}`: {e}")))?;
    if text.len() as u64 > limit {
        return Err(PetuniaError::invalid_input(format!(
            "package entry `{name}` exceeds budget"
        )));
    }
    Ok(text)
}

fn write_package(
    document: &Document,
    file: File,
    manifest: &PackageManifest,
) -> Result<File, PetuniaError> {
    let mut zip = zip::ZipWriter::new(file);
    let manifest_options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip.start_file(MANIFEST_PATH, manifest_options)
        .map_err(|e| PetuniaError::io(format!("zip manifest: {e}")))?;
    let manifest_json = serde_json::to_string_pretty(manifest)
        .map_err(|e| PetuniaError::io(format!("serialize manifest: {e}")))?;
    zip.write_all(manifest_json.as_bytes())
        .map_err(|e| PetuniaError::io(format!("write manifest: {e}")))?;

    let metadata = crate::binary_resources::write(document, &mut zip)?;
    let document_options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file(DOCUMENT_PATH, document_options)
        .map_err(|e| PetuniaError::io(format!("zip document: {e}")))?;
    serde_json::to_writer_pretty(&mut zip, &metadata)
        .map_err(|e| PetuniaError::io(format!("write document: {e}")))?;
    zip.finish()
        .map_err(|e| PetuniaError::io(format!("finish package: {e}")))
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
        let temp_dir = tempfile::tempdir().unwrap();
        let dir = temp_dir.path();
        let path = dir.join("roundtrip.PTND");
        let _ = std::fs::remove_file(&path);
        let doc = sample_doc();
        save_package(&doc, &path).unwrap();
        // No temp residue after atomic save.
        assert!(!std::fs::read_dir(dir).unwrap().any(|entry| entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".petunia-save-")));
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
