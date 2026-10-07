# 09.7.5 — Masks, Pixel Selection, Feathering, Morphology & Mip/Preview Semantics

# Mask representation

Pixel masks use normalized single-channel tiled format; choose 8/16-bit precision per document/feature policy. Vector masks are separate semantic objects rasterized at evaluation time.

# PixelSelection

Selection mask is view/session editing resource unless promoted to channel/mask. Combine operations are max/min/subtractive/intersection equivalents defined mathematically for soft masks.

# Combine

New = B

Add = max(A,B)

Intersect = min(A,B)

Subtract = A × (1-B) or exact policy documented for soft masks.

Avoid binary-only assumptions.

# Feather

Gaussian/edge-distance based feather strategy produces soft mask non-destructively where selection property permits. Radius in document/pixel units explicitly defined.

# Morphology

Grow/Shrink, smooth and border selection operations can be implemented via distance transform/morphology with edge behavior specified.

# Marching ants

Derived contour visualization at threshold (e.g. 0.5) and screen-space animation; never canonical boundary.

# Mips

Raster layer display mips are derived after color/pixel semantics. Selection/mask preview may use lower-res temporary representation but commit/full-quality queries use base resolution.

# Sampling

Transforms/filters define edge modes: transparent/clamp/repeat/mirror where applicable. Masks default transparent outside extent.

# Tests

Soft combine algebra, feather across tile boundaries, transformed masks, vector+pixel compound mask, mip invalidation and threshold visualization consistency.