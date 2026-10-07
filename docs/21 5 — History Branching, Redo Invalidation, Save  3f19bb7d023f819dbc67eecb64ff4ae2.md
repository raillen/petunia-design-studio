# 21.5 — History Branching, Redo Invalidation, Save Points, Recovery & Persistence Policy

# Linear V1 history

V1 exposes linear undo/redo. Undo then new mutation invalidates redo branch. Internal retained branch experimentation is not user-visible unless future ADR introduces history tree.

# Current pointer

History entries indexed by monotonic transaction sequence; current pointer indicates last applied entry. Undo moves backward by inverse payload; redo forward by retained post payload.

# Save marker

Store last explicitly saved document revision/fingerprint separately. Dirty = current semantic revision differs from saved snapshot, including when undo returns exactly saved state if revision identity/content hash policy can detect equivalence.

# Persistence

Full undo history is not stored inside PTND V1. Save/reopen starts new session history. This avoids huge files and execution-version dependencies.

# Recovery

Recovery journal may contain command/delta/checkpoint information but is replayed only by recovery subsystem with versioned journal schema. It is not identical to History UI store.

# Plugin version

History payload never depends on calling old plugin code to undo core mutation. Plugin request resolves into host-owned Commands/deltas before commit.

# Tests

Undo/new edit clears redo, saved state crossing, reopen history empty, plugin disabled after mutation and recovery journal independent.