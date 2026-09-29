//! Text-on-path over the cached layout (F7.3).
//!
//! Positions glyphs from **real shaped advances** ([`TypeSystem::shape_run_basic`],
//! `Shaping::Basic`, the same fast path [`crate::layout`] uses) over **one shared
//! flatten** of the target outline. Handles are the run edges (`start` of the first
//! glyph, `end` of the last), so they coincide with the uniform span endpoints
//! within `1e-6`; hit-testing resolves a document point to a byte offset through
//! the same per-character slices.
//!
//! Caching: advances are memoized per `(content, family, size, spacing)` in a
//! bounded 512-entry cache cleared on overflow (same bound as
//! [`TextLayout::layout_cached`); eviction only costs a re-shape, never
//! correctness. [`layout_for_span`] additionally warms the layout cache so span
//! work never re-shapes an already-laid-out story. [`TypeSystem::shape_count`] /
//! [`TypeSystem::reset_shape_count`] stay the deterministic probe: a second call
//! with identical inputs shapes zero new runs.
//!
//! Single-line flow: `'\n'` carries zero advance (it breaks lines in straight
//! text, never on a path) and its zero-width slice is never hit; offsets of
//! `'\n'` bytes are unreachable on purpose.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, OnceLock};

use petunia_design_foundation::TextStoryId;
use petunia_design_geometry::GPoint;

use crate::fonts::TypeSystem;
use crate::layout::TextLayout;
use crate::story::{
    CharacterStyle, CharacterStyleRun, ParagraphStyle, ParagraphStyleRun, TextAlign, TextOffset,
    TextStory,
};

/// Bound for the advance cache: cleared (not LRU-evicted) on overflow, mirroring
/// [`TextLayout`]'s 512-entry bound.
const MAX_CACHED_ADVANCES: usize = 512;

fn advance_cache() -> &'static Mutex<HashMap<u64, Vec<f64>>> {
    static CACHE: OnceLock<Mutex<HashMap<u64, Vec<f64>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn advance_cache_key(content: &str, family: &str, size: f64, letter_spacing: f64) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut h);
    family.hash(&mut h);
    size.to_bits().hash(&mut h);
    letter_spacing.to_bits().hash(&mut h);
    h.finish()
}

/// Builds the single-run story a span shapes. One uniform run covers the whole
/// content; `line_height` travels in the paragraph run (it never changes
/// advances, but keeps the layout key exact).
#[must_use]
pub fn story_for_span(
    content: &str,
    family: &str,
    size: f64,
    letter_spacing: f64,
    line_height: f64,
) -> TextStory {
    let end = TextOffset::new(content.len());
    TextStory {
        id: TextStoryId::new(1),
        content: content.to_string(),
        char_runs: vec![CharacterStyleRun {
            start: TextOffset::new(0),
            end,
            style: CharacterStyle {
                font_family: family.to_string(),
                font_size: size,
                letter_spacing,
                ..CharacterStyle::default()
            },
        }],
        para_runs: vec![ParagraphStyleRun {
            start: TextOffset::new(0),
            end,
            style: ParagraphStyle {
                align: TextAlign::Left,
                line_height,
                ..ParagraphStyle::default()
            },
        }],
    }
}

/// Cached layout for a span (single line, `max_width = None`). Warms the
/// 512-entry layout cache; callers that also need advances should call
/// [`advances_for_span`] (separate 512-entry cache, same shaping path).
#[must_use]
pub fn layout_for_span(
    content: &str,
    family: &str,
    size: f64,
    letter_spacing: f64,
    line_height: f64,
) -> TextLayout {
    let story = story_for_span(content, family, size, letter_spacing, line_height);
    TextLayout::layout_cached(&story, None)
}

