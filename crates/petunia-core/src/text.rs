//! Authorial text model: Unicode intent, runs and flow.
//!
//! The Core keeps Unicode plus typographic intent. Glyph IDs,
//! positions, line boxes and atlas caches are Engine/Render derived
//! state and never persist here.

use crate::error::{CoreError, Result};
use crate::id::{ObjectId, StyleId};
use crate::math::{Point, Size2};
use serde::{Deserialize, Serialize};

/// Byte offsets into the object string. Both ends must sit on UTF-8
/// boundaries with `start <= end`; cursor motion works on graphemes
/// at the Engine/UI layers, never raw bytes here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextRange {
    pub start: u32,
    pub end: u32,
}

impl TextRange {
    /// Validate against the owning string.
    pub fn new(start: u32, end: u32, text: &str) -> Result<Self> {
        if start > end {
            return Err(CoreError::InvariantViolation(format!(
                "text range start beyond end: {start}..{end}"
            )));
        }
        for boundary in [start, end] {
            if !(boundary as usize <= text.len() && text.is_char_boundary(boundary as usize)) {
                return Err(CoreError::InvariantViolation(format!(
                    "text offset outside UTF-8 boundary: {boundary}"
                )));
            }
        }
        Ok(Self { start, end })
    }

    /// True when `other` starts where this range ends without overlap.
    #[must_use]
    pub fn abuts(self, other: TextRange) -> bool {
        self.end <= other.start
    }
}

/// Reference to a character style living in the style registry.
/// Gaps between runs fall back to the object/document default; inline
/// markup inside the native string is never a representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterStyleRef {
    pub style: StyleId,
}

/// One styled span. Runs stay ordered and non-overlapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextRun {
    pub range: TextRange,
    pub style: CharacterStyleRef,
}

/// One paragraph span over a paragraph style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParagraphRun {
    pub range: TextRange,
    pub style: ParagraphStyleRef,
}

/// Reference to a paragraph style in the style registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParagraphStyleRef {
    pub style: StyleId,
}

/// Typographic font intent. Resolution to an available file/face is
/// derived state; a missing font never rewrites the reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontRef {
    pub family: String,
    pub style_name: Option<String>,
    pub resource: Option<crate::id::ResourceId>,
    pub axes: Vec<FontAxis>,
}

/// One variable-font axis coordinate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontAxis {
    pub tag: String,
    pub value: f32,
}

impl FontRef {
    /// The family must be named; axis tags must be non-empty with
    /// finite values.
    pub fn new(
        family: impl Into<String>,
        style_name: Option<String>,
        resource: Option<crate::id::ResourceId>,
        axes: Vec<FontAxis>,
    ) -> Result<Self> {
        let candidate = Self {
            family: family.into(),
            style_name,
            resource,
            axes,
        };
        candidate.validate()?;
        Ok(candidate)
    }

    /// Re-check the construction invariants on any instance,
    /// including deserialized ones.
    pub fn validate(&self) -> Result<()> {
        if self.family.trim().is_empty() {
            return Err(CoreError::InvariantViolation(
                "font family must be named".to_string(),
            ));
        }
        for axis in &self.axes {
            if axis.tag.trim().is_empty() || !axis.value.is_finite() {
                return Err(CoreError::InvariantViolation(format!(
                    "invalid font axis rejected: {axis:?}"
                )));
            }
        }
        Ok(())
    }
}

/// Text sizing container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TextContainer {
    /// Grows with content; no mandatory wrap frame.
    Artistic,
    /// Wrapped inside a frame with an overflow policy.
    Frame(TextFrameSpec),
    /// Set along an authorial path or shape.
    OnPath(TextPathRef),
}

/// Frame geometry plus overflow policy.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TextFrameSpec {
    pub size: Size2,
    pub overflow: TextOverflow,
}

/// What happens past the frame edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TextOverflow {
    /// Keep overflowing (layout still reports it).
    #[default]
    Overflow,
    /// Clip rendering to the frame.
    Clip,
}

/// Reference to an authorial path/shape plus placement parameters.
/// Glyph placement along the curve is derived.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TextPathRef {
    pub target: ObjectId,
    pub start_offset: f64,
}

impl TextPathRef {
    /// The start offset must be finite.
    pub fn new(target: ObjectId, start_offset: f64) -> Result<Self> {
        if !start_offset.is_finite() {
            return Err(CoreError::InvariantViolation(format!(
                "non-finite text path offset rejected: {start_offset}"
            )));
        }
        Ok(Self {
            target,
            start_offset,
        })
    }
}

/// Single-direction link to the next frame. The previous frame is
/// derived by query; storing both would create divergent sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextFlow {
    pub next: Option<ObjectId>,
}

