# 18.23 — Marquee Selection Tool Family

# Variants

Rectangle, Ellipse, Single Row, Single Column.

# Target

PixelSelection session state. Combine mode always visible.

# States

Idle -> DragShape -> AdjustOrigin/Constraint -> Commit/Cancel.

# Modifiers

Shift constrains geometry; Alt/Option center-out; Space repositions. Add/Subtract/Intersect modifiers are reflected by cursor/HUD and context bar.

# Rasterization

SelectionEngine turns geometry into antialiased coverage. Feather policy is explicit and consistent.

# Context

New/Add/Subtract/Intersect, feather, antialias, fixed aspect/size if enabled.

# Commands

SetPixelSelection/CombinePixelSelection.

# Accessibility

Numeric bounds and Select menu operations provide non-drag alternatives.

# Tests

Subpixel edges, feather, row/column, combine math, transformed target, cancel and undo.