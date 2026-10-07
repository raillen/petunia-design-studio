# 18.06 — Parametric Shape Tool Family Specification

# Family

Rectangle, Ellipse, Rounded Rectangle, Polygon, Star, Triangle, Diamond, Pie/Donut, Line/Arrow, Cog and registered parametric shapes.

# Identity

Each shape has ToolId ptnd.tool.shape.<kind> and versioned ShapeKind parameters.

# State machine

Idle -> DragCreate -> LiveParameterAdjust -> Commit/Cancel.

# Gesture

Drag establishes primary bounds/axis. Shift constrains. Alt/Option creates from center. Click-without-drag follows default-size/numeric-create policy.

# Canonical

Store semantic parameters until explicit Convert to Curves.

# Context

Width/height and shape-specific properties such as radius, sides, inner radius, angles, teeth and arrowheads.

# On-canvas handles

Each handle maps to a stable PropertyId and the same SetShapeParameters command used by numeric UI.

# Commands

CreateShape, SetShapeParameters, ConvertShapeToPath.

# Tests

All drag directions, modifier order, parameter clamps, rotated objects, conversion geometry and PTND roundtrip.