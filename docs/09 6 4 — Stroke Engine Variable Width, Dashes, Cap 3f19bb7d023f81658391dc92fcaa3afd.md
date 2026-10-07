# 09.6.4 — Stroke Engine: Variable Width, Dashes, Caps, Joins, Alignment, Arrowheads & Expansion

# Stroke definition

Centerline path + StrokeStyle:

width, alignment, cap, join, miterLimit, dashArray, dashOffset, variableProfile, start/end markers.

# Width

Non-negative document-space width. Hairline can be semantic special case if print/export requires device-independent thin stroke.

# Alignment

Center baseline. Inside/Outside require closed contour orientation/fill semantics and may become derived clipping/offset behavior; unsupported ambiguous cases produce defined fallback/disable.

# Caps

Butt, Round, Square baseline. Custom marker/arrowhead is separate geometry placed at tangent.

# Joins

Miter, Round, Bevel. Miter falls back to bevel when ratio exceeds miterLimit using exact documented formula.

# Dashes

Dash pattern normalized positive sequence; odd arrays duplicated; zero/invalid values rejected. Dash offset wraps total pattern length. Dash phase across closed contours defined.

# Variable width

Profile is normalized path parameter → width multiplier using control points/curve. Arc-length mapping is required so width profile follows visual distance, not raw cubic t. Profile can include pressure-derived samples simplified at commit.

# Expansion

ExpandStroke uses identical semantic evaluator as renderer. Pipeline computes stroked outline, caps/joins/dashes/markers, resolves overlaps according fill rule, outputs VectorPath.

# Transform behavior

Scale stroke option determines whether object transform scales width. Nonuniform transform handling is explicit: transform geometry and stroke metrics or expand/evaluate in appropriate space.

# Tests

Renderer-vs-expanded equivalence, sharp corners, tiny/huge widths, closed dash seam, nonuniform transform, variable width extrema, markers.