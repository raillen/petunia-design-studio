# 18.24 — Lasso / Polygon / Magnetic Selection Tools

# Freehand Lasso

Records pointer path, simplifies safely and closes polygon at release; SelectionEngine rasterizes coverage.

# Polygon Lasso

Clicks add vertices; double-click/Enter closes; Backspace removes last; Esc cancels current construction.

# Magnetic Lasso

Native edge service evaluates image/composite gradient near pointer. Anchors placed manually/automatically; preview path follows strongest bounded-cost route.

# Sampling source

Current Layer versus Composite explicitly selected for magnetic analysis.

# Combine

New/Add/Subtract/Intersect shared across variants.

# Performance

Magnetic analysis uses cached lower-resolution/ROI representation while pointer moves and refines final path if needed.

# Accessibility

Polygon anchors can be keyboard-adjusted; final selection can be transformed/numerically edited through commands.

# Tests

Low/high contrast edges, high-res source, backtracking anchors, combine, cancellation and stale-source revision.