/// Per-character advances in `char_indices` order, cached.
///
/// Each paragraph (split on `'\n'`, which carries `0.0`) is shaped once with
/// [`TypeSystem::shape_run_basic`]; glyph advances map back to characters via
/// glyph `start`/`end` byte ranges, with segment-average fallback for uncovered
/// characters and `letter_spacing` applied per character. Mirrors
/// [`crate::layout`]'s shaping without duplicating its line breaking.
#[must_use]
pub fn advances_for_span(content: &str, family: &str, size: f64, letter_spacing: f64) -> Vec<f64> {
    let key = advance_cache_key(content, family, size, letter_spacing);
    if let Some(hit) = advance_cache()
        .lock()
        .expect("on-path advance cache is poisoned")
        .get(&key)
    {
        return hit.clone();
    }
    let computed = advances_uncached(content, family, size, letter_spacing);
    let mut guard = advance_cache()
        .lock()
        .expect("on-path advance cache is poisoned");
    if guard.len() >= MAX_CACHED_ADVANCES {
        guard.clear();
    }
    guard.insert(key, computed.clone());
    computed
}

/// Clears the advance cache. Test hook and memory hygiene entry point.
pub fn clear_advance_cache() {
    advance_cache()
        .lock()
        .expect("on-path advance cache is poisoned")
        .clear();
}

fn advances_uncached(content: &str, family: &str, size: f64, letter_spacing: f64) -> Vec<f64> {
    let size_f32 = size as f32;
    let mut out = Vec::with_capacity(content.chars().count());
    // `split_terminator`-style walk that keeps `'\n'` placeholders so the table
    // stays 1:1 with `char_indices`.
    let mut para_start = 0usize;
    for (para_idx, para) in content.split('\n').enumerate() {
        if para_idx > 0 {
            out.push(0.0); // the '\n' itself: breaks straight lines, never measures
        }
        if para.is_empty() {
            para_start += 1;
            continue;
        }
        let glyphs = TypeSystem::shape_run_basic(para, family, size_f32);
        let chars: Vec<(usize, char)> = para.char_indices().collect();
        let total: f32 = glyphs.iter().map(|g| g.advance_px).sum();
        let avg = if chars.is_empty() {
            0.0
        } else {
            total / chars.len() as f32
        };
        for (idx, (rel, _)) in chars.iter().enumerate() {
            let next_rel = chars.get(idx + 1).map_or(para.len(), |(r, _)| *r);
            let mut adv = 0.0f32;
            let mut covered = false;
            for g in &glyphs {
                if g.start >= *rel && g.start < next_rel {
                    adv += g.advance_px;
                    covered = true;
                }
            }
            if !covered {
                adv = avg;
            }
            adv += letter_spacing as f32;
            let _ = para_start;
            out.push(f64::from(adv));
        }
        para_start += para.len() + 1;
    }
    out
}

/// Per-character `(t0, t1)` path fractions for a span.
///
/// Character `i` owns the slice proportional to `advances[i]`; `start` is the
/// first slice's `t0` and `end` the last slice's `t1` by construction, which is
/// what makes layout handles coincide with the uniform span endpoints. Inverted
/// spans are ordered; zero-total advances (empty or all-`'\n'`) distribute
/// uniformly so every character stays addressable.
#[must_use]
pub fn glyph_ts(start: f64, end: f64, advances: &[f64]) -> Vec<(f64, f64)> {
    if advances.is_empty() {
        return Vec::new();
    }
    let (mut a, mut b) = (start, end);
    if b < a {
        std::mem::swap(&mut a, &mut b);
    }
    let total: f64 = advances.iter().sum();
    if total < 1e-9 {
        let n = advances.len() as f64;
        return (0..advances.len())
            .map(|i| {
                let t0 = a + (b - a) * (i as f64 / n);
                let t1 = a + (b - a) * ((i + 1) as f64 / n);
                (t0, t1)
            })
            .collect();
    }
    let mut cum = 0.0;
    advances
        .iter()
        .map(|adv| {
            let t0 = a + (b - a) * (cum / total);
            cum += *adv;
            let t1 = a + (b - a) * (cum / total);
            (t0, t1)
        })
        .collect()
}

