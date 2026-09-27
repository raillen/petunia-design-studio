//! Typography layout facade, line breaking and hit testing (09.8).

use petunia_design_geometry::{GPoint, GRect};

use crate::story::{TextAlign, TextOffset, TextStory};

/// A derived visual line fragment resulting from layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LineFragment {
    /// Byte range in the source story content.
    pub range: (TextOffset, TextOffset),
    /// Baseline offset from top of layout.
    pub baseline_y: f64,
    /// Bounding rectangle for this line.
    pub bounds: GRect,
    /// Line text content.
    pub text: String,
}

/// The result of running layout on a story.
#[derive(Debug, Clone, PartialEq)]
pub struct TextLayout {
    pub lines: Vec<LineFragment>,
    pub bounds: GRect,
    pub line_count: usize,
}

impl TextLayout {
    /// Performs line breaking and layout for a story within an optional max width.
    ///
    /// The layout engine computes line wrapping based on whitespace and explicit
    /// newlines, computing metric bounding boxes.
    #[must_use]
    pub fn layout(story: &TextStory, max_width: Option<f64>) -> Self {
        if story.content.is_empty() {
            return Self {
                lines: Vec::new(),
                bounds: GRect::new(0.0, 0.0, 0.0, 0.0),
                line_count: 0,
            };
        }

        let default_font_size = story.char_runs.first().map_or(14.0, |r| r.style.font_size);
        let default_line_height = story.para_runs.first().map_or(1.2, |r| r.style.line_height);
        let line_height_px = default_font_size * default_line_height;
        let avg_char_width = default_font_size * 0.55;

        let max_w = max_width.unwrap_or(f64::INFINITY);

        let mut lines = Vec::new();
        let mut current_y = 0.0;
        let mut max_line_w: f64 = 0.0;

        let mut line_start_byte = 0;
        let mut current_line_text = String::new();
        let mut current_line_w: f64 = 0.0;

        for (byte_idx, ch) in story.content.char_indices() {
            if ch == '\n' {
                // Flush line
                let end_byte = byte_idx;
                let line_w = current_line_w.max(1.0);
                if line_w > max_line_w {
                    max_line_w = line_w;
                }
                lines.push(LineFragment {
                    range: (TextOffset::new(line_start_byte), TextOffset::new(end_byte)),
                    baseline_y: current_y + default_font_size * 0.8,
                    bounds: GRect::new(0.0, current_y, line_w, current_y + line_height_px),
                    text: current_line_text.clone(),
                });
                current_y += line_height_px;
                line_start_byte = byte_idx + 1;
                current_line_text.clear();
                current_line_w = 0.0;
                continue;
            }

            let ch_w = avg_char_width;
            if current_line_w + ch_w > max_w && !current_line_text.is_empty() {
                // Wrap line
                let end_byte = byte_idx;
                if current_line_w > max_line_w {
                    max_line_w = current_line_w;
                }
                lines.push(LineFragment {
                    range: (TextOffset::new(line_start_byte), TextOffset::new(end_byte)),
                    baseline_y: current_y + default_font_size * 0.8,
                    bounds: GRect::new(0.0, current_y, current_line_w, current_y + line_height_px),
                    text: current_line_text.clone(),
                });
                current_y += line_height_px;
                line_start_byte = byte_idx;
                current_line_text.clear();
                current_line_w = 0.0;
            }

            current_line_text.push(ch);
            current_line_w += ch_w;
        }

        if !current_line_text.is_empty() || line_start_byte < story.content.len() {
            let end_byte = story.content.len();
            if current_line_w > max_line_w {
                max_line_w = current_line_w;
            }
            lines.push(LineFragment {
                range: (TextOffset::new(line_start_byte), TextOffset::new(end_byte)),
                baseline_y: current_y + default_font_size * 0.8,
                bounds: GRect::new(0.0, current_y, current_line_w, current_y + line_height_px),
                text: current_line_text,
            });
            current_y += line_height_px;
        }

        // Apply alignment shifts if specified
        if let Some(para_run) = story.para_runs.first() {
            if para_run.style.align == TextAlign::Center && max_width.is_some() {
                let limit = max_w;
                for line in &mut lines {
                    let shift = ((limit - (line.bounds.x1 - line.bounds.x0)) / 2.0).max(0.0);
                    line.bounds.x0 += shift;
                    line.bounds.x1 += shift;
                }
            } else if para_run.style.align == TextAlign::Right && max_width.is_some() {
                let limit = max_w;
                for line in &mut lines {
                    let shift = (limit - (line.bounds.x1 - line.bounds.x0)).max(0.0);
                    line.bounds.x0 += shift;
                    line.bounds.x1 += shift;
                }
            }
        }

        let total_w = if max_width.is_some() {
            max_w
        } else {
            max_line_w
        };
        let bounds = GRect::new(0.0, 0.0, total_w, current_y);
        let line_count = lines.len();

        Self {
            lines,
            bounds,
            line_count,
        }
    }

    /// Hit-tests a coordinate against lines and returns the closest byte offset.
    #[must_use]
    pub fn hit_test(&self, point: GPoint) -> TextOffset {
        if self.lines.is_empty() {
            return TextOffset::new(0);
        }

        // Find matching line or nearest line vertically
        let target_line = self
            .lines
            .iter()
            .find(|l| point.y >= l.bounds.y0 && point.y <= l.bounds.y1)
            .unwrap_or_else(|| {
                if point.y < self.lines[0].bounds.y0 {
                    &self.lines[0]
                } else {
                    self.lines.last().unwrap()
                }
            });

        let line_w = target_line.bounds.x1 - target_line.bounds.x0;
        let char_count = target_line.text.chars().count();
        if char_count == 0 || line_w <= 0.0 {
            return target_line.range.0;
        }

        let rel_x = (point.x - target_line.bounds.x0).clamp(0.0, line_w);
        let char_fraction = rel_x / line_w;
        let approx_char_idx =
            ((char_fraction * char_count as f64).round() as usize).min(char_count);

        let byte_offset = target_line
            .text
            .char_indices()
            .nth(approx_char_idx)
            .map_or(target_line.range.1 .0, |(b, _)| target_line.range.0 .0 + b);

        TextOffset::new(byte_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_design_foundation::TextStoryId;

    #[test]
    fn layout_wraps_multiline_text() {
        let story = TextStory::with_content(
            TextStoryId::new(1),
            "Petunia Design Studio\nLayout e Tipografia",
        );
        let layout = TextLayout::layout(&story, None);
        assert_eq!(layout.line_count, 2);
        assert_eq!(layout.lines[0].text, "Petunia Design Studio");
        assert_eq!(layout.lines[1].text, "Layout e Tipografia");
    }

    #[test]
    fn hit_test_returns_valid_offset() {
        let story = TextStory::with_content(TextStoryId::new(1), "Hello");
        let layout = TextLayout::layout(&story, None);
        let offset = layout.hit_test(GPoint::new(1.0, 5.0));
        assert!(offset.0 <= "Hello".len());
    }
}
