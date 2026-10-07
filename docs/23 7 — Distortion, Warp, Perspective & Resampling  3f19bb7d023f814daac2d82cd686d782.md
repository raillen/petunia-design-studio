# 23.7 — Distortion, Warp, Perspective & Resampling Filter Contracts

# Distortion family

Each distortion declares coordinate mapping f(output)->source for stable inverse sampling where possible, bounds policy and resampler.

# Perspective

Projective transform matrix/homography separate from arbitrary warp. Singular mapping rejected. Raster live transform uses source image + homography + filter.

# Envelope/Mesh warp

Canonical control lattice with normalized local coordinates and interpolation method (bilinear/bicubic/etc.) versioned.

# Resampling

Nearest, Bilinear, Bicubic, Lanczos candidate filters with kernel/support definitions. Default selected per operation, not backend whim.

# Edge

Transparent/clamp/mirror/repeat explicit.

# Quality

Preview may use bilinear; final uses requested filter. Canonical command stores requested final filter where output depends on it.

# Tests

Grid image, straight-line preservation for projective, mesh continuity, singular rejection and CPU/GPU parity.