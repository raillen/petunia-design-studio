# 18.37 — Smudge / Blur / Sharpen Brush Specification

# Smudge

Stateful brush samples/picks up color from source and transports/mixes along stroke. Params strength, pickup/load, size/hardness, pressure. Native engine maintains stroke-local carry state.

# Blur brush

Applies local blur operator under brush mask. Kernel radius/strength derived from tool params; tile neighborhood expanded to avoid seams.

# Sharpen brush

Local unsharp/sharpen operator with bounded strength; repeated application accumulates predictably.

# Target/selection

Raster/PixelMask where operator valid; selection constrains.

# Performance

Neighborhood operations native/GPU candidate; never per-dab Python.

# Tests

tile edges, repeated strokes, transparent regions, smudge carry determinism, 16-bit and undo.