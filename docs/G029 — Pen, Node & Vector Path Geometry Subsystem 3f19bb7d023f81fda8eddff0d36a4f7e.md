# G029 — Pen, Node & Vector Path Geometry Subsystem

# Goal

Introduce full cubic path canonical model and professional Pen/Node editing.

# Depends

G003–G006, G022, G024, G025, G026, 09.6 specs.

# Primary

editor-engineer + systems-architect. Performance/test independent.

# Skills

lang-cpp, editor-tooling, performance-native, fuzz-grammar-testing, rendering-2d.

# Deliverables

Contour/PathNode model with stable IDs; cubic evaluation/bounds/nearest point; path renderer/tessellation; Pen Tool state machine; Node Tool selection/handles; insert/delete-preserve; join/break/close/reverse; node snapping; PTND schema additions; SVG path export/import-ready representation.

# Acceptance

Complex multi-contour cubic paths can be drawn/edited precisely, save/reopen with IDs/geometry, undo all topology changes and render/export without discrepancy.

# Tests

Bezier math/property tests, insert exactness, tangent/intersection basics, fuzz malformed/extreme paths, node accessibility, 100k-node performance fixture and renderer goldens.

# Non-goals

Full Boolean engine/Shape Builder/offset stroke unless needed as follow-up.