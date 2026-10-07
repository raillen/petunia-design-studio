# 18.05 — Vector Brush Tool Specification

# Identity

ToolId ptnd.tool.vector_brush.

# Result

Editable centerline VectorPath plus vector brush/stroke appearance with variable width/texture semantics.

# Input

Normalized pen samples; pressure/tilt/rotation feed appearance dynamics. Native renderer evaluates preview.

# Context

preset, width, opacity, stabilization, pressure, smoothing and blend.

# Expansion

Expand Stroke/Appearance yields vector geometry when representable; textured semantics may require rasterized expansion with explicit fidelity.

# Missing resource

Retain centerline and diagnostic fallback if brush resource unavailable.

# History/tests

One stroke transaction. Test pressure, tilt, missing preset, expand fidelity, save/reopen and long-stroke performance.