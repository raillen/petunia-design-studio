# 09.14.2 — Plugin Host Process, RPC, Sandbox, Quotas, Crash Isolation & Permissions

# Process topology

Main app spawns one host per plugin or pooled host by trust policy; strongest isolation prefers per-plugin process. Host starts minimal environment and receives ephemeral IPC endpoint + capability token.

# IPC

Versioned framed protocol over local pipe/socket. Messages: handshake, register contributions, query snapshot, action request/result, panel model updates, tool events, job/progress, permission request/denial, shutdown.

# Serialization

Schema-generated JSON/MessagePack/Protobuf candidate chosen by ADR. No pickle. Payload sizes bounded. Binary transfers use shared memory/temp broker handle only under explicit protocol.

# Authentication

Child endpoint tied to launch nonce/token and OS user. Do not accept arbitrary local client as plugin process.

# Files

Plugin sees broker API, not main app filesystem. User file picker grants explicit handle; scoped storage maps to dedicated plugin directory; path traversal normalized/rejected.

# Network

No inherited unrestricted host network via SDK. OS sandbox should block direct network where feasible; broker applies HTTPS allowlist, redirect policy, timeout, byte cap and credential isolation.

# Process execution

Denied by default. No inherited shell/process API. A future permission requires separate threat model and user-facing rationale.

# Environment

Strip secrets/tokens, minimize environment, define working directory. Plugin cannot read MCP credentials, app crash bundles or other plugin storage.

# Quotas

Wall/CPU per call, memory, message rate/size, concurrent jobs, file handles, network count/bytes, storage and document mutation batch size.

# Failure

Protocol violation, crash or timeout -> terminate host; unregister contributions; abort open plugin transactions; display health notification; document remains valid. Restart is explicit/controlled.

# Update

Stop old host, verify package/signature/hash, compare permissions, prompt escalation, migrate plugin-scoped storage if plugin declares migration, start new version.

# Audit

Record plugin_id/version, contribution/action IDs, permission decisions, duration/error without private payload by default.