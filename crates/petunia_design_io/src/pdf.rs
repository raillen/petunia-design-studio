//! High-fidelity vector PDF export engine using `krilla`.
//!
//! Maps Petunia Document surfaces into PDF pages with vector paths, fills,
//! strokes, RGB and CMYK color preservation, and preflight fidelity analysis.

use petunia_design_document::{Document, DocumentObject, Surface};
use petunia_design_foundation::PetuniaError;
use krilla::color::{cmyk, rgb};
use krilla::geom::{PathBuilder, Rect as KrillaRect, Transform};
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, FillRule, LineCap, LineJoin, Paint, Stroke, StrokeDash};
use krilla::Document as KrillaDocument;
use serde::{Deserialize, Serialize};

/// Export fidelity grade according to the Petunia capability contract (09.11).
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
    /// Software creator metadata tag (defaults to "Petunia Design Studio").
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
            creator: "Petunia Design Studio".to_string(),
            default_page_width: 595.28,
            default_page_height: 841.89,
            allow_degradations: true,
        }
    }
}

/// Exports a Petunia canonical document into high-quality vector PDF bytes.
pub fn export_document_pdf(
    document: &Document,
    options: &PdfExportOptions,
) -> Result<(Vec<u8>, PreflightReport), PetuniaError> {
    let mut krilla_doc = KrillaDocument::new();
    let mut report = PreflightReport {
        surfaces: document.surfaces().len(),
        objects: 0,
        degradations: Vec::new(),
        passed: true,
    };

    if document.surfaces().is_empty() {
        // PDF requires at least one page
        let page_settings =
            PageSettings::from_wh(options.default_page_width, options.default_page_height)
                .ok_or_else(|| PetuniaError::invalid_input("Invalid default page dimensions"))?;
        let page = krilla_doc.start_page_with(page_settings);
        page.finish();
        let bytes = krilla_doc
            .finish()
            .map_err(|e| PetuniaError::io(format!("Failed to finalize PDF document: {e:?}")))?;
        return Ok((bytes, report));
    }

    for surface in document.surfaces() {
        export_surface_page(&mut krilla_doc, surface, options, &mut report)?;
    }

    let has_destructive = report.degradations.iter().any(|d| {
        d.grade == FidelityGrade::DestructiveDegradation || d.grade == FidelityGrade::Unsupported
    });

    if has_destructive && !options.allow_degradations {
        report.passed = false;
        return Err(PetuniaError::invalid_input(
            "PDF export aborted due to blocking preflight degradations",
        ));
    }

    let bytes = krilla_doc
        .finish()
        .map_err(|e| PetuniaError::io(format!("Failed to finalize PDF document: {e:?}")))?;

    Ok((bytes, report))
}

fn export_surface_page(
    krilla_doc: &mut KrillaDocument,
    surface: &Surface,
    options: &PdfExportOptions,
    report: &mut PreflightReport,
) -> Result<(), PetuniaError> {
    let width = options.default_page_width;
    let height = options.default_page_height;

    let page_settings = PageSettings::from_wh(width, height).ok_or_else(|| {
        PetuniaError::invalid_input(format!("Invalid surface dimensions {width}x{height}"))
    })?;

    let mut page = krilla_doc.start_page_with(page_settings);
    let mut krilla_surface = page.surface();

    for (i, obj) in surface.objects().iter().enumerate() {
        report.objects += 1;
        export_object(&mut krilla_surface, surface, obj, i, width, height, report);
    }

    krilla_surface.finish();
    page.finish();
    Ok(())
}

