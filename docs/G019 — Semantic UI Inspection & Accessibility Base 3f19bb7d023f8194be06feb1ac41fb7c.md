# G019 — Semantic UI Inspection & Accessibility Baseline

# Goal

Establish stable semantic IDs/tree and accessibility bridge before UI expands.

# Depends

G013–G018.

# Primary

accessibility-reviewer + ui-component-engineer.

# Skills

accessibility, keyboard-accessibility, screen-reader, focus-management, testing-quality.

# Deliverables

SemanticId conventions; UI semantic tree service; window/tool/panel/control states; QAccessible names/roles for custom base controls; focus-zone manager; keyboard cycle actions; developer inspector panel/API; test snapshot format.

# Acceptance

Main shell, Layers and Properties can be navigated keyboard-only and inspected semantically. Missing accessible name is CI-detectable for registered controls.

# Tests

Focus traversal, semantic snapshot golden, high contrast, 200% scaling, screen reader smoke and localization expansion sample.

# Non-goals

Full canvas object accessibility tree for all future tools.