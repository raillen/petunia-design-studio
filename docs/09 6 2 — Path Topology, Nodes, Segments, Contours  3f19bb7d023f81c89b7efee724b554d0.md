# 09.6.2 — Path Topology, Nodes, Segments, Contours & ID Preservation

# Canonical path

```
VectorPath
  PathId
  FillRule
  Contour[]
    ContourId
    closed
    Segment[]
```

Baseline segment kinds: Line and CubicBezier. Quadratic input is converted to cubic on import unless a future schema explicitly preserves quadratics.

# Node representation

Editing UI reasons in terms of logical nodes. A node has NodeId, anchor position and handle semantics sufficient to reconstruct adjoining cubic segments.

Recommended semantic model:

```cpp
enum class NodeKind { Cusp, Smooth, Symmetric };

struct PathNode {
  NodeId id;
  Vec2d anchor;
  std::optional<Vec2d> in_handle;
  std::optional<Vec2d> out_handle;
  NodeKind kind;
};
```

Contour serialization may use node-centric or segment-centric layout, but public editing semantics must be node-centric and lossless.

# Contours

Open contour: N nodes, N-1 logical edges.

Closed contour: N nodes, N edges including last→first.

Minimum viable contour constraints documented for 0/1-node transient editing vs committed document state.

# Fill rule

NonZero and EvenOdd are canonical enum values. Winding orientation may influence import/export/boolean algorithms but does not replace explicit FillRule.

# Topology operations

Insert node preserves exact curve shape by De Casteljau subdivision.

Reverse swaps contour order and in/out handles while preserving geometry.

Join validates endpoints and may reverse one contour deterministically.

Break duplicates logical endpoint topology without coordinate perturbation.

Delete-preserve fits or reconstructs neighboring span under defined tolerance.

# Stable IDs

Unchanged nodes retain NodeId across edits. Inserted nodes get new IDs. Deleted IDs never alias new nodes within session/document.

Boolean/shape-builder output is new topology; old→new provenance map may be emitted as derived result for selection transfer but IDs are not reused unless exact preservation is provable.

# Subselection remapping

Geometry command returns TopologyChangeMap:

- preserved NodeId;
- deleted NodeId;
- inserted NodeId;
- provenance from old segment/node to new objects.

UI uses it to preserve selected nodes when sensible.

# Serialization

Path schema explicitly stores node IDs and contour IDs so automation/undo can maintain stable semantic references across save/load.

# Tests

Insert at t=0/1/0.5, reverse twice identity, join/break, closed/open conversion, self-intersecting paths, one-node malformed input, ID preservation and serialized roundtrip.