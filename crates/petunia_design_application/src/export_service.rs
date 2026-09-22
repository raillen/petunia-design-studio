//! Export pipeline behind `ptnd.action.file.export` (08.10, 09.11, 15.G).
//!
//! Export is a *typed request* resolved from an action payload, not a
//! toolkit-held file dialog. That keeps the same operation reachable from the
//! File menu, a shortcut, the command palette, MCP or a plugin, and it keeps
//! the "where do I write" decision (the file dialog) in the UI adapter where
//! it belongs.
//!
//! The three V1 targets map to the engines that already own them:
//! - PDF → [`petunia_design_io::pdf`] (vector, preflight report included);
//! - SVG → [`petunia_design_io::svg`] (vector, W3C envelope);
//! - PNG → [`petunia_design_render::SoftwarePixelCompositor`] rasterized per
//!   surface, encoded by [`petunia_design_io::image_io`].
//!
//! Export never mutates the document and never enters history.

use std::path::{Path, PathBuf};

use petunia_design_document::{Document, Surface};
use petunia_design_foundation::{PetuniaError, SurfaceId};
use petunia_design_io::{
    export_document_pdf, export_document_svg, export_raster, PdfExportOptions, RasterExportOptions,
    RasterFormat, RawRasterImage,
};
use petunia_design_render::{PixelBufferRgba8, SoftwarePixelCompositor};
use serde::{Deserialize, Serialize};

/// Artifact formats the V1 export action can produce.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    /// Rasterised active surface (PNG-24 with alpha).
    Png,
    /// Whole document as W3C SVG.
    Svg,
    /// Whole document as vector PDF.
    Pdf,
}

impl ExportFormat {
    /// Parses the payload spelling of a format.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "png" => Some(Self::Png),
            "svg" => Some(Self::Svg),
            "pdf" => Some(Self::Pdf),
            _ => None,
        }
    }

    /// Canonical file suffix, without the dot.
    #[must_use]
    pub const fn suffix(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Svg => "svg",
            Self::Pdf => "pdf",
        }
    }

    /// Infers the format from a file suffix.
    #[must_use]
    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?;
        Self::parse(extension)
    }

    /// Whether the format rasterizes a single surface (instead of the document).
    #[must_use]
    pub const fn is_surface_scoped(self) -> bool {
        matches!(self, Self::Png)
    }
}

/// A fully resolved export: format, destination and raster scope.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExportRequest {
    /// Artifact format to produce.
    pub format: ExportFormat,
    /// Destination path. The suffix is normalized to the format.
    pub path: PathBuf,
    /// Surface to rasterize. `None` uses the active surface, then the first.
    pub surface: Option<SurfaceId>,
}

impl ExportRequest {
    /// Creates a request, normalizing the suffix to the chosen format.
    #[must_use]
    pub fn new(format: ExportFormat, path: impl Into<PathBuf>) -> Self {
        let path = with_format_suffix(path.into(), format);
        Self {
            format,
            path,
            surface: None,
        }
    }

    /// Restricts a raster export to an explicit surface.
    #[must_use]
    pub fn on_surface(mut self, surface: SurfaceId) -> Self {
        self.surface = Some(surface);
        self
    }

    /// Resolves a request from an action payload.
    ///
    /// Required: `path`. Optional: `format` (else inferred from `path`) and
    /// `surface` (else `active_surface`). Guessing a format from nothing is
    /// refused rather than defaulted, so a caller can never export a format it
    /// did not ask for.
    pub fn from_payload(
        payload: &serde_json::Value,
        active_surface: Option<SurfaceId>,
    ) -> Result<Self, PetuniaError> {
        let path = payload
            .get("path")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                PetuniaError::invalid_input("file.export requires a non-empty `path` payload field")
            })?;
        let path = PathBuf::from(path);
        let format = match payload.get("format").and_then(serde_json::Value::as_str) {
            Some(spelling) => ExportFormat::parse(spelling).ok_or_else(|| {
                PetuniaError::invalid_input(format!(
                    "file.export format must be png|svg|pdf, got `{spelling}`"
                ))
            })?,
            None => ExportFormat::from_path(&path).ok_or_else(|| {
                PetuniaError::invalid_input(
                    "file.export needs a `format` or a path with a png|svg|pdf suffix",
                )
            })?,
        };
        let surface = payload
            .get("surface")
            .and_then(serde_json::Value::as_u64)
            .map(SurfaceId::new)
            .or(active_surface);
        Ok(Self {
            format,
            path: with_format_suffix(path, format),
            surface: if format.is_surface_scoped() {
                surface
            } else {
                None
            },
        })
    }
}

