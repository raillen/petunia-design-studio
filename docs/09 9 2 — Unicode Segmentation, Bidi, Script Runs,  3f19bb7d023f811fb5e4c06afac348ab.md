# 09.9.2 — Unicode Segmentation, Bidi, Script Runs, Font Fallback & HarfBuzz Shaping Contract

# Pipeline

Paragraph text -> bidi analysis -> script/language segmentation -> style/font candidate runs -> fallback resolution -> HarfBuzz shaping -> visual glyph runs.

# Bidi

Use Unicode Bidirectional Algorithm via proven library/Qt/ICU adapter. Logical storage order remains canonical. Visual ordering is derived. Caret affinity distinguishes ambiguous bidi boundaries.

# Grapheme

Cursor left/right and deletion operate extended grapheme clusters according Unicode version pinned by dependency. Word navigation follows locale/platform policy separately.

# Script/language

Language tag from style/paragraph/document; script auto-detected with Common/Inherited handling. HarfBuzz buffer receives direction/script/language explicitly.

# Font fallback

Requested font face first; for missing glyphs resolve fallback by platform/document policy in minimal spans. Fallback must not permanently rewrite requested font style.

# Shaping

Features include kerning/ligatures and explicit OpenType tags. Variable font coordinates are applied to face instance. Cluster mapping retained from glyphs back to logical text.

# Emoji/color fonts

Support COLR/CPAL, CBDT/CBLC, sbix/SVG font formats according FreeType/backend capability; unsupported color glyph falls back predictably.

# Missing glyph

Use .notdef/tofu diagnostic with source character info; preflight lists unresolved glyphs/font resources.

# Cache

Shape cache key = text slice hash + computed style/font instance + script/language/direction/features. Invalidated only on relevant changes.

# Tests

Arabic, Hebrew mixed bidi, Indic shaping, Thai, CJK, emoji, ligatures, combining marks, fallback across fonts and variable axes.