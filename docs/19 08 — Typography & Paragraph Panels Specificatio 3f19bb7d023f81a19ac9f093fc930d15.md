# 19.08 — Typography & Paragraph Panels Specification

# Typography

Font family searchable with preview/recent/favorite, style, variable axes, size, leading, tracking, kerning, baseline shift, color and OpenType feature groups.

# Paragraph

alignment/justification, indents, spacing before/after, tabs, hyphenation/language, keep options, baseline grid, list features if implemented.

# Target

Caret selection, text object, multi-text selection or current text defaults. Scope clearly identified.

# Styles

Character/Paragraph style selector with override indicator, Clear Overrides, Redefine/Create Style.

# Font missing

Requested family shown with warning/fallback, not replaced label silently.

# Performance

Font list virtualized/indexed; preview rendering async/cached.

# Accessibility/tests

Font search keyboard-first; OpenType features named; variable axes numeric. Test bidi text, mixed styles, missing font, style overrides and IME while panel changes.