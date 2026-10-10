//! Headless text evaluation. Unicode and authorial styles become positioned
//! glyphs and vector outlines without modifying the source or font references.
use super::{FontQuery, FontRegistry, HyphenationProvider, PatternHyphenation, ResolvedFace};
use petunia_core::{
    styles::{
        CharacterStyle, FontSlant, ParagraphStyle, StyleDefinition, StyleRegistry, TextAlignment,
    },
    text::{TextContainer, TextObject},
    ColorSource, Contour, NodeKind, PathNode, Point, SegmentKind, VectorPath,
};
use unicode_bidi::BidiInfo;
use unicode_script::{Script, UnicodeScript};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, thiserror::Error)]
pub enum TextEvaluationError {
    #[error("invalid authorial text/style: {0}")]
    Invalid(String),
    #[error("no usable font for {0}")]
    MissingFont(String),
    #[error("invalid font data")]
    InvalidFont,
    #[error("text-on-path requires a path layout provider")]
    MissingPathProvider,
}

/// A glyph and its source UTF-8 cluster. Coordinates are local document units.
#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    pub glyph_id: u16,
    pub source_byte: usize,
    pub origin: Point,
    pub advance: f64,
    pub family: String,
    pub synthetic: bool,
}
#[derive(Debug, Clone)]
pub struct GlyphOutline {
    pub source_byte: usize,
    pub origin: Point,
    pub path: VectorPath,
    pub color: ColorSource,
}
#[derive(Debug, Clone)]
pub struct LayoutLine {
    pub source: std::ops::Range<usize>,
    pub width: f64,
    pub baseline: f64,
}
#[derive(Debug, Clone, Default)]
pub struct TextLayout {
    pub glyphs: Vec<PositionedGlyph>,
    pub outlines: Vec<GlyphOutline>,
    pub lines: Vec<LayoutLine>,
    pub overflow: bool,
    pub warnings: Vec<String>,
}

struct FontSpan {
    range: std::ops::Range<usize>,
    face: ResolvedFace,
    style: CharacterStyle,
    script: Script,
}

