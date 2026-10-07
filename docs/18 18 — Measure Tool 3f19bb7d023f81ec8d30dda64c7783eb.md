# 18.18 — Measure Tool

# Identity

ToolId ptnd.tool.measure.

# Purpose

Non-destructive measurement of distance, delta, angle, bounds, gap and object relationships.

# Interaction

Hover object surfaces candidate geometry. Click-drag defines arbitrary two-point measurement; clicking two objects can measure nearest gaps/centers according submode.

# Output

HUD shows distance, dx/dy, angle and units. Smart object measurement can show width/height/gaps.

# Snapping

Measure endpoints use SnapEngine categories without modifying objects.

# Persistence

Measurements are view-only by default. “Create Guide from Measurement” or future Annotation tool creates canonical object separately.

# Clipboard

Copy measurement value/action supports formatted current units and raw numeric option.

# Accessibility

Status/Info panel exposes all numeric values; keyboard point/object selection alternatives through semantic inspector advanced path.

# Tests

Rotated objects, different units, huge coordinates, guides/snapping, no document revision change.