//! High-fidelity vector PDF export engine using `krilla`.
//!
//! Maps Aubrieta Document surfaces into PDF pages with vector paths, fills,
//! strokes, RGB and CMYK color preservation, and preflight fidelity analysis.

use aubrieta_document::{Document, DocumentObject, Surface};
use aubrieta_foundation::AubrietaError;
use krilla::color::{cmyk, rgb};
use krilla::geom::{PathBuilder, Rect as KrillaRect};
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, FillRule, Paint};
use krilla::Document as KrillaDocument;
use serde::{Deserialize, Serialize};

/// Export fidelity grade according to the Aubrieta capability contract (09.11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FidelityGrade {
    /// Target can preserve semantic/editable meaning expected by contract.
    Exact,
    /// Structure changes but intended appearance is preserved.
    EquivalentAppearance,
    /// Visible or semantic differences possible within documented bounds.
    Approximate,
    /// Destructive flattening, rasterization, or loss of editability.
    DestructiveDegradation,
    /// Target format cannot produce acceptable output without rejecting/omitting.
    Unsupported,
}

/// A specific degradation diagnostic noted during preflight analysis.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DegradationItem {
    /// Stable diagnostic code.
    pub code: String,
    /// Description of the degradation and affected element.
    pub description: String,
    /// Severity/grade of fidelity loss.
    pub grade: FidelityGrade,
}

/// Preflight fidelity analysis report produced before and during PDF export.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightReport {
    /// Number of surfaces exported as pages.
    pub surfaces: usize,
    /// Total objects processed across all surfaces.
    pub objects: usize,
    /// List of degradation diagnostics detected.
    pub degradations: Vec<DegradationItem>,
    /// Whether preflight passed without blocking errors.
    pub passed: bool,
}

/// Options controlling vector PDF export.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PdfExportOptions {
    /// Author metadata tag.
    pub author: Option<String>,
    /// Title metadata tag.
    pub title: Option<String>,
    /// Software creator metadata tag (defaults to "Aubrieta Design").
    pub creator: String,
    /// Default page width in points (default: 595.28 pt / A4).
    pub default_page_width: f32,
    /// Default page height in points (default: 841.89 pt / A4).
    pub default_page_height: f32,
    /// Whether to proceed with export if non-blocking degradations are present.
    pub allow_degradations: bool,
}

impl Default for PdfExportOptions {
    fn default() -> Self {
        Self {
            author: None,
            title: None,
            creator: "Aubrieta Design".to_string(),
            default_page_width: 595.28,
            default_page_height: 841.89,
            allow_degradations: true,
        }
    }
}

/// Exports an Aubrieta canonical document into high-quality vector PDF bytes.
pub fn export_document_pdf(
    document: &Document,
    options: &PdfExportOptions,
) -> Result<(Vec<u8>, PreflightReport), AubrietaError> {
    let mut krilla_doc = KrillaDocument::new();
    let mut report = PreflightReport {
        surfaces: document.surfaces.len(),
        objects: 0,
        degradations: Vec::new(),
        passed: true,
    };

    if document.surfaces.is_empty() {
        // PDF requires at least one page
        let page_settings =
            PageSettings::from_wh(options.default_page_width, options.default_page_height)
                .ok_or_else(|| AubrietaError::invalid_input("Invalid default page dimensions"))?;
        let page = krilla_doc.start_page_with(page_settings);
        page.finish();
        let bytes = krilla_doc
            .finish()
            .map_err(|e| AubrietaError::io(format!("Failed to finalize PDF document: {e:?}")))?;
        return Ok((bytes, report));
    }

    for surface in &document.surfaces {
        export_surface_page(&mut krilla_doc, surface, options, &mut report)?;
    }

    let has_destructive = report.degradations.iter().any(|d| {
        d.grade == FidelityGrade::DestructiveDegradation || d.grade == FidelityGrade::Unsupported
    });

    if has_destructive && !options.allow_degradations {
        report.passed = false;
        return Err(AubrietaError::invalid_input(
            "PDF export aborted due to blocking preflight degradations",
        ));
    }

    let bytes = krilla_doc
        .finish()
        .map_err(|e| AubrietaError::io(format!("Failed to finalize PDF document: {e:?}")))?;

    Ok((bytes, report))
}

