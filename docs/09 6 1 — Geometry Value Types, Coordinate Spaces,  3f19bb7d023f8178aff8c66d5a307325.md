# 09.6.1 — Geometry Value Types, Coordinate Spaces, Precision & Tolerance Policy

# Purpose

Freeze the numeric and topological contracts used by every vector tool, renderer projection, snapping query and export adapter.

# Coordinate spaces

Define explicit spaces:

- **DocumentSpace** — canonical double-precision coordinates.
- **SurfaceSpace** — optional Surface-local coordinates.
- **ObjectLocalSpace** — pre-transform shape/path coordinates.
- **World/ComposedSpace** — object after parent transforms in document coordinates.
- **ViewSpace** — pan/zoom/rotation-independent logical viewport coordinates.
- **DeviceSpace** — physical pixels after DPR.

No API accepts a raw Vec2 without documenting its space. Prefer semantic wrapper types or function naming that makes conversion explicit.

# Core value types

```cpp
struct Vec2d { double x, y; };
struct Rectd { Vec2d min, max; };
struct Affine2D { double m00,m01,m02,m10,m11,m12; };
struct CubicBezier { Vec2d p0,p1,p2,p3; };
struct LineSegment { Vec2d p0,p1; };
struct AngleRad { double value; };
```

If strong wrappers are too costly in hot code, enforce space at service boundaries and naming conventions.

# Precision

Canonical geometry uses IEEE-754 double. GPU/render conversion to float occurs only after origin rebasing or equivalent precision management if large-document tests require it.

# Invalid numbers

NaN/Inf are rejected at parsing, command validation and geometry service boundaries. A geometry algorithm must never silently emit non-finite canonical state.

# Tolerance model

No global epsilon. Tolerances are operation-specific and scale-aware:

- hit-testing: screen-space tolerance converted to document units;
- coincidence: magnitude-aware relative/absolute pair;
- flattening: explicit geometric error in document/device units;
- boolean predicates: robust/adaptive predicate implementation where possible;
- curve simplification: caller-supplied maximum deviation.

# Transform decomposition

Canonical transforms may be stored as affine matrix plus optional UI-friendly decomposition metadata only if stable. Matrix is truth. Singular/near-singular transforms are valid only where operation semantics define them; tools must detect and constrain impossible inversions.

# Bounds

Maintain at least:

- geometric bounds;
- stroke/appearance bounds;
- effect-expanded visual bounds;
- conservative invalidation bounds.

Bounds are derived and tagged with revision dependencies.

# Units

Document coordinates are unitless canonical length interpreted through document unit metadata. UI conversion is projection; geometry engine never stores “px/cm/mm” strings.

# Tests

Extreme coordinate magnitude, denormals, nearly singular transforms, inverse roundtrip, screen/document conversion at extreme zoom, tolerance consistency, NaN rejection and randomized affine composition.