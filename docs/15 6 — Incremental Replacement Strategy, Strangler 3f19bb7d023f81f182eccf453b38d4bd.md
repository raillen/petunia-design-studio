# 15.6 — Incremental Replacement Strategy, Strangler Boundaries & No-Dual-Truth Rule

# Strategy

Rebuild through stable semantic boundaries rather than port every module line-by-line.

# No dual truth

At any milestone one subsystem has one canonical owner. Do not keep Python and C++ mutable document copies synchronized.

# Temporary adapters

Legacy readers/export or fixture generators can exist in tooling. Production path cannot branch indefinitely between old/new cores for the same document semantics.

# Vertical slices

Replace enough kernel+UI+renderer to support complete workflows, then expand. Avoid recreating every old panel before save/render loop works.

# Cutover

For each subsystem define:

new authority;

legacy dependency;

compatibility adapter;

parity tests;

cutover revision;

removal task.

# Rollback

Source-control rollback of implementation is allowed; saved file compatibility must remain explicit. Never require user to downgrade document without migration/export path.