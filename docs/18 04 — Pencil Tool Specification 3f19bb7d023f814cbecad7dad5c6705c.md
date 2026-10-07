# 18.04 — Pencil Tool Specification

# Purpose

Freehand vector creation with smoothing, stabilization and optional sculpt of an existing path.

# Pipeline

Pointer samples -> native resampling -> stabilizer -> polyline preview -> cubic fitting -> simplification within geometric error -> VectorPath commit.

# State machine

Idle -> FreehandCapture -> FitPending -> PreviewFinal -> Commit/Cancel.

# Context

Smoothing, stabilizer, width, pressure-to-width, sculpt mode, close path, fill/stroke.

# Pen data

Pressure and tilt influence appearance only when the selected mode requires it; geometry must not accidentally encode device noise.

# Sculpt

Resolve compatible existing path span, stage replacement, preserve continuity/tangent policy and NodeIds where possible.

# Commands

CreateFreehandPath, ReplacePathSpan/SculptPath.

# Performance

No Python loop per sample. Fitting may refine after pointer-up but visual stroke feedback remains immediate.

# Tests

Deterministic fitting, maximum deviation, simplification, self intersections, very short strokes, closure and undo.