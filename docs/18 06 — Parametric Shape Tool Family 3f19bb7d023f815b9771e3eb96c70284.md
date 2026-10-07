# 18.06 — Parametric Shape Tool Family

# Family

Rectangle, Rounded Rectangle, Ellipse, Polygon, Star, Triangle, Diamond, Trapezoid, Pie/Donut, Line/Arrow, Cog and future registered shapes.

# Shared state

Idle -> DragCreate -> AdjustCreation -> Commit/Cancel. Optional click-without-drag opens numeric creation flow.

# Gesture

Drag defines bounds/orientation. Shift constrains canonical proportions; Alt/Option creates from center; Space may reposition creation box if enabled.

# Shape schema

Each ShapeKind declares typed parameters, defaults, min/max constraints, on-canvas handles, context controls, path evaluator and export behavior.

# Semantic handles

Examples: corner radii, polygon points, star inner radius, pie angles, arrow head/tail, cog teeth. Drag edits parameter rather than arbitrary nodes.

# Appearance

New shape inherits current Design appearance defaults. Fill/stroke editable before/during creation through context.

# Convert

ConvertToCurves evaluates current semantic geometry to VectorPath preserving transform/appearance. Explicit destructive-to-parametric command.

# Registration

Plugins may contribute shape kinds only through deterministic schema/evaluator contracts; document never stores a Python callable.

# Accessibility

All semantic handles have numeric property equivalents. Shape type and current parameters announced.

# Tests

Parameter boundaries, negative drag directions, modifiers, handles, convert equivalence, boolean participation, serialization.