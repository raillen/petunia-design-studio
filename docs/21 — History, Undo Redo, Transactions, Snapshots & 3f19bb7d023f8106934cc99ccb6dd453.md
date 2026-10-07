# 21 — History, Undo/Redo, Transactions, Snapshots & Recovery Semantics

# Goal

Make history a first-class architecture with explicit memory/storage behavior for vector, text, raster and asynchronous operations.

# Principles

One logical user intent = one history transaction.

Undo is deterministic and independent from current UI state.

Preview/staging never pollutes history.

Raster and huge operations use COW/delta strategies.

Session history and crash recovery are different systems.

# Coverage

transaction nesting, coalescing, inverse/delta strategies, memory budgets, redo branches, save points, async commit, plugin/MCP attribution, history labels, persistence policy and tests.

[21.1 — Command Record, Transaction Nesting, Coalescing & Attribution](21%201%20%E2%80%94%20Command%20Record,%20Transaction%20Nesting,%20Coales%203f19bb7d023f816a9395cd574729c9aa.md)

[21.2 — Undo Payload Strategies by Domain: Vector, Text, Raster, Resources & Large Operations](21%202%20%E2%80%94%20Undo%20Payload%20Strategies%20by%20Domain%20Vector,%20T%203f19bb7d023f8182be3ad3d78f0d34b4.md)

[21.3 — History Memory Budget, Compression, Spill-to-Disk, Pruning & User UX](21%203%20%E2%80%94%20History%20Memory%20Budget,%20Compression,%20Spill-t%203f19bb7d023f81aba7f6d1023c214403.md)

[21.4 — Async Jobs, Atomic Commit, Cancellation & Undo for Expensive Operations](21%204%20%E2%80%94%20Async%20Jobs,%20Atomic%20Commit,%20Cancellation%20&%20U%203f19bb7d023f81458ed8eacf4abf154a.md)

[21.5 — History Branching, Redo Invalidation, Save Points, Recovery & Persistence Policy](21%205%20%E2%80%94%20History%20Branching,%20Redo%20Invalidation,%20Save%20%203f19bb7d023f819dbc67eecb64ff4ae2.md)