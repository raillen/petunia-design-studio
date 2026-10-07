# 21.2 — Sharpen Family: Unsharp Mask, High Pass & Local Contrast

# Unsharp Mask

Parameters: radius, amount, threshold and channel/luminance mode. Reference: source + amount*(source - blur(source)) with threshold semantics precisely defined.

# High Pass

Radius and neutral/preview mode. Output encoding defines neutral value and alpha policy. Common workflow may combine with Overlay through user layer, but High Pass effect itself does not silently set blend mode unless preset explicitly does.

# Sharpen

Simple sharpen preset maps to defined kernel/USM parameters rather than opaque backend algorithm.

# Local contrast

If included, amount/radius/protect highlights/shadows or equivalent schema; not conflated with ordinary sharpen.

# Color

Luminance-only option uses defined color/luma model; component clipping/float range behavior documented.

# ROI

Expands by blur/kernel radius. Tiled evaluation requests halo to avoid seams.

# Tests

Edge step response, threshold, high-frequency patterns, alpha/transparency, high bit depth, CPU/GPU and repeated application.