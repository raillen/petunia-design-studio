# 18.27 — Flood Fill & Raster Gradient Specifications

# Flood Fill

Seed point + tolerance + connectivity + anti-alias + sample source determine native region. Fill source may be solid/pattern according capability.

# Sample source

Current Target or Composite Snapshot. Composite mode captures coherent revision before region analysis.

# Context

Tolerance, contiguous, anti-alias, sample merged, opacity, blend, fill source.

# Large regions

Analysis is cancellable and bounded. Commit applies to active PixelTarget and PixelSelection intersection.

# Raster Gradient

Uses GradientDefinition compatible with vector semantics where practical. Direct mode rasterizes into target; Live mode creates generator/effect node if explicitly selected.

# Context

Gradient type, stops, opacity, dithering, blend and Direct/Live mode.

# Commands

ApplyFloodFill, ApplyRasterGradient, AddLiveGradient.

# Tests

Flat/complex regions, alpha edges, selection intersection, profiles, huge image cancellation and gradient banding/dither.