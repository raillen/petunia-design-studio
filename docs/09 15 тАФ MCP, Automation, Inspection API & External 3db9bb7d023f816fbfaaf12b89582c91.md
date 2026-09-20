# 09.15 — MCP, Automation, Inspection API & External Control Contracts

# One automation model

MCP is an adapter over the same semantic registries as UI/plugins. It must not bypass validation, permission or Command boundaries.

# API layers

1. **Discovery:** capabilities, actions, schemas, supported formats/tools;
2. **Document query:** summaries, object/resource lookup, selection-independent semantic snapshots;
3. **Mutation:** typed Action/Command requests with atomic transactions;
4. **Long tasks:** JobId + progress/cancel/result;
5. **Inspection:** UI semantic tree, focus, panels, active tool, viewport transforms, diagnostics;
6. **Testing:** controlled synthetic input/screenshot hooks in development/test builds.

# Stable identifiers

Use namespaced IDs and structured schemas, not UI labels. Object/Surface/Resource IDs are serialized safely. Text and icons never become command identity.

# Transactions

Automation can begin a scoped transaction or submit one atomic batch. Failure policy is explicit: all-or-nothing by default for mutations; partial batch only via API designed for it.

# Revision control

Read responses include document revision. Mutation can optionally require expected revision to avoid acting on stale context.

# Security

MCP transport/authentication is outside domain crates. Local server is not implicitly trusted to expose arbitrary filesystem/network. Sensitive operations require configured policy/consent.

# Observability

Every automation mutation records source=`mcp`, action ID, duration, affected IDs and result in diagnostic trace without logging private document contents by default.

# Agent-friendly inspection

Expose semantic hit targets and control IDs through accessibility/inspection layer, not pixel coordinates alone. Screenshots supplement semantics.

# Compatibility

Version protocol schemas independently; discovery advertises versions and optional methods. Deprecations have explicit window.

# Tests

Replay deterministic automation scripts, stale revision, cancel long export, malformed parameters, permissions, transaction rollback, UI inspection under alternate locale/theme/density.

# Canonical API usability contract

This page defines the automation architecture; **09.29 — MCP API Usability, Agent Contracts, Safety, Discovery & Deterministic Automation** is the canonical contract for method ergonomics, task-oriented helpers, context-efficient discovery, dry-run/explain behavior, structured recovery, idempotency, filesystem mediation and agent cookbooks.

A method is not complete merely because its schema exists. Stable MCP surfaces require human-readable documentation, minimal and advanced examples, permission/side-effect notes, transaction/revision semantics and deterministic error recovery.

# Parity gate

No MCP method may become a privileged shortcut around the domain. For any mutation exposed both in UI and MCP, conformance tests compare the resulting semantic document state. If MCP needs behavior unavailable through Actions/Commands/typed application ports, improve the shared semantic contract rather than adding MCP-only business logic.