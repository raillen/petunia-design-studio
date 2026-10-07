# 18.10 — Fill & Gradient Tool Specification

# Identity

ToolId ptnd.tool.fill_gradient. Design.

# Target

Active AppearanceEntry fill; fallback resolves primary fill and exposes target.

# Types

Linear, Radial, Conical baseline. Mesh is separate future specification.

# State machine

Idle -> TargetHover -> AxisCreate/Edit -> StopSelect/Drag -> Commit.

# Geometry

Stored in paint/object coordinate space with explicit transform.

# Stop model

StopId, normalized position, ColorValue, opacity, interpolation and midpoint. Duplicate positions allowed with deterministic order.

# Interaction

Drag creates/reorients axis; click axis adds stop; drag stop moves; Delete removes subject to minimum valid count.

# Context

Type, spread, interpolation, reverse, selected stop position/color/opacity.

# Commands

SetFill, SetGradientGeometry, AddGradientStop, SetGradientStop, RemoveGradientStop.

# Tests

Transforms, duplicate stops, radial focal constraints, conical wrap, multi-fill target and export fidelity.