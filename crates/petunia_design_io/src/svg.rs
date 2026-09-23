//! SVG vector export and import adapter (09.11, 09.22).

use petunia_design_document::Document;
use petunia_design_foundation::PetuniaError;
use petunia_design_geometry::{GPath, GPoint, PathVerb};

/// Exports a `GPath` as an SVG path data string (`d="..."`).
#[must_use]
pub fn export_path_d(path: &GPath) -> String {
    let mut d = String::new();
    for (i, verb) in path.verbs.iter().enumerate() {
        if i > 0 {
            d.push(' ');
        }
        match verb {
            PathVerb::MoveTo(p) => d.push_str(&format!("M {:.2} {:.2}", p.x, p.y)),
            PathVerb::LineTo(p) => d.push_str(&format!("L {:.2} {:.2}", p.x, p.y)),
            PathVerb::QuadTo(p1, p) => {
                d.push_str(&format!("Q {:.2} {:.2} {:.2} {:.2}", p1.x, p1.y, p.x, p.y))
            }
            PathVerb::CubicTo(p1, p2, p) => d.push_str(&format!(
                "C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2}",
                p1.x, p1.y, p2.x, p2.y, p.x, p.y
            )),
            PathVerb::Close => d.push('Z'),
        }
    }
    d
}

/// Parses an SVG path `d` string with `M/m L/l Q/q C/c H/h V/v Z/z`
/// commands into a `GPath` (F-13). Relative commands offset from the current
/// point; numbers are whitespace-separated (matching [`export_path_d`]).
pub fn parse_path_d(d: &str) -> Result<GPath, PetuniaError> {
    let mut path = GPath::new();
    let tokens: Vec<&str> = d.split_whitespace().collect();
    let mut i = 0;
    let mut current = GPoint::ORIGIN;

    macro_rules! take_num {
        ($label:expr) => {{
            if i >= tokens.len() {
                return Err(PetuniaError::invalid_input(concat!(
                    "incomplete ",
                    $label,
                    " command in SVG path"
                )));
            }
            let v: f64 = tokens[i].parse().map_err(|_| {
                PetuniaError::invalid_input(format!("invalid number in {0} command", $label))
            })?;
            i += 1;
            v
        }};
    }

    while i < tokens.len() {
        match tokens[i] {
            "M" | "m" => {
                let relative = tokens[i] == "m";
                i += 1;
                let x = take_num!("M");
                let y = take_num!("M");
                let pt = if relative {
                    GPoint::new(current.x + x, current.y + y)
                } else {
                    GPoint::new(x, y)
                };
                path.push(PathVerb::MoveTo(pt))
                    .map_err(PetuniaError::invalid_input)?;
                current = pt;
            }
            "L" | "l" => {
                let relative = tokens[i] == "l";
                i += 1;
                let x = take_num!("L");
                let y = take_num!("L");
                let pt = if relative {
                    GPoint::new(current.x + x, current.y + y)
                } else {
                    GPoint::new(x, y)
                };
                path.push(PathVerb::LineTo(pt))
                    .map_err(PetuniaError::invalid_input)?;
                current = pt;
            }
            "Q" | "q" => {
                let relative = tokens[i] == "q";
                i += 1;
                let x1 = take_num!("Q");
                let y1 = take_num!("Q");
                let x = take_num!("Q");
                let y = take_num!("Q");
                let (c, pt) = if relative {
                    (
                        GPoint::new(current.x + x1, current.y + y1),
                        GPoint::new(current.x + x, current.y + y),
                    )
                } else {
                    (GPoint::new(x1, y1), GPoint::new(x, y))
                };
                path.push(PathVerb::QuadTo(c, pt))
                    .map_err(PetuniaError::invalid_input)?;
                current = pt;
            }
            "C" | "c" => {
                let relative = tokens[i] == "c";
                i += 1;
                let x1 = take_num!("C");
                let y1 = take_num!("C");
                let x2 = take_num!("C");
                let y2 = take_num!("C");
                let x = take_num!("C");
                let y = take_num!("C");
                let (c1, c2, pt) = if relative {
                    (
                        GPoint::new(current.x + x1, current.y + y1),
                        GPoint::new(current.x + x2, current.y + y2),
                        GPoint::new(current.x + x, current.y + y),
                    )
                } else {
                    (
                        GPoint::new(x1, y1),
                        GPoint::new(x2, y2),
                        GPoint::new(x, y),
                    )
                };
                path.push(PathVerb::CubicTo(c1, c2, pt))
                    .map_err(PetuniaError::invalid_input)?;
                current = pt;
            }
            "H" | "h" => {
                let relative = tokens[i] == "h";
                i += 1;
                let x = take_num!("H");
                let pt = if relative {
                    GPoint::new(current.x + x, current.y)
                } else {
                    GPoint::new(x, current.y)
                };
                path.push(PathVerb::LineTo(pt))
                    .map_err(PetuniaError::invalid_input)?;
                current = pt;
            }
            "V" | "v" => {
                let relative = tokens[i] == "v";
                i += 1;
                let y = take_num!("V");
                let pt = if relative {
                    GPoint::new(current.x, current.y + y)
                } else {
                    GPoint::new(current.x, y)
                };
                path.push(PathVerb::LineTo(pt))
                    .map_err(PetuniaError::invalid_input)?;
                current = pt;
            }
            "Z" | "z" => {
                path.push(PathVerb::Close)
                    .map_err(PetuniaError::invalid_input)?;
                i += 1;
            }
            unknown => {
                return Err(PetuniaError::invalid_input(format!(
                    "unsupported or unrecognized SVG path command `{unknown}`"
                )));
            }
        }
    }

    Ok(path)
}

