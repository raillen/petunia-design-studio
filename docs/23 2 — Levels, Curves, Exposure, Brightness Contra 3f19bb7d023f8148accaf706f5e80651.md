# 23.2 — Levels, Curves, Exposure, Brightness/Contrast & Tonal Adjustments

# Levels

Channels: composite/luminance/model channels according color mode.

Params: inputBlack, inputWhite, gamma, outputBlack, outputWhite. Normalize ranges to semantic 0..1 internally; UI maps bit-depth values.

# Curves

Per-channel ordered control points x/y ∈ [0,1], interpolation monotonic/spline policy explicit, endpoints default (0,0)/(1,1). Duplicate x handling rejected/merged deterministically. Optional free endpoints if product chooses.

# Exposure

Exposure EV stops as float, optional offset/gamma only if separate adjustment model chosen. Formula documented against linear-light values: multiply by 2^EV before downstream transfer.

# Brightness/Contrast

Choose normative algorithm rather than vague sliders. Specify pivot/contrast curve and whether operation is perceptual or linear; CPU reference generates expected patches.

# Clipping

Adjustments can produce out-of-range float intermediates; clip only at operation/format boundary defined by pipeline.

# UI

Histogram backdrop derived, channel selector, reset, numeric entry, before/after toggle.

# Tests

Gray ramps, color patches, 8/16/float inputs, channel-specific operation, monotonic curves and no banding beyond quantization reference.