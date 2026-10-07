# 18.05 — Vector Brush Tool

# Identity

ToolId ptnd.tool.vector_brush.

# Difference from Pencil

Creates centerline plus editable vector-brush appearance/width profile instead of plain geometry alone.

# Parameters

preset, base width, pressure/tilt mapping, smoothing, opacity, texture/scatter where vector-compatible, blend.

# Pipeline

samples -> stabilization -> centerline fit -> width/profile fit -> brush appearance -> renderer.

# Editing

Centerline remains Node-editable; width/profile editable separately. Preset changes do not retroactively alter committed stroke unless deliberately stored as linked brush resource.

# Export

Unsupported vector brush expands to geometry or rasterizes through degradation plan.

# Commands

CreateVectorBrushStroke, SetStrokeProfile, ExpandStroke.

# Tests

Pressure profile, transformed stroke, expansion equivalence, SVG/PDF degradation and performance.