# 09.9.4 — Caret, Selection, Hit Testing, IME, Clipboard & Text Editing Commands

# Caret position

TextPosition = logical text index + affinity (upstream/downstream) sufficient to resolve bidi/line boundary ambiguity.

# Hit test

Given point in text object, layout returns nearest caret position, line, cluster and optional glyph. Selection drag maps continuously without splitting grapheme cluster unless platform convention permits codepoint-level advanced movement.

# Horizontal movement

Left/right follows visual caret order for bidi according platform text convention. Ctrl/Option word movement uses Unicode word boundaries and locale rules.

# Vertical movement

Preserve preferred x across lines. Home/End behavior platform/profile-defined; document shortcuts can expose logical line start/end alternatives.

# Selection

Range stores logical endpoints + direction. Rendering can produce multiple visual rectangles for bidi range.

# IME

Composition range is temporary overlay over canonical text edit state. Preedit attributes rendered; final commit creates EditTextRange command. Document/selection change while composing follows safe finalize/cancel platform convention.

# Clipboard

Plain text always; rich Petunia text/style fragment for internal copy; optional HTML/RTF adapter if fidelity/security justified. Pasting sanitizes external markup and maps styles explicitly.

# Commands

InsertText, DeleteRange, ReplaceRange, ApplyCharacterStyle, ApplyParagraphStyle and story/frame linking. Typing/coalescing rules operate at transaction layer.

# Undo

Consecutive typing coalesces until caret move/style change/time/context break. IME composition produces one committed history entry.

# Tests

Ligature caret, bidi range, emoji deletion as grapheme, IME Japanese/Chinese/Korean, style-preserving copy/paste and undo coalescing.