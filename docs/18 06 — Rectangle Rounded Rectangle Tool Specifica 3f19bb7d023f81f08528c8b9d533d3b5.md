# 18.06 — Rectangle / Rounded Rectangle Tool Specification

# Identity

Rectangle/rounded rectangle parametric shape tool.

# Creation

Drag creates RectShape. Shift constrains square; Alt/Option center-out; reverse drag quadrants normalize size/transform.

# Params

width, height, four radii, linking mode and transform. Live handles edit corners without path conversion.

# Radius policy

Clamp to non-overlap rule; per-corner and linked editing explicit. Numeric context mirrors effective values.

# Context

W/H, radii/link, fill/stroke and snapping.

# Commands

CreateParametricShape, SetShapeParameters, ConvertShapeToPath.

# Accessibility/tests

Numeric Create Shape route. Test reverse drag, square, center-out, unequal radii, extreme radius, transform and SVG mapping.