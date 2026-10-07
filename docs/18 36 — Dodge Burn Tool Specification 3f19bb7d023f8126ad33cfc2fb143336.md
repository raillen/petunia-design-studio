# 18.36 — Dodge / Burn Tool Specification

# Modes

Dodge lightens; Burn darkens. Range: Shadows, Midtones, Highlights. Exposure/strength and Protect Tones.

# Direct pixel algorithm

Specify luminance/tone weighting curve and color preservation domain. CPU reference required; do not implement arbitrary multiplicative RGB without defined semantics.

# Non-destructive option

If live dodge/burn layer or paint adjustment exists, it is separate target/mode with explicit representation.

# Brush

Uses size/hardness/flow/pressure. One stroke transaction.

# Target

Raster pixels/mask only where meaningful. Active selection restricts.

# Tests

gray ramps, saturated colors, repeated strokes, ranges, protect tones, 16-bit and undo.