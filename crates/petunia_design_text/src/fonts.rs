//! Real text foundation: system fonts, shaping and fallback (F7.1/F7.2).
//!
//! Owns the single process-wide [`cosmic_text::FontSystem`] behind a
//! `std` [`Mutex`] in a [`OnceLock`], and exposes shaping **metrics only**
//! ([`TypeSystem::shape_run`] with [`Shaping::Advanced`], and the fast
//! [`TypeSystem::shape_run_basic`] with [`Shaping::Basic`]). No rasterization
//! happens here; glyph bitmap caching (`SwashCache`) is a later step.
//!
//! Shaping choice:
//! - [`TypeSystem::shape_run`] (`Advanced`) resolves complex scripts and
//!   per-script fallback through `fontdb`; keep it wherever glyph identity
//!   across scripts matters (multilingual runs, fallback reporting).
//! - [`TypeSystem::shape_run_basic`] (`Basic`, no HarfBuzz itemization) is
//!   the fast path used by [`crate::layout`] line breaking: one glyph per
//!   character with real proportional advances and font ascent/descent.
//!   Complex-script clusters still get real per-glyph advances, but without
//!   advanced contextual shaping.
//!
//! [`TypeSystem::shape_count`] / [`TypeSystem::reset_shape_count`] are
//! per-thread test hooks proving the layout cache does not re-shape.

use std::cell::Cell;
use std::sync::{Mutex, MutexGuard, OnceLock};

use cosmic_text::{Attrs, AttrsList, Family, FontSystem, ShapeLine, Shaping};

/// Real font vertical metrics at a requested size, in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontMetrics {
    /// Pixels above the baseline (`glyph.ascent * size`, clamped `>= 0`).
    pub ascent_px: f32,
    /// Pixels below the baseline (`glyph.descent * size`, clamped `>= 0`).
    pub descent_px: f32,
}

thread_local! {
    /// Per-thread shaping call counter. Thread-local (not global) so cache
    /// tests stay deterministic under `cargo test` parallelism: only the
    /// current thread's shapings are observed.
    static SHAPE_CALLS: Cell<u64> = const { Cell::new(0) };
}

fn count_shape() {
    SHAPE_CALLS.with(|c| c.set(c.get().saturating_add(1)));
}

/// One shaped glyph: metrics only, no pixels.
///
/// `glyph_id == 0` is `.notdef` (tofu): the fallback chain found no system
/// font covering the character. Callers must surface that loudly instead of
/// rendering tofu silently.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapedGlyph {
    /// Font-specific glyph index; `0` means `.notdef` (uncovered).
    pub glyph_id: u16,
    /// Horizontal advance in pixels at the requested size.
    pub advance_px: f32,
    /// System family that actually supplied the glyph (may differ from the
    /// requested family when fallback resolved another script).
    pub fallback_family: String,
    /// Byte offset where this glyph's cluster starts in the shaped string.
    pub start: usize,
    /// Byte offset where this glyph's cluster ends in the shaped string.
    pub end: usize,
    /// Font ascent in pixels at the requested size (see [`FontMetrics`]).
    pub ascent_px: f32,
    /// Font descent in pixels at the requested size (see [`FontMetrics`]).
    pub descent_px: f32,
}

/// Process-wide font system, created once on first use.
///
/// `FontSystem::new` scans system fonts (up to ~1s in release), so it must
/// exist exactly once per app and be shared.
static FONT_SYSTEM: OnceLock<Mutex<FontSystem>> = OnceLock::new();

fn global() -> &'static Mutex<FontSystem> {
    FONT_SYSTEM.get_or_init(|| Mutex::new(FontSystem::new()))
}

/// Facade over the global [`FontSystem`].
pub struct TypeSystem;

impl TypeSystem {
    pub(crate) fn lock() -> MutexGuard<'static, FontSystem> {
        global().lock().unwrap_or_else(|error| error.into_inner())
    }

    /// Shapes one line of text and returns per-glyph metrics.
    ///
    /// Uses [`Shaping::Advanced`] so complex scripts and per-script font
    /// fallback resolve through `fontdb`; proportional advances come from
    /// the real fonts, never from a fixed width factor.
    #[must_use]
    pub fn shape_run(text: &str, family: &str, size_px: f32) -> Vec<ShapedGlyph> {
        Self::shape_impl(text, family, size_px, Shaping::Advanced)
    }

    /// Fast shaping path for layout: [`Shaping::Basic`] (no HarfBuzz
    /// itemization), one glyph per character with real advances and font
    /// ascent/descent. Used by [`crate::layout`] line breaking.
    #[must_use]
    pub fn shape_run_basic(text: &str, family: &str, size_px: f32) -> Vec<ShapedGlyph> {
        Self::shape_impl(text, family, size_px, Shaping::Basic)
    }

    /// Real vertical font metrics for `family` at `size_px`, in pixels.
    ///
    /// Shapes a short probe (`"Hg"`, covering ascender-to-baseline range)
    /// through the same counted path as layout and takes the maximum
    /// ascent/descent, so the values always come from the actual resolved
    /// font (including fallback), never from a `size * factor` heuristic.
    #[must_use]
    pub fn font_metrics(family: &str, size_px: f32) -> FontMetrics {
        assert!(
            size_px.is_finite() && size_px > 0.0,
            "font_metrics needs a finite positive size, got {size_px}"
        );
        let probe = Self::shape_impl("Hg", family, size_px, Shaping::Basic);
        let mut ascent_px = 0.0f32;
        let mut descent_px = 0.0f32;
        for glyph in &probe {
            ascent_px = ascent_px.max(glyph.ascent_px);
            descent_px = descent_px.max(glyph.descent_px);
        }
        FontMetrics {
            ascent_px,
            descent_px,
        }
    }

    /// Number of shapings performed on the current thread. Test hook for
    /// proving the layout cache serves repeated layouts without re-shaping.
    #[must_use]
    pub fn shape_count() -> u64 {
        SHAPE_CALLS.with(|c| c.get())
    }

    /// Resets the per-thread shaping counter. Test hook.
    pub fn reset_shape_count() {
        SHAPE_CALLS.with(|c| c.set(0));
    }

    fn shape_impl(text: &str, family: &str, size_px: f32, shaping: Shaping) -> Vec<ShapedGlyph> {
        assert!(
            size_px.is_finite() && size_px > 0.0,
            "shape_run needs a finite positive size, got {size_px}"
        );
        if text.is_empty() {
            return Vec::new();
        }
        count_shape();
        let mut system = Self::lock();
        let attrs = Attrs::new().family(Family::Name(family));
        let attrs_list = AttrsList::new(&attrs);
        let line = ShapeLine::new(&mut system, text, &attrs_list, shaping, 4);

        let mut out = Vec::new();
        for span in &line.spans {
            for word in &span.words {
                for glyph in &word.glyphs {
                    let fallback_family = system
                        .db()
                        .face(glyph.font_id)
                        .and_then(|face| face.families.first().map(|(name, _)| name.clone()))
                        .unwrap_or_default();
                    out.push(ShapedGlyph {
                        glyph_id: glyph.glyph_id,
                        advance_px: glyph.width(size_px),
                        fallback_family,
                        start: glyph.start,
                        end: glyph.end,
                        ascent_px: (size_px * glyph.ascent).max(0.0),
                        descent_px: (size_px * glyph.descent).max(0.0),
                    });
                }
            }
        }
        out
    }
}
