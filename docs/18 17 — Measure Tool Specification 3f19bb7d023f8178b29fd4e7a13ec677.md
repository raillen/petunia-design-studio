# 18.17 — Measure Tool Specification

# Identity

ToolId ptnd.tool.measure. Shared.

# Purpose

Inspect distance, angle, delta and object spacing without mutating the document by default.

# State machine

Idle -> HoverGeometry -> FirstPointArmed -> Drag/SecondPoint -> ResultVisible -> Reset.

# Snapping

Measurement endpoints use SnapEngine with configurable candidate classes. Measurement snapping never commits document state.

# Output

Distance, delta X/Y, angle and optionally object edge/center spacing. Units follow document/user preference.

# HUD

Screen-space readout avoids pointer hotspot; values can be copied. Status/semantic inspection exposes same values.

# Persistent conversion

Explicit actions may Create Guide From Measurement or Create Annotation if/when those features exist; such actions are normal Commands.

# Accessibility

Keyboard can focus measurement results, copy values and enter exact endpoint coordinates through inspector.

# Tests

Rotated geometry, mixed units, extreme zoom, snapping and zero document revision for inspect-only flow.