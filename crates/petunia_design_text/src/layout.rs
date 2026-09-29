//! Typography layout facade, line breaking and hit testing (09.8, F7.2).
//!
//! Line breaking keeps the historical greedy semantics (wrap before the
//! character that overflows `max_width`, hard break on `\n`) but sums **real
//! shaped advances** ([`TypeSystem::shape_run_basic`]) instead of the former
//! `0.55 * size` estimate, and derives baselines from real font
//! ascent/descent. Alignment behavior is unchanged (Left/Center/Right shifts
//! only); real alignment is a separate concern.
//!
//! Layouts are cached per (content + run attributes + sizes + max width) in
//! [`TextLayout::layout_cached`], so repeated layouts of an unchanged story
//! do not re-shape. [`TextLayout::layout`] redirects there.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, OnceLock};

use petunia_design_geometry::{GPoint, GRect};

use crate::fonts::TypeSystem;
use crate::story::{TextAlign, TextOffset, TextStory};

/// A derived visual line fragment resulting from layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LineFragment {
    /// Byte range in the source story content.
    pub range: (TextOffset, TextOffset),
    /// Baseline offset from top of layout.
    pub baseline_y: f64,
    /// Bounding rectangle for this line.
    pub bounds: GRect,
    /// Line text content.
    pub text: String,
}

/// The result of running layout on a story.
#[derive(Debug, Clone, PartialEq)]
pub struct TextLayout {
    pub lines: Vec<LineFragment>,
    pub bounds: GRect,
    pub line_count: usize,
}

impl TextLayout {
    /// Performs line breaking and layout for a story within an optional max width.
    ///
    /// Redirects to [`TextLayout::layout_cached`]: same result, served from
    /// the layout cache when the story and width were already laid out.
    #[must_use]
    pub fn layout(story: &TextStory, max_width: Option<f64>) -> Self {
        Self::layout_cached(story, max_width)
    }

    /// Cached layout entry point (F7.2).
    ///
    /// The cache key hashes the story content, every character/paragraph run
    /// attribute affecting geometry (family, size, spacing, weight, style,
    /// alignment, line height) and the max width bits. Fill color and other
    /// paint-only attributes are intentionally excluded: they never change
    /// geometry, so including them would only cause needless re-shaping.
    /// The cache holds at most [`MAX_CACHED_LAYOUTS`] entries and clears on
    /// overflow (bounded memory over unbounded editing sessions).
    #[must_use]
    pub fn layout_cached(story: &TextStory, max_width: Option<f64>) -> Self {
        let key = layout_cache_key(story, max_width);
        if let Some(hit) = layout_cache()
            .lock()
            .expect("layout cache is poisoned")
            .get(&key)
        {
            return hit.clone();
        }
        let computed = Self::layout_uncached(story, max_width);
        let mut guard = layout_cache().lock().expect("layout cache is poisoned");
        if guard.len() >= MAX_CACHED_LAYOUTS {
            guard.clear();
        }
        guard.insert(key, computed.clone());
        computed
    }

    /// Clears the layout cache. Test hook (deterministic cache-miss tests)
    /// and a memory hygiene entry point for long sessions.
    pub fn clear_layout_cache() {
        layout_cache()
            .lock()
            .expect("layout cache is poisoned")
            .clear();
    }

