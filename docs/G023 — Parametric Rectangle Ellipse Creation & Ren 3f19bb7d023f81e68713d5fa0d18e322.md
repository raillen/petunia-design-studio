# G023 — Parametric Rectangle/Ellipse Creation & Rendering

# Goal

Deliver first real Design creation tools backed by canonical parametric shapes.

# Depends

G005, G016, G022.

# Primary

editor-engineer + renderer-engineer.

# Skills

editor-tooling, lang-cpp, lang-python, rendering-2d, interaction-design.

# Deliverables

ShapeKind Rectangle/Ellipse schemas; CreateShape commands; Design tool controllers; drag/modifier creation; shape evaluator to vector/render primitive; live properties width/height/corner radius for rectangle baseline; tool context controls; Layers/Properties integration.

# Acceptance

User can create/select/edit shapes, undo/redo, serialize snapshot-ready data and render correctly at transforms/zoom.

# Tests

Gesture modifiers, numeric create/edit, negative drag, corner radius constraints, undo, semantic UI and render golden.

# Non-goals

Pen/path node editing, booleans.