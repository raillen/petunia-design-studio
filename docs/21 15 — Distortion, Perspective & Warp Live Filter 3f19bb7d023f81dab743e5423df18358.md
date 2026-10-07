# 21.15 — Distortion, Perspective & Warp Live Filters

# Scope

Separate image-space distortion filters from canonical object ProjectiveTransform/Warp tool. Filters operate rendered input and remain nondestructive.

# Candidate IDs

lens/pinch, twirl, ripple/wave, displacement-map, perspective-distort filter where useful.

# Mapping

Each defines inverse mapping output point -> source point for hole-free sampling. Sampling filter and out-of-bounds edge mode explicit.

# ROI

Potentially nonlocal. Filter descriptor supplies mapBounds/inverseBounds or declares full-input requirement.

# Displacement

Resource input + channel/scale mapping; missing resource explicit. Color profile of displacement interpreted as data channels under defined rules.

# GPU

Natural fragment/compute path but CPU reference required.

# UI

On-canvas center/control handles where useful plus numeric parameters.

# Tests

Identity parameters, bounds, edge modes, interpolation, resource missing, GPU differential.