# 23.5 — Noise, Denoise, Vignette, Posterize, Threshold & Generator Effects

# Noise

amount, distribution (uniform/Gaussian), monochrome/color, seed. Random algorithm/version deterministic for same seed. Live effect seed canonical; “new seed” is explicit action.

# Denoise

If V1 includes denoise, algorithm/version and parameters must be specified. Non-deterministic ML/network denoise is separate provider capability, not built-in semantic effect.

# Vignette

amount, midpoint, roundness, feather and center. Formula/radial shape defined in normalized layer/object bounds coordinates.

# Posterize

Levels per channel or luminance model; quantization formula and minimum levels defined.

# Threshold

Threshold scalar/channel mode and output colors (black/white or mask) explicit.

# Gradient Map

GradientDefinition maps luminance/selected scalar to color; interpolation color space defined.

# Procedural generators

Checker/noise/gradient generators are canonical generator nodes only when parameter schema and deterministic bounds are defined.

# Tests

Fixed-seed noise hash, vignette symmetry, posterize exact levels, threshold edges and generator save/reopen deterministic.