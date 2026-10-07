# 01.3 — Snapshot, Query & Presentation Projection Contracts

# Snapshot principle

Worker/UI/plugin/MCP reads immutable snapshots or bounded query DTOs; no shared mutable document containers.

# DocumentSnapshot

Captures semantic revision and immutable references/COW structures sufficient for evaluation/save/export/query. Snapshot creation should be cheap relative to full deep copy.

# QueryService

get object summary/detail; hierarchy page; property values; bounds; resources; text metadata. Supports batch/pagination/projection and permission filtering.

# Presentation projections

LayerTreeSnapshot, HistorySnapshot, ResourceListSnapshot, ActionAvailabilitySnapshot, PropertyValueSnapshot, JobSnapshot. These are shaped for UI scale but still semantic/toolkit-neutral.

# ChangeSet application

Qt models may patch existing adapter state from ChangeSet; if diff too complex/stale, request new snapshot. Never derive canonical truth by reversing UI changes.

# Staleness

Every snapshot has revision/generation. UI can display slightly stale derived thumbnail but mutation requires current context/validation.

# Large data

Pixel buffers and huge object lists are not embedded in general snapshots. Use handles/paged queries/buffer contracts.

# Tests

Concurrent read during mutation, stale query, batch 100k IDs, snapshot lifetime after object deletion and no mutable alias leak.