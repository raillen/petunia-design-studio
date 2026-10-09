//! Generated vector content: traces and standard generators.
//!
//! Both families persist intent plus parameters; evaluated geometry
//! stays derived. Encoding algorithms (Reed-Solomon, mask scoring,
//! trace fitting) belong to the Engine or a specialized backend.

use crate::appearance::Appearance;
use crate::color::ColorValue;
use crate::error::{CoreError, Result};
use crate::id::ResourceId;
use serde::{Deserialize, Serialize};

/// Vector derived from a raster source. Editing parameters keeps the
/// scene identity; `Expand Trace` materializes new paths. An
/// unresolved source keeps the object valid with unavailable
/// evaluation, never an emptied result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceObject {
    pub source: ResourceId,
    pub spec: ImageTraceSpec,
}

/// Trace intent: mode, sensitivity and quality knobs. Threshold
/// conversion math lives in the Engine and stays versioned there.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageTraceSpec {
    pub mode: TraceMode,
    pub alpha_policy: AnalysisAlphaPolicy,
    pub threshold: f32,
    pub min_region_area_px: f64,
    pub corner_sensitivity: f32,
    pub smoothing: f32,
    pub curve_tolerance: f64,
    pub ignore_background: Option<ColorValue>,
}

impl ImageTraceSpec {
    /// Threshold sits in `0..1`; areas and smoothing stay finite and
    /// non-negative; grayscale/color levels must select at least one.
    pub fn validate(&self) -> Result<()> {
        if !self.threshold.is_finite() || !(0.0..=1.0).contains(&self.threshold) {
            return Err(CoreError::InvariantViolation(format!(
                "trace threshold out of 0..=1: {}",
                self.threshold
            )));
        }
        for (label, value) in [
            ("min region area", self.min_region_area_px),
            ("smoothing", f64::from(self.smoothing)),
            ("curve tolerance", self.curve_tolerance),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(CoreError::InvariantViolation(format!(
                    "invalid trace {label} rejected: {value}"
                )));
            }
        }
        if !self.corner_sensitivity.is_finite() {
            return Err(CoreError::InvariantViolation(format!(
                "non-finite corner sensitivity rejected: {}",
                self.corner_sensitivity
            )));
        }
        match self.mode {
            TraceMode::Grayscale { levels } | TraceMode::Color { colors: levels } => {
                if levels == 0 {
                    return Err(CoreError::InvariantViolation(
                        "trace levels must select at least one".to_string(),
                    ));
                }
            }
            TraceMode::Monochrome => {}
        }
        Ok(())
    }
}

impl Default for ImageTraceSpec {
    /// Conservative starting point: monochrome at mid threshold.
    fn default() -> Self {
        Self {
            mode: TraceMode::Monochrome,
            alpha_policy: AnalysisAlphaPolicy::Ignore,
            threshold: 0.5,
            min_region_area_px: 4.0,
            corner_sensitivity: 0.7,
            smoothing: 1.0,
            curve_tolerance: 0.5,
            ignore_background: None,
        }
    }
}

/// Trace color depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceMode {
    Monochrome,
    Grayscale { levels: u16 },
    Color { colors: u16 },
}

/// How source alpha feeds region analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AnalysisAlphaPolicy {
    #[default]
    Ignore,
    UseAsMask,
}

/// Vector from a structured standard (QR, barcode): data plus rules,
/// never a geometric primitive. The matrix and module pattern stay
/// derived; the object reuses the normal `Appearance`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneratedVectorObject {
    pub generator: GeneratorSpec,
    pub appearance: Appearance,
}

/// Supported generator standards. New built-ins need stable
/// parameters, deterministic evaluation and real value; the enum is
/// not an open catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GeneratorSpec {
    QrCode(QrCodeSpec),
    Barcode(BarcodeSpec),
}

/// QR payload plus encoding policies. Reed-Solomon, mask scoring and
/// placement belong to the Engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QrCodeSpec {
    pub payload: QrPayload,
    pub error_correction: QrErrorCorrection,
    pub version: QrVersionPolicy,
    pub mask: QrMaskPolicy,
    pub quiet_zone_modules: u8,
}

impl QrCodeSpec {
    /// Fixed versions cover QR 1–40; fixed masks cover 0–7.
    pub fn new(
        payload: QrPayload,
        error_correction: QrErrorCorrection,
        version: QrVersionPolicy,
        mask: QrMaskPolicy,
        quiet_zone_modules: u8,
    ) -> Result<Self> {
        if let QrVersionPolicy::Fixed(number) = version {
            if !(1..=40).contains(&number) {
                return Err(CoreError::InvariantViolation(format!(
                    "QR version out of 1..=40: {number}"
                )));
            }
        }
        if let QrMaskPolicy::Fixed(number) = mask {
            if number > 7 {
                return Err(CoreError::InvariantViolation(format!(
                    "QR mask out of 0..=7: {number}"
                )));
            }
        }
        Ok(Self {
            payload,
            error_correction,
            version,
            mask,
            quiet_zone_modules,
        })
    }
}

