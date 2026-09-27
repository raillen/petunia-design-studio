//! Canonical text story model and boundary-validated offsets (09.8).

use petunia_design_color::ColorValue;
use petunia_design_foundation::{PetuniaError, TextStoryId};
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// UTF-8 byte offset into a story's content. Always validated against
/// character boundaries before use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TextOffset(pub usize);

impl TextOffset {
    #[must_use]
    pub const fn new(offset: usize) -> Self {
        Self(offset)
    }

    #[must_use]
    pub const fn byte_index(self) -> usize {
        self.0
    }
}

/// Character style properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterStyle {
    pub font_family: String,
    pub font_size: f64,
    pub font_weight: u16,
    pub italic: bool,
    pub letter_spacing: f64,
    pub fill_color: Option<ColorValue>,
}

impl Default for CharacterStyle {
    fn default() -> Self {
        Self {
            font_family: "Inter".to_string(),
            font_size: 14.0,
            font_weight: 400,
            italic: false,
            letter_spacing: 0.0,
            fill_color: None,
        }
    }
}

/// A contiguous slice of character style within a story.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterStyleRun {
    pub start: TextOffset,
    pub end: TextOffset,
    pub style: CharacterStyle,
}

/// Paragraph alignment options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

/// Paragraph-level styling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParagraphStyle {
    pub align: TextAlign,
    pub line_height: f64,
    pub indent: f64,
    pub paragraph_spacing: f64,
}

impl Default for ParagraphStyle {
    fn default() -> Self {
        Self {
            align: TextAlign::Left,
            line_height: 1.2,
            indent: 0.0,
            paragraph_spacing: 8.0,
        }
    }
}

/// A contiguous slice of paragraph style within a story.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParagraphStyleRun {
    pub start: TextOffset,
    pub end: TextOffset,
    pub style: ParagraphStyle,
}

/// Canonical text story entity (09.8).
///
/// Separates text content from text containers (frames, text-on-path) so
/// linked frames share one story without duplicating strings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStory {
    pub id: TextStoryId,
    pub content: String,
    pub char_runs: Vec<CharacterStyleRun>,
    pub para_runs: Vec<ParagraphStyleRun>,
}

impl TextStory {
    /// Creates an empty story with a given stable ID.
    #[must_use]
    pub fn new(id: TextStoryId) -> Self {
        Self {
            id,
            content: String::new(),
            char_runs: Vec::new(),
            para_runs: Vec::new(),
        }
    }

    /// Creates a story populated with initial text and default styles.
    #[must_use]
    pub fn with_content(id: TextStoryId, initial_text: &str) -> Self {
        let content = initial_text.to_string();
        let end = TextOffset::new(content.len());
        Self {
            id,
            char_runs: vec![CharacterStyleRun {
                start: TextOffset::new(0),
                end,
                style: CharacterStyle::default(),
            }],
            para_runs: vec![ParagraphStyleRun {
                start: TextOffset::new(0),
                end,
                style: ParagraphStyle::default(),
            }],
            content,
        }
    }

    /// Validates that an offset lies on a valid UTF-8 scalar boundary within bounds.
    pub fn validate_offset(&self, offset: TextOffset) -> Result<(), PetuniaError> {
        let idx = offset.byte_index();
        if idx > self.content.len() {
            return Err(PetuniaError::invalid_input(format!(
                "offset {} is past text end {}",
                idx,
                self.content.len()
            )));
        }
        if !self.content.is_char_boundary(idx) {
            return Err(PetuniaError::invalid_input(format!(
                "offset {} is not a Unicode char boundary",
                idx
            )));
        }
        Ok(())
    }

    /// Validates a range of offsets.
    pub fn validate_range(&self, range: Range<TextOffset>) -> Result<(), PetuniaError> {
        self.validate_offset(range.start)?;
        self.validate_offset(range.end)?;
        if range.start > range.end {
            return Err(PetuniaError::invalid_input(format!(
                "invalid inverted range {}..{}",
                range.start.0, range.end.0
            )));
        }
        Ok(())
    }

    /// Inserts text at a valid boundary offset, adjusting style runs accordingly.
    pub fn insert_str(&mut self, at: TextOffset, text: &str) -> Result<(), PetuniaError> {
        self.validate_offset(at)?;
        let idx = at.byte_index();
        let inserted_len = text.len();

        self.content.insert_str(idx, text);

        // Adjust char runs
        for run in &mut self.char_runs {
            if run.end.0 >= idx {
                run.end.0 += inserted_len;
            }
            if run.start.0 > idx {
                run.start.0 += inserted_len;
            }
        }
        if self.char_runs.is_empty() {
            self.char_runs.push(CharacterStyleRun {
                start: TextOffset::new(0),
                end: TextOffset::new(self.content.len()),
                style: CharacterStyle::default(),
            });
        }

        // Adjust para runs
        for run in &mut self.para_runs {
            if run.end.0 >= idx {
                run.end.0 += inserted_len;
            }
            if run.start.0 > idx {
                run.start.0 += inserted_len;
            }
        }
        if self.para_runs.is_empty() {
            self.para_runs.push(ParagraphStyleRun {
                start: TextOffset::new(0),
                end: TextOffset::new(self.content.len()),
                style: ParagraphStyle::default(),
            });
        }

        Ok(())
    }

    /// Deletes a range of text, shifting style runs and removing empty runs.
    pub fn delete_range(&mut self, range: Range<TextOffset>) -> Result<(), PetuniaError> {
        self.validate_range(range.clone())?;
        let start = range.start.byte_index();
        let end = range.end.byte_index();
        let deleted_len = end - start;

        self.content.drain(start..end);

        for run in &mut self.char_runs {
            if run.start.0 >= end {
                run.start.0 -= deleted_len;
                run.end.0 -= deleted_len;
            } else if run.end.0 > start {
                run.end.0 = (run.end.0.saturating_sub(deleted_len)).max(run.start.0);
            }
        }
        self.char_runs.retain(|r| r.end.0 > r.start.0);

        for run in &mut self.para_runs {
            if run.start.0 >= end {
                run.start.0 -= deleted_len;
                run.end.0 -= deleted_len;
            } else if run.end.0 > start {
                run.end.0 = (run.end.0.saturating_sub(deleted_len)).max(run.start.0);
            }
        }
        self.para_runs.retain(|r| r.end.0 > r.start.0);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_delete_maintains_valid_char_boundaries() {
        let id = TextStoryId::new(1);
        let mut story = TextStory::with_content(id, "Olá Mundo!");
        assert_eq!(story.content, "Olá Mundo!");

        // 'á' is 2 bytes in UTF-8 (indices 2..4)
        assert!(story.validate_offset(TextOffset::new(0)).is_ok());
        assert!(story.validate_offset(TextOffset::new(2)).is_ok());
        assert!(story.validate_offset(TextOffset::new(3)).is_err()); // middle of 'á'
        assert!(story.validate_offset(TextOffset::new(4)).is_ok());

        // Insert at boundary
        story
            .insert_str(TextOffset::new(4), " Belo")
            .expect("insert at valid boundary");
        assert_eq!(story.content, "Olá Belo Mundo!");

        // Delete " Belo"
        story
            .delete_range(TextOffset::new(4)..TextOffset::new(9))
            .expect("delete valid range");
        assert_eq!(story.content, "Olá Mundo!");
    }
}
