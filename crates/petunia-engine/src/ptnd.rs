//! PTND physical container: ZIP/ZIP64 packages, manifest and limits.
//!
//! Layout on disk follows `00-architecture/ptnd-format.md`:
//!
//! ```text
//! document.ptnd
//! ├── manifest.json
//! ├── document.json
//! ├── resources/
//! ├── extensions/
//! └── previews/
//! ```
//!
//! The writer emits stored and deflated entries with CRC32, sizes up
//! front and a central directory, plus Zip64 structures only when a
//! count or size exceeds the classic 32-bit fields. The reader
//! enforces the same subset and verifies every CRC; encryption, data
//! descriptors, multi-disk disks and unknown methods fail with a
//! typed error instead of a best-effort parse. Entry names are
//! validated against traversal, duplicates, case collisions,
//! directories and symlinks. Saves are atomic: complete synced file
//! first, platform rename second.

use crate::error::{EngineError, Result};
use flate2::write::DeflateEncoder;
use flate2::{read::DeflateDecoder, Compression};
use petunia_core::{Document, DocumentId, ResourceId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::io::Read;
use std::sync::Arc;

/// Bounds for one package operation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PackageLimits {
    /// Largest accepted container in bytes.
    pub max_bytes: u64,
    /// Largest accepted entry count.
    pub max_entries: usize,
    /// Largest accepted uncompressed entry in bytes.
    pub max_entry_bytes: u64,
    /// Largest accepted total of uncompressed entries in bytes.
    pub max_total_bytes: u64,
    /// Largest accepted uncompressed/compressed ratio per entry.
    /// Highly compressible input still passes; bombs do not.
    pub max_ratio: u64,
    /// Longest accepted entry name in bytes.
    pub max_name_len: usize,
    /// Deepest accepted entry path (`a/b/c` is depth 3).
    pub max_path_depth: usize,
}

impl Default for PackageLimits {
    fn default() -> Self {
        Self {
            max_bytes: 512 << 20,
            max_entries: 8192,
            max_entry_bytes: 256 << 20,
            max_total_bytes: 2 << 30,
            max_ratio: 500,
            max_name_len: 512,
            max_path_depth: 16,
        }
    }
}

/// Compression per entry. Both methods are lossless; content-aware
/// selection (store for already-compressed media) is follow-up work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZipMethod {
    Stored,
    Deflated,
}

/// Manifest living at `manifest.json` inside every package.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PtndManifest {
    pub format: String,
    pub schema_version: u32,
    pub document_id: String,
    pub required_capabilities: Vec<String>,
    pub entries: Vec<String>,
}

/// Format marker every manifest carries.
pub const MANIFEST_FORMAT: &str = "petunia-design-document";

/// Manifest entry holding the versioned document DTO.
pub const DOCUMENT_ENTRY: &str = "document.json";

/// Manifest entry holding this manifest.
pub const MANIFEST_ENTRY: &str = "manifest.json";

/// Prefix for resource blob entries.
pub const RESOURCES_PREFIX: &str = "resources/";

/// Canonical entry path for the optional document preview thumbnail.
pub const PREVIEWS_ENTRY: &str = "previews/thumbnail.png";

/// Prefix for namespaced plugin/extension payload entries.
pub const EXTENSIONS_PREFIX: &str = "extensions/";

impl PtndManifest {
    /// Manifest for a package: canonical format marker, schema v1,
    /// canonical document identity and the sorted entry set.
    #[must_use]
    pub fn new(document_id: DocumentId, mut entries: Vec<String>) -> Self {
        entries.sort();
        Self {
            format: MANIFEST_FORMAT.to_string(),
            schema_version: 1,
            document_id: document_id.to_string(),
            required_capabilities: Vec::new(),
            entries,
        }
    }

    /// Compact canonical JSON bytes for the manifest entry.
    pub fn to_json(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|error| EngineError::Manifest(format!("manifest encode failed: {error}")))
    }

    /// Shape checks that need no package context.
    pub fn validate(&self) -> Result<()> {
        if self.format != MANIFEST_FORMAT {
            return Err(EngineError::Manifest(format!(
                "unknown format marker {:?}",
                self.format
            )));
        }
        if self.schema_version != 1 {
            return Err(EngineError::Manifest(format!(
                "unsupported manifest schema {}",
                self.schema_version
            )));
        }
        let parsed = self
            .document_id
            .parse::<uuid::Uuid>()
            .map_err(|_| EngineError::Manifest("manifest document_id is not a UUID".to_string()))?;
        if parsed.hyphenated().to_string() != self.document_id {
            return Err(EngineError::Manifest(
                "manifest document_id is not canonical lowercase UUID text".to_string(),
            ));
        }
        Ok(())
    }
}

fn package(message: impl Into<String>) -> EngineError {
    EngineError::Package(message.into())
}

fn limit(message: impl Into<String>) -> EngineError {
    EngineError::Limit(message.into())
}

fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn get_u16(input: &[u8], offset: usize) -> Result<u16> {
    input
        .get(offset..offset + 2)
        .and_then(|pair| <[u8; 2]>::try_from(pair).ok())
        .map(u16::from_le_bytes)
        .ok_or_else(|| package("truncated zip header"))
}

fn get_u32(input: &[u8], offset: usize) -> Result<u32> {
    input
        .get(offset..offset + 4)
        .and_then(|quad| <[u8; 4]>::try_from(quad).ok())
        .map(u32::from_le_bytes)
        .ok_or_else(|| package("truncated zip header"))
}

fn get_u64(input: &[u8], offset: usize) -> Result<u64> {
    input
        .get(offset..offset + 8)
        .and_then(|chunk| <[u8; 8]>::try_from(chunk).ok())
        .map(u64::from_le_bytes)
        .ok_or_else(|| package("truncated zip header"))
}

fn as_usize(value: u64, what: &str) -> Result<usize> {
    usize::try_from(value).map_err(|_| package(format!("{what} out of range")))
}

/// Entry names are container-relative UTF-8 paths: no absolute
/// paths, no parent escapes, no empty segments, no backslashes.
/// Checked on write and on read, so foreign packages cannot smuggle
/// traversal past the loader.
pub fn validate_entry_name(name: &str, limits: &PackageLimits) -> Result<()> {
    if name.is_empty() || name.len() > limits.max_name_len {
        return Err(package(format!("entry name out of range: {name:?}")));
    }
    if name.starts_with('/') || name.contains('\\') {
        return Err(package(format!(
            "entry name is not container-relative: {name:?}"
        )));
    }
    let mut depth = 0usize;
    for segment in name.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(package(format!(
                "entry name escapes its directory: {name:?}"
            )));
        }
        depth += 1;
    }
    if depth > limits.max_path_depth {
        return Err(package(format!("entry path too deep: {name:?}")));
    }
    Ok(())
}

/// Is this name a directory placeholder rather than a file entry?
fn is_directory_name(name: &str) -> bool {
    name.ends_with('/')
}

