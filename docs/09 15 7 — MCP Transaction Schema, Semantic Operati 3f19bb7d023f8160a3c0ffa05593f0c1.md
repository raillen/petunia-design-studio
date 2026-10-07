# 09.15.7 — MCP Transaction Schema, Semantic Operations & UI Equivalence Conformance

# Transaction request

documentSessionId; expectedRevision; operations[]; label/action provenance; dryRun; idempotencyKey optional.

# Operation set

Prefer Action invocation and Property mutations. Generic low-level operations only for stable semantic primitives: create typed object with schema, delete IDs, set properties, reparent/reorder, resource bind. No raw container index mutation without IDs.

# Atomicity

All operations validate against same initial/staged transaction context. Failure returns operation index/path and commits nothing.

# Limits

Maximum operations/payload/created objects per transaction to prevent abuse. Large generation uses Job/service.

# Undo

Successful transaction becomes one history entry unless request explicitly supplies allowed grouping metadata; MCP cannot mark mutation “non-undoable” for convenience.

# UI equivalence

For each core cookbook workflow:

UI Action sequence -> canonical normalized snapshot A.

MCP transaction/helper -> snapshot B.

A and B must be semantically equal except provenance/view state.

# Selection/view

Document transaction does not implicitly change local user selection unless method specifically targets a view and permission allows; automation side effects on UI are explicit.

# Tests

Partial failure, stale revision, duplicate idempotency key, 10k-op limit, undo and UI parity.