    /// Uncached layout over real shaped advances. Same greedy breaking
    /// semantics as the former heuristic (`\n` flushes, even empty; wrap
    /// before the overflowing character), only the widths and vertical
    /// metrics are real now.
    fn layout_uncached(story: &TextStory, max_width: Option<f64>) -> Self {
        if story.content.is_empty() {
            return Self {
                lines: Vec::new(),
                bounds: GRect::new(0.0, 0.0, 0.0, 0.0),
                line_count: 0,
            };
        }

        let default_font_size = story.char_runs.first().map_or(14.0, |r| r.style.font_size);
        let default_family = story
            .char_runs
            .first()
            .map_or_else(|| "Inter".to_string(), |r| r.style.font_family.clone());
        let default_line_height = story.para_runs.first().map_or(1.2, |r| r.style.line_height);
        let line_height_px = default_font_size * default_line_height;

        let advances = shaped_advances(story);
        debug_assert_eq!(advances.len(), story.content.chars().count());

        let max_w = max_width.unwrap_or(f64::INFINITY);

        let mut lines = Vec::new();
        let mut current_y = 0.0;
        let mut max_line_w: f64 = 0.0;

        let mut line_start_byte = 0;
        let mut current_line_text = String::new();
        let mut current_line_w: f64 = 0.0;
        let mut line_ascent: f64 = 0.0;
        let mut line_descent: f64 = 0.0;

        // Empty-line fallback: real probe of the run style at the break.
        let empty_metrics = |byte: usize| -> (f64, f64) {
            let style = style_at(story, byte, &default_family, default_font_size);
            let m = TypeSystem::font_metrics(&style.family, style.size as f32);
            (f64::from(m.ascent_px), f64::from(m.descent_px))
        };
        let flush_line = |text: &str,
                          start: usize,
                          end: usize,
                          width: f64,
                          ascent: f64,
                          descent: f64,
                          y: f64|
         -> LineFragment {
            let box_h = line_height_px.max(ascent + descent);
            LineFragment {
                range: (TextOffset::new(start), TextOffset::new(end)),
                baseline_y: y + ascent,
                bounds: GRect::new(0.0, y, width, y + box_h),
                text: text.to_string(),
            }
        };

        for (char_idx, (byte_idx, ch)) in story.content.char_indices().enumerate() {
            let (ch_w, ch_ascent, ch_descent) = advances[char_idx];
            if ch == '\n' {
                // Flush line
                let end_byte = byte_idx;
                let line_w = current_line_w.max(1.0);
                if line_w > max_line_w {
                    max_line_w = line_w;
                }
                let (ascent, descent) = if current_line_text.is_empty() {
                    empty_metrics(line_start_byte)
                } else {
                    (line_ascent, line_descent)
                };
                let box_h = line_height_px.max(ascent + descent);
                lines.push(flush_line(
                    &current_line_text,
                    line_start_byte,
                    end_byte,
                    line_w,
                    ascent,
                    descent,
                    current_y,
                ));
                current_y += box_h;
                line_start_byte = byte_idx + 1;
                current_line_text.clear();
                current_line_w = 0.0;
                line_ascent = 0.0;
                line_descent = 0.0;
                continue;
            }

            if current_line_w + ch_w > max_w && !current_line_text.is_empty() {
                // Wrap line
                let end_byte = byte_idx;
                if current_line_w > max_line_w {
                    max_line_w = current_line_w;
                }
                let box_h = line_height_px.max(line_ascent + line_descent);
                lines.push(flush_line(
                    &current_line_text,
                    line_start_byte,
                    end_byte,
                    current_line_w,
                    line_ascent,
                    line_descent,
                    current_y,
                ));
                current_y += box_h;
                line_start_byte = byte_idx;
                current_line_text.clear();
                current_line_w = 0.0;
                line_ascent = 0.0;
                line_descent = 0.0;
            }

            current_line_text.push(ch);
            current_line_w += ch_w;
            line_ascent = line_ascent.max(ch_ascent);
            line_descent = line_descent.max(ch_descent);
        }

        if !current_line_text.is_empty() || line_start_byte < story.content.len() {
            let end_byte = story.content.len();
            if current_line_w > max_line_w {
                max_line_w = current_line_w;
            }
            let (ascent, descent) = if line_ascent <= 0.0 && line_descent <= 0.0 {
                empty_metrics(line_start_byte)
            } else {
                (line_ascent, line_descent)
            };
            let box_h = line_height_px.max(ascent + descent);
            lines.push(flush_line(
                &current_line_text,
                line_start_byte,
                end_byte,
                current_line_w,
                ascent,
                descent,
                current_y,
            ));
            current_y += box_h;
        }

        // Apply alignment shifts if specified
        if let Some(para_run) = story.para_runs.first() {
            if para_run.style.align == TextAlign::Center && max_width.is_some() {
                let limit = max_w;
                for line in &mut lines {
                    let shift = ((limit - (line.bounds.x1 - line.bounds.x0)) / 2.0).max(0.0);
                    line.bounds.x0 += shift;
                    line.bounds.x1 += shift;
                }
            } else if para_run.style.align == TextAlign::Right && max_width.is_some() {
                let limit = max_w;
                for line in &mut lines {
                    let shift = (limit - (line.bounds.x1 - line.bounds.x0)).max(0.0);
                    line.bounds.x0 += shift;
                    line.bounds.x1 += shift;
                }
            }
        }

        let total_w = if max_width.is_some() {
            max_w
        } else {
            max_line_w
        };
        let bounds = GRect::new(0.0, 0.0, total_w, current_y);
        let line_count = lines.len();

        Self {
            lines,
            bounds,
            line_count,
        }
    }

