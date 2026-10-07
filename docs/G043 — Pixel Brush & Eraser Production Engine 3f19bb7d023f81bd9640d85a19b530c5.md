# G043 — Pixel Brush & Eraser Production Engine

# Goal

Deliver low-latency deterministic painting.

# Depends

G041–G042, input system, 09.7.3, 18.19–18.20.

# Primary

engine-engineer + performance-agent.

# Deliverables

BrushPreset schema; native BrushStrokeSession; resampling/stabilizers/dynamics/dabs; texture/scatter; erase operator; tile damage; cursor; Brushes/Settings UI baseline.

# Acceptance

Pressure/tilt strokes are responsive, seam-free, one undo entry and deterministic for fixed preset/seed.

# Tests

Pen fixtures, tile edges, high bit depth, alpha, long strokes, p99 latency and memory history.