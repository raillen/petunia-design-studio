# 04.5 — Native Third-Party Libraries: Geometry, Raster, Text, Color, Compression & IO Evaluation

# Rule

Libraries are implementation candidates, not domain contracts. Every dependency gets license, maintenance, security and replacement assessment.

# Text

HarfBuzz — shaping.

FreeType — font parsing/metrics/outlines/raster support.

Platform/Qt adapters — font discovery and fallback metadata.

# Color

LittleCMS — ICC transform baseline candidate.

OpenColorIO — optional advanced color pipeline candidate, not automatic replacement for ICC desktop-design semantics.

# Raster/codec

libvips — large image pipeline/import/export candidate.

libjpeg-turbo — JPEG.

libpng — PNG.

libtiff — TIFF.

WebP/libwebp — WebP.

AVIF/HEIF codecs only after redistribution/security/license review.

# Compression/container

ZIP library chosen for strict bounded reader/writer behavior.

Zstandard for internal resources where benchmarks justify it.

# Geometry

Boolean/offset/path library candidates require robust pathological fixture evaluation. No library is accepted solely because it is popular; wrapper interface isolates replacement.

# PDF/SVG

Dedicated parser/writer choices are separate ADRs because fidelity/security requirements differ sharply. Import and export may use different libraries.

# Dependency adapter

Third-party types do not become canonical public model types. Convert at module boundary.

# Updating

Dependency upgrades run focused corpus: malformed input, fidelity, regression, performance and ABI/build tests.