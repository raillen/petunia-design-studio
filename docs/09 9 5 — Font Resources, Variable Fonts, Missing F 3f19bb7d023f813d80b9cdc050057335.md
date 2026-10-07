# 09.9.5 — Font Resources, Variable Fonts, Missing Fonts, Embedding, Subsetting & Text-to-Curves

# Font identity

FontFaceDescriptor includes source fingerprint, family, subfamily/style, PostScript name where available, weight/stretch/slant, variation axes and licensing flags.

# Requested vs resolved

Text style stores requested font identity/family characteristics. Derived resolver selects available FontFaceId. Missing request is never overwritten by fallback silently.

# Variable fonts

Canonical style stores axis tag→value plus named-instance metadata optional. Values clamped/validated to font axis min/default/max.

# Embedded fonts

PTND may embed font resource only when licensing permissions allow and product policy enables. Manifest records embedding origin/license flags and checksum.

# Subsetting

Export adapter may subset glyphs for PDF/SVG according font embedding rights. Subsetter output is export resource, never canonical font mutation.

# Missing Fonts UI

Group by requested face, show affected story/object count, fallback currently used, replacement candidates and scope (document/style/selection). Replacement command can preserve style intent.

# Text-to-curves

Use exact glyph outline at resolved axes/features after shaping. Preserve glyph placement/transform. Result grouping policy explicit. Decorations/effects convert according appearance semantics.

# Font cache

Parsed faces/shaping fonts/glyph outlines are derived caches keyed by fingerprint + variation.

# Security

Fonts parsed under strict limits/fuzz corpus; collection files and malformed tables tested.

# Tests

Variable axes, missing font reopen, embedded resource roundtrip, licensed-not-embeddable preflight, subset reopen in PDF validator and text-to-curves visual equivalence.