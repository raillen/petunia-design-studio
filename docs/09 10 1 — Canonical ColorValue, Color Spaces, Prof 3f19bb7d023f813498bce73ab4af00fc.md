# 09.10.1 — Canonical ColorValue, Color Spaces, Profiles, Alpha & Swatch Semantics

# ColorValue

A color is not four anonymous floats.

```
ColorValue {
  ColorModel model
  ColorSpaceRef space/profile
  channels[]
  alpha
}
```

Models: RGB, CMYK, Lab, Gray; Spot/Registration represented as named semantic ink references plus alternate color.

# Channel semantics

Ranges are defined per model: RGB/Gray normalized, CMYK normalized percentages internally, Lab L*/a*/b* semantic ranges. UI converts to familiar numeric display.

# Alpha

Alpha is independent coverage, not another color channel and not ICC-transformed. Spot alpha/opacity applies after ink semantic resolution.

# Profiles

ColorSpaceRef can reference built-in well-known space or ResourceId ICC profile. A document has working spaces/policies but individual placed resources may retain embedded source profile.

# Assign vs Convert

Assign Profile changes interpretation metadata without channel conversion.

Convert Profile transforms channel values to preserve appearance under intent. These are separate Commands with preview/preflight.

# Swatches

Swatch can be process ColorValue, GlobalColor reference target, SpotInk or Registration. Global swatch consumers store SwatchId linkage, not copied channel values.

# Serialization

Store exact semantic model/profile/channels and swatch references. UI-generated hex strings are never canonical color.

# Tests

Same numeric RGB under different profiles differs in display; assign vs convert; alpha invariant through ICC; global swatch propagation; spot alternate display.