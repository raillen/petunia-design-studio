# 21.14 — Shadows, Glows & Outline Layer Effects

# Families

Drop Shadow, Inner Shadow, Outer Glow, Inner Glow, Outline/Stroke Effect.

# Common

Color, opacity, blend mode, radius/blur, spread/choke, offset/angle where applicable.

# Shadow

Build source alpha mask -> offset -> spread/morphology -> blur -> colorize -> composite behind/inside according kind.

# Glow

Alpha edge -> spread/blur -> color/gradient optional -> composite outside/inside.

# Outline

Distance/expanded alpha geometry around object; position outside/inside/center where meaningful.

# ROI

Outer effects expand by offset + spread + blur radius; inner effects stay within source bounds but require halo for blur.

# Scale

Effect dimensions can scale with object according explicit property.

# UI

Effects panel/Properties with checkbox enable, preview and grouped parameters.

# Tests

Transparent source, nested group, vector/raster, scale transforms, ROI, export rasterization and GPU parity.