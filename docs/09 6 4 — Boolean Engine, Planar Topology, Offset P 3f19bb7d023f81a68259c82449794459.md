# 09.6.4 — Boolean Engine, Planar Topology, Offset Paths & Shape Builder Internals

# Boolean contract

Input: evaluated closed filled contours + FillRule + transforms normalized into common coordinate space.

Operations: Union, Subtract, Intersect, XOR, Divide.

Output: normalized path objects/contours + diagnostics/provenance.

# Preprocess

- reject non-finite geometry;
- normalize transforms;
- optionally remove numerically irrelevant duplicate vertices under operation tolerance;
- split curves at intersections;
- classify edge fragments against operands.

# Topology

Boolean engine must reason in planar arrangement/winding space, not raster approximation. Curve-preserving output is preferred; if candidate library flattens to polylines, acceptance requires explicit error/fidelity analysis.

# Coincident edges

Define deterministic handling for coincident/opposite-direction spans. Regression corpus must include identical rectangles, shared edges, kissing corners and nested holes.

# Result cleanup

Merge compatible adjacent segments, remove zero-length edges, close contours within robust topology rule and preserve fill semantics. Simplification must not change shape beyond tolerance.

# Provenance

Optional BooleanProvenance maps output segments to input object/segment sources for style decisions, debugging and selection transfer.

# Style inheritance

Baked boolean operation defines appearance source:

- Union/Intersect/XOR default from frontmost/key object policy;
- Subtract default from minuend;
- Divide can inherit source region styles where provenance supports it, otherwise documented deterministic fallback.

# Offset

Offset service supports distance, join, miter limit, cap for open path, cleanup and self-intersection resolution. Positive/negative orientation semantics independent from contour winding via fill-region interpretation.

# Shape Builder

1. collect selected shape regions;
2. construct arrangement;
3. derive atomic faces;
4. hit-test pointer to face;
5. accumulate selected face set add/subtract;
6. reconstruct boundary paths on commit.

# Cancellation/perf

Complex booleans run cancellable job above cost threshold. Interactive preview may operate on cached/evaluated operands but final commit uses exact configured quality.

# Differential tests

Compare vetted libraries/reference outputs on large corpus; area/winding invariants, topology validity, no self-invalid contours, deterministic ordering under same inputs.