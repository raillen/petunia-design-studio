# 09.6.2 — Geometric Predicates, Tolerances, Robustness, Intersections & Numerical Policy

# No global epsilon

Different problems require scale-aware tolerances. Define ToleranceContext from document scale, viewport scale and algorithm domain; exact predicates used where available.

# Predicate categories

orientation/side tests, bounding overlap, point-on-curve, root classification, parallelism, coincidence, winding, segment intersection and containment.

# Screen vs document tolerance

Hit/snapping tolerances are expressed in screen pixels then transformed to document-space bounds. Boolean/topology tolerances are document/geometry precision rules independent from zoom.

# Cubic intersections

Use subdivision/root isolation with bounded recursion and robust termination. Return parameter pairs + classification: crossing, tangent, overlap/coincident candidate, endpoint.

# Coincident geometry

Represent coincident ranges explicitly during boolean/shape-builder planning instead of collapsing to arbitrary intersection points.

# Parameter clamping

Numerically derived t values within tolerance of [0,1] clamp; farther values rejected. Sorting intersections uses stable tie-breakers.

# Bounding boxes

Tight cubic bounds derived from derivative extrema, not control hull only, for precise hit/cull where needed; fast conservative bounds may coexist as derived cache.

# Precision extremes

Coordinates above supported safe magnitude trigger diagnostic or normalization strategy; algorithm must not silently overflow intermediate computations.

# Deterministic ordering

Topology reconstruction sorts events with deterministic key so different platforms do not produce random object/node ordering.

# Differential tests

Compare robust reference/library and candidate implementation on generated/pathological corpus. Invariants: no missing intersections, boolean area sanity, reverse/transform consistency.

# Fuzz

Coincident edges, near-tangent cubics, microscopic loops, huge scales, repeated nodes, self intersections, random transforms.