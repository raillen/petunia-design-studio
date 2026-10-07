# 21.06 — HSL & Vibrance Adjustments

# HSL ID

ptnd.adjustment.hsl.

Ranges Master, Reds, Yellows, Greens, Cyans, Blues, Magentas with hue-center/range semantics fixed.

# HSL parameters

hue shift, saturation, lightness; optional range boundaries/softness. Conversion model and hue wrap exactly defined.

# Vibrance ID

ptnd.adjustment.vibrance.

Parameters vibrance and optional saturation. Algorithm preferentially affects low-saturation colors; formula must be specified and CPU oracle retained, not proprietary handwave.

# Skin protection

If supported later, explicit algorithm/parameter, never hidden.

# Alpha

Unaffected.

# UI

Color range selector/wheel, sliders, eyedropper to select range where supported.

# ROI

Point.

# Tests

Hue wheel, grayscale stability, saturation extremes, range boundaries, wide gamut and GPU parity.