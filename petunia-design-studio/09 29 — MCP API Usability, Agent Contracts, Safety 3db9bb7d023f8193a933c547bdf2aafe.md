# 09.29 — MCP API Usability, Agent Contracts, Safety, Discovery & Deterministic Automation

<aside>
🤖

**MCP is a product API, not a debug backdoor.** It must be understandable by people, deterministic for agents, complete enough for real workflows and constrained by the same Commands, permissions and validation as the UI.

</aside>

# Design goals

The MCP surface optimizes for:

- self-discovery;
- stable semantic names;
- small-context operation for LLMs;
- safe defaults;
- deterministic mutations;
- explicit transactions/revisions;
- structured errors;
- capability-based permission;
- inspection without pixel-only automation.

# Two-level API

Expose both:

1. **Task-oriented helpers** for common workflows;
2. **Generic semantic primitives** for full coverage.

Examples of helpers:

- `document.create_rectangle`;
- `selection.set_fill`;
- `text.create_frame`;
- `photo.add_adjustment`;
- `data_merge.preview_record`;
- `export.surface`.

Generic layer may expose `actions.execute`, property schema/query/edit, transaction batches and registry discovery.

Helpers must delegate to canonical Actions/Commands.

# Naming policy

Names are stable, explicit and searchable. Prefer `document.create_rectangle` over overloaded `create`. Avoid toolkit/UI words unless the API is specifically inspecting UI.

Every method has:

- stable method ID;
- one-sentence intent;
- input schema;
- output schema;
- permission/capability requirements;
- side effects;
- undo/transaction semantics;
- revision behavior;
- examples;
- error codes;
- related methods/help topics.

# Discovery

Minimum discovery methods/concepts:

- application/API version;
- available capabilities;
- loaded document sessions;
- action registry;
- property/schema registry;
- supported import/export formats;
- plugin-provided contributions;
- permission policy status;
- optional methods by version.

Discovery responses should be filterable/paginated to avoid flooding LLM context.

# Context-efficient schemas

Return summaries first, detail on demand. Example:

```
document.summary -> counts, surfaces, selection, revision, issues
object.list -> compact IDs/types/names
object.get -> requested object detail
property.describe -> schema only when needed
```

Do not dump full document JSON unless explicitly requested and safe.

# Stable IDs

Agents operate on ObjectId/SurfaceId/ResourceId/ActionId/PropertyId. Display labels are metadata. Responses state document/session identity and revision so stale plans can be detected.

# Revision-safe mutation

Mutation requests may include `expected_revision`. On mismatch return structured stale-context error with current revision and safe next step; do not silently apply an operation to unknown newer state.

# Atomic batches

Default mutation batch is all-or-nothing. Partial modes must be explicit and return per-item outcomes. One logical automation task should normally create one coherent undo/history transaction.

# Dry-run / explain

High-impact operations should support an `explain` or dry-run path when meaningful. It may return:

- affected IDs/counts;
- export degradations;
- required permissions;
- unsupported capabilities;
- estimated class of long-running work;
- warnings/preflight.

Dry-run cannot mutate canonical document state.

# Long jobs

Return `JobId` for expensive work. Standard operations:

- `jobs.get`;
- `jobs.cancel`;
- optional subscription/event stream;
- structured phase/progress/result/diagnostic.

Agents must not poll at unbounded frequency.

# Inspection API

Expose semantic state:

- application/window/document sessions;
- active Persona/tool;
- selection summary;
- panel visibility/focus;
- accessibility/semantic tree;
- actionable control IDs;
- viewport transform;
- semantic hit targets;
- diagnostics/tasks.

Pixel screenshots are supplemental evidence, never the only automation model.

# Controlled input

Synthetic pointer/key input is test/developer capability, not the preferred production automation path. It must require explicit development/test permission and cannot bypass Action/Command security.

# Error model for humans and agents

Every error includes:

- `code`;
- category;
- short localized/user-facing TextId metadata where applicable;
- plain technical explanation in API language;
- field/path that failed validation;
- required capability/permission;
- recoverable boolean;
- suggested next method/action when deterministic;
- docs/help topic ID.

Avoid ambiguous messages such as `invalid request` without field-level details.

# Safety model

MCP transport/auth is adapter-specific, but method authorization is semantic. Separate permission scopes for:

- read document;
- write document;
- export/write external file;
- open/read external files;
- network-backed adapters;
- clipboard;
- plugin administration;
- settings/workspace mutation;
- developer input/screenshot/diagnostics.

A local connection is not automatically omnipotent.

# Filesystem mediation

Methods should prefer explicit user-selected/granted destinations or broker handles over arbitrary absolute paths. File grants can carry scope and lifetime. Export cannot overwrite unrelated files without normal conflict policy.

# Privacy

Default inspection responses avoid document pixel/content payloads unless necessary. Diagnostic traces record IDs/timings/status, not arbitrary private artwork/text. Sensitive resource content requires explicit method + scope.

# Idempotency and retries

Where a mutation can be safely retried, support optional idempotency/request IDs. Otherwise clearly document non-idempotent behavior. Agents must not blindly retry unknown write failures.

# Pagination and streaming

Large lists use cursor pagination. Binary/large resource data uses bounded dedicated transfer mechanisms, not giant JSON strings. Event subscriptions have backpressure/coalescing.

# MCP documentation style

For every method publish:

- **What it does**;
- **When to use it**;
- **Simple example**;
- **Advanced example**;
- **Permissions**;
- **Undo/transaction behavior**;
- **Errors and recovery**;
- **Related methods**.

Use copyable examples and realistic workflows rather than schema dumps alone.

# Agent cookbook

Maintain tested recipes:

1. inspect current document;
2. create and style vector artwork;
3. select/query/edit objects by semantic properties;
4. nondestructive Photo edit;
5. batch rename/rearrange layers;
6. Data Merge setup/preview/generation;
7. preflight and export;
8. diagnose missing resources/fonts;
9. discover plugin-provided capabilities;
10. run a UI/accessibility gauntlet through inspection APIs.

# Conformance

MCP and UI must produce equivalent canonical outcomes when invoking the same action/command. Maintain fixtures that execute a workflow through headless Command API and MCP and compare semantic document snapshots.

# Code-agent rule

An agent implementing a new Action/Property that should be automatable must either expose it through generic MCP discovery automatically or document why it is intentionally hidden. Hand-written parallel MCP business logic is forbidden.