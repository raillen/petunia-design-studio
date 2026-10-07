# 09.3.2 — Gesture Transactions, Coalescing, Nested Commands, Async Commit & Cancellation

# Gesture lifecycle

```
begin_gesture()
  -> preview state
  -> zero or many update_preview()
  -> commit(transaction)
or
  -> cancel()
```

Preview is not canonical history. Canonical mutation occurs once at commit except tools explicitly designed for incremental durable edits.

# Coalescing classes

- continuous pointer gesture;
- numeric scrub;
- repeated keyboard nudge;
- typing within same text context;
- repeated brush samples inside one stroke;
- slider adjustment session.

# Merge key

Coalescing requires compatible ActionId + target set + property/operation + explicit merge token. Time alone never merges unrelated actions.

# Nested commands

Transaction may contain subcommands but history presents one logical entry by default. Nested transaction failure rolls back all child changes.

# Async operation

Long job computes against immutable snapshot. Completion returns StagedResult with base_revision. Before commit:

1. validate session alive;
2. validate required objects/resources;
3. compare revision/conflict policy;
4. either commit short transaction, rebase if formally supported, or reject stale result.

# Cancellation

Cancel before commit drops staged state/resources. Cancel during native job propagates stop_token. Once atomic commit begins, either complete or rollback; UI must not expose half-cancelled state.

# Text typing

IME composition remains preview/staged until commit event. Ordinary consecutive text insertions can coalesce until caret/selection/context changes, command type changes, explicit navigation occurs or timeout policy closes group.

# Nudge

Auto-repeat key events share gesture token from first keydown until release/idle. Undo restores original transform once.

# Parameter sliders

Mouse down begins edit session; value changes preview; mouse up commits one Command. Keyboard arrow edits may coalesce by focus session.

# Tests

Lost pointer capture, Esc, window deactivation, plugin/MCP async conflict, stale filter job, nested failure, redo after coalesced edits, composition cancellation.