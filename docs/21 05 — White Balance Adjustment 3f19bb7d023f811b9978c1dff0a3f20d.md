# 21.05 — White Balance Adjustment

# ID

ptnd.adjustment.white_balance.

# Parameters

temperature/tint user-friendly representation plus optional direct white-point/source-target chromaticity in advanced schema.

# Math

Convert working color through appropriate XYZ/chromatic adaptation model; use specified adaptation transform (Bradford/CAT candidate locked by ADR) rather than arbitrary RGB channel scaling.

# Picker

Neutral picker samples selected image region and derives temperature/tint/white point; result shown before commit.

# Color models

Primarily RGB/scene-like image workflow. CMYK document may require internal conversion and is enabled only with validated semantics.

# Alpha

Unaffected.

# ROI

Point.

# Tests

Known color-temperature patches, neutral gray correction, picker, wide-gamut profiles and GPU/CPU tolerance.