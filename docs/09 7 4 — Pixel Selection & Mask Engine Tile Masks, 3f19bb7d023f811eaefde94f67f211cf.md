# 09.7.4 — Pixel Selection & Mask Engine: Tile Masks, Feathering, Morphology, Refine & Marching Ants

# Representation

Selection/mask uses scalar coverage tiles, preferably normalized 0..1 in 8/16/float implementation chosen by precision needs. Selection truth is coverage, not polygon edge.

# Combine

Replace: S=N.

Add: max/union semantic.

Subtract: S * (1-N) or defined coverage subtraction.

Intersect: S*N.

Exact coverage equations are fixed for antialiased masks.

# Rasterization

Vector/marquee/lasso shapes rasterize to coverage through common mask rasterizer with antialiasing policy. Pixel-center convention documented.

# Feather

Gaussian/distance-field-based feather semantics define radius in document pixels and edge behavior. Applying feather repeatedly to canonical selection differs from editable feather property; UI distinguishes.

# Morphology

Grow/Shrink, smooth and border operations use defined radius/connectivity and tile halo loading.

# Refine

Edge-aware refine service consumes source image/composite snapshot + initial selection + user trimap/brush hints. Output staged coverage; algorithm replaceable behind interface.

# Marching ants

Derived edge extraction on threshold contour; viewport rendering animates dash in screen space. Never serialized as selection.

# Mask transform

Pixel mask has coordinate transform/extent relationship to target/document. Resampling policy explicit when target transforms.

# Channels

Selection can convert to stored alpha/channel resource through Command. Channel→selection is snapshot/coverage conversion.

# Tests

Coverage combine math, feather across tile boundaries, morphology halos, vector mask raster parity, transformed mask, refine cancel and high-DPI overlay.