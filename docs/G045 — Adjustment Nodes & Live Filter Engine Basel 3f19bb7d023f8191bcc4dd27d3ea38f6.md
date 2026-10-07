# G045 — Adjustment Nodes & Live Filter Engine Baseline

# Goal

Build nondestructive Photo adjustment/effect evaluation.

# Depends

G039–G044, render frame graph, 21.x.

# Primary

engine/renderer engineer.

# Deliverables

AdjustmentNode/EffectNode schemas; Levels, Curves, Exposure, HSL, Gaussian Blur, Unsharp/High Pass baseline; masks; ROI; parameter UI; CPU reference/GPU paths.

# Acceptance

Mixed document supports editable adjustments/live filters with preview/final convergence and PTND roundtrip.

# Tests

Reference images, effect ordering, masks, ROI seams, CPU/GPU tolerance and export.