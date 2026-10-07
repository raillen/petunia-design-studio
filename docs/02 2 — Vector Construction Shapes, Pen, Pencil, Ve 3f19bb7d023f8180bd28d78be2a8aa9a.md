# 02.2 — Vector Construction: Shapes, Pen, Pencil, Vector Brush, Corners & Knife

# Shapes

Live parametric primitives remain editable through canvas handles/context fields. Rectangle, Ellipse, Rounded Rect, Polygon, Star, Triangle, Diamond, Pie/Donut, Line/Arrow and Cog baseline.

# Pen

Precise Bezier construction, continuation, closing, handle conversion, snapping, smart/polygon submodes.

# Pencil

Freehand path fitted natively; smoothing and sculpt mode. Suitable for mouse/pen illustration without destructive pixelization.

# Vector Brush

Creates editable vector stroke with width/profile/texture semantics. It differs from Pencil by appearance/pressure representation.

# Corner

Live corner modifier on eligible path nodes. Multiple corner types and numeric radius.

# Knife/Scissors

Topology tools for slicing paths/shapes; selection/context rules prevent accidental broad edits.

# Shared guarantees

All construction operations have preview, cancel, one undo entry and exact numeric refinement after pointer gesture.

# Implementation

ToolController Python for interaction; geometry/hit/snapping/fitting/topology C++.