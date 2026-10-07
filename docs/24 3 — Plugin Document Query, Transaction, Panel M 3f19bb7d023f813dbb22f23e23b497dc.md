# 24.3 — Plugin Document Query, Transaction, Panel Model & Tool Event Messages

# Query

DocumentSummaryRequest/Response, ObjectQuery with filters/pagination, ObjectSnapshotRequest, PropertyGet. Responses are immutable snapshots with revision.

# Mutation

ExecuteActionRequest preferred. Advanced TransactionRequest contains validated semantic operations or Action calls, expectedRevision and atomic flag. Plugin never submits raw C++ memory mutation.

# Panel model

PanelModelRequest{panelId, viewState, cursor/page}. PanelModelResponse contains declarative rows/sections/controls. Updates use diff/revision so high-volume panel data avoids full rebuild.

# Tool events

ToolEventBatch contains normalized pointer events, current viewport/selection snapshot references and event sequence. Host rate-limits/coalesces. Plugin returns OverlayDescriptor and optionally staged Action intent—not raw GPU commands.

# Binary/resources

Large image/resource data is accessed through brokered handle/stream with size/type permission, not inline base64 giant messages.

# Transaction result

newRevision, affectedIds, ChangeSet summary, warnings and history label metadata.

# Tests

Stale revision, paginated query, atomic rollback, panel diff order, tool event overflow/backpressure and plugin timeout.