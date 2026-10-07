# 10.3.2 — Live Shape Handles, Constraint Solving & Parameter Editing

# Handle descriptor

HandleId, role, document/view position, cursor, draggable axes/constraint, linked PropertyId(s), snap eligibility and accessibility label.

# Gesture

Tool starts ShapeParameterEditSession with original params. Pointer maps to proposed parameter, applies constraints/clamps/snapping, renders preview, commit one SetShapeParameters command.

# Coupled params

Linked rectangle radii, star inner/outer radius and cog tooth geometry use small constraint solver/function; no cyclic widget callbacks.

# Numeric sync

Context/Properties reflect preview values without committing separate commands. Typing numeric during active gesture either finalizes/cancels gesture predictably.

# Modifier

Shift/Alt semantics only when defined per handle and status bar displays effect.

# Degeneracy

When shape becomes too small for visible handle separation, hit-test priority and handle hiding rules preserve editability via Properties.

# Tests

Handle drag under rotated/scaled object, linked/unlinked radii, snap, extreme zoom and keyboard numeric parity.