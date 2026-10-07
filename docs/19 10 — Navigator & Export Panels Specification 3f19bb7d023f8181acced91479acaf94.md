# 19.10 — Navigator & Export Panels Specification

# Navigator

PanelId ptnd.panel.navigator. Derived thumbnail of active document/Surface(s), current viewport rectangle and zoom field/buttons.

# Interaction

Drag viewport rectangle pans; wheel/controls zoom. View-only, no document revision.

# Large docs

Thumbnail updates incremental/async, coalesced on ChangeSets, never full-resolution render per edit.

# Export Panel

PanelId ptnd.panel.export. Lists export targets/slices/Surfaces, preset, scale variants, file naming and destination status.

# Slice rows

Stable ExportTargetId, name, area, preset, scales/suffixes, enable, path/destination grant state.

# Actions

Create/Delete slice, apply preset, quick export selected/all, open full Export dialog, reveal result.

# Jobs

Rows display current JobId/progress/result; retries do not duplicate successful outputs without policy.

# Tests

Navigator huge canvas, mixed DPI, many slices, missing destination grant, batch cancel/retry and accessibility.