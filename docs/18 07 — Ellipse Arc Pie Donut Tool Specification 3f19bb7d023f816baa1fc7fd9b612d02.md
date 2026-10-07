# 18.07 — Ellipse / Arc / Pie / Donut Tool Specification

# Identity

ToolId ptnd.tool.ellipse with parametric modes.

# Parameters

width, height, startAngle, endAngle, closureMode Full/Arc/Pie and optional innerRadius for donut. Angle convention and wrap are globally defined.

# Creation

Drag bbox; Shift circle; Alt/Option center-out. Arc handles edit start/end; inner-radius handle edits donut thickness.

# Context

mode, angles, inner radius, fill/stroke, W/H and snapping.

# Conversion

Convert to Curves uses bounded-error cubic approximation; export chooses native ellipse/path based target.

# Accessibility/tests

Numeric controls mirror every handle. Test wrap across 360°, zero arc, donut extremes, transform and save/export.