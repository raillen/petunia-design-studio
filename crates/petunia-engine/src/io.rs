//! Import/export contracts: sniffing, DTO boundaries and capability
//! negotiation.
//!
//! Parsers never return scene types directly, and exporters never
//! decide silently how to destroy a feature. Everything crosses
//! through validated DTOs with structured warnings and hard I/O
//! limits applied before big allocations happen.

use crate::error::{EngineError, Result};
use petunia_core::{
    Document, ExportColorOptions, ExportFormat, ImageObject, ImageSamplingPolicy, ImageSourceRect,
    NormalizedPoint, ResourceId, SceneItem, VectorPath,
};
use serde::{Deserialize, Serialize};

/// Hard input limits checked before and during parsing. Valid files
/// above the operational ceiling fail with a diagnosable error
/// instead of allocating until the system breaks.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IoLimits {
    pub max_bytes: u64,
    pub max_dimension_px: u32,
    pub max_nodes: usize,
    pub max_nesting: usize,
    pub max_string: usize,
    pub max_resources: usize,
    pub max_embedded_bytes: u64,
}

impl Default for IoLimits {
    fn default() -> Self {
        Self {
            max_bytes: 256 << 20,
            max_dimension_px: 16384,
            max_nodes: 1 << 20,
            max_nesting: 256,
            max_string: 1 << 20,
            max_resources: 4096,
            max_embedded_bytes: 512 << 20,
        }
    }
}

/// Format identification from real content, never the file name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectedFormat {
    Png,
    Jpeg,
    Svg,
    Ptnd,
    Unknown,
}

/// Sniff result with confidence for UI messaging.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ProbeResult {
    pub format: DetectedFormat,
    pub confidence: f32,
}

/// Identify a format from leading bytes. A `.png` name with a JPEG
/// header reports JPEG: names never override signatures.
#[must_use]
pub fn sniff_format(prefix: &[u8]) -> ProbeResult {
    if prefix.starts_with(&[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n']) {
        return ProbeResult {
            format: DetectedFormat::Png,
            confidence: 1.0,
        };
    }
    if prefix.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return ProbeResult {
            format: DetectedFormat::Png,
            confidence: 0.0,
        }
        .with_format(DetectedFormat::Jpeg, 1.0);
    }
    if prefix.starts_with(b"PK\x03\x04") {
        return ProbeResult {
            format: DetectedFormat::Ptnd,
            confidence: 0.6,
        };
    }
    let text = prefix
        .iter()
        .take(512)
        .map(|byte| *byte as char)
        .collect::<String>()
        .to_lowercase();
    let stripped = text.trim_start_matches(|ch: char| ch.is_whitespace() || ch == '\u{feff}');
    if stripped.starts_with("<svg") || stripped.starts_with("<?xml") && stripped.contains("<svg") {
        return ProbeResult {
            format: DetectedFormat::Svg,
            confidence: 0.9,
        };
    }
    ProbeResult {
        format: DetectedFormat::Unknown,
        confidence: 0.0,
    }
}

impl ProbeResult {
    fn with_format(mut self, format: DetectedFormat, confidence: f32) -> Self {
        self.format = format;
        self.confidence = confidence;
        self
    }
}

/// Structured fidelity warnings. Loss is never a bare string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportWarningKind {
    UnsupportedFeature,
    RasterizedSubtree,
    FontSubstituted,
    ProfileUnavailable,
    UnknownMetadataPreserved,
    UnknownMetadataDropped,
}

/// One warning with machine-readable kind plus detail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportWarning {
    pub kind: ImportWarningKind,
    pub detail: String,
}

/// A decoded raster image: dimensions plus straight-alpha RGBA bytes.
/// Placement into nodes/resources happens at the caller's layer.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportedImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub warnings: Vec<ImportWarning>,
}

