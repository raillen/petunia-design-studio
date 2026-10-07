# 21.5 — Levels, Curves, Exposure & Brightness/Contrast Adjustment Schemas

# Levels

Parameters per Master/channel: inputBlack, inputWhite, gamma/midtone, outputBlack, outputWhite. Ranges normalized semantic values with UI conversion to 0–255/16-bit percentages as appropriate.

# Curves

Channel selector plus ordered ControlPoint{id,x,y,kind}. x strictly nondecreasing; duplicate x policy explicit. Interpolation uses monotonic cubic/Bezier or defined curve model; extrapolation clamps by default.

# Exposure

Exposure EV, offset and gamma if chosen model mirrors established photographic adjustment. Exact formula documented in color working space; not generic brightness slider.

# Brightness/Contrast

Formula/curve model versioned. If perceptual mode differs from legacy linear mode, separate AdjustmentKind/version.

# Channel applicability

RGB/CMYK/Gray/Lab channels vary by document/render space. Unsupported channel mode is disabled rather than silently remapped.

# Histogram

UI backdrop is derived; adjustment math does not depend on histogram.

# Alpha

Adjustments affect color, not alpha, unless explicitly selected mask/channel operation.

# Tests

Known ramps/patches, clipping, neutral identity, channel isolation, CMYK policy, float HDR range and GPU/CPU oracle.