/// What an export produced, for status reporting.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExportOutcome {
    /// Path actually written (suffix normalized).
    pub path: PathBuf,
    /// Bytes written.
    pub bytes: usize,
    /// Format produced.
    pub format: ExportFormat,
    /// Surfaces exported (one for PNG, all export-enabled ones otherwise).
    pub surfaces: usize,
    /// Degradation codes the engine reported, if any.
    pub degradations: Vec<String>,
}

/// Normalizes a destination so it always carries the format's suffix.
#[must_use]
pub fn with_format_suffix(path: PathBuf, format: ExportFormat) -> PathBuf {
    match path.extension().and_then(|value| value.to_str()) {
        Some(existing) if existing.eq_ignore_ascii_case(format.suffix()) => path,
        _ => path.with_extension(format.suffix()),
    }
}

/// Runs an export request and writes the artifact to disk.
pub fn export_document(
    document: &Document,
    request: &ExportRequest,
) -> Result<ExportOutcome, PetuniaError> {
    if document.surfaces().is_empty() {
        return Err(PetuniaError::invalid_input(
            "nothing to export: the document has no surfaces",
        ));
    }
    let mut degradations: Vec<String> = Vec::new();
    let (bytes, surfaces) = match request.format {
        ExportFormat::Svg => (
            export_document_svg(document).into_bytes(),
            export_enabled(document),
        ),
        ExportFormat::Pdf => {
            let (bytes, report) = export_document_pdf(document, &PdfExportOptions::default())?;
            degradations.extend(report.degradations.iter().map(|item| item.code.clone()));
            (bytes, export_enabled(document))
        }
        ExportFormat::Png => {
            let surface = resolve_surface(document, request.surface)?;
            let (bytes, items) = render_surface_png(surface)?;
            degradations.extend(items.into_iter().map(|item| item.code));
            (bytes, 1)
        }
    };
    std::fs::write(&request.path, &bytes).map_err(|error| {
        PetuniaError::io(format!(
            "could not write export {}: {error}",
            request.path.display()
        ))
    })?;
    Ok(ExportOutcome {
        path: request.path.clone(),
        bytes: bytes.len(),
        format: request.format,
        surfaces,
        degradations,
    })
}

/// Counts surfaces included in batch export.
#[must_use]
pub fn export_enabled(document: &Document) -> usize {
    document
        .surfaces()
        .iter()
        .filter(|surface| surface.export_enabled)
        .count()
}

/// Resolves which surface a raster export targets.
fn resolve_surface(
    document: &Document,
    requested: Option<SurfaceId>,
) -> Result<&Surface, PetuniaError> {
    if let Some(id) = requested {
        return document.surface(id).map_err(|_| {
            PetuniaError::not_found(format!(
                "export target surface `{id}` is not in the document"
            ))
        });
    }
    document
        .surfaces()
        .first()
        .ok_or_else(|| PetuniaError::invalid_input("nothing to export: no surface to rasterize"))
}

/// Rasterizes one surface and encodes it as PNG.
///
/// Surface geometry is in pasteboard coordinates, while the CPU compositor
/// maps document coordinates straight into the buffer. A surface that does not
/// start at the origin would therefore render off-buffer, so the buffer is
/// sized to cover the surface *and* its offset, then cropped back to the
/// surface rectangle. No scaling: V1 exports at 1:1 pixels per point.
fn render_surface_png(
    surface: &Surface,
) -> Result<(Vec<u8>, Vec<petunia_design_io::DegradationItem>), PetuniaError> {
    let [origin_x, origin_y, width, height] = surface.bounds();
    if width <= 0.0 || height <= 0.0 {
        return Err(PetuniaError::invalid_input(format!(
            "surface `{}` has non-positive dimensions ({width}x{height})",
            surface.id
        )));
    }
    let full_width = (origin_x.max(0.0) + width).ceil().max(1.0) as u32;
    let full_height = (origin_y.max(0.0) + height).ceil().max(1.0) as u32;
    let full = SoftwarePixelCompositor::render_surface_rgba8(
        surface,
        full_width,
        full_height,
        [0, 0, 0, 0],
    );
    let cropped = crop_surface_rect(&full, [origin_x, origin_y, width, height]);
    let raw = RawRasterImage::from_rgba8(cropped.width, cropped.height, cropped.data)?;
    export_raster(
        &raw,
        &RasterExportOptions {
            format: RasterFormat::Png,
            jpeg_quality: 90,
            allow_degradations: true,
        },
    )
}

