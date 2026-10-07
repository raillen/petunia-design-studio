# 09.15 — MCP API: Discovery, Semantic Automation, Inspection, Jobs & Safety

# Principle

MCP é product API sobre semantic core, não debug backdoor.

# Surface

Discovery, document query, action/mutation, jobs, import/export, plugin capabilities, preferences with scope, semantic UI inspection and development-only synthetic input.

# Two levels

Task helpers: document.create_rectangle, selection.set_fill, text.create_frame, photo.add_adjustment, export.surface.

Generic: actions.execute, properties.describe/get/set, transactions.execute, object queries, registry discovery.

# IDs

DocumentSessionId, ObjectId, SurfaceId, ResourceId, ActionId, PropertyId. Labels são metadata.

# Context efficiency

summary first, detail on demand. Pagination/filtering everywhere large. Nunca dump full document by default.

# Revisions

Reads return revision. Writes may require expected_revision. Stale context returns current revision + recovery suggestion.

# Transactions

Atomic batches default. Partial mode explicit. One logical task usually one undo transaction.

# Dry-run

High-impact operations offer explain/dry-run: affected IDs/count, required permissions, degradations, conflicts, estimated job class.

# Jobs

Long task returns JobId; get/cancel/events with backpressure. No unbounded polling.

# Semantic UI inspection

window/session/persona/tool/selection, visible panels, focus, semantic controls, viewport transform, semantic hit targets, diagnostics. Screenshot supplemental only.

# Safety scopes

document read/write, external file read/write grants, network adapters, clipboard, plugin admin, settings, developer screenshot/input/diagnostics. Local transport is not omnipotent by default.

# Transport

Local stdio/socket or remote Streamable HTTP adapter behind auth. Transport never changes domain authorization.

# Idempotency

Optional request ID for safe retry. Unknown write failure is not blindly repeated.

# Audit

Mutations record source=mcp, client/session, ActionId, duration/result/affected IDs without logging private artwork/text payload by default.

# Conformance

Same workflow through headless Action API and MCP must yield semantically equal document snapshot.

[09.15.1 — MCP Method Catalog, Schemas, Transactions, Jobs & Cookbook](09%2015%201%20%E2%80%94%20MCP%20Method%20Catalog,%20Schemas,%20Transaction%203f19bb7d023f81fdbc23c80390f11aa8.md)

[09.15.2 — MCP Transport, Authentication, Scopes, File Grants, Privacy & Audit](09%2015%202%20%E2%80%94%20MCP%20Transport,%20Authentication,%20Scopes,%20F%203f19bb7d023f8113ab53cec7bbae563f.md)

[09.15.3 — MCP Semantic UI Inspection, Accessibility Tree, Controlled Input & Test Mode](09%2015%203%20%E2%80%94%20MCP%20Semantic%20UI%20Inspection,%20Accessibilit%203f19bb7d023f819ab106e1f4e70b805e.md)

[09.15.4 — MCP Error Model, Method Envelope, Pagination, Revisions & Idempotency](09%2015%204%20%E2%80%94%20MCP%20Error%20Model,%20Method%20Envelope,%20Pagina%203f19bb7d023f813c959ed4738a3ec975.md)

[09.15.5 — MCP Concrete Core Method Set V1](09%2015%205%20%E2%80%94%20MCP%20Concrete%20Core%20Method%20Set%20V1%203f19bb7d023f81cf8352cd0d8d86012d.md)

[09.15.6 — MCP Object Query Language, Filters, Projection & Context-Efficient Retrieval](09%2015%206%20%E2%80%94%20MCP%20Object%20Query%20Language,%20Filters,%20Proj%203f19bb7d023f817ab635fd5e25c19ef7.md)

[09.15.7 — MCP Transaction Schema, Semantic Operations & UI Equivalence Conformance](09%2015%207%20%E2%80%94%20MCP%20Transaction%20Schema,%20Semantic%20Operati%203f19bb7d023f8160a3c0ffa05593f0c1.md)

[09.15.4 — MCP Normative Method Set v1 & Common Envelopes](09%2015%204%20%E2%80%94%20MCP%20Normative%20Method%20Set%20v1%20&%20Common%20Env%203f19bb7d023f810e9b87da65133793b8.md)

[09.15.5 — MCP JSON Schemas, Object Query Language, Dry-Run Plans & Idempotency](09%2015%205%20%E2%80%94%20MCP%20JSON%20Schemas,%20Object%20Query%20Language,%203f19bb7d023f81d38025df8eea993abb.md)

[09.15.6 — MCP Tool Permissions, Remote Deployment, Audit & Abuse Resistance](09%2015%206%20%E2%80%94%20MCP%20Tool%20Permissions,%20Remote%20Deployment,%203f19bb7d023f81e8a66bee1d319b530f.md)