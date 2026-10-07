# 23.4 — Gaussian/Motion Blur, Sharpen, High Pass & Convolution Edge Policies

# Gaussian Blur

Canonical param sigma or radius with documented relation. Prefer sigma internal; UI radius maps consistently. Kernel truncation e.g. ceil(3σ) defines ROI expansion.

# Edge modes

Transparent, Clamp, Mirror, Repeat only where effect exposes/needs. Default for layer blur explicitly selected to avoid dark seams.

# Algorithm

CPU separable convolution reference; GPU may use optimized multi-pass/downsample for preview/final within tolerance. Large sigma preview can reduce resolution; final quality converges.

# Motion Blur

Length in document/pixel units + angle; sampling kernel definition and ROI expansion from vector extent.

# Unsharp Mask

amount, radius/sigma, threshold. Formula original + amount*(original-blurred) with threshold semantics defined in color/luminance domain.

# High Pass

radius + output neutral midpoint semantics; common overlay workflow is user choice, not automatically baked blend.

# Sharpen

If distinct simple sharpen exists, kernel/formula versioned.

# Alpha

Whether blur convolves premultiplied RGBA together or separates alpha is explicitly defined; avoid color fringe.

# Tests

Impulse response, edge tile seams, transparent colored edges, large sigma, GPU/CPU error and ROI partial rendering.