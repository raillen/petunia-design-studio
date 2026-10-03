//! Vector PDF export for the supported subset using `krilla`.
//!
//! Maps Petunia Document surfaces into PDF pages with vector paths, fills,
//! strokes, RGB and CMYK color preservation, and preflight fidelity analysis.

use krilla::color::{cmyk, rgb};
use krilla::geom::{PathBuilder, Rect as KrillaRect, Transform};
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, FillRule, LineCap, LineJoin, Paint, Stroke, StrokeDash};
use krilla::Document as KrillaDocument;
use petunia_design_document::{Document, ShapeKind};
use petunia_design_foundation::PetuniaError;
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

/// Strict, scene-based export. Rasterization is admitted only by an explicit
/// option, and does not change any editable source in the document.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PdfExportOptions {
    pub author: Option<String>,
    pub title: Option<String>,
    pub creator: String,
    pub default_page_width: f32,
    pub default_page_height: f32,
    pub allow_degradations: bool,
    pub raster_fallback_dpi: f64,
    pub include_bleed: bool,
}
impl Default for PdfExportOptions {
    fn default() -> Self {
        Self {
            author: None,
            title: None,
            creator: "Petunia Design Studio".into(),
            default_page_width: 595.28,
            default_page_height: 841.89,
            allow_degradations: false,
            raster_fallback_dpi: 300.,
            include_bleed: false,
        }
    }
}
fn invalid(reason: impl Into<String>) -> PetuniaError {
    PetuniaError::invalid_input(reason)
}
fn unit(value: f64) -> Result<NormalizedF32, PetuniaError> {
    if !value.is_finite() {
        return Err(invalid("nonfinite PDF opacity"));
    }
    NormalizedF32::new(value as f32).ok_or_else(|| invalid("PDF opacity outside 0–1"))
}
fn number(value: f64) -> Result<f32, PetuniaError> {
    if !value.is_finite() || value.abs() > 10_000_000. {
        return Err(invalid("PDF coordinate budget exceeded"));
    }
    Ok(value as f32)
}
fn transform(frame: petunia_design_geometry::GAffine) -> Result<Transform, PetuniaError> {
    let [a, b, c, d, e, f] = frame.coeffs;
    Ok(Transform::from_row(
        number(a)?,
        number(b)?,
        number(c)?,
        number(d)?,
        number(e)?,
        number(f)?,
    ))
}
fn path(
    geometry: &petunia_design_geometry::GPath,
) -> Result<Option<krilla::geom::Path>, PetuniaError> {
    use petunia_design_geometry::PathVerb;
    let mut builder = PathBuilder::new();
    for verb in &geometry.verbs {
        match *verb {
            PathVerb::MoveTo(p) => builder.move_to(number(p.x)?, number(p.y)?),
            PathVerb::LineTo(p) => builder.line_to(number(p.x)?, number(p.y)?),
            PathVerb::QuadTo(c, p) => {
                builder.quad_to(number(c.x)?, number(c.y)?, number(p.x)?, number(p.y)?)
            }
            PathVerb::CubicTo(a, b, p) => builder.cubic_to(
                number(a.x)?,
                number(a.y)?,
                number(b.x)?,
                number(b.y)?,
                number(p.x)?,
                number(p.y)?,
            ),
            PathVerb::Close => builder.close(),
        }
    }
    Ok(builder.finish())
}
fn rule(node: &petunia_design_render::RenderNode) -> FillRule {
    if node.prepared_text().is_some()
        || node.source().fill_rule == petunia_design_geometry::FillRule::NonZero
    {
        FillRule::NonZero
    } else {
        FillRule::EvenOdd
    }
}
fn blend(mode: petunia_design_document::BlendMode) -> krilla::blend::BlendMode {
    use krilla::blend::BlendMode as P;
    use petunia_design_document::BlendMode as D;
    match mode {
        D::Normal => P::Normal,
        D::Multiply => P::Multiply,
        D::Screen => P::Screen,
        D::Overlay => P::Overlay,
        D::Darken => P::Darken,
        D::Lighten => P::Lighten,
        D::ColorDodge => P::ColorDodge,
        D::ColorBurn => P::ColorBurn,
        D::HardLight => P::HardLight,
        D::SoftLight => P::SoftLight,
        D::Difference => P::Difference,
        D::Exclusion => P::Exclusion,
        D::Hue => P::Hue,
        D::Saturation => P::Saturation,
        D::Color => P::Color,
        D::Luminosity => P::Luminosity,
    }
}
fn issue(report: &mut PreflightReport, code: &str, description: String, grade: FidelityGrade) {
    report.degradations.push(DegradationItem {
        code: code.into(),
        description,
        grade,
    });
}
fn checked_color(
    token: &str,
    has_cmyk_profile: bool,
    report: &mut PreflightReport,
) -> Result<krilla::color::Color, PetuniaError> {
    use krilla::color::{separation, RegularColor};
    let t = petunia_design_foundation::normalized(token);
    let t = t.trim();
    if let Some(ink) = petunia_design_color::icc::parse_cmyk_token(t)? {
        if !has_cmyk_profile {
            return Err(invalid("PDF CMYK requires an assigned ICC press profile"));
        }
        if ink
            .iter()
            .any(|v| (*v * 255. - (*v * 255.).round()).abs() > 1e-5)
        {
            issue(
                report,
                "CMYK_QUANTIZED_8",
                format!("{t}: PDF backend quantizes process color to 8-bit channels"),
                FidelityGrade::Approximate,
            );
        }
        let [c, m, y, k] = ink.map(|v| (v * 255.).round() as u8);
        return Ok(cmyk::Color::new(c, m, y, k).into());
    }
    if let Some(inner) = t.strip_prefix("spot(").and_then(|s| s.strip_suffix(')')) {
        let (name, fallback) = inner
            .split_once(',')
            .ok_or_else(|| invalid("PDF spot requires a name and fallback"))?;
        let name = name.trim();
        if name.is_empty() || name.len() > 512 || name.chars().any(char::is_control) {
            return Err(invalid("invalid PDF spot name"));
        }
        let color = checked_color(fallback.trim(), has_cmyk_profile, report)?;
        let krilla::color::Color::Regular(color) = color else {
            return Err(invalid("nested PDF spot fallback"));
        };
        return Ok(separation::Color::new(
            255,
            separation::SeparationSpace::new(
                separation::SeparationColorant::Custom(name.into()),
                color,
            ),
        )
        .into());
    }
    if t == "registration" {
        return Ok(separation::Color::new(
            255,
            separation::SeparationSpace::new(
                separation::SeparationColorant::AllColorants,
                RegularColor::from(cmyk::Color::new(255, 255, 255, 255)),
            ),
        )
        .into());
    }
    let known = matches!(
        t,
        "ptnd.red/500"
            | "ptnd.blue/500"
            | "ptnd.green/500"
            | "ptnd.yellow/500"
            | "ptnd.gray/900"
            | "ptnd.gray/500"
            | "ptnd.white"
            | "ptnd.black"
            | "ptnd.purple/500"
            | "ptnd.cyan/500"
    ) || t == "transparent"
        || t.starts_with('#')
        || t.starts_with("rgb(")
        || t.starts_with("gray(")
        || t.starts_with("lab(");
    if !known {
        issue(
            report,
            "COLOR_TOKEN_SUBSTITUTED",
            format!("Unregistered color {t}; neutral fallback matches the document renderer"),
            FidelityGrade::Approximate,
        );
    }
    let rgb = petunia_design_document::resolve_color_to_rgb(t).map(|v| (v * 255.).round() as u8);
    Ok(rgb::Color::new(rgb[0], rgb[1], rgb[2]).into())
}
fn paint(
    value: &petunia_design_document::Paint,
    has_profile: bool,
    report: &mut PreflightReport,
) -> Result<Option<Paint>, PetuniaError> {
    use krilla::paint::{LinearGradient, RadialGradient, SpreadMethod, Stop};
    use petunia_design_document::Paint as D;
    let mut stops =
        |stops: &[petunia_design_document::GradientStop]| -> Result<Vec<Stop>, PetuniaError> {
            let mut result = Vec::with_capacity(stops.len());
            for stop in stops {
                result.push(Stop {
                    offset: unit(stop.offset)?,
                    color: checked_color(&stop.color, has_profile, report)?,
                    opacity: unit(
                        stop.opacity
                            * f64::from(
                                petunia_design_document::resolve_color_to_rgba(&stop.color)[3],
                            ),
                    )?,
                });
            }
            result.sort_by(|a, b| a.offset.get().total_cmp(&b.offset.get()));
            if result.len() < 2 {
                return Err(invalid("PDF gradient requires at least two stops"));
            }
            // PDF shading stops must use one common color space. Preserve process
            // ink values; reject mixed spaces rather than guessing conversions.
            if result
                .iter()
                .any(|stop| matches!(stop.color, krilla::color::Color::Special(_)))
                || result.windows(2).any(|p| match (&p[0].color, &p[1].color) {
                    (krilla::color::Color::Regular(a), krilla::color::Color::Regular(b)) => {
                        std::mem::discriminant(a) != std::mem::discriminant(b)
                    }
                    _ => true,
                })
            {
                return Err(invalid("PDF gradient uses mixed color spaces"));
            }
            Ok(result)
        };
    Ok(match value {
        D::None => None,
        D::Solid(token) => Some(checked_color(token, has_profile, report)?.into()),
        D::LinearGradient(g) => Some(
            LinearGradient {
                x1: number(g.start[0])?,
                y1: number(g.start[1])?,
                x2: number(g.end[0])?,
                y2: number(g.end[1])?,
                transform: Transform::identity(),
                spread_method: SpreadMethod::Pad,
                stops: stops(&g.stops)?,
                anti_alias: true,
            }
            .into(),
        ),
        D::RadialGradient(g) => Some(
            RadialGradient {
                fx: number(g.center[0])?,
                fy: number(g.center[1])?,
                fr: 0.,
                cx: number(g.center[0])?,
                cy: number(g.center[1])?,
                cr: number(g.radius)?,
                transform: Transform::identity(),
                spread_method: SpreadMethod::Pad,
                stops: stops(&g.stops)?,
                anti_alias: true,
            }
            .into(),
        ),
    })
}

