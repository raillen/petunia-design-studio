# 18.21 — Measure Tool Specification

# Purpose

Non-mutating measurement of geometry, distance and angle with optional explicit conversion to guide.

# Interaction

Hover objects exposes nearest geometric references. Click-drag between points shows distance, ΔX, ΔY and angle. Object-to-object hover can show spacing.

# Snapping

Measurement points use SnapEngine but can temporarily disable with modifier.

# HUD

Displays values with document units and copy-to-clipboard action. Optional pinned measurement is view annotation unless user creates canonical guide/dimension feature in future.

# Commands

None for ordinary measurement. CreateGuideFromMeasurement explicit mutation.

# Accessibility

Keyboard target cycling and Info/Measure panel exposes numeric values.

# Tests

Rotated objects, extreme zoom, unit changes, equal spacing and no document revision on measure.