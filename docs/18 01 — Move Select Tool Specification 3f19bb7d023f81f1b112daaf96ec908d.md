# 18.01 — Move / Select Tool Specification

# Identity

ToolId: ptnd.tool.move. Primary Action: ptnd.tool.activate.move. Default shortcut: V. Accepted targets: selectable document objects and Surfaces.

# States

Idle → HoverCandidate → Pressed → DragTransform | MarqueeSelect | HandleTransform | PivotMove → Commit/Cancel.

# Click and selection

Top eligible hit selects. Shift toggles membership. Select-through/cycle uses ordered HitCandidate stack. Locked objects may be selectable by preference but never movable.

# Drag

Capture initial transforms. Shift constrains dominant axis; Alt/Option duplicates staged selection; snapping applies ranked transform solution. HUD shows delta/coordinates.

# Handles

Bounding handles resize; rotation affordance outside corners; skew only when explicit. Handles stay screen-space sized. Context exposes X/Y/W/H, anchor, rotation, shear, aspect lock, scale strokes/effects.

# Multi-selection

Aggregate selection frame while preserving local transforms. Key object is for alignment, not implicit pivot.

# Duplicate drag

Clone on gesture start with new IDs; cancel removes staged clones; commit creates DuplicateObjectsAndTransform.

# Commands

TransformObjects, DuplicateObjectsAndTransform, SetObjectTransform. Selection is view state.

# Accessibility

Transform panel/numeric commands are keyboard equivalents. Selection changes announced.

# Performance and tests

Meet pointer-preview SLO. Test nested transforms, locks, symbols, duplicate cancel, extreme zoom, snap hysteresis, undo/redo and save/reopen.