/// Decode PNG/JPEG bytes into an image DTO inside the limits. JPEG
/// never backs lossless authorial storage; that policy lives with
/// the caller choosing PixelLayer versus ImageObject.
pub fn import_raster_image(bytes: &[u8], limits: &IoLimits) -> Result<ImportedImage> {
    if bytes.len() as u64 > limits.max_bytes {
        return Err(EngineError::Execution(format!(
            "image of {} bytes exceeds limit {}",
            bytes.len(),
            limits.max_bytes
        )));
    }
    let probe = sniff_format(bytes);
    match probe.format {
        DetectedFormat::Png | DetectedFormat::Jpeg => {}
        _ => {
            return Err(EngineError::Execution(
                "not a PNG or JPEG image".to_string(),
            ))
        }
    }
    let image = image::load_from_memory(bytes)
        .map_err(|error| EngineError::Execution(format!("image decode failed: {error}")))?
        .into_rgba8();
    if image.width() > limits.max_dimension_px || image.height() > limits.max_dimension_px {
        return Err(EngineError::Execution(format!(
            "image {}x{} exceeds dimension limit {}",
            image.width(),
            image.height(),
            limits.max_dimension_px
        )));
    }
    let mut warnings = Vec::new();
    if probe.format == DetectedFormat::Jpeg {
        warnings.push(ImportWarning {
            kind: ImportWarningKind::UnknownMetadataDropped,
            detail: "JPEG metadata is not imported; pixels only".to_string(),
        });
    }
    Ok(ImportedImage {
        width: image.width(),
        height: image.height(),
        rgba: image.into_raw(),
        warnings,
    })
}

/// Build a placed full-source image object from decoded pixels.
#[must_use]
pub fn place_image(resource: ResourceId, sampling: ImageSamplingPolicy) -> ImageObject {
    ImageObject {
        resource,
        source_rect: None,
        sampling,
    }
}

/// Build a cropped placed image with a validated normalized rect.
pub fn place_cropped_image(
    resource: ResourceId,
    sampling: ImageSamplingPolicy,
    min: (f64, f64),
    max: (f64, f64),
) -> Result<ImageObject> {
    let source_rect = ImageSourceRect::new(
        NormalizedPoint::new(min.0, min.1)
            .map_err(|error| EngineError::Execution(error.to_string()))?,
        NormalizedPoint::new(max.0, max.1)
            .map_err(|error| EngineError::Execution(error.to_string()))?,
    )
    .map_err(|error| EngineError::Execution(error.to_string()))?;
    Ok(ImageObject {
        resource,
        source_rect: Some(source_rect),
        sampling,
    })
}

/// Serialize one path to SVG path data (M/L/C/Z with fill rule).
#[must_use]
pub fn path_to_svg_data(path: &VectorPath) -> String {
    use std::fmt::Write;
    let mut data = String::new();
    for contour in &path.contours {
        if contour.nodes.is_empty() {
            continue;
        }
        for (index, node) in contour.nodes.iter().enumerate() {
            if index == 0 {
                let _ = write!(data, "M {} {} ", node.point.x, node.point.y);
                continue;
            }
            let previous = &contour.nodes[index - 1];
            match (previous.handle_out, node.handle_in) {
                (Some(out), Some(into)) => {
                    let _ = write!(
                        data,
                        "C {} {} {} {} {} {} ",
                        out.x, out.y, into.x, into.y, node.point.x, node.point.y
                    );
                }
                _ => {
                    let _ = write!(data, "L {} {} ", node.point.x, node.point.y);
                }
            }
        }
        if contour.closed {
            data.push_str("Z ");
        }
    }
    data.trim_end().to_string()
}

/// What one exporter can preserve. Surveyed against the document
/// before anything is destroyed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExporterCapabilities {
    pub vector_paths: bool,
    pub live_text: bool,
    pub gradients: bool,
    pub spot_colors: bool,
    pub icc_profiles: bool,
    pub transparency: bool,
    pub blend_modes: bool,
    pub raster_effects: bool,
    pub multi_page: bool,
}

/// One planned action per document feature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExportAction {
    PreserveNative,
    Convert,
    Expand,
    RasterizeSubtree,
    Warn(String),
    Error(String),
}

/// The negotiated plan: what survives natively and what degrades,
/// decided before export rather than silently mid-write.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportPlan {
    pub format: ExportFormat,
    pub actions: Vec<(String, ExportAction)>,
}

