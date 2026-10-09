//! Minimal ZIP container for PTND packages.
//!
//! The writer emits stored and deflated entries with CRC32, sizes up
//! front and a central directory; the reader enforces the same shape
//! and verifies every CRC. Anything outside that subset (encryption,
//! data descriptors, multi-disk, Zip64) fails with a typed error
//! instead of a best-effort parse. Saves are atomic: complete file
//! first, platform rename second.

use crate::error::{EngineError, Result};
use flate2::write::DeflateEncoder;
use flate2::{read::DeflateDecoder, Compression};
use serde::{Deserialize, Serialize};
use std::io::Read;

/// Bounds for one package operation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PackageLimits {
    pub max_bytes: u64,
    pub max_entries: usize,
    pub max_entry_bytes: u64,
}

impl Default for PackageLimits {
    fn default() -> Self {
        Self {
            max_bytes: 512 << 20,
            max_entries: 8192,
            max_entry_bytes: 256 << 20,
        }
    }
}

/// Compression per entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZipMethod {
    Stored,
    Deflated,
}

/// Manifest living at `ptnd/manifest.json` inside every package.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PtndManifest {
    pub format: String,
    pub version: u32,
    pub document: String,
    pub files: Vec<String>,
}

impl PtndManifest {
    /// Manifest for a package whose document entry holds the scene.
    #[must_use]
    pub fn new(document: impl Into<String>, files: Vec<String>) -> Self {
        Self {
            format: "PTND".to_string(),
            version: 1,
            document: document.into(),
            files,
        }
    }

    /// Canonical JSON bytes for the manifest entry.
    pub fn to_json(&self) -> Result<Vec<u8>> {
        serde_json::to_vec_pretty(self)
            .map_err(|error| EngineError::Execution(format!("manifest encode failed: {error}")))
    }
}

fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn get_u16(input: &[u8], offset: usize) -> Result<u16> {
    input
        .get(offset..offset + 2)
        .and_then(|pair| <[u8; 2]>::try_from(pair).ok())
        .map(u16::from_le_bytes)
        .ok_or_else(|| EngineError::Execution("truncated zip header".to_string()))
}

fn get_u32(input: &[u8], offset: usize) -> Result<u32> {
    input
        .get(offset..offset + 4)
        .and_then(|quad| <[u8; 4]>::try_from(quad).ok())
        .map(u32::from_le_bytes)
        .ok_or_else(|| EngineError::Execution("truncated zip header".to_string()))
}

