# 08.24 — Design Persona Tool-by-Tool UX Contract & Affinity Mapping

# Purpose

This page is the parity/deep-interaction companion to 08.7 and 10.1–10.8.

# Mapping

Move/Node/Pen/Pencil/Vector Brush/Corner/Knife/Shape/Shape Builder/Fill/Transparency/Artistic Text/Frame Text/Surface/Measure/Hand/Zoom each receive:

- Petunia ToolId;
- comparable professional workflow observed in Affinity/other editors;
- deliberate divergence;
- activation/shortcut;
- pointer phases;
- modifier table;
- context bar controls;
- HUD/overlay;
- dependent panels;
- Actions/Commands;
- disabled reasons;
- accessibility route;
- test fixture.

# Professional expectations

Pen must support uninterrupted contour construction and precise handle editing; Node must support multi-node transforms; Move must expose numeric precision and smart guides; Shape tools remain live/parametric; Fill/Transparency edit on-canvas; Text editing must not kick user into separate app mode.

# Consistency gate

If two Design tools manipulate same property (e.g. fill), they must invoke same PropertyId/Command semantics and present compatible controls, not parallel state stores.

# Reference divergence

Petunia may surface live/destructive toggle more explicitly than Affinity and may use command palette/semantic inspector as additional access paths.