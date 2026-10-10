//! Standard encoders behind the Core's authorial generator specifications.
//!
//! QR uses Nayuki's MIT-licensed `qrcodegen`; barcodes use the MIT-licensed
//! `barcoders`. Geometry is derived in exact module coordinates. No encoder
//! type crosses the Engine boundary, and no scene nodes are allocated.

use petunia_core::generated::{
    BarcodeSpec, BarcodeSymbology, GeneratorSpec, QrCodeSpec, QrErrorCorrection, QrMaskPolicy,
    QrPayload, QrVersionPolicy,
};
use petunia_core::{ContourId, NodeId, VectorPath};
use qrcodegen::{Mask, QrCode, QrCodeEcc, QrSegment, Version};
use std::collections::BTreeMap;
use thiserror::Error;

/// A complete compound path and its preferred untransformed bounds.
#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedGeometry {
    pub path: VectorPath,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GeneratorError {
    #[error("generator payload is empty or exceeds the operational limit")]
    InvalidPayload,
    #[error("payload does not fit the requested QR version")]
    PayloadTooLarge,
    #[error("QR version must be in 1..=40")]
    UnsupportedVersion,
    #[error("QR mask must be in 0..=7")]
    InvalidMask,
    #[error("quiet zone is below the interoperability minimum or is not finite")]
    InvalidQuietZone,
    #[error("invalid barcode character or data length")]
    InvalidCharacter,
    #[error("invalid barcode check digit")]
    InvalidChecksum,
    #[error("barcode dimensions must be finite and positive")]
    InvalidDimensions,
    #[error("barcode symbology is outside the v0.1 contract")]
    UnsupportedSymbology,
}

/// An owned module matrix. Quiet zones are geometry margins, not matrix data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrMatrix {
    pub size: usize,
    pub dark: Vec<bool>,
}

pub trait QrEncoder {
    fn encode(&self, spec: &QrCodeSpec) -> Result<QrMatrix, GeneratorError>;
}

#[derive(Debug, Default)]
pub struct StandardQrEncoder;

impl QrEncoder for StandardQrEncoder {
    fn encode(&self, spec: &QrCodeSpec) -> Result<QrMatrix, GeneratorError> {
        let QrPayload::Text(text) = &spec.payload;
        if text.is_empty() || text.len() > 16_384 {
            return Err(GeneratorError::InvalidPayload);
        }
        if spec.quiet_zone_modules < 4 {
            return Err(GeneratorError::InvalidQuietZone);
        }
        let (minimum, maximum) = match spec.version {
            QrVersionPolicy::Auto => (1, 40),
            QrVersionPolicy::Fixed(version) if (1..=40).contains(&version) => (version, version),
            QrVersionPolicy::Fixed(_) => return Err(GeneratorError::UnsupportedVersion),
        };
        let mask = match spec.mask {
            QrMaskPolicy::Auto => None,
            QrMaskPolicy::Fixed(mask) if mask <= 7 => Some(Mask::new(mask)),
            QrMaskPolicy::Fixed(_) => return Err(GeneratorError::InvalidMask),
        };
        let correction = match spec.error_correction {
            QrErrorCorrection::Low => QrCodeEcc::Low,
            QrErrorCorrection::Medium => QrCodeEcc::Medium,
            QrErrorCorrection::Quartile => QrCodeEcc::Quartile,
            QrErrorCorrection::High => QrCodeEcc::High,
        };
        // ECI 26 identifies UTF-8 for interoperable non-ASCII text decoding.
        let mut segments = QrSegment::make_segments(text);
        if !text.is_ascii() {
            segments.insert(0, QrSegment::make_eci(26));
        }
        let code = QrCode::encode_segments_advanced(
            &segments,
            correction,
            Version::new(minimum),
            Version::new(maximum),
            mask,
            false, // Preserve the authorial ECC; do not silently boost it.
        )
        .map_err(|_| GeneratorError::PayloadTooLarge)?;
        let size = code.size() as usize;
        let dark = (0..size * size)
            .map(|index| code.get_module((index % size) as i32, (index / size) as i32))
            .collect();
        Ok(QrMatrix { size, dark })
    }
}

/// Evaluate without mutating the specification or any Document state.
pub fn evaluate_generator(spec: &GeneratorSpec) -> Result<GeneratedGeometry, GeneratorError> {
    let mut geometry = match spec {
        GeneratorSpec::QrCode(spec) => {
            let matrix = StandardQrEncoder.encode(spec)?;
            let margin = f64::from(spec.quiet_zone_modules);
            GeneratedGeometry {
                path: coalesce_modules(&matrix.dark, matrix.size, matrix.size, margin),
                width: matrix.size as f64 + 2.0 * margin,
                height: matrix.size as f64 + 2.0 * margin,
            }
        }
        GeneratorSpec::Barcode(spec) => {
            let modules = barcode_modules(spec)?;
            let width = modules.len() as f64 + 2.0 * spec.quiet_zone_modules;
            let mut path = VectorPath::new();
            let mut start = 0;
            while start < modules.len() {
                if modules[start] == 0 {
                    start += 1;
                    continue;
                }
                let mut end = start + 1;
                while end < modules.len() && modules[end] == 1 {
                    end += 1;
                }
                path.contours.extend(
                    VectorPath::rect(
                        spec.quiet_zone_modules + start as f64,
                        0.0,
                        (end - start) as f64,
                        spec.bar_height_modules,
                    )
                    .contours,
                );
                start = end;
            }
            GeneratedGeometry {
                path,
                width,
                height: spec.bar_height_modules,
            }
        }
    };
    let seed = serde_json::to_vec(spec).map_err(|_| GeneratorError::InvalidPayload)?;
    stabilize_path(&mut geometry.path, &seed);
    Ok(geometry)
}