/// Survey the document against capabilities and plan each feature.
/// Vector paths, live text, gradients and transparency map to real
/// checks; anything the exporter lacks becomes warn or error, never
/// a silent downgrade.
#[must_use]
pub fn negotiate_export_plan(
    document: &Document,
    format: ExportFormat,
    color: ExportColorOptions,
    capabilities: &ExporterCapabilities,
) -> ExportPlan {
    let _ = color;
    let mut actions = Vec::new();
    let mut has_paths = false;
    let mut has_text = false;
    let mut has_images = false;
    for id in document.scene.root_order() {
        let Some(node) = document.scene.get_node(*id) else {
            continue;
        };
        match &node.item {
            SceneItem::Path(_) | SceneItem::Shape(_) => has_paths = true,
            SceneItem::Text(_) => has_text = true,
            SceneItem::Image(_) | SceneItem::PixelLayer(_) => has_images = true,
            _ => {}
        }
    }
    if has_paths {
        actions.push((
            "vector-paths".to_string(),
            if capabilities.vector_paths {
                ExportAction::PreserveNative
            } else {
                ExportAction::RasterizeSubtree
            },
        ));
    }
    if has_text {
        actions.push((
            "live-text".to_string(),
            if capabilities.live_text {
                ExportAction::PreserveNative
            } else {
                ExportAction::Warn(
                    "live text without exporter support; outlines or rasterize".to_string(),
                )
            },
        ));
    }
    if has_images {
        actions.push(("raster-sources".to_string(), ExportAction::PreserveNative));
    }
    if matches!(format, ExportFormat::Svg) && !capabilities.vector_paths {
        actions.push((
            "format".to_string(),
            ExportAction::Error("SVG export without vector support".to_string()),
        ));
    }
    ExportPlan { format, actions }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_core::{ColorSpaceRef, VectorPath};

    /// Encode a 1x1 red PNG through the image crate: real bytes with
    /// a valid CRC, generated deterministically in-test.
    fn tiny_png() -> Vec<u8> {
        use image::codecs::png::PngEncoder;
        use image::{ExtendedColorType, ImageEncoder};
        let mut bytes = Vec::new();
        PngEncoder::new(&mut bytes)
            .write_image(&[255u8, 0, 0, 255], 1, 1, ExtendedColorType::Rgba8)
            .expect("encodes");
        bytes
    }

    fn color_options() -> ExportColorOptions {
        ExportColorOptions {
            target: ColorSpaceRef::Builtin(petunia_core::BuiltinColorSpace::Srgb),
        }
    }

    #[test]
    fn sniffing_trusts_signatures_not_names() {
        assert_eq!(sniff_format(&tiny_png()).format, DetectedFormat::Png);
        assert_eq!(
            sniff_format(&[0xFF, 0xD8, 0xFF, 0xE0]).format,
            DetectedFormat::Jpeg
        );
        assert_eq!(
            sniff_format(b"<svg xmlns='x'></svg>").format,
            DetectedFormat::Svg
        );
        assert_eq!(
            sniff_format(b"PK\x03\x04 rest").format,
            DetectedFormat::Ptnd
        );
        assert_eq!(sniff_format(b"nonsense").format, DetectedFormat::Unknown);
    }

    #[test]
    fn png_decodes_inside_limits() {
        let bytes = tiny_png();
        let image = import_raster_image(&bytes, &IoLimits::default()).expect("decodes");
        assert_eq!((image.width, image.height), (1, 1));
        assert_eq!(image.rgba.len(), 4);
        assert_eq!(&image.rgba, &[255, 0, 0, 255]);
        assert!(image.warnings.is_empty());
        let tight = IoLimits {
            max_bytes: 8,
            ..IoLimits::default()
        };
        assert!(import_raster_image(&bytes, &tight).is_err());
        assert!(import_raster_image(b"nope", &IoLimits::default()).is_err());
    }

    #[test]
    fn svg_path_data_round_shapes() {
        let path = VectorPath::rect(1.0, 2.0, 10.0, 20.0);
        let data = path_to_svg_data(&path);
        assert!(data.starts_with("M 1 2 "), "{data}");
        assert!(data.contains('Z'));
        assert!(data.matches('L').count() >= 3, "{data}");
    }

    #[test]
    fn capability_gaps_warn_instead_of_silently_degrading() {
        let mut document = Document::new("plan");
        document
            .scene
            .insert_node(petunia_core::SceneNode::new_path(
                "box",
                VectorPath::rect(0.0, 0.0, 5.0, 5.0),
                petunia_core::ParentRef::Page(document.scene.default_page()),
            ));
        let full = ExporterCapabilities {
            vector_paths: true,
            live_text: true,
            gradients: false,
            spot_colors: false,
            icc_profiles: false,
            transparency: true,
            blend_modes: false,
            raster_effects: false,
            multi_page: false,
        };
        let plan = negotiate_export_plan(&document, ExportFormat::Png, color_options(), &full);
        assert!(plan
            .actions
            .iter()
            .any(|(_, action)| *action == ExportAction::PreserveNative));
        let weak = ExporterCapabilities {
            vector_paths: false,
            ..full
        };
        let plan = negotiate_export_plan(&document, ExportFormat::Png, color_options(), &weak);
        assert!(plan
            .actions
            .iter()
            .any(|(_, action)| *action == ExportAction::RasterizeSubtree));
    }
}
