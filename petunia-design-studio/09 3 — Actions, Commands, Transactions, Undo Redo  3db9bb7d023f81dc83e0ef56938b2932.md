# 09.3 — Actions, Commands, Transactions, Undo/Redo & ChangeSets

# Layers of intent

`Action` is semantic user intent and discoverability metadata. `Command` is validated domain mutation. `Transaction` groups previews/continuous edits. `ChangeSet` describes committed consequences.

# Action metadata

Stable ActionId, localized TextId, semantic IconId, category, default shortcuts by context, enable/visible predicates, parameter schema, permission requirements and automation exposure.

# Command contract

Commands validate preconditions against a document revision and produce either `Committed(ChangeSet)` or structured failure. Domain commands never show UI.

# Transactions

Continuous operations use `begin/update/commit/cancel`: drag transform, node edit, gradient, brush stroke, slider scrub. Preview state must not flood undo. Cancel restores exact pre-transaction state.

# Undo architecture

Prefer operation/delta-based reversible commands for semantic edits. Large raster edits may store compressed tile deltas or checkpoints. Snapshot whole-document undo is fallback/testing only.

# History rules

Coalesce compatible edits within explicit policy; never merge across unrelated actions. History entries have localized labels derived from Action/Command metadata. Undo/redo must restore selection-relevant IDs when possible without making selection canonical document state.

# ChangeSet

Contains affected object/resource IDs, mutation categories, invalidation hints, created/deleted IDs, optional bounds hints and document revision. Consumers subscribe to ChangeSets instead of observing raw storage.

# Failure/concurrency

Commands against stale assumptions revalidate rather than corrupt. Background-generated commands must return to the authoritative mutation thread/executor. No renderer/job thread mutates DocumentStore.

# Tests

Property tests for undo(command(state)) == state where applicable, transaction cancel identity, coalescing boundaries, nested transaction rejection/policy, large history memory budgets and serialization/reopen around history-independent document state.

# Authoritative revision model

Every committed document mutation increments monotonic `DocumentRevision(u64)`. Preview updates inside an active interaction do not advance the committed revision until commit. Consumers may cache against revision + finer-grained dependency keys.

Commands carry an execution context containing at least:

- document/session identity;
- originating `ActionId` when user/automation initiated;
- optional `expected_revision` for optimistic concurrency;
- actor/source (`UI`, `Plugin`, `MCP`, `Importer`, `System`);
- permission/capability context;
- cancellation/deadline metadata where applicable;
- correlation/trace ID.

# Single-writer rule

A `DocumentSession` has one authoritative mutation lane. Multiple readers/snapshots/jobs may run concurrently, but all canonical writes return to that lane. Rust implementation may use a dedicated executor/task or strictly-owned application thread; the semantic rule is more important than the mechanism.

No `Mutex<DocumentStore>` is exposed as a general escape hatch. A background worker computes immutable results and submits a validated command/result application back to the mutation lane.

# Command execution phases

Canonical phases:

1. resolve command type/schema;
2. check permission/capability;
3. validate parameters and referenced IDs;
4. check `expected_revision`/preconditions;
5. build mutation plan + inverse/undo data;
6. apply atomically to working document state;
7. run fast invariants;
8. produce `ChangeSet` + history entry;
9. advance revision;
10. notify derived-state/event subscribers after commit.

A failure before phase 8 must leave the last committed state logically unchanged. Panics are bugs, not command errors.

# Structured command result

Use an explicit result family rather than booleans/string errors:

```
CommandResult
├── Committed { revision, change_set, history_entry_id }
├── NoOp { revision, reason }
├── Rejected { code, diagnostics, current_revision }
├── Conflict { expected_revision, current_revision, retry_hint }
└── Cancelled { state_restored: true }
```

`NoOp` is valid for semantically harmless requests such as aligning an already-aligned object; it does not create history noise.

# Transaction model

Public editing transactions are **single-level per document interaction lane** in V1. Nested public transactions are rejected because hidden nesting makes cancel/rollback/history boundaries ambiguous. Internal command helpers may compose mutation primitives inside the owning top-level transaction but may not independently commit.

Transaction state machine:

```
Idle
 → Begun(pre_state, transaction_id)
 → Previewing(update*)
 → Committed(one history entry)
   or Cancelled(exact restoration)
```

If a new incompatible transaction is requested while one is active, the caller must explicitly commit/cancel the first; no implicit auto-commit.

# Preview semantics

