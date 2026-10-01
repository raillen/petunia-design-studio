//! SVG vector export and import adapter (09.11, 09.22).

use petunia_design_document::Document;
use petunia_design_foundation::PetuniaError;
use petunia_design_geometry::{GPath, PathVerb};

/// Exports a `GPath` as an SVG path data string (`d="..."`).
#[must_use]
pub fn export_path_d(path: &GPath) -> String {
    let mut d = String::new();
    for (i, verb) in path.verbs.iter().enumerate() {
        if i > 0 {
            d.push(' ');
        }
        match verb {
            PathVerb::MoveTo(p) => d.push_str(&format!("M {} {}", p.x, p.y)),
            PathVerb::LineTo(p) => d.push_str(&format!("L {} {}", p.x, p.y)),
            PathVerb::QuadTo(p1, p) => d.push_str(&format!("Q {} {} {} {}", p1.x, p1.y, p.x, p.y)),
            PathVerb::CubicTo(p1, p2, p) => d.push_str(&format!(
                "C {} {} {} {} {} {}",
                p1.x, p1.y, p2.x, p2.y, p.x, p.y
            )),
            PathVerb::Close => d.push('Z'),
        }
    }
    d
}

/// Parses bounded SVG path grammar, including compact numbers, repetition,
/// relative coordinates and reflected Bézier controls. Elliptical arcs are
/// outside this import subset and produce a capability error.
pub fn parse_path_d(d: &str) -> Result<GPath, PetuniaError> {
    crate::svg_input::parse_path(d)
}

/// Strict scene-based export. Unsupported resources fail before destination writes.
pub fn export_document_svg(document: &Document) -> Result<String, PetuniaError> {
    crate::svg_scene::export(document)
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_design_geometry::GPoint;

    #[test]
    fn path_d_roundtrip() {
        let mut path = GPath::new();
        path.push(PathVerb::MoveTo(GPoint::new(0.0, 0.0))).unwrap();
        path.push(PathVerb::LineTo(GPoint::new(10.0, 20.0)))
            .unwrap();
        path.push(PathVerb::Close).unwrap();

        let d = export_path_d(&path);
        assert_eq!(d, "M 0 0 L 10 20 Z");

        let parsed = parse_path_d(&d).expect("parse valid path d");
        assert_eq!(parsed.verbs.len(), 3);
        assert_eq!(parsed.verbs[0], PathVerb::MoveTo(GPoint::new(0.0, 0.0)));
        assert_eq!(parsed.verbs[1], PathVerb::LineTo(GPoint::new(10.0, 20.0)));
        assert_eq!(parsed.verbs[2], PathVerb::Close);
    }

    #[test]
    fn export_document_emits_valid_svg_envelope() {
        let doc = Document::new();
        let svg = export_document_svg(&doc).unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>\n"));
    }
}
