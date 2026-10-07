# 20.2 — Page Boxes, Bleed, Trim, Media, Crop Marks, Registration Marks & Color Bars

# Surface print boxes

TrimBox derived from page/Surface intended final size.

BleedBox = Trim plus per-side bleed.

MediaBox/output page includes marks/slugs when export settings require.

CropBox/ArtBox only when PDF output semantics explicitly need them.

# Bleed

Per-side values in Surface. Content can extend into bleed; preflight warns critical objects too near trim if rule configured.

# Marks

Crop/trim marks, registration marks, color bars, page info/slugs. Settings define offset, length, stroke width and which sides/pages.

# Registration color

Marks that require all plates use Registration semantic color, not process black.

# Color bars

Built-in standard-ish control strip must be clearly documented and not claim certification unless matching standard/preset. Custom printer marks can be future preset resource.

# Export

Marks expand output MediaBox without changing canonical Surface/document artwork. Preview overlay optional.

# Multi-page

Marks generated per page with correct orientation/creep policy if imposition ever added; V1 is no imposition engine unless explicit.

# Tests

Bleed asymmetry, marks dimensions, registration separations, rotated page and PDF boxes validator.