/// Byte offset of the character owning path fraction `t`.
///
/// Clamps outside `[start, end]` to the span edges (`0` / `content.len()`);
/// zero-width slices (`'\n'`) are never hit and resolve to their successor.
#[must_use]
pub fn offset_for_t(content: &str, advances: &[f64], start: f64, end: f64, t: f64) -> TextOffset {
    let char_count = content.chars().count();
    if content.is_empty() || advances.is_empty() || char_count == 0 {
        return TextOffset::new(0);
    }
    let (mut a, mut b) = (start, end);
    if b < a {
        std::mem::swap(&mut a, &mut b);
    }
    if t <= a {
        return TextOffset::new(0);
    }
    if t >= b {
        return TextOffset::new(content.len());
    }
    let slices = glyph_ts(a, b, advances);
    let bytes: Vec<usize> = content.char_indices().map(|(byte, _)| byte).collect();
    for (idx, (_, t1)) in slices.iter().enumerate() {
        // Zero-width slices never own a fraction; the strict `<` skips them
        // toward their successor. The last slice is inclusive of `b`.
        let is_last = idx + 1 == slices.len();
        if t < *t1 || (is_last && t <= *t1) {
            // Skip zero-width predecessors: advance to the first non-empty
            // slice at or after `idx` so `'\n'` bytes stay unreachable.
            let mut target = idx;
            while target + 1 < slices.len() && slices[target].1 <= slices[target].0 {
                target += 1;
            }
            let byte = bytes.get(target).copied().unwrap_or(content.len());
            return TextOffset::new(byte);
        }
    }
    TextOffset::new(content.len())
}

/// Arc-length walk over one shared flatten (mirrors `GPath::sample_at` and the
/// session `cached_sample_at` walk, without re-flattening).
#[must_use]
pub fn sample_walk(polys: &[Vec<GPoint>], t: f64) -> Option<(GPoint, f64)> {
    let total: f64 = polys
        .iter()
        .map(|c| c.windows(2).map(|w| w[0].distance_to(w[1])).sum::<f64>())
        .sum();
    if total < 1e-9 {
        let pt = polys.iter().flatten().next().copied()?;
        return Some((pt, 0.0));
    }
    let mut target = t.clamp(0.0, 1.0) * total;
    for contour in polys {
        for w in contour.windows(2) {
            let seg = w[0].distance_to(w[1]);
            if target <= seg {
                let f = if seg < 1e-12 { 0.0 } else { target / seg };
                let pt = GPoint::new(
                    w[0].x + (w[1].x - w[0].x) * f,
                    w[0].y + (w[1].y - w[0].y) * f,
                );
                return Some((pt, (w[1].y - w[0].y).atan2(w[1].x - w[0].x)));
            }
            target -= seg;
        }
    }
    let last = polys.iter().flatten().next_back().copied()?;
    Some((last, 0.0))
}

/// Nearest-fraction walk over one shared flatten (mirrors `GPath::nearest_t`).
#[must_use]
pub fn nearest_walk(polys: &[Vec<GPoint>], pt: GPoint) -> Option<f64> {
    let total: f64 = polys
        .iter()
        .map(|c| c.windows(2).map(|w| w[0].distance_to(w[1])).sum::<f64>())
        .sum();
    if total < 1e-9 {
        return None;
    }
    let mut best = (f64::INFINITY, 0.0);
    let mut acc = 0.0;
    for contour in polys {
        for w in contour.windows(2) {
            let (abx, aby) = (w[1].x - w[0].x, w[1].y - w[0].y);
            let len2 = (abx * abx + aby * aby).max(1e-12);
            let f = (((pt.x - w[0].x) * abx + (pt.y - w[0].y) * aby) / len2).clamp(0.0, 1.0);
            let proj = GPoint::new(w[0].x + abx * f, w[0].y + aby * f);
            let d = proj.distance_to(pt);
            if d < best.0 {
                let seg = abx.hypot(aby);
                best = (d, (acc + seg * f) / total);
            }
            acc += abx.hypot(aby);
        }
    }
    Some(best.1.clamp(0.0, 1.0))
}

