# 18.04 — Pencil Tool Specification

# Identity

ToolId ptnd.tool.pencil.

# Purpose and states

Freehand vector path. Idle → StrokeArmed → Drawing → FinalizingCurveFit → Commit/Cancel.

# Input and fitting

Raw samples batch native; smoothing/stabilizer, resampling, simplification and cubic fitting use bounded error. Result should be substantially independent from event rate.

# Pressure

Optional pressure creates vector width profile separate from centerline geometry.

# Sculpt

Drawing over selected compatible path can replace nearest span with visible affected-range preview.

# Context

smoothing, stabilizer, sculpt, close path, fill/stroke and pressure.

# Commit

CreatePath or EditPathSpan one transaction.

# Accessibility/tests

Pointer-optimized; Pen/Node is precise alternative. Test slow/fast sampling, sharp corners, loops, sculpt, cancel and deterministic fitting.