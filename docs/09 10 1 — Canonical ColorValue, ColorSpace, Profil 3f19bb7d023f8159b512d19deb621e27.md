# 09.10.1 — Canonical ColorValue, ColorSpace, Profiles, Swatches, Spot & Registration Semantics

# ColorValue

Canonical color is typed:

```
ColorValue
  model: RGB|CMYK|Gray|Lab|Spot|Registration
  components[]
  alpha
  colorSpace/ProfileRef or SpotColorId
```

Component ranges and encoding are model-specific and validated.

# RGB

Values stored semantically in declared RGB color space/profile; not assumed sRGB.

# CMYK

Four process components in associated ICC CMYK space. Alpha separate coverage channel. CMYK values are preserved when document semantics require process-color editability.

# Lab

CIELAB with explicit white point/profile context. UI ranges standardized.

# Spot

SpotColor resource contains name, optional vendor/ink metadata and alternate process ColorValue for screen/fallback. Object references SpotColorId so renaming/alternate update propagates.

# Registration

Special semantic color meaning all separations/plates in print output; not equivalent to 100C100M100Y100K process black.

# Swatch

Swatch can contain direct ColorValue, global linked color, spot, gradient/pattern. Global reference stores SwatchId rather than copied components.

# Alpha

Object opacity and color alpha remain separate layers of semantics where applicable; compositing combines them but UI can edit independently.

# Serialization

Profiles/spot IDs explicit; floating component precision sufficient and finite. UI roundtrip formatting must not quantize canonical value unintentionally.

# Tests

RGB/CMYK/Lab/Gray/spot roundtrip, swatch link, registration export, alpha and profile replacement.