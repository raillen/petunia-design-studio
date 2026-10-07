# 09.6.3 — Curve Mathematics, Flattening, Intersections, Nearest-Point & Robust Predicates

# Cubic evaluation

Use standard Bernstein/De Casteljau evaluation. Derivatives, extrema and subdivision are centralized utilities used by bounds, hit-test, length approximation and flattening.

# Bounds

Exact cubic axis-aligned bounds solve derivative roots per axis and evaluate valid t∈(0,1). Conservative fast bounds may exist for culling but cannot replace exact bounds where semantics require.

# Flattening

Adaptive subdivision until flatness/error <= requested tolerance. Tolerance chosen by caller:

- renderer: device-space visual error;
- boolean: geometry-quality tolerance;
- export: final-quality target;
- hit-test: interactive tolerance.

# Length

Adaptive numerical approximation with cached result keyed by path revision/tolerance. Do not treat polyline flatten at arbitrary render tolerance as canonical path length.

# Nearest point

Return:

```
NearestPointResult {
  contour_id
  segment_index/id
  t
  point
  distance_squared
  tangent
}
```

Use robust root-finding/subdivision strategy with bounded iterations and fallback.

# Intersections

Supported pairs: line-line, line-cubic, cubic-cubic. Result distinguishes proper crossing, tangent, endpoint touch and overlap/coincident span when algorithm supports classification.

# Robust predicates

Orientation/intersection classification should use adaptive exact predicates or vetted robust geometry library to avoid epsilon-only topology decisions.

# Degenerate curves

Cubic with coincident handles/points is legal and must reduce gracefully. Zero-length edges excluded or normalized where operation requires nonzero tangent.

# Arc/shape conversion

Ellipse and other parametric shapes can evaluate directly for rendering or convert to cubic approximation with documented error bound for path-only algorithms/export.

# Fuzz

Generate random curves with clustered points, huge/small scales, tangencies and overlaps. Invariants: no NaN, symmetric intersection results, distance≥0, subdivision continuity.