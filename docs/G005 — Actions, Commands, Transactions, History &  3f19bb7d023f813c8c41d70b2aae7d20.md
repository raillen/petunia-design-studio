# G005 — Actions, Commands, Transactions, History & ChangeSets

# Goal

Make Action -> Command -> Transaction the only canonical mutation path.

# Depends

G003, G004.

# Primary

editor-engineer + architect. Independent tester/quality-reviewer.

# Skills

editor-tooling, lang-cpp, architecture-quality, state-management, testing-quality.

# Deliverables

ActionId/descriptor baseline; Command type envelope; TransactionBuilder; DocumentMutator private API; ChangeSet; revision counter; HistoryManager; Undo/Redo; linear redo invalidation; transform/property/create/delete initial commands; coalescing token framework; savepoint state.

# Acceptance

No public DocumentStore mutation outside transaction service. Multi-command failure rolls back. Undo/redo restores semantic snapshot exactly. ChangeSet identifies only affected IDs/properties/hierarchy.

# Tests

Atomic rollback, create/delete/transform undo-redo, redo truncation, nested transaction, coalescing, invalid command, stale expected revision and savepoint dirty state.

# Evidence

Normalized snapshots before/after/undo and mutation API architecture review.

# Non-goals

Raster history optimization, recovery journal, plugin/MCP Actions.