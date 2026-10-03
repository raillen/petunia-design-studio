//! Advanced shaped glyph outlines for GUI-free scene rendering. Originals stay
//! text; these bounded, immutable paths are rebuildable display resources.
use crate::TypeSystem;
use cosmic_text::{Align, Attrs, Buffer, Family, Hinting, Metrics, Shaping, Style, Weight, Wrap};
use petunia_design_geometry::{GAffine, GPath, GPoint, GRect, PathVerb};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex, OnceLock,
};

/// Direction-relative alignment for a uniform paragraph.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FlowAlignment {
    #[default]
    Start,
    Center,
    End,
}
#[derive(Clone, Debug, PartialEq)]
/// Uniform-style frame input in document points, independent of GUI types.
pub struct TextFrameSpec {
    /// Original UTF-8 content.
    pub content: String,
    /// Requested family name or CSS generic family.
    pub family: String,
    /// Positive font em size in points.
    pub font_size: f64,
    /// Line height as a multiplier of font size, matching the document field.
    pub line_height: f64,
    /// Tracking in document points; converted to EM at the shaping boundary.
    pub letter_spacing: f64,
    /// Numeric font weight, in 1..=1000.
    pub weight: u16,
    pub italic: bool,
    pub alignment: FlowAlignment,
    /// False for artistic text, preserving explicit paragraph breaks only.
    pub wrap: bool,
    /// Positive wrap width; height does not implicitly clip ink.
    pub width: f64,
}
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
/// Recoverable admission, capability and cancellation failures.
pub enum TextRenderError {
    #[error("invalid text render input: {0}")]
    Invalid(&'static str),
    #[error("text render resource limit: {0}")]
    Limit(&'static str),
    #[error("unavailable text capability: {0}")]
    Unsupported(&'static str),
    #[error("text preparation cancelled")]
    Cancelled,
}
#[derive(Clone, Debug, PartialEq)]
/// Shaped logical glyph metadata; font database handles are not persisted.
pub struct PositionedGlyph {
    pub glyph_id: u16,
    pub source_range: [usize; 2],
    pub origin: GPoint,
    pub advance: f32,
    pub font_family: String,
    /// Index into the immutable font resources of this prepared text.
    pub font_resource: usize,
    pub rtl: bool,
    pub bounds: [f64; 4],
}
#[derive(Debug)]
pub struct PreparedFont {
    pub bytes: Arc<Vec<u8>>,
    pub face_index: u32,
    pub subset_embedding_allowed: bool,
}
#[derive(Debug)]
struct Residency {
    bytes: usize,
    account: Arc<AtomicUsize>,
}
impl Drop for Residency {
    fn drop(&mut self) {
        self.account.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}
#[derive(Debug)]
/// Shared immutable outline and glyph records with charged cache residency.
pub struct PreparedText {
    outline: GPath,
    glyphs: Vec<PositionedGlyph>,
    fonts: Vec<PreparedFont>,
    line_count: usize,
    height: f64,
    missing_family: bool,
    lines: Vec<PreparedLine>,
    boundaries: Vec<usize>,
    _residency: Residency,
}
#[derive(Clone, Debug)]
struct PreparedLine {
    range: [usize; 2],
    top: f64,
    height: f64,
}
impl PreparedText {
    /// Borrowed ink; scenes retain this owner rather than escaping an uncharged path.
    pub fn outline(&self) -> &GPath {
        &self.outline
    }
    /// Visual-order glyphs with source UTF-8 clusters.
    pub fn glyphs(&self) -> &[PositionedGlyph] {
        &self.glyphs
    }
    /// Exact font faces used by shaping, captured on the worker for export.
    pub fn fonts(&self) -> &[PreparedFont] {
        &self.fonts
    }
    /// Number of laid out lines, including wrapped lines.
    pub fn line_count(&self) -> usize {
        self.line_count
    }
    /// Logical laid-out height in points.
    pub fn flow_height(&self) -> f64 {
        self.height
    }
    pub fn missing_family(&self) -> bool {
        self.missing_family
    }
    /// Logical cluster rectangles, in the exact shaped visual positions. This
    /// includes spaces without pretending their outlines contain visible ink.
    pub fn logical_bounds(&self) -> Option<GRect> {
        self.glyphs.iter().fold(None, |bounds, glyph| {
            let [x, y, w, h] = glyph.bounds;
            let next = GRect::new(x, y, x + w, y + h);
            bounds.map_or(Some(next), |prior: GRect| prior.union(next))
        })
    }
    pub fn hit_test(&self, point: GPoint) -> bool {
        self.glyphs.iter().any(|g| {
            let [x, y, w, h] = g.bounds;
            point.x >= x && point.x <= x + w && point.y >= y && point.y <= y + h
        })
    }
    /// Actual outline footprint, absent for whitespace.
    pub fn ink_bounds(&self) -> Option<GRect> {
        self.outline.bounding_box()
    }
    /// Caret geometry in the same local frame as the artwork outlines. Logical
    /// graphemes within a ligature divide its advance; bidi reverses the edges.
    pub fn caret_rect(&self, offset: usize) -> GRect {
        let offset = self
            .boundaries
            .iter()
            .copied()
            .take_while(|&b| b <= offset)
            .last()
            .unwrap_or(0);
        let glyph = self
            .glyphs
            .iter()
            .find(|g| g.source_range[0] <= offset && offset < g.source_range[1])
            .or_else(|| {
                self.glyphs
                    .iter()
                    .rev()
                    .find(|g| g.source_range[1] == offset)
            });
        if let Some(glyph) = glyph {
            let blank = self
                .lines
                .iter()
                .find(|line| line.range == [offset, offset]);
            if let Some(line) = blank {
                return GRect::new(0., line.top, 0., line.top + line.height);
            }
            let x = self.cluster_x(glyph, offset);
            return GRect::new(x, glyph.bounds[1], x, glyph.bounds[1] + glyph.bounds[3]);
        }
        let line = self
            .lines
            .iter()
            .find(|line| line.range[0] <= offset && offset <= line.range[1])
            .or_else(|| self.lines.last());
        line.map_or(GRect::new(0., 0., 0., self.height.max(1.)), |line| {
            GRect::new(0., line.top, 0., line.top + line.height)
        })
    }
    /// Visual hit → logical UTF-8 grapheme boundary, including blank paragraphs.
    pub fn offset_at_point(&self, point: GPoint) -> usize {
        let Some(line) = self.lines.iter().min_by(|a, b| {
            (point.y - (a.top + a.height / 2.))
                .abs()
                .total_cmp(&(point.y - (b.top + b.height / 2.)).abs())
        }) else {
            return 0;
        };
        let mut best = (f64::INFINITY, line.range[0]);
        for glyph in self
            .glyphs
            .iter()
            .filter(|g| (g.bounds[1] - line.top).abs() < 0.01)
        {
            for &offset in self.cluster_boundaries(glyph) {
                let distance = (point.x - self.cluster_x(glyph, offset)).abs();
                if distance < best.0 {
                    best = (distance, offset);
                }
            }
        }
        best.1
    }
    /// Rectangles are in visual order; a bidi selection can be disjoint.
    pub fn selection_rects(&self, range: std::ops::Range<usize>) -> Vec<GRect> {
        if range.is_empty() {
            return Vec::new();
        }
        let mut result: Vec<GRect> = Vec::new();
        for glyph in &self.glyphs {
            let start = range.start.max(glyph.source_range[0]);
            let end = range.end.min(glyph.source_range[1]);
            if start >= end {
                continue;
            }
            let a = self.cluster_x(glyph, start);
            let b = self.cluster_x(glyph, end);
            let rect = GRect::new(
                a.min(b),
                glyph.bounds[1],
                a.max(b),
                glyph.bounds[1] + glyph.bounds[3],
            );
            if let Some(previous) = result.last_mut().filter(|p| {
                (p.y0 - rect.y0).abs() < 0.01 && rect.x0 <= p.x1 + 0.5 && rect.x1 >= p.x0 - 0.5
            }) {
                *previous = previous.union(rect).unwrap_or(rect);
            } else {
                result.push(rect);
            }
        }
        result
    }
    fn cluster_x(&self, glyph: &PositionedGlyph, offset: usize) -> f64 {
        let stops = self.cluster_boundaries(glyph);
        let count = stops.len().saturating_sub(1).max(1);
        let position = stops.partition_point(|&b| b < offset).min(count);
        let fraction = position as f64 / count as f64;
        glyph.bounds[0] + glyph.bounds[2] * if glyph.rtl { 1. - fraction } else { fraction }
    }
    fn cluster_boundaries(&self, glyph: &PositionedGlyph) -> &[usize] {
        let start = self
            .boundaries
            .partition_point(|&b| b < glyph.source_range[0]);
        let end = self
            .boundaries
            .partition_point(|&b| b <= glyph.source_range[1]);
        &self.boundaries[start..end]
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Key {
    content: String,
    family: String,
    metrics: [u64; 4],
    style: (u16, bool, FlowAlignment, bool),
}
impl From<&TextFrameSpec> for Key {
    fn from(s: &TextFrameSpec) -> Self {
        Self {
            content: s.content.clone(),
            family: s.family.clone(),
            style: (s.weight, s.italic, s.alignment, s.wrap),
            metrics: [
                s.font_size.to_bits(),
                s.line_height.to_bits(),
                s.letter_spacing.to_bits(),
                s.width.to_bits(),
            ],
        }
    }
}
struct Entry {
    image: Arc<PreparedText>,
    touched: u64,
}
#[derive(Default)]
struct State {
    entries: HashMap<Key, Entry>,
    clock: u64,
}
struct Cache {
    state: Mutex<State>,
    cold: Mutex<()>,
    live: Arc<AtomicUsize>,
}
fn cache() -> &'static Cache {
    static CACHE: OnceLock<Cache> = OnceLock::new();
    CACHE.get_or_init(|| Cache {
        state: Mutex::new(State::default()),
        cold: Mutex::new(()),
        live: Arc::new(AtomicUsize::new(0)),
    })
}
const MAX_BYTES: usize = 64 * 1024 * 1024;
const MAX_ENTRIES: usize = 128;
const MAX_TEXT: usize = 64 * 1024;
const MAX_GLYPHS: usize = 65_536;
const MAX_VERBS: usize = 1_000_000;
fn hit(cache: &Cache, key: &Key) -> Option<Arc<PreparedText>> {
    let mut state = cache.state.lock().unwrap_or_else(|e| e.into_inner());
    state.clock = state.clock.saturating_add(1);
    let clock = state.clock;
    state.entries.get_mut(key).map(|entry| {
        entry.touched = clock;
        entry.image.clone()
    })
}
fn evict(state: &mut State) -> bool {
    let key = state
        .entries
        .iter()
        .filter(|(_, entry)| Arc::strong_count(&entry.image) == 1)
        .min_by_key(|(_, entry)| entry.touched)
        .map(|(key, _)| key.clone());
    if let Some(key) = key {
        state.entries.remove(&key);
        true
    } else {
        false
    }
}
/// System-font discovery/shaping is serialized behind the existing process
/// font system. Workers invoke this; cache hits share paths and glyph records.
/// Font database/private shaping allocations are not a whole-process quota.
pub fn prepare_text(
    spec: &TextFrameSpec,
    cancelled: &dyn Fn() -> bool,
) -> Result<Arc<PreparedText>, TextRenderError> {
    if cancelled() {
        return Err(TextRenderError::Cancelled);
    }
    if spec.content.len() > MAX_TEXT || spec.family.len() > 1024 {
        return Err(TextRenderError::Limit("text/family bytes"));
    }
    let values = [
        spec.font_size,
        spec.line_height,
        spec.width,
        spec.font_size * spec.line_height,
    ];
    if !(1..=1000).contains(&spec.weight)
        || !values
            .iter()
            .all(|v| v.is_finite() && *v > 0.0 && *v <= 1_000_000.0)
        || !spec.letter_spacing.is_finite()
        || spec.letter_spacing.abs() > 1_000_000.0
    {
        return Err(TextRenderError::Invalid(
            "finite positive frame/font metrics required",
        ));
    }
    let cache = cache();
    let key = Key::from(spec);
    if let Some(hit) = hit(cache, &key) {
        return Ok(hit);
    }
    let _cold = cache.cold.lock().unwrap_or_else(|e| e.into_inner());
    if cancelled() {
        return Err(TextRenderError::Cancelled);
    }
    if let Some(hit) = hit(cache, &key) {
        return Ok(hit);
    }
    let (outline, glyphs, lines, height, missing_family, line_records, fonts) =
        shape_outlines(spec, cancelled)?;
    let boundaries: Vec<_> = crate::editing::boundaries(&spec.content).collect();
    if cancelled() {
        return Err(TextRenderError::Cancelled);
    }
    let bytes = outline
        .verbs
        .capacity()
        .checked_mul(std::mem::size_of::<PathVerb>())
        .and_then(|n| n.checked_add(glyphs.capacity() * std::mem::size_of::<PositionedGlyph>()))
        .and_then(|n| {
            n.checked_add(
                glyphs
                    .iter()
                    .map(|g| g.font_family.capacity())
                    .sum::<usize>(),
            )
        })
        .and_then(|n| {
            n.checked_add(
                key.content.capacity()
                    + key.family.capacity()
                    + boundaries.capacity() * std::mem::size_of::<usize>()
                    + line_records.capacity() * std::mem::size_of::<PreparedLine>()
                    + fonts.iter().map(|f| f.bytes.capacity()).sum::<usize>(),
            )
        })
        .ok_or(TextRenderError::Limit("prepared text bytes"))?;
    if bytes > MAX_BYTES {
        return Err(TextRenderError::Limit("prepared text bytes"));
    }
    let mut state = cache.state.lock().unwrap_or_else(|e| e.into_inner());
    while state.entries.len() >= MAX_ENTRIES
        || cache
            .live
            .load(Ordering::Acquire)
            .checked_add(bytes)
            .is_none_or(|n| n > MAX_BYTES)
    {
        if !evict(&mut state) {
            return Err(TextRenderError::Limit("prepared text pinned by scenes"));
        }
    }
    cache.live.fetch_add(bytes, Ordering::AcqRel);
    let text = Arc::new(PreparedText {
        outline,
        glyphs,
        fonts,
        line_count: lines,
        height,
        missing_family,
        lines: line_records,
        boundaries,
        _residency: Residency {
            bytes,
            account: cache.live.clone(),
        },
    });
    state.clock = state.clock.saturating_add(1);
    let touched = state.clock;
    state.entries.insert(
        key,
        Entry {
            image: text.clone(),
            touched,
        },
    );
    Ok(text)
}

type ShapedOutlines = (
    GPath,
    Vec<PositionedGlyph>,
    usize,
    f64,
    bool,
    Vec<PreparedLine>,
    Vec<PreparedFont>,
);
fn shape_outlines(
    spec: &TextFrameSpec,
    cancelled: &dyn Fn() -> bool,
) -> Result<ShapedOutlines, TextRenderError> {
    let mut system = TypeSystem::lock();
    if cancelled() {
        return Err(TextRenderError::Cancelled);
    }
    let family = match spec.family.as_str() {
        "sans-serif" => Family::SansSerif,
        "serif" => Family::Serif,
        "monospace" => Family::Monospace,
        _ => Family::Name(&spec.family),
    };
    let missing_family = matches!(family, Family::Name(_))
        && !system.db().faces().any(|face| {
            face.families
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case(&spec.family))
        });
    let attrs = Attrs::new()
        .family(family)
        .weight(Weight(spec.weight))
        .style(if spec.italic {
            Style::Italic
        } else {
            Style::Normal
        })
        .letter_spacing((spec.letter_spacing / spec.font_size) as f32);
    let mut buffer = Buffer::new(
        &mut system,
        Metrics::new(
            spec.font_size as f32,
            (spec.font_size * spec.line_height) as f32,
        ),
    );
    buffer.set_size(Some(spec.width as f32), None);
    buffer.set_wrap(if spec.wrap {
        Wrap::WordOrGlyph
    } else {
        Wrap::None
    });
    buffer.set_hinting(Hinting::Disabled);
    let alignment = match spec.alignment {
        FlowAlignment::Start => None,
        FlowAlignment::Center => Some(Align::Center),
        FlowAlignment::End => Some(Align::End),
    };
    buffer.set_text(&spec.content, &attrs, Shaping::Advanced, alignment);
    buffer.shape_until_scroll(&mut system, false);
    let mut combined = GPath::new();
    let mut glyphs = Vec::new();
    let mut fonts = Vec::new();
    let mut font_indices = HashMap::new();
    let mut font_bytes = 0usize;
    let mut lines = 0;
    let mut height = 0.0_f64;
    let mut line_records = Vec::new();
    let mut line_offsets: Vec<_> = cosmic_text::LineIter::new(&spec.content)
        .map(|(range, _)| range.start)
        .collect();
    // Buffer keeps a final empty paragraph, including for an empty source.
    line_offsets.push(spec.content.len());
    // Deduplicate font parsing/outline extraction within this prepared flow.
    let mut units: HashMap<(fontdb::ID, u16), (Arc<GPath>, f64, String)> = HashMap::new();
    let mut unit_verbs = 0usize;
    for run in buffer.layout_runs() {
        if cancelled() {
            return Err(TextRenderError::Cancelled);
        }
        lines += 1;
        height = height.max(f64::from(run.line_top + run.line_height));
        let line_offset = *line_offsets
            .get(run.line_i)
            .ok_or(TextRenderError::Invalid("invalid source paragraph"))?;
        line_records.push(PreparedLine {
            range: [
                line_offset + run.glyphs.iter().map(|g| g.start).min().unwrap_or(0),
                line_offset + run.glyphs.iter().map(|g| g.end).max().unwrap_or(0),
            ],
            top: f64::from(run.line_top),
            height: f64::from(run.line_height),
        });
        for glyph in run.glyphs {
            if cancelled() {
                return Err(TextRenderError::Cancelled);
            }
            if glyphs.len() >= MAX_GLYPHS {
                return Err(TextRenderError::Limit("glyph count"));
            }
            if let std::collections::hash_map::Entry::Vacant(e) = font_indices.entry(glyph.font_id)
            {
                if fonts.len() >= 64 {
                    return Err(TextRenderError::Limit("text font resources"));
                }
                let font = system
                    .db()
                    .with_face_data(glyph.font_id, |bytes, face_index| {
                        if bytes.len() > 16 * 1024 * 1024 {
                            return Err(TextRenderError::Limit("font file bytes"));
                        }
                        let face = ttf_parser::Face::parse(bytes, face_index)
                            .map_err(|_| TextRenderError::Invalid("invalid export font"))?;
                        let allowed = matches!(
                            face.permissions(),
                            Some(
                                ttf_parser::Permissions::Installable
                                    | ttf_parser::Permissions::Editable
                                    | ttf_parser::Permissions::PreviewAndPrint
                            )
                        ) && face.is_subsetting_allowed()
                            && face.is_outline_embedding_allowed();
                        Ok(PreparedFont {
                            bytes: Arc::new(bytes.to_vec()),
                            face_index,
                            subset_embedding_allowed: allowed,
                        })
                    })
                    .ok_or(TextRenderError::Unsupported("font resource unavailable"))??;
                font_bytes += font.bytes.len();
                if font_bytes > 32 * 1024 * 1024 {
                    return Err(TextRenderError::Limit("aggregate font bytes"));
                }
                e.insert(fonts.len());
                fonts.push(font);
            }
            let cluster = run
                .text
                .get(glyph.start..glyph.end)
                .ok_or(TextRenderError::Invalid("invalid shaped UTF-8 cluster"))?;
            let invisible = cluster.chars().all(|c| {
                c.is_whitespace()
                    || matches!(c, '\u{00ad}' | '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{fe00}'..='\u{fe0f}')
            });
            if glyph.glyph_id == 0 && !invisible {
                return Err(TextRenderError::Unsupported(
                    "font fallback has uncovered glyphs",
                ));
            }
            let key = (glyph.font_id, glyph.glyph_id);
            if let std::collections::hash_map::Entry::Vacant(entry) = units.entry(key) {
                let value = system
                    .db()
                    .with_face_data(glyph.font_id, |data, index| {
                        if data.len() > 32 * 1024 * 1024 {
                            return Err(TextRenderError::Limit("individual system font bytes"));
                        }
                        let face = ttf_parser::Face::parse(data, index)
                            .map_err(|_| TextRenderError::Invalid("invalid system font"))?;
                        if invisible {
                            return Ok((GPath::new(), f64::from(face.units_per_em())));
                        }
                        if face.is_variable() {
                            return Err(TextRenderError::Unsupported(
                                "variable-font outline coordinates",
                            ));
                        }
                        let id = ttf_parser::GlyphId(glyph.glyph_id);
                        if face.is_color_glyph(id)
                            || face.glyph_svg_image(id).is_some()
                            || face.glyph_raster_image(id, u16::MAX).is_some()
                        {
                            return Err(TextRenderError::Unsupported(
                                "color/bitmap/SVG glyph rendering",
                            ));
                        }
                        let mut builder = Outline {
                            path: GPath::new(),
                            failed: false,
                        };
                        let found = face.outline_glyph(id, &mut builder);
                        if builder.failed {
                            return Err(TextRenderError::Limit("glyph outline verbs/allocation"));
                        }
                        if found.is_none() {
                            return Err(TextRenderError::Unsupported("font glyph has no outline"));
                        }
                        // TTF and CFF outer contours may have opposite signs.
                        // Normalize whole-glyph orientation before concatenating;
                        // counter relationships and Bézier curves stay intact.
                        if builder.path.signed_area() < 0.0 {
                            builder.path = builder.path.reversed_contours();
                        }
                        Ok((builder.path, f64::from(face.units_per_em())))
                    })
                    .ok_or(TextRenderError::Unsupported("font face unavailable"))??;
                unit_verbs = unit_verbs
                    .checked_add(value.0.verbs.len())
                    .ok_or(TextRenderError::Limit("glyph outline scratch"))?;
                if unit_verbs > MAX_VERBS {
                    return Err(TextRenderError::Limit("glyph outline scratch"));
                }
                let family = system
                    .db()
                    .face(glyph.font_id)
                    .and_then(|face| face.families.first())
                    .map_or_else(String::new, |(name, _)| name.clone());
                if family.len() > 1024 {
                    return Err(TextRenderError::Limit("resolved font family bytes"));
                }
                entry.insert((Arc::new(value.0), value.1, family));
            }
            let (path, upem, family) = &units[&key];
            if combined
                .verbs
                .len()
                .checked_add(path.verbs.len())
                .is_none_or(|n| n > MAX_VERBS)
            {
                return Err(TextRenderError::Limit("text outline verbs"));
            }
            combined
                .verbs
                .try_reserve(path.verbs.len())
                .map_err(|_| TextRenderError::Limit("text outline allocation"))?;
            let origin = GPoint::new(
                f64::from(glyph.x + glyph.font_size * glyph.x_offset),
                f64::from(run.line_y + glyph.y - glyph.font_size * glyph.y_offset),
            );
            let scale = f64::from(glyph.font_size) / upem;
            let placed = path.transformed(
                GAffine::translate(origin.x, origin.y).after(GAffine::scale(scale, -scale)),
            );
            if !placed.is_finite() {
                return Err(TextRenderError::Invalid("nonfinite glyph placement"));
            }
            combined.verbs.extend(placed.verbs);
            glyphs
                .try_reserve(1)
                .map_err(|_| TextRenderError::Limit("glyph record allocation"))?;
            let line_offset = *line_offsets
                .get(run.line_i)
                .ok_or(TextRenderError::Invalid("invalid source paragraph"))?;
            glyphs.push(PositionedGlyph {
                glyph_id: glyph.glyph_id,
                source_range: [line_offset + glyph.start, line_offset + glyph.end],
                origin,
                advance: glyph.w,
                font_family: family.clone(),
                font_resource: font_indices[&glyph.font_id],
                rtl: glyph.level.is_rtl(),
                bounds: [
                    f64::from(glyph.x),
                    f64::from(run.line_top),
                    f64::from(glyph.w),
                    f64::from(run.line_height),
                ],
            });
        }
    }
    Ok((
        combined,
        glyphs,
        lines,
        height,
        missing_family,
        line_records,
        fonts,
    ))
}
struct Outline {
    path: GPath,
    failed: bool,
}
impl Outline {
    fn push(&mut self, verb: PathVerb) {
        if self.failed {
            return;
        }
        if self.path.verbs.len() >= MAX_VERBS
            || self.path.verbs.try_reserve(1).is_err()
            || self.path.push(verb).is_err()
        {
            self.failed = true;
        }
    }
}
impl ttf_parser::OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.push(PathVerb::MoveTo(GPoint::new(f64::from(x), f64::from(y))));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.push(PathVerb::LineTo(GPoint::new(f64::from(x), f64::from(y))));
    }
    fn quad_to(&mut self, x: f32, y: f32, px: f32, py: f32) {
        self.push(PathVerb::QuadTo(
            GPoint::new(f64::from(x), f64::from(y)),
            GPoint::new(f64::from(px), f64::from(py)),
        ));
    }
    fn curve_to(&mut self, x: f32, y: f32, x2: f32, y2: f32, px: f32, py: f32) {
        self.push(PathVerb::CubicTo(
            GPoint::new(f64::from(x), f64::from(y)),
            GPoint::new(f64::from(x2), f64::from(y2)),
            GPoint::new(f64::from(px), f64::from(py)),
        ));
    }
    fn close(&mut self) {
        self.push(PathVerb::Close);
    }
}