/// Export without publishing partially generated bytes after cancellation.
pub fn export_document_pdf(
    document: &Document,
    options: &PdfExportOptions,
) -> Result<(Vec<u8>, PreflightReport), PetuniaError> {
    export_document_pdf_cancellable(
        document,
        options,
        &petunia_design_jobs::CancellationToken::new(),
    )
}
pub fn export_document_pdf_cancellable(
    document: &Document,
    options: &PdfExportOptions,
    cancellation: &petunia_design_jobs::CancellationToken,
) -> Result<(Vec<u8>, PreflightReport), PetuniaError> {
    document.validate()?;
    if !options.raster_fallback_dpi.is_finite()
        || !(36.0..=1200.0).contains(&options.raster_fallback_dpi)
    {
        return Err(invalid("PDF fallback DPI must be 36–1200"));
    }
    let included: Vec<_> = document
        .surfaces()
        .iter()
        .filter(|s| s.export_enabled)
        .collect();
    if included.is_empty() {
        return Err(invalid("no surfaces enabled for PDF export"));
    }
    let mut profile: Option<petunia_design_color::IccProfile> = None;
    let mut admit_profile = |next: &petunia_design_color::IccProfile| -> Result<(), PetuniaError> {
        if !next.is_press_profile() {
            return Err(invalid("PDF press profile must be a CMYK output profile"));
        }
        if profile.as_ref().is_some_and(|p| p.id() != next.id()) {
            return Err(invalid("PDF content has different press profiles; explicitly convert native layers/surfaces to a common target"));
        }
        profile = Some(next.clone());
        Ok(())
    };
    for surface in &included {
        if let Some(next) = &surface.cmyk_profile {
            admit_profile(next)?;
        }
        for object in surface.objects() {
            check_cancelled(cancellation)?;
            match &object.shape {
                Some(ShapeKind::Raster { layer }) if layer.is_cmyk() => admit_profile(
                    layer
                        .cmyk_profile()
                        .ok_or_else(|| invalid("PDF native layer ICC profile missing"))?,
                )?,
                Some(ShapeKind::Image {
                    data: Some(source), ..
                }) => {
                    let image = petunia_design_raster::ImageCache::shared()
                        .prepare(source, &|| cancellation.is_cancelled())
                        .map_err(|e| invalid(e.to_string()))?;
                    if image.source_format().is_cmyk() {
                        let next = petunia_design_color::IccProfile::new(
                            "Embedded PDF CMYK profile".into(),
                            std::sync::Arc::new(
                                image
                                    .icc_profile()
                                    .ok_or_else(|| invalid("PDF native image ICC profile missing"))?
                                    .to_vec(),
                            ),
                        )?;
                        admit_profile(&next)?;
                    }
                }
                _ => {}
            }
        }
    }
    let settings = krilla::SerializeSettings {
        no_device_cs: true,
        cmyk_profile: profile
            .as_ref()
            .map(|p| {
                krilla::icc::ICCProfile::new(p.bytes())
                    .ok_or_else(|| invalid("PDF ICC profile rejected"))
            })
            .transpose()?,
        ..Default::default()
    };
    let mut output = KrillaDocument::new_with(settings);
    let mut metadata = krilla::metadata::Metadata::new().creator(options.creator.clone());
    if let Some(title) = &options.title {
        metadata = metadata.title(title.clone());
    }
    if let Some(author) = &options.author {
        metadata = metadata.authors(vec![author.clone()]);
    }
    output.set_metadata(metadata);
    let mut report = PreflightReport {
        surfaces: included.len(),
        objects: included.iter().map(|s| s.objects().len()).sum(),
        passed: true,
        degradations: Vec::new(),
    };
    for surface in included {
        check_cancelled(cancellation)?;
        let scene = petunia_design_render::RenderSurface::extract_cancellable(surface, &|| {
            cancellation.is_cancelled()
        })
        .map_err(|e| invalid(e.to_string()))?;
        let mut writer = PageWriter {
            scene: &scene,
            has_profile: surface.cmyk_profile.is_some(),
            report: &mut report,
            origin: [surface.origin[0], surface.origin[1]],
            cancellation,
            fonts: std::collections::HashMap::new(),
        };
        let mut rasterize = false;
        for id in scene.roots() {
            rasterize |= writer.preflight(*id, 0)?;
        }
        let bleed = if options.include_bleed {
            surface.bleed
        } else {
            petunia_design_document::Bleed::ZERO
        };
        let width = surface.dimensions[0] + bleed.left + bleed.right;
        let height = surface.dimensions[1] + bleed.top + bleed.bottom;
        writer.origin = [
            surface.origin[0] - bleed.left,
            surface.origin[1] - bleed.top,
        ];
        let media = KrillaRect::from_xywh(0., 0., number(width)?, number(height)?)
            .ok_or_else(|| invalid("invalid PDF media box"))?;
        let trim = KrillaRect::from_xywh(
            number(bleed.left)?,
            number(bleed.top)?,
            number(surface.dimensions[0])?,
            number(surface.dimensions[1])?,
        )
        .ok_or_else(|| invalid("invalid PDF trim box"))?;
        let settings = PageSettings::from_wh(number(width)?, number(height)?)
            .ok_or_else(|| invalid("invalid PDF page"))?
            .with_crop_box(Some(media))
            .with_bleed_box(Some(media))
            .with_trim_box(Some(trim));
        if rasterize && !options.allow_degradations {
            return Err(invalid("PDF contains effects/modifiers requiring explicit rasterization; export PNG or allow degradations"));
        }
        let mut page = output.start_page_with(settings);
        let mut canvas = page.surface();
        if rasterize {
            let scale = options.raster_fallback_dpi / 72.;
            let pw = (width * scale).ceil();
            let ph = (height * scale).ceil();
            if pw * ph > petunia_design_render::RenderLimits::default().max_output_pixels as f64 {
                return Err(invalid("PDF fallback pixel budget exceeded"));
            }
            let bg = surface.background_rgba8()?;
            let request = petunia_design_render::RenderRequest {
                viewport: petunia_design_geometry::GRect::new(
                    surface.origin[0] - bleed.left,
                    surface.origin[1] - bleed.top,
                    surface.origin[0] + surface.dimensions[0] + bleed.right,
                    surface.origin[1] + surface.dimensions[1] + bleed.bottom,
                ),
                width: pw as u32,
                height: ph as u32,
                background: [0; 4],
            };
            let mut backdrop = petunia_design_render::PixelBufferRgba8::with_fill(
                request.width,
                request.height,
                [0; 4],
            );
            for y in 0..request.height {
                check_cancelled(cancellation)?;
                let wy = request.viewport.y0
                    + (f64::from(y) + 0.5) * request.viewport.height() / f64::from(request.height);
                if wy < surface.origin[1] || wy >= surface.origin[1] + surface.dimensions[1] {
                    continue;
                }
                for x in 0..request.width {
                    let wx = request.viewport.x0
                        + (f64::from(x) + 0.5) * request.viewport.width()
                            / f64::from(request.width);
                    if wx >= surface.origin[0] && wx < surface.origin[0] + surface.dimensions[0] {
                        let offset = (y as usize * request.width as usize + x as usize) * 4;
                        backdrop.data[offset..offset + 4].copy_from_slice(&bg);
                    }
                }
            }
            let pixels = petunia_design_render::CpuRenderer::default()
                .render_over_cancellable(&scene, request, &backdrop, cancellation)
                .map_err(|e| invalid(e.to_string()))?;
            let image = image_from_rgba(
                pixels.width,
                pixels.height,
                petunia_design_raster::PixelFormat::Rgba8,
                &pixels.data,
            )?;
            canvas.draw_image(
                image,
                krilla::geom::Size::from_wh(number(width)?, number(height)?)
                    .ok_or_else(|| invalid("PDF raster size"))?,
            );
            issue(writer.report,"PAGE_RASTERIZED",format!("Surface {} rendered at {} DPI; vector/editable/font/ink semantics are rasterized",surface.name,options.raster_fallback_dpi),FidelityGrade::DestructiveDegradation);
            let mut native_ink = surface.objects().iter().any(|object| matches!(&object.shape, Some(ShapeKind::Raster { layer }) if layer.is_cmyk()));
            for object in surface.objects() {
                if let Some(ShapeKind::Image {
                    data: Some(source), ..
                }) = &object.shape
                {
                    native_ink |= petunia_design_raster::ImageCache::shared()
                        .prepare(source, &|| cancellation.is_cancelled())
                        .map_err(|e| invalid(e.to_string()))?
                        .source_format()
                        .is_cmyk();
                }
            }
            if native_ink {
                issue(writer.report, "CMYK_PAGE_CONVERTED_TO_RGB", "Native process ink was explicitly converted to the RGB composite by page rasterization; ink separations are not preserved".into(), FidelityGrade::DestructiveDegradation);
            }
        } else {
            let mut scope = ScopedSurface::new(&mut canvas);
            let canvas = &mut scope;
            if let Some(background) = &surface.background {
                canvas.set_fill(Some(Fill {
                    paint: checked_color(background, writer.has_profile, writer.report)?.into(),
                    opacity: unit(f64::from(
                        petunia_design_document::resolve_color_to_rgba(background)[3],
                    ))?,
                    rule: FillRule::NonZero,
                }));
                let geometry = petunia_design_geometry::GPath::rect(
                    petunia_design_geometry::GRect::new(
                        bleed.left,
                        bleed.top,
                        bleed.left + surface.dimensions[0],
                        bleed.top + surface.dimensions[1],
                    ),
                    0.,
                    0.,
                );
                if let Some(path) = path(&geometry)? {
                    canvas.draw_path(&path);
                }
                canvas.set_fill(None);
            }
            for id in scene.roots() {
                writer.node(canvas, *id, 0, false)?;
            }
        }
        canvas.finish();
        page.finish();
    }
    if !options.allow_degradations
        && report.degradations.iter().any(|d| {
            matches!(
                d.grade,
                FidelityGrade::Approximate
                    | FidelityGrade::DestructiveDegradation
                    | FidelityGrade::Unsupported
            )
        })
    {
        return Err(invalid(format!(
            "PDF preflight rejected: {}",
            report
                .degradations
                .iter()
                .map(|d| d.code.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    check_cancelled(cancellation)?;
    let bytes = output
        .finish()
        .map_err(|e| PetuniaError::io(format!("finalize PDF: {e:?}")))?;
    if bytes.len() > 256 * 1024 * 1024 {
        return Err(invalid("PDF output byte budget exceeded"));
    }
    check_cancelled(cancellation)?;
    Ok((bytes, report))
}
fn check_cancelled(token: &petunia_design_jobs::CancellationToken) -> Result<(), PetuniaError> {
    if token.is_cancelled() {
        Err(invalid("PDF export cancelled"))
    } else {
        Ok(())
    }
}
struct PageWriter<'a> {
    scene: &'a petunia_design_render::RenderSurface,
    has_profile: bool,
    report: &'a mut PreflightReport,
    origin: [f64; 2],
    cancellation: &'a petunia_design_jobs::CancellationToken,
    fonts:
        std::collections::HashMap<(petunia_design_foundation::ObjectId, usize), krilla::text::Font>,
}
impl PageWriter<'_> {
    fn preflight(
        &mut self,
        id: petunia_design_foundation::ObjectId,
        depth: usize,
    ) -> Result<bool, PetuniaError> {
        check_cancelled(self.cancellation)?;
        if depth >= 128 {
            return Err(invalid("PDF hierarchy depth budget"));
        }
        let node = self
            .scene
            .node(id)
            .ok_or_else(|| invalid("missing PDF node"))?;
        let source = node.source();
        if !source.visible {
            return Ok(false);
        }
        let app = source.effective_appearance();
        let mut rasterize = app.effects.iter().any(|e| e.visible)
            || app.adjustments.iter().any(|a| a.visible)
            || app.strokes.iter().any(|s| {
                s.visible && s.alignment != petunia_design_document::StrokeAlignment::Center
            })
            || source.modifiers.iter().any(|m| {
                m.enabled
                    && matches!(
                        m.kind,
                        petunia_design_document::ModifierKind::TransparentGradient { .. }
                    )
            })
            || source.mask_mode == petunia_design_document::MaskMode::Luminance;
        for value in app
            .fills
            .iter()
            .filter(|f| f.visible)
            .map(|f| &f.paint)
            .chain(app.strokes.iter().filter(|s| s.visible).map(|s| &s.paint))
        {
            let _ = paint(value, self.has_profile, self.report)?;
        }
        if let Some(text) = node.prepared_text() {
            if text.missing_family() {
                issue(
                    self.report,
                    "FONT_SUBSTITUTED",
                    format!(
                        "{} uses a fallback font; missing requested family",
                        source.name
                    ),
                    FidelityGrade::Approximate,
                );
            }
            if text.fonts().iter().any(|f| !f.subset_embedding_allowed) {
                return Err(invalid(format!(
                    "{}: font permissions prohibit subset embedding",
                    source.name
                )));
            }
            if source.modifiers.iter().any(|m| m.enabled) {
                rasterize = true;
            }
            if source.text_style.flow == petunia_design_document::TextFlow::Frame
                && source.bounds.is_some_and(|b| text.flow_height() > b[3])
            {
                issue(
                    self.report,
                    "TEXT_OVERSET",
                    format!("{} has text clipped by its frame height", source.name),
                    FidelityGrade::EquivalentAppearance,
                );
            }
        }
        for child in source.children.iter().copied() {
            rasterize |= self.preflight(child, depth + 1)?;
        }
        Ok(rasterize)
    }
    fn clip(
        &mut self,
        out: &mut ScopedSurface,
        id: petunia_design_foundation::ObjectId,
        mode: petunia_design_document::MaskMode,
        depth: usize,
    ) -> Result<(), PetuniaError> {
        let mask = self
            .scene
            .node(id)
            .ok_or_else(|| invalid("missing PDF mask"))?;
        if mode == petunia_design_document::MaskMode::Vector {
            let geometry = if mask.source().visible {
                mask.geometry().transformed(
                    petunia_design_geometry::GAffine::translate(-self.origin[0], -self.origin[1])
                        .after(mask.local_to_world()),
                )
            } else {
                petunia_design_geometry::GPath::new()
            };
            // PDF's W n with an empty path is a genuinely empty clip.
            let clip = path(&geometry)?.unwrap_or_else(|| {
                let mut b = PathBuilder::new();
                b.move_to(0., 0.);
                b.line_to(0., 0.);
                b.close();
                b.finish().expect("empty clip path")
            });
            out.push_clip_path(&clip, &rule(mask));
        } else {
            let mut builder = out.stream_builder();
            let mut content = builder.surface();
            self.node(&mut content, id, depth + 1, true)?;
            content.finish();
            let stream = builder.finish();
            out.push_mask(krilla::mask::Mask::new(
                stream,
                krilla::mask::MaskType::Alpha,
            ));
        }
        Ok(())
    }
    fn node(
        &mut self,
        out: &mut krilla::surface::Surface,
        id: petunia_design_foundation::ObjectId,
        depth: usize,
        as_mask: bool,
    ) -> Result<(), PetuniaError> {
        use petunia_design_document::{ContainerRole, MaskMode, ShapeKind, TextFlow};
        check_cancelled(self.cancellation)?;
        if depth >= 128 {
            return Err(invalid("PDF composition depth budget"));
        }
        let node = self
            .scene
            .node(id)
            .ok_or_else(|| invalid("missing PDF object"))?;
        let source = node.source();
        if !source.visible || (source.is_clip_mask && !as_mask) {
            return Ok(());
        }
        let app = source.effective_appearance();
        let mut scope = ScopedSurface::new(out);
        let out = &mut scope;
        out.push_blend_mode(blend(app.blend_mode));
        out.push_opacity(unit(app.opacity)?);
        out.push_isolated();
        let mut clip_count = 0;
        if let Some(mask_id) = source.clip_mask_id {
            let grouped = source
                .parent
                .and_then(|id| self.scene.node(id))
                .is_some_and(|n| {
                    n.source().role == Some(ContainerRole::ClipGroup)
                        && n.children().contains(&mask_id)
                });
            if !grouped {
                let mask = self
                    .scene
                    .node(mask_id)
                    .ok_or_else(|| invalid("PDF mask reference missing"))?;
                let mode = if mask.source().mask_mode != MaskMode::Vector {
                    mask.source().mask_mode
                } else {
                    source.mask_mode
                };
                self.clip(out, mask_id, mode, depth)?;
                clip_count += 1;
            }
        }
        if source.role == Some(ContainerRole::ClipGroup) {
            let masks: Vec<_> = source
                .children
                .iter()
                .copied()
                .filter(|id| {
                    self.scene
                        .node(*id)
                        .is_some_and(|n| n.source().is_clip_mask)
                })
                .collect();
            if masks.len() != 1 {
                return Err(invalid("PDF clip group requires one mask"));
            }
            self.clip(
                out,
                masks[0],
                self.scene
                    .node(masks[0])
                    .expect("admitted mask")
                    .source()
                    .mask_mode,
                depth,
            )?;
            clip_count += 1;
        }
        out.push_transform(&transform(
            petunia_design_geometry::GAffine::translate(-self.origin[0], -self.origin[1])
                .after(node.local_to_world()),
        )?);
        let geometry = path(node.geometry())?;
        let frame_clip =
            node.prepared_text().is_some() && source.text_style.flow == TextFlow::Frame;
        if frame_clip {
            let [_, _, w, h] = source
                .bounds
                .ok_or_else(|| invalid("PDF text frame missing"))?;
            let clip = path(&petunia_design_geometry::GPath::rect(
                petunia_design_geometry::GRect::new(0., 0., w, h),
                0.,
                0.,
            ))?
            .ok_or_else(|| invalid("empty PDF text frame"))?;
            out.push_clip_path(&clip, &FillRule::NonZero);
        }
        out.set_fill(None);
        out.set_stroke(None);
        if let Some(ShapeKind::Image { data, .. }) = &source.shape {
            let data = data
                .as_ref()
                .ok_or_else(|| invalid("PDF linked image is unavailable"))?;
            let decoded = petunia_design_raster::decode_image(data.as_slice(), Default::default())
                .map_err(|e| invalid(e.to_string()))?;
            let image = if decoded.format.is_cmyk() {
                let profile = petunia_design_color::IccProfile::new(
                    "Embedded PDF image profile".into(),
                    decoded
                        .icc_profile
                        .ok_or_else(|| invalid("PDF CMYK image profile missing"))?,
                )?;
                image_from_cmyka(
                    decoded.width,
                    decoded.height,
                    decoded.format,
                    &decoded.data,
                    &profile,
                )?
            } else {
                let raw = crate::import_raster(data.as_slice(), 128 * 1024 * 1024)?;
                image_from_rgba(raw.width, raw.height, raw.format, &raw.data)?
            };
            if let Some(coverage) = &geometry {
                out.push_clip_path(coverage, &rule(node));
            }
            let [_, _, w, h] = source
                .bounds
                .ok_or_else(|| invalid("PDF image bounds missing"))?;
            out.draw_image(
                image,
                krilla::geom::Size::from_wh(number(w)?, number(h)?)
                    .ok_or_else(|| invalid("PDF image dimensions"))?,
            );
            if geometry.is_some() {
                out.pop();
            }
        } else if let Some(ShapeKind::Raster { layer }) = &source.shape {
            let image = if layer.is_cmyk() {
                let data = layer.cmyka_bytes(&|| self.cancellation.is_cancelled())?;
                let profile = layer
                    .cmyk_profile()
                    .ok_or_else(|| invalid("PDF CMYK raster profile missing"))?;
                image_from_cmyka(
                    layer.width(),
                    layer.height(),
                    layer.tiles().format,
                    &data,
                    profile,
                )?
            } else {
                let mut data = Vec::new();
                let sixteen =
                    layer.tiles().format.bit_depth() == petunia_design_raster::BitDepth::Sixteen;
                let count =
                    layer.width() as usize * layer.height() as usize * if sixteen { 8 } else { 4 };
                data.try_reserve_exact(count)
                    .map_err(|_| invalid("PDF raster allocation"))?;
                for y in 0..layer.height() {
                    check_cancelled(self.cancellation)?;
                    for x in 0..layer.width() {
                        for value in layer.pixel(i64::from(x), i64::from(y))? {
                            if sixteen {
                                data.extend_from_slice(
                                    &((value * 65535.).round() as u16).to_le_bytes(),
                                );
                            } else {
                                data.push((value * 255.).round() as u8);
                            }
                        }
                    }
                }
                image_from_rgba(
                    layer.width(),
                    layer.height(),
                    if sixteen {
                        petunia_design_raster::PixelFormat::Rgba16
                    } else {
                        petunia_design_raster::PixelFormat::Rgba8
                    },
                    &data,
                )?
            };
            if let Some(coverage) = &geometry {
                out.push_clip_path(coverage, &rule(node));
            }
            let [_, _, w, h] = source
                .bounds
                .ok_or_else(|| invalid("PDF raster bounds missing"))?;
            out.draw_image(
                image,
                krilla::geom::Size::from_wh(number(w)?, number(h)?)
                    .ok_or_else(|| invalid("PDF raster dimensions"))?,
            );
            if geometry.is_some() {
                out.pop();
            }
        } else {
            for fill in app.fills.iter().filter(|f| f.visible) {
                out.push_blend_mode(blend(fill.blend_mode));
                out.set_fill(
                    paint(&fill.paint, self.has_profile, self.report)?.map(|paint| Fill {
                        paint,
                        opacity: unit(fill.opacity * paint_alpha(&fill.paint))
                            .expect("validated fill opacity"),
                        rule: rule(node),
                    }),
                );
                self.draw(out, node, geometry.as_ref())?;
                out.pop();
            }
        }
        out.set_fill(None);
        for stroke in app.strokes.iter().filter(|s| s.visible && s.width > 0.) {
            let Some(paint) = paint(&stroke.paint, self.has_profile, self.report)? else {
                continue;
            };
            out.set_stroke(Some(Stroke {
                paint,
                width: number(stroke.width)?,
                miter_limit: number(stroke.miter_limit)?,
                opacity: unit(stroke.opacity * paint_alpha(&stroke.paint))?,
                line_cap: match stroke.cap {
                    petunia_design_document::StrokeCap::Butt => LineCap::Butt,
                    petunia_design_document::StrokeCap::Round => LineCap::Round,
                    petunia_design_document::StrokeCap::Square => LineCap::Square,
                },
                line_join: match stroke.join {
                    petunia_design_document::StrokeJoin::Miter => LineJoin::Miter,
                    petunia_design_document::StrokeJoin::Round => LineJoin::Round,
                    petunia_design_document::StrokeJoin::Bevel => LineJoin::Bevel,
                },
                dash: if stroke.dash_array.is_empty() {
                    None
                } else {
                    Some(StrokeDash {
                        array: stroke
                            .dash_array
                            .iter()
                            .map(|v| number(*v))
                            .collect::<Result<_, _>>()?,
                        offset: number(stroke.dash_offset)?,
                    })
                },
            }));
            out.push_blend_mode(blend(stroke.blend_mode));
            self.draw(out, node, geometry.as_ref())?;
            out.pop();
        }
        out.set_stroke(None);
        out.set_fill(None);
        if frame_clip {
            out.pop();
        }
        out.pop();
        for child in source.children.iter().copied() {
            self.node(out, child, depth + 1, as_mask)?;
        }
        for _ in 0..clip_count {
            out.pop();
        }
        out.pop();
        out.pop();
        out.pop();
        Ok(())
    }
    fn draw(
        &mut self,
        out: &mut krilla::surface::Surface,
        node: &petunia_design_render::RenderNode,
        geometry: Option<&krilla::geom::Path>,
    ) -> Result<(), PetuniaError> {
        if let Some(text) = node.prepared_text() {
            let Some(petunia_design_document::ShapeKind::Text {
                content, font_size, ..
            }) = &node.source().shape
            else {
                return Err(invalid("prepared PDF text source missing"));
            };
            for glyph in text.glyphs() {
                check_cancelled(self.cancellation)?;
                let key = (node.id(), glyph.font_resource);
                if let std::collections::hash_map::Entry::Vacant(entry) = self.fonts.entry(key) {
                    let source = &text.fonts()[glyph.font_resource];
                    let font = krilla::text::Font::new(
                        krilla::Data::from(source.bytes.clone()),
                        source.face_index,
                    )
                    .ok_or_else(|| invalid("PDF font parser rejected shaped face"))?;
                    entry.insert(font);
                }
                let item = krilla::text::KrillaGlyph {
                    glyph_id: krilla::text::GlyphId::new(u32::from(glyph.glyph_id)),
                    text_range: glyph.source_range[0]..glyph.source_range[1],
                    x_advance: glyph.advance / *font_size as f32,
                    x_offset: 0.,
                    y_offset: 0.,
                    y_advance: 0.,
                    location: None,
                };
                out.draw_glyphs(
                    krilla::geom::Point::from_xy(number(glyph.origin.x)?, number(glyph.origin.y)?),
                    &[item],
                    self.fonts[&key].clone(),
                    content,
                    *font_size as f32,
                    false,
                );
            }
        } else if let Some(geometry) = geometry {
            out.draw_path(geometry);
        }
        Ok(())
    }
}
#[derive(Clone)]
struct PdfImage {
    colors: std::sync::Arc<Vec<u8>>,
    alpha: std::sync::Arc<Vec<u8>>,
    profile: std::sync::Arc<Vec<u8>>,
    width: u32,
    height: u32,
    sixteen: bool,
    cmyk: bool,
    digest: [u8; 32],
}
impl std::hash::Hash for PdfImage {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        self.digest.hash(h);
        self.width.hash(h);
        self.height.hash(h);
        self.sixteen.hash(h);
        self.cmyk.hash(h);
    }
}
impl krilla::image::CustomImage for PdfImage {
    fn color_channel(&self) -> &[u8] {
        &self.colors
    }
    fn alpha_channel(&self) -> Option<&[u8]> {
        Some(&self.alpha)
    }
    fn bits_per_component(&self) -> krilla::image::BitsPerComponent {
        if self.sixteen {
            krilla::image::BitsPerComponent::Sixteen
        } else {
            krilla::image::BitsPerComponent::Eight
        }
    }
    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    fn icc_profile(&self) -> Option<&[u8]> {
        Some(&self.profile)
    }
    fn color_space(&self) -> krilla::image::ImageColorspace {
        if self.cmyk {
            krilla::image::ImageColorspace::Cmyk
        } else {
            krilla::image::ImageColorspace::Rgb
        }
    }
}
fn image_from_rgba(
    width: u32,
    height: u32,
    format: petunia_design_raster::PixelFormat,
    data: &[u8],
) -> Result<krilla::image::Image, PetuniaError> {
    use sha2::{Digest, Sha256};
    let sixteen = match format {
        petunia_design_raster::PixelFormat::Rgba8 => false,
        petunia_design_raster::PixelFormat::Rgba16 => true,
        _ => return Err(invalid("PDF image requires straight RGBA")),
    };
    let channels = if sixteen { 8 } else { 4 };
    let count = u64::from(width) * u64::from(height);
    if count > 16_777_216 || count as usize * channels != data.len() {
        return Err(invalid("PDF image pixel budget/layout"));
    }
    let mut colors = Vec::with_capacity(data.len() / 4 * 3);
    let mut alpha = Vec::with_capacity(data.len() / 4);
    for pixel in data.chunks_exact(channels) {
        if sixteen {
            for channel in pixel[..6].as_chunks::<2>().0.iter() {
                colors.extend_from_slice(&[channel[1], channel[0]]);
            }
            alpha.extend_from_slice(&[pixel[7], pixel[6]]);
        } else {
            colors.extend_from_slice(&pixel[..3]);
            alpha.push(pixel[3]);
        }
    }
    let image = PdfImage {
        colors: std::sync::Arc::new(colors),
        alpha: std::sync::Arc::new(alpha),
        profile: std::sync::Arc::new(petunia_design_color::IccProfile::srgb()?.bytes().to_vec()),
        width,
        height,
        sixteen,
        cmyk: false,
        digest: Sha256::digest(data).into(),
    };
    krilla::image::Image::from_custom(image, true).map_err(invalid)
}

fn image_from_cmyka(
    width: u32,
    height: u32,
    format: petunia_design_raster::PixelFormat,
    data: &[u8],
    profile: &petunia_design_color::IccProfile,
) -> Result<krilla::image::Image, PetuniaError> {
    use sha2::{Digest, Sha256};
    if !format.is_cmyk() || !profile.is_press_profile() {
        return Err(invalid("PDF native CMYK layout/profile mismatch"));
    }
    let bpp = format.bytes_per_pixel();
    let count = u64::from(width) * u64::from(height);
    if width == 0
        || height == 0
        || count > 16_777_216
        || data.len() > 128 * 1024 * 1024
        || count as usize * bpp != data.len()
    {
        return Err(invalid("PDF native CMYK pixel budget/layout"));
    }
    let sixteen = format == petunia_design_raster::PixelFormat::Cmyka16;
    let mut colors = Vec::new();
    let mut alpha = Vec::new();
    colors
        .try_reserve_exact(data.len() / 5 * 4)
        .map_err(|_| invalid("PDF CMYK color allocation"))?;
    alpha
        .try_reserve_exact(data.len() / 5)
        .map_err(|_| invalid("PDF CMYK alpha allocation"))?;
    for pixel in data.chunks_exact(bpp) {
        if sixteen {
            for p in pixel[..8].as_chunks::<2>().0 {
                colors.extend_from_slice(&[p[1], p[0]]);
            }
            alpha.extend_from_slice(&[pixel[9], pixel[8]]);
        } else {
            colors.extend_from_slice(&pixel[..4]);
            alpha.push(pixel[4]);
        }
    }
    let mut hash = Sha256::new();
    hash.update(data);
    hash.update(profile.bytes());
    hash.update([u8::from(sixteen), 4]);
    krilla::image::Image::from_custom(
        PdfImage {
            colors: std::sync::Arc::new(colors),
            alpha: std::sync::Arc::new(alpha),
            profile: std::sync::Arc::new(profile.bytes().to_vec()),
            width,
            height,
            sixteen,
            cmyk: true,
            digest: hash.finalize().into(),
        },
        true,
    )
    .map_err(invalid)
}

// Always unwind graphics state on a recoverable export error. krilla requires
// balanced pushes even when Surface is dropped before finish().
struct ScopedSurface<'s, 'd> {
    surface: &'s mut krilla::surface::Surface<'d>,
    depth: usize,
}
impl<'s, 'd> ScopedSurface<'s, 'd> {
    fn new(surface: &'s mut krilla::surface::Surface<'d>) -> Self {
        Self { surface, depth: 0 }
    }
    fn push_transform(&mut self, t: &Transform) {
        self.surface.push_transform(t);
        self.depth += 1;
    }
    fn push_clip_path(&mut self, p: &krilla::geom::Path, rule: &FillRule) {
        self.surface.push_clip_path(p, rule);
        self.depth += 1;
    }
    fn push_blend_mode(&mut self, b: krilla::blend::BlendMode) {
        self.surface.push_blend_mode(b);
        self.depth += 1;
    }
    fn push_opacity(&mut self, o: NormalizedF32) {
        self.surface.push_opacity(o);
        self.depth += 1;
    }
    fn push_isolated(&mut self) {
        self.surface.push_isolated();
        self.depth += 1;
    }
    fn push_mask(&mut self, m: krilla::mask::Mask) {
        self.surface.push_mask(m);
        self.depth += 1;
    }
    fn pop(&mut self) {
        self.surface.pop();
        self.depth -= 1;
    }
}
impl<'s, 'd> std::ops::Deref for ScopedSurface<'s, 'd> {
    type Target = krilla::surface::Surface<'d>;
    fn deref(&self) -> &Self::Target {
        self.surface
    }
}
impl std::ops::DerefMut for ScopedSurface<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.surface
    }
}
impl Drop for ScopedSurface<'_, '_> {
    fn drop(&mut self) {
        while self.depth > 0 {
            self.pop();
        }
    }
}

fn paint_alpha(paint: &petunia_design_document::Paint) -> f64 {
    match paint {
        petunia_design_document::Paint::Solid(token) => {
            f64::from(petunia_design_document::resolve_color_to_rgba(token)[3])
        }
        _ => 1.,
    }
}
