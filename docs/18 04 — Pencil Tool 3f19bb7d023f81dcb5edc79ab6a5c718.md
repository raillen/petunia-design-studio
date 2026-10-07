# 18.04 — Pencil Tool

# Identity

ToolId ptnd.tool.pencil.

# Purpose

Freehand vector path creation with smoothing and optional sculpt.

# Input

Pointer/pen samples sent in batches to native fitter. Preview uses provisional smooth polyline/cubic.

# Parameters

smoothing/tolerance; stabilizer; pressure influence; sculpt existing path; optional close-near-start.

# Fitting

Native fitter preserves corners/endpoints under defined curvature/speed policy. Tolerance is screen-aware but bounded in document coordinates.

# Sculpt

Starting near selected compatible path identifies parameter span and replaces it while preserving unaffected NodeIds when possible.

# Commit

One CreatePath/EditPath history entry per stroke. Esc/capture loss discards staged preview.

# Performance

Long strokes stream/simplify samples to bound memory.

# Tests

Different event rates, sharp corners, loops, sculpt span, pressure, save/undo.