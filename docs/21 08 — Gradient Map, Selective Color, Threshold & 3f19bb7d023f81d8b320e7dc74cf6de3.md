# 21.08 — Gradient Map, Selective Color, Threshold & Posterize

# Gradient Map

ID ptnd.adjustment.gradient_map.

Compute luminance/value according defined model then sample GradientDefinition. Parameters reverse/dither optional. Stops use semantic colors/profile conversion.

# Selective Color

ID ptnd.adjustment.selective_color.

Color ranges Reds/Yellows/Greens/Cyans/Blues/Magentas/Whites/Neutrals/Blacks with CMYK-style component adjustments. Relative/Absolute mode semantics must match documented formula and oracle.

# Threshold

ID ptnd.adjustment.threshold.

Luminance/channel threshold with value range normalized; optional channel mode. Output black/white semantic values.

# Posterize

ID ptnd.adjustment.posterize.

Quantize levels per channel or luminance according levels >=2. Dither option explicit.

# ROI

All point effects.

# UI

Gradient editor; selective color range + CMYK sliders; threshold histogram/slider; posterize levels.

# Tests

Reference ramps/color patches, thresholds boundaries, posterize levels, gradient mapping and serialization.