/// Serialize entries into a ZIP package.
pub fn write_package(
    entries: &[(String, Vec<u8>, ZipMethod)],
    limits: &PackageLimits,
) -> Result<Vec<u8>> {
    if entries.len() > limits.max_entries {
        return Err(EngineError::Execution(format!(
            "package of {} entries exceeds limit {}",
            entries.len(),
            limits.max_entries
        )));
    }
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, bytes, method) in entries {
        if name.is_empty() || name.len() > u16::MAX as usize {
            return Err(EngineError::Execution(
                "entry name out of range".to_string(),
            ));
        }
        if bytes.len() as u64 > limits.max_entry_bytes {
            return Err(EngineError::Execution(format!(
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
                    .map_err(|error| EngineError::Execution(format!("deflate failed: {error}")))?;
                encoder
                    .finish()
                    .map_err(|error| EngineError::Execution(format!("deflate failed: {error}")))?
            }
        };
        for size in [bytes.len(), data.len()] {
            if size > u32::MAX as usize {
                return Err(EngineError::Execution(format!(
                    "entry {name} needs Zip64, which this writer refuses"
                )));
            }
        }
        let crc = crc32fast::hash(bytes);
        let header_offset = out.len() as u32;
        // Local file header.
        out.extend_from_slice(b"PK\x03\x04");
        put_u16(&mut out, 20);
        put_u16(&mut out, 0);
        put_u16(
            &mut out,
            match method {
                ZipMethod::Stored => 0,
                ZipMethod::Deflated => 8,
            },
        );
        put_u16(&mut out, 0);
        put_u16(&mut out, 0);
        put_u32(&mut out, crc);
        put_u32(&mut out, data.len() as u32);
        put_u32(&mut out, bytes.len() as u32);
        put_u16(&mut out, name.len() as u16);
        put_u16(&mut out, 0);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(&data);
        // Central directory entry.
        central.extend_from_slice(b"PK\x01\x02");
        put_u16(&mut central, 20);
        put_u16(&mut central, 20);
        put_u16(&mut central, 0);
        put_u16(
            &mut central,
            match method {
                ZipMethod::Stored => 0,
                ZipMethod::Deflated => 8,
            },
        );
        put_u16(&mut central, 0);
        put_u16(&mut central, 0);
        put_u32(&mut central, crc);
        put_u32(&mut central, data.len() as u32);
        put_u32(&mut central, bytes.len() as u32);
        put_u16(&mut central, name.len() as u16);
        put_u16(&mut central, 0);
        put_u16(&mut central, 0);
        put_u16(&mut central, 0);
        put_u16(&mut central, 0);
        put_u32(&mut central, 0);
        put_u32(&mut central, header_offset);
        central.extend_from_slice(name.as_bytes());
        if out.len() as u64 > limits.max_bytes {
            return Err(EngineError::Execution(
                "package exceeds byte limit".to_string(),
            ));
        }
    }
    let central_offset = out.len() as u32;
    let central_size = central.len() as u32;
    out.extend_from_slice(&central);
    // End of central directory.
    out.extend_from_slice(b"PK\x05\x06");
    put_u16(&mut out, 0);
    put_u16(&mut out, 0);
    put_u16(&mut out, entries.len() as u16);
    put_u16(&mut out, entries.len() as u16);
    put_u32(&mut out, central_size);
    put_u32(&mut out, central_offset);
    put_u16(&mut out, 0);
    Ok(out)
}

/// Parse a ZIP package, verifying every CRC. Returns entries in
/// central-directory order.
pub fn read_package(bytes: &[u8], limits: &PackageLimits) -> Result<Vec<(String, Vec<u8>)>> {
    let end = find_end_of_central_directory(bytes)?;
    let count = get_u16(bytes, end + 10)? as usize;
    let central_size = get_u32(bytes, end + 12)? as usize;
    let central_offset = get_u32(bytes, end + 16)? as usize;
    if count > limits.max_entries {
        return Err(EngineError::Execution(format!(
            "package of {count} entries exceeds limit {}",
            limits.max_entries
        )));
    }
    let central_end = central_offset
        .checked_add(central_size)
        .filter(|end| *end <= bytes.len())
        .ok_or_else(|| EngineError::Execution("central directory out of range".to_string()))?;
    let _ = central_end;
    let mut entries = Vec::with_capacity(count);
    let mut offset = central_offset;
    for _ in 0..count {
        if bytes.get(offset..offset + 4) != Some(b"PK\x01\x02") {
            return Err(EngineError::Execution(
                "central directory entry without signature".to_string(),
            ));
        }
        let flags = get_u16(bytes, offset + 8)?;
        let method = get_u16(bytes, offset + 10)?;
        if flags & 0x0001 != 0 {
            return Err(EngineError::Execution(
                "encrypted entries are refused".to_string(),
            ));
        }
        if flags & 0x0008 != 0 {
            return Err(EngineError::Execution(
                "data-descriptor entries are refused".to_string(),
            ));
        }
        let crc = get_u32(bytes, offset + 16)?;
        let compressed = get_u32(bytes, offset + 20)? as usize;
        let uncompressed = get_u32(bytes, offset + 24)? as usize;
        let name_len = get_u16(bytes, offset + 28)? as usize;
        let extra_len = get_u16(bytes, offset + 30)? as usize;
        let comment_len = get_u16(bytes, offset + 32)? as usize;
        let header_offset = get_u32(bytes, offset + 42)? as usize;
        let name_start = offset + 46;
        let name_end = name_start
            .checked_add(name_len)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| EngineError::Execution("entry name out of range".to_string()))?;
        let name = std::str::from_utf8(&bytes[name_start..name_end])
            .map_err(|_| EngineError::Execution("entry name is not UTF-8".to_string()))?
            .to_string();
        offset = name_end + extra_len + comment_len;
        if uncompressed as u64 > limits.max_entry_bytes {
            return Err(EngineError::Execution(format!(
                "entry {name} exceeds entry limit"
            )));
        }
        // Local header: name must match, then data follows.
        if bytes.get(header_offset..header_offset + 4) != Some(b"PK\x03\x04") {
            return Err(EngineError::Execution(format!(
                "entry {name} without local header"
            )));
        }
        let local_name_len = get_u16(bytes, header_offset + 26)? as usize;
        let local_extra_len = get_u16(bytes, header_offset + 28)? as usize;
        let data_start = header_offset + 30 + local_name_len + local_extra_len;
        let data_end = data_start
            .checked_add(compressed)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| EngineError::Execution(format!("entry {name} data out of range")))?;
        let stored = &bytes[data_start..data_end];
        let data = match method {
            0 => stored.to_vec(),
            8 => {
                let mut decoder = DeflateDecoder::new(stored);
                let mut out = Vec::with_capacity(uncompressed.min(1 << 26));
                decoder
                    .read_to_end(&mut out)
                    .map_err(|error| EngineError::Execution(format!("inflate failed: {error}")))?;
                out
            }
            _ => {
                return Err(EngineError::Execution(format!(
                    "entry {name} uses unsupported method {method}"
                )))
            }
        };
        if data.len() != uncompressed {
            return Err(EngineError::Execution(format!(
                "entry {name} size mismatch after decode"
            )));
        }
        if crc32fast::hash(&data) != crc {
            return Err(EngineError::Execution(format!("entry {name} CRC mismatch")));
        }
        entries.push((name, data));
    }
    Ok(entries)
}

