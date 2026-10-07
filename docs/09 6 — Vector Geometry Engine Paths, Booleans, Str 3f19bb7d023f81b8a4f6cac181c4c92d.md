# 09.6 — Vector Geometry Engine: Paths, Booleans, Strokes, Hit Testing & Snapping

# Numeric policy

Document coordinates use double precision. Render conversion to float occurs at controlled backend boundary. Robust predicates use dedicated algorithms/tolerances, never global epsilon magic.

# Path

Contours of nodes/segments with line/cubic baseline. Editing operations preserve IDs where possible for selection/automation stability.

# Geometry services

bounds, nearest point, intersections, winding/fill rule, flattening, offset, stroke outline, boolean, simplify, split, join, reverse, transform.

# Boolean

Robust library/implementation hidden behind IBooleanEngine. Differential fixtures compare candidate implementations where practical. Live Boolean stores operands + operation and evaluates derived result.

# Strokes

Centerline + width/profile + cap/join/dash/alignment. Rendering and Expand Stroke must share semantic evaluation to avoid mismatch.

# Parametric shapes

Shape parameters remain canonical until explicit Convert to Curves. Renderer/boolean requests evaluated path snapshot.

# Hit testing

Spatial index narrows objects; shape-specific exact hit test ranks fill/stroke/nodes/handles by screen-space tolerance.

# Snapping

Snap query accepts pointer document coordinate, viewport scale, active tool, filters and candidates. Returns ranked target, snapped transform/point and overlay descriptors.

# Performance

No Python crossing per geometry primitive. Batch queries for visible bounds/hit candidates. Benchmarks cover thousands/millions of nodes and pathological intersections.

# Fuzz

Random paths, self intersections, extreme scales, coincident edges, NaN rejection, boolean invariants and roundtrip serialization.

[09.6.1 — Vector Core Data Model: Coordinates, Paths, Contours, Nodes, Segments & IDs](09%206%201%20%E2%80%94%20Vector%20Core%20Data%20Model%20Coordinates,%20Paths%203f19bb7d023f814fbe98f810dabd59dd.md)

[09.6.2 — Geometric Predicates, Tolerances, Robustness, Intersections & Numerical Policy](09%206%202%20%E2%80%94%20Geometric%20Predicates,%20Tolerances,%20Robustn%203f19bb7d023f81b3ae35c6811fbc5072.md)

[09.6.3 — Boolean Engine, Planar Topology, Region Classification & ID/Style Reconstruction](09%206%203%20%E2%80%94%20Boolean%20Engine,%20Planar%20Topology,%20Region%20C%203f19bb7d023f81a4adecf2b0e9ef459d.md)

[09.6.4 — Stroke Engine: Variable Width, Dashes, Caps, Joins, Alignment, Arrowheads & Expansion](09%206%204%20%E2%80%94%20Stroke%20Engine%20Variable%20Width,%20Dashes,%20Cap%203f19bb7d023f81658391dc92fcaa3afd.md)

[09.6.5 — Hit Testing, Spatial Index, Snapping Ranking, Hysteresis & Overlay Contract](09%206%205%20%E2%80%94%20Hit%20Testing,%20Spatial%20Index,%20Snapping%20Rank%203f19bb7d023f81fc91f0f003a80384e7.md)

[09.6.1 — Geometry Value Types, Coordinate Spaces, Precision & Tolerance Policy](09%206%201%20%E2%80%94%20Geometry%20Value%20Types,%20Coordinate%20Spaces,%20%203f19bb7d023f8178aff8c66d5a307325.md)

[09.6.2 — Path Topology, Nodes, Segments, Contours & ID Preservation](09%206%202%20%E2%80%94%20Path%20Topology,%20Nodes,%20Segments,%20Contours%20%203f19bb7d023f81c89b7efee724b554d0.md)

[09.6.3 — Curve Mathematics, Flattening, Intersections, Nearest-Point & Robust Predicates](09%206%203%20%E2%80%94%20Curve%20Mathematics,%20Flattening,%20Intersecti%203f19bb7d023f81fc9008f552be101405.md)

[09.6.4 — Boolean Engine, Planar Topology, Offset Paths & Shape Builder Internals](09%206%204%20%E2%80%94%20Boolean%20Engine,%20Planar%20Topology,%20Offset%20P%203f19bb7d023f81a68259c82449794459.md)

[09.6.5 — Hit Testing, Spatial Index, Snapping Ranking, Hysteresis & Smart Guides](09%206%205%20%E2%80%94%20Hit%20Testing,%20Spatial%20Index,%20Snapping%20Rank%203f19bb7d023f819c9ba2c931128c57b3.md)