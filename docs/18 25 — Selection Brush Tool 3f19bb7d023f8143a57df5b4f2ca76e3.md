# 18.25 — Selection Brush Tool

# Identity

ToolId ptnd.tool.selection_brush.

# Purpose

Paint PixelSelection coverage with optional edge-aware assist.

# Brush

Mask-oriented brush pipeline: size, hardness, spacing, stabilizer and optional pressure-to-size. It never paints color pixels.

# Modes

Add/Subtract primary. New clears only at explicit stroke start. Intersect, if offered, is explicit.

# Edge-aware

Optional native edge detector/segmentation adjusts boundary from selected sample source. Strength/tolerance visible in context.

# Overlay

Selection coverage/marching ants + brush cursor + optional quick-mask overlay.

# Commit

One selection operation per stroke. Selection undo behavior follows History policy.

# Performance

No per-dab Python work. Edge analysis is ROI/cached and cancellable.

# Tests

Soft edges, subtract, tile boundary, edge-aware revision change, pressure and long stroke.