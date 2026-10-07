# 25.2 — PDF Import/Export Capability Matrix

# Role

PDF is professional output/interchange, not guaranteed editability-equivalent roundtrip.

# Export geometry

Vector paths, clipping, images and standard transparency/blends preserved where writer/profile supports. Parametric/live editing semantics flatten to PDF graphics while appearance can remain Exact.

# Text

Text can remain selectable/searchable with embedded/subset fonts if licensing and shaping representation support. Complex effects may outline/rasterize by degradation plan.

# Color

RGB/CMYK/Gray/ICC/spot/output intent supported according chosen PDF profile and writer evidence. Overprint and registration require prepress implementation.

# Pages

Surface/Page targets map naturally to PDF pages; boxes/bleed/marks per prepress spec.

# Import

PDF import reconstructs editable objects heuristically from graphics operators; it cannot promise original source semantics. Text reconstruction, clipping groups, transparency and image placements graded separately.

# PDF/X

Conformance is separate target profile, never inferred from ordinary PDF export.

# Security

Parser treats embedded files, actions, JavaScript, external references and malformed object graphs as hostile; active content disabled.

# Tests

Reference PDFs, transparency groups, fonts/subsets, CMYK/spot, multipage, malformed corpus and external validator/viewer checks.