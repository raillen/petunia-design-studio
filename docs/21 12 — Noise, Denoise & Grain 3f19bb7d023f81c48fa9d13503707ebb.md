# 21.12 — Noise, Denoise & Grain

# Noise ID

ptnd.effect.noise.

Parameters amount, distribution Uniform|Gaussian, monochrome/color, seed, scale if procedural. Deterministic seed canonical.

# Grain

Optional artistic grain uses texture/noise generator with size/roughness/contrast and blend; separate ID if semantics differ.

# Denoise

V1 scope may include median/basic denoise. Advanced ML/temporal denoise is separate provider/roadmap.

Median parameters radius/kernel, channel/luminance policy and edge mode.

# ROI

Noise point effect; median/denoise expands by radius.

# Color

Noise generated in defined working representation; monochrome preserves hue more predictably.

# UI

Amount/distribution/monochrome/seed advanced; denoise strength/radius.

# Tests

Seed determinism, statistics, flat field, alpha, high-bit-depth and ROI seams.