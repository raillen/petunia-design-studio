# 18.11 — Transparency Tool

# Identity

ToolId ptnd.tool.transparency.

# Purpose

Edit object/appearance transparency gradients and opacity masks without altering fill color.

# Target

Selected object or Appearance entry transparency/mask. If none exists, tool can create a default linear opacity mask through explicit command.

# Interaction

On-canvas handles mirror gradient grammar but manipulate opacity coverage. Stop values 0..1 opacity; visual checker/preview clarifies transparent end.

# Modes

Linear/Radial/Conical transparency; uniform opacity remains Properties/Appearance value. Image/vector mask editing routes to their own target/tool.

# Context

Mask/opacity target; type; spread; reverse; selected stop opacity/position; unlink/remove mask.

# Commands

CreateOpacityMask, SetOpacityMaskGeometry, Add/Move/RemoveOpacityStop, SetOpacityStop, RemoveOpacityMask.

# Composition

Mask coverage multiplies existing object/appearance alpha at defined stack position. Renderer and export use same semantics.

# Accessibility

Ordered stop list + numeric opacity/position; semantic description “transparent/opaque”.

# Tests

Nested opacity, multiple fills, group mask, transformed object, export raster/SVG/PDF and undo.