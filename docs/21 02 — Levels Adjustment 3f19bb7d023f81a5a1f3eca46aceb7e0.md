# 21.02 — Levels Adjustment

# ID

ptnd.adjustment.levels.

# Parameters per channel

inputBlack, inputWhite, gamma/midpoint, outputBlack, outputWhite. Channel selector Master and model channels supported by document mode.

# Math

Normalize x between input black/white, clamp according policy, apply power/gamma mapping with well-defined gamma parameterization, remap output range. Master and channel operations order fixed.

# Alpha

Alpha unaffected unless Alpha channel explicitly selected and adjustment supports it.

# Color

For RGB, master operates defined luminance/component policy; exact implementation documented in CPU oracle. CMYK/Gray use native channel mapping or disabled portions.

# UI

Histogram backdrop, black/mid/white handles, numeric fields, output handles, clipping preview optional. Drag one transaction.

# ROI

Point operation; no expansion.

# Serialization

channels object with explicit values; defaults omitted only if schema supports canonical normalization.

# Tests

Ramp reference, endpoints, gamma=1 identity, per-channel, 8/16/float, masks and GPU parity.