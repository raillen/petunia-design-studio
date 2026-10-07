# 18.34 — Blur / Sharpen Brush Tools

# Family

Localized Blur and Sharpen brush operators sharing BrushEngine mask/dynamics.

# Blur

Applies local convolution/low-pass under stroke mask. Radius/strength tied to tool parameter, not brush hardness.

# Sharpen

Uses defined unsharp/high-frequency operator with amount/radius/threshold or simplified strength mapped to canonical parameters.

# Neighborhood

Tool requests tile halo; processing avoids seams at tile boundaries. Source snapshot policy prevents recursive instability within dab unless intentional accumulated mode specified.

# Context

brush common controls + operator strength/radius and sample mode.

# Commit

Raster delta per stroke.

# Tests

Impulse/checker fixtures, seams, alpha edges, repeated passes, GPU/CPU parity and undo.