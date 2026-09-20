# 09.5 — Vector Geometry Engine: Numeric Policy, Paths, Boolean, Stroke & Snapping Contracts

# Boundary

`aubrieta_geometry` exposes Aubrieta types/contracts; Kurbo/iOverlay/Lyon are adapters/engines behind it.

# Numeric policy

Document geometry uses f64 for canonical coordinates unless a later ADR proves otherwise. Define epsilon/tolerance by operation and scale; never use one global magic epsilon. Serialization avoids lossy rounding except format adapters.

# Path model

Move/Line/Quad/Cubic/Close segments, explicit subpaths, fill rule, node semantic metadata separated from render tessellation. Degenerate segments are preserved or normalized by explicit command, not silently during unrelated edits.

# Boolean pipeline

Validate inputs → normalize/flatten within controlled tolerance → overlay operation → reconstruct contours → preserve/provide provenance where available → simplify only under defined tolerance. Live Boolean stores operands+operation; baked boolean commits output path.

# Stroke

Semantic stroke includes width, alignment, joins, caps, miter, dash pattern/phase, variable-width profile and brush semantics. Outline/expand is deterministic command with tolerance metadata.

# Spatial/snapping

Separate coarse spatial query from exact geometry candidate evaluation. Snap candidates have type, source, target, world distance, screen-space score, axis/constraint and visual hint. Priority is deterministic and configurable.

# Robustness

Self-intersection, coincident edges, tiny segments, extreme coordinates and nearly-tangent curves get fuzz fixtures. Invalid numerical result returns diagnostic rather than NaNs entering document state.

# Performance

Interactive node/path editing targets incremental local recomputation; boolean heavy work may background-preview, but final command must be deterministic from canonical inputs.

# Differential tests

Reference/alternate engine comparison where practical; algebraic invariants; fuzz minimization; golden SVG/path fixtures.

# Coordinate and finite-value contract

Canonical geometry uses finite `f64`. NaN and ±Infinity are rejected at command/import normalization boundaries. External formats that encode invalid/extreme values produce diagnostics and bounded repair/skip behavior rather than allowing non-finite values into the document.

Document coordinates are unit-agnostic logical units interpreted by document/surface unit metadata; the geometry engine must not bake DPI into vector coordinates. Rasterization/export converts logical geometry to device/pixel coordinates through an explicit transform.

# Tolerance policy

No global epsilon. Define named policies by operation and context, for example:

```
GeometryTolerance {
  absolute_floor,
  relative_scale_factor,
  angular_tolerance,
  flattening_error,
  merge_distance
}
```

Tolerance is derived from operation scale/bounds and clamped to documented floors/ceilings. Interactive preview may use a coarser named quality profile; destructive commit/export uses deterministic final profile. Tests verify scale invariance across tiny/huge equivalent shapes.

# Path normalization

Normalization is explicit and side-effect free. It may canonicalize representation for an algorithm but does not silently rewrite the source path unless a command such as Simplify/Clean Path commits that change. Algorithm-local normalization may:

- remove computationally zero-length edges under operation tolerance;
- normalize winding for overlay internals;
- split intersections;
- flatten curves for engines that require polylines.

The original semantic curve data remains intact for live operations.

# Node semantics

Editable path nodes maintain semantic node IDs local to the path where useful for selection/history/provenance. Node type (`cusp`, `smooth`, `symmetric`) is editing metadata describing handle constraints, not a replacement for the actual control-point coordinates. Converting node type is a command that deterministically updates handles according to documented rule.

# Boolean result contract

Boolean operations define:

- accepted input object kinds and conversion rules;
- fill-rule interpretation;
- open-path handling (reject or documented stroke/closure semantics per operation);
- output winding/fill rule;
- contour ordering that is deterministic for identical inputs;
- preservation of source operands for Live Boolean;
- provenance map where available for appearance transfer/debugging;
- diagnostic when geometry is numerically ambiguous rather than emitting corrupt contours.

Live Boolean evaluation must never mutate operands. `Bake/Expand` snapshots evaluated final geometry into ordinary path objects in one undoable transaction.

# Appearance transfer for destructive geometry

When a destructive geometry command combines objects with incompatible appearance, the command's Action/Parameter schema must expose the policy rather than guess. V1 default should preserve the appearance of the designated primary/top operand where industry convention is clear; commands that cannot preserve a meaningful appearance return an explicit result/policy choice.

# Stroke model details

Canonical stroke stores semantic parameters separately from generated outline. Required V1 fields where supported:

- width in document units;
- alignment (`center`, `inside`, `outside`) with behavior defined for open paths;
- cap/join/miter limit;
- dash array + phase with normalization rules;
- start/end markers via resource/reference IDs;
- variable-width profile as normalized samples/control representation.

Negative/non-finite widths are invalid. Zero width has explicit hairline/none semantics determined by document model, never backend accident.

# Offset and outline

Offset/Outline Stroke uses a dedicated geometry operation with join/cap/miter/tolerance policy. Self-intersections created by offsets are resolved deterministically. The expanded result records no hidden dependency on iOverlay/Kurbo internals.

# Snapping architecture

Snapping is two-stage:

1. spatial query collects candidates in a world-space search region derived from a screen-space snap radius;
2. candidate evaluator scores exact geometric relation in screen space/context.

Canonical candidate data includes stable source ID/sub-element descriptor, snap kind, world point/axis, screen distance, priority class and visual-hint payload. Scoring tie-breakers are deterministic: explicit priority → screen distance → stable semantic ordering.

Snapping never mutates geometry directly; it returns a constraint/proposed transform consumed by the active tool transaction.

# Hit testing

Hit testing and snapping share spatial primitives but are distinct policies. Hit testing accounts for visible stroke width, handles and screen-space minimum pick radius. Selection cycling for overlapping objects uses stable z/order + previously selected context rather than hash-map iteration order.

# Extreme-coordinate safety

Operations must have explicit guards for coordinate magnitude, multiplication overflow in intermediate calculations, recursion/segment explosion and pathological intersection counts. Heavy geometry work receives a complexity budget/cancellation hook through the job system when offloaded. Reaching a guard returns a structured diagnostic with operation/objects affected.

# Determinism

Given the same canonical inputs, operation parameters, evaluator version and architecture-supported floating-point environment, committed geometry results should be stable enough for save/reopen, undo and regression fixtures. Do not depend on randomized hash iteration for contour/order output. Where cross-platform floating differences are unavoidable, tests use documented geometric tolerance and structural invariants rather than byte equality.

# Fuzz corpus categories

Maintain minimized fixtures for:

- self intersections;
- coincident/overlapping edges;
- touching vertices/edges;
- nearly parallel/tangent curves;
- microscopic segments relative to bounds;
- huge coordinate ranges;
- many nested contours/holes;
- repeated points/degenerate handles;
- open/closed mixed inputs;
- adversarial segment counts.

Each production geometry bug becomes a permanent corpus fixture.