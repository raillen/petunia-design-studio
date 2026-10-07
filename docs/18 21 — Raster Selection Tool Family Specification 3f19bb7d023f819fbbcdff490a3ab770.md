# 18.21 — Raster Selection Tool Family Specification

# Family

Rectangle/Ellipse/Row/Column Marquee, Freehand/Polygon/Magnetic Lasso and Selection Brush.

# Common output

PixelSelection staging mask. Selection is revisioned editing state and may later become canonical mask/channel through explicit command.

# Combine modes

New, Add, Subtract, Intersect. Mode is visible in context bar. Keyboard modifiers may temporarily override but UI reflects effective mode.

# Marquee

Drag geometry in document/pixel coordinates. Rectangle/Ellipse support feather/anti-alias. Row/Column resolve exactly one image pixel row/column at target raster resolution, not one screen pixel.

# Polygon lasso

Click adds vertices; Backspace removes last; Enter/double-click closes; Esc cancels staged polygon.

# Freehand lasso

Pointer path native-resampled; closure at pointer-up; smoothing policy explicit.

# Magnetic lasso

Native edge analysis proposes anchors/curve. User may insert/delete anchors. Source snapshot/revision prevents stale edge path from committing after image change.

# Selection Brush

Brush-like paint into mask with edge-aware option. Add/Subtract may map to pen modifier.

# Commands

SetPixelSelection, CombinePixelSelection, ClearSelection, InvertSelection.

# Tests

Combine algebra, feather tile edges, huge image, magnetic stale revision, row/column semantics, cancellation and selection-to-mask roundtrip.