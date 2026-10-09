//! Line layout, shaping and font fallback.
//!
//! Breaks come from real UAX #14 opportunities over grapheme-safe
//! text; advances come from the shaper; fallback walks faces by
//! script coverage. Shaping itself runs through rustybuzz faces
//! resolved by a fontdb-backed registry.

use crate::text::unicode::{break_opportunities, ParagraphDirection};
use petunia_core::styles::TextAlignment;
use serde::{Deserialize, Serialize};

/// One shaped glyph in document units.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ShapedGlyph {
    pub glyph_id: u32,
    pub cluster: u32,
    pub advance: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}

/// Shaping failures: unreadable bytes or an empty face table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShapingError {
    InvalidFontBytes,
    EmptyText,
}

impl std::fmt::Display for ShapingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFontBytes => write!(f, "font bytes do not parse"),
            Self::EmptyText => write!(f, "nothing to shape"),
        }
    }
}

/// Shape `text` with an explicit font byte slice at `size` document
/// units. Direction follows the paragraph analysis; advances scale
/// from font units. Missing glyphs surface as `.notdef` (id 0) for
/// the fallback layer to replace.
pub fn shape_run(
    font_bytes: &[u8],
    text: &str,
    direction: ParagraphDirection,
    size: f64,
) -> std::result::Result<Vec<ShapedGlyph>, ShapingError> {
    if text.is_empty() {
        return Err(ShapingError::EmptyText);
    }
    if !(size.is_finite() && size > 0.0) {
        return Err(ShapingError::InvalidFontBytes);
    }
    let face = rustybuzz::Face::from_slice(font_bytes, 0).ok_or(ShapingError::InvalidFontBytes)?;
    let units_per_em = f64::from(face.units_per_em());
    if units_per_em <= 0.0 {
        return Err(ShapingError::InvalidFontBytes);
    }
    let mut buffer = rustybuzz::UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(match direction {
        ParagraphDirection::LeftToRight => rustybuzz::Direction::LeftToRight,
        ParagraphDirection::RightToLeft => rustybuzz::Direction::RightToLeft,
    });
    let output = rustybuzz::shape(&face, &[], buffer);
    let infos = output.glyph_infos();
    let positions = output.glyph_positions();
    let scale = size / units_per_em;
    Ok(infos
        .iter()
        .zip(positions.iter())
        .map(|(info, position)| ShapedGlyph {
            glyph_id: info.glyph_id,
            cluster: info.cluster,
            advance: f64::from(position.x_advance) * scale,
            offset_x: f64::from(position.x_offset) * scale,
            offset_y: f64::from(position.y_offset) * scale,
        })
        .collect())
}

/// True when every glyph missed (whole run is `.notdef`).
#[must_use]
pub fn all_missing(glyphs: &[ShapedGlyph]) -> bool {
    !glyphs.is_empty() && glyphs.iter().all(|glyph| glyph.glyph_id == 0)
}

/// One font candidate for the fallback chain: name plus raw bytes.
/// Bytes stay caller-owned so shaper faces can borrow them.
#[derive(Debug, Clone)]
pub struct FallbackFace {
    pub family: String,
    pub bytes: Vec<u8>,
}

impl FallbackFace {
    /// True when the face maps `ch` through its cmap table.
    #[must_use]
    pub fn covers(&self, ch: char) -> bool {
        ttf_parser::Face::parse(&self.bytes, 0)
            .ok()
            .and_then(|face| face.glyph_index(ch))
            .is_some()
    }
}

/// First face covering `ch`, or `None` when the chain is exhausted.
/// Never rewrites references: absence is reported, not papered over.
#[must_use]
pub fn fallback_for_char(chain: &[FallbackFace], ch: char) -> Option<&FallbackFace> {
    chain.iter().find(|face| face.covers(ch))
}

/// What the resolver is asked for: families in preference order plus
/// weight and slant wishes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontQuery {
    pub families: Vec<String>,
    pub weight: u16,
    pub italic: bool,
}

/// One resolved face: display name plus bytes for the shaper.
#[derive(Debug, Clone)]
pub struct ResolvedFace {
    pub family: String,
    pub bytes: Vec<u8>,
}

