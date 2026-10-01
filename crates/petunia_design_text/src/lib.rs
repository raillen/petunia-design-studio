#![forbid(unsafe_code)]

//! Typography, text editing and layout engine contracts (09.8).
//!
//! Separates canonical text content (`TextStory`) from containers (`TextFrame`,
//! `TextOnPath`). Third-party shaping/raster engines sit behind this facade;
//! external types do not leak into document serialization.

pub mod fonts;
pub mod layout;
pub mod on_path;
pub mod outlines;
pub mod story;
pub use outlines::{prepare_text, PreparedText, TextFrameSpec, TextRenderError};

pub use fonts::{FontMetrics, ShapedGlyph, TypeSystem};

pub use layout::{LineFragment, TextLayout};
pub use story::{
    CharacterStyle, CharacterStyleRun, ParagraphStyle, ParagraphStyleRun, TextAlign, TextOffset,
    TextStory,
};
