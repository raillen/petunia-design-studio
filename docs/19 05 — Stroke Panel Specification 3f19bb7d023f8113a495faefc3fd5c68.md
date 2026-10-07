# 19.05 — Stroke Panel Specification

# Identity

PanelId ptnd.panel.stroke. Design.

# Source

Selected StrokeAppearance entry or common stroke properties across selection.

# Controls

Width/unit, alignment, cap, join, miter limit, dash array/offset, arrowhead start/end, scale-with-object, variable-width/profile editor and brush style where compatible.

# Dash editor

Supports ordered numeric pattern, normalization policy, offset and preview. Invalid negative/empty patterns handled explicitly.

# Profile editor

Graphical width curve plus accessible numeric control-point table. Edits use same PropertyIds as on-canvas width tool.

# Mixed

Independent fields can show Mixed while others common. Applying width should not overwrite different dash profiles.

# Actions

Reset, copy/paste stroke, expand stroke, save stroke/brush preset.

# Tests

Caps/joins, miter clamp, dash odd/even patterns, arrowheads, variable profile, multi-selection and expand equivalence.