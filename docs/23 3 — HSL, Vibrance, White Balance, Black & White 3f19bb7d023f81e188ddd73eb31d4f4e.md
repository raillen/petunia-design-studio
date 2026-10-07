# 23.3 — HSL, Vibrance, White Balance, Black & White & Selective Color

# HSL

Master plus hue-range channels Red/Yellow/Green/Cyan/Blue/Magenta. Hue shift degrees, saturation/lightness normalized percent. Range overlap/interpolation explicitly defined; reference algorithm documented.

# Vibrance

Protect already-saturated colors/skin bias only if mathematically specified. Otherwise implement saturation adjustment first and label honestly.

# White Balance

Temperature/tint are perceptual controls mapped to chromatic adaptation/white-point transform. Store normalized Kelvin/tint or relative values with exact algorithm/version.

# Black & White

Channel mixer weights for source color components plus optional tint. Default preset designed to maintain luminance; weights visible/editable.

# Selective Color

CMYK-like corrections within color ranges require precise published/reference-compatible algorithm. If not implementable confidently for V1, mark Post-V1 rather than fake HSL substitute.

# Color model

Effects define supported document spaces. Generic screen RGB conversion is not automatically acceptable for CMYK-native editing; analyzer may convert through working space with disclosure.

# Tests

Color wheel, skin/chart patches, neutral preservation, extreme sliders and CPU/GPU parity.