/// Evaluate artistic or framed text. Caller supplies defaults because the Core
/// intentionally does not persist a synthetic default style on every object.
pub fn evaluate_text(
    object: &TextObject,
    styles: &StyleRegistry,
    fonts: &FontRegistry,
    default_character: &CharacterStyle,
    default_paragraph: &ParagraphStyle,
) -> Result<TextLayout, TextEvaluationError> {
    TextObject::validate(&object.text, &object.runs, &object.paragraphs)
        .map_err(|e| TextEvaluationError::Invalid(e.to_string()))?;
    default_character
        .validate()
        .map_err(|e| TextEvaluationError::Invalid(e.to_string()))?;
    default_paragraph
        .validate()
        .map_err(|e| TextEvaluationError::Invalid(e.to_string()))?;
    let (width, height) = match object.container {
        TextContainer::Artistic => (f64::INFINITY, f64::INFINITY),
        TextContainer::Frame(frame)
            if frame.size.width.is_finite()
                && frame.size.width > 0.0
                && frame.size.height.is_finite()
                && frame.size.height > 0.0 =>
        {
            (frame.size.width, frame.size.height)
        }
        TextContainer::Frame(_) => {
            return Err(TextEvaluationError::Invalid("frame dimensions".into()))
        }
        TextContainer::OnPath(_) => return Err(TextEvaluationError::MissingPathProvider),
    };
    if object.text.len() > 1 << 20 {
        return Err(TextEvaluationError::Invalid(
            "text layout exceeds 1 MiB guard".into(),
        ));
    }
    let mut face_cache = std::collections::HashMap::<(String, bool, String), ResolvedFace>::new();
    let mut result = TextLayout::default();
    let mut byte = 0;
    let mut baseline = 0.0;
    for paragraph in object.text.split_inclusive('\n') {
        let content = paragraph.trim_end_matches(['\n', '\r']);
        let paragraph_style = match object
            .paragraphs
            .iter()
            .find(|run| run.range.start as usize <= byte && byte < run.range.end as usize)
        {
            Some(run) => match styles.get(run.style.style) {
                Some(StyleDefinition::Paragraph(style)) => style,
                _ => {
                    return Err(TextEvaluationError::Invalid(
                        "missing paragraph style".into(),
                    ))
                }
            },
            None => default_paragraph,
        };
        paragraph_style
            .validate()
            .map_err(|e| TextEvaluationError::Invalid(e.to_string()))?;
        if paragraph_style.baseline_grid.is_some() {
            result
                .warnings
                .push("baseline grid requires document grid provider".into());
        }
        let mut spans: Vec<FontSpan> = Vec::new();
        let mut current_script = Script::Common;
        for (offset, cluster) in content.grapheme_indices(true) {
            if let Some(script) = cluster
                .chars()
                .map(|ch| ch.script())
                .find(|script| !matches!(script, Script::Common | Script::Inherited))
            {
                current_script = script;
            }
            let start = byte + offset;
            let style = match object
                .runs
                .iter()
                .find(|run| run.range.start as usize <= start && start < run.range.end as usize)
            {
                Some(run) => match styles.get(run.style.style) {
                    Some(StyleDefinition::Character(style)) => style,
                    _ => {
                        return Err(TextEvaluationError::Invalid(
                            "missing character style".into(),
                        ))
                    }
                },
                None => default_character,
            };
            style
                .validate()
                .map_err(|e| TextEvaluationError::Invalid(e.to_string()))?;
            // A style boundary inside a grapheme cannot be shaped as two independent characters.
            if object.runs.iter().any(|run| {
                [run.range.start as usize, run.range.end as usize]
                    .iter()
                    .any(|edge| *edge > start && *edge < start + cluster.len())
            }) {
                return Err(TextEvaluationError::Invalid(
                    "style splits grapheme cluster".into(),
                ));
            }
            let query = FontQuery {
                families: vec![style.font.family.clone()],
                weight: 400,
                italic: style.slant != FontSlant::Normal,
            };
            let key = (style.font.family.clone(), query.italic, cluster.to_string());
            let face = if let Some(face) = face_cache.get(&key) {
                face.clone()
            } else {
                let face = fonts
                    .resolve_cluster(&query, cluster)
                    .ok_or_else(|| TextEvaluationError::MissingFont(style.font.family.clone()))?;
                face_cache.insert(key, face.clone());
                face
            };
            if face.family != style.font.family
                && !result
                    .warnings
                    .iter()
                    .any(|warning| warning.contains(&style.font.family))
            {
                result.warnings.push(format!(
                    "font {} resolved with fallback {}",
                    style.font.family, face.family
                ));
            }
            if let Some(previous) = spans.last_mut().filter(|span| {
                span.face.family == face.family
                    && span.face.index == face.index
                    && span.style == *style
                    && span.script == current_script
            }) {
                previous.range.end += cluster.len();
            } else {
                spans.push(FontSpan {
                    range: offset..offset + cluster.len(),
                    face,
                    style: style.clone(),
                    script: current_script,
                });
            }
        }
        let bidi = BidiInfo::new(content, None);
        let mut advances = std::collections::BTreeMap::<usize, f64>::new();
        if let Some(para) = bidi.paragraphs.first() {
            for span in &spans {
                // Shape full connected spans, split at bidi level boundaries.
                for range in level_ranges(&bidi, span.range.clone()) {
                    for glyph in shape(
                        span,
                        content,
                        range.clone(),
                        bidi.levels[range.start].is_rtl(),
                    )? {
                        *advances.entry(range.start + glyph.0).or_default() += glyph.2;
                    }
                }
            }
            let max_width =
                (width - paragraph_style.left_indent - paragraph_style.right_indent).max(0.0);
            let mut hyphenation = std::collections::BTreeMap::new();
            if paragraph_style.hyphenation {
                for (start, word) in content.unicode_word_indices() {
                    if let Some(span) = spans
                        .iter()
                        .find(|span| span.range.start <= start && start < span.range.end)
                    {
                        let width = shape(span, "-", 0..1, false)?
                            .iter()
                            .map(|glyph| glyph.2)
                            .sum::<f64>();
                        for offset in PatternHyphenation.opportunities(word, &span.style.language) {
                            hyphenation.insert(start + offset, width);
                        }
                    }
                }
            }
            let lines = fit_lines(
                content,
                &advances,
                max_width,
                paragraph_style.first_line_indent,
                &hyphenation,
            );
            baseline += paragraph_style.space_before;
            for (line_index, (range, hyphenated)) in lines.into_iter().enumerate() {
                baseline += paragraph_style.line_height;
                let indent = if line_index == 0 {
                    paragraph_style.first_line_indent
                } else {
                    0.0
                };
                let line_width: f64 = advances
                    .range(range.clone())
                    .map(|(_, width)| width)
                    .sum::<f64>()
                    + if hyphenated {
                        hyphenation[&range.end]
                    } else {
                        0.0
                    };
                let available = (max_width - indent).max(0.0);
                if line_width > available || baseline > height {
                    result.overflow = true;
                }
                let alignment = match paragraph_style.alignment {
                    TextAlignment::Center => (available - line_width).max(0.0) / 2.0,
                    TextAlignment::Right => (available - line_width).max(0.0),
                    _ => 0.0,
                };
                let mut x = paragraph_style.left_indent + indent + alignment;
                let spaces = content[range.clone()]
                    .chars()
                    .filter(|ch| *ch == ' ')
                    .count();
                let justify = if paragraph_style.alignment == TextAlignment::Justify
                    && range.end < content.len()
                    && spaces > 0
                {
                    (available - line_width).max(0.0) / spaces as f64
                } else {
                    0.0
                };
                let (_, visual_runs) = bidi.visual_runs(para, range.clone());
                for visual in visual_runs {
                    let rtl = bidi.levels[visual.start].is_rtl();
                    let mut portions: Vec<_> = spans
                        .iter()
                        .filter_map(|span| {
                            let begin = span.range.start.max(visual.start);
                            let end = span.range.end.min(visual.end);
                            (begin < end).then_some((span, begin..end))
                        })
                        .collect();
                    if rtl {
                        portions.reverse();
                    }
                    for (span, portion) in portions {
                        let face = parse_face(span)?;
                        for (cluster, glyph_id, advance, dx, dy) in
                            shape(span, content, portion.clone(), rtl)?
                        {
                            let source = portion.start + cluster;
                            let origin =
                                Point::new(x + dx, baseline - dy - span.style.baseline_shift);
                            let mut outline = Outline::new(
                                origin,
                                span.style.size / f64::from(face.units_per_em()),
                            );
                            let has_outline = face
                                .outline_glyph(ttf_parser::GlyphId(glyph_id), &mut outline)
                                .is_some();
                            if has_outline {
                                result.outlines.push(GlyphOutline {
                                    source_byte: byte + source,
                                    origin,
                                    path: outline.finish(),
                                    color: span.style.color.clone(),
                                });
                            } else if !content[source..]
                                .chars()
                                .next()
                                .is_some_and(char::is_whitespace)
                            {
                                result
                                    .warnings
                                    .push(format!("glyph {glyph_id} has no vector outline"));
                            }
                            result.glyphs.push(PositionedGlyph {
                                glyph_id,
                                source_byte: byte + source,
                                origin,
                                advance,
                                family: span.face.family.clone(),
                                synthetic: false,
                            });
                            x += advance;
                            if content[source..].starts_with(' ') {
                                x += justify;
                            }
                        }
                    }
                }
                if hyphenated {
                    if let Some(span) = spans
                        .iter()
                        .find(|span| span.range.start < range.end && range.end <= span.range.end)
                    {
                        let face = parse_face(span)?;
                        for (_, glyph_id, advance, dx, dy) in shape(span, "-", 0..1, false)? {
                            let origin =
                                Point::new(x + dx, baseline - dy - span.style.baseline_shift);
                            let mut outline = Outline::new(
                                origin,
                                span.style.size / f64::from(face.units_per_em()),
                            );
                            if face
                                .outline_glyph(ttf_parser::GlyphId(glyph_id), &mut outline)
                                .is_some()
                            {
                                result.outlines.push(GlyphOutline {
                                    source_byte: byte + range.end,
                                    origin,
                                    path: outline.finish(),
                                    color: span.style.color.clone(),
                                });
                            }
                            result.glyphs.push(PositionedGlyph {
                                glyph_id,
                                source_byte: byte + range.end,
                                origin,
                                advance,
                                family: span.face.family.clone(),
                                synthetic: true,
                            });
                            x += advance;
                        }
                    }
                }
                result.lines.push(LayoutLine {
                    source: byte + range.start..byte + range.end,
                    width: line_width,
                    baseline,
                });
            }
        } else {
            baseline += paragraph_style.line_height;
            result.lines.push(LayoutLine {
                source: byte..byte,
                width: 0.0,
                baseline,
            });
        }
        baseline += paragraph_style.space_after;
        byte += paragraph.len();
    }
    Ok(result)
}