/// Serialize entries into a ZIP package, emitting Zip64 structures
/// only when a count or size exceeds the classic fields.
pub fn write_package(
    entries: &[(String, Vec<u8>, ZipMethod)],
    limits: &PackageLimits,
) -> Result<Vec<u8>> {
    if entries.len() > limits.max_entries {
        return Err(limit(format!(
            "package of {} entries exceeds limit {}",
            entries.len(),
            limits.max_entries
        )));
    }
    let mut seen = HashSet::new();
    for (name, _, _) in entries {
        validate_entry_name(name, limits)?;
        if is_directory_name(name) {
            return Err(package(format!("directory entries are refused: {name:?}")));
        }
        if !seen.insert(name) {
            return Err(package(format!("duplicate entry name: {name:?}")));
        }
    }
    // Decide the container shape up front: offsets are only known
    // while writing, and the mode cannot change mid-stream.
    let mut encoded: Vec<Vec<u8>> = Vec::with_capacity(entries.len());
    let mut header_len: u64 = 0;
    for (name, bytes, method) in entries {
        if bytes.len() as u64 > limits.max_entry_bytes {
            return Err(limit(format!(
                "entry {name} of {} bytes exceeds limit {}",
                bytes.len(),
                limits.max_entry_bytes
            )));
        }
        let data = match method {
            ZipMethod::Stored => bytes.clone(),
            ZipMethod::Deflated => {
                let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
                use std::io::Write;
                encoder
                    .write_all(bytes)
                    .map_err(|error| package(format!("deflate failed: {error}")))?;
                encoder
                    .finish()
                    .map_err(|error| package(format!("deflate failed: {error}")))?
            }
        };
        header_len += 30 + name.len() as u64 + data.len() as u64;
        encoded.push(data);
    }
    let zip64 = entries.len() > 0xFFFF
        || encoded
            .iter()
            .any(|data| data.len() as u64 > u32::MAX as u64)
        || header_len > u32::MAX as u64;
    if header_len > limits.max_bytes {
        return Err(limit("package exceeds byte limit".to_string()));
    }

    let mut out = Vec::new();
    let mut central = Vec::new();
    for ((name, bytes, method), data) in entries.iter().zip(encoded.iter()) {
        let crc = crc32fast::hash(bytes);
        let uncompressed = bytes.len() as u64;
        let compressed = data.len() as u64;
        let header_offset = out.len() as u64;
        let method_code = match method {
            ZipMethod::Stored => 0,
            ZipMethod::Deflated => 8,
        };
        // Local file header.
        out.extend_from_slice(b"PK\x03\x04");
        put_u16(&mut out, if zip64 { 45 } else { 20 });
        put_u16(&mut out, 0);
        put_u16(&mut out, method_code);
        put_u16(&mut out, 0);
        put_u16(&mut out, 0);
        put_u32(&mut out, crc);
        if zip64 {
            put_u32(&mut out, 0xFFFF_FFFF);
            put_u32(&mut out, 0xFFFF_FFFF);
        } else {
            put_u32(&mut out, compressed as u32);
            put_u32(&mut out, uncompressed as u32);
        }
        put_u16(&mut out, name.len() as u16);
        if zip64 {
            // Zip64 extra field: uncompressed + compressed, 8 bytes each.
            put_u16(&mut out, 28);
            out.extend_from_slice(name.as_bytes());
            put_u16(&mut out, 0x0001);
            put_u16(&mut out, 24);
            put_u64(&mut out, uncompressed);
            put_u64(&mut out, compressed);
            put_u64(&mut out, header_offset);
        } else {
            put_u16(&mut out, 0);
            out.extend_from_slice(name.as_bytes());
        }
        out.extend_from_slice(data);
        // Central directory entry.
        central.extend_from_slice(b"PK\x01\x02");
        put_u16(&mut central, if zip64 { 45 } else { 20 });
        put_u16(&mut central, if zip64 { 45 } else { 20 });
        put_u16(&mut central, 0);
        put_u16(&mut central, method_code);
        put_u16(&mut central, 0);
        put_u16(&mut central, 0);
        put_u32(&mut central, crc);
        if zip64 {
            put_u32(&mut central, 0xFFFF_FFFF);
            put_u32(&mut central, 0xFFFF_FFFF);
        } else {
            put_u32(&mut central, compressed as u32);
            put_u32(&mut central, uncompressed as u32);
        }
        put_u16(&mut central, name.len() as u16);
        if zip64 {
            put_u16(&mut central, 28);
        } else {
            put_u16(&mut central, 0);
        }
        put_u16(&mut central, 0);
        put_u16(&mut central, 0);
        put_u16(&mut central, 0);
        put_u32(&mut central, 0);
        if zip64 {
            put_u32(&mut central, 0xFFFF_FFFF);
        } else {
            put_u32(&mut central, header_offset as u32);
        }
        central.extend_from_slice(name.as_bytes());
        if zip64 {
            put_u16(&mut central, 0x0001);
            put_u16(&mut central, 24);
            put_u64(&mut central, uncompressed);
            put_u64(&mut central, compressed);
            put_u64(&mut central, header_offset);
        }
        if out.len() as u64 > limits.max_bytes {
            return Err(limit("package exceeds byte limit".to_string()));
        }
    }
    let central_offset = out.len() as u64;
    let central_size = central.len() as u64;
    out.extend_from_slice(&central);
    if zip64 {
        // Zip64 end of central directory record + locator.
        let eocd64_offset = out.len() as u64;
        out.extend_from_slice(b"PK\x06\x06");
        put_u64(&mut out, 44);
        put_u16(&mut out, 45);
        put_u16(&mut out, 45);
        put_u32(&mut out, 0);
        put_u32(&mut out, 0);
        put_u64(&mut out, entries.len() as u64);
        put_u64(&mut out, entries.len() as u64);
        put_u64(&mut out, central_size);
        put_u64(&mut out, central_offset);
        out.extend_from_slice(b"PK\x06\x07");
        put_u32(&mut out, 0);
        put_u64(&mut out, eocd64_offset);
        put_u32(&mut out, 1);
    }
    // End of central directory (with saturation markers in Zip64 mode).
    out.extend_from_slice(b"PK\x05\x06");
    put_u16(&mut out, 0);
    put_u16(&mut out, 0);
    if zip64 {
        put_u16(&mut out, 0xFFFF);
        put_u16(&mut out, 0xFFFF);
    } else {
        put_u16(&mut out, entries.len() as u16);
        put_u16(&mut out, entries.len() as u16);
    }
    if zip64 {
        put_u32(&mut out, 0xFFFF_FFFF);
        put_u32(&mut out, 0xFFFF_FFFF);
    } else {
        put_u32(&mut out, central_size as u32);
        put_u32(&mut out, central_offset as u32);
    }
    put_u16(&mut out, 0);
    Ok(out)
}

/// Central directory location after resolving a Zip64 locator when
/// the classic fields saturate.
struct CentralDirectory {
    count: u64,
    size: u64,
    offset: u64,
}

fn locate_central(bytes: &[u8]) -> Result<CentralDirectory> {
    let end = find_end_of_central_directory(bytes)?;
    let count = get_u16(bytes, end + 10)? as u64;
    let size = get_u32(bytes, end + 12)? as u64;
    let offset = get_u32(bytes, end + 16)? as u64;
    if count != 0xFFFF && size != 0xFFFF_FFFF && offset != 0xFFFF_FFFF {
        return Ok(CentralDirectory {
            count,
            size,
            offset,
        });
    }
    // Zip64: the locator sits in the 20 bytes before the classic end.
    if end < 20 || bytes.get(end - 20..end - 16) != Some(b"PK\x06\x07") {
        return Err(package("zip64 end locator not found"));
    }
    let eocd64 = get_u64(bytes, end - 12)? as usize;
    if bytes.get(eocd64..eocd64 + 4) != Some(b"PK\x06\x06") {
        return Err(package("zip64 end record not found"));
    }
    if get_u16(bytes, eocd64 + 14)? > 45 {
        return Err(package("zip64 record needs unsupported features"));
    }
    if get_u32(bytes, eocd64 + 16)? != 0 || get_u32(bytes, eocd64 + 20)? != 0 {
        return Err(package("multi-disk packages are refused"));
    }
    Ok(CentralDirectory {
        count: get_u64(bytes, eocd64 + 32)?,
        size: get_u64(bytes, eocd64 + 40)?,
        offset: get_u64(bytes, eocd64 + 48)?,
    })
}

