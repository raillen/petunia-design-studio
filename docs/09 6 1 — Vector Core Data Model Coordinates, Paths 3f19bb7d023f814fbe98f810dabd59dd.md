# 09.6.1 — Vector Core Data Model: Coordinates, Paths, Contours, Nodes, Segments & IDs

# Coordinate model

Document coordinates use IEEE-754 double. Unit conversion occurs at UI/import/export boundaries. Geometry kernel rejects NaN/Inf before canonical storage.

# Core types

```cpp
struct Vec2d { double x, y; };
struct Rectd { Vec2d min, max; };
struct Affine2D { double a,b,c,d,tx,ty; };

enum class NodeKind { Cusp, Smooth, Symmetric };
enum class SegmentKind { Line, Cubic };

struct Handle {
  Vec2d position;
  bool enabled;
};

struct PathNode {
  NodeId id;
  Vec2d position;
  Handle in_handle;
  Handle out_handle;
  NodeKind kind;
};

struct Contour {
  ContourId id;
  std::vector<PathNode> nodes;
  bool closed;
};

struct VectorPathData {
  std::vector<Contour> contours;
  FillRule fill_rule;
};
```

Exact C++ syntax may change, semantics do not.

# Segment semantics

Segment between node i and i+1:

- line when both relevant handles disabled/collinear collapsed under exact representation rule;
- cubic using node i out-handle and node i+1 in-handle.

Quadratic input is converted to exact cubic during import unless a later ADR adds native quadratic storage.

# Local/world transform

Path geometry is stored in object-local coordinates. Object transform is canonical affine/projective node outside raw path geometry. Editing can choose local or transformed-space interaction but commits canonical local geometry/transform intentionally.

# Contours

Open contour has no implicit final segment. Closed contour joins last→first. Fill of open contour follows renderer/export rule explicitly; edit UI distinguishes closure state.

# IDs

ObjectId stable for object lifetime. ContourId stable across edits preserving contour identity. NodeId stable when node survives. Derived segments may have SegmentRef(contour, startNodeId) rather than persistent IDs.

# Winding

Canonical fill rule NonZero or EvenOdd. Contour orientation is not sole semantic truth but maintained predictably for export/boolean output.

# Degeneracy

Zero-length segments/nodes may temporarily exist during editing but validation/cleanup policies define when collapsed. Automatic cleanup must never silently alter intended geometry beyond tolerance.

# Serialization

All coordinates finite; node/contour order semantic. Handles serialized explicitly. Compact binary cache may exist but JSON canonical schema remains authoritative.

# Tests

Exact cubic conversion, open/closed behavior, transform roundtrip, ID persistence, zero-length contours, huge coordinates, serialization.