/// Per-glyph centers over one shared flatten: `(point, tangent, byte_start, byte_end)`.
///
/// Centers drive rendering and hit-testing; run edges (`glyph_ts` first `t0`,
/// last `t1`) are the handles and coincide with the span endpoints.
#[must_use]
pub fn glyph_positions(
    polys: &[Vec<GPoint>],
    content: &str,
    advances: &[f64],
    start: f64,
    end: f64,
) -> Vec<(GPoint, f64, usize, usize)> {
    let slices = glyph_ts(start, end, advances);
    let bytes: Vec<usize> = content.char_indices().map(|(byte, _)| byte).collect();
    let char_count = slices.len();
    let mut out = Vec::with_capacity(char_count);
    for (idx, (t0, t1)) in slices.iter().enumerate() {
        let center = (*t0 + *t1) / 2.0;
        let Some((pt, angle)) = sample_walk(polys, center) else {
            continue;
        };
        let byte_start = bytes.get(idx).copied().unwrap_or(content.len());
        let byte_end = bytes.get(idx + 1).copied().unwrap_or(content.len());
        out.push((pt, angle, byte_start, byte_end));
    }
    out
}

/// Byte offset of the glyph nearest `pt`, over one shared flatten.
///
/// Combines [`nearest_walk`] (path fraction) with [`offset_for_t`] (layout
/// slices): a click on the span resolves through the runs, never through
/// uniform sampling.
#[must_use]
pub fn offset_at_point(
    polys: &[Vec<GPoint>],
    content: &str,
    advances: &[f64],
    start: f64,
    end: f64,
    pt: GPoint,
) -> Option<TextOffset> {
    if content.is_empty() || advances.is_empty() {
        return None;
    }
    let t = nearest_walk(polys, pt)?;
    Some(offset_for_t(content, advances, start, end, t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advances_are_cached_without_reshaping() {
        clear_advance_cache();
        TextLayout::clear_layout_cache();
        TypeSystem::reset_shape_count();
        let first = advances_for_span("Petunia Typography", "Inter", 24.0, 0.0);
        let after_first = TypeSystem::shape_count();
        assert!(after_first > 0, "first call must shape");
        assert_eq!(first.len(), "Petunia Typography".chars().count());
        assert!(first.iter().all(|a| *a > 0.0));

        let second = advances_for_span("Petunia Typography", "Inter", 24.0, 0.0);
        assert_eq!(
            TypeSystem::shape_count(),
            after_first,
            "second call must not re-shape"
        );
        assert_eq!(first, second);
    }

    #[test]
    fn glyph_edges_anchor_span_endpoints() {
        let advances = vec![10.0, 20.0, 10.0];
        let slices = glyph_ts(0.25, 1.0, &advances);
        assert!((slices[0].0 - 0.25).abs() < 1e-12);
        assert!((slices.last().unwrap().1 - 1.0).abs() < 1e-12);
        // Proportional widths: middle owns half the span.
        assert!((slices[1].1 - slices[1].0 - 0.375).abs() < 1e-12);
    }

    #[test]
    fn offset_round_trips_through_slices() {
        let content = "abcd";
        let advances = vec![10.0, 10.0, 10.0, 10.0];
        assert_eq!(
            offset_for_t(content, &advances, 0.0, 1.0, 0.0),
            TextOffset::new(0)
        );
        assert_eq!(
            offset_for_t(content, &advances, 0.0, 1.0, 0.3),
            TextOffset::new(1)
        );
        assert_eq!(
            offset_for_t(content, &advances, 0.0, 1.0, 0.9),
            TextOffset::new(3)
        );
        assert_eq!(
            offset_for_t(content, &advances, 0.0, 1.0, 1.0),
            TextOffset::new(4)
        );
    }
}
