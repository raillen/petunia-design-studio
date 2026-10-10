//! A linked frame chain is evaluated as an immutable text stream.
use super::{evaluate_text, FontRegistry, TextEvaluationError, TextLayout};
use petunia_core::{
    CharacterStyle, ObjectId, ParagraphStyle, StyleRegistry, TextContainer, TextFlow, TextObject,
};
use std::collections::HashMap;

/// Fragments of authorial content concatenate in chain order. A frame consumes
/// whole laid-out lines, then passes its remaining text into the next frame.
/// Source byte offsets in results address that concatenated stream.
pub fn evaluate_text_flow(
    frames: &[(ObjectId, &TextObject)],
    styles: &StyleRegistry,
    fonts: &FontRegistry,
    character: &CharacterStyle,
    paragraph: &ParagraphStyle,
) -> Result<HashMap<ObjectId, TextLayout>, TextEvaluationError> {
    if frames.is_empty() || frames.len() > 64 {
        return Err(TextEvaluationError::Invalid(
            "linked frame count limit".into(),
        ));
    }
    let mut text = String::new();
    let mut runs = Vec::new();
    let mut paragraphs = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (index, (id, object)) in frames.iter().enumerate() {
        if !seen.insert(*id)
            || !matches!(object.container, TextContainer::Frame(_))
            || object.flow.next != frames.get(index + 1).map(|(id, _)| *id)
        {
            return Err(TextEvaluationError::Invalid(
                "invalid linked frame topology/container".into(),
            ));
        }
        TextObject::validate(&object.text, &object.runs, &object.paragraphs)
            .map_err(|error| TextEvaluationError::Invalid(error.to_string()))?;
        let offset = text.len() as u32;
        text.push_str(&object.text);
        if text.len() > 65536 {
            return Err(TextEvaluationError::Invalid(
                "linked story text limit".into(),
            ));
        }
        runs.extend(object.runs.iter().copied().map(|mut run| {
            run.range.start += offset;
            run.range.end += offset;
            run
        }));
        paragraphs.extend(object.paragraphs.iter().copied().map(|mut run| {
            run.range.start += offset;
            run.range.end += offset;
            run
        }));
    }
    let mut consumed = 0usize;
    let mut result = HashMap::new();
    for (id, object) in frames {
        let TextContainer::Frame(frame) = object.container else {
            return Err(TextEvaluationError::Invalid(
                "linked frame container changed".into(),
            ));
        };
        let mut runtime = (*object).clone();
        runtime.text = text[consumed..].to_string();
        runtime.flow = TextFlow { next: None };
        runtime.runs = runs
            .iter()
            .copied()
            .filter_map(|mut run| {
                if run.range.end as usize <= consumed {
                    return None;
                }
                run.range.start = run.range.start.saturating_sub(consumed as u32);
                run.range.end -= consumed as u32;
                Some(run)
            })
            .collect();
        runtime.paragraphs = paragraphs
            .iter()
            .copied()
            .filter_map(|mut run| {
                if run.range.end as usize <= consumed {
                    return None;
                }
                run.range.start = run.range.start.saturating_sub(consumed as u32);
                run.range.end -= consumed as u32;
                Some(run)
            })
            .collect();
        let mut layout = evaluate_text(&runtime, styles, fonts, character, paragraph)?;
        let fits = layout
            .lines
            .iter()
            .take_while(|line| line.baseline <= frame.size.height)
            .count();
        let end = if fits == layout.lines.len() {
            runtime.text.len()
        } else if fits == 0 {
            0
        } else {
            layout.lines[fits].source.start
        };
        layout.lines.truncate(fits);
        layout.glyphs.retain(|glyph| {
            glyph.source_byte < end || (glyph.synthetic && glyph.source_byte == end)
        });
        let placed: std::collections::HashSet<_> = layout
            .glyphs
            .iter()
            .map(|glyph| {
                (
                    glyph.source_byte,
                    glyph.origin.x.to_bits(),
                    glyph.origin.y.to_bits(),
                )
            })
            .collect();
        layout.outlines.retain(|outline| {
            placed.contains(&(
                outline.source_byte,
                outline.origin.x.to_bits(),
                outline.origin.y.to_bits(),
            ))
        });
        for glyph in &mut layout.glyphs {
            glyph.source_byte += consumed;
        }
        for outline in &mut layout.outlines {
            outline.source_byte += consumed;
        }
        for line in &mut layout.lines {
            line.source.start += consumed;
            line.source.end += consumed;
        }
        consumed += end;
        layout.overflow = consumed < text.len() && object.flow.next.is_none();
        result.insert(*id, layout);
    }
    Ok(result)
}