/// 64-bit values hidden in a Zip64 extra field. Only the fields whose
/// classic counterpart saturated are present, in spec order:
/// uncompressed, compressed, header offset.
fn zip64_extra(
    extra: &[u8],
    need_uncompressed: bool,
    need_compressed: bool,
    need_offset: bool,
) -> Result<(Option<u64>, Option<u64>, Option<u64>)> {
    let mut cursor = 0usize;
    while cursor + 4 <= extra.len() {
        let tag = u16::from_le_bytes([extra[cursor], extra[cursor + 1]]);
        let size = u16::from_le_bytes([extra[cursor + 2], extra[cursor + 3]]) as usize;
        let body = extra
            .get(cursor + 4..cursor + 4 + size)
            .ok_or_else(|| package("truncated zip64 extra field"))?;
        if tag == 0x0001 {
            let mut at = 0usize;
            let take = |at: &mut usize| -> Result<u64> {
                let value = body
                    .get(*at..*at + 8)
                    .and_then(|chunk| <[u8; 8]>::try_from(chunk).ok())
                    .map(u64::from_le_bytes)
                    .ok_or_else(|| package("truncated zip64 extra field"))?;
                *at += 8;
                Ok(value)
            };
            let uncompressed = need_uncompressed.then(|| take(&mut at)).transpose()?;
            let compressed = need_compressed.then(|| take(&mut at)).transpose()?;
            let offset = need_offset.then(|| take(&mut at)).transpose()?;
            return Ok((uncompressed, compressed, offset));
        }
        cursor += 4 + size;
    }
    Err(package("zip64 sizes without a zip64 extra field"))
}

/// Parse a ZIP package, verifying every CRC. Returns entries in
/// central-directory order. Sizes, counts, ratios, names and symlinks
/// are all enforced against `limits`; nothing panics on hostile input.
pub fn read_package(bytes: &[u8], limits: &PackageLimits) -> Result<Vec<(String, Vec<u8>)>> {
    if bytes.len() as u64 > limits.max_bytes {
        return Err(limit(format!(
            "package of {} bytes exceeds limit {}",
            bytes.len(),
            limits.max_bytes
        )));
    }
    let central = locate_central(bytes)?;
    if central.count > limits.max_entries as u64 {
        return Err(limit(format!(
            "package of {} entries exceeds limit {}",
            central.count, limits.max_entries
        )));
    }
    let central_end = central
        .offset
        .checked_add(central.size)
        .filter(|end| *end <= bytes.len() as u64)
        .ok_or_else(|| package("central directory out of range"))?;
    let _ = central_end;
    let mut entries = Vec::with_capacity(central.count.min(1 << 20) as usize);
    let mut names = HashSet::new();
    let mut folded = HashSet::new();
    let mut total_uncompressed: u64 = 0;
    let mut total_compressed: u64 = 0;
    let mut offset = central.offset;
    for _ in 0..central.count {
        let at = as_usize(offset, "central directory entry")?;
        if bytes.get(at..at + 4) != Some(b"PK\x01\x02") {
            return Err(package("central directory entry without signature"));
        }
        let made_by = get_u16(bytes, at + 4)?;
        let version_needed = get_u16(bytes, at + 6)?;
        if version_needed > 45 {
            return Err(package(format!(
                "entry needs unsupported zip features (version {version_needed})"
            )));
        }
        let flags = get_u16(bytes, at + 8)?;
        let method = get_u16(bytes, at + 10)?;
        if flags & 0x0001 != 0 {
            return Err(package("encrypted entries are refused"));
        }
        if flags & 0x0008 != 0 {
            return Err(package("data-descriptor entries are refused"));
        }
        let crc = get_u32(bytes, at + 16)?;
        let compressed32 = get_u32(bytes, at + 20)?;
        let uncompressed32 = get_u32(bytes, at + 24)?;
        let name_len = get_u16(bytes, at + 28)? as u64;
        let extra_len = get_u16(bytes, at + 30)? as u64;
        let comment_len = get_u16(bytes, at + 32)? as u64;
        let disk = get_u16(bytes, at + 34)?;
        if disk != 0 {
            return Err(package("multi-disk packages are refused"));
        }
        let external_attrs = get_u32(bytes, at + 38)?;
        let header_off32 = get_u32(bytes, at + 42)?;
        // Unix symlink or directory entries never become files.
        if made_by >> 8 == 3 {
            let mode = external_attrs >> 16;
            if mode & 0o170_000 == 0o120_000 {
                return Err(package("symlink entries are refused"));
            }
            if mode & 0o170_000 == 0o040_000 {
                return Err(package("directory entries are refused"));
            }
        }
        let name_start = offset + 46;
        let name_end = name_start
            .checked_add(name_len)
            .filter(|end| *end <= bytes.len() as u64)
            .ok_or_else(|| package("entry name out of range"))?;
        let (name_start, name_end) = (
            as_usize(name_start, "entry name")?,
            as_usize(name_end, "entry name")?,
        );
        let name = std::str::from_utf8(&bytes[name_start..name_end])
            .map_err(|_| package("entry name is not UTF-8"))?
            .to_string();
        validate_entry_name(&name, limits)?;
        if is_directory_name(&name) {
            return Err(package(format!("directory entries are refused: {name:?}")));
        }
        if !names.insert(name.clone()) {
            return Err(package(format!("duplicate entry name: {name:?}")));
        }
        if !folded.insert(name.to_lowercase()) {
            return Err(package(format!("case-conflicting entry name: {name:?}")));
        }
        offset = name_end as u64 + extra_len + comment_len;
        // Resolve sizes, following Zip64 extra fields when saturated.
        let need_sizes = compressed32 == 0xFFFF_FFFF
            || uncompressed32 == 0xFFFF_FFFF
            || header_off32 == 0xFFFF_FFFF;
        let (mut compressed, mut uncompressed, mut header_offset) = (
            compressed32 as u64,
            uncompressed32 as u64,
            header_off32 as u64,
        );
        if need_sizes {
            let extra_start = as_usize(offset - extra_len - comment_len, "zip64 extra")?;
            let extra_end = as_usize(offset - comment_len, "zip64 extra")?;
            let (extra_uncompressed, extra_compressed, extra_offset) = zip64_extra(
                &bytes[extra_start..extra_end],
                uncompressed32 == 0xFFFF_FFFF,
                compressed32 == 0xFFFF_FFFF,
                header_off32 == 0xFFFF_FFFF,
            )?;
            if let Some(value) = extra_uncompressed {
                uncompressed = value;
            }
            if let Some(value) = extra_compressed {
                compressed = value;
            }
            if let Some(value) = extra_offset {
                header_offset = value;
            }
        }
        if uncompressed > limits.max_entry_bytes {
            return Err(limit(format!("entry {name} exceeds entry limit")));
        }
        // Local header: signature plus a matching name, then data.
        let header_at = as_usize(header_offset, "local header")?;
        if bytes.get(header_at..header_at + 4) != Some(b"PK\x03\x04") {
            return Err(package(format!("entry {name} without local header")));
        }
        if get_u16(bytes, header_at + 8)? != method {
            return Err(package(format!("entry {name} method mismatch")));
        }
        let local_flags = get_u16(bytes, header_at + 6)?;
        if local_flags & 0x0008 != 0 {
            return Err(package(format!("entry {name} uses a data descriptor")));
        }
        let local_name_len = get_u16(bytes, header_at + 26)? as u64;
        let local_extra_len = get_u16(bytes, header_at + 28)? as u64;
        let data_start = header_offset + 30 + local_name_len + local_extra_len;
        let data_end = data_start
            .checked_add(compressed)
            .filter(|end| *end <= bytes.len() as u64)
            .ok_or_else(|| package(format!("entry {name} data out of range")))?;
        let (data_start, data_end) = (
            as_usize(data_start, "entry data")?,
            as_usize(data_end, "entry data")?,
        );
        // The local name must describe the same entry.
        let local_name_start = as_usize(header_offset + 30, "entry name")?;
        let local_name_end = local_name_start
            .checked_add(local_name_len as usize)
            .filter(|end| *end <= data_start)
            .ok_or_else(|| package(format!("entry {name} local name out of range")))?;
        if bytes.get(local_name_start..local_name_end) != Some(name.as_bytes()) {
            return Err(package(format!("entry {name} name mismatch")));
        }
        let stored = &bytes[data_start..data_end];
        if stored.len() as u64 != compressed {
            return Err(package(format!("entry {name} size mismatch")));
        }
        let data = match method {
            0 => stored.to_vec(),
            8 => {
                // Bounded inflation: a lying header can never force an
                // unbounded allocation.
                let mut decoder = DeflateDecoder::new(stored);
                let mut out = Vec::new();
                decoder
                    .by_ref()
                    .take(limits.max_entry_bytes + 1)
                    .read_to_end(&mut out)
                    .map_err(|error| package(format!("inflate failed: {error}")))?;
                if out.len() as u64 > limits.max_entry_bytes {
                    return Err(limit(format!("entry {name} exceeds entry limit")));
                }
                out
            }
            _ => {
                return Err(package(format!(
                    "entry {name} uses unsupported method {method}"
                )))
            }
        };
        if data.len() as u64 != uncompressed {
            return Err(package(format!("entry {name} size mismatch after decode")));
        }
        if crc32fast::hash(&data) != crc {
            return Err(package(format!("entry {name} CRC mismatch")));
        }
        check_ratio(&name, uncompressed, compressed, limits)?;
        total_uncompressed = total_uncompressed
            .checked_add(uncompressed)
            .filter(|total| *total <= limits.max_total_bytes)
            .ok_or_else(|| {
                limit(format!(
                    "package exceeds total budget {}",
                    limits.max_total_bytes
                ))
            })?;
        total_compressed = total_compressed.saturating_add(compressed);
        entries.push((name, data));
    }
    Ok(entries)
}

