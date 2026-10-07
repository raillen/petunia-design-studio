# 09.3 — Actions, Commands, Transactions, Undo/Redo & ChangeSets

# Definitions

**Action** = user/application intent with availability, title/help, shortcut and typed parameter schema.

**Command** = validated semantic mutation payload.

**Transaction** = atomic ordered command group.

**ChangeSet** = resulting IDs/properties/structure invalidated.

# Flow

ActionContext snapshots selection/document/tool state -> action resolves Command(s) -> validator -> DocumentMutator -> transaction -> ChangeSet -> history + derived invalidation.

# Undo

Commands either provide inverse state/delta or transaction captures minimal before-state. Undo/redo must be deterministic and not depend on current UI widgets.

# Continuous gesture

Pointer drag starts preview transaction context. Intermediate preview updates derived/staged state; history records one logical commit at pointer up. Cancel discards preview.

# Coalescing

Numeric scrub, keyboard nudge repetition and text typing can coalesce under explicit policy/time/context. Coalescing never merges semantically separate actions accidentally.

# Expected revision

Automation/plugins can require document revision. Stale revision rejects before mutation.

# Failure

Any command failure before commit rolls back entire transaction. No partial state leaks. Long native operations stage external result then short commit transaction.

# History

History is session state; optional persistence policy separate from canonical document. Labels come from ActionId metadata, not arbitrary strings.

[09.3.1 — History Storage Model: Inverses, Deltas, Snapshots, COW & Memory Budgets](09%203%201%20%E2%80%94%20History%20Storage%20Model%20Inverses,%20Deltas,%20S%203f19bb7d023f8161b711f16fcaf56ace.md)

[09.3.2 — Gesture Transactions, Coalescing, Nested Commands, Async Commit & Cancellation](09%203%202%20%E2%80%94%20Gesture%20Transactions,%20Coalescing,%20Nested%20%203f19bb7d023f81be90b8e9061cfd6d05.md)

[09.3.3 — Command Schema, ChangeSet Semantics, Conflict Rules & Replay Guarantees](09%203%203%20%E2%80%94%20Command%20Schema,%20ChangeSet%20Semantics,%20Conf%203f19bb7d023f8125aeb0e01bb60939d5.md)

[09.3.1 — Command Envelope, Preconditions, Validation & Result Schema](09%203%201%20%E2%80%94%20Command%20Envelope,%20Preconditions,%20Validati%203f19bb7d023f8110b36cd74a51307490.md)

[09.3.2 — Built-in Command Catalog v1 by Domain](09%203%202%20%E2%80%94%20Built-in%20Command%20Catalog%20v1%20by%20Domain%203f19bb7d023f818e9236d723435c0da5.md)

[09.3.3 — Action Descriptor Schema, Availability, Shortcuts & Command Mapping](09%203%203%20%E2%80%94%20Action%20Descriptor%20Schema,%20Availability,%20S%203f19bb7d023f8115b5c6fca13877638c.md)