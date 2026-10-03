//! Toolkit-neutral UTF-8/grapheme editing. IME preedit is a derived display
//! value; only committed content can become a document Command.
use crate::TextRenderError;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

const MAX_TEXT_BYTES: usize = 64 * 1024;
const MAX_UNDO_BYTES: usize = 4 * 1024 * 1024;
const MAX_UNDO: usize = 64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextSelection {
    pub anchor: usize,
    pub focus: usize,
}
impl TextSelection {
    pub fn range(self) -> Range<usize> {
        self.anchor.min(self.focus)..self.anchor.max(self.focus)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct TextEditBuffer {
    content: String,
    selection: TextSelection,
    preedit: Option<(String, Option<(usize, usize)>)>,
    undo: Vec<(String, TextSelection)>,
    redo: Vec<(String, TextSelection)>,
}
impl TextEditBuffer {
    pub fn new(content: String) -> Result<Self, TextRenderError> {
        if content.len() > MAX_TEXT_BYTES {
            return Err(TextRenderError::Limit("text editor bytes"));
        }
        Ok(Self {
            content,
            selection: TextSelection::default(),
            preedit: None,
            undo: Vec::new(),
            redo: Vec::new(),
        })
    }
    pub fn content(&self) -> &str {
        &self.content
    }
    pub fn selection(&self) -> TextSelection {
        self.selection
    }
    pub fn set_selection(&mut self, anchor: usize, focus: usize) {
        self.preedit = None;
        self.selection = TextSelection {
            anchor: floor_boundary(&self.content, anchor),
            focus: floor_boundary(&self.content, focus),
        };
    }
    pub fn select_all(&mut self) {
        self.set_selection(0, self.content.len());
    }
    pub fn has_preedit(&self) -> bool {
        self.preedit.is_some()
    }
    pub fn set_preedit(
        &mut self,
        text: String,
        cursor: Option<(usize, usize)>,
    ) -> Result<(), TextRenderError> {
        if self
            .content
            .len()
            .saturating_sub(self.selection.range().len())
            .saturating_add(text.len())
            > MAX_TEXT_BYTES
        {
            return Err(TextRenderError::Limit("IME preedit bytes"));
        }
        self.preedit = (!text.is_empty()).then_some((text, cursor));
        Ok(())
    }
    pub fn cancel_preedit(&mut self) {
        self.preedit = None;
    }
    pub fn display_content(&self) -> String {
        let Some((text, _)) = &self.preedit else {
            return self.content.clone();
        };
        let mut result = self.content.clone();
        result.replace_range(self.selection.range(), text);
        result
    }
    pub fn display_selection(&self) -> TextSelection {
        let Some((text, cursor)) = &self.preedit else {
            return self.selection;
        };
        let start = self.selection.range().start;
        let (a, b) = cursor.unwrap_or((text.len(), text.len()));
        TextSelection {
            anchor: start + floor_boundary(text, a),
            focus: start + floor_boundary(text, b),
        }
    }
    pub fn preedit_range(&self) -> Option<Range<usize>> {
        self.preedit.as_ref().map(|(text, _)| {
            let start = self.selection.range().start;
            start..start + text.len()
        })
    }
    pub fn insert(&mut self, text: &str) -> Result<bool, TextRenderError> {
        let range = self.selection.range();
        if self
            .content
            .len()
            .saturating_sub(range.len())
            .saturating_add(text.len())
            > MAX_TEXT_BYTES
        {
            return Err(TextRenderError::Limit("text editor bytes"));
        }
        self.preedit = None;
        if self.content.get(range.clone()) == Some(text) {
            return Ok(false);
        }
        self.remember();
        let end = range.start + text.len();
        self.content.replace_range(range, text);
        // Inserting a combining mark/ZWJ can join its neighbouring grapheme.
        let end = ceil_boundary(&self.content, end);
        self.selection = TextSelection {
            anchor: end,
            focus: end,
        };
        Ok(true)
    }
    pub fn delete(&mut self, backward: bool) -> Result<bool, TextRenderError> {
        if self.has_preedit() {
            self.cancel_preedit();
            return Ok(false);
        }
        if self.selection.anchor == self.selection.focus {
            let at = self.selection.focus;
            let next = if backward {
                previous_boundary(&self.content, at)
            } else {
                next_boundary(&self.content, at)
            };
            self.selection.anchor = next;
        }
        self.insert("")
    }
    pub fn move_horizontal(&mut self, forward: bool, extend: bool) {
        self.cancel_preedit();
        let range = self.selection.range();
        let at = if !extend && !range.is_empty() {
            if forward {
                range.end
            } else {
                range.start
            }
        } else if forward {
            next_boundary(&self.content, self.selection.focus)
        } else {
            previous_boundary(&self.content, self.selection.focus)
        };
        if !extend {
            self.selection.anchor = at;
        }
        self.selection.focus = at;
    }
    pub fn move_to(&mut self, at: usize, extend: bool) {
        let at = floor_boundary(&self.content, at);
        self.set_selection(if extend { self.selection.anchor } else { at }, at);
    }
    pub fn undo(&mut self) -> bool {
        self.cancel_preedit();
        let Some((content, selection)) = self.undo.pop() else {
            return false;
        };
        self.redo.push((
            std::mem::replace(&mut self.content, content),
            self.selection,
        ));
        self.selection = selection;
        true
    }
    pub fn redo(&mut self) -> bool {
        self.cancel_preedit();
        let Some((content, selection)) = self.redo.pop() else {
            return false;
        };
        self.undo.push((
            std::mem::replace(&mut self.content, content),
            self.selection,
        ));
        self.selection = selection;
        true
    }
    fn remember(&mut self) {
        self.redo.clear();
        while !self.undo.is_empty()
            && (self.undo.len() >= MAX_UNDO
                || self.undo.iter().map(|(text, _)| text.len()).sum::<usize>() + self.content.len()
                    > MAX_UNDO_BYTES)
        {
            self.undo.remove(0);
        }
        self.undo.push((self.content.clone(), self.selection));
    }
}
pub(crate) fn boundaries(text: &str) -> impl Iterator<Item = usize> + '_ {
    text.grapheme_indices(true)
        .map(|(index, _)| index)
        .chain(std::iter::once(text.len()))
}
fn floor_boundary(text: &str, at: usize) -> usize {
    boundaries(text)
        .take_while(|&index| index <= at)
        .last()
        .unwrap_or(0)
}
fn ceil_boundary(text: &str, at: usize) -> usize {
    boundaries(text)
        .find(|&index| index >= at)
        .unwrap_or(text.len())
}
fn previous_boundary(text: &str, at: usize) -> usize {
    boundaries(text)
        .take_while(|&index| index < at)
        .last()
        .unwrap_or(0)
}
fn next_boundary(text: &str, at: usize) -> usize {
    boundaries(text)
        .find(|&index| index > at)
        .unwrap_or(text.len())
}
