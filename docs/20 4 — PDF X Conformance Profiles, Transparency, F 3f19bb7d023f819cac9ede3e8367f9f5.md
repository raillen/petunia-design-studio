# 20.4 — PDF/X Conformance Profiles, Transparency, Fonts & External Validation

# Profiles

Candidate targets: PDF/X-1a, PDF/X-3, PDF/X-4 according actual writer capability and market need. Each profile is independent; no generic “PDF/X compatible” claim.

# Conformance descriptor

Defines PDF version, allowed color spaces, output intent requirements, transparency policy, font requirements, metadata, page boxes and prohibited features.

# Transparency

PDF/X-1a may require flattening depending exact standard/version; PDF/X-4 supports live transparency in appropriate form. Export plan chooses preserve/flatten based on selected profile.

# Fonts

Embed/subset where licensing permits. Missing/forbidden embedding produces preflight error or explicit outline fallback. Font substitution is never silent.

# Validation

Release claim requires external validator corpus/tool plus internal structural checks. Export report records validator status when available.

# Preflight

Errors are generated before encoding whenever determinable: RGB object in CMYK-only profile, missing output intent, unsupported transparency, font issues, spot conflicts.

# Tests

Known-good/known-bad files, external validator agreement, embedded/subset fonts, transparency cases and metadata.