/// Copies the `[x, y, w, h]` rectangle out of a rasterized document buffer.
fn crop_surface_rect(buffer: &PixelBufferRgba8, rect: [f64; 4]) -> PixelBufferRgba8 {
    let [x, y, width, height] = rect;
    let left = x.floor().max(0.0) as u32;
    let top = y.floor().max(0.0) as u32;
    let crop_width = (width.round() as u32).max(1);
    let crop_height = (height.round() as u32).max(1);
    if left == 0 && top == 0 && crop_width == buffer.width && crop_height == buffer.height {
        return buffer.clone();
    }
    let mut data = vec![0u8; (crop_width as usize) * (crop_height as usize) * 4];
    for row in 0..crop_height {
        let source_y = top + row;
        if source_y >= buffer.height {
            break;
        }
        let source_start = ((source_y as usize) * (buffer.width as usize) + left as usize) * 4;
        let source_end = source_start + (crop_width as usize) * 4;
        if source_end > buffer.data.len() {
            continue;
        }
        let target_start = (row as usize) * (crop_width as usize) * 4;
        data[target_start..target_start + (crop_width as usize) * 4]
            .copy_from_slice(&buffer.data[source_start..source_end]);
    }
    PixelBufferRgba8 {
        width: crop_width,
        height: crop_height,
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creation::create_shape_commands;
    use crate::session::DocumentSession;
    use crate::{Command, CommandRequest};
    use petunia_design_document::ShapeKind;

    /// Builds a real document through the public command lane: one surface
    /// holding one blue square. No test-only constructor is needed because
    /// the export service only reads.
    ///
    /// A fresh session owns no surface (`DocumentSession::new`), so the
    /// surface is created explicitly — same as any real session does.
    fn document_with_square() -> Document {
        let mut session = DocumentSession::new("export");
        let surface = SurfaceId::new(1);
        session
            .execute_command(CommandRequest::new(Command::CreateSurface {
                id: surface,
                name: "Surface".to_string(),
            }))
            .expect("surface creates");
        session.set_active_surface(surface);
        let id = session.next_object_id();
        let commands = create_shape_commands(
            surface,
            id,
            "Square",
            ShapeKind::Rectangle {
                corner_radii: [0.0; 4],
            },
            Some([10.0, 10.0, 40.0, 40.0]),
            Some("ptnd.blue/500".to_string()),
            None,
        );
        session
            .transact("Square", commands)
            .expect("the square commits");
        session.document().clone()
    }

    #[test]
    fn format_parsing_covers_the_v1_targets_only() {
        assert_eq!(ExportFormat::parse("PNG"), Some(ExportFormat::Png));
        assert_eq!(ExportFormat::parse("svg"), Some(ExportFormat::Svg));
        assert_eq!(ExportFormat::parse("pdf"), Some(ExportFormat::Pdf));
        assert_eq!(ExportFormat::parse("tiff"), None);
        assert_eq!(
            ExportFormat::from_path(Path::new("/tmp/a.PDF")),
            Some(ExportFormat::Pdf)
        );
        assert_eq!(ExportFormat::from_path(Path::new("/tmp/a")), None);
    }

    #[test]
    fn payload_requires_a_destination_and_a_known_format() {
        let missing = ExportRequest::from_payload(&serde_json::json!({}), None);
        assert!(missing.is_err());

        let guessed = ExportRequest::from_payload(&serde_json::json!({"path": "/tmp/a"}), None);
        assert!(guessed.is_err(), "an unknown suffix must not be guessed at");

        let explicit = ExportRequest::from_payload(
            &serde_json::json!({"path": "/tmp/a", "format": "png", "surface": 7}),
            None,
        )
        .expect("a complete payload resolves");
        assert_eq!(explicit.format, ExportFormat::Png);
        assert_eq!(explicit.path, PathBuf::from("/tmp/a.png"));
        assert_eq!(explicit.surface, Some(SurfaceId::new(7)));

        let bad = ExportRequest::from_payload(
            &serde_json::json!({"path": "/tmp/a", "format": "gif"}),
            None,
        );
        assert!(bad.is_err());
    }

    #[test]
    fn document_scoped_formats_ignore_the_surface_field() {
        let request = ExportRequest::from_payload(
            &serde_json::json!({"path": "/tmp/a.pdf", "surface": 3}),
            None,
        )
        .expect("pdf resolves from the suffix");
        assert_eq!(request.format, ExportFormat::Pdf);
        assert_eq!(
            request.surface, None,
            "PDF exports the document, not a surface"
        );
    }

    #[test]
    fn png_export_renders_the_document_artwork() {
        // Proves the pipeline goes through the compositor rather than writing
        // an empty canvas: the square must leave non-transparent pixels.
        let document = document_with_square();
        let surface = &document.surfaces()[0];
        let [origin_x, origin_y, width, height] = surface.bounds();
        let full = SoftwarePixelCompositor::render_surface_rgba8(
            surface,
            (origin_x + width).ceil() as u32,
            (origin_y + height).ceil() as u32,
            [0, 0, 0, 0],
        );
        let rendered = crop_surface_rect(&full, surface.bounds());
        assert!(
            rendered
                .data
                .iter()
                .skip(3)
                .step_by(4)
                .any(|alpha| *alpha != 0),
            "the square did not paint anything"
        );
    }

    #[test]
    fn png_export_writes_a_real_png_for_the_active_surface() {
        let document = document_with_square();
        let dir = std::env::temp_dir().join(format!("ptnd-export-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let request = ExportRequest::new(ExportFormat::Png, dir.join("surface"));
        let outcome = export_document(&document, &request).expect("png export succeeds");
        let bytes = std::fs::read(&outcome.path).expect("artifact exists");
        assert!(bytes.starts_with(b"\x89PNG"), "not a PNG file");
        assert_eq!(outcome.surfaces, 1);
        assert_eq!(outcome.bytes, bytes.len());
        assert_eq!(
            outcome.path.extension().and_then(|e| e.to_str()),
            Some("png")
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn svg_and_pdf_exports_produce_their_envelopes() {
        let document = document_with_square();
        let dir = std::env::temp_dir().join(format!("ptnd-export-doc-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");

        let svg = export_document(
            &document,
            &ExportRequest::new(ExportFormat::Svg, dir.join("a")),
        )
        .expect("svg export succeeds");
        let svg_bytes = std::fs::read(&svg.path).expect("artifact exists");
        assert!(String::from_utf8_lossy(&svg_bytes).contains("<svg"));

        let pdf = export_document(
            &document,
            &ExportRequest::new(ExportFormat::Pdf, dir.join("a")),
        )
        .expect("pdf export succeeds");
        let pdf_bytes = std::fs::read(&pdf.path).expect("artifact exists");
        assert!(pdf_bytes.starts_with(b"%PDF"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn export_refuses_an_empty_document() {
        let document = Document::new();
        let request = ExportRequest::new(ExportFormat::Png, std::env::temp_dir().join("x.png"));
        assert!(export_document(&document, &request).is_err());
    }

    #[test]
    fn export_rejects_a_surface_that_is_not_in_the_document() {
        let document = document_with_square();
        let request = ExportRequest::new(ExportFormat::Png, std::env::temp_dir().join("y.png"))
            .on_surface(SurfaceId::new(9999));
        assert!(export_document(&document, &request).is_err());
    }

    #[test]
    fn crop_keeps_the_surface_rect_when_it_is_offset() {
        let buffer = PixelBufferRgba8::with_fill(20, 20, [9, 9, 9, 255]);
        let cropped = crop_surface_rect(&buffer, [4.0, 6.0, 8.0, 8.0]);
        assert_eq!(cropped.width, 8);
        assert_eq!(cropped.height, 8);
        assert_eq!(cropped.data.len(), 8 * 8 * 4);
        assert_eq!(cropped.get_pixel(0, 0), Some([9, 9, 9, 255]));
    }

    #[test]
    fn crop_is_identity_when_the_surface_fills_the_buffer() {
        let buffer = PixelBufferRgba8::with_fill(5, 5, [1, 2, 3, 4]);
        let cropped = crop_surface_rect(&buffer, [0.0, 0.0, 5.0, 5.0]);
        assert_eq!(cropped, buffer);
    }
}
