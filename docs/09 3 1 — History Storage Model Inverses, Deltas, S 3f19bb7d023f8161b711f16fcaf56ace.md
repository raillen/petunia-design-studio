# 09.3.1 — History Storage Model: Inverses, Deltas, Snapshots, COW & Memory Budgets

# Objective

Define exactly how Undo/Redo stores reversible state without exploding RAM or coupling history to UI.

# HistoryEntry

```
HistoryEntry
  transaction_id
  action_id
  label_metadata
  timestamp/session sequence
  command records[]
  memory_cost
  affected_ids[]
  merge_group?
  savepoint relation
```

# Storage strategies

**Scalar/property edit** — before/after typed value.

**Transform** — before/after transforms keyed ObjectId.

**Hierarchy mutation** — retained serialized/subtree object records + parent/index metadata.

**Text edit** — range splice delta: start, removed text/runs, inserted text/runs, style deltas.

**Vector topology** — structural patch containing affected contours/nodes and stable-ID mapping.

**Brush/raster edit** — tile-level COW snapshots or compressed rectangular deltas.

**Large destructive filter** — staged new tile set + retained old tile references under COW.

**Resource relink** — before/after ResourceBinding metadata, not duplicated binary unless replacement owns new resource.

**Bulk generated objects** — retained created IDs/object payload for undo delete; source state for redo recreation.

# Copy-on-write

Tile/object payloads may be shared between canonical revisions and history until mutation. COW block has immutable payload + reference ownership. Mutation clones only touched block.

# Memory budget

HistoryManager maintains configurable soft/hard budget. Entries report approximate resident/compressed cost. Policy:

1. compress cold raster deltas;
2. release recomputable derived payload;
3. spill eligible history payload to temp backing;
4. prune oldest entries only after preserving current savepoint semantics;
5. notify user only when undo depth materially reduced.

# Savepoint

History tracks revision corresponding to explicit save. Dirty = canonical revision differs from saved semantic revision, not merely history pointer index.

# Redo

Any new committed document mutation after undo invalidates redo branch unless future branching history ADR is adopted. View-only selection/navigation never invalidates redo.

# Persistence

V1 history is session-local and not required inside .PTND. Recovery journal may reuse command/delta representations but is a separate durability system.

# Determinism

Redo re-applies stored semantic payload/result, not reruns nondeterministic algorithms unless command stores all seeds/source snapshot necessary.

# Tests

Budget pruning, COW alias isolation, 10k text edits, huge brush sequence, delete/undo nested hierarchy, filter undo under memory pressure, savepoint dirty correctness, crash journal independence.