# 18.07 — Corner Tool

# Purpose

Create and edit nondestructive corner modifications on eligible vector nodes.

# Eligibility

Node must have valid incoming/outgoing segments. Degenerate/open endpoints support only types explicitly defined.

# States

Idle -> HoverEligible -> Select -> DragRadius -> Commit/Cancel. Multi-select can apply one radius/type to compatible nodes.

# Types

Rounded, Chamfer, Concave and future registered types. Each evaluator defines exact geometry and feasible radius.

# Constraints

Radius clamps against neighboring geometry. Adjacent corner solving is deterministic and reports actual clamped value.

# Canonical

CornerModifier references NodeId and stores type/radius/parameters. Base geometry remains intact until BakeCorners.

# Context

Type, radius, link selected radii, clear, bake.

# Commands

SetCornerModifier, ClearCornerModifier, BakeCorners.

# Errors

Ineligible node shows disabled reason; no silent conversion.

# Accessibility

Numeric editor and selected-node list provide full alternative to dragging.

# Tests

Adjacent corners, tiny segments, closed paths, transformed paths, multiselect, mixed values, bake equivalence, undo.