fn level_ranges(bidi: &BidiInfo<'_>, range: std::ops::Range<usize>) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut begin = range.start;
    for offset in range.start + 1..range.end {
        if bidi.levels[offset] != bidi.levels[begin] {
            ranges.push(begin..offset);
            begin = offset;
        }
    }
    ranges.push(begin..range.end);
    ranges
}

fn parse_face(span: &FontSpan) -> Result<ttf_parser::Face<'_>, TextEvaluationError> {
    let mut face = ttf_parser::Face::parse(&span.face.bytes, span.face.index)
        .map_err(|_| TextEvaluationError::InvalidFont)?;
    for axis in &span.style.font.axes {
        if axis.tag.len() != 4 {
            return Err(TextEvaluationError::Invalid(
                "font axis tag must have four bytes".into(),
            ));
        }
        let bytes: [u8; 4] = axis
            .tag
            .as_bytes()
            .try_into()
            .map_err(|_| TextEvaluationError::InvalidFont)?;
        face.set_variation(ttf_parser::Tag::from_bytes(&bytes), axis.value);
    }
    Ok(face)
}
type GlyphData = (usize, u16, f64, f64, f64);
fn shape(
    span: &FontSpan,
    content: &str,
    range: std::ops::Range<usize>,
    rtl: bool,
) -> Result<Vec<GlyphData>, TextEvaluationError> {
    let mut face = rustybuzz::Face::from_slice(&span.face.bytes, span.face.index)
        .ok_or(TextEvaluationError::InvalidFont)?;
    let variations: Vec<_> = span
        .style
        .font
        .axes
        .iter()
        .filter_map(|axis| {
            axis.tag
                .as_bytes()
                .try_into()
                .ok()
                .map(|tag| rustybuzz::Variation {
                    tag: ttf_parser::Tag::from_bytes(tag),
                    value: axis.value,
                })
        })
        .collect();
    face.set_variations(&variations);
    let mut buffer = rustybuzz::UnicodeBuffer::new();
    buffer.push_str(&content[range]);
    buffer.set_direction(if rtl {
        rustybuzz::Direction::RightToLeft
    } else {
        rustybuzz::Direction::LeftToRight
    });
    if let Ok(language) = span.style.language.parse() {
        buffer.set_language(language);
    }
    buffer.guess_segment_properties();
    let features: Result<Vec<rustybuzz::Feature>, _> = span
        .style
        .features
        .iter()
        .map(|feature| feature.parse())
        .collect();
    let features =
        features.map_err(|_| TextEvaluationError::Invalid("invalid OpenType feature".into()))?;
    let output = rustybuzz::shape(&face, &features, buffer);
    let scale = span.style.size / f64::from(face.units_per_em());
    Ok(output
        .glyph_infos()
        .iter()
        .zip(output.glyph_positions())
        .map(|(info, pos)| {
            (
                info.cluster as usize,
                info.glyph_id as u16,
                f64::from(pos.x_advance) * scale + span.style.tracking,
                f64::from(pos.x_offset) * scale,
                f64::from(pos.y_offset) * scale,
            )
        })
        .collect())
}
fn fit_lines(
    text: &str,
    advances: &std::collections::BTreeMap<usize, f64>,
    width: f64,
    indent: f64,
    hyphenation: &std::collections::BTreeMap<usize, f64>,
) -> Vec<(std::ops::Range<usize>, bool)> {
    let opportunities: std::collections::HashSet<_> = unicode_linebreak::linebreaks(text)
        .map(|(offset, _)| offset)
        .collect();
    let clusters: Vec<_> = advances
        .keys()
        .copied()
        .chain(std::iter::once(text.len()))
        .collect();
    let mut lines = Vec::new();
    let mut begin = 0;
    let mut index = 0;
    while index + 1 < clusters.len() {
        let limit = width - if lines.is_empty() { indent } else { 0.0 };
        let mut total = 0.0;
        let mut last_break = None;
        let mut last_hyphen = None;
        let start_index = index;
        while index + 1 < clusters.len() {
            let amount = advances[&clusters[index]];
            if total + amount > limit && index > start_index {
                break;
            }
            total += amount;
            index += 1;
            if opportunities.contains(&clusters[index]) {
                last_break = Some(index);
            }
            if hyphenation
                .get(&clusters[index])
                .is_some_and(|hyphen| total + hyphen <= limit)
            {
                last_hyphen = Some(index);
            }
        }
        let mut hyphenated = false;
        if index + 1 < clusters.len() {
            if let Some(end) = last_break.filter(|end| *end > start_index) {
                index = end;
            } else if let Some(end) = last_hyphen.filter(|end| *end > start_index) {
                index = end;
                hyphenated = true;
            }
        }
        let end = clusters[index];
        lines.push((begin..end, hyphenated));
        begin = end;
    }
    if lines.is_empty() {
        lines.push((0..text.len(), false));
    }
    lines
}

