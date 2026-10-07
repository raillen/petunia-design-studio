# 18.31 — Raster Gradient Tool Specification

# Target

Raster layer/mask with active selection. Mode can be Direct Pixels or Live Generator/Fill if product exposes.

# Interaction

Drag axis/handles using shared GradientDefinition. Preview native on target. Selection restricts coverage.

# Direct commit

Rasterize gradient through color engine into touched tiles, one tile delta transaction.

# Live

Insert Generator/Fill node with same geometry and colors; non-destructive. UI makes mode explicit.

# Context

gradient kind/spread/reverse, stops, blend/opacity, dither option for low bit-depth output if implemented.

# Tests

8-bit banding/dither, 16-bit, alpha gradient, selection edge, direct/live equivalence and undo.