/// Exports a document's surfaces and objects into a standalone SVG document string (F-13).
/// Each object becomes a `<path>` with its canonical outline, resolved fill,
/// stroke, opacity and rotation. Invisible objects and mask boundaries are
/// skipped; masks are emitted as `<clipPath>` defs referenced by clipped
/// content. Non-vector state (gradients sampled at center, effects, blend
/// modes) is approximated with an explicit `<!-- -->` note.
#[must_use]
pub fn export_document_svg(document: &Document) -> String {
    let mut svg = String::new();
    svg.push_str(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1920 1080">"#);
    svg.push('\n');

    for surface in document.surfaces() {
        svg.push_str(&format!(
            r#"  <g id="{}" data-name="{}">"#,
            surface.id,
            escape_xml(&surface.name)
        ));
        svg.push('\n');

        // Clip path definitions for mask boundaries on this surface.
        for mask in surface
            .objects()
            .iter()
            .filter(|o| o.is_clip_mask && o.visible)
        {
            let d = export_path_d(&mask.evaluated_path());
            if d.is_empty() {
                continue;
            }
            svg.push_str(&format!(
                r#"    <defs><clipPath id="clip-{}"><path d="{}"/></clipPath></defs>"#,
                mask.id, d
            ));
            svg.push('\n');
        }

        for obj in surface.objects() {
            if !obj.visible || obj.is_clip_mask {
                continue;
            }
            svg.push_str(&export_object_svg(surface, obj));
            svg.push('\n');
        }

        svg.push_str("  </g>\n");
    }

    svg.push_str("</svg>\n");
    svg
}

/// Exports one object as an SVG `<path>` element string.
fn export_object_svg(
    surface: &petunia_design_document::Surface,
    obj: &petunia_design_document::DocumentObject,
) -> String {
    let eff = obj.effective_appearance();
    let entry_opacity = eff
        .primary_fill()
        .map(|f| f.opacity)
        .or_else(|| eff.primary_stroke().map(|s| s.opacity))
        .unwrap_or(1.0);
    let total_opacity = (obj.sampled_opacity() * entry_opacity).clamp(0.0, 1.0);

    // Outline; un-outlinable shapes (text) fall back to their bounds rect.
    // Attached text-on-path exports as <text><textPath> below instead.
    let is_text_on_path = matches!(
        &obj.shape,
        Some(petunia_design_document::ShapeKind::Text {
            on_path: Some(_),
            ..
        })
    );
    if is_text_on_path {
        return export_text_on_path_svg(surface, obj);
    }
    let mut outline = obj.evaluated_path();
    let mut notes: Vec<String> = Vec::new();
    if outline.verbs.is_empty() {
        if let Some(b) = obj.bounds {
            outline = GPath::rect(
                petunia_design_geometry::GRect::new(b[0], b[1], b[0] + b[2], b[1] + b[3]),
                0.0,
                0.0,
            );
            notes.push("text exported as bounds rect (glyph outlining requires font shaping)".to_string());
        } else {
            return format!(
                r#"    <!-- {} skipped: no outline and no bounds -->"#,
                escape_xml(&obj.name)
            );
        }
    }
    let d = export_path_d(&outline);

    // Fill: solid tokens resolve to hex; gradients sample center.
    let mut fill_attr = "none".to_string();
    if let Some(entry) = eff.primary_fill() {
        match &entry.paint {
            petunia_design_document::Paint::None => {}
            petunia_design_document::Paint::Solid(token) => {
                fill_attr = svg_color(token);
            }
            petunia_design_document::Paint::LinearGradient(g) => {
                fill_attr = g
                    .sample_rgba(0.5)
                    .map(|(rgb, _)| rgb_to_hex(rgb))
                    .unwrap_or_else(|| "none".to_string());
                notes.push("linear gradient sampled at center".to_string());
            }
            petunia_design_document::Paint::RadialGradient(g) => {
                fill_attr = g
                    .sample_rgba(0.5)
                    .map(|(rgb, _)| rgb_to_hex(rgb))
                    .unwrap_or_else(|| "none".to_string());
                notes.push("radial gradient sampled at center".to_string());
            }
        }
    } else if let Some(token) = obj.fill.as_deref() {
        fill_attr = svg_color(token);
    }
    if eff.fills.iter().filter(|f| f.visible).count() > 1 {
        notes.push("only primary fill exported".to_string());
    }

    // Stroke from the primary stroke entry.
    let mut stroke_attr = "stroke=\"none\"".to_string();
    if let Some(entry) = eff.primary_stroke() {
        match &entry.paint {
            petunia_design_document::Paint::Solid(token) => {
                stroke_attr = format!(
                    r#"stroke="{}" stroke-width="{:.2}""#,
                    svg_color(token),
                    entry.width.max(0.0)
                );
            }
            _ => {
                notes.push("non-solid stroke omitted".to_string());
            }
        }
        if entry.alignment != petunia_design_document::StrokeAlignment::Center {
            notes.push("stroke alignment exported as centered".to_string());
        }
        if !entry.dash_array.is_empty() {
            let dashes: Vec<String> =
                entry.dash_array.iter().map(|v| format!("{v:.2}")).collect();
            stroke_attr.push_str(&format!(r#" stroke-dasharray="{}""#, dashes.join(" ")));
        }
    }
    if eff.strokes.iter().filter(|s| s.visible).count() > 1 {
        notes.push("only primary stroke exported".to_string());
    }
    for effect in eff.effects.iter().filter(|e| e.visible) {
        let kind = match &effect.kind {
            petunia_design_document::EffectKind::DropShadow { .. } => "drop shadow",
            petunia_design_document::EffectKind::InnerShadow { .. } => "inner shadow",
            petunia_design_document::EffectKind::GaussianBlur { .. } => "gaussian blur",
        };
        notes.push(format!("{kind} effect omitted"));
    }
    if eff.blend_mode != petunia_design_document::BlendMode::Normal {
        notes.push(format!("blend mode {:?} exported as normal", eff.blend_mode));
    }

    let mut attrs = format!(
        r#"id="{}" data-name="{}" d="{}" fill="{}" {}"#,
        obj.id,
        escape_xml(&obj.name),
        d,
        fill_attr,
        stroke_attr
    );
    if total_opacity < 1.0 {
        attrs.push_str(&format!(r#" opacity="{:.3}""#, total_opacity));
    }
    if obj.rotation.abs() > f64::EPSILON {
        if let Some(b) = obj.bounds {
            // Top-left pivot matches the document model (`T(origin) * R`).
            attrs.push_str(&format!(
                r#" transform="rotate({:.2} {:.2} {:.2})""#,
                obj.rotation.to_degrees(),
                b[0],
                b[1]
            ));
        }
    }
    if let Some(mask_id) = obj.clip_mask_id {
        if surface.objects().iter().any(|o| o.id == mask_id) {
            attrs.push_str(&format!(r#" clip-path="url(#clip-{mask_id})""#));
        } else {
            notes.push("unknown clip mask: drawn unclipped".to_string());
        }
    }

    let mut out = format!("    <path {attrs}/>");
    if !notes.is_empty() {
        out.push_str(&format!("<!-- {}: {} -->", escape_xml(&obj.name), notes.join("; ")));
    }
    out
}

/// Exports attached text-on-path as `<text><textPath href="#target">`.
/// The target keeps its own `<path>` element (exported separately), so the
/// reference resolves by id. Glyph shaping stays future work: content rides
/// the evaluated outline between `start` and `end` fractions.
fn export_text_on_path_svg(
    surface: &petunia_design_document::Surface,
    obj: &petunia_design_document::DocumentObject,
) -> String {
    let (content, font_family, font_size, attachment) = match &obj.shape {
        Some(petunia_design_document::ShapeKind::Text {
            content,
            font_family,
            font_size,
            on_path: Some(attachment),
            ..
        }) => (content, font_family, font_size, attachment),
        _ => return String::new(),
    };
    let target_d = surface
        .objects()
        .iter()
        .find(|o| o.id == attachment.target)
        .map(|t| export_path_d(&t.evaluated_path()))
        .unwrap_or_default();
    let fill = obj
        .effective_appearance()
        .primary_fill()
        .map(|f| match &f.paint {
            petunia_design_document::Paint::Solid(token) => svg_color(token),
            _ => "currentColor".to_string(),
        })
        .unwrap_or_else(|| "currentColor".to_string());
    // SVG 2 href with xlink fallback for older renderers.
    format!(
        "    <text font-family=\"{}\" font-size=\"{:.1}\" fill=\"{}\"><textPath href=\"#{}\" xlink:href=\"#{}\" startOffset=\"{:.1}%\" side=\"left\"><!-- path d=\"{}\" span {:.3}..{:.3} -->{}</textPath></text>",
        escape_xml(font_family),
        font_size,
        fill,
        attachment.target,
        attachment.target,
        attachment.start * 100.0,
        target_d,
        attachment.start,
        attachment.end,
        escape_xml(content),
    )
}

/// Resolves a color token/literal to an SVG `#rrggbb` paint string.
fn svg_color(token: &str) -> String {
    let rgb = petunia_design_document::resolve_color_to_rgb(token);
    rgb_to_hex(rgb)
}

/// Formats a 0..=1 sRGB triple as `#rrggbb`.
fn rgb_to_hex(rgb: [f32; 3]) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8
    )
}

/// Escapes XML special characters in names and notes.
fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_d_roundtrip() {
        let mut path = GPath::new();
        path.push(PathVerb::MoveTo(GPoint::new(0.0, 0.0))).unwrap();
        path.push(PathVerb::LineTo(GPoint::new(10.0, 20.0)))
            .unwrap();
        path.push(PathVerb::Close).unwrap();

        let d = export_path_d(&path);
        assert_eq!(d, "M 0.00 0.00 L 10.00 20.00 Z");

        let parsed = parse_path_d(&d).expect("parse valid path d");
        assert_eq!(parsed.verbs.len(), 3);
        assert_eq!(parsed.verbs[0], PathVerb::MoveTo(GPoint::new(0.0, 0.0)));
        assert_eq!(parsed.verbs[1], PathVerb::LineTo(GPoint::new(10.0, 20.0)));
        assert_eq!(parsed.verbs[2], PathVerb::Close);
    }

    #[test]
    fn export_document_emits_valid_svg_envelope() {
        let doc = Document::new();
        let svg = export_document_svg(&doc);
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>\n"));
    }
}