/// Validated barcode modules, including guards and checksum but excluding margins.
pub fn barcode_modules(spec: &BarcodeSpec) -> Result<Vec<u8>, GeneratorError> {
    use barcoders::sym::{code128::Code128, ean13::EAN13};
    if !spec.bar_height_modules.is_finite() || spec.bar_height_modules <= 0.0 {
        return Err(GeneratorError::InvalidDimensions);
    }
    if !spec.quiet_zone_modules.is_finite() || spec.quiet_zone_modules < 10.0 {
        return Err(GeneratorError::InvalidQuietZone);
    }
    if spec.data.is_empty() || spec.data.len() > 16_384 {
        return Err(GeneratorError::InvalidPayload);
    }
    match spec.symbology {
        BarcodeSymbology::Ean13 | BarcodeSymbology::UpcA => {
            let digits = validated_gtin(
                &spec.data,
                if spec.symbology == BarcodeSymbology::UpcA {
                    11
                } else {
                    12
                },
            )?;
            let data = if spec.symbology == BarcodeSymbology::UpcA {
                format!("0{digits}")
            } else {
                digits
            };
            EAN13::new(data)
                .map(|code| code.encode())
                .map_err(|_| GeneratorError::InvalidCharacter)
        }
        BarcodeSymbology::Code128 => {
            // A/B cover ASCII; deterministic C runs compact consecutive digit pairs.
            let data = code128_input(&spec.data)?;
            Code128::new(data)
                .map(|code| code.encode())
                .map_err(|_| GeneratorError::InvalidCharacter)
        }
        BarcodeSymbology::Code39 => Err(GeneratorError::UnsupportedSymbology),
    }
}

fn validated_gtin(data: &str, payload_length: usize) -> Result<String, GeneratorError> {
    if !data.bytes().all(|value| value.is_ascii_digit())
        || !(data.len() == payload_length || data.len() == payload_length + 1)
    {
        return Err(GeneratorError::InvalidCharacter);
    }
    let payload = &data[..payload_length];
    let sum: u32 = payload
        .bytes()
        .rev()
        .enumerate()
        .map(|(index, digit)| u32::from(digit - b'0') * if index % 2 == 0 { 3 } else { 1 })
        .sum();
    let check = ((10 - sum % 10) % 10) as u8 + b'0';
    if data.len() > payload_length && data.as_bytes()[payload_length] != check {
        return Err(GeneratorError::InvalidChecksum);
    }
    Ok(payload.to_owned())
}