fn export_object(
    krilla_surface: &mut krilla::surface::Surface,
    surface: &Surface,
    obj: &DocumentObject,
    index: usize,
    page_w: f32,
    page_h: f32,
    report: &mut PreflightReport,
) {
    if !obj.visible {
        return;
    }
    // Mask boundaries are clip sources, never painted content (10.5).
    if obj.is_clip_mask {
        return;
    }
    let label = if obj.name.is_empty() {
        format!("object `{}`", obj.id)
    } else {
        format!("'{}'", obj.name)
    };

    let eff = obj.effective_appearance();
    let entry_opacity = eff
        .primary_fill()
        .map(|f| f.opacity)
        .or_else(|| eff.primary_stroke().map(|s| s.opacity))
        .unwrap_or(1.0);
    let total_opacity = (eff.opacity * entry_opacity).clamp(0.0, 1.0);
    if total_opacity <= 0.0 {
        return;
    }

    // Geometry: canonical outline; legacy grid fallback when unbounded so
    // old headless fixtures keep exporting (F-13).
    let outline = obj.evaluated_path();
    let mut pb = PathBuilder::new();
    let mut has_geometry = false;
    for verb in &outline.verbs {
        has_geometry = true;
        match verb {
            petunia_design_geometry::PathVerb::MoveTo(p) => pb.move_to(p.x as f32, p.y as f32),
            petunia_design_geometry::PathVerb::LineTo(p) => pb.line_to(p.x as f32, p.y as f32),
            petunia_design_geometry::PathVerb::QuadTo(c, p) => {
                pb.quad_to(c.x as f32, c.y as f32, p.x as f32, p.y as f32);
            }
            petunia_design_geometry::PathVerb::CubicTo(c1, c2, p) => pb.cubic_to(
                c1.x as f32,
                c1.y as f32,
                c2.x as f32,
                c2.y as f32,
                p.x as f32,
                p.y as f32,
            ),
            petunia_design_geometry::PathVerb::Close => pb.close(),
        }
    }
    if !has_geometry {
        report.degradations.push(DegradationItem {
            code: "GEOMETRY_FALLBACK_RECT".to_string(),
            description: format!(
                "{label} has no vector outline (e.g. un-outlined text): exported as bounds rect"
            ),
            grade: FidelityGrade::Approximate,
        });
        // Legacy index-grid fallback rect.
        let x = (index as f32 * 40.0).min(page_w - 80.0);
        let y = (index as f32 * 40.0).min(page_h - 80.0);
        pb.push_rect(
            KrillaRect::from_xywh(x + 20.0, y + 20.0, 60.0, 60.0)
                .unwrap_or_else(|| KrillaRect::from_xywh(0.0, 0.0, 10.0, 10.0).unwrap()),
        );
    }
    let krilla_path = match pb.finish() {
        Some(p) => p,
        None => return,
    };

    // Fill from the appearance stack: primary entry; gradients are sampled
    // at center with an explicit degradation (vector gradients POST_V1).
    let (fill_paint, _fill_entry_alpha) = match eff.primary_fill() {
        Some(entry) => match &entry.paint {
            petunia_design_document::Paint::None => (None, 1.0),
            petunia_design_document::Paint::Solid(token) => {
                let (paint, _) = resolve_fill_paint(Some(token.as_ref()), report);
                (Some(paint), entry.opacity as f32)
            }
            petunia_design_document::Paint::LinearGradient(g) => {
                report.degradations.push(DegradationItem {
                    code: "GRADIENT_FLATTENED".to_string(),
                    description: format!("{label} linear gradient sampled at center stop"),
                    grade: FidelityGrade::Approximate,
                });
                (sample_gradient_paint(g.sample_rgba(0.5)), entry.opacity as f32)
            }
            petunia_design_document::Paint::RadialGradient(g) => {
                report.degradations.push(DegradationItem {
                    code: "GRADIENT_FLATTENED".to_string(),
                    description: format!("{label} radial gradient sampled at center stop"),
                    grade: FidelityGrade::Approximate,
                });
                (sample_gradient_paint(g.sample_rgba(0.5)), entry.opacity as f32)
            }
        },
        None => match obj.fill.as_deref() {
            Some(token) => {
                let (paint, _) = resolve_fill_paint(Some(token), report);
                (Some(paint), 1.0)
            }
            None => (None, 1.0),
        },
    };
    if eff.fills.iter().filter(|f| f.visible).count() > 1 {
        report.degradations.push(DegradationItem {
            code: "MULTI_FILL_FLATTENED".to_string(),
            description: format!("{label} exports only its primary fill; secondary fills omitted"),
            grade: FidelityGrade::Approximate,
        });
    }

    // Stroke from the primary stroke entry (centered; alignment approximated).
    let krilla_stroke = eff.primary_stroke().and_then(|entry| match &entry.paint {
        petunia_design_document::Paint::None => None,
        petunia_design_document::Paint::Solid(token) => {
            let (paint, _) = resolve_fill_paint(Some(token.as_ref()), report);
            if entry.alignment != petunia_design_document::StrokeAlignment::Center {
                report.degradations.push(DegradationItem {
                    code: "STROKE_ALIGNMENT_APPROXIMATED".to_string(),
                    description: format!("{label} stroke alignment exported as centered"),
                    grade: FidelityGrade::EquivalentAppearance,
                });
            }
            let dash = if entry.dash_array.is_empty() {
                None
            } else {
                Some(StrokeDash {
                    array: entry.dash_array.iter().map(|d| *d as f32).collect(),
                    offset: entry.dash_offset as f32,
                })
            };
            Some(Stroke {
                paint,
                width: entry.width.max(0.0) as f32,
                miter_limit: entry.miter_limit as f32,
                line_cap: match entry.cap {
                    petunia_design_document::StrokeCap::Butt => LineCap::Butt,
                    petunia_design_document::StrokeCap::Round => LineCap::Round,
                    petunia_design_document::StrokeCap::Square => LineCap::Square,
                },
                line_join: match entry.join {
                    petunia_design_document::StrokeJoin::Miter => LineJoin::Miter,
                    petunia_design_document::StrokeJoin::Round => LineJoin::Round,
                    petunia_design_document::StrokeJoin::Bevel => LineJoin::Bevel,
                },
                opacity: NormalizedF32::new(entry.opacity as f32).unwrap_or(NormalizedF32::ONE),
                dash,
            })
        }
        _ => {
            report.degradations.push(DegradationItem {
                code: "GRADIENT_STROKE_FLATTENED".to_string(),
                description: format!("{label} gradient stroke omitted"),
                grade: FidelityGrade::Approximate,
            });
            None
        }
    });
    if eff.strokes.iter().filter(|s| s.visible).count() > 1 {
        report.degradations.push(DegradationItem {
            code: "MULTI_STROKE_FLATTENED".to_string(),
            description: format!(
                "{label} exports only its primary stroke; secondary strokes omitted"
            ),
            grade: FidelityGrade::Approximate,
        });
    }

    // Effects are not vector-exportable: explicit degradation per entry.
    for effect in eff.effects.iter().filter(|e| e.visible) {
        let kind = match &effect.kind {
            petunia_design_document::EffectKind::DropShadow { .. } => "drop shadow",
            petunia_design_document::EffectKind::InnerShadow { .. } => "inner shadow",
            petunia_design_document::EffectKind::GaussianBlur { .. } => "gaussian blur",
        };
        report.degradations.push(DegradationItem {
            code: "EFFECT_NOT_EXPORTED".to_string(),
            description: format!("{label} {kind} effect omitted from vector PDF"),
            grade: FidelityGrade::Approximate,
        });
    }

    // Non-normal blend modes on export: PDF supports them, pass through is
    // out of scope for the headless exporter — record and export as Normal.
    let stack_blend = eff.blend_mode;
    if stack_blend != petunia_design_document::BlendMode::Normal {
        report.degradations.push(DegradationItem {
            code: "BLEND_MODE_FLATTENED".to_string(),
            description: format!("{label} blend mode {stack_blend:?} exported as Normal"),
            grade: FidelityGrade::Approximate,
        });
    }

    let opacity_f32 = NormalizedF32::new(total_opacity as f32).unwrap_or(NormalizedF32::ONE);

    // Rotation about the bounds top-left, matching the document model
    // (`local_transform = T(origin) * R`). krilla angles are degrees.
    let rotation_guard = match obj.bounds {
        Some(b) if obj.rotation.abs() > f64::EPSILON => {
            krilla_surface.push_transform(&Transform::from_rotate_at(
                obj.rotation.to_degrees() as f32,
                b[0] as f32,
                b[1] as f32,
            ));
            1
        }
        _ => 0,
    };

    // Clip content to its mask outline via a real PDF clip path (10.5).
    // Mask boundaries carry vector shapes through `to_path`; un-outlinable
    // masks (e.g. text) degrade explicitly instead of clipping wrongly.
    let mut clip_guard = false;
    if let Some(mask_id) = obj.clip_mask_id {
        if let Some(mask) = surface.objects().iter().find(|o| o.id == mask_id) {
            let mask_verbs = mask.evaluated_path().verbs;
            if mask_verbs.is_empty() {
                report.degradations.push(DegradationItem {
                    code: "CLIP_MASK_UNOUTLINABLE".to_string(),
                    description: format!("{label} mask has no vector outline: drawn unclipped"),
                    grade: FidelityGrade::Approximate,
                });
            } else {
                let mut clip_pb = PathBuilder::new();
                for verb in &mask_verbs {
                    match verb {
                        petunia_design_geometry::PathVerb::MoveTo(p) => {
                            clip_pb.move_to(p.x as f32, p.y as f32);
                        }
                        petunia_design_geometry::PathVerb::LineTo(p) => {
                            clip_pb.line_to(p.x as f32, p.y as f32);
                        }
                        petunia_design_geometry::PathVerb::QuadTo(c, p) => {
                            clip_pb.quad_to(c.x as f32, c.y as f32, p.x as f32, p.y as f32);
                        }
                        petunia_design_geometry::PathVerb::CubicTo(c1, c2, p) => clip_pb.cubic_to(
                            c1.x as f32,
                            c1.y as f32,
                            c2.x as f32,
                            c2.y as f32,
                            p.x as f32,
                            p.y as f32,
                        ),
                        petunia_design_geometry::PathVerb::Close => clip_pb.close(),
                    }
                }
                if let Some(clip_path) = clip_pb.finish() {
                    krilla_surface.push_clip_path(&clip_path, &FillRule::NonZero);
                    clip_guard = true;
                }
            }
        } else {
            report.degradations.push(DegradationItem {
                code: "CLIP_MASK_MISSING".to_string(),
                description: format!("{label} references unknown mask: drawn unclipped"),
                grade: FidelityGrade::Approximate,
            });
        }
    }

    if let Some(paint) = fill_paint {
        krilla_surface.set_fill(Some(Fill {
            paint,
            opacity: opacity_f32,
            rule: FillRule::NonZero,
        }));
    } else {
        krilla_surface.set_fill(None);
    }
    krilla_surface.set_stroke(krilla_stroke);
    krilla_surface.draw_path(&krilla_path);

    krilla_surface.set_fill(None);
    krilla_surface.set_stroke(None);
    for _ in 0..rotation_guard {
        krilla_surface.pop();
    }
    if clip_guard {
        krilla_surface.pop();
    }
}

/// Samples a gradient center color into an sRGB paint for export flattening.
fn sample_gradient_paint(sample: Option<([f32; 3], f32)>) -> Option<Paint> {
    let (rgb, _) = sample?;
    Some(
        rgb::Color::new(
            (rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8,
            (rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8,
            (rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        )
        .into(),
    )
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
        "ptnd.red/500" => (rgb::Color::new(239, 68, 68).into(), NormalizedF32::ONE),
        "ptnd.blue/500" => (rgb::Color::new(59, 130, 246).into(), NormalizedF32::ONE),
        "ptnd.green/500" => (rgb::Color::new(34, 197, 94).into(), NormalizedF32::ONE),
        "ptnd.yellow/500" => (rgb::Color::new(234, 179, 8).into(), NormalizedF32::ONE),
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
    use petunia_design_document::DocumentMutator;
    use petunia_design_foundation::IdGenerator;

    #[test]
    fn export_document_emits_valid_pdf_stream() {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let s_id = gen.next_surface();
        let mut mutator = DocumentMutator::new(&mut doc);
        mutator.add_surface(s_id, "Page 1").unwrap();

        let mut obj1 = DocumentObject::new(gen.next_object(), "Rect");
        obj1.fill = Some("ptnd.red/500".to_string());
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
