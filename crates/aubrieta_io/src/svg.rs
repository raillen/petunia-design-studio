//! SVG vector export and import adapter (09.11, 09.22).

use aubrieta_document::Document;
use aubrieta_foundation::AubrietaError;
use aubrieta_geometry::{GPath, GPoint, PathVerb};

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

/// Parses an SVG path `d` string containing `M`, `L` and `Z` commands into a `GPath`.
pub fn parse_path_d(d: &str) -> Result<GPath, AubrietaError> {
    let mut path = GPath::new();
    let tokens: Vec<&str> = d.split_whitespace().collect();
    let mut i = 0;

    while i < tokens.len() {
        match tokens[i] {
            "M" | "m" => {
                if i + 2 >= tokens.len() {
                    return Err(AubrietaError::invalid_input(
                        "incomplete M command in SVG path",
                    ));
                }
                let x: f64 = tokens[i + 1].parse().map_err(|_| {
                    AubrietaError::invalid_input("invalid x coordinate in M command")
                })?;
                let y: f64 = tokens[i + 2].parse().map_err(|_| {
                    AubrietaError::invalid_input("invalid y coordinate in M command")
                })?;
                path.push(PathVerb::MoveTo(GPoint::new(x, y)))
                    .map_err(AubrietaError::invalid_input)?;
                i += 3;
            }
            "L" | "l" => {
                if i + 2 >= tokens.len() {
                    return Err(AubrietaError::invalid_input(
                        "incomplete L command in SVG path",
                    ));
                }
                let x: f64 = tokens[i + 1].parse().map_err(|_| {
                    AubrietaError::invalid_input("invalid x coordinate in L command")
                })?;
                let y: f64 = tokens[i + 2].parse().map_err(|_| {
                    AubrietaError::invalid_input("invalid y coordinate in L command")
                })?;
                path.push(PathVerb::LineTo(GPoint::new(x, y)))
                    .map_err(AubrietaError::invalid_input)?;
                i += 3;
            }
            "Z" | "z" => {
                path.push(PathVerb::Close)
                    .map_err(AubrietaError::invalid_input)?;
                i += 1;
            }
            unknown => {
                return Err(AubrietaError::invalid_input(format!(
                    "unsupported or unrecognized SVG path command `{unknown}`"
                )));
            }
        }
    }

    Ok(path)
}

/// Exports a document's surfaces and objects into a standalone SVG document string.
#[must_use]
pub fn export_document_svg(document: &Document) -> String {
    let mut svg = String::new();
    svg.push_str(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1920 1080">"#);
    svg.push('\n');

    for surface in &document.surfaces {
        svg.push_str(&format!(
            r#"  <g id="{}" data-name="{}">"#,
            surface.id, surface.name
        ));
        svg.push('\n');

        for obj in &surface.objects {
            let fill_val = obj.fill.as_deref().unwrap_or("none");
            svg.push_str(&format!(
                r#"    <rect id="{}" data-name="{}" fill="{}" width="100" height="100"/>"#,
                obj.id, obj.name, fill_val
            ));
            svg.push('\n');
        }

        svg.push_str("  </g>\n");
    }

    svg.push_str("</svg>\n");
    svg
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
