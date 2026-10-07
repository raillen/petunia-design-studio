# 10.6 — Typography Tools, Artistic Text, Text Frames, Text-on-Path & Layout Editing

# Artistic Text

Point-based text object with origin/transform. Click starts editing. Visual size changes may use font size or object transform according tool behavior; explicit Convert/Scale policy prevents surprising text metrics.

# Frame Text

Text content flows inside frame geometry. Frame stores inset, columns, vertical alignment, auto-size options and link ports. Overflow is derived.

# Editing mode

Text tool click object enters caret editing. Selection is grapheme/logical-text aware. Arrow movement, word/line navigation, Home/End and platform conventions use text engine mappings, not byte indexes.

# IME

Composition is staged UI state; commit produces EditTextCommand. Candidate position tracks caret. Cancel composition restores previous text.

# Formatting

Character: font, family style, variable axes, size, color, baseline, tracking, kerning, language, OpenType.

Paragraph: alignment, justification, leading policy, spacing before/after, indents, tabs, hyphenation, keep options as scoped.

# Styles

CharacterStyle and ParagraphStyle references with local overrides. UI indicates linked style + overrides; Clear Overrides and Redefine Style are actions.

# Text frames flow

Output port click/drag links to another frame or creates new frame. Flow graph cycle invalid. Unlink preserves story content/order according documented rule.

# Text on path

Text object references path or internal path snapshot by semantic relationship. Start/end offsets, baseline offset, reverse/flip. Editing path updates layout non-destructively.

# Shape text

Optional text-in-shape can use frame geometry derived from shape; conversion semantics documented.

# Convert to Curves

Shapes every glyph through font outline engine into groups/paths; warns loss of editability/accessibility/search. Original retained only through undo or optional duplicate command.

# Missing fonts

Never silently rewrite requested font metadata. Fallback is derived/substitution state. Missing Fonts UI offers replacement.

# GUI

Context bar optimized for common character controls; Typography and Paragraph panels for advanced. Text frame ports/overflow icons on canvas. Properties exposes frame geometry/layout.

# Commands

CreateArtisticText, CreateTextFrame, EditTextRange, SetCharacterStyle, SetParagraphStyle, LinkTextFrames, Unlink, SetFrameGeometry, SetTextPath, ConvertTextToCurves.

# Export

PDF/SVG preserve text when target supports fonts/features; otherwise preflight outlines/rasterizes with explicit fidelity.

# Tests

Bidi, Arabic/Indic/CJK, emoji/ZWJ, combining marks, variable fonts, long linked frames, IME, copy/paste, overflow and font substitution.