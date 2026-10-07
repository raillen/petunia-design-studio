# 21.2 — Undo Payload Strategies by Domain: Vector, Text, Raster, Resources & Large Operations

# Strategy classes

**ValueDelta** — small scalar/property before/after.

**StructuralDelta** — hierarchy insert/remove/reorder with retained object payload.

**TextDelta** — range splice + style-run delta.

**TileCOWDelta** — old/new immutable tile handles.

**ResourceDelta** — resource index/link/embed changes.

**SnapshotRef** — immutable subsystem snapshot for huge operation when cheaper.

**RecomputeInverse** only if mathematically exact/deterministic and input state retained.

# Vector

Transforms/properties use ValueDelta. Path topology stores changed contours/nodes/provenance; do not snapshot whole document for one node edit.

# Text

Store replaced range, inserted text and affected style spans. Large paste can retain rope/piece snapshot references.

# Raster

First-write per tile captures old blob; post tile retained for redo. History may compress cold blobs.

# Delete/group

Structural delta retains removed subtree + parent/index and resources that would otherwise become unreachable.

# Import

Atomic import transaction may retain inserted ObjectIds/resources and optional pre-import selection; undo removes all imported canonical artifacts.

# Huge filters

If 80% of image changes, COW still works but storage cost estimated. History strategy can spill blobs to temp history store.

# Non-undoable

Only administrative operations like clearing history itself. User document mutations should not silently bypass undo because operation is expensive.

# Tests

Undo/redo semantic equality, resource lifetime, massive subtree delete, huge image filter and memory-spill restoration.