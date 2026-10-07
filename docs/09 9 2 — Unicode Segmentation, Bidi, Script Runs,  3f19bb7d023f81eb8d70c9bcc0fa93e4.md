# 09.9.2 — Unicode Segmentation, Bidi, Script Runs, Font Fallback & HarfBuzz Shaping Contract

# Pipeline

Text range -> paragraph direction -> Unicode bidi levels -> script/language segmentation -> font fallback segmentation -> HarfBuzz shaping -> glyph runs.

# Bidi

Use Unicode Bidirectional Algorithm through vetted library/service. Canonical text remains logical order. Visual ordering exists only in layout result.

# Script/language

Detect script with explicit user language/style override. Common/Inherited characters associate with neighboring runs according Unicode rules.

# Font fallback

Resolver inputs requested family/style/axes, codepoints/script/language and platform/user font inventory. Output deterministic ordered FontFaceId runs for current environment. Missing requested font metadata remains preserved.

# HarfBuzz

Buffer sets direction, script, language, cluster level and OpenType features. Variation coordinates applied to font. Shaping output stores glyph IDs, advances, offsets and cluster mapping.

# Cluster mapping

Never assume one glyph per codepoint. Ligatures, marks, reordered glyphs and emoji sequences produce cluster map used for caret/hit testing.

# Feature policy

Kerning/ligatures defaults follow font/platform typography policy; user style can toggle named OpenType features. Unsupported feature retained as style request where appropriate.

# Fallback diagnostics

Layout can report substituted faces/ranges so Missing Fonts UI and preflight identify exact affected content.

# Tests

Arabic joining/bidi, Devanagari, Thai, CJK, combining accents, emoji family, Latin ligatures, mixed RTL/LTR numbers and variable font shaping fixtures.