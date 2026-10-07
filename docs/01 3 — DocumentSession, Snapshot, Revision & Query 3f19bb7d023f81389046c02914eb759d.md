# 01.3 — DocumentSession, Snapshot, Revision & Query Consistency Model

# Session

DocumentSession owns mutable canonical document, history, revision, jobs/resources/watchers and access policy.

# Revision

Every committed canonical transaction increments monotonically increasing DocumentRevision. View-only operations do not.

# Snapshot

DocumentSnapshot is immutable coherent read view at a revision. Background jobs/export/render/query can retain it without holding write lock.

# Queries

QueryService reads a snapshot/revision and returns stable IDs/value DTOs. Large queries paginate against one snapshot token where consistency matters.

# Writes

Command validation occurs against current revision/expected preconditions. Plugin/MCP can provide expectedRevision; UI tool gesture stores start revision/dependency fingerprint.

# Stale work

Background derived results tagged input revision/dependency generation. Publication discards/rebases only under explicit subsystem rules.

# Multi-view

Multiple views share session revision but have independent Selection/Viewport/Persona. A commit notifies all views through ChangeSet.

# Thread safety

Canonical mutation serialized through session transaction gate. Read snapshots immutable. No arbitrary concurrent object mutation.

# Tests

Concurrent query during write, stale job, two views, revision mismatch and snapshot lifetime after subsequent edits.