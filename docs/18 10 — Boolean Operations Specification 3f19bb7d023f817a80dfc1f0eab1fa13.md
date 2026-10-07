# 18.10 — Boolean Operations Specification

# Actions

Union, Subtract, Intersect, XOR, Divide plus live variants.

# Operand ordering

Key/front object defines subtraction minuend and default style source. Order-sensitive operations expose this clearly.

# Live

Create LiveBoolean node retaining operand objects; operands remain editable and result is derived.

# Baked

Evaluate current geometry, create path result and replace/keep originals according option.

# Context

Pathfinder/toolbar buttons, Live toggle, disabled reason and large-result warning for Divide.

# Undo/export

One transaction restores exact hierarchy. External formats lacking live booleans expand with explicit fidelity.

# Tests

Coincident edges, holes, transformed operands, style inheritance, live edit, bake/undo.