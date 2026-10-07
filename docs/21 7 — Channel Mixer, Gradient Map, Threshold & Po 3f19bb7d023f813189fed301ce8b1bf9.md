# 21.7 — Channel Mixer, Gradient Map, Threshold & Posterize

# Channel Mixer

Output channel receives weighted inputs + constant. Matrix representation versioned per color model. Monochrome mode has its own normalized weights and output mapping.

# Gradient Map

Maps luminance/scalar metric to GradientDefinition. Luminance model documented; interpolation uses ColorEngine/gradient interpolation rules. Alpha preserved by default.

# Threshold

Threshold scalar and source channel/luminance mode. Output black/white semantic colors or mask mode if used as mask effect.

# Posterize

Levels per channel or unified level count; quantization formula, endpoints and dithering option explicit.

# UI

Parameter schemas can generate most controls; Gradient Map reuses gradient editor component.

# Tests

Matrix identity, constants, grayscale, gradient endpoints, threshold edge equality, posterize count and high bit depth.