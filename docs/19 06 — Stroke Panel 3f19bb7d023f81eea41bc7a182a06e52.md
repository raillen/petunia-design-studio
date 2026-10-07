# 19.06 — Stroke Panel

# Identity

PanelId ptnd.panel.stroke.

# Controls

Width, unit, alignment, cap, join, miter limit, dash pattern/offset, start/end markers, variable-width profile and scale-with-object policy.

# Target

Selected Stroke AppearanceEntry. Multi-selection shows Same/Mixed per property.

# Dash editor

Ordered numeric dash/gap values with presets and live preview. Invalid sequences rejected with explanation. Offset scrubbing coalesced.

# Profile editor

Graph of normalized arc-length vs width multiplier; add/move/delete points, interpolation mode, reset. Numeric table alternative required.

# Markers

Searchable marker list with size/orientation/scale options; marker geometry resources versioned.

# Commands

SetStrokeProperty, SetDashPattern, SetStrokeProfile, SetMarkers.

# Accessibility

Icon controls named; graph keyboard/numeric editing.

# Tests

Mixed selection, dash normalization, marker scaling, profile editing, expansion equivalence and undo.