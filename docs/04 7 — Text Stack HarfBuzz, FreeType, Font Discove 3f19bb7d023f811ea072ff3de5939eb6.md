# 04.7 — Text Stack: HarfBuzz, FreeType, Font Discovery, ICU/Unicode Services & Hyphenation

# Shaping

HarfBuzz receives Unicode runs, direction, script/language, font face and OpenType features. Output glyph IDs/positions are derived layout.

# Font loading

FreeType validates/loads font resources and exposes outlines/metrics. Host font discovery uses platform/Qt adapters but maps to Petunia FontFaceId descriptors.

# Unicode

Use robust Unicode services for grapheme, bidi, line-break and normalization behavior. ICU is a candidate where Qt/stdlib facilities are insufficient; exact dependency decided by text-engine ADR.

# Hyphenation

Pluggable language dictionary/service with explicit locale. Missing dictionary degrades to no automatic hyphenation, not invented rules.

# Font fallback

Fallback resolution is deterministic enough for session/platform and explicitly reported. Requested family metadata remains canonical even when fallback renders.

# Variable fonts

Axes and named instances preserved semantically; UI exposes supported axes with ranges/default.

# Font security

Fonts are hostile binary inputs. Bounds, malformed corpus and optional helper-process isolation are evaluated.

# Export

Font embedding/subsetting policy is adapter-specific and must respect licensing bits/requirements.