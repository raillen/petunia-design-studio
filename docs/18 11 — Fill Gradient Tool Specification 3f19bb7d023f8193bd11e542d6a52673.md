# 18.11 — Fill / Gradient Tool Specification

# Target

Active FillEntry in Appearance or default fill of selection. Multiple-fill target is always visible.

# Modes

Solid, Linear, Radial, Conical and Image/Pattern transforms where supported.

# Gradient interaction

Drag axis. Click line adds stop. Drag stop moves. Drag-away delete requires visible threshold/affordance. Midpoint handles adjust interpolation midpoint.

# Modifiers

Shift angle-constrains. Radial secondary handles control radius/focal semantics.

# Context

kind, spread, reverse, selected stop position/color/opacity and target entry.

# Commands

SetFillKind, SetGradientGeometry, Add/Move/DeleteGradientStop, SetStopColor, ReverseGradient.

# Accessibility/tests

Stop list editable numerically. Test overlapping stops, radial focal point, multiple appearance entries and export.