/// Validate a set of `(owner, next)` flow links: no self-links and no
/// cycles. Target type checks belong to document validation.
pub fn validate_flow_links(links: &[(ObjectId, Option<ObjectId>)]) -> Result<()> {
    use std::collections::{HashMap, HashSet};

    let graph: HashMap<ObjectId, ObjectId> = links
        .iter()
        .filter_map(|(owner, next)| next.map(|target| (*owner, target)))
        .collect();
    for (owner, _) in links {
        if graph.get(owner) == Some(owner) {
            return Err(CoreError::InvariantViolation(format!(
                "text flow links to itself: {owner}"
            )));
        }
        let mut seen = HashSet::from([*owner]);
        let mut cursor = *owner;
        while let Some(next) = graph.get(&cursor) {
            if !seen.insert(*next) {
                return Err(CoreError::InvariantViolation(
                    "text flow cycle detected".to_string(),
                ));
            }
            cursor = *next;
        }
    }
    Ok(())
}

/// Authorial text object: Unicode plus runs, paragraphs, container
/// and flow. Font sizes stay in document units; the scene transform
/// stays on the node and never rewrites glyphs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextObject {
    pub text: String,
    pub runs: Vec<TextRun>,
    pub paragraphs: Vec<ParagraphRun>,
    pub container: TextContainer,
    pub flow: TextFlow,
}

impl TextObject {
    /// Runs must be ordered, non-overlapping and valid against `text`.
    /// Paragraph spans follow the same range rules.
    pub fn new(
        text: String,
        runs: Vec<TextRun>,
        paragraphs: Vec<ParagraphRun>,
        container: TextContainer,
        flow: TextFlow,
    ) -> Result<Self> {
        Self::validate(&text, &runs, &paragraphs)?;
        Ok(Self {
            text,
            runs,
            paragraphs,
            container,
            flow,
        })
    }

    /// Re-check the construction invariants on any instance,
    /// including deserialized ones.
    pub fn validate(text: &str, runs: &[TextRun], paragraphs: &[ParagraphRun]) -> Result<()> {
        let mut cursor = 0u32;
        for run in runs {
            TextRange::new(run.range.start, run.range.end, text)?;
            if run.range.start < cursor {
                return Err(CoreError::InvariantViolation(
                    "text runs must be ordered and non-overlapping".to_string(),
                ));
            }
            cursor = run.range.end;
        }
        for paragraph in paragraphs {
            TextRange::new(paragraph.range.start, paragraph.range.end, text)?;
        }
        Ok(())
    }

    /// Position where point text starts; frames and paths resolve
    /// their own origins.
    #[must_use]
    pub fn anchor(&self) -> Point {
        Point::new(0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style_ref() -> CharacterStyleRef {
        CharacterStyleRef {
            style: StyleId::new_v4(),
        }
    }

    #[test]
    fn ranges_validate_utf8_boundaries_and_order() {
        let text = "héllo";
        assert!(TextRange::new(0, 6, text).is_ok());
        assert!(TextRange::new(0, 2, text).is_err());
        assert!(TextRange::new(1, 2, text).is_err());
        assert!(TextRange::new(4, 2, text).is_err());
    }

    #[test]
    fn runs_must_not_overlap() {
        let text = "abcdef".to_string();
        let good = vec![
            TextRun {
                range: TextRange::new(0, 2, &text).expect("valid"),
                style: style_ref(),
            },
            TextRun {
                range: TextRange::new(2, 6, &text).expect("valid"),
                style: style_ref(),
            },
        ];
        assert!(TextObject::new(
            text.clone(),
            good,
            Vec::new(),
            TextContainer::Artistic,
            TextFlow { next: None },
        )
        .is_ok());
        let overlapping = vec![
            TextRun {
                range: TextRange::new(0, 4, &text).expect("valid"),
                style: style_ref(),
            },
            TextRun {
                range: TextRange::new(2, 6, &text).expect("valid"),
                style: style_ref(),
            },
        ];
        assert!(TextObject::new(
            text,
            overlapping,
            Vec::new(),
            TextContainer::Artistic,
            TextFlow { next: None },
        )
        .is_err());
    }

    #[test]
    fn flow_rejects_self_links_and_cycles() {
        let a = ObjectId::new_v4();
        let b = ObjectId::new_v4();
        assert!(validate_flow_links(&[(a, Some(b)), (b, None)]).is_ok());
        assert!(validate_flow_links(&[(a, Some(a))]).is_err());
        assert!(validate_flow_links(&[(a, Some(b)), (b, Some(a))]).is_err());
    }

    #[test]
    fn font_ref_requires_family_and_finite_axes() {
        assert!(FontRef::new("Inter", None, None, Vec::new()).is_ok());
        assert!(FontRef::new("  ", None, None, Vec::new()).is_err());
        assert!(FontRef::new(
            "Inter",
            None,
            None,
            vec![FontAxis {
                tag: String::new(),
                value: 400.0,
            }],
        )
        .is_err());
    }

    #[test]
    fn text_serialization_round_trip() {
        let text = "Hi".to_string();
        let object = TextObject::new(
            text.clone(),
            vec![TextRun {
                range: TextRange::new(0, 2, &text).expect("valid"),
                style: style_ref(),
            }],
            Vec::new(),
            TextContainer::Artistic,
            TextFlow { next: None },
        )
        .expect("valid");
        let json = serde_json::to_string(&object).expect("serializable");
        let back: TextObject = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(back, object);
    }
}
