# G034 — Unicode, Bidi, Font Fallback & HarfBuzz Shaping Engine

# Goal

Shape multilingual text correctly.

# Depends

G033, text stack 04.7, 09.9.2.

# Primary

systems-architect/editor-engineer.

# Deliverables

Bidi/script/language segmentation; FontFace resolver; HarfBuzz shaping adapter; glyph cluster mapping; variable font coordinates; fallback diagnostics/cache.

# Acceptance

Arabic/Hebrew/Indic/CJK/emoji fixtures produce stable glyph runs/caret mappings and missing-glyph diagnostics.

# Tests

Pinned fonts, shaping golden vectors, fallback, variable axes, cache invalidation and performance.