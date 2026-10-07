# 21.04 — Exposure, Brightness / Contrast Adjustments

# Exposure ID

ptnd.adjustment.exposure.

Parameters: exposureEV, offset optional, gamma optional. Exposure applies multiplicative 2^EV in defined linear-light working space before display encoding.

# Brightness/Contrast ID

ptnd.adjustment.brightness_contrast.

Parameters brightness, contrast, optional preserve luminosity/legacy mode only if explicit.

# Contrast

Pivot/midpoint and formula documented; avoid implementation-defined “looks right.” Prefer perceptual/linear strategy validated by fixtures.

# Alpha

Unaffected.

# UI

Sliders + numeric with reset and histogram/clipping feedback where useful.

# ROI

Point effects.

# Tests

Neutral gray ramps, EV doubling, identity defaults, highlights/clipping, negative values, masks, color preservation and GPU parity.