# 18.17 — Artistic Text Tool Specification

# Identity

Text tool submode Artistic.

# Creation

Click creates point text object and enters caret mode. Initial style comes from current text defaults, not arbitrary previous object unless preference says so.

# Editing

TextInputService/IME drives story edits. Clicking existing artistic text enters editing at hit-tested caret; dragging over text selects ranges.

# Resize

Move tool scaling may transform object; dedicated text context size changes font size. UI distinguishes scale transform from typographic size.

# Context

font family/style, variable axes shortcut, size, leading where applicable, color, alignment, tracking and text style.

# Finish

Esc exits caret editing to object selection; second Esc may clear selection. Tool remains active.

# Commands

CreateArtisticText, EditTextRange, ApplyCharacterStyle, ConvertTextToCurves.

# Tests

IME, bidi, variable fonts, missing font, click position, copy/paste, transform vs font size and save/export.