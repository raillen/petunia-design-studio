# 09.15.4 — MCP Error Model, Method Envelope, Pagination, Revisions & Idempotency

# Common request context

client/session identity supplied by authenticated transport; optional documentSessionId, expectedRevision, requestId/idempotencyKey, locale, dryRun.

# Common response

ok/result OR error; warnings[]; documentRevision where relevant; nextCursor; JobId; audit correlation ID optional.

# Error taxonomy

INVALID_ARGUMENT, NOT_FOUND, AMBIGUOUS_TARGET, STALE_REVISION, PERMISSION_DENIED, CAPABILITY_UNAVAILABLE, UNSUPPORTED, VALIDATION_FAILED, CONFLICT, BUSY, CANCELLED, RESOURCE_LIMIT, IO_ERROR, EXPORT_DEGRADATION_BLOCKED, INTERNAL.

# Error fields

code, category, message, field/path, objectIds, currentRevision, requiredScope/capability, recoverable, suggestedActionId/helpId. Never require parsing prose.

# Pagination

cursor opaque, stable for underlying query snapshot/revision where feasible. Limit bounded by method. Cursor expiry/error explicit.

# Revision

Every document query that can lead to write returns revision. Writes accept expectedRevision; absent revision allowed only low-risk helper policy. Stale writes never silently apply to guessed objects.

# Idempotency

Methods declare None, SafeRetry, KeyedIdempotent. Keyed mutation stores bounded recent request result per client/session to avoid duplicate action after transport retry.

# Dry run

Returns wouldChange, affected IDs/count, validation errors, permissions, degradation/preflight, estimated job class without canonical mutation.

# Tests

Every method/error code example, stale write, retry after timeout, cursor invalidation and permission escalation.