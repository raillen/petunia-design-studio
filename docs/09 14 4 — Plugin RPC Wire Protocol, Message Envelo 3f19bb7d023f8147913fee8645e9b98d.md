# 09.14.4 — Plugin RPC Wire Protocol, Message Envelope, Versioning & Error Codes

# Envelope

Every RPC message includes protocolVersion, messageType, requestId/correlationId, pluginId, session token/nonce context and payload. No pickle or arbitrary Python object serialization.

# Core messages

HandshakeRequest/Response

RegisterContributions

UnregisterContributions

QueryDocument

QuerySelection

ExecuteAction

Begin/Commit/AbortTransaction

PanelDataRequest/Response

ToolEventBatch

ToolOverlayUpdate

JobStart/Progress/Result/Cancel

PermissionDenied

HealthPing/Pong

Shutdown.

# Versioning

Protocol version negotiated at handshake. Message payloads use schema versions; backward-compatible optional fields default explicitly. Incompatible major version fails before plugin activation.

# Errors

Stable codes: incompatible_version, invalid_manifest, permission_denied, stale_revision, invalid_request, quota_exceeded, timeout, cancelled, unavailable_capability, plugin_internal_error. Human text non-contractual.

# Limits

Max message bytes, nesting, array counts, tool event batch length and response timeouts enforced before deserialization allocation.

# Ordering

Per-request responses correlate by requestId; document mutation requests are serialized through host application service and revisioned. Panel/model updates can be coalesced.

# Binary

Large immutable binary payload uses negotiated shared-memory/file-handle transport with descriptor+size+hash and explicit release; never inline unbounded base64.

# Tests

Malformed length, unknown message, replay nonce, out-of-order response, oversized payload, stale transaction and protocol downgrade.