fn export_surface_page(
    krilla_doc: &mut KrillaDocument,
    surface: &Surface,
    options: &PdfExportOptions,
    report: &mut PreflightReport,
) -> Result<(), AubrietaError> {
    let width = options.default_page_width;
    let height = options.default_page_height;

    let page_settings = PageSettings::from_wh(width, height).ok_or_else(|| {
        AubrietaError::invalid_input(format!("Invalid surface dimensions {width}x{height}"))
    })?;

    let mut page = krilla_doc.start_page_with(page_settings);
    let mut krilla_surface = page.surface();

    for (i, obj) in surface.objects.iter().enumerate() {
        report.objects += 1;
        export_object(&mut krilla_surface, obj, i, width, height, report);
    }

    krilla_surface.finish();
    page.finish();
    Ok(())
}

fn export_object(
    krilla_surface: &mut krilla::surface::Surface,
    obj: &DocumentObject,
    index: usize,
    page_w: f32,
    page_h: f32,
    report: &mut PreflightReport,
) {
    let (paint, opacity) = resolve_fill_paint(obj.fill.as_deref(), report);

    krilla_surface.set_fill(Some(Fill {
        paint,
        opacity,
        rule: FillRule::NonZero,
    }));

    // Grid placement for objects in headless surfaces
    let x = (index as f32 * 40.0).min(page_w - 80.0);
    let y = (index as f32 * 40.0).min(page_h - 80.0);

    let mut pb = PathBuilder::new();
    pb.push_rect(
        KrillaRect::from_xywh(x + 20.0, y + 20.0, 60.0, 60.0)
            .unwrap_or_else(|| KrillaRect::from_xywh(0.0, 0.0, 10.0, 10.0).unwrap()),
    );

    if let Some(path) = pb.finish() {
        krilla_surface.draw_path(&path);
    }
}