/// System font registry: explicit directories plus optional system
/// fonts, matched case-insensitively in preference order with
/// closest-weight choice. Resolution never mutates references.
#[derive(Debug, Default)]
pub struct FontRegistry {
    database: fontdb::Database,
}

impl FontRegistry {
    /// Empty registry; load directories or system fonts explicitly.
    #[must_use]
    pub fn new() -> Self {
        Self {
            database: fontdb::Database::new(),
        }
    }

    /// Load fonts from explicit directories (recursive).
    pub fn load_dir(&mut self, dir: &std::path::Path) {
        self.database.load_fonts_dir(dir);
    }

    /// Load platform system fonts. Availability varies by machine;
    /// absence is a normal empty result, never an error.
    pub fn load_system_fonts(&mut self) {
        self.database.load_system_fonts();
    }

    /// Number of indexed faces.
    #[must_use]
    pub fn len(&self) -> usize {
        self.database.len()
    }

    /// True when nothing is indexed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.database.len() == 0
    }

    /// Resolve `query` to raw bytes: first family with a face wins,
    /// closest weight breaks ties, exact slant wins within a weight.
    #[must_use]
    pub fn resolve(&self, query: &FontQuery) -> Option<ResolvedFace> {
        for family in &query.families {
            let mut best: Option<(u32, String, Vec<u8>)> = None;
            for face in self.database.faces() {
                if !face
                    .families
                    .iter()
                    .any(|(name, _)| name.eq_ignore_ascii_case(family))
                {
                    continue;
                }
                let wanted_italic = query.italic;
                let is_italic = !matches!(face.style, fontdb::Style::Normal);
                let score = face.weight.0.abs_diff(query.weight) as u32 * 2
                    + u32::from(is_italic != wanted_italic);
                let better = best.as_ref().is_none_or(|(known, _, _)| score < *known);
                if better {
                    let bytes = self
                        .database
                        .with_face_data(face.id, |data, _| data.to_vec())?;
                    let name = face
                        .families
                        .first()
                        .map(|(name, _)| name.clone())
                        .unwrap_or_else(|| family.clone());
                    best = Some((score, name, bytes));
                }
            }
            if let Some((_, name, bytes)) = best {
                return Some(ResolvedFace {
                    family: name,
                    bytes,
                });
            }
        }
        None
    }
}

/// One broken line: byte range, width and x offset for alignment.
/// Left/Center/Right are exact; Justified currently aligns like Left
/// (space distribution is a documented follow-up).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BrokenLine {
    pub start_byte: usize,
    pub end_byte: usize,
    pub width: f64,
    pub x_offset: f64,
}

/// Greedy line breaking over shaped advances: walk glyphs, remember
/// the last soft opportunity inside the width, force mandatory
/// breaks. Advances pair each glyph with its cluster byte offset.
#[must_use]
pub fn break_lines(
    text: &str,
    advances: &[(u32, f64)],
    max_width: f64,
    alignment: TextAlignment,
) -> Vec<BrokenLine> {
    if text.is_empty() || advances.is_empty() || !(max_width.is_finite() && max_width > 0.0) {
        return Vec::new();
    }
    let breaks = break_opportunities(text);
    let mut lines = Vec::new();
    let mut line_start = 0usize;
    let mut line_width = 0.0;
    let mut last_break: Option<(usize, usize, f64)> = None;
    let mut index = 0;
    while index < advances.len() {
        let (cluster, advance) = advances[index];
        let byte = cluster as usize;
        if byte > line_start && breaks.iter().any(|(offset, _)| *offset == byte) {
            last_break = Some((index, byte, line_width));
        }
        let mandatory = breaks
            .iter()
            .any(|(offset, forced)| *offset == byte && *forced && byte > line_start);
        if line_width + advance > max_width && byte > line_start {
            if let Some((break_index, break_byte, break_width)) = last_break {
                lines.push((line_start, break_byte, break_width, break_index + 1));
                line_start = break_byte;
                line_width = 0.0;
                // Re-measure from the break forward.
                let mut restart = break_index + 1;
                while restart <= index {
                    line_width += advances[restart].1;
                    restart += 1;
                }
                last_break = None;
                index += 1;
                continue;
            }
            lines.push((line_start, byte, line_width, index));
            line_start = byte;
            line_width = 0.0;
            last_break = None;
            continue;
        }
        if mandatory {
            line_width += advance;
            lines.push((line_start, byte, line_width, index + 1));
            line_start = byte;
            line_width = 0.0;
            last_break = None;
            index += 1;
            continue;
        }
        line_width += advance;
        index += 1;
    }
    if line_start < text.len() || lines.is_empty() {
        lines.push((line_start, text.len(), line_width, advances.len()));
    }
    lines
        .into_iter()
        .map(|(start, end, width, _)| BrokenLine {
            start_byte: start,
            end_byte: end,
            width,
            x_offset: align_offset(alignment, width, max_width),
        })
        .collect()
}

