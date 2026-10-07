# 21.6 — White Balance, HSL, Vibrance, Black & White & Selective Color

# White Balance

Temperature/tint or sampled-neutral representation. Canonical parameters and transform model versioned; sampled point resolves to parameters at command time rather than storing arbitrary screen pixel.

# HSL

Master plus hue-range channels Red/Yellow/Green/Cyan/Blue/Magenta. Parameters Hue shift, Saturation, Lightness. Range boundaries/feather semantics fixed.

# Vibrance

Protect already-saturated colors/skin-tones only if exact algorithm exists and is documented. Otherwise use simpler saturation adjustment rather than opaque marketing behavior.

# Black & White

Channel contribution weights for source primaries/ranges, optional tint. Normalize/clamp policy documented.

# Selective Color

Color families + Cyan/Magenta/Yellow/Black adjustments; Relative/Absolute modes if implemented with precise formula and CMYK conversion semantics.

# Color space

Each adjustment states whether operates in HSL-like derived space, linear RGB, Lab or process color. Conversions use ColorEngine.

# Tests

Neutral identity, hue wheel, skin-tone fixtures only if algorithm promises it, extreme saturation, grayscale, profiles and GPU/CPU.