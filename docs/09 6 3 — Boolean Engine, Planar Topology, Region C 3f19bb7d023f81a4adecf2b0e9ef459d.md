# 09.6.3 — Boolean Engine, Planar Topology, Region Classification & ID/Style Reconstruction

# Input normalization

Evaluate parametric shapes/live transforms to immutable path snapshots. Preserve source object/contour/node provenance metadata separately.

# Pipeline

1. broad-phase bounds;
2. intersect contour segments;
3. split at ordered parameters;
4. build directed planar edge graph;
5. classify edge sides/regions by fill rule and Boolean operation;
6. select surviving directed edges;
7. reconstruct contours;
8. simplify only under safe tolerance;
9. map provenance/styles;
10. validate topology.

# Operations

Union/Add, Subtract A-B, Intersect, XOR, Divide. Multi-operand operation order is explicit and documented.

# Holes/orientation

Output contour orientation normalized by convention for interoperability while fill-rule remains authoritative.

# Self-intersections

Input self-intersecting paths interpreted using declared fill rule; engine does not require simple polygons.

# ID mapping

Unchanged segments/nodes may preserve NodeId only when semantics guarantee identity. New intersection nodes get new IDs plus provenance references. Tool selection remapping receives OldToNewGeometryMap.

# Style policy

Baked boolean output appearance derives from topmost/primary operand according command option. Divide pieces may inherit appearance from visible contributing source; rule is deterministic.

# Live boolean

Canonical node stores operation + ordered children/refs. Evaluation cache fingerprints child geometry revisions. Enter isolation mode edits operands; Bake invokes same BooleanEngine.

# Failure

Non-convergent/pathological operation returns typed GeometryError with involved IDs and no mutation. UI can suggest simplify/rasterize but cannot silently degrade.

# Tests

Holes, fill rules, coincident boundaries, nested shapes, self intersections, multi-operand ordering, divide piece count, provenance mapping, style, fuzz and undo.