struct Outline {
    path: VectorPath,
    contour: Option<Contour>,
    origin: Point,
    scale: f64,
}
impl Outline {
    fn new(origin: Point, scale: f64) -> Self {
        Self {
            path: VectorPath::new(),
            contour: None,
            origin,
            scale,
        }
    }
    fn point(&self, x: f32, y: f32) -> Point {
        Point::new(
            self.origin.x + f64::from(x) * self.scale,
            self.origin.y - f64::from(y) * self.scale,
        )
    }
    fn finish(mut self) -> VectorPath {
        if let Some(contour) = self.contour.take() {
            self.path.push_contour(contour);
        }
        self.path
    }
}
impl ttf_parser::OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        if let Some(contour) = self.contour.take() {
            self.path.push_contour(contour);
        }
        let mut contour = Contour::new(false);
        contour.push_node(PathNode::line(self.point(x, y), NodeKind::Cusp));
        self.contour = Some(contour);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let point = self.point(x, y);
        if let Some(contour) = &mut self.contour {
            contour.push_node(PathNode::line(point, NodeKind::Cusp));
        }
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let control = self.point(x1, y1);
        let end = self.point(x, y);
        if let Some(contour) = &mut self.contour {
            if let Some(previous) = contour.nodes.last_mut() {
                let start = previous.point;
                previous.outgoing = SegmentKind::Cubic;
                previous.handle_out = Some(Point::new(
                    start.x + (control.x - start.x) * 2.0 / 3.0,
                    start.y + (control.y - start.y) * 2.0 / 3.0,
                ));
                let incoming = Point::new(
                    end.x + (control.x - end.x) * 2.0 / 3.0,
                    end.y + (control.y - end.y) * 2.0 / 3.0,
                );
                contour.push_node(PathNode::with_handles(
                    end,
                    Some(incoming),
                    None,
                    NodeKind::Cusp,
                ));
            }
        }
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let out = self.point(x1, y1);
        let incoming = self.point(x2, y2);
        let end = self.point(x, y);
        if let Some(contour) = &mut self.contour {
            if let Some(previous) = contour.nodes.last_mut() {
                previous.outgoing = SegmentKind::Cubic;
                previous.handle_out = Some(out);
            }
            contour.push_node(PathNode::with_handles(
                end,
                Some(incoming),
                None,
                NodeKind::Cusp,
            ));
        }
    }
    fn close(&mut self) {
        if let Some(mut contour) = self.contour.take() {
            if contour.nodes.len() > 1
                && contour.nodes.first().map(|n| n.point) == contour.nodes.last().map(|n| n.point)
            {
                if let Some(last) = contour.nodes.pop() {
                    if let Some(first) = contour.nodes.first_mut() {
                        first.handle_in = last.handle_in;
                    }
                }
            }
            contour.closed = true;
            self.path.push_contour(contour);
        }
    }
}