/// Encoded QR content. Binary payloads arrive as further variants;
/// text covers the v0.1 contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QrPayload {
    Text(String),
}

impl QrPayload {
    /// Payloads must not be empty.
    pub fn text(content: impl Into<String>) -> Result<Self> {
        let content = content.into();
        if content.is_empty() {
            return Err(CoreError::InvariantViolation(
                "QR payload must not be empty".to_string(),
            ));
        }
        Ok(Self::Text(content))
    }
}

/// QR error correction level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QrErrorCorrection {
    Low,
    #[default]
    Medium,
    Quartile,
    High,
}

/// QR version selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QrVersionPolicy {
    #[default]
    Auto,
    Fixed(u8),
}

/// QR mask selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QrMaskPolicy {
    #[default]
    Auto,
    Fixed(u8),
}

/// Barcode symbology plus data. Charsets, checksums and bar encoding
/// evaluate in the Engine per symbology; human-readable labels use
/// the normal text system instead of embedded outlines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BarcodeSpec {
    pub symbology: BarcodeSymbology,
    pub data: String,
    pub quiet_zone_modules: f64,
    pub bar_height_modules: f64,
}

impl BarcodeSpec {
    /// Data must be non-empty; quiet zone finite and non-negative;
    /// bar height finite and positive.
    pub fn new(
        symbology: BarcodeSymbology,
        data: impl Into<String>,
        quiet_zone_modules: f64,
        bar_height_modules: f64,
    ) -> Result<Self> {
        let data = data.into();
        if data.is_empty() {
            return Err(CoreError::InvariantViolation(
                "barcode data must not be empty".to_string(),
            ));
        }
        if !quiet_zone_modules.is_finite() || quiet_zone_modules < 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid barcode quiet zone rejected: {quiet_zone_modules}"
            )));
        }
        if !bar_height_modules.is_finite() || bar_height_modules <= 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "invalid barcode bar height rejected: {bar_height_modules}"
            )));
        }
        Ok(Self {
            symbology,
            data,
            quiet_zone_modules,
            bar_height_modules,
        })
    }
}

/// Symbologies covered by the initial contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BarcodeSymbology {
    Code128,
    Ean13,
    UpcA,
    Code39,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> ImageTraceSpec {
        let spec = ImageTraceSpec {
            mode: TraceMode::Color { colors: 8 },
            ..Default::default()
        };
        spec.validate().expect("valid test spec");
        spec
    }

    #[test]
    fn trace_spec_validates_ranges() {
        assert!(spec().threshold == 0.5);
        let zero_levels = ImageTraceSpec {
            mode: TraceMode::Grayscale { levels: 0 },
            ..Default::default()
        };
        assert!(zero_levels.validate().is_err());
        let bad_threshold = ImageTraceSpec {
            threshold: 1.5,
            ..Default::default()
        };
        assert!(bad_threshold.validate().is_err());
    }

    #[test]
    fn qr_spec_validates_version_mask_and_payload() {
        assert!(QrCodeSpec::new(
            QrPayload::text("https://petunia.design").expect("valid"),
            QrErrorCorrection::Medium,
            QrVersionPolicy::Auto,
            QrMaskPolicy::Auto,
            4,
        )
        .is_ok());
        assert!(QrCodeSpec::new(
            QrPayload::text("x").expect("valid"),
            QrErrorCorrection::Medium,
            QrVersionPolicy::Fixed(41),
            QrMaskPolicy::Auto,
            4,
        )
        .is_err());
        assert!(QrPayload::text("").is_err());
    }

    #[test]
    fn barcode_spec_requires_data_and_geometry() {
        assert!(BarcodeSpec::new(BarcodeSymbology::Code128, "1234", 10.0, 50.0).is_ok());
        assert!(BarcodeSpec::new(BarcodeSymbology::Ean13, "", 10.0, 50.0).is_err());
        assert!(BarcodeSpec::new(BarcodeSymbology::Ean13, "123", 10.0, 0.0).is_err());
    }

    #[test]
    fn generated_object_serialization_round_trip() {
        let object = GeneratedVectorObject {
            generator: GeneratorSpec::QrCode(
                QrCodeSpec::new(
                    QrPayload::text("hi").expect("valid"),
                    QrErrorCorrection::Medium,
                    QrVersionPolicy::Auto,
                    QrMaskPolicy::Auto,
                    4,
                )
                .expect("valid"),
            ),
            appearance: Appearance { items: Vec::new() },
        };
        let json = serde_json::to_string(&object).expect("serializable");
        let back: GeneratedVectorObject = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(back, object);
    }
}
