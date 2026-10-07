# 09.6.5 — Hit Testing, Spatial Index, Snapping Ranking, Hysteresis & Smart Guides

# Spatial index

Per-document/view derived index stores conservative object visual/geometric bounds keyed by ObjectId/revision. Candidate structures: R-tree/BVH; implementation hidden behind SpatialIndex.

# Hit-test stages

1. convert pointer screen tolerance to document tolerance;
2. query spatial candidates;
3. rank by paint order/tool filter;
4. object-specific exact test: handles/nodes/stroke/fill/text/image;
5. return ordered HitCandidate list.

# Hit candidate

Includes ObjectId, sub-element kind/id, distance, z-order rank, capability flags and semantic cursor hint.

# Selection cycling

Repeated click at similar screen position/time can cycle candidates using stable candidate fingerprint. Moving beyond threshold resets.

# Snapping providers

Geometry nodes, path tangents, bounding box edges/centers, guides, grid, margins, columns, text baselines, Surface boundaries, equal-spacing candidates and perspective grid.

# Snap query

Input: proposed point/transform, active tool, selected IDs to ignore, viewport scale, enabled categories, angle constraint and previous active snap.

Output: snapped solution + ranked candidate(s) + overlay descriptors + score.

# Scoring

Score combines screen-space distance, provider priority, semantic relevance to active manipulation, axis compatibility and hysteresis bonus. Exact weights are centralized and testable, not scattered constants.

# Hysteresis

Once snapped, retain target until pointer leaves release threshold > acquisition threshold. Prevent flicker between near candidates.

# Smart guides

Derived overlays include source/target anchors, equal-gap indicators and distance labels. Overlay lifetime limited to gesture/frame.

# Performance budgets

Hit-test and snap queries target sub-frame latency with 100k+ visible objects using index pruning. Benchmark candidate count and worst pathological overlap.

# Tests

Dense overlap, hidden/locked objects, nested transforms, extreme zoom, equal-spacing ambiguity, guide+geometry competition, hysteresis and selection-cycle determinism.