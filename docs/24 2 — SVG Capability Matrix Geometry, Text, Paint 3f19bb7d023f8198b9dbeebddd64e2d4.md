# 24.2 — SVG Capability Matrix: Geometry, Text, Paint, Effects, Masks & Editability

# Geometry

Paths/lines/basic shapes Exact where direct. Parametric Petunia-only shape semantics export as standard SVG shapes/path => EquivalentAppearance but loses Petunia live parameters if reimported.

# Text

Simple artistic text may export editable text with font/style constraints. Frame text, linked stories, advanced OpenType/layout can be Approximate or outline/rasterize. Preflight lists.

# Paint

Solid/linear/radial gradients native. Conical gradient may require approximation/rasterization because SVG standard support differs. Multiple appearance fills/strokes may expand duplicate geometry/groups.

# Stroke

Basic width/cap/join/dash native. Variable width/profile expands outline.

# Masks/clips

Vector clip/mask native subset; pixel masks may embed raster mask. Complex compound/effects graded.

# Effects

SVG filters only used when equivalent, portable and tested; otherwise rasterize affected subtree or reject per policy.

# Color

RGB/ICC support depends SVG target/version/viewer; CMYK/spot generally not reliable standard SVG => degradation.

# Surfaces

One SVG per selected Surface. Multi-Surface package is batch outputs.

# Import

Parse standard SVG into canonical geometry/text/paint with fidelity report; unsupported foreign filters preserved only via extension/raw fallback if safe and useful, otherwise approximate/rasterize.

# Tests

Inkscape/browser/reference viewer corpus and reimport semantic comparison.