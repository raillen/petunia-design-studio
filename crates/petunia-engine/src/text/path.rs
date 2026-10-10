//! Glyph placement by arc length on evaluated authorial geometry.
use super::{evaluate_text, FontRegistry, TextEvaluationError, TextLayout};
use petunia_core::{
    CharacterStyle, ParagraphStyle, Point, StyleRegistry, TextContainer, TextObject, Tolerance,
    VectorPath,
};

/// The supplied path is in the text object's coordinate space. Glyph outlines
/// rotate as rigid shapes at their baseline origins; source strings stay intact.
pub fn evaluate_text_on_path(
    object: &TextObject,
    path: &VectorPath,
    styles: &StyleRegistry,
    fonts: &FontRegistry,
    character: &CharacterStyle,
    paragraph: &ParagraphStyle,
) -> Result<TextLayout, TextEvaluationError> {
    let TextContainer::OnPath(reference) = object.container else {
        return Err(TextEvaluationError::Invalid("text is not on a path".into()));
    };
    path.validate()
        .map_err(|error| TextEvaluationError::Invalid(error.to_string()))?;
    let contour = path
        .contours
        .first()
        .ok_or_else(|| TextEvaluationError::Invalid("empty text path".into()))?;
    let mut points = crate::geometry::bezier::try_flatten_contour(contour, Tolerance(0.05), 65536)
        .map_err(|error| TextEvaluationError::Invalid(error.to_string()))?;
    if contour.closed && points.first() != points.last() {
        if let Some(first) = points.first().copied() {
            points.push(first);
        }
    }
    let mut segments = Vec::new();
    let mut total = 0.0;
    for pair in points.windows(2) {
        let length = (pair[1].x - pair[0].x).hypot(pair[1].y - pair[0].y);
        if length > 1e-12 {
            segments.push((total, length, pair[0], pair[1]));
            total += length;
        }
    }
    if segments.is_empty() {
        return Err(TextEvaluationError::Invalid("degenerate text path".into()));
    }
    let locate = |distance: f64| {
        let segment = segments
            .iter()
            .find(|(start, len, _, _)| distance < start + len)
            .unwrap_or(&segments[segments.len() - 1]);
        let (start, length, a, b) = *segment;
        let ux = (b.x - a.x) / length;
        let uy = (b.y - a.y) / length;
        (
            Point::new(a.x + ux * (distance - start), a.y + uy * (distance - start)),
            ux,
            uy,
        )
    };
    let mut artistic = object.clone();
    artistic.container = TextContainer::Artistic;
    let mut layout = evaluate_text(&artistic, styles, fonts, character, paragraph)?;
    let baseline = layout.lines.first().map_or(0.0, |line| line.baseline);
    let origin_x = paragraph.left_indent + paragraph.first_line_indent;
    let map_origin = |origin: Point| {
        let (point, ux, uy) = locate(origin.x - origin_x + reference.start_offset);
        (
            Point::new(
                point.x - uy * (origin.y - baseline),
                point.y + ux * (origin.y - baseline),
            ),
            ux,
            uy,
        )
    };
    for outline in &mut layout.outlines {
        let old = outline.origin;
        let (origin, ux, uy) = map_origin(old);
        let map_point = |point: Point| {
            Point::new(
                origin.x + ux * (point.x - old.x) - uy * (point.y - old.y),
                origin.y + uy * (point.x - old.x) + ux * (point.y - old.y),
            )
        };
        for contour in &mut outline.path.contours {
            for node in &mut contour.nodes {
                node.point = map_point(node.point);
                node.handle_in = node.handle_in.map(map_point);
                node.handle_out = node.handle_out.map(map_point);
            }
        }
        outline.origin = origin;
    }
    for glyph in &mut layout.glyphs {
        let distance = glyph.origin.x - origin_x + reference.start_offset;
        if distance < 0.0 || distance + glyph.advance > total {
            layout.overflow = true;
        }
        glyph.origin = map_origin(glyph.origin).0;
    }
    if layout.lines.len() > 1 {
        layout
            .warnings
            .push("additional paragraphs occupy parallel path baselines".into());
    }
    Ok(layout)
}
