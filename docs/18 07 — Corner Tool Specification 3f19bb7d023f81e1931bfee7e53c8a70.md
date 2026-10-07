# 18.07 — Corner Tool Specification

# Identity

ToolId ptnd.tool.corner. Persona Design.

# Targets

Eligible VectorPath nodes.

# Canonical model

Live CornerModifier references ObjectId + NodeId, corner type, requested radius and effective radius policy. Source path remains editable until Bake.

# State machine

Idle -> HoverEligibleCorner -> DragRadius | MultiCornerEdit -> Commit/Cancel.

# Interaction

Hover shows candidate radius handle. Drag changes requested radius. Multiple selected nodes update in one staged transaction. Numeric field targets the same property.

# Corner kinds

Round, Chamfer, Concave and future namespaced variants. Each kind defines geometric construction and clamp behavior.

# Constraints

Effective radius cannot exceed adjacent segment capacity. Neighboring corners that compete for edge length use deterministic clamping and expose constrained state.

# Context/HUD

Type, requested radius, effective radius and Bake Corners. HUD indicates clamp.

# Commands

SetCornerModifier, RemoveCornerModifier, BakeCorners.

# Tests

Acute/obtuse angles, adjacent corners, open endpoints, transforms, undo, serialization and bake equivalence.