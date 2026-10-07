# G25 — Snapping & Smart Guides Baseline

# Goal

Implement stable ranked snapping shared by transform and future tools.

# Depends

G24.

# Authority

09.6.5, 02.6, 08.27.

# Owner

editor-engineer + systems-architect.

# Deliverables

SnapEngine, geometry/bbox/guide/grid providers, score/hysteresis, overlay descriptors, status/HUD integration and settings.

# Acceptance

No snap flicker in near-equal candidates; Shift constraints combine correctly; disable categories works; view zoom changes threshold in screen-space not document-space.

# Evidence

dense fixture benchmark and deterministic ranking tests.