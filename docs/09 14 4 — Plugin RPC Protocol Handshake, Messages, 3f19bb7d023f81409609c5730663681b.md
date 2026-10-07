# 09.14.4 — Plugin RPC Protocol: Handshake, Messages, Framing, Errors & Versioning

# Transport

Local authenticated IPC chosen per platform (Unix domain socket/named pipe/loopback fallback only by ADR). Framing protocol versioned independently from Plugin SDK.

# Envelope

```
protocolVersion
messageId
pluginId
sessionId
type
requestId/replyTo?
deadline?
payload
```

No pickle, object pointers or Python-specific serialization.

# Core message types

HandshakeRequest/Response; RegisterContributions; Unregister; ActionInvokeRequest/Result; QueryRequest/Result; TransactionRequest/Result; PanelModelRequest/Result; PanelEvent; ToolEventBatch; OverlayUpdate; JobStart/Progress/Result/Cancel; FileGrantRequest/Result; NetworkRequest/Result; Diagnostic; Ping/Pong; Shutdown.

# Handshake

Host sends app version, SDK versions, protocol versions, granted capabilities, resource limits and locale/theme metadata safe to expose. Plugin responds package/plugin version, supported protocol/SDK and desired contributions. Incompatible handshake stops before activation.

# Framing

Length-prefixed bounded frames or equivalent library contract. Maximum message size enforced before allocation. Large binary data uses negotiated broker/shared resource handle.

# Request semantics

Every request has timeout/deadline class. Mutation request carries expected_revision and idempotency/request token where retry possible.

# Errors

Stable code, category, recoverable, message parameters, field/path, required capability and optional retryAfter. Internal traceback only in developer diagnostics, not wire contract by default.

# Backpressure

Per-plugin inflight requests/message queue bounded. UI event streams coalesce pointer/model updates; plugin cannot force unbounded main-thread callbacks.

# Versioning

Protocol additive fields ignored only when declared optional. Breaking message change increments protocol major. SDK semantic version and protocol transport version remain distinct.

# Tests

Fragmented frames, oversized messages, unknown type, duplicate requestId, timeout, process death mid-request, stale revision and protocol downgrade refusal.