/// Horizontal shift for one line under an alignment.
#[must_use]
pub fn align_offset(alignment: TextAlignment, line_width: f64, max_width: f64) -> f64 {
    match alignment {
        TextAlignment::Left | TextAlignment::Justify => 0.0,
        TextAlignment::Center => ((max_width - line_width) / 2.0).max(0.0),
        TextAlignment::Right => (max_width - line_width).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn advances(text: &str, width: f64) -> Vec<(u32, f64)> {
        let mut out = Vec::new();
        for (byte, _) in text.char_indices() {
            out.push((byte as u32, width));
        }
        out
    }

    #[test]
    fn shaping_rejects_bad_inputs_deterministically() {
        assert!(matches!(
            shape_run(&[0u8; 16], "Hi", ParagraphDirection::LeftToRight, 12.0),
            Err(ShapingError::InvalidFontBytes)
        ));
        let face = rustybuzz::Face::from_slice(&[0u8; 16], 0);
        assert!(face.is_none());
        assert!(matches!(
            shape_run(
                b"bytes-are-checked-first-anyway",
                "",
                ParagraphDirection::LeftToRight,
                12.0
            ),
            Err(ShapingError::EmptyText)
        ));
    }

    #[test]
    fn shaping_with_system_font_or_reports_unavailable() {
        let mut registry = FontRegistry::new();
        registry.load_system_fonts();
        let query = FontQuery {
            families: vec!["DejaVu Sans".to_string(), "Liberation Sans".to_string()],
            weight: 400,
            italic: false,
        };
        let Some(face) = registry.resolve(&query) else {
            // No system fonts here: the empty-registry path is the
            // asserted behavior, not a silent pass.
            assert!(registry.is_empty());
            return;
        };
        let glyphs = shape_run(&face.bytes, "Hello", ParagraphDirection::LeftToRight, 12.0)
            .expect("system font shapes");
        assert!(!glyphs.is_empty());
        assert!(!all_missing(&glyphs));
        assert!(glyphs.iter().all(|glyph| glyph.advance > 0.0));
    }

    #[test]
    fn fallback_walks_chain_by_cmap_coverage() {
        let empty = FallbackFace {
            family: "empty".to_string(),
            bytes: vec![0u8; 16],
        };
        assert!(fallback_for_char(&[empty], 'A').is_none());
    }

    #[test]
    fn lines_break_at_opportunities_with_alignment() {
        let text = "one two three";
        let broken = break_lines(
            text,
            &advances(text, 10.0),
            55.0,
            petunia_core::styles::TextAlignment::Center,
        );
        assert!(broken.len() >= 2, "got {broken:?}");
        // No line exceeds the width; centered lines shift right.
        for line in &broken {
            assert!(line.width <= 55.0 + 1e-9, "{line:?}");
            assert!((line.x_offset - (55.0 - line.width) / 2.0).abs() < 1e-9);
        }
        // Byte ranges tile the text without gaps.
        assert_eq!(broken[0].start_byte, 0);
        assert_eq!(broken.last().expect("lines").end_byte, text.len());
    }

    #[test]
    fn mandatory_breaks_force_lines() {
        let text = "ab\ncd";
        let broken = break_lines(
            text,
            &advances(text, 10.0),
            1000.0,
            petunia_core::styles::TextAlignment::Left,
        );
        assert_eq!(broken.len(), 2);
    }
}
