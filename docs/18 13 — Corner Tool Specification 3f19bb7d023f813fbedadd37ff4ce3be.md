# 18.13 — Corner Tool Specification

# Identity

ToolId ptnd.tool.corner.

# Preconditions

Selected path nodes eligible for live corner modification. Parametric shape native corner controls remain shape params unless explicitly converted/bridged.

# Interaction

Hover eligible corner shows corner handle. Drag sets radius in document units. Multiple selected corners can edit together with same delta/value policy.

# Types

Round, Chamfer, Concave and additional styles only when CornerEvaluator defines exact construction. Type is canonical modifier parameter, not appearance-only hint.

# Radius

Bound by adjacent segment geometry. UI can show requested and clamped/effective radius when neighboring corners compete.

# Live vs bake

Live CornerModifier remains editable. Bake Corners replaces modified spans with explicit path geometry.

# Commands

SetCornerModifier, SetCornerRadius, BakeCorners.

# Tests

Adjacent short segments, multiple corners, transformed path, radius clamping, bake visual equivalence and undo.