fn code128_input(data: &str) -> Result<String, GeneratorError> {
    if !data.is_ascii() || data.bytes().any(|byte| byte == 127) {
        return Err(GeneratorError::InvalidCharacter);
    }
    // barcoders' explicit set markers are Unicode, so cannot collide with ASCII.
    let bytes = data.as_bytes();
    let mut output = String::new();
    let mut active = ' ';
    let mut index = 0;
    while index < bytes.len() {
        let digits = bytes[index..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        let next = if digits >= 4 {
            'C'
        } else if bytes[index] < 32 {
            'A'
        } else {
            'B'
        };
        if active != next {
            output.push(match next {
                'A' => 'À',
                'B' => 'Ɓ',
                _ => 'Ć',
            });
            active = next;
        }
        if next == 'C' {
            let count = digits - digits % 2;
            output.push_str(&data[index..index + count]);
            index += count;
        } else {
            output.push(char::from(bytes[index]));
            index += 1;
        }
    }
    Ok(output)
}

fn coalesce_modules(modules: &[bool], width: usize, height: usize, margin: f64) -> VectorPath {
    let mut path = VectorPath::new();
    let mut active: BTreeMap<(usize, usize), (usize, usize)> = BTreeMap::new();
    for row in 0..=height {
        let mut next = BTreeMap::new();
        let mut column = 0;
        while row < height && column < width {
            if !modules[row * width + column] {
                column += 1;
                continue;
            }
            let start = column;
            while column < width && modules[row * width + column] {
                column += 1;
            }
            let span = (start, column);
            next.insert(
                span,
                active
                    .remove(&span)
                    .map_or((row, 1), |(top, rows)| (top, rows + 1)),
            );
        }
        for ((left, right), (top, rows)) in active {
            path.contours.extend(
                VectorPath::rect(
                    margin + left as f64,
                    margin + top as f64,
                    (right - left) as f64,
                    rows as f64,
                )
                .contours,
            );
        }
        active = next;
    }
    path
}

/// Derived IDs are stable across reevaluation; authorial expansion must allocate fresh IDs.
pub(crate) fn stabilize_path(path: &mut VectorPath, seed: &[u8]) {
    let mut hash = blake3::Hasher::new();
    hash.update(b"petunia-generated-v1");
    hash.update(seed);
    for (contour_index, contour) in path.contours.iter_mut().enumerate() {
        let mut contour_hash = hash.clone();
        contour_hash.update(&(contour_index as u64).to_le_bytes());
        let uuid = |hash: blake3::Hash| {
            let mut bytes = [0; 16];
            bytes.copy_from_slice(&hash.as_bytes()[..16]);
            uuid::Uuid::from_bytes(bytes)
        };
        contour.id = ContourId::from_uuid(uuid(contour_hash.clone().finalize()));
        for (index, node) in contour.nodes.iter_mut().enumerate() {
            let mut node_hash = contour_hash.clone();
            node_hash.update(&(index as u64).to_le_bytes());
            node.id = NodeId::from_uuid(uuid(node_hash.finalize()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn qr(text: &str) -> QrCodeSpec {
        QrCodeSpec::new(
            QrPayload::text(text).unwrap(),
            QrErrorCorrection::Medium,
            QrVersionPolicy::Auto,
            QrMaskPolicy::Auto,
            4,
        )
        .unwrap()
    }
    #[test]
    fn independent_decoder_reads_unicode_and_all_ecc_levels() {
        for ecc in [
            QrErrorCorrection::Low,
            QrErrorCorrection::Medium,
            QrErrorCorrection::Quartile,
            QrErrorCorrection::High,
        ] {
            let mut spec = qr("Petúnia design 日本語");
            spec.error_correction = ecc;
            let matrix = StandardQrEncoder.encode(&spec).unwrap();
            let size = (matrix.size + 8) * 6;
            let pixels: Vec<u8> = (0..size * size)
                .map(|index| {
                    let x = index % size / 6;
                    let y = index / size / 6;
                    if x >= 4
                        && y >= 4
                        && x < matrix.size + 4
                        && y < matrix.size + 4
                        && matrix.dark[(y - 4) * matrix.size + x - 4]
                    {
                        0
                    } else {
                        255
                    }
                })
                .collect();
            let mut decoder = quircs::Quirc::default();
            let decoded: Vec<_> = decoder
                .identify(size, size, &pixels)
                .map(|code| code.unwrap().decode().unwrap())
                .collect();
            assert_eq!(decoded.len(), 1);
            assert_eq!(decoded[0].payload, "Petúnia design 日本語".as_bytes());
        }
    }
    #[test]
    fn fixed_version_never_expands_and_matrix_is_deterministic() {
        let mut spec = qr(&"X".repeat(300));
        spec.version = QrVersionPolicy::Fixed(1);
        assert_eq!(
            StandardQrEncoder.encode(&spec),
            Err(GeneratorError::PayloadTooLarge)
        );
        spec = qr("HELLO WORLD");
        spec.version = QrVersionPolicy::Fixed(1);
        spec.mask = QrMaskPolicy::Fixed(3);
        assert_eq!(StandardQrEncoder.encode(&spec).unwrap().size, 21);
        let input = GeneratorSpec::QrCode(spec);
        assert_eq!(
            evaluate_generator(&input).unwrap(),
            evaluate_generator(&input).unwrap()
        );
    }
    #[test]
    fn module_coalescing_preserves_coverage_and_merges_rows() {
        let path = coalesce_modules(&[true; 8], 4, 2, 4.0);
        assert_eq!(path.contours.len(), 1);
        assert_eq!(
            path.contours[0].nodes[2].point,
            petunia_core::Point::new(8.0, 6.0)
        );
    }
    #[test]
    fn gtin_known_checksums_and_guards() {
        let spec = |kind, data| BarcodeSpec::new(kind, data, 10.0, 50.0).unwrap();
        let ean = barcode_modules(&spec(BarcodeSymbology::Ean13, "4006381333931")).unwrap();
        assert_eq!(ean.len(), 95);
        assert_eq!(&ean[..3], &[1, 0, 1]);
        assert_eq!(
            ean,
            barcode_modules(&spec(BarcodeSymbology::Ean13, "400638133393")).unwrap()
        );
        assert_eq!(
            barcode_modules(&spec(BarcodeSymbology::Ean13, "4006381333932")),
            Err(GeneratorError::InvalidChecksum)
        );
        assert_eq!(
            barcode_modules(&spec(BarcodeSymbology::UpcA, "036000291452"))
                .unwrap()
                .len(),
            95
        );
        assert!(barcode_modules(&spec(BarcodeSymbology::Code128, "AB123456cd\n")).is_ok());
        assert_eq!(
            barcode_modules(&spec(BarcodeSymbology::Code39, "ABC")),
            Err(GeneratorError::UnsupportedSymbology)
        );
    }
}
