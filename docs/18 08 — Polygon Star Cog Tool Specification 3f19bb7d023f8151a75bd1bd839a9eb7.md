# 18.08 — Polygon / Star / Cog Tool Specification

# Shared model

Parametric radial shapes preserve center, radii, rotation and shape-specific integer parameters.

# Polygon

sides ≥3, outer radius, rotation, optional rounding.

# Star

points, outer radius, inner ratio/radius, rotation, optional rounding.

# Cog

teeth count and bounded tooth/root geometry parameters.

# Creation

Drag center→outer radius. Shift angle-constrains. Wheel/shortcut side-count adjustment only if mirrored visibly in context field.

# Handles

Inner radius, rotation, rounding and count-sensitive handles with HUD.

# Validation

Bound counts/radii to protect performance and prevent degenerate geometry.

# Commands/tests

CreateParametricShape, SetShapeParameters, ConvertShapeToPath. Test min/max counts, self-intersection policy, tiny/huge radii and rounded conversion.