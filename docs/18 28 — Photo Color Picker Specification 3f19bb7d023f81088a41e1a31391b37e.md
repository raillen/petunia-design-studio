# 18.28 — Photo Color Picker Specification

# Identity

Photo specialization of shared Eyedropper engine.

# Sample

Point or averaged radius; Current Layer or Composite; document-space value, rendered working value and optional display value are distinct modes.

# Channel awareness

When a channel/mask is active PixelTarget, picker reports scalar/channel value and composite value separately where meaningful.

# Info panel

Hover updates coordinates, components, alpha, profile/space and sample statistics at throttled UI rate without blocking canvas.

# Apply

Click may set foreground/fill color if Apply mode enabled; inspect-only never increments document revision.

# Soft proof

Proof affects Display sample only. Document/composite source samples remain explicitly identified.

# Tests

16-bit/float, CMYK, alpha, mask/channel target, average radius, transformed raster and inspect-only revision.