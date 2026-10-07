# 25.1 — SVG Import/Export Capability Matrix

# Baseline

SVG is a V1 vector interchange target, not a second native format.

# Geometry

Paths/lines/basic shapes: Exact when representable.

Parametric Petunia shapes: export as equivalent SVG primitives or paths; editability grade depends mapping.

Booleans/live shapes: preserve appearance; live semantics usually Expanded.

Clips: Exact/Equivalent if standard clipPath/mask supports semantics.

Strokes: standard width/cap/join/dash Exact; variable/profile/textured strokes may Expand or Rasterize.

# Paint

Solid/linear/radial gradients: Exact/Equivalent.

Conical/advanced gradients: Approximate/Rasterize unless supported through accepted extension.

Multiple fills/strokes: may expand into grouped objects.

Blend modes: map CSS/SVG supported subset; unsupported preflight.

Effects: SVG filter mapping only when verified; otherwise expand/rasterize.

# Text

Artistic text can remain text where font/features/layout map safely. Frame text, complex story flow, hyphenation and advanced OpenType usually Approximate/Outlined or split text. Missing/unsupported fonts preflight.

# Color

SVG commonly RGB-oriented. ICC/CMYK/Spot preservation is limited; export plan converts or reports unsupported semantics explicitly.

# Pages

One SVG target per Surface; no custom fake multipage extension in baseline.

# Import

Parse hostile XML/SVG with external resources/scripts disabled by default. CSS/style resolution bounded. Embedded/raster resources brokered.

# Tests

SVG 1.1/2 commonly encountered constructs, transforms, nested clips, text, gradients, malicious external references and external viewer roundtrip.