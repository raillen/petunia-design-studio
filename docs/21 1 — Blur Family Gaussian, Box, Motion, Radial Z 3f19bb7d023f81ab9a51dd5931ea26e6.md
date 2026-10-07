# 21.1 — Blur Family: Gaussian, Box, Motion, Radial/Zoom & Edge Modes

# Common schema

Every blur declares radius/extent units, quality tier, edge mode, alpha handling, color-space assumption, ROI expansion, CPU reference and GPU implementation capability.

# Gaussian Blur

Parameters: sigma/radius mapping, horizontal/vertical lock, optional anisotropic radius. Reference kernel uses mathematically documented Gaussian approximation/exact separable policy. ROI expansion >= ceil(kernel support) according quality implementation.

# Box Blur

Radius X/Y, iterations. Used as explicit effect only; internal Gaussian approximation cannot leak as different visual effect.

# Motion Blur

Length, angle, center/reference point if needed, edge mode. ROI expands along motion vector.

# Radial/Zoom Blur

Center, angle/amount, quality/sample count. Preview can use fewer samples; final quality deterministic.

# Edge modes

Transparent, Clamp, Mirror/Reflect and Repeat only where effect semantics support. Default fixed per effect and serialized.

# Alpha

Blur operates premultiplied coverage/color according compositor spec to prevent dark fringes. Mask blur may operate scalar only.

# Performance

Large radius can switch algorithm (separable, IIR/FFT candidate) if output tolerance meets reference. Strategy is implementation detail.

# Tests

Impulse response, edges, alpha halos, huge radii, CPU/GPU tolerance, ROI seams and preview/final convergence.