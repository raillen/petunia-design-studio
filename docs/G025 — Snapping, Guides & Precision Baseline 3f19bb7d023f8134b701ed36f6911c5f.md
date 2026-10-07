# G025 — Snapping, Guides & Precision Baseline

# Goal

Add deterministic professional snapping/guide feedback to shape transform/create workflows.

# Depends

G024, G003.

# Primary

editor-engineer + systems-architect.

# Skills

editor-tooling, input-handling, performance-native, interaction-design.

# Deliverables

SnapCandidate types; screen-space tolerance; object bbox/center/node-like shape anchors; horizontal/vertical guides; grid baseline; ranking; hysteresis; equal-spacing simple candidates; overlay descriptors; snap preferences master/category toggles; guide Commands.

# Acceptance

Move/resize/create tools all call same SnapEngine. Snapping does not flicker between near-equal targets under hysteresis fixture. Visual and semantic indicators identify target.

# Tests

High/low zoom, rotated objects, guides/grid, equal gaps, toggles, hysteresis, hidden/locked and p95 query budget.

# Non-goals

Bezier tangent snapping, perspective grid.