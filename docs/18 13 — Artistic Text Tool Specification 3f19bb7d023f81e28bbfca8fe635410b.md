# 18.13 — Artistic Text Tool Specification

# Identity

ToolId ptnd.tool.text_artistic. Persona Design. Default shortcut T or text-tool group profile.

# Creation

Click empty canvas creates staged TextObject + TextStory at insertion point and immediately enters caret editing. If the user exits without content, the staged object is removed unless explicitly converted to an empty frame by policy.

# Existing text

Click compatible text enters editing at hit-tested grapheme/caret position. Click-drag selects text while in editing state. Escape leaves text-editing mode before leaving the Text Tool.

# Canonical semantics

Artistic text is point-anchored and auto-expands with content. Transform is separate from font size. Scaling handles follow a documented mode: scale object transform by default, while font-size field changes typographic size.

# State machine

Idle -> HoverText/Canvas -> CreateInsertion or EnterTextEdit -> IME/TextEditing -> CommitEdit/Exit -> Idle.

# Context

Family, style, variable font instance/axes shortcut, size, color, leading quick control, alignment, basic OpenType toggles, language and style reference.

# Commands

CreateArtisticText, EditTextRange, SetCharacterProperties, ApplyCharacterStyle, ConvertTextToCurves.

# IME

Composition range is staged presentation state. Commit emits a semantic text edit transaction; cancel composition restores original text.

# Accessibility

Expose editable text role, caret, selection, current paragraph/character properties and validation errors through Qt accessibility. Screen reader editing must not depend on canvas pixel inspection.

# Performance

Typing should remain below perceptible input latency on long stories; layout invalidation is paragraph/run scoped where possible.

# Tests

Empty creation, Unicode graphemes, bidi, variable fonts, IME, transform vs font-size behavior, copy/paste, undo coalescing and save/reopen.