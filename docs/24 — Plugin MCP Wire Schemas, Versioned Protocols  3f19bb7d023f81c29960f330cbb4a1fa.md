# 24 — Plugin/MCP Wire Schemas, Versioned Protocols & Generated Contracts

# Purpose

Fechar a lacuna entre arquitetura semântica e protocolo real.

# Common requirements

Versioned envelope; requestId; protocolVersion; client/plugin identity; bounded payload size; typed error; cancellation; capability/permission context; correlationId.

# Plugin message families

Hello/Handshake, RegisterContributions, QuerySnapshot, ExecuteAction, Begin/CommitTransaction, PanelModelRequest/Update, ToolEventBatch, JobStart/Progress/Complete, FileGrantRequest, NetworkRequest, PermissionDenied, Diagnostic, Shutdown.

# MCP method families

app.*, sessions.*, document.*, object.*, selection.*, actions.*, properties.*, transactions.*, import.*, export.*, jobs.*, plugins.*, ui.*, resources.*, diagnostics.*.

# Serialization

No pickle. Canonical JSON Schema is baseline for MCP; plugin IPC may use MessagePack/CBOR/Protobuf only through ADR if schema generation/debuggability remain excellent.

# Generated code

Schemas generate Python types/stubs and validation DTOs where practical. Wire schema and semantic Action/Property schemas share references to avoid drift.

# Compatibility

Major breaks handshake; minor additive fields ignored/preserved per schema; capability negotiation selects optional methods/features.

[24.1 — Common Protocol Envelope, Errors, Versioning, IDs & Limits](24%201%20%E2%80%94%20Common%20Protocol%20Envelope,%20Errors,%20Versionin%203f19bb7d023f81e98045f8344af210e2.md)

[24.2 — Plugin IPC Handshake, Registration & Contribution Protocol](24%202%20%E2%80%94%20Plugin%20IPC%20Handshake,%20Registration%20&%20Contri%203f19bb7d023f8144b59ae3b81d1584da.md)

[24.3 — Plugin Document Query, Transaction, Panel Model & Tool Event Messages](24%203%20%E2%80%94%20Plugin%20Document%20Query,%20Transaction,%20Panel%20M%203f19bb7d023f813dbb22f23e23b497dc.md)

[24.4 — MCP Concrete Method Catalog & Schema Contract](24%204%20%E2%80%94%20MCP%20Concrete%20Method%20Catalog%20&%20Schema%20Contra%203f19bb7d023f817b8b9fe27ee6ba2f52.md)

[24.5 — MCP Authentication, Scope Matrix, Grants, Audit & Remote Deployment](24%205%20%E2%80%94%20MCP%20Authentication,%20Scope%20Matrix,%20Grants,%20A%203f19bb7d023f8129b158e96b02c8b46d.md)

[24.6 — Generated Schemas, SDK Stubs, Compatibility Tests & Protocol Fuzzing](24%206%20%E2%80%94%20Generated%20Schemas,%20SDK%20Stubs,%20Compatibility%203f19bb7d023f81998e4fef6d51a440a1.md)