# 09.9.4 — Caret, Selection, Hit Testing, IME, Clipboard, Bidi Editing & Text Commands

# Hit test

Given document point in TextFrame, map through transform -> line -> visual glyph cluster -> logical caret position + affinity. Clicking ligature chooses nearest valid cluster/caret stop.

# Caret

CaretPosition = logical index + affinity + frame context. Height/orientation derived from line/glyph metrics. Blink is view state.

# Selection

Logical range(s) rendered as visual rectangles across bidi runs/lines. Shift navigation extends logically according platform behavior.

# Navigation

Left/right visual caret movement; Ctrl/Option word movement per platform profile; up/down preserves desired x. Home/End line vs story behavior configurable/platform standard.

# IME

Composition range and attributes are staged presentation state overlaying story snapshot. commitString generates semantic EditText command; preedit changes never enter history. Candidate window anchored to transformed caret screen coordinates.

# Clipboard

Copy exports plain text + rich Petunia transfer + HTML/RTF candidate where useful. Paste into text sanitizes/imports styles per Paste/Paste Without Formatting action.

# Commands

InsertText, DeleteRange, ReplaceRange, ApplyCharacterStyle, ApplyParagraphStyle, SplitParagraph, JoinParagraph, InsertField. Typing transaction coalescing uses text context token.

# Accessibility

Expose text value, selection and caret through Qt accessibility APIs for active editor where feasible; screen readers can navigate text rather than canvas pixels.

# Tests

IME composition cancellation, ligatures, bidi arrowing, selections across frames, copy/paste multilingual and transformed text.