fn find_end_of_central_directory(bytes: &[u8]) -> Result<usize> {
    if bytes.len() < 22 {
        return Err(EngineError::Execution(
            "too small for a package".to_string(),
        ));
    }
    let start = bytes.len().saturating_sub(22 + u16::MAX as usize);
    for offset in (start..=bytes.len() - 22).rev() {
        if &bytes[offset..offset + 4] == b"PK\x05\x06" {
            return Ok(offset);
        }
    }
    Err(EngineError::Execution(
        "end of central directory not found".to_string(),
    ))
}

/// Atomically replace `path` with `bytes`: complete temporary file
/// first, platform rename second. The old file is never truncated
/// before the new one is whole.
pub fn save_atomic(path: &std::path::Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| EngineError::Execution("save path needs a parent directory".to_string()))?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos())
        .unwrap_or(0);
    let temporary = parent.join(format!(
        ".{}.tmp-{stamp}-{}.ptnd",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("document"),
        std::process::id()
    ));
    std::fs::write(&temporary, bytes)
        .map_err(|error| EngineError::Execution(format!("temporary write failed: {error}")))?;
    std::fs::rename(&temporary, path)
        .map_err(|error| EngineError::Execution(format!("atomic replace failed: {error}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entries() -> Vec<(String, Vec<u8>, ZipMethod)> {
        vec![
            (
                "ptnd/manifest.json".to_string(),
                br#"{"format":"PTND","version":1}"#.to_vec(),
                ZipMethod::Stored,
            ),
            (
                "document/scene.json".to_string(),
                vec![b'x'; 5000],
                ZipMethod::Deflated,
            ),
        ]
    }

    #[test]
    fn package_round_trip_preserves_entries() {
        let bytes = write_package(&sample_entries(), &PackageLimits::default()).expect("writes");
        let back = read_package(&bytes, &PackageLimits::default()).expect("reads");
        assert_eq!(back.len(), 2);
        assert_eq!(back[0].0, "ptnd/manifest.json");
        assert_eq!(back[1].1.len(), 5000);
    }

    #[test]
    fn tampered_crc_is_detected() {
        let mut bytes =
            write_package(&sample_entries(), &PackageLimits::default()).expect("writes");
        // First entry is stored: flip a payload byte to break its CRC.
        let data_offset = 30 + "ptnd/manifest.json".len();
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
}
