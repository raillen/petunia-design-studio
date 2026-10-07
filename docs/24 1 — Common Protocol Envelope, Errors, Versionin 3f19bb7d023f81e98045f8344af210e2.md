# 24.1 — Common Protocol Envelope, Errors, Versioning, IDs & Limits

# Envelope

Every request carries protocolVersion, requestId, client/plugin identity/session, method/type, payload and optional expectedRevision/correlationId. Every response echoes requestId and returns result or structured error.

# Versioning

Major protocol mismatch fails handshake. Minor additive fields are ignorable according schema. Optional capabilities are negotiated explicitly.

# Error

code, category, messageKey/messageParams, field/path, recoverable, retryable, requiredPermission/capability, currentRevision where relevant and diagnostics correlationId. Raw internal exception text is not public contract.

# IDs

Opaque stable strings with type-specific fields in schema: ObjectId, SurfaceId, ResourceId, JobId, PanelId, ActionId, PropertyId. Never encode pointer/index assumptions.

# Size limits

Per-message byte limit, nesting, string/array length, binary-handle count and total outstanding requests. Oversize data uses paging/stream/job/shared resource protocol.

# Cancellation

Long request returns JobId or supports cancellation token/request. Cancel result reports whether operation was cancelled before commit or already atomically completed.

# Idempotency

Mutation can include idempotency/request key when safe retry is needed. Server stores bounded result key window scoped to client/session.

# Tests

Malformed schema, unknown fields/versions, oversized payload, duplicate request ID, cancellation race and error stability.