/// Refuse decompression bombs: extreme expansion ratios never reach
/// the allocator beyond their bounded entry cap.
fn check_ratio(
    name: &str,
    uncompressed: u64,
    compressed: u64,
    limits: &PackageLimits,
) -> Result<()> {
    if uncompressed == 0 {
        return Ok(());
    }
    if compressed == 0 {
        return Err(limit(format!("entry {name} expands from nothing")));
    }
    if uncompressed / compressed > limits.max_ratio {
        return Err(limit(format!(
            "entry {name} expands {uncompressed} from {compressed} bytes"
        )));
    }
    Ok(())
}

fn find_end_of_central_directory(bytes: &[u8]) -> Result<usize> {
    if bytes.len() < 22 {
        return Err(package("too small for a package"));
    }
    let start = bytes.len().saturating_sub(22 + u16::MAX as usize);
    for offset in (start..=bytes.len() - 22).rev() {
        if bytes.get(offset..offset + 4) == Some(b"PK\x05\x06") {
            return Ok(offset);
        }
    }
    Err(package("end of central directory not found"))
}

/// Complete package content to be written: document, embedded resource
/// blobs, optional preview thumbnail and namespaced extensions.
pub struct PackagePayload<'a> {
    pub document: &'a Document,
    pub blobs: &'a BTreeMap<ResourceId, Vec<u8>>,
    pub preview: Option<&'a [u8]>,
    pub extensions: &'a BTreeMap<String, Vec<u8>>,
}

/// Fully decoded PTND package: document, embedded blobs, optional
/// preview thumbnail and namespaced extension entries.
#[derive(Debug, Clone)]
pub struct LoadedPackage {
    pub document: Document,
    pub blobs: BTreeMap<ResourceId, Vec<u8>>,
    pub preview: Option<Vec<u8>>,
    pub extensions: BTreeMap<String, Vec<u8>>,
}

/// Save a full PTND package with previews and extensions.
pub fn save_package(payload: &PackagePayload<'_>, limits: &PackageLimits) -> Result<Vec<u8>> {
    use petunia_core::ResourceSource;
    payload.document.validate()?;
    let dto = petunia_core::DocumentDtoV1::from_document(payload.document);
    let document_bytes = serde_json::to_vec(&dto)
        .map_err(|error| EngineError::Manifest(format!("document encode failed: {error}")))?;
    let mut resource_entries: Vec<(String, Vec<u8>)> = Vec::new();
    for (id, record) in payload.document.resources.iter() {
        let ResourceSource::Embedded { entry } = &record.source else {
            continue;
        };
        let name = format!("{RESOURCES_PREFIX}{entry}");
        validate_entry_name(&name, limits)?;
        let Some(bytes) = payload.blobs.get(&id) else {
            return Err(EngineError::Manifest(format!(
                "embedded resource {id} has no bytes to save"
            )));
        };
        resource_entries.push((name, bytes.clone()));
    }
    resource_entries.sort_by(|left, right| left.0.cmp(&right.0));

    let mut extension_entries: Vec<(String, Vec<u8>)> = Vec::new();
    for (name, bytes) in payload.extensions {
        if !name.starts_with(EXTENSIONS_PREFIX) {
            return Err(EngineError::Manifest(format!(
                "extension entry must start with `{EXTENSIONS_PREFIX}`: {name:?}"
            )));
        }
        validate_entry_name(name, limits)?;
        extension_entries.push((name.clone(), bytes.clone()));
    }
    extension_entries.sort_by(|left, right| left.0.cmp(&right.0));

    let mut names = vec![MANIFEST_ENTRY.to_string(), DOCUMENT_ENTRY.to_string()];
    names.extend(resource_entries.iter().map(|(name, _)| name.clone()));
    if payload.preview.is_some() {
        names.push(PREVIEWS_ENTRY.to_string());
    }
    names.extend(extension_entries.iter().map(|(name, _)| name.clone()));
    names.sort();

    let manifest = PtndManifest::new(payload.document.id, names);
    let manifest_bytes = manifest.to_json()?;

    let mut entries = vec![
        (
            MANIFEST_ENTRY.to_string(),
            manifest_bytes,
            ZipMethod::Deflated,
        ),
        (
            DOCUMENT_ENTRY.to_string(),
            document_bytes,
            ZipMethod::Deflated,
        ),
    ];
    for (name, bytes) in resource_entries {
        entries.push((name, bytes, ZipMethod::Deflated));
    }
    if let Some(preview_bytes) = payload.preview {
        entries.push((
            PREVIEWS_ENTRY.to_string(),
            preview_bytes.to_vec(),
            ZipMethod::Stored,
        ));
    }
    for (name, bytes) in extension_entries {
        entries.push((name, bytes, ZipMethod::Deflated));
    }
    write_package(&entries, limits)
}

/// Save one document as a PTND package: manifest, versioned document
/// DTO and every embedded resource blob. Linked resources travel by
/// reference only; unknown extra blobs are never smuggled in.
pub fn save_document(
    document: &Document,
    blobs: &BTreeMap<ResourceId, Vec<u8>>,
    limits: &PackageLimits,
) -> Result<Vec<u8>> {
    save_package(
        &PackagePayload {
            document,
            blobs,
            preview: None,
            extensions: &BTreeMap::new(),
        },
        limits,
    )
}

