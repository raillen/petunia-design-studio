# 21.1 — Command Record, Transaction Nesting, Coalescing & Attribution

# History record

Each committed transaction stores TransactionId, ActionId/root intent, source (UI/shortcut/plugin/MCP), localized-label metadata key, affected IDs, revision before/after, timestamp/session sequence and undo payload strategy.

# Nested transactions

Only application services may open nested scopes. Nested child scopes merge into parent unless explicitly marked sub-transaction for diagnostics; user history sees one logical entry.

# Preview

beginPreview creates staging context with original canonical snapshot references. updatePreview replaces staged delta; commit materializes one transaction; cancel discards all.

# Coalescing

Rules are typed by Action/Command family:

- typing: adjacent compatible text edits;
- nudge: repeated same selection/direction within active key gesture;
- numeric scrub: begin/end editor session;
- brush: never coalesce distinct pointer strokes by default;
- property repeated programmatic actions: no implicit coalesce unless caller supplies coalescing key.

# Attribution

Plugin/MCP transaction records source identifier but user-facing History label remains semantic action. Diagnostics can reveal source.

# Invalid nesting

A plugin or UI callback cannot commit another independent transaction during synchronous validation of current mutation. Reentrant write attempts fail with typed error.

# Tests

Nested rollback, preview cancellation, typing boundaries, nudge coalesce, action source, reentrancy rejection and deterministic labels.