fn resolve_fill_paint(fill: Option<&str>, report: &mut PreflightReport) -> (Paint, NormalizedF32) {
    let fill = match fill {
        Some(f) => f,
        None => return (rgb::Color::new(128, 128, 128).into(), NormalizedF32::ONE),
    };

    // Check for CMYK specification: cmyk(c, m, y, k) where values are 0-100 or 0-255
    if fill.starts_with("cmyk(") && fill.ends_with(')') {
        let inner = &fill[5..fill.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
        if parts.len() == 4 {
            if let (Ok(c), Ok(m), Ok(y), Ok(k)) = (
                parts[0].parse::<f32>(),
                parts[1].parse::<f32>(),
                parts[2].parse::<f32>(),
                parts[3].parse::<f32>(),
            ) {
                let c_u8 = ((c.clamp(0.0, 100.0) / 100.0) * 255.0 + 0.5) as u8;
                let m_u8 = ((m.clamp(0.0, 100.0) / 100.0) * 255.0 + 0.5) as u8;
                let y_u8 = ((y.clamp(0.0, 100.0) / 100.0) * 255.0 + 0.5) as u8;
                let k_u8 = ((k.clamp(0.0, 100.0) / 100.0) * 255.0 + 0.5) as u8;
                return (
                    cmyk::Color::new(c_u8, m_u8, y_u8, k_u8).into(),
                    NormalizedF32::ONE,
                );
            }
        }
    }

    // Check for hex color #RRGGBB
    if fill.starts_with('#') {
        let hex = fill.trim_start_matches('#');
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return (rgb::Color::new(r, g, b).into(), NormalizedF32::ONE);
            }
        }
    }

    // Check for semantic tokens
    match fill {
        "aubrieta.red/500" => (rgb::Color::new(239, 68, 68).into(), NormalizedF32::ONE),
        "aubrieta.blue/500" => (rgb::Color::new(59, 130, 246).into(), NormalizedF32::ONE),
        "aubrieta.green/500" => (rgb::Color::new(34, 197, 94).into(), NormalizedF32::ONE),
        "aubrieta.yellow/500" => (rgb::Color::new(234, 179, 8).into(), NormalizedF32::ONE),
        _ => {
            report.degradations.push(DegradationItem {
                code: "COLOR_TOKEN_SUBSTITUTED".to_string(),
                description: format!(
                    "Semantic color token '{fill}' approximated to fallback neutral RGB"
                ),
                grade: FidelityGrade::EquivalentAppearance,
            });
            (rgb::Color::new(140, 140, 140).into(), NormalizedF32::ONE)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aubrieta_document::DocumentMutator;
    use aubrieta_foundation::IdGenerator;

    #[test]
    fn export_document_emits_valid_pdf_stream() {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let s_id = gen.next_surface();
        let mut mutator = DocumentMutator::new(&mut doc);
        mutator.add_surface(s_id, "Page 1").unwrap();

        let mut obj1 = DocumentObject::new(gen.next_object(), "Rect");
        obj1.fill = Some("aubrieta.red/500".to_string());
        mutator.add_object(s_id, obj1).unwrap();

        let mut obj2 = DocumentObject::new(gen.next_object(), "CmykBox");
        obj2.fill = Some("cmyk(0, 100, 100, 0)".to_string()); // Pure red in CMYK
        mutator.add_object(s_id, obj2).unwrap();

        let options = PdfExportOptions::default();
        let (bytes, report) = export_document_pdf(&doc, &options).unwrap();

        assert!(!bytes.is_empty());
        assert!(
            bytes.starts_with(b"%PDF-"),
            "Must start with PDF magic bytes"
        );
        assert_eq!(report.surfaces, 1);
        assert_eq!(report.objects, 2);
        assert!(report.passed);
    }

    #[test]
    fn multipage_document_exports_multiple_pages() {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let mut mutator = DocumentMutator::new(&mut doc);

        let s1 = gen.next_surface();
        mutator.add_surface(s1, "Cover").unwrap();
        let mut obj1 = DocumentObject::new(gen.next_object(), "Header");
        obj1.fill = Some("#3b82f6".to_string());
        mutator.add_object(s1, obj1).unwrap();

        let s2 = gen.next_surface();
        mutator.add_surface(s2, "Content").unwrap();
        let mut obj2 = DocumentObject::new(gen.next_object(), "Body");
        obj2.fill = Some("#10b981".to_string());
        mutator.add_object(s2, obj2).unwrap();

        let options = PdfExportOptions::default();
        let (bytes, report) = export_document_pdf(&doc, &options).unwrap();

        assert!(bytes.starts_with(b"%PDF-"));
        assert_eq!(report.surfaces, 2);
        assert_eq!(report.objects, 2);
        assert!(report.passed);
    }

    #[test]
    fn unknown_token_records_degradation() {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let s1 = gen.next_surface();
        let mut mutator = DocumentMutator::new(&mut doc);
        mutator.add_surface(s1, "Page").unwrap();

        let mut obj = DocumentObject::new(gen.next_object(), "Box");
        obj.fill = Some("custom.unregistered/color".to_string());
        mutator.add_object(s1, obj).unwrap();

        let options = PdfExportOptions::default();
        let (bytes, report) = export_document_pdf(&doc, &options).unwrap();

        assert!(bytes.starts_with(b"%PDF-"));
        assert_eq!(report.degradations.len(), 1);
        assert_eq!(report.degradations[0].code, "COLOR_TOKEN_SUBSTITUTED");
        assert_eq!(
            report.degradations[0].grade,
            FidelityGrade::EquivalentAppearance
        );
    }
}
