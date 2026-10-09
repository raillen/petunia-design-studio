//! Positioned text: resolved glyphs, never reshaped in render.
//!
//! The text engine owns shaping and layout; primitives carry glyph
//! IDs, positions, paint and bounds. Atlas slots stay runtime state.

use crate::paint::RenderPaint;
use petunia_core::{ObjectId, Rect, ResourceId};
use serde::{Deserialize, Serialize};

/// One positioned glyph in document space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PositionedGlyph {
    pub glyph_id: u32,
    pub cluster: u32,
    pub x: f64,
    pub y: f64,
    pub advance: f64,
}

/// One run in a single face at one size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextRun {
    pub font: ResourceId,
    pub size: f64,
    pub glyphs: Vec<PositionedGlyph>,
}

/// Evaluated text primitive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextPrimitive {
    pub source: ObjectId,
    pub runs: Vec<TextRun>,
    pub paint: RenderPaint,
    pub bounds: Rect,
}
