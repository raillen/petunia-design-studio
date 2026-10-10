//! Text engine: Unicode algorithms, shaping, fallback and layout.
//!
//! Grapheme-correct editing, BiDi visual order, script runs and UAX
//! #14 breaks come from dedicated libraries. Shaping runs through
//! rustybuzz faces; font resolution never rewrites references.

pub mod layout;
pub mod unicode;

pub use layout::{
    align_offset, all_missing, break_lines, fallback_for_char, shape_run, BrokenLine, FallbackFace,
    FontQuery, FontRegistry, ResolvedFace, ShapedGlyph, ShapingError,
};
pub use unicode::{
    analyze_paragraph, break_opportunities, embedding_levels, grapheme_boundaries, script_runs,
    ParagraphDirection,
};

pub mod evaluate;
pub mod flow;
pub mod path;
pub use evaluate::{
    evaluate_text, GlyphOutline, LayoutLine, PositionedGlyph, TextEvaluationError, TextLayout,
};
pub use flow::evaluate_text_flow;
pub use path::evaluate_text_on_path;

pub mod hyphenation;
pub use hyphenation::{HyphenationProvider, PatternHyphenation};
