# 19.07 — Typography & Paragraph Panels Specification

# Typography

PanelId ptnd.panel.typography. Family/style search, variable axes, size, leading, tracking, kerning, baseline, scale, language, OpenType features and decorations.

# Font picker

Virtualized searchable family list with recent/favorites, missing-font state and variable-font badges. Preview does not block text input.

# Paragraph

PanelId ptnd.panel.paragraph. Alignment/justification, spacing before/after, indents, tabs, hyphenation, keep rules and baseline-grid alignment.

# Styles

CharacterStyle/ParagraphStyle linkage visible with override badges. Clear Overrides and Redefine actions explicit.

# Multi-selection

Text ranges/objects report Mixed without destroying independent attributes.

# Accessibility

OpenType toggles named; variable axis values numeric; tabs editor has list/table keyboard alternative.

# Performance

Opening font dropdown must not synchronously render every font preview. Font metadata/preview caches lazy.

# Tests

Variable fonts, missing fonts, bidi paragraphs, mixed styles, long font lists and keyboard editing.