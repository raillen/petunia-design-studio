# 19.07 — Transform Panel

# Identity

PanelId ptnd.panel.transform.

# Fields

X, Y, W, H, rotation, shear/skew, anchor 3×3, aspect lock, coordinate-space selector, scale-stroke/effects and optional pivot.

# Coordinate spaces

Document, Surface-local, parent/local and selection bounds where meaningful. Active space labeled.

# Mixed selection

Collective selection bounds can be shown while mixed individual transforms remain Mixed.

# Expressions

Numeric fields accept arithmetic/unit conversion through safe parser. Relative operations only if documented.

# Editing

Label scrubbing, arrows, typed input; continuous edit one transaction; Esc reverts active edit.

# Commands

TransformObjects, SetObjectTransform, SetSelectionBoundsTransform.

# Accessibility

Anchor grid has named positions; keyboard-first.

# Tests

Rotated objects, multiple parents, units, negative scale, lock aspect, precision and undo.