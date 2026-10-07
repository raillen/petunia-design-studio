# 09.15.1 — MCP Method Catalog, Schemas, Transactions, Jobs & Cookbook

# Namespace families

**app.** version, capabilities, sessions.

**document.** summary, metadata, save, validate, preflight.

**object.** list/get/query.

**selection.** get/set/query-compatible actions.

**actions.** list/describe/execute.

**properties.** describe/get/set.

**transactions.** dry_run/execute.

**tools.** describe active tool/state; activation when allowed.

**import. / export.** formats, analyze, start job.

**jobs.** get/cancel/list.

**plugins.** contributions/status (admin actions separate scope).

**ui.** semantic window/panel/focus tree.

**diagnostics.** bounded structured diagnostics.

**resources.** metadata/grants, never arbitrary raw bytes by default.

# Common envelope

Responses identify API version, session/document/revision where relevant, result or typed error, warnings and next cursor/job.

# Action execute

Input ActionId + typed args + optional session/selection override + expected_revision + dry_run/request_id. Output transaction/result summary + new revision + affected IDs.

# Property set

Uses PropertyId and target IDs; validates schema and multi-selection policy. One atomic transaction by default.

# Query

Object queries filter type/name/tag/property with bounded expressions and pagination; no arbitrary code query.

# Export

export.analyze returns capability/degradation plan. export.start takes resolved preset/targets/file grant and returns JobId. Job result returns ExportReport.

# Job events

Optional subscription stream coalesces progress. Progress fields phase/current/total/message metadata; rate limited.

# Cookbook minimal flows

1. app.sessions -> document.summary;
2. document.create_rectangle helper -> selection.set_fill;
3. object.query(type=text) -> properties.set;
4. photo.add_adjustment -> properties.set curves;
5. export.analyze -> export.start -> jobs.get;
6. ui.inspect -> action execute via semantic control ID.

# Schema generation

MCP JSON schemas originate same Action/Property metadata where feasible to prevent drift.