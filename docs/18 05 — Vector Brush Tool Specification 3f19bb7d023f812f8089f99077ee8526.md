# 18.05 — Vector Brush Tool Specification

# Output

Editable centerline/path plus StrokeAppearance/BrushStyle preserving pressure, width and texture semantics instead of raster pixels.

# Input

Normalized pen batches with pressure, tilt and velocity. Native engine fits path and profile.

# Context

Brush preset, base width, opacity, smoothing, stabilizer, pressure response, texture/scatter and blend.

# Preview

Approximate native vector-stroke tessellation during capture; idle/final pass can refine mesh without altering canonical path/profile.

# Editing

Node Tool edits centerline. Stroke/Profile editor edits width. Expand Stroke is explicit conversion.

# Commands

CreateVectorBrushStroke, SetStrokeProfile, SetBrushStyle.

# Export

Standard strokes preserved where representable; textured/specialty strokes preflight expansion or rasterization.

# Tests

Pressure curves, texture resources, transformed path, expansion equivalence and export fidelity.