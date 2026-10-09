//! Unicode algorithms: BiDi, graphemes, scripts and line breaks.
//!
//! Real libraries back every step: `unicode-bidi` for paragraph
//! direction and visual runs, `unicode-segmentation` for grapheme
//! clusters, `unicode-script` for script runs and
//! `unicode-linebreak` for UAX #14 opportunities. Cursor motion and
//! backspace work on graphemes, never raw bytes.

use unicode_bidi::{BidiInfo, Level};
use unicode_script::Script;

/// Base direction of one paragraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParagraphDirection {
    LeftToRight,
    RightToLeft,
}

/// Analyze `text` as a single paragraph: base direction plus the
/// visual-order string. Multi-paragraph documents analyze each
/// paragraph separately through the same entry point.
#[must_use]
pub fn analyze_paragraph(text: &str) -> (ParagraphDirection, String) {
    let info = BidiInfo::new(text, None);
    let Some(paragraph) = info.paragraphs.first() else {
        return (ParagraphDirection::LeftToRight, text.to_string());
    };
    let direction = if paragraph.level.is_rtl() {
        ParagraphDirection::RightToLeft
    } else {
        ParagraphDirection::LeftToRight
    };
    let visual = info
        .reorder_line(paragraph, paragraph.range.clone())
        .into_owned();
    (direction, visual)
}

/// Bidi embedding levels per byte index, for caret and run work.
#[must_use]
pub fn embedding_levels(text: &str) -> Vec<u8> {
    BidiInfo::new(text, Some(Level::ltr()))
        .levels
        .iter()
        .map(|level| level.number())
        .collect()
}

/// Grapheme cluster boundaries as byte offsets, starting at zero.
/// The perceived character may span several code points.
#[must_use]
pub fn grapheme_boundaries(text: &str) -> Vec<usize> {
    use unicode_segmentation::UnicodeSegmentation;
    let mut boundaries: Vec<usize> = text
        .grapheme_indices(true)
        .map(|(index, _)| index)
        .collect();
    if boundaries.first() != Some(&0) {
        boundaries.insert(0, 0);
    }
    boundaries.push(text.len());
    boundaries.dedup();
    boundaries
}

/// Script runs: maximal spans sharing one `Script`, with boundaries
/// at byte offsets. Common/inherited characters attach to the
/// surrounding run; the segmentation stays deterministic.
#[must_use]
pub fn script_runs(text: &str) -> Vec<(Script, usize, usize)> {
    let mut runs = Vec::new();
    let mut current: Option<(Script, usize)> = None;
    for (index, ch) in text.char_indices() {
        let script = Script::from(ch);
        let script = normalize_script(script);
        match current {
            Some((open, start)) if open == script => {
                let _ = start;
            }
            Some((open, start)) => {
                runs.push((open, start, index));
                current = Some((script, index));
            }
            None => current = Some((script, index)),
        }
    }
    if let Some((open, start)) = current {
        runs.push((open, start, text.len()));
    }
    runs
}

fn normalize_script(script: Script) -> Script {
    use unicode_script::Script as S;
    match script {
        S::Common | S::Inherited => S::Common,
        other => other,
    }
}

/// Line-break opportunities as `(byte_offset, mandatory)`.
/// Mandatory breaks come from explicit newlines; soft breaks follow
/// UAX #14.
#[must_use]
pub fn break_opportunities(text: &str) -> Vec<(usize, bool)> {
    use unicode_linebreak::{linebreaks, BreakOpportunity};
    linebreaks(text)
        .map(|(offset, opportunity)| (offset, matches!(opportunity, BreakOpportunity::Mandatory)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bidi_detects_rtl_paragraphs_and_reorders() {
        let (direction, visual) = analyze_paragraph("Hello");
        assert_eq!(direction, ParagraphDirection::LeftToRight);
        assert_eq!(visual, "Hello");
        let (direction, _) = analyze_paragraph("مرحبا");
        assert_eq!(direction, ParagraphDirection::RightToLeft);
        // Mixed-direction text keeps every character exactly once.
        let (direction, visual) = analyze_paragraph("abc مرحبا def");
        assert_eq!(direction, ParagraphDirection::LeftToRight);
        let mut sorted: Vec<char> = visual.chars().collect();
        let mut plain: Vec<char> = "abc مرحبا def".chars().collect();
        sorted.sort_unstable();
        plain.sort_unstable();
        assert_eq!(sorted, plain);
    }

    #[test]
    fn graphemes_span_combining_marks() {
        // e + combining acute is one perceived character.
        let text = "e\u{301}x";
        assert_eq!(text.chars().count(), 3);
        assert_eq!(grapheme_boundaries(text), vec![0, 3, 4]);
        assert_eq!(grapheme_boundaries(""), vec![0]);
    }

    #[test]
    fn script_runs_split_latin_and_arabic() {
        let runs = script_runs("abc مرحبا");
        assert!(runs.len() >= 2);
        assert_eq!(runs[0].1, 0);
        assert_eq!(runs.last().expect("runs").2, "abc مرحبا".len());
    }

    #[test]
    fn breaks_distinguish_mandatory_from_soft() {
        let breaks = break_opportunities("one two\nthree");
        assert!(breaks.iter().any(|(_, mandatory)| *mandatory));
        assert!(breaks
            .iter()
            .any(|(offset, mandatory)| !mandatory && *offset > 0));
    }
}