    /// Hit-tests a coordinate against lines and returns the closest byte offset.
    #[must_use]
    pub fn hit_test(&self, point: GPoint) -> TextOffset {
        if self.lines.is_empty() {
            return TextOffset::new(0);
        }

        // Find matching line or nearest line vertically
        let target_line = self
            .lines
            .iter()
            .find(|l| point.y >= l.bounds.y0 && point.y <= l.bounds.y1)
            .unwrap_or_else(|| {
                if point.y < self.lines[0].bounds.y0 {
                    &self.lines[0]
                } else {
                    self.lines.last().unwrap()
                }
            });

        let line_w = target_line.bounds.x1 - target_line.bounds.x0;
        let char_count = target_line.text.chars().count();
        if char_count == 0 || line_w <= 0.0 {
            return target_line.range.0;
        }

        let rel_x = (point.x - target_line.bounds.x0).clamp(0.0, line_w);
        let char_fraction = rel_x / line_w;
        let approx_char_idx =
            ((char_fraction * char_count as f64).round() as usize).min(char_count);

        let byte_offset = target_line
            .text
            .char_indices()
            .nth(approx_char_idx)
            .map_or(target_line.range.1 .0, |(b, _)| target_line.range.0 .0 + b);

        TextOffset::new(byte_offset)
    }
}

/// Bound for the layout cache: cleared (not LRU-evicted) on overflow, so
/// memory stays bounded over long editing sessions. Layouts are pure values
/// of their inputs, so eviction only costs a re-shape, never correctness.
const MAX_CACHED_LAYOUTS: usize = 512;

fn layout_cache() -> &'static Mutex<HashMap<u64, TextLayout>> {
    static CACHE: OnceLock<Mutex<HashMap<u64, TextLayout>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Cache key: content + every run attribute that affects geometry + width.
///
/// Paint-only attributes (e.g. fill color) are excluded on purpose. Weight
/// and italic are keyed even though shaping currently resolves family+size
/// only, so the key stays correct if shaping starts honoring them.
fn layout_cache_key(story: &TextStory, max_width: Option<f64>) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    story.content.hash(&mut h);
    for run in &story.char_runs {
        run.start.0.hash(&mut h);
        run.end.0.hash(&mut h);
        run.style.font_family.hash(&mut h);
        run.style.font_size.to_bits().hash(&mut h);
        run.style.font_weight.hash(&mut h);
        run.style.italic.hash(&mut h);
        run.style.letter_spacing.to_bits().hash(&mut h);
    }
    for run in &story.para_runs {
        run.start.0.hash(&mut h);
        run.end.0.hash(&mut h);
        let align: u8 = match run.style.align {
            TextAlign::Left => 0,
            TextAlign::Center => 1,
            TextAlign::Right => 2,
            TextAlign::Justify => 3,
        };
        align.hash(&mut h);
        run.style.line_height.to_bits().hash(&mut h);
        run.style.indent.to_bits().hash(&mut h);
        run.style.paragraph_spacing.to_bits().hash(&mut h);
    }
    match max_width {
        None => 0u8.hash(&mut h),
        Some(w) => {
            1u8.hash(&mut h);
            w.to_bits().hash(&mut h);
        }
    }
    h.finish()
}

/// Resolved per-character shaping style: the run covering `byte`, or the
/// story default when no run covers it (e.g. stories built without runs).
struct ResolvedStyle {
    family: String,
    size: f64,
    spacing: f64,
}

fn style_at(
    story: &TextStory,
    byte: usize,
    default_family: &str,
    default_size: f64,
) -> ResolvedStyle {
    story
        .char_runs
        .iter()
        .find(|r| r.start.0 <= byte && byte < r.end.0)
        .map_or(
            ResolvedStyle {
                family: default_family.to_string(),
                size: default_size,
                spacing: 0.0,
            },
            |r| ResolvedStyle {
                family: r.style.font_family.clone(),
                size: r.style.font_size,
                spacing: r.style.letter_spacing,
            },
        )
}