/// Load a full PTND package: manifest, document, embedded blobs,
/// optional preview and namespaced extensions.
pub fn load_package(bytes: &[u8], limits: &PackageLimits) -> Result<LoadedPackage> {
    use petunia_core::ResourceSource;
    let entries = read_package(bytes, limits)?;
    let table: BTreeMap<&str, &[u8]> = entries
        .iter()
        .map(|(name, data)| (name.as_str(), data.as_slice()))
        .collect();
    let Some(manifest_bytes) = table.get(MANIFEST_ENTRY) else {
        return Err(EngineError::Manifest(
            "package has no manifest.json".to_string(),
        ));
    };
    let manifest: PtndManifest = serde_json::from_slice(manifest_bytes)
        .map_err(|error| EngineError::Manifest(format!("manifest parse failed: {error}")))?;
    manifest.validate()?;
    if !manifest.required_capabilities.is_empty() {
        return Err(EngineError::Capability(format!(
            "package requires unimplemented capabilities: {}",
            manifest.required_capabilities.join(", "),
        )));
    }
    let mut actual: Vec<String> = table.keys().map(|name| name.to_string()).collect();
    actual.sort();
    if actual != manifest.entries {
        return Err(EngineError::Manifest(format!(
            "package entries do not match the manifest: {} vs {}",
            actual.join(", "),
            manifest.entries.join(", "),
        )));
    }
    let Some(document_bytes) = table.get(DOCUMENT_ENTRY) else {
        return Err(EngineError::Manifest(
            "package has no document.json".to_string(),
        ));
    };
    let dto: petunia_core::DocumentDtoV1 = serde_json::from_slice(document_bytes)
        .map_err(|error| EngineError::Manifest(format!("document parse failed: {error}")))?;
    let document = dto.into_document()?;
    if manifest.document_id != document.id.to_string() {
        return Err(EngineError::Manifest(format!(
            "manifest identifies {}, document identifies {}",
            manifest.document_id, document.id,
        )));
    }
    let mut blobs = BTreeMap::new();
    for (id, record) in document.resources.iter() {
        let ResourceSource::Embedded { entry } = &record.source else {
            continue;
        };
        let name = format!("{RESOURCES_PREFIX}{entry}");
        let Some(bytes) = table.get(name.as_str()) else {
            return Err(EngineError::Manifest(format!(
                "embedded resource {id} has no {name} entry"
            )));
        };
        blobs.insert(id, bytes.to_vec());
    }
    let preview = table.get(PREVIEWS_ENTRY).map(|bytes| bytes.to_vec());
    let mut extensions = BTreeMap::new();
    for (name, bytes) in &table {
        if name.starts_with(EXTENSIONS_PREFIX) {
            extensions.insert(name.to_string(), bytes.to_vec());
        }
    }
    Ok(LoadedPackage {
        document,
        blobs,
        preview,
        extensions,
    })
}

/// Load one document from a PTND package: container safety, manifest
/// and schema checks, entry-set match, DTO validation and resource
/// coverage. Returns the document plus its embedded blobs keyed by
/// logical resource identity.
pub fn load_document(
    bytes: &[u8],
    limits: &PackageLimits,
) -> Result<(Document, BTreeMap<ResourceId, Vec<u8>>)> {
    let pkg = load_package(bytes, limits)?;
    Ok((pkg.document, pkg.blobs))
}

/// Content-addressed blob store with copy-on-write sharing.
///
/// Blobs with identical bytes share a single heap allocation via
/// `Arc<Vec<u8>>`. Cloned stores share their data until modified.
#[derive(Debug, Clone, Default)]
pub struct BlobStore {
    blobs: BTreeMap<ResourceId, Arc<Vec<u8>>>,
    by_hash: BTreeMap<petunia_core::ContentHash, Arc<Vec<u8>>>,
}

impl BlobStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or share bytes under a resource identity. If identical
    /// bytes already exist under another identity, they share the
    /// same backing allocation (COW deduplication).
    pub fn insert(&mut self, id: ResourceId, bytes: Vec<u8>) -> Arc<Vec<u8>> {
        let hash = petunia_core::ContentHash::new(&bytes);
        let shared = self
            .by_hash
            .entry(hash)
            .or_insert_with(|| Arc::new(bytes))
            .clone();
        self.blobs.insert(id, shared.clone());
        shared
    }

    /// Insert an already shared allocation.
    pub fn insert_shared(&mut self, id: ResourceId, shared: Arc<Vec<u8>>) {
        let hash = petunia_core::ContentHash::new(&shared);
        self.by_hash.entry(hash).or_insert_with(|| shared.clone());
        self.blobs.insert(id, shared);
    }

    /// Get shared blob bytes by resource identity.
    #[must_use]
    pub fn get(&self, id: ResourceId) -> Option<&Arc<Vec<u8>>> {
        self.blobs.get(&id)
    }

    /// Convert to owned byte maps for package persistence.
    #[must_use]
    pub fn to_map(&self) -> BTreeMap<ResourceId, Vec<u8>> {
        self.blobs
            .iter()
            .map(|(id, arc)| (*id, (**arc).clone()))
            .collect()
    }

    /// Build a store from an existing map, deduplicating identical blobs.
    #[must_use]
    pub fn from_map(map: &BTreeMap<ResourceId, Vec<u8>>) -> Self {
        let mut store = Self::new();
        for (id, bytes) in map {
            store.insert(*id, bytes.clone());
        }
        store
    }

    /// Number of tracked resources.
    #[must_use]
    pub fn len(&self) -> usize {
        self.blobs.len()
    }

    /// Number of unique backing allocations.
    #[must_use]
    pub fn unique_allocations(&self) -> usize {
        self.by_hash.len()
    }

    /// Bytes retained by deduplicated immutable allocations, including undo resources.
    #[must_use]
    pub fn retained_bytes(&self) -> usize {
        self.by_hash
            .values()
            .fold(0usize, |total, bytes| total.saturating_add(bytes.len()))
    }

    /// True when no blobs are tracked.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blobs.is_empty()
    }
}

/// Atomically replace `path` with `bytes`: complete synced file
/// first, platform rename second. The old file is never truncated
/// before the new one is whole.
pub fn save_atomic(path: &std::path::Path, bytes: &[u8]) -> Result<()> {
    atomic_write(path, bytes, "ptnd")
}

/// Atomically replace `path` with `bytes`, tagging the temporary
/// sibling with `tag`. Shared by package saves and recovery files.
pub(crate) fn atomic_write(path: &std::path::Path, bytes: &[u8], tag: &str) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| EngineError::Execution("save path needs a parent directory".to_string()))?;
    let parent = if parent.as_os_str().is_empty() {
        std::path::Path::new(".")
    } else {
        parent
    };
    let temporary = parent.join(format!(
        ".{}.tmp-{}.{tag}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("document"),
        uuid::Uuid::new_v4()
    ));
    let outcome = (|| {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|error| EngineError::Execution(format!("temporary create failed: {error}")))?;
        use std::io::Write;
        file.write_all(bytes)
            .map_err(|error| EngineError::Execution(format!("temporary write failed: {error}")))?;
        file.sync_all()
            .map_err(|error| EngineError::Execution(format!("temporary sync failed: {error}")))?;
        drop(file);
        std::fs::rename(&temporary, path)
            .map_err(|error| EngineError::Execution(format!("atomic replace failed: {error}")))?;
        // Persist the directory entry as well as the file's contents on Unix.
        #[cfg(unix)]
        std::fs::File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| EngineError::Execution(format!("directory sync failed: {error}")))?;
        Ok(())
    })();
    if outcome.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    outcome
}

use std::time::SystemTime;

/// Snapshot of a file's state on disk to detect concurrent external edits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFingerprint {
    pub content_hash: petunia_core::ContentHash,
    pub size_bytes: u64,
    pub modified: SystemTime,
}

