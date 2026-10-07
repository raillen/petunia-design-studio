# 21.11 — Sharpen, Unsharp Mask & High Pass

# Sharpen

Simple sharpen effect may be convenience preset over Unsharp Mask rather than separate undocumented kernel.

# Unsharp Mask ID

ptnd.effect.unsharp_mask.

Parameters radius, amount, threshold. Output = input + amount*(input - blur(input)) gated by threshold in defined luminance/channel metric.

# High Pass ID

ptnd.effect.high_pass.

Parameters radius and optional monochrome/neutral representation. Produces high-frequency residual centered on neutral value suited to blend workflows.

# ROI

Radius-based expansion matching underlying blur.

# Color

Sharpen operates defined working luminance/color channels and preserves alpha. Avoid sharpening unassociated transparent RGB unexpectedly.

# UI

Radius/amount/threshold, preview at 100% reminder if useful.

# Tests

Edge/step fixtures, noise threshold, alpha edge, high-pass neutral baseline, CPU/GPU and large radius.