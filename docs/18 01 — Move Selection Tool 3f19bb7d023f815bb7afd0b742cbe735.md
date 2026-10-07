# 18.01 — Move / Selection Tool

# Identity

ToolId: ptnd.tool.move. Default shortcut: V.

# Purpose

Select, move, resize, rotate, shear and duplicate object selections without changing object type.

# States

Idle -> HoverCandidate -> Pressed -> DragMove | HandleResize | Rotate | Shear | PivotMove -> Commit/Cancel.

# Selection

Click top eligible object; Shift toggles; select-through modifier cycles candidates; drag empty area marquee-selects. Locked objects may be selectable depending preference but cannot transform. Hidden objects excluded.

# Transform gesture

Capture initial transforms/bounds/pivot and selection revision. TransformPreviewSession computes world delta, constraints and snapping. Canonical document unchanged until commit.

# Modifiers

Shift constrains axis/rotation/aspect by subgesture. Alt/Option duplicates-on-drag or resizes from center. Esc cancels. Arrows nudge; Shift arrows use large nudge.

# Context bar

Reference point 3×3; X/Y/W/H; lock aspect; rotation; shear; coordinate space; scale-stroke/effects; snapping.

# HUD

Delta X/Y, W/H, angle, snap measurements and equal-spacing indicators.

# Commands

TransformObjects; DuplicateAndTransform; optional SetPivot. Selection is view state.

# Boundary

HitTestService, SnapEngine and transform math native C++. Python ToolController owns gesture/event state.

# Accessibility

All transforms available through Transform panel/keyboard. Selection count/target announced.

# Performance

Pointer preview within one frame p95 on normal fixture; no Python per-object traversal.

# Tests

Rotated groups, masks, symbols, duplicate cancel, extreme zoom, nudge coalescing, save/undo.