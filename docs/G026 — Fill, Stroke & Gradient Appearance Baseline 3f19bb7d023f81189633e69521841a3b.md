# G026 — Fill, Stroke & Gradient Appearance Baseline

# Goal

Implement first professional Appearance stack subset shared by shapes and future paths.

# Depends

G023, G016, G022.

# Primary

editor-engineer + renderer-engineer.

# Skills

editor-tooling, rendering-2d, color-science, lang-cpp.

# Deliverables

AppearanceEntry IDs; one/multiple Fill/Stroke support architecture; Solid Fill; Linear/Radial Gradient; basic Stroke width/cap/join/dash; ColorValue RGB baseline with future profile hooks; Appearance/Color/Stroke panel minimal implementations; Fill/Gradient tool; serialization schema updates.

# Acceptance

Rectangle/ellipse can hold ordered fill/stroke entries, edit live from panels/tool, undo/redo, save/reopen and render/export consistently.

# Tests

Multiple entries, gradient stops/geometry, stroke caps/joins/dashes, mixed selection, undo, render goldens and schema roundtrip.

# Non-goals

Full CMYK management, variable stroke, effects/blend breadth.