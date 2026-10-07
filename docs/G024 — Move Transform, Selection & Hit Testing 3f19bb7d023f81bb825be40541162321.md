# G024 — Move / Transform, Selection & Hit Testing

# Goal

Implement robust object selection and transform workflow for first vertical slice.

# Depends

G023, G017, G018.

# Primary

editor-engineer + systems-architect.

# Skills

editor-tooling, input-handling, lang-cpp, performance-native.

# Deliverables

Spatial index baseline; HitTestService for shapes; SelectionState; Move Tool; bounding box/handles overlay; move/resize/rotate; numeric Transform panel hookup; duplicate-drag; nudge; TransformObjects command; one-gesture undo.

# Acceptance

Pointer and numeric transform produce same canonical transform semantics. 10k simple objects selection/transform stays within initial budgets on Tier R fixture.

# Tests

Overlaps/cycling, rotated shapes, shift/alt modifiers, duplicate cancel, locked object, keyboard nudge, undo/redo and performance.

# Non-goals

Node/path hit testing, full snapping (G025).