/// Per-character `(advance_px, ascent_px, descent_px)` for the whole story,
/// in `char_indices` order.
///
/// Each paragraph (split on `\n`, which carries no advance) is grouped into
/// maximal segments of equal `(family, size)` and shaped once with
/// [`TypeSystem::shape_run_basic`]. Glyph advances map back to characters via
/// the glyph `start`/`end` byte ranges; a character with no covering glyph
/// (ligature tail under `Basic` shaping is not expected, but defended
/// against) falls back to the segment average so no character collapses to
/// zero width. `letter_spacing` applies per character on top.
fn shaped_advances(story: &TextStory) -> Vec<(f64, f64, f64)> {
    let default_family = story
        .char_runs
        .first()
        .map_or_else(|| "Inter".to_string(), |r| r.style.font_family.clone());
    let default_size = story.char_runs.first().map_or(14.0, |r| r.style.font_size);

    let mut out = Vec::with_capacity(story.content.chars().count());
    let mut para_start = 0usize;
    // Trailing `split_terminator`-style iteration that keeps the byte offset
    // of every paragraph start, including a final empty slice which yields
    // no characters and therefore no shaping work. Each consumed `\n` gets a
    // zero-width placeholder so the table stays 1:1 with `char_indices`.
    for (para_idx, para) in story.content.split('\n').enumerate() {
        if para_idx > 0 {
            out.push((0.0, 0.0, 0.0)); // the '\n' itself: breaks, never measures
        }
        let para_end = para_start + para.len();
        let mut seg_start = para_start;
        while seg_start < para_end {
            let head = style_at(story, seg_start, &default_family, default_size);
            let mut seg_end = seg_start;
            for (rel, ch) in para[seg_start - para_start..].char_indices() {
                let byte = seg_start + rel;
                let s = style_at(story, byte, &default_family, default_size);
                if s.family != head.family || s.size != head.size {
                    break;
                }
                seg_end = byte + ch.len_utf8();
            }
            shape_segment(
                &story.content[seg_start..seg_end],
                seg_start,
                &head,
                story,
                &default_family,
                default_size,
                &mut out,
            );
            seg_start = seg_end;
        }
        para_start = para_end + 1; // skip the '\n'
    }
    out
}

/// Shapes one uniform-style segment and pushes per-character metrics.
#[allow(clippy::too_many_arguments)]
fn shape_segment(
    text: &str,
    seg_base: usize,
    head: &ResolvedStyle,
    story: &TextStory,
    default_family: &str,
    default_size: f64,
    out: &mut Vec<(f64, f64, f64)>,
) {
    let size_f32 = head.size as f32;
    let glyphs = TypeSystem::shape_run_basic(text, &head.family, size_f32);
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let total: f32 = glyphs.iter().map(|g| g.advance_px).sum();
    let avg = if chars.is_empty() {
        0.0
    } else {
        total / chars.len() as f32
    };
    for (idx, (rel, _)) in chars.iter().enumerate() {
        let byte = seg_base + rel;
        let style = style_at(story, byte, default_family, default_size);
        let next_rel = chars.get(idx + 1).map_or(text.len(), |(r, _)| *r);
        let mut adv = 0.0f32;
        let mut asc = 0.0f32;
        let mut des = 0.0f32;
        let mut covered = false;
        for g in &glyphs {
            if g.start >= *rel && g.start < next_rel {
                adv += g.advance_px;
                asc = asc.max(g.ascent_px);
                des = des.max(g.descent_px);
                covered = true;
            }
        }
        if !covered {
            adv = avg;
            let m = TypeSystem::font_metrics(&head.family, size_f32);
            asc = m.ascent_px;
            des = m.descent_px;
        }
        adv += style.spacing as f32;
        out.push((f64::from(adv), f64::from(asc), f64::from(des)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_design_foundation::TextStoryId;

    #[test]
    fn layout_wraps_multiline_text() {
        let story = TextStory::with_content(
            TextStoryId::new(1),
            "Petunia Design Studio\nLayout e Tipografia",
        );
        let layout = TextLayout::layout(&story, None);
        assert_eq!(layout.line_count, 2);
        assert_eq!(layout.lines[0].text, "Petunia Design Studio");
        assert_eq!(layout.lines[1].text, "Layout e Tipografia");
    }

    #[test]
    fn hit_test_returns_valid_offset() {
        let story = TextStory::with_content(TextStoryId::new(1), "Hello");
        let layout = TextLayout::layout(&story, None);
        let offset = layout.hit_test(GPoint::new(1.0, 5.0));
        assert!(offset.0 <= "Hello".len());
    }
}
