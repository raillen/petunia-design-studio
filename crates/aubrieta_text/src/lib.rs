#![forbid(unsafe_code)]

//! Typography, text editing and layout engine contracts (09.8).
//!
//! Separates canonical text content (`TextStory`) from containers (`TextFrame`,
//! `TextOnPath`). Third-party shaping/raster engines sit behind this facade;
//! external types do not leak into document serialization.

pub mod layout;
pub mod story;

pub use layout::{LineFragment, TextLayout};
pub use story::{
    CharacterStyle, CharacterStyleRun, ParagraphStyle, ParagraphStyleRun, TextAlign, TextOffset,
    TextStory,
};