Continuous interaction may update the in-memory working state so canvas/render/property feedback stays immediate. Preview updates emit `PreviewChangeSet { transaction_id, sequence }` to session/render/UI consumers, but:

- they are not serialized/autosaved as committed document state;
- they do not create undo entries;
- plugins/MCP observing committed document state must be able to distinguish preview from committed data;
- export/save waits for or explicitly resolves the active transaction rather than capturing an accidental mid-drag state;
- cancel applies the recorded inverse/restore data and emits a final preview-restored notification.

# Undo record classes

Use the cheapest semantically reliable inverse representation:

- scalar/property edit → old typed value;
- structural edit → old parent/index/order/reference metadata;
- path edit → compact geometry delta or prior affected path payload;
- create/delete → serialized semantic subtree/resource delta sufficient for exact ID restoration;
- raster stroke/filter destructive commit → compressed dirty-tile before-data + metadata;
- very large operation → bounded checkpoint/delta hybrid.

Whole-document snapshots are prohibited as normal V1 history entries.

# Undo memory budget and eviction

History has an explicit memory budget configurable by product policy. Default recommendation: dynamic bounded budget with a conservative floor and ceiling (for example minimum 256 MiB, default target around 10% of available memory, hard cap configurable), rather than unbounded growth. Exact shipping values must be benchmarked and recorded in product configuration.

Eviction removes oldest undoable groups while preserving document validity. The UI must expose when history has been truncated. A single operation that exceeds the normal budget may use a temporary checkpoint/spill strategy or require user-visible warning if it cannot be made safely undoable; it must never silently disable undo.

# Raster undo spill policy

Large raster deltas may be compressed and spilled to Aubrieta-managed temporary storage through the platform/cache service. Spill files:

- are session-scoped and namespaced;
- are never treated as canonical document resources;
- are checksummed/versioned;
- are cleaned on normal shutdown and scavenged safely after crash;
- are encrypted only if a later security/privacy ADR requires it, but must never include more source data than needed;
- failure to spill returns a structured operation/history diagnostic before committing an operation whose undo guarantee would be violated.

# History grouping/coalescing

Coalescing is driven by explicit `CoalescingKey` + transaction policy, never by a blind time window alone. Examples:

- typing in one text editing session may coalesce until cursor/selection/action boundary;
- repeated arrow nudges may coalesce while same selection/context remains active;
- a slider scrub is already one transaction;
- commands from different actor/source or different target sets do not merge accidentally.

Each committed history entry stores semantic label inputs (`ActionId`, target count/type metadata), not prelocalized strings, so history can localize at presentation time.

# ChangeSet contract

`ChangeSet` is a committed semantic delta summary, not an alternative mutation payload. Required fields include:

- before/after revision;
- created/deleted/modified typed IDs;
- structural changes (parent/order/surface);
- property/schema keys changed;
- resource/style/symbol/data-source changes;
- coarse mutation categories;
- conservative old/new bounds hints where known;
- evaluation invalidation seeds;
- optional selection-recovery hints;
- source/correlation metadata.

Consumers may over-invalidate safely but must never assume a ChangeSet contains enough data to reconstruct the command or bypass the document API.

# Conflict policy for MCP/plugins

External automation that performs read→decide→write should normally send `expected_revision`. On conflict, the host returns current revision plus stable diagnostics; it does not silently replay semantically dangerous commands against a different selection/document state.

Idempotent or target-ID-complete commands may opt into safe revalidation. Retrying is declared by command metadata; callers must not guess.

# Save/autosave interaction

Canonical save serializes the latest **committed** revision. If an interactive transaction is active, explicit Save requests the interaction owner to commit or cancel first; autosave snapshots the last committed revision without forcing the user's gesture to finish. History itself is session state and is not required for canonical reopen unless a future ADR adds persistent history.

# Failure injection requirements

Tests must inject failures at mutation-plan creation, allocation, raster spill, post-apply invariant check and subscriber notification boundaries. Expected behavior:

- no partially committed canonical state;
- revision only advances after successful commit;
- failed subscriber/derived-cache work cannot roll back or corrupt a successful document commit;
- transaction cancel remains exact after arbitrarily many preview updates;
- stale external revisions produce deterministic conflict results.

# Observability

Every command/transaction emits structured tracing fields: command/action ID, document ID, source, duration by phase, affected-count, revision before/after, undo bytes, ChangeSet categories and result code. Do not log document contents, text, paths or user filenames by default; diagnostics follow 09.19 privacy rules.