impl FileFingerprint {
    /// Capture the fingerprint of a file on disk.
    pub fn capture(path: &std::path::Path) -> Result<Self> {
        let metadata = std::fs::metadata(path)
            .map_err(|error| EngineError::Execution(format!("failed to stat {path:?}: {error}")))?;
        let bytes = std::fs::read(path)
            .map_err(|error| EngineError::Execution(format!("failed to read {path:?}: {error}")))?;
        let content_hash = petunia_core::ContentHash::new(&bytes);
        Ok(Self {
            content_hash,
            size_bytes: metadata.len(),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        })
    }
}

/// Status of an external modification check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictStatus {
    /// File matches the baseline exactly.
    Clean,
    /// File was modified on disk by another process.
    ModifiedExternally {
        current_hash: petunia_core::ContentHash,
        disk_modified: SystemTime,
    },
    /// File was deleted on disk.
    Deleted,
}

/// Conflict detector comparing the current file state to the captured baseline.
#[derive(Debug, Clone)]
pub struct ExternalConflictDetector {
    baseline: FileFingerprint,
}

impl ExternalConflictDetector {
    /// Create a detector from a captured baseline.
    #[must_use]
    pub fn new(baseline: FileFingerprint) -> Self {
        Self { baseline }
    }

    /// Check the file against the baseline.
    #[must_use]
    pub fn check(&self, path: &std::path::Path) -> ConflictStatus {
        if !path.exists() {
            return ConflictStatus::Deleted;
        }
        match FileFingerprint::capture(path) {
            Ok(current) => {
                if current.content_hash == self.baseline.content_hash {
                    ConflictStatus::Clean
                } else {
                    ConflictStatus::ModifiedExternally {
                        current_hash: current.content_hash,
                        disk_modified: current.modified,
                    }
                }
            }
            Err(_) => ConflictStatus::Deleted,
        }
    }

    /// Assert no conflict exists, returning a typed `EngineError::Conflict` on mismatch.
    pub fn assert_safe_to_save(&self, path: &std::path::Path) -> Result<()> {
        match self.check(path) {
            ConflictStatus::Clean => Ok(()),
            ConflictStatus::ModifiedExternally { current_hash, .. } => {
                Err(EngineError::Conflict(format!(
                    "file {path:?} changed externally (current hash: {})",
                    current_hash.to_tagged()
                )))
            }
            ConflictStatus::Deleted => Err(EngineError::Conflict(format!(
                "file {path:?} was deleted on disk"
            ))),
        }
    }
}

/// Cross-process advisory lock retained by an OS file handle.
/// The metadata sidecar remains after release: unlinking it would let two
/// contenders lock different inodes for the same document.
#[derive(Debug)]
pub struct CooperativeFileLock {
    file: Option<std::fs::File>,
}

#[derive(Serialize, Deserialize)]
struct LockPayload {
    pid: u32,
    created_unix_ms: u64,
}

impl CooperativeFileLock {
    /// Acquire atomically; the kernel releases stale locks when the owner exits.
    pub fn acquire(target_file: &std::path::Path) -> Result<Self> {
        let mut lock_name = target_file.as_os_str().to_os_string();
        lock_name.push(".lock");
        let lock_path = std::path::PathBuf::from(lock_name);
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|error| EngineError::Execution(format!("open lock failed: {error}")))?;
        file.try_lock().map_err(|error| {
            EngineError::Conflict(format!("cannot acquire lock for {target_file:?}: {error}"))
        })?;
        let payload = LockPayload {
            pid: std::process::id(),
            created_unix_ms: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|duration| duration.as_millis() as u64)
                .unwrap_or(0),
        };
        let bytes = serde_json::to_vec(&payload)
            .map_err(|error| EngineError::Execution(format!("lock metadata failed: {error}")))?;
        use std::io::{Seek, Write};
        file.set_len(0)
            .and_then(|()| file.rewind())
            .and_then(|()| file.write_all(&bytes))
            .and_then(|()| file.sync_all())
            .map_err(|error| EngineError::Execution(format!("lock write failed: {error}")))?;
        Ok(Self { file: Some(file) })
    }

    /// Closing the retained handle releases the OS lock.
    pub fn release(&mut self) {
        self.file.take();
    }
}

impl Drop for CooperativeFileLock {
    fn drop(&mut self) {
        self.release();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_core::{Document, SceneNode, VectorPath};

    fn sample_entries() -> Vec<(String, Vec<u8>, ZipMethod)> {
        vec![
            (
                "manifest.json".to_string(),
                br#"{"format":"petunia-design-document","schema_version":1}"#.to_vec(),
                ZipMethod::Stored,
            ),
            (
                "document.json".to_string(),
                vec![b'x'; 5000],
                ZipMethod::Deflated,
            ),
        ]
    }

    fn document_with_resources() -> (Document, BTreeMap<ResourceId, Vec<u8>>) {
        use petunia_core::{
            ParentRef, ResourceKind, ResourceMetadata, ResourceRecord, ResourceSource,
        };
        let mut document = Document::new("package");
        let page = document.scene.default_page();
        document.scene.insert_node(SceneNode::new_path(
            "box",
            VectorPath::rect(0.0, 0.0, 10.0, 10.0),
            ParentRef::Page(page),
        ));
        let image_id = ResourceId::new_v4();
        document.resources.insert(ResourceRecord {
            id: image_id,
            kind: ResourceKind::Image,
            source: ResourceSource::new_embedded("images/logo.png").expect("entry"),
            content_hash: None,
            metadata: ResourceMetadata::default(),
        });
        let linked_id = ResourceId::new_v4();
        document.resources.insert(ResourceRecord {
            id: linked_id,
            kind: ResourceKind::Font,
            source: ResourceSource::new_linked("file:///fonts/a.ttf").expect("uri"),
            content_hash: None,
            metadata: ResourceMetadata::default(),
        });
        let mut blobs = BTreeMap::new();
        blobs.insert(image_id, vec![1, 2, 3, 4, 5]);
        (document, blobs)
    }

    #[test]
    fn package_round_trip_preserves_entries() {
        let bytes = write_package(&sample_entries(), &PackageLimits::default()).expect("writes");
        // Classic shapes stay classic: no Zip64 structures involved.
        assert!(!bytes.windows(4).any(|w| w == b"PK\x06\x06"));
        let back = read_package(&bytes, &PackageLimits::default()).expect("reads");
        assert_eq!(back.len(), 2);
        assert_eq!(back[0].0, "manifest.json");
        assert_eq!(back[1].1.len(), 5000);
    }

    #[test]
    fn tampered_crc_is_detected() {
        let mut bytes =
            write_package(&sample_entries(), &PackageLimits::default()).expect("writes");
        // First entry is stored: flip a payload byte to break its CRC.
        let data_offset = 30 + "manifest.json".len();
        bytes[data_offset] ^= 0xFF;
        assert!(read_package(&bytes, &PackageLimits::default()).is_err());
    }

    #[test]
    fn truncation_and_limits_are_rejected() {
        let bytes = write_package(&sample_entries(), &PackageLimits::default()).expect("writes");
        assert!(read_package(&bytes[..bytes.len() / 2], &PackageLimits::default()).is_err());
        assert!(read_package(b"not a package", &PackageLimits::default()).is_err());
        let tight = PackageLimits {
            max_entries: 1,
            ..PackageLimits::default()
        };
        assert!(write_package(&sample_entries(), &tight).is_err());
    }

    #[test]
    fn conflict_detector_identifies_external_modifications() {
        let dir = std::env::temp_dir().join(format!("petunia-conflict-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("doc.ptnd");
        let initial_bytes = b"initial content";
        std::fs::write(&path, initial_bytes).expect("writes");

        let baseline = FileFingerprint::capture(&path).expect("captures");
        let detector = ExternalConflictDetector::new(baseline);
        assert_eq!(detector.check(&path), ConflictStatus::Clean);
        assert!(detector.assert_safe_to_save(&path).is_ok());

        // External process modifies the file on disk:
        std::fs::write(&path, b"external edit").expect("writes");
        assert!(matches!(
            detector.check(&path),
            ConflictStatus::ModifiedExternally { .. }
        ));
        assert!(matches!(
            detector.assert_safe_to_save(&path),
            Err(EngineError::Conflict(_))
        ));

        // External process deletes the file:
        let _ = std::fs::remove_file(&path);
        assert_eq!(detector.check(&path), ConflictStatus::Deleted);
        assert!(matches!(
            detector.assert_safe_to_save(&path),
            Err(EngineError::Conflict(_))
        ));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cooperative_lock_prevents_concurrent_acquisition() {
        let dir = std::env::temp_dir().join(format!("petunia-lock-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("doc.ptnd");

        let lock1 = CooperativeFileLock::acquire(&path).expect("acquires lock");
        // Second instance attempts to acquire while active:
        assert!(matches!(
            CooperativeFileLock::acquire(&path),
            Err(EngineError::Conflict(_))
        ));

        // Dropping or releasing lock1 frees it:
        drop(lock1);
        let lock2 = CooperativeFileLock::acquire(&path).expect("acquires after drop");
        drop(lock2);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn atomic_save_writes_complete_files() {
        let dir = std::env::temp_dir().join(format!("petunia-ptnd-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("doc.ptnd");
        let bytes = write_package(&sample_entries(), &PackageLimits::default()).expect("writes");
        save_atomic(&path, &bytes).expect("saves");
        let back = std::fs::read(&path).expect("reads back");
        assert_eq!(back, bytes);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn document_save_load_round_trip() {
        let (document, blobs) = document_with_resources();
        let bytes = save_document(&document, &blobs, &PackageLimits::default()).expect("saves");
        let (back, back_blobs) = load_document(&bytes, &PackageLimits::default()).expect("loads");
        assert_eq!(back.id, document.id);
        assert_eq!(back.scene.len(), 1);
        assert_eq!(back_blobs.len(), 1);
        assert!(back.validate().is_ok());
    }

    #[test]
    fn package_with_preview_and_extensions_round_trip() {
        let (document, blobs) = document_with_resources();
        let preview = vec![137, 80, 78, 71, 13, 10, 26, 10]; // PNG magic
        let mut extensions = BTreeMap::new();
        extensions.insert(
            "extensions/com.example.effect/config.json".to_string(),
            br#"{"speed":1}"#.to_vec(),
        );

        let payload = PackagePayload {
            document: &document,
            blobs: &blobs,
            preview: Some(&preview),
            extensions: &extensions,
        };
        let bytes = save_package(&payload, &PackageLimits::default()).expect("saves");
        let loaded = load_package(&bytes, &PackageLimits::default()).expect("loads");
        assert_eq!(loaded.document.id, document.id);
        assert_eq!(loaded.blobs.len(), 1);
        assert_eq!(loaded.preview, Some(preview));
        assert_eq!(loaded.extensions.len(), 1);
        assert_eq!(
            loaded
                .extensions
                .get("extensions/com.example.effect/config.json"),
            Some(&br#"{"speed":1}"#.to_vec())
        );
    }

    #[test]
    fn blob_store_cow_and_deduplication() {
        let mut store = BlobStore::new();
        let id1 = ResourceId::new_v4();
        let id2 = ResourceId::new_v4();
        let id3 = ResourceId::new_v4();

        let data1 = vec![1, 2, 3, 4, 5];
        let data2 = vec![1, 2, 3, 4, 5]; // Identical bytes
        let data3 = vec![6, 7, 8];

        let arc1 = store.insert(id1, data1);
        let arc2 = store.insert(id2, data2);
        let arc3 = store.insert(id3, data3);

        assert_eq!(store.len(), 3);
        assert_eq!(
            store.unique_allocations(),
            2,
            "identical bytes share backing storage"
        );
        assert!(
            Arc::ptr_eq(&arc1, &arc2),
            "identical bytes share the exact same Arc pointer"
        );
        assert!(!Arc::ptr_eq(&arc1, &arc3));
    }

    #[test]
    fn missing_embedded_bytes_fail_the_save() {
        let (document, _) = document_with_resources();
        assert!(save_document(&document, &BTreeMap::new(), &PackageLimits::default()).is_err());
    }

    #[test]
    fn manifest_mismatches_are_refused() {
        let (document, blobs) = document_with_resources();
        let bytes = save_document(&document, &blobs, &PackageLimits::default()).expect("saves");
        let entries = read_package(&bytes, &PackageLimits::default()).expect("reads");
        let table: BTreeMap<String, Vec<u8>> = entries.into_iter().collect();
        let rewrap = |mut tampered: BTreeMap<String, Vec<u8>>| {
            let rebuilt: Vec<(String, Vec<u8>, ZipMethod)> = std::mem::take(&mut tampered)
                .into_iter()
                .map(|(name, data)| (name, data, ZipMethod::Stored))
                .collect();
            write_package(&rebuilt, &PackageLimits::default()).expect("rewrites")
        };
        // Unknown schema.
        let mut manifest: PtndManifest =
            serde_json::from_slice(&table["manifest.json"]).expect("parses");
        manifest.schema_version = 999;
        let mut tampered = table.clone();
        tampered.insert(
            "manifest.json".to_string(),
            serde_json::to_vec(&manifest).expect("encodes"),
        );
        assert!(load_document(&rewrap(tampered), &PackageLimits::default()).is_err());
        // Unknown required capability.
        let mut manifest: PtndManifest =
            serde_json::from_slice(&table["manifest.json"]).expect("parses");
        manifest.required_capabilities = vec!["future.render.v9".to_string()];
        let mut tampered = table.clone();
        tampered.insert(
            "manifest.json".to_string(),
            serde_json::to_vec(&manifest).expect("encodes"),
        );
        assert!(load_document(&rewrap(tampered), &PackageLimits::default()).is_err());
        // Entry set mismatch: drop the resource entry.
        let mut tampered = table.clone();
        tampered.remove("resources/images/logo.png");
        assert!(load_document(&rewrap(tampered), &PackageLimits::default()).is_err());
    }

    #[test]
    fn hostile_names_never_become_files() {
        for hostile in [
            "../evil.json",
            "/absolute.json",
            "a/../../b.json",
            "a//b.json",
            "a/./b.json",
            "back\\slash.json",
            "",
        ] {
            assert!(
                validate_entry_name(hostile, &PackageLimits::default()).is_err(),
                "{hostile:?}"
            );
            // The writer refuses them too.
            assert!(write_package(
                &[(hostile.to_string(), b"x".to_vec(), ZipMethod::Stored,)],
                &PackageLimits::default(),
            )
            .is_err());
        }
        // Raw traversal bytes from a foreign writer fail at load.
        let hostile = raw_package(
            &[RawEntry {
                name: b"../evil.json".to_vec(),
                data: b"x".to_vec(),
                ..RawEntry::default()
            }],
            false,
        );
        assert!(read_package(&hostile, &PackageLimits::default()).is_err());
        // Duplicate and case-conflicting names fail at load.
        let dup = raw_package(
            &[
                RawEntry {
                    name: b"a.json".to_vec(),
                    data: b"1".to_vec(),
                    ..RawEntry::default()
                },
                RawEntry {
                    name: b"a.json".to_vec(),
                    data: b"2".to_vec(),
                    ..RawEntry::default()
                },
            ],
            false,
        );
        assert!(read_package(&dup, &PackageLimits::default()).is_err());
        let folded = raw_package(
            &[
                RawEntry {
                    name: b"Logo.PNG".to_vec(),
                    data: b"1".to_vec(),
                    ..RawEntry::default()
                },
                RawEntry {
                    name: b"logo.png".to_vec(),
                    data: b"2".to_vec(),
                    ..RawEntry::default()
                },
            ],
            false,
        );
        assert!(read_package(&folded, &PackageLimits::default()).is_err());
        // Symlink placeholders never become files.
        let link = raw_package(
            &[RawEntry {
                name: b"link".to_vec(),
                data: b"x".to_vec(),
                made_by: 3 << 8,
                external_attrs: 0o120_777 << 16,
                ..RawEntry::default()
            }],
            false,
        );
        assert!(read_package(&link, &PackageLimits::default()).is_err());
    }

    #[test]
    fn zip64_packages_parse_with_64_bit_sizes() {
        let raw = raw_package(
            &[RawEntry {
                name: b"document.json".to_vec(),
                data: b"{}".to_vec(),
                ..RawEntry::default()
            }],
            true,
        );
        let back = read_package(&raw, &PackageLimits::default()).expect("zip64 reads");
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].0, "document.json");
        assert_eq!(back[0].1, b"{}");
    }

    #[test]
    fn lying_headers_and_bombs_stay_bounded() {
        // Declared size far above the limit: refused before inflation.
        let lying = raw_package(
            &[RawEntry {
                name: b"big.bin".to_vec(),
                data: vec![0u8; 64],
                uncompressed_override: Some(1 << 40),
                ..RawEntry::default()
            }],
            false,
        );
        let tight = PackageLimits {
            max_entry_bytes: 1 << 20,
            ..PackageLimits::default()
        };
        assert!(read_package(&lying, &tight).is_err());
        // Extreme expansion ratio: refused without allocating it.
        let bomb = write_package(
            &[(
                "bomb.bin".to_string(),
                vec![b'x'; 5000],
                ZipMethod::Deflated,
            )],
            &PackageLimits::default(),
        )
        .expect("writes");
        let strict = PackageLimits {
            max_ratio: 2,
            ..PackageLimits::default()
        };
        assert!(read_package(&bomb, &strict).is_err());
        assert!(read_package(&bomb, &PackageLimits::default()).is_ok());
        // Total budget across individually fine entries.
        let pair = write_package(
            &[
                ("a.bin".to_string(), vec![b'a'; 1000], ZipMethod::Deflated),
                ("b.bin".to_string(), vec![b'b'; 1000], ZipMethod::Deflated),
            ],
            &PackageLimits::default(),
        )
        .expect("writes");
        let capped = PackageLimits {
            max_total_bytes: 1500,
            ..PackageLimits::default()
        };
        assert!(read_package(&pair, &capped).is_err());
    }

    /// One raw stored entry with explicit knobs for hostile shapes the
    /// safe writer never emits.
    #[derive(Default)]
    struct RawEntry {
        name: Vec<u8>,
        data: Vec<u8>,
        method: u16,
        made_by: u16,
        external_attrs: u32,
        uncompressed_override: Option<u64>,
    }

    /// Minimal hand-built package: local headers, central directory
    /// and end record, optionally with Zip64 structures.
    fn raw_package(entries: &[RawEntry], zip64: bool) -> Vec<u8> {
        use crc32fast::hash;
        let mut out = Vec::new();
        let mut central = Vec::new();
        for entry in entries {
            let crc = hash(&entry.data);
            let uncompressed = entry
                .uncompressed_override
                .unwrap_or(entry.data.len() as u64);
            let header_offset = out.len() as u64;
            out.extend_from_slice(b"PK\x03\x04");
            put_u16(&mut out, if zip64 { 45 } else { 20 });
            put_u16(&mut out, 0);
            put_u16(&mut out, entry.method);
            put_u16(&mut out, 0);
            put_u16(&mut out, 0);
            put_u32(&mut out, crc);
            if zip64 {
                put_u32(&mut out, 0xFFFF_FFFF);
                put_u32(&mut out, 0xFFFF_FFFF);
            } else {
                put_u32(&mut out, entry.data.len() as u32);
                put_u32(&mut out, uncompressed.min(u32::MAX as u64) as u32);
            }
            put_u16(&mut out, entry.name.len() as u16);
            if zip64 {
                put_u16(&mut out, 28);
                out.extend_from_slice(&entry.name);
                put_u16(&mut out, 0x0001);
                put_u16(&mut out, 24);
                put_u64(&mut out, uncompressed);
                put_u64(&mut out, entry.data.len() as u64);
                put_u64(&mut out, header_offset);
            } else {
                put_u16(&mut out, 0);
                out.extend_from_slice(&entry.name);
            }
            out.extend_from_slice(&entry.data);
            central.extend_from_slice(b"PK\x01\x02");
            put_u16(&mut central, entry.made_by);
            put_u16(&mut central, if zip64 { 45 } else { 20 });
            put_u16(&mut central, 0);
            put_u16(&mut central, entry.method);
            put_u16(&mut central, 0);
            put_u16(&mut central, 0);
            put_u32(&mut central, crc);
            if zip64 {
                put_u32(&mut central, 0xFFFF_FFFF);
                put_u32(&mut central, 0xFFFF_FFFF);
            } else {
                put_u32(&mut central, entry.data.len() as u32);
                put_u32(&mut central, uncompressed.min(u32::MAX as u64) as u32);
            }
            put_u16(&mut central, entry.name.len() as u16);
            if zip64 {
                put_u16(&mut central, 28);
            } else {
                put_u16(&mut central, 0);
            }
            put_u16(&mut central, 0);
            put_u16(&mut central, 0);
            put_u16(&mut central, 0);
            put_u32(&mut central, entry.external_attrs);
            if zip64 {
                put_u32(&mut central, 0xFFFF_FFFF);
            } else {
                put_u32(&mut central, header_offset as u32);
            }
            central.extend_from_slice(&entry.name);
            if zip64 {
                put_u16(&mut central, 0x0001);
                put_u16(&mut central, 24);
                put_u64(&mut central, uncompressed);
                put_u64(&mut central, entry.data.len() as u64);
                put_u64(&mut central, header_offset);
            }
        }
        let central_offset = out.len() as u64;
        let central_size = central.len() as u64;
        out.extend_from_slice(&central);
        if zip64 {
            let eocd64 = out.len() as u64;
            out.extend_from_slice(b"PK\x06\x06");
            put_u64(&mut out, 44);
            put_u16(&mut out, 45);
            put_u16(&mut out, 45);
            put_u32(&mut out, 0);
            put_u32(&mut out, 0);
            put_u64(&mut out, entries.len() as u64);
            put_u64(&mut out, entries.len() as u64);
            put_u64(&mut out, central_size);
            put_u64(&mut out, central_offset);
            out.extend_from_slice(b"PK\x06\x07");
            put_u32(&mut out, 0);
            put_u64(&mut out, eocd64);
            put_u32(&mut out, 1);
        }
        out.extend_from_slice(b"PK\x05\x06");
        put_u16(&mut out, 0);
        put_u16(&mut out, 0);
        if zip64 {
            put_u16(&mut out, 0xFFFF);
            put_u16(&mut out, 0xFFFF);
        } else {
            put_u16(&mut out, entries.len() as u16);
            put_u16(&mut out, entries.len() as u16);
        }
        if zip64 {
            put_u32(&mut out, 0xFFFF_FFFF);
            put_u32(&mut out, 0xFFFF_FFFF);
        } else {
            put_u32(&mut out, central_size as u32);
            put_u32(&mut out, central_offset as u32);
        }
        put_u16(&mut out, 0);
        out
    }
}
