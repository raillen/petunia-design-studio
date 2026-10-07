# 21.8 — Distortion, Perspective-Compatible & Warp Filter Contract

# Scope

Pixel/image live filters distinct from canonical geometric projective transform/warp objects.

# Candidate filters

Lens-like distortion, pinch/bulge, twirl, ripple/wave and displacement map as roadmap/feature-led.

# Common schema

Center, amount/strength, radius/extent, interpolation/resampler, edge mode and quality. Displacement additionally ResourceId/channel/scale.

# Sampling

Inverse mapping preferred to avoid holes. Resampler defined (nearest/bilinear/bicubic/Lanczos where suitable). Alpha/color sampling occurs in documented representation.

# ROI

Distortions can require nonlocal source bounds; engine computes source footprint from output ROI or falls back full input for unbounded transforms.

# Preview

Interactive quality can reduce sampling; final export full quality.

# Tests

Identity parameters, borders, alpha, high-frequency grid, large